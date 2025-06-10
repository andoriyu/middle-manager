use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Memory entity representing a node in the knowledge graph
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MemoryEntity {
    pub name: String,
    pub labels: Vec<String>,
    pub observations: Vec<String>,
    pub properties: HashMap<String, String>,
}

/// Check if a string is in snake_case format
pub fn is_snake_case(s: &str) -> bool {
    s.chars().all(|c| c.is_lowercase() || c == '_' || c.is_numeric())
}
