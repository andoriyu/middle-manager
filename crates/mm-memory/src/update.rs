use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::value::MemoryValue;

/// Operations to modify an entity
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct EntityUpdate {
    /// Observations modifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observations: Option<ObservationsUpdate>,
    /// Properties modifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<PropertiesUpdate>,
    /// Labels modifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<LabelsUpdate>,
}

/// Update operations for observations
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ObservationsUpdate {
    /// Add observations to the entity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add: Option<Vec<String>>,
    /// Remove specific observations from the entity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<Vec<String>>,
    /// Replace all observations with this set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set: Option<Vec<String>>,
}

/// Update operations for properties
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PropertiesUpdate {
    /// Add or update properties on the entity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add: Option<HashMap<String, MemoryValue>>,
    /// Remove properties from the entity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<Vec<String>>,
    /// Replace all properties with this map
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set: Option<HashMap<String, MemoryValue>>,
}

/// Update operations for labels
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LabelsUpdate {
    /// Labels to add to the entity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add: Option<Vec<String>>,
    /// Labels to remove from the entity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<Vec<String>>,
}

/// Update operations for a relationship
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct RelationshipUpdate {
    /// Properties modifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<PropertiesUpdate>,
}
