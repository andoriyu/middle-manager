//! Core domain logic for the Middle Manager project.
//!
//! This crate represents the application "core" in the hexagonal
//! architecture. It defines operations on the memory graph and the
//! ports (interfaces) that adapters must implement. The code here is
//! completely independent of external protocols or infrastructure and
//! focuses purely on business rules.
#![warn(clippy::all)]
pub mod error;
mod operations;
mod ports;
mod root;

pub use error::{CoreError, CoreResult};
pub use mm_memory::MemoryService;
pub use operations::{
    CreateEntitiesCommand, CreateEntitiesResult, CreateRelationshipsCommand,
    CreateRelationshipsResult, CreateTaskCommand, CreateTaskResult, DeleteEntitiesCommand,
    DeleteEntitiesResult, DeleteRelationshipsCommand, DeleteRelationshipsResult, DeleteTaskCommand,
    DeleteTaskResult, FindEntitiesByLabelsCommand, FindEntitiesByLabelsResult,
    FindEntitiesByLabelsResultType, FindRelationshipsCommand, FindRelationshipsResult,
    FindRelationshipsResultType, GetEntityCommand, GetEntityResult, GetProjectContextCommand,
    GetProjectContextResult, GetTaskCommand, GetTaskResult, ListProjectsCommand,
    ListProjectsResult, ProjectFilter, UpdateEntityCommand, UpdateEntityResult,
    UpdateRelationshipCommand, UpdateRelationshipResult, UpdateTaskCommand, UpdateTaskResult,
    create_entities, create_relationships, create_task, delete_entities, delete_relationships,
    delete_task, find_entities_by_labels, find_relationships, get_entity, get_project_context,
    get_task, list_projects, update_entity, update_relationship, update_task,
};
pub use ports::Ports;
pub use root::{Root, RootCollection};

// Re-export types from mm-memory
pub use mm_memory::{MemoryEntity, MemoryRelationship, ProjectContext};
pub use operations::{Priority, TaskProperties, TaskStatus, TaskType};
