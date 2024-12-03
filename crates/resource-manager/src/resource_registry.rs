use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize)]
pub struct ResourceRegistry {
    pub version: i32,
    pub items: Vec<ResourceRegistryItem>,
}
impl ResourceRegistry {
    pub fn serialize(&self) -> String {
        if let Some(json) = serde_json::to_string_pretty(self).ok() {
            json
        } else {
            String::from("{}")
        }
    }
}
impl Debug for ResourceRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(str) = serde_json::to_string_pretty(self).ok() {
            f.write_str(&str)
        } else {
            f.write_str(&format!("ResourceIndex {:?}", self.version))
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct ResourceRegistryItem {
    pub name: String,
    pub files: Vec<String>,
}
