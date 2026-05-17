use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    pub archetype: String,
    pub preferred_victory: String,
    pub forbidden_actions: Vec<String>,
    pub required_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    pub id: u32,
    pub name: String,
    pub leader: String,
    pub description: String,
    pub bonuses: Vec<String>,
    pub penalties: Vec<String>,
    pub color: String,
    pub personality: Personality,
}

pub fn load_factions(path: &str) -> anyhow::Result<Vec<Faction>> {
    let data = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}
