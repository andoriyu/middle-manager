pub mod domain;
pub mod ports;
pub mod adapters;
pub mod service;

// Re-export main types for convenience
pub use domain::entity::MemoryEntity;
pub use domain::error::{MemoryError, MemoryResult};
pub use domain::validation_error::ValidationError;
pub use ports::repository::MemoryRepository;
pub use adapters::neo4j::{Neo4jRepository, Neo4jConfig};
pub use service::memory::MemoryService;

/// Create a Neo4j-based memory service
pub async fn create_neo4j_service(
    config: Neo4jConfig
) -> Result<MemoryService<Neo4jRepository, neo4rs::Error>, MemoryError<neo4rs::Error>> {
    let repository = Neo4jRepository::new(config).await?;
    Ok(MemoryService::new(repository))
}
