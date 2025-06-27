use anyhow::{Context, Result};
use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};
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
pub struct Population {
    pub key: String,
    pub name: String,
    pub sprite_human_x: u32,
    pub sprite_human_y: u32,
    pub sprite_progenitor_x: u32,
    pub sprite_progenitor_y: u32,
    pub is_tile_worker: bool,
    pub is_specialist: bool,
    pub is_drone: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameData {
    pub metadata: Metadata,
    pub factions: Vec<Faction>,
    pub units: Vec<Unit>,
    pub population: Vec<Population>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub extracted_from: String,
    pub extraction_date: String,
    pub version: String,
}

fn read_gls_file(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))
}

fn extract_factions(glsmac_path: &Path) -> Result<Vec<Faction>> {
    let factions_file = glsmac_path.join("GLSMAC_data/default/factions.gls.js");
    
    if !factions_file.exists() {
        println!("Warning: Factions file not found at {}", factions_file.display());
        return Ok(Vec::new());
    }
    
    let content = read_gls_file(&factions_file)?;
    let mut factions = Vec::new();
    
    // Parse faction definitions with regex
    let faction_regex = Regex::new(
        r#"\{\s*"([^"]+)",\s*"([^"]+)",\s*"([^"]+)",\s*(\w+),\s*(\w+)(?:,\s*(\w+))?\s*\}"#
    )?;
    
    for cap in faction_regex.captures_iter(&content) {
        let faction = Faction {
            key: cap[1].to_string(),
            name: cap[2].to_string(),
            filename: cap[3].to_string(),
            is_naval: &cap[4] == "true",
            is_progenitor: &cap[5] == "true",
        };
        factions.push(faction);
    }
    
    println!("Extracted {} factions", factions.len());
    Ok(factions)
}

fn extract_units(glsmac_path: &Path) -> Result<Vec<Unit>> {
    let units_file = glsmac_path.join("GLSMAC_data/default/units.gls.js");
    
    if !units_file.exists() {
        println!("Warning: Units file not found at {}", units_file.display());
        return Ok(Vec::new());
    }
    
    let content = read_gls_file(&units_file)?;
    let mut units = Vec::new();
    
    // Define patterns for different unit types
    let unit_patterns = [
        (r"FungalTower\(\s*\"([^\"]+)\",\s*(\d+),\s*(\d+)\s*\)", "FungalTower", 0),
        (r"MindWorms\(\s*\"([^\"]+)\",\s*(\d+),\s*(\d+),\s*(\d+)\s*\)", "MindWorms", 1),
        (r"SeaLurk\(\s*\"([^\"]+)\",\s*(\d+),\s*(\d+),\s*(\d+)\s*\)", "SeaLurk", 4),
        (r"SporeLauncher\(\s*\"([^\"]+)\",\s*(\d+),\s*(\d+),\s*(\d+)\s*\)", "SporeLauncher", 1),
    ];
    
    for (pattern, unit_key, default_movement) in unit_patterns {
        let regex = Regex::new(pattern)?;
        
        for cap in regex.captures_iter(&content) {
            let movement = if cap.len() > 4 {
                cap[4].parse::<u32>().unwrap_or(default_movement)
            } else {
                default_movement
            };
            
            let unit = Unit {
                key: unit_key.to_string(),
                name: cap[1].to_string(),
                sprite_x: cap[2].parse()?,
                sprite_y: cap[3].parse()?,
                movement_per_turn: movement,
                unit_type: "native_lifeform".to_string(),
            };
            units.push(unit);
        }
    }
    
    println!("Extracted {} units", units.len());
    Ok(units)
}

fn extract_population(glsmac_path: &Path) -> Result<Vec<Population>> {
    let pop_file = glsmac_path.join("GLSMAC_data/default/base_population.gls.js");
    
    if !pop_file.exists() {
        println!("Warning: Population file not found at {}", pop_file.display());
        return Ok(Vec::new());
    }
    
    let content = read_gls_file(&pop_file)?;
    let mut population = Vec::new();
    
    // Parse population function calls
    let pop_regex = Regex::new(r"(\w+)\(\s*(\d+),\s*(\d+),\s*(\d+),\s*(\d+)\s*\)")?;
    
    for cap in pop_regex.captures_iter(&content) {
        let pop_type = &cap[1];
        
        let pop = Population {
            key: pop_type.to_string(),
            name: pop_type.to_string(),
            sprite_human_x: cap[2].parse()?,
            sprite_human_y: cap[3].parse()?,
            sprite_progenitor_x: cap[4].parse()?,
            sprite_progenitor_y: cap[5].parse()?,
            is_tile_worker: matches!(pop_type, "Worker" | "Talent"),
            is_specialist: !matches!(pop_type, "Worker" | "Talent" | "Drone" | "DronePlus"),
            is_drone: pop_type.contains("Drone"),
        };
        population.push(pop);
    }
    
    println!("Extracted {} population types", population.len());
    Ok(population)
}

