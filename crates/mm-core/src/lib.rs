pub mod error;
mod operations;
mod ports;

pub use error::{CoreError, CoreResult};
pub use mm_memory::MemoryService;
pub use operations::{
    AddObservationsCommand, AddObservationsError, AddObservationsResult, CreateEntityCommand,
    CreateEntityError, CreateEntityResult, GetEntityCommand, GetEntityError, GetEntityResult,
    RemoveAllObservationsCommand, RemoveAllObservationsError, RemoveAllObservationsResult,
    RemoveObservationsCommand, RemoveObservationsError, RemoveObservationsResult,
    SetObservationsCommand, SetObservationsError, SetObservationsResult, add_observations,
    create_entity, get_entity, remove_all_observations, remove_observations, set_observations,
};
pub use ports::Ports;

// Re-export types from mm-memory
pub use mm_memory::MemoryEntity;
