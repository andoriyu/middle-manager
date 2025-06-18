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

    /// Enforce use of default labels
    #[serde(default = "MemoryConfig::default_true")]
    pub default_labels: bool,

    /// Additional labels allowed when default_labels is enabled
    #[serde(default)]
    pub additional_labels: HashSet<String>,
}

/// Default tag used when none is specified in the configuration
pub const DEFAULT_MEMORY_TAG: &str = "Memory";

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
];

/// Default set of allowed label names derived from the schema
pub const DEFAULT_LABELS: &[&str] = &[
    "User",
    "ObservationCompaction",
    "Service",
    "Tool",
    "Host",
    "LabelTaxonomy",
    "Architecture",
    "Specification",
    "Development",
    "NamespaceRegistry",
    "Feature",
    "Data",
    "Memory",
    "Library",
    "Tag",
    "Process",
    "RelationshipStandardization",
    "Project",
    "Component",
    "Platform",
    "DuplicateRelationshipCleanup",
    "Domain",
    "UI",
    "Agent",
    "GitRepository",
    "SystemGroup",
    "Planned",
    "SystemType",
    "Utility",
    "Methodology",
    "Active",
    "GitConvention",
    "Convention",
    "LabelCategory",
    "Task",
    "Pattern",
    "Technology",
    "Backend",
    "Documentation",
    "Principle",
    "Branch",
    "TechnologyGroup",
    "Temporal",
    "Testing",
    "Infrastructure",
    "Frontend",
    "Package",
    "UsefulQuery",
    "File",
    "DevOps",
    "Concept",
    "OrphanNodeCleanup",
    "Note",
    "Framework",
    "Configuration",
    "Maintenance",
    "Label",
    "Language",
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
            default_tag: Some(DEFAULT_MEMORY_TAG.to_string()),
            default_relationships: true,
            additional_relationships: HashSet::new(),
            default_labels: true,
            additional_labels: HashSet::new(),
        }
    }
}