fn generate_rust_modules(data: &GameData, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    
    // Generate factions module
    let factions_code = format!(
        r#"// Auto-generated from GLSMAC data extraction
use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {{
    pub key: String,
    pub name: String,
    pub filename: String,
    pub is_naval: bool,
    pub is_progenitor: bool,
}}

pub const FACTIONS: &[Faction] = &[
{}
];

pub fn get_faction(key: &str) -> Option<&'static Faction> {{
    FACTIONS.iter().find(|f| f.key == key)
}}

pub fn get_all_factions() -> &'static [Faction] {{
    FACTIONS
}}
"#,
        data.factions
            .iter()
            .map(|f| format!(
                r#"    Faction {{
        key: "{}".to_string(),
        name: "{}".to_string(),
        filename: "{}".to_string(),
        is_naval: {},
        is_progenitor: {},
    }},"#,
                f.key, f.name, f.filename, f.is_naval, f.is_progenitor
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
    
    fs::write(output_dir.join("factions.rs"), factions_code)?;
    
    // Generate units module
    let units_code = format!(
        r#"// Auto-generated from GLSMAC data extraction
use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unit {{
    pub key: String,
    pub name: String,
    pub sprite_x: u32,
    pub sprite_y: u32,
    pub movement_per_turn: u32,
    pub unit_type: String,
}}

pub const UNITS: &[Unit] = &[
{}
];

pub fn get_unit(key: &str) -> Option<&'static Unit> {{
    UNITS.iter().find(|u| u.key == key)
}}

pub fn get_all_units() -> &'static [Unit] {{
    UNITS
}}
"#,
        data.units
            .iter()
            .map(|u| format!(
                r#"    Unit {{
        key: "{}".to_string(),
        name: "{}".to_string(),
        sprite_x: {},
        sprite_y: {},
        movement_per_turn: {},
        unit_type: "{}".to_string(),
    }},"#,
                u.key, u.name, u.sprite_x, u.sprite_y, u.movement_per_turn, u.unit_type
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
    
    fs::write(output_dir.join("units.rs"), units_code)?;
    
    // Generate mod.rs
    let mod_code = r#"// Auto-generated game data modules
pub mod factions;
pub mod units;
pub mod population;

pub use factions::*;
pub use units::*;
pub use population::*;
"#;
    
    fs::write(output_dir.join("mod.rs"), mod_code)?;
    
    println!("Generated Rust modules in {}", output_dir.display());
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    if !args.glsmac_path.exists() {
        anyhow::bail!("GLSMAC path does not exist: {}", args.glsmac_path.display());
    }
    
    println!("Extracting GLSMAC data from: {}", args.glsmac_path.display());
    
    // Extract all data
    let factions = extract_factions(&args.glsmac_path)?;
    let units = extract_units(&args.glsmac_path)?;
    let population = extract_population(&args.glsmac_path)?;
    
    let game_data = GameData {
        metadata: Metadata {
            extracted_from: args.glsmac_path.display().to_string(),
            extraction_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            version: "1.0".to_string(),
        },
        factions,
        units,
        population,
    };
    
    if args.rust_modules {
        // Generate Rust modules
        let output_dir = Path::new("src/data");
        generate_rust_modules(&game_data, output_dir)?;
    } else {
        // Save to JSON
        let json = serde_json::to_string_pretty(&game_data)?;
        fs::write(&args.output, json)?;
        println!("Data saved to: {}", args.output.display());
    }
    
    println!("Extraction complete!");
    println!("- {} factions", game_data.factions.len());
    println!("- {} units", game_data.units.len());
    println!("- {} population types", game_data.population.len());
    
    Ok(())
}
