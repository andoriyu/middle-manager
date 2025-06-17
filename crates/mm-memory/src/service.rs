use crate::{
    DEFAULT_RELATIONSHIPS, MemoryConfig, MemoryEntity, MemoryRelationship, MemoryRepository,
    MemoryResult, ValidationError, ValidationErrorKind,
};
use mm_utils::is_snake_case;

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
}

impl<R> MemoryService<R>
where
    R: MemoryRepository,
{
    /// Create a new memory service with the given repository
    pub fn new(repository: R, config: MemoryConfig) -> Self {
        Self { repository, config }
    }

    /// Create a new entity in the memory graph
    pub async fn create_entity(&self, entity: &MemoryEntity) -> MemoryResult<(), R::Error> {
        let mut tagged = entity.clone();
        if let Some(tag) = &self.config.default_tag {
            if !tagged.labels.contains(tag) {
                tagged.labels.push(tag.clone());
            }
        }

        let mut errors = Vec::new();
        if tagged.name.is_empty() {
            errors.push(ValidationErrorKind::EmptyEntityName);
        }
        if tagged.labels.is_empty() {
            errors.push(ValidationErrorKind::NoLabels(tagged.name.clone()));
        }

        if !errors.is_empty() {
            return Err(ValidationError(errors).into());
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

    /// Create a relationship between two entities
    pub async fn create_relationship(
        &self,
        relationship: &MemoryRelationship,
    ) -> MemoryResult<(), R::Error> {
        let mut errors = Vec::new();
        if relationship.from.is_empty() || relationship.to.is_empty() {
            errors.push(ValidationErrorKind::EmptyEntityName);
        }

        if !is_snake_case(&relationship.name) {
            errors.push(ValidationErrorKind::InvalidRelationshipFormat(
                relationship.name.clone(),
            ));
        }

        if self.config.default_relationships
            && !DEFAULT_RELATIONSHIPS.contains(&relationship.name.as_str())
            && !self
                .config
                .additional_relationships
                .contains(&relationship.name)
        {
            errors.push(ValidationErrorKind::UnknownRelationship(
                relationship.name.clone(),
            ));
        }

        if !errors.is_empty() {
            return Err(ValidationError(errors).into());
        }

        self.repository.create_relationship(relationship).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockMemoryRepository;
    use crate::ValidationErrorKind;
    use std::collections::{HashMap, HashSet};

    #[tokio::test]
    async fn test_default_tag_added() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entity()
            .withf(|e| e.labels.contains(&"Memory".to_string()))
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: Some("Memory".to_string()),
                default_relationships: true,
                additional_relationships: HashSet::new(),
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

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: None,
                default_relationships: true,
                additional_relationships: HashSet::new(),
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
    async fn test_empty_labels_adds_default_tag() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entity()
            .withf(|e| e.labels.len() == 1 && e.labels.contains(&"Memory".to_string()))
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: Some("Memory".to_string()),
                default_relationships: true,
                additional_relationships: HashSet::new(),
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

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: None,
                default_relationships: true,
                additional_relationships: HashSet::new(),
            },
        );

        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec![],
            observations: vec![],
            properties: std::collections::HashMap::new(),
        };

        let result = service.create_entity(&entity).await;
        assert!(matches!(
            result,
            Err(crate::MemoryError::ValidationError(ref e))
                if e.0.contains(&ValidationErrorKind::NoLabels("test:entity".to_string()))
        ));
    }

    #[tokio::test]
    async fn test_create_relationship_allowed() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationship().returning(|_| Ok(()));
        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: None,
                default_relationships: true,
                additional_relationships: HashSet::new(),
            },
        );

        let rel = MemoryRelationship {
            from: "a".to_string(),
            to: "b".to_string(),
            name: "related_to".to_string(),
            properties: HashMap::new(),
        };

        let result = service.create_relationship(&rel).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_relationship_unknown() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationship().never();
        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: None,
                default_relationships: true,
                additional_relationships: HashSet::new(),
            },
        );

        let rel = MemoryRelationship {
            from: "a".to_string(),
            to: "b".to_string(),
            name: "custom_rel".to_string(),
            properties: HashMap::new(),
        };

        let result = service.create_relationship(&rel).await;
        assert!(matches!(
            result,
            Err(crate::MemoryError::ValidationError(ref e))
                if e.0.contains(&ValidationErrorKind::UnknownRelationship("custom_rel".to_string()))
        ));
    }

    #[tokio::test]
    async fn test_create_entity_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entity()
            .returning(|_| Err(crate::MemoryError::query_error("fail")));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: Some("Memory".to_string()),
                default_relationships: true,
                additional_relationships: HashSet::new(),
            },
        );

        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec![],
            properties: HashMap::new(),
        };

        let result = service.create_entity(&entity).await;
        assert!(matches!(result, Err(crate::MemoryError::QueryError { .. })));
    }

    #[tokio::test]
    async fn test_create_relationship_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationship()
            .returning(|_| Err(crate::MemoryError::query_error("fail")));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_tag: None,
                default_relationships: true,
                additional_relationships: HashSet::new(),
            },
        );

        let rel = MemoryRelationship {
            from: "a".to_string(),
            to: "b".to_string(),
            name: "related_to".to_string(),
            properties: HashMap::new(),
        };

        let result = service.create_relationship(&rel).await;
        assert!(matches!(result, Err(crate::MemoryError::QueryError { .. })));
    }
}
