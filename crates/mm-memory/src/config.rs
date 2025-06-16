use serde::{Deserialize, Serialize};

/// Configuration options for memory service behavior
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryConfig {
    /// Optional tag automatically added to every created entity
    #[serde(default)]
    pub default_tag: Option<String>,
}

/// Default tag used when none is specified in the configuration
pub const DEFAULT_MEMORY_TAG: &str = "Memory";

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            default_tag: Some(DEFAULT_MEMORY_TAG.to_string()),
        }
    }
}
