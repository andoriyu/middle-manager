mod config;
pub mod error;
mod operations;
mod ports;
mod service;

pub use config::Config;
pub use error::{CoreError, Error, Result as CoreResult};
pub use operations::{
    CreateEntityCommand, CreateEntityError, CreateEntityResult, GetEntityCommand, GetEntityError,
    GetEntityResult, create_entity, get_entity,
};
pub use ports::Ports;
pub use service::MemoryService;

// Re-export necessary types from mm-memory-neo4j
pub use mm_memory_neo4j::{
    MemoryEntity, MemoryService as MemoryServiceImpl, Neo4jRepository, create_neo4j_service, neo4rs,
};

#[cfg(test)]
pub use service::MockMemoryService;
