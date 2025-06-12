mod config;
mod error;
mod ports;
mod operations;
mod service;

pub use config::Config;
pub use error::{Error, Result as CoreResult};
pub use ports::Ports;
pub use operations::{
    GetEntityCommand, GetEntityError, GetEntityResult, get_entity,
    CreateEntityCommand, CreateEntityError, CreateEntityResult, create_entity,
};
pub use service::MemoryService;

// Re-export necessary types from mm-memory
pub use mm_memory::{
    MemoryEntity, 
    create_neo4j_service, 
    Neo4jRepository, 
    neo4rs,
    MemoryService as MemoryServiceImpl,
};

#[cfg(test)]
pub use service::MockMemoryService;
