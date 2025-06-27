use anyhow::Result;

mod data;
mod game;

use crate::data::*;

fn main() -> Result<()> {
    println!("SMAC Reimplementation - Alpha Centauri");
    println!("=====================================");
    
    // Test loaded data
    println!("\nLoaded Game Data:");
    
    let all_factions = get_all_factions();
    println!("- {} factions loaded", all_factions.len());
    
    for faction in all_factions.iter().take(3) {
        println!("  * {} ({})", faction.name, faction.key);
    }
    
    let all_units = get_all_units();
    println!("- {} units loaded", all_units.len());
    
    for unit in all_units {
        println!("  * {} (movement: {})", unit.name, unit.movement_per_turn);
    }
    
    // TODO: Initialize game engine (Bevy/Godot/Custom)
    // TODO: Load assets
    // TODO: Initialize game state
    // TODO: Start main game loop
    
    println!("\nGame initialization complete!");
    println!("Next steps: Implement game engine integration");
    
    Ok(())
}
