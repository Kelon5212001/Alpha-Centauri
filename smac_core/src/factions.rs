//! Faction content access for SMAC Rust.

pub use crate::content::{load_faction_definitions, FactionDefinition, FactionPersonality};

pub fn try_all_factions() -> Result<Vec<FactionDefinition>, String> {
    load_faction_definitions()
}

pub fn all_factions() -> Vec<FactionDefinition> {
    try_all_factions().expect("bundled faction content must parse")
}
