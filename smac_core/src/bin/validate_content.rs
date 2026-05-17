use smac_core::content_api::{
    load_facility_definitions, load_faction_definitions, load_production_definitions,
    load_runtime_tech_definitions, load_unit_definitions, validate_bundled_content,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    if let Err(errors) = validate_bundled_content() {
        let mut message = format!("Content validation failed with {} issue(s):", errors.len());
        for error in errors {
            message.push_str(&format!("\n- {error}"));
        }
        return Err(message);
    }

    let factions = load_faction_definitions()?;
    let techs = load_runtime_tech_definitions()?;
    let units = load_unit_definitions()?;
    let facilities = load_facility_definitions()?;
    let production = load_production_definitions()?;

    println!(
        "Content validation passed: {} factions, {} techs, {} units, {} facilities, {} production items.",
        factions.len(),
        techs.len(),
        units.len(),
        facilities.len(),
        production.len()
    );

    Ok(())
}
