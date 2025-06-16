pub mod error;
mod operations;
mod ports;


pub use error::{CoreError, CoreResult};
pub use operations::{
    CreateEntityCommand, CreateEntityError, CreateEntityResult, GetEntityCommand, GetEntityError,
    GetEntityResult, create_entity, get_entity,
};
pub use ports::Ports;
pub use mm_memory::MemoryService;

// Re-export types from mm-memory
pub use mm_memory::MemoryEntity;


