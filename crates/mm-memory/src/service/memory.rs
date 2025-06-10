use std::error::Error as StdError;
use std::marker::PhantomData;

use crate::domain::entity::MemoryEntity;
use crate::domain::error::MemoryResult;
use crate::ports::repository::MemoryRepository;

/// Service for memory operations
pub struct MemoryService<R, E>
where
    R: MemoryRepository<E>,
    E: StdError + Send + Sync + 'static
{
    repository: R,
    _error_type: PhantomData<E>,
}

impl<R, E> MemoryService<R, E>
where
    R: MemoryRepository<E>,
    E: StdError + Send + Sync + 'static
{
    /// Create a new memory service with the given repository
    pub fn new(repository: R) -> Self {
        Self { 
            repository,
            _error_type: PhantomData,
        }
    }
    
    /// Create a new entity in the memory graph
    pub async fn create_entity(&self, entity: &MemoryEntity) -> MemoryResult<(), E> {
        // Validation is now handled in the repository
        self.repository.create_entity(entity).await
    }
    
    /// Find an entity by name
    pub async fn find_entity_by_name(&self, name: &str) -> MemoryResult<Option<MemoryEntity>, E> {
        // Validation is now handled in the repository
        self.repository.find_entity_by_name(name).await
    }
}
