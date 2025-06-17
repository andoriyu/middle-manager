use rust_mcp_sdk::macros::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Memory relationship representing an edge between entities
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct MemoryRelationship {
    /// Name of the source entity
    pub from: String,
    /// Name of the target entity
    pub to: String,
    /// Relationship type in snake_case
    pub name: String,
    /// Additional key-value properties
    #[serde(default)]
    pub properties: HashMap<String, String>,
}
