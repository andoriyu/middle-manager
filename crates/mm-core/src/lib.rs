pub mod error;
mod operations;
mod ports;

pub use error::{CoreError, CoreResult};
pub use mm_memory::MemoryService;
pub use operations::{
    AddObservationsCommand, CreateEntityCommand, CreateEntityResult, CreateRelationshipCommand,
    GetEntityCommand, GetEntityResult, RemoveAllObservationsCommand, RemoveObservationsCommand,
    SetObservationsCommand, add_observations, create_entity, create_relationship, get_entity,
    remove_all_observations, remove_observations, set_observations,
};
pub use ports::Ports;

// Re-export types from mm-memory
pub use mm_memory::MemoryEntity;
