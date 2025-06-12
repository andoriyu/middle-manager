mod config;
mod error;
mod mcp;
mod service;

pub use config::Config;
pub use error::{Error, Result};
pub use mcp::{CreateEntityTool, GetEntityTool};
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
