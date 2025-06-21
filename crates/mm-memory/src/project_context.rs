use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::entity::MemoryEntity;

/// Comprehensive context information about a project and its related entities
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema, Default)]
pub struct ProjectContext {
    /// The project entity itself
    pub project: MemoryEntity,

    /// Technologies used in the project
    pub technologies: Vec<MemoryEntity>,

    /// Notes related to the project
    pub notes: Vec<MemoryEntity>,

    /// Components of the project
    pub components: Vec<MemoryEntity>,

    /// Tasks related to the project
    pub tasks: Vec<MemoryEntity>,

    /// Other related entities that don't fit into specific categories
    pub other_related_entities: Vec<MemoryEntity>,
}

impl ProjectContext {
    /// Create a new ProjectContext with the given project entity
    pub fn new(project: MemoryEntity) -> Self {
        Self {
            project,
            technologies: Vec::new(),
            notes: Vec::new(),
            components: Vec::new(),
            tasks: Vec::new(),
            other_related_entities: Vec::new(),
        }
    }

    /// Add a related entity to the appropriate category based on its labels
    pub fn add_related_entity(&mut self, entity: MemoryEntity) {
        let labels: Vec<&str> = entity.labels.iter().map(|s| s.as_str()).collect();

        if labels.contains(&"Technology")
            || labels.contains(&"Framework")
            || labels.contains(&"Library")
            || labels.contains(&"Language")
        {
            self.technologies.push(entity);
        } else if labels.contains(&"Note") {
            self.notes.push(entity);
        } else if labels.contains(&"Component") {
            self.components.push(entity);
        } else if labels.contains(&"Task") {
            self.tasks.push(entity);
        } else {
            self.other_related_entities.push(entity);
        }
    }
}
