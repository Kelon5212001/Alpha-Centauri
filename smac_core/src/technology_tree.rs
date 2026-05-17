use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::OnceLock;

const TECHNOLOGY_TREE_JSON: &str = include_str!("../../data/technology_tree.json");
static TECHNOLOGY_DEFINITIONS: OnceLock<Result<Vec<Technology>, String>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Technology {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TechCategory,
    pub level: u8,
    pub cost: i32,
    pub prerequisites: Vec<String>,
    pub enables: TechEnables,
    pub quote: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TechCategory {
    Build,
    Discover,
    Explore,
    Conquer,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TechEnables {
    pub units: Vec<String>,
    pub facilities: Vec<String>,
    #[serde(default)]
    pub secret_projects: Vec<String>,
    #[serde(default)]
    pub orbital: Vec<String>,
    #[serde(default)]
    pub weapons: Vec<String>,
    #[serde(default)]
    pub armor: Vec<String>,
    pub abilities: Vec<String>,
    pub terraforming: Vec<String>,
    pub social_engineering: Vec<String>,
}

pub struct TechnologyTree {
    technologies: Vec<Technology>,
}

impl TechnologyTree {
    pub fn new() -> Self {
        Self::try_new().expect("bundled technology tree content must parse for runtime access")
    }

    pub fn try_new() -> Result<Self, String> {
        Ok(Self {
            technologies: canonical_technologies()?.clone(),
        })
    }

    pub fn get_technology(&self, id: &str) -> Option<&Technology> {
        self.technologies.iter().find(|t| t.id == id)
    }

    pub fn get_available_technologies(&self, researched: &HashSet<String>) -> Vec<&Technology> {
        self.technologies
            .iter()
            .filter(|tech| {
                !researched.contains(&tech.id)
                    && tech.prerequisites.iter().all(|p| researched.contains(p))
            })
            .collect()
    }

    pub fn all_technologies(&self) -> &[Technology] {
        &self.technologies
    }

    pub fn validate_prerequisites(&self) -> Result<(), String> {
        for tech in &self.technologies {
            for prereq in &tech.prerequisites {
                if !self.technologies.iter().any(|t| t.id == *prereq) {
                    return Err(format!(
                        "Technology '{}' has invalid prerequisite '{}'",
                        tech.name, prereq
                    ));
                }
            }
        }
        Ok(())
    }
}

pub(crate) fn canonical_technologies() -> Result<&'static Vec<Technology>, String> {
    TECHNOLOGY_DEFINITIONS
        .get_or_init(|| {
            serde_json::from_str(TECHNOLOGY_TREE_JSON).map_err(|error| {
                format!(
                    "bundled technology tree content must be valid JSON for the core game: {error}"
                )
            })
        })
        .as_ref()
        .map_err(Clone::clone)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn starting_count() {
        let tree = TechnologyTree::new();
        assert_eq!(tree.get_available_technologies(&HashSet::new()).len(), 5);
    }
}
