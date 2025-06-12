use async_trait::async_trait;
use std::error::Error as StdError;

#[cfg(test)]
use mockall::automock;

use crate::domain::entity::MemoryEntity;
use crate::domain::error::MemoryResult;

/// Repository interface for memory operations
///
/// This trait defines the interface for interacting with the memory store.
/// It is generic over the error type `E` to allow for different error types
/// in different implementations.
///
/// Implementations of this trait should handle the details of interacting
/// with the specific memory store technology (e.g., Neo4j, MongoDB, etc.).
#[cfg_attr(test, automock)]
#[async_trait]
pub trait MemoryRepository<E>
where
    E: StdError + Send + Sync + 'static,
{
    /// Create a new entity in the memory graph
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to create
    ///
    /// # Returns
    ///
    /// `Ok(())` if the entity was created successfully, or an error if something went wrong
    ///
    /// # Errors
    ///
    /// Returns a `MemoryError` if:
    /// - The entity name is empty
    /// - The entity has no labels
    /// - There was an error connecting to the memory store
    /// - There was an error executing the query
    async fn create_entity(&self, entity: &MemoryEntity) -> MemoryResult<(), E>;

    /// Find an entity by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to find
    ///
    /// # Returns
    ///
    /// `Ok(Some(entity))` if the entity was found, `Ok(None)` if the entity was not found,
    /// or an error if something went wrong
    ///
    /// # Errors
    ///
    /// Returns a `MemoryError` if:
    /// - The name is empty
    /// - There was an error connecting to the memory store
    /// - There was an error executing the query
    async fn find_entity_by_name(&self, name: &str) -> MemoryResult<Option<MemoryEntity>, E>;
}
