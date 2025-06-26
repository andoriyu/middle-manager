#[macro_use]
mod common;
mod generic;
mod git;
// Re-export label constants from the memory crate
pub use mm_memory::labels;
mod projects;
mod tasks;

pub(crate) use validate_name;

pub mod create_entity;
pub mod create_relationship;
pub mod delete_entities;
pub mod delete_relationships;
pub mod find_entities_by_labels;
pub mod find_relationships;
pub mod get_entity;
pub mod get_project_context;
pub mod list_projects;
pub mod update_entity;
pub mod update_relationship;

pub use create_entity::{CreateEntitiesCommand, CreateEntitiesResult, create_entities};
pub use create_relationship::{
    CreateRelationshipsCommand, CreateRelationshipsResult, create_relationships,
};
pub use delete_entities::{DeleteEntitiesCommand, DeleteEntitiesResult, delete_entities};
pub use delete_relationships::{
    DeleteRelationshipsCommand, DeleteRelationshipsResult, delete_relationships,
};
pub use find_entities_by_labels::{
    FindEntitiesByLabelsCommand, FindEntitiesByLabelsResult, FindEntitiesByLabelsResultType,
    find_entities_by_labels,
};
pub use find_relationships::{
    FindRelationshipsCommand, FindRelationshipsResult, FindRelationshipsResultType,
    find_relationships,
};
pub use generic::{get_entity_generic, update_entity_generic};
pub use get_entity::{GetEntityCommand, GetEntityResult, get_entity};
pub use get_project_context::{
    GetProjectContextCommand, GetProjectContextResult, ProjectFilter, get_project_context,
};
pub use labels::*;
pub use list_projects::{ListProjectsCommand, ListProjectsResult, list_projects};
pub use projects::{ProjectContext, ProjectProperties, ProjectStatus, ProjectType};
pub use tasks::{
    CreateTasksCommand, CreateTasksResult, DeleteTaskCommand, DeleteTaskResult, GetTaskCommand,
    GetTaskResult, Priority, TaskInput, TaskProperties, TaskStatus, TaskType, UpdateTaskCommand,
    UpdateTaskResult, create_tasks, delete_task, get_task, update_task,
};
pub use update_entity::{UpdateEntityCommand, UpdateEntityResult, update_entity};
pub use update_relationship::{
    UpdateRelationshipCommand, UpdateRelationshipResult, update_relationship,
};
