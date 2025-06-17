use crate::MemoryEntity;
use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use mm_memory::{MemoryRepository, ValidationError};

/// Command to retrieve an entity by name
#[derive(Debug, Clone)]
pub struct GetEntityCommand {
    pub name: String,
}

/// Error types that can occur when getting an entity
/// Result type for the get_entity operation
pub type GetEntityResult<E> = CoreResult<Option<MemoryEntity>, E>;

/// Get an entity by name
///
/// # Arguments
///
/// * `ports` - The ports containing required services
/// * `command` - The command containing the entity name to retrieve
///
/// # Returns
///
/// The entity if found, or None if not found
pub async fn get_entity<R>(ports: &Ports<R>, command: GetEntityCommand) -> GetEntityResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    // Validate command
    if command.name.is_empty() {
        return Err(CoreError::Validation(ValidationError::EmptyEntityName));
    }

    // Find entity using the memory service
    ports
        .memory_service
        .find_entity_by_name(&command.name)
        .await
        .map_err(CoreError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use mockall::predicate::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_entity_success() {
        let mut mock_repo = MockMemoryRepository::new();
        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec![],
            properties: HashMap::new(),
        };

        mock_repo
            .expect_find_entity_by_name()
            .with(eq("test:entity"))
            .returning(move |_| Ok(Some(entity.clone())));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let command = GetEntityCommand {
            name: "test:entity".to_string(),
        };

        let result = get_entity(&ports, command).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "test:entity");
    }

    #[tokio::test]
    async fn test_get_entity_empty_name() {
        let mut mock_repo = MockMemoryRepository::new();
        mock_repo.expect_find_entity_by_name().never();
        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = GetEntityCommand {
            name: "".to_string(),
        };

        let result = get_entity(&ports, command).await;
        assert!(matches!(
            result,
            Err(CoreError::Validation(ValidationError::EmptyEntityName))
        ));
    }

    #[tokio::test]
    async fn test_get_entity_repository_error() {
        use mm_memory::MemoryError;

        let mut mock_repo = MockMemoryRepository::new();
        mock_repo
            .expect_find_entity_by_name()
            .with(eq("test:entity"))
            .returning(|_| Err(MemoryError::query_error("db error")));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = GetEntityCommand {
            name: "test:entity".to_string(),
        };

        let result = get_entity(&ports, command).await;

        assert!(matches!(result, Err(CoreError::Memory(_))));
    }

    #[tokio::test]
    async fn test_get_entity_not_found() {
        let mut mock_repo = MockMemoryRepository::new();
        mock_repo
            .expect_find_entity_by_name()
            .with(eq("missing:entity"))
            .returning(|_| Ok(None));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = GetEntityCommand {
            name: "missing:entity".to_string(),
        };

        let result = get_entity(&ports, command).await.unwrap();
        assert!(result.is_none());
    }
}
