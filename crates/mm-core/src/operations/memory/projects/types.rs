use chrono::{DateTime, Utc};
use mm_memory::{MemoryEntity, value::MemoryValue};
use std::collections::HashMap;

use crate::operations::memory::{git::types::GitRepositoryProperties, tasks::TaskProperties};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Properties for Project entities
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ProjectProperties {
    /// Description of the project
    pub description: String,

    /// Creation date
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,

    /// Last updated date
    #[schemars(with = "String")]
    pub updated_at: DateTime<Utc>,

    /// Project status
    pub status: ProjectStatus,

    /// Project type
    pub project_type: ProjectType,
}

impl Default for ProjectProperties {
    fn default() -> Self {
        ProjectProperties {
            description: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: ProjectStatus::Active,
            project_type: ProjectType::Other,
        }
    }
}

/// Context information about a project

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
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

impl From<HashMap<String, MemoryValue>> for ProjectProperties {
    fn from(mut map: HashMap<String, MemoryValue>) -> Self {
        let description = match map.remove("description") {
            Some(MemoryValue::String(s)) => s,
            Some(v) => v.to_string(),
            None => String::new(),
        };
        let created_at = match map.remove("created_at") {
            Some(MemoryValue::DateTime(dt)) => dt.with_timezone(&Utc),
            Some(MemoryValue::String(s)) => DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            _ => Utc::now(),
        };
        let updated_at = match map.remove("updated_at") {
            Some(MemoryValue::DateTime(dt)) => dt.with_timezone(&Utc),
            Some(MemoryValue::String(s)) => DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            _ => Utc::now(),
        };
        let status = match map.remove("status") {
            Some(MemoryValue::String(s)) => match s.to_lowercase().as_str() {
                "active" => ProjectStatus::Active,
                "maintenance" => ProjectStatus::Maintenance,
                "archived" => ProjectStatus::Archived,
                "planning" => ProjectStatus::Planning,
                _ => ProjectStatus::Active,
            },
            _ => ProjectStatus::Active,
        };
        let project_type = match map.remove("project_type") {
            Some(MemoryValue::String(s)) => match s.to_lowercase().as_str() {
                "application" => ProjectType::Application,
                "library" => ProjectType::Library,
                "tool" => ProjectType::Tool,
                "configuration" => ProjectType::Configuration,
                "documentation" => ProjectType::Documentation,
                _ => ProjectType::Other,
            },
            _ => ProjectType::Other,
        };
        ProjectProperties {
            description,
            created_at,
            updated_at,
            status,
            project_type,
        }
    }
}

impl From<ProjectProperties> for HashMap<String, MemoryValue> {
    fn from(props: ProjectProperties) -> Self {
        let mut map = HashMap::new();
        map.insert(
            "description".to_string(),
            MemoryValue::String(props.description),
        );
        map.insert(
            "created_at".to_string(),
            MemoryValue::DateTime(props.created_at.into()),
        );
        map.insert(
            "updated_at".to_string(),
            MemoryValue::DateTime(props.updated_at.into()),
        );
        map.insert(
            "status".to_string(),
            MemoryValue::String(
                match props.status {
                    ProjectStatus::Active => "active",
                    ProjectStatus::Maintenance => "maintenance",
                    ProjectStatus::Archived => "archived",
                    ProjectStatus::Planning => "planning",
                }
                .to_string(),
            ),
        );
        map.insert(
            "project_type".to_string(),
            MemoryValue::String(
                match props.project_type {
                    ProjectType::Application => "application",
                    ProjectType::Library => "library",
                    ProjectType::Tool => "tool",
                    ProjectType::Configuration => "configuration",
                    ProjectType::Documentation => "documentation",
                    ProjectType::Other => "other",
                }
                .to_string(),
            ),
        );
        map
    }
}
