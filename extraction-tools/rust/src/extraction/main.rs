use anyhow::{Context, Result, bail};
use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "extract_glsmac")]
#[command(about = "Extract SMAC data from GLSMAC project")]
struct Args {
    /// Path to GLSMAC repository
    glsmac_path: PathBuf,
    
    /// Output JSON file path
    #[arg(short, long, default_value = "glsmac_data.json")]
    output: PathBuf,
    
    /// Generate Rust modules instead of JSON
    #[arg(short, long)]
    rust_modules: bool,
    
    /// Convert to Godot format
    #[arg(short, long)]
    godot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    pub key: String,
    pub name: String,
    pub filename: String,
    pub is_naval: bool,
    pub is_progenitor: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unit {
    pub key: String,
    pub name: String,
    pub sprite_x: u32,
    pub sprite_y: u32,
    pub movement_per_turn: u32,
    pub unit_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationType {
    pub key: String,
    pub name: String,
    pub category: String,
    pub is_tile_worker: bool,
    pub is_specialist: bool,
    pub is_drone: bool,
    pub human_sprite_x: Option<u32>,
    pub human_sprite_y: Option<u32>,
    pub progenitor_sprite_x: Option<u32>,
    pub progenitor_sprite_y: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractedData {
    pub factions: Vec<Faction>,
    pub units: Vec<Unit>,
    pub population_types: Vec<PopulationType>,
    pub technologies: Vec<Technology>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Technology {
    pub key: String,
    pub name: String,
    // Placeholder for now
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GodotData {
    pub faction_definitions: HashMap<String, Faction>,
    pub unit_definitions: HashMap<String, Unit>,
    pub population_definitions: HashMap<String, PopulationType>,
    pub technology_tree: HashMap<String, Technology>,
}

struct GLSParser {
    include_cache: HashMap<String, String>,
}

impl GLSParser {
    fn new() -> Self {
        Self {
            include_cache: HashMap::new(),
        }
    }
    
    fn parse_file(&mut self, path: &Path) -> Result<String> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        
        // Process includes
        let include_re = Regex::new(r#"#include\(\s*'([^']+)'\s*\)"#)?;
        let mut processed = content.clone();
        
        for cap in include_re.captures_iter(&content) {
            let include_path = &cap[1];
            let full_path = path.parent().unwrap().join(format!("{}.gls.js", include_path));
            
            if let Ok(included_content) = fs::read_to_string(&full_path) {
                processed = processed.replace(&cap[0], &included_content);
            }
        }
        
        // Process uppercase directive
        let uppercase_re = Regex::new(r"#uppercase\(([^)]+)\)")?;
        for cap in uppercase_re.captures_iter(&processed.clone()) {
            let text = &cap[1];
            processed = processed.replace(&cap[0], &text.to_uppercase());
        }
        
        Ok(processed)
    }
    
    fn extract_factions(&mut self, glsmac_path: &Path) -> Result<Vec<Faction>> {
        let factions_path = glsmac_path.join("GLSMAC_data/default/factions.gls.js");
        let content = self.parse_file(&factions_path)?;
        
        let mut factions = Vec::new();
        
        // Parse faction array with new format
        let faction_re = Regex::new(r#"\['([^']+)',\s*'([^']+)',\s*\{([^}]*)\}\]"#)?;
        
        for cap in faction_re.captures_iter(&content) {
            let name = cap[1].to_string();
            let filename = cap[2].to_string();
            let options = &cap[3];
            let key = name.to_uppercase().replace(" ", "_").replace("'", "");
            
            let faction = Faction {
                key: key.clone(),
                name,
                filename,
                is_naval: options.contains("is_naval: true"),
                is_progenitor: options.contains("is_progenitor: true"),
            };
            
            factions.push(faction);
        }
        
        Ok(factions)
    }
    
    fn extract_units(&mut self, glsmac_path: &Path) -> Result<Vec<Unit>> {
        let units_path = glsmac_path.join("GLSMAC_data/default/units/defs.gls.js");
        let content = self.parse_file(&units_path)?;
        
        let mut units = Vec::new();
        
        // Parse native lifeforms from the units array
        let units_re = Regex::new(r"native_lifeform\('([^']+)',\s*'([^']+)',\s*'([^']+)',\s*(\d+),\s*(\d+)\)")?;
        
        for cap in units_re.captures_iter(&content) {
            let key = cap[1].to_string();
            let name = cap[2].to_string();
            let movement_type = cap[3].to_string();
            let movement_per_turn: u32 = cap[4].parse()?;
            let base_y: u32 = cap[5].parse()?;
            
            let unit = Unit {
                key,
                name,
                sprite_x: 2,  // From the render definition
                sprite_y: base_y,
                movement_per_turn,
                unit_type: movement_type,
            };
            units.push(unit);
        }
        
        Ok(units)
    }
    
    fn extract_population(&mut self, glsmac_path: &Path) -> Result<Vec<PopulationType>> {
        let pop_path = glsmac_path.join("GLSMAC_data/default/bases/pops.gls.js");
        let content = self.parse_file(&pop_path)?;
        
        let mut population_types = Vec::new();
        
        // Parse define() function calls
        let pop_re = Regex::new(r"define\(game,\s*'([^']+)',\s*'([^']+)',\s*\[([^\]]+)\],\s*\[([^\]]+)\],\s*\{([^}]*)\}\)")?;
        
        for cap in pop_re.captures_iter(&content) {
            let key = cap[1].to_string();
            let name = cap[2].to_string();
            let human_coords = &cap[3];
            let prog_coords = &cap[4];
            let properties = &cap[5];
            
            // Extract first coordinate from arrays
            let human_x = if let Some(x_cap) = Regex::new(r"(\d+)")?.captures(human_coords) {
                Some(x_cap[1].parse()?)
            } else {
                None
            };
            
            let prog_x = if let Some(x_cap) = Regex::new(r"(\d+)")?.captures(prog_coords) {
                Some(x_cap[1].parse()?)
            } else {
                None
            };
            
            let pop_type = PopulationType {
                key: key.clone(),
                name: name.clone(),
                category: if properties.contains("tile_worker") {
                    "tile_worker".to_string()
                } else if key == "Drone" || key == "DronePlus" {
                    "drone".to_string()
                } else {
                    "specialist".to_string()
                },
                is_tile_worker: properties.contains("tile_worker"),
                is_specialist: !properties.contains("tile_worker") && key != "Drone" && key != "DronePlus",
                is_drone: key == "Drone" || key == "DronePlus",
                human_sprite_x: human_x,
                human_sprite_y: Some(501), // From the file
                progenitor_sprite_x: prog_x,
                progenitor_sprite_y: Some(41), // From the file
            };
            
            population_types.push(pop_type);
        }
        
        Ok(population_types)
    }
}

fn generate_rust_modules(data: &ExtractedData, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    
    // Generate factions.rs
    let mut factions_code = String::from("// Auto-generated faction definitions\n\n");
    factions_code.push_str("use std::collections::HashMap;\n\n");
    factions_code.push_str("#[derive(Debug, Clone)]\n");
    factions_code.push_str("pub struct Faction {\n");
    factions_code.push_str("    pub key: &'static str,\n");
    factions_code.push_str("    pub name: &'static str,\n");
    factions_code.push_str("    pub filename: &'static str,\n");
    factions_code.push_str("    pub is_naval: bool,\n");
    factions_code.push_str("    pub is_progenitor: bool,\n");
    factions_code.push_str("}\n\n");
    factions_code.push_str("lazy_static::lazy_static! {\n");
    factions_code.push_str("    pub static ref FACTIONS: HashMap<&'static str, Faction> = {\n");
    factions_code.push_str("        let mut m = HashMap::new();\n");
    
    for faction in &data.factions {
        factions_code.push_str(&format!(
            "        m.insert(\"{}\", Faction {{\n",
            faction.key
        ));
        factions_code.push_str(&format!("            key: \"{}\",\n", faction.key));
        factions_code.push_str(&format!("            name: \"{}\",\n", faction.name));
        factions_code.push_str(&format!("            filename: \"{}\",\n", faction.filename));
        factions_code.push_str(&format!("            is_naval: {},\n", faction.is_naval));
        factions_code.push_str(&format!("            is_progenitor: {},\n", faction.is_progenitor));
        factions_code.push_str("        });\n");
    }
    
    factions_code.push_str("        m\n");
    factions_code.push_str("    };\n");
    factions_code.push_str("}\n");
    
    fs::write(output_dir.join("factions.rs"), factions_code)?;
    
    // Similar generation for units.rs and population.rs...
    
    Ok(())
}

fn generate_godot_scripts(data: &GodotData, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    
    // Generate FactionDefinitions.gd
    let mut factions_gd = String::from("# Auto-generated faction definitions\n");
    factions_gd.push_str("class_name FactionDefinitions\n\n");
    factions_gd.push_str("static var _factions = {\n");
    
    for (key, faction) in &data.faction_definitions {
        factions_gd.push_str(&format!("    \"{}\": {{\n", key));
        factions_gd.push_str(&format!("        \"key\": \"{}\",\n", faction.key));
        factions_gd.push_str(&format!("        \"name\": \"{}\",\n", faction.name));
        factions_gd.push_str(&format!("        \"filename\": \"{}\",\n", faction.filename));
        factions_gd.push_str(&format!("        \"is_naval\": {},\n", faction.is_naval));
        factions_gd.push_str(&format!("        \"is_progenitor\": {},\n", faction.is_progenitor));
        factions_gd.push_str("    },\n");
    }
    
    factions_gd.push_str("}\n\n");
    factions_gd.push_str("static func get_faction(key: String):\n");
    factions_gd.push_str("    return _factions.get(key)\n\n");
    factions_gd.push_str("static func get_all_factions():\n");
    factions_gd.push_str("    return _factions.values()\n");
    
    fs::write(output_dir.join("FactionDefinitions.gd"), factions_gd)?;
    
    // Similar for other definitions...
    
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Validate GLSMAC path
    if !args.glsmac_path.exists() {
        bail!("GLSMAC path does not exist: {}", args.glsmac_path.display());
    }
    
    println!("Extracting data from GLSMAC at: {}", args.glsmac_path.display());
    
    let mut parser = GLSParser::new();
    
    // Extract all data
    let factions = parser.extract_factions(&args.glsmac_path)
        .context("Failed to extract factions")?;
    println!("Extracted {} factions", factions.len());
    
    let units = parser.extract_units(&args.glsmac_path)
        .context("Failed to extract units")?;
    println!("Extracted {} units", units.len());
    
    let population_types = parser.extract_population(&args.glsmac_path)
        .context("Failed to extract population types")?;
    println!("Extracted {} population types", population_types.len());
    
    let technologies = Vec::new(); // Placeholder
    
    let data = ExtractedData {
        factions,
        units,
        population_types,
        technologies,
    };
    
    if args.godot {
        // Convert to Godot format
        let godot_data = GodotData {
            faction_definitions: data.factions.iter()
                .map(|f| (f.key.clone(), f.clone()))
                .collect(),
            unit_definitions: data.units.iter()
                .map(|u| (u.key.clone(), u.clone()))
                .collect(),
            population_definitions: data.population_types.iter()
                .map(|p| (p.key.clone(), p.clone()))
                .collect(),
            technology_tree: HashMap::new(),
        };
        
        // Save Godot JSON
        let godot_json = serde_json::to_string_pretty(&godot_data)?;
        fs::write(&args.output, godot_json)?;
        
        // Generate GDScript files
        let script_dir = args.output.parent().unwrap().join("godot_scripts");
        generate_godot_scripts(&godot_data, &script_dir)?;
        
        println!("Generated Godot data and scripts");
    } else if args.rust_modules {
        // Generate Rust modules
        let module_dir = args.output.parent().unwrap().join("rust_modules");
        generate_rust_modules(&data, &module_dir)?;
        println!("Generated Rust modules at: {}", module_dir.display());
    } else {
        // Save raw JSON
        let json = serde_json::to_string_pretty(&data)?;
        fs::write(&args.output, json)?;
        println!("Saved raw data to: {}", args.output.display());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_faction_parsing() {
        let test_content = r#"
        ['Gaians', 'gaians', {}],
        ['Pirates', 'pirates', {is_naval: true}],
        "#;
        
        // Test parsing logic here
    }
}
