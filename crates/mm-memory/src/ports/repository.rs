use std::error::Error as StdError;

use crate::domain::entity::MemoryEntity;
use crate::domain::error::MemoryResult;

/// Repository interface for memory operations
pub trait MemoryRepository<E>
where
    E: StdError + Send + Sync + 'static
{
    /// Create a new entity in the memory graph
    async fn create_entity(&self, entity: &MemoryEntity) -> MemoryResult<(), E>;
    
    /// Find an entity by name
    async fn find_entity_by_name(&self, name: &str) -> MemoryResult<Option<MemoryEntity>, E>;
}
