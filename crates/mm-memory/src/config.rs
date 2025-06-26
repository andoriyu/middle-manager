use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::labels::*;

/// Configuration options for memory service behavior
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryConfig {
    /// Optional label automatically added to every created entity
    #[serde(default)]
    pub default_label: Option<String>,

    /// Enforce use of default relationships
    #[serde(default = "MemoryConfig::default_true")]
    pub allow_default_relationships: bool,

    /// Relationship names allowed when `allow_default_relationships` is enabled
    #[serde(default)]
    pub allowed_relationships: HashSet<String>,

    /// Enforce use of default labels
    #[serde(default = "MemoryConfig::default_true")]
    pub allow_default_labels: bool,

    /// Label names allowed when `allow_default_labels` is enabled
    #[serde(default)]
    pub allowed_labels: HashSet<String>,

    /// Optional default project name to use when not explicitly specified
    #[serde(default)]
    pub default_project: Option<String>,

    /// Name of the agent using this configuration
    #[serde(default)]
    pub agent_name: String,
}

/// Default label used when none is specified in the configuration
pub const DEFAULT_MEMORY_LABEL: &str = "Memory";

/// Default set of allowed relationship names
pub const DEFAULT_RELATIONSHIPS: &[&str] = &[
    "relates_to",
    "owns",
    "makes",
    "uses",
    "uses_when_needed",
    "contains",
    "includes",
    "runs",
    "works_on",
    "performs",
    "branch_of",
    "follows",
    "implements",
    "references",
    "tagged_with",
    "example_of",
    "depends_on",
];

/// Default set of allowed label names derived from the schema
pub const DEFAULT_LABELS: &[&str] = &[
    USER_LABEL,
    OBSERVATION_COMPACTION_LABEL,
    SERVICE_LABEL,
    TOOL_LABEL,
    HOST_LABEL,
    LABEL_TAXONOMY_LABEL,
    ARCHITECTURE_LABEL,
    SPECIFICATION_LABEL,
    DEVELOPMENT_LABEL,
    NAMESPACE_REGISTRY_LABEL,
    FEATURE_LABEL,
    DATA_LABEL,
    MEMORY_LABEL,
    LIBRARY_LABEL,
    TAG_LABEL,
    PROCESS_LABEL,
    RELATIONSHIP_STANDARDIZATION_LABEL,
    PROJECT_LABEL,
    COMPONENT_LABEL,
    PLATFORM_LABEL,
    DUPLICATE_RELATIONSHIP_CLEANUP_LABEL,
    DOMAIN_LABEL,
    UI_LABEL,
    AGENT_LABEL,
    GIT_REPOSITORY_LABEL,
    SYSTEM_GROUP_LABEL,
    PLANNED_LABEL,
    SYSTEM_TYPE_LABEL,
    UTILITY_LABEL,
    METHODOLOGY_LABEL,
    ACTIVE_LABEL,
    GIT_CONVENTION_LABEL,
    CONVENTION_LABEL,
    LABEL_CATEGORY_LABEL,
    TASK_LABEL,
    PATTERN_LABEL,
    TECHNOLOGY_LABEL,
    BACKEND_LABEL,
    DOCUMENTATION_LABEL,
    PRINCIPLE_LABEL,
    BRANCH_LABEL,
    TECHNOLOGY_GROUP_LABEL,
    TEMPORAL_LABEL,
    TESTING_LABEL,
    INFRASTRUCTURE_LABEL,
    FRONTEND_LABEL,
    PACKAGE_LABEL,
    USEFUL_QUERY_LABEL,
    FILE_LABEL,
    DEV_OPS_LABEL,
    CONCEPT_LABEL,
    ORPHAN_NODE_CLEANUP_LABEL,
    NOTE_LABEL,
    FRAMEWORK_LABEL,
    CONFIGURATION_LABEL,
    MAINTENANCE_LABEL,
    LABEL_LABEL,
    LANGUAGE_LABEL,
];

impl MemoryConfig {
    /// Helper for serde default of boolean true
    fn default_true() -> bool {
        true
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            default_label: Some(DEFAULT_MEMORY_LABEL.to_string()),
            allow_default_relationships: true,
            allowed_relationships: HashSet::default(),
            allow_default_labels: true,
            allowed_labels: HashSet::default(),
            default_project: None,
            agent_name: "unknown".to_string(),
        }
    }
}
