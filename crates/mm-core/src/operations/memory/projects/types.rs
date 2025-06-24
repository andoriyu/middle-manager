use chrono::{DateTime, Utc};
use mm_memory::MemoryEntity;

use crate::operations::memory::{git::types::GitRepositoryProperties, tasks::TaskProperties};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Properties for Project entities
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct ProjectProperties {
    /// Description of the project
    pub description: String,

    /// Creation date
    pub created_at: DateTime<Utc>,

    /// Last updated date
    pub updated_at: DateTime<Utc>,

    /// Project status
    pub status: ProjectStatus,

    /// Project type
    pub project_type: ProjectType,

}

/// Context information about a project

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ProjectContext {
    /// The project entity
    pub project: MemoryEntity<ProjectProperties>,

    /// Associated git repository (if any)
    pub git_repository: Option<MemoryEntity<GitRepositoryProperties>>,

    /// Tasks associated with the project
    pub tasks: Vec<MemoryEntity<TaskProperties>>,

    /// Technologies used in the project
    pub technologies: Vec<MemoryEntity>,

    /// Notes related to the project
    pub notes: Vec<MemoryEntity>,

    /// Other related entities
    pub other_related_entities: Vec<MemoryEntity>,
}

/// Project status
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub enum ProjectStatus {
    Active,
    Maintenance,
    Archived,
    Planning,
}

/// Project type
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub enum ProjectType {
    Application,
    Library,
    Tool,
    Configuration,
    Documentation,
    Other,
}
