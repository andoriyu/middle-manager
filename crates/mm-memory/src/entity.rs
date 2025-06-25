use crate::BasicEntityProperties;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::relationship::MemoryRelationship;

/// Memory entity representing a node in the knowledge graph
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema, Default)]
pub struct MemoryEntity<P = BasicEntityProperties>
where
    P: JsonSchema
        + Into<BasicEntityProperties>
        + From<BasicEntityProperties>
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
    pub relationships: Vec<MemoryRelationship>,
}
