use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::relationship::MemoryRelationship;
use crate::value::MemoryValue;

/// Memory entity representing a node in the knowledge graph
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema, Default)]
pub struct MemoryEntity<P = HashMap<String, MemoryValue>>
where
    P: JsonSchema
        + Into<HashMap<String, MemoryValue>>
        + From<HashMap<String, MemoryValue>>
        + Clone
        + std::fmt::Debug
        + Default,
{
    /// Unique name of the entity
    pub name: String,
    /// Labels for categorizing the entity
    pub labels: Vec<String>,
    /// Facts or notes about the entity
    pub observations: Vec<String>,
    /// Additional key-value properties
    #[serde(default)]
    pub properties: P,
    /// Relationships connected to the entity
    #[serde(default)]
    pub relationships: Vec<MemoryRelationship<P>>,
}
