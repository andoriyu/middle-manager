use std::error::Error as StdError;
use std::marker::PhantomData;

use crate::domain::entity::MemoryEntity;
use crate::domain::error::MemoryResult;
use crate::ports::repository::MemoryRepository;

/// Service for memory operations
///
/// This service provides a high-level API for interacting with the memory store.
/// It uses a repository to perform the actual operations, and adds additional
/// business logic as needed.
///
/// The service is generic over the repository type `R` and the error type `E`,
/// allowing it to work with different repository implementations.
pub struct MemoryService<R, E>
where
    R: MemoryRepository<E>,
    E: StdError + Send + Sync + 'static,
{
    /// The repository used to perform memory operations
    repository: R,

    /// Optional default tag to be applied to every created entity
    default_tag: Option<String>,

    /// Phantom data to track the error type
    _error_type: PhantomData<E>,
}

impl<R, E> MemoryService<R, E>
where
    R: MemoryRepository<E>,
    E: StdError + Send + Sync + 'static,
{
    /// Create a new memory service with the given repository
    ///
    /// # Arguments
    ///
    /// * `repository` - The repository to use for memory operations
    ///
    /// # Returns
    ///
    /// A new `MemoryService` that uses the given repository
    pub fn new(repository: R) -> Self {
        Self {
            repository,
            default_tag: Some("Memory".to_string()),
            _error_type: PhantomData,
        }
    }

    /// Create a new memory service with a custom default tag
    pub fn with_default_tag(repository: R, default_tag: Option<String>) -> Self {
        Self {
            repository,
            default_tag,
            _error_type: PhantomData,
        }
    }

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
    pub async fn create_entity(&self, entity: &MemoryEntity) -> MemoryResult<(), E> {
        let mut to_create = entity.clone();
        if let Some(tag) = &self.default_tag {
            if !to_create.labels.contains(tag) {
                to_create.labels.push(tag.clone());
            }
        }
        // Validation is handled in the repository
        self.repository.create_entity(&to_create).await
    }

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
pub async fn find_entity_by_name(&self, name: &str) -> MemoryResult<Option<MemoryEntity>, E> {
        // Validation is handled in the repository
        self.repository.find_entity_by_name(name).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use mockall::predicate::*;
    use crate::ports::repository::MockMemoryRepository;

    #[tokio::test]
    async fn test_default_tag_added() {
        let mut mock = MockMemoryRepository::<std::io::Error>::new();
        mock.expect_create_entity()
            .withf(|e| e.labels.contains(&"Memory".to_string()) && e.labels.contains(&"Test".to_string()))
            .returning(|_| Ok(()));

        let service = MemoryService::new(mock);

        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec![],
            properties: HashMap::new(),
        };

        let result = service.create_entity(&entity).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_default_tag_disabled() {
        let mut mock = MockMemoryRepository::<std::io::Error>::new();
        mock.expect_create_entity()
            .withf(|e| e.labels == vec!["Test".to_string()])
            .returning(|_| Ok(()));

        let service = MemoryService::with_default_tag(mock, None);

        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec![],
            properties: HashMap::new(),
        };

        let result = service.create_entity(&entity).await;
        assert!(result.is_ok());
    }
}
