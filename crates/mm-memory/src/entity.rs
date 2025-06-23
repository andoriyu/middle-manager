use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::relationship::MemoryRelationship;

/// Memory entity representing a node in the knowledge graph
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema, Default)]
pub struct MemoryEntity {
    /// Unique name of the entity
    pub name: String,
    /// Labels for categorizing the entity
    pub labels: Vec<String>,
    /// Facts or notes about the entity
    pub observations: Vec<String>,
    /// Additional key-value properties
    #[serde(default)]
    pub properties: HashMap<String, String>,
    /// Relationships connected to the entity
    #[serde(default)]
    pub relationships: Vec<MemoryRelationship>,
}
