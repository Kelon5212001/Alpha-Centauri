use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Technology {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub prerequisites: Vec<String>,
    pub unlocks: Vec<String>,
    pub effects: Vec<String>,
    pub flavor: String,
}

pub fn load_tech_tree(path: &str) -> anyhow::Result<Vec<Technology>> {
    let data = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}
