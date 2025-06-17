use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Configuration options for memory service behavior
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryConfig {
    /// Optional tag automatically added to every created entity
    #[serde(default)]
    pub default_tag: Option<String>,

    /// Enforce use of default relationships
    #[serde(default = "MemoryConfig::default_true")]
    pub default_relationships: bool,

    /// Additional relationships allowed when default_relationships is enabled
    #[serde(default)]
    pub additional_relationships: HashSet<String>,
}

/// Default tag used when none is specified in the configuration
pub const DEFAULT_MEMORY_TAG: &str = "Memory";

/// Default set of allowed relationship names
pub const DEFAULT_RELATIONSHIPS: &[&str] = &["related_to", "parent_of", "child_of"];

impl MemoryConfig {
    /// Helper for serde default of boolean true
    fn default_true() -> bool {
        true
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            default_tag: Some(DEFAULT_MEMORY_TAG.to_string()),
            default_relationships: true,
            additional_relationships: HashSet::new(),
        }
    }
}
