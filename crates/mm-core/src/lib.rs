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
    AddObservationsCommand, AddObservationsResult, CreateEntitiesCommand, CreateEntitiesResult,
    CreateRelationshipsCommand, CreateRelationshipsResult, DeleteEntitiesCommand,
    DeleteEntitiesResult, DeleteRelationshipsCommand, DeleteRelationshipsResult,
    FindEntitiesByLabelsCommand, FindEntitiesByLabelsResult, FindEntitiesByLabelsResultType,
    FindRelationshipsCommand, FindRelationshipsResult, FindRelationshipsResultType,
    GetEntityCommand, GetEntityResult, GetProjectContextCommand, GetProjectContextResult,
    ListProjectsCommand, ListProjectsResult, ProjectFilter, RemoveAllObservationsCommand,
    RemoveAllObservationsResult, RemoveObservationsCommand, RemoveObservationsResult,
    SetObservationsCommand, SetObservationsResult, UpdateEntityCommand, UpdateEntityResult,
    UpdateRelationshipCommand, UpdateRelationshipResult, add_observations, create_entities,
    create_relationships, delete_entities, delete_relationships, find_entities_by_labels,
    find_relationships, get_entity, get_project_context, list_projects, remove_all_observations,
    remove_observations, set_observations, update_entity, update_relationship,
};
pub use ports::Ports;
pub use root::{Root, RootCollection};

// Re-export types from mm-memory
pub use mm_memory::{MemoryEntity, MemoryRelationship, ProjectContext};
