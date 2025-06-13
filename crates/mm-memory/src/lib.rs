/*!
# Memory Management for Middle Manager

This crate provides a memory management system for the Middle Manager project,
implementing a knowledge graph using Neo4j as the backend storage.

## Architecture

The crate follows hexagonal architecture (also known as ports and adapters):

- **Domain**: Core business logic and entities
- **Ports**: Interface definitions for interacting with external systems
- **Adapters**: Implementations of ports for specific technologies
- **Service**: Application services that coordinate domain operations


## Error Handling

The crate uses a flexible error handling system with generic error types:

- `MemoryError<E>`: Generic error type that can wrap adapter-specific errors
- `ValidationError`: Domain-specific validation errors
- `MemoryResult<T, E>`: Result type for memory operations

## Relationship Naming Convention

All relationships in the knowledge graph follow snake_case naming convention:
- `uses`
- `contains`
- `relates_to`
- `implements`
- etc.
*/

pub mod adapters;
pub mod domain;
pub mod ports;
pub mod service;

// Re-export main types for convenience
pub use adapters::neo4j::{Neo4jConfig, Neo4jRepository};
pub use domain::entity::MemoryEntity;
pub use domain::error::{MemoryError, MemoryResult};
pub use domain::validation_error::ValidationError;
pub use ports::repository::MemoryRepository;
use serde::{Deserialize, Serialize};
pub use service::memory::MemoryService;

/// Configuration options for memory service behavior
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryConfig {
    /// Optional tag automatically added to every created entity
    #[serde(default)]
    pub default_tag: Option<String>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            default_tag: Some("Memory".to_string()),
        }
    }
}

// Re-export neo4rs for use by other crates
pub use neo4rs;
pub type Error = neo4rs::Error;

/// Create a Neo4j-based memory service
///
/// This is a convenience function that creates a Neo4j repository and wraps it in a memory service.
///
/// # Arguments
///
/// * `config` - Configuration for connecting to Neo4j
///
/// # Returns
///
/// A memory service that uses Neo4j as the backend storage
///
/// # Errors
///
/// Returns a `MemoryError` if the connection to Neo4j fails
///
/// # Example
///
/// ```no_run
/// use mm_memory::{Neo4jConfig, create_neo4j_service};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = Neo4jConfig {
///         uri: "neo4j://localhost:7688".to_string(),
///         username: "neo4j".to_string(),
///         password: "password".to_string(),
///     };
///
///     let service = create_neo4j_service(config, MemoryConfig::default()).await?;
///     
///     // Use the service...
///     
///     Ok(())
/// }
/// ```
pub async fn create_neo4j_service(
    config: Neo4jConfig,
    memory_config: MemoryConfig,
) -> Result<MemoryService<Neo4jRepository, neo4rs::Error>, MemoryError<neo4rs::Error>> {
    let repository = Neo4jRepository::new(config).await?;
    Ok(MemoryService::new(repository, memory_config))
}
