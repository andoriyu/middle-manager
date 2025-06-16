use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Memory entity representing a node in the knowledge graph
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MemoryEntity {
    /// Unique name of the entity
    pub name: String,
    /// Labels for categorizing the entity
    pub labels: Vec<String>,
    /// Facts or notes about the entity
    pub observations: Vec<String>,
    /// Additional key-value properties
    pub properties: HashMap<String, String>,
}
