use async_trait::async_trait;
use std::error::Error as StdError;

use mm_memory::{MemoryEntity, MemoryRepository};
use mm_memory_neo4j::MemoryService as MemoryServiceImpl;
#[cfg(test)]
use mockall::automock;

use crate::error::{CoreError, CoreResult};

/// Memory service trait for mm-core
#[cfg_attr(test, automock)]
#[async_trait]
pub trait MemoryService<E>
where
    E: StdError + Send + Sync + 'static,
{
    /// Create a new entity in the memory graph
    async fn create_entity(&self, entity: &MemoryEntity) -> CoreResult<(), E>;

    /// Find an entity by name
    async fn find_entity_by_name(&self, name: &str) -> CoreResult<Option<MemoryEntity>, E>;
}

// Implement MemoryService directly for the MemoryServiceImpl from mm-memory-neo4j
#[async_trait]
impl<R, E> MemoryService<E> for MemoryServiceImpl<R, E>
where
    R: MemoryRepository<E> + Sync,
    E: StdError + Send + Sync + 'static,
{
    async fn create_entity(&self, entity: &MemoryEntity) -> CoreResult<(), E> {
        self.create_entity(entity).await.map_err(CoreError::from)
    }

    async fn find_entity_by_name(&self, name: &str) -> CoreResult<Option<MemoryEntity>, E> {
        self.find_entity_by_name(name)
            .await
            .map_err(CoreError::from)
    }
}
