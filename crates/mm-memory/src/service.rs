use std::marker::PhantomData;

use crate::{MemoryConfig, MemoryEntity, MemoryRepository, MemoryResult, ValidationError};

/// Service for memory operations
///
/// This service provides a high-level API for interacting with the memory store.
/// It uses a repository to perform the actual operations, and adds additional
/// business logic as needed.
///
/// The service is generic over the repository type `R`, allowing it to work with
/// different repository implementations.
pub struct MemoryService<R>
where
    R: MemoryRepository,
{
    /// The repository used to perform memory operations
    repository: R,

    /// Configuration for the service
    config: MemoryConfig,

    /// Phantom data to track the error type
    _error_type: PhantomData<R::Error>,
}

impl<R> MemoryService<R>
where
    R: MemoryRepository,
{
    /// Create a new memory service with the given repository
    pub fn new(repository: R, config: MemoryConfig) -> Self {
        Self {
            repository,
            config,
            _error_type: PhantomData,
        }
    }

    /// Create a new entity in the memory graph
    pub async fn create_entity(&self, entity: &MemoryEntity) -> MemoryResult<(), R::Error> {
        let mut tagged = entity.clone();
        if let Some(tag) = &self.config.default_tag {
            if !tagged.labels.contains(tag) {
                tagged.labels.push(tag.clone());
            }
        }

        if tagged.labels.is_empty() {
            return Err(ValidationError::NoLabels(tagged.name.clone()).into());
        }

        self.repository.create_entity(&tagged).await
    }

    /// Find an entity by name
    pub async fn find_entity_by_name(
        &self,
        name: &str,
    ) -> MemoryResult<Option<MemoryEntity>, R::Error> {
        self.repository.find_entity_by_name(name).await
    }

    /// Replace all observations for an entity
    pub async fn set_observations(
        &self,
        name: &str,
        observations: &[String],
    ) -> MemoryResult<(), R::Error> {
        self.repository.set_observations(name, observations).await
    }

    /// Add observations to an entity
    pub async fn add_observations(
        &self,
        name: &str,
        observations: &[String],
    ) -> MemoryResult<(), R::Error> {
        self.repository.add_observations(name, observations).await
    }

    /// Remove all observations from an entity
    pub async fn remove_all_observations(&self, name: &str) -> MemoryResult<(), R::Error> {
        self.repository.remove_all_observations(name).await
    }

    /// Remove specific observations from an entity
    pub async fn remove_observations(
        &self,
        name: &str,
        observations: &[String],
    ) -> MemoryResult<(), R::Error> {
        self.repository
            .remove_observations(name, observations)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockMemoryRepository;

    #[tokio::test]
    async fn test_default_tag_added() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entity()
            .withf(|e| e.labels.contains(&"Memory".to_string()))
            .returning(|_| Ok(()));
        mock.expect_find_entity_by_name().returning(|_| Ok(None));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: Some("Memory".to_string()),
            },
        );
        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec![],
            properties: std::collections::HashMap::new(),
        };

        service.create_entity(&entity).await.unwrap();
    }

    #[tokio::test]
    async fn test_no_default_tag() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entity()
            .withf(|e| !e.labels.contains(&"Memory".to_string()))
            .returning(|_| Ok(()));
        mock.expect_find_entity_by_name().returning(|_| Ok(None));

        let service = MemoryService::new(mock, MemoryConfig { default_tag: None });
        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec![],
            properties: std::collections::HashMap::new(),
        };

        service.create_entity(&entity).await.unwrap();
    }

    #[tokio::test]
    async fn test_empty_labels_adds_default_tag() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entity()
            .withf(|e| e.labels.len() == 1 && e.labels.contains(&"Memory".to_string()))
            .returning(|_| Ok(()));
        mock.expect_find_entity_by_name().returning(|_| Ok(None));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: Some("Memory".to_string()),
            },
        );

        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec![],
            observations: vec![],
            properties: std::collections::HashMap::new(),
        };

        let result = service.create_entity(&entity).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_empty_labels_without_default_tag_fails() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entity().never();
        mock.expect_find_entity_by_name().returning(|_| Ok(None));

        let service = MemoryService::new(mock, MemoryConfig { default_tag: None });

        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec![],
            observations: vec![],
            properties: std::collections::HashMap::new(),
        };

        let result = service.create_entity(&entity).await;
        assert!(matches!(
            result,
            Err(crate::MemoryError::ValidationError(
                ValidationError::NoLabels(_)
            ))
        ));
    }
}
