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

#[cfg(test)]
mod test_utils;

pub use error::{CoreError, CoreResult};
pub use mm_memory::MemoryService;
pub use operations::{
    AddObservationsCommand, AddObservationsResult, CreateEntityCommand, CreateEntityResult,
    CreateRelationshipCommand, CreateRelationshipResult, GetEntityCommand, GetEntityResult,
    RemoveAllObservationsCommand, RemoveAllObservationsResult, RemoveObservationsCommand,
    RemoveObservationsResult, SetObservationsCommand, SetObservationsResult, add_observations,
    create_entity, create_relationship, get_entity, remove_all_observations, remove_observations,
    set_observations,
};
pub use ports::Ports;

// Re-export types from mm-memory
pub use mm_memory::{MemoryEntity, MemoryRelationship};
