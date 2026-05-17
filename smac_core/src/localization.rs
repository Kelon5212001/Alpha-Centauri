use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocalizationData {
    pub strings: HashMap<String, String>,
}

pub struct Localization {
    data: LocalizationData,
}

impl Localization {
    pub fn new() -> Self {
        Self {
            data: LocalizationData::default(),
        }
    }

    pub fn load(json: &str) -> Result<Self, serde_json::Error> {
        let data: LocalizationData = serde_json::from_str(json)?;
        Ok(Self { data })
    }

    pub fn get(&self, key: &str) -> String {
        self.data
            .strings
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }
}
