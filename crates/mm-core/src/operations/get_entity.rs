use crate::MemoryEntity;
use crate::error::CoreError;
use crate::ports::Ports;
use mm_memory::MemoryRepository;
use thiserror::Error;

/// Command to retrieve an entity by name
#[derive(Debug, Clone)]
pub struct GetEntityCommand {
    pub name: String,
}

/// Error types that can occur when getting an entity
#[derive(Debug, Error)]
pub enum GetEntityError<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    #[error("Repository error: {0}")]
    Repository(#[from] CoreError<E>),

    #[error("Validation error: {0}")]
    Validation(String),
}

/// Result type for the get_entity operation
pub type GetEntityResult<E> = Result<Option<MemoryEntity>, GetEntityError<E>>;

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
        return Err(GetEntityError::Validation(
            "Entity name cannot be empty".to_string(),
        ));
    }

    // Find entity using the memory service
    match ports
        .memory_service
        .find_entity_by_name(&command.name)
        .await
    {
        Ok(entity) => Ok(entity),
        Err(e) => Err(GetEntityError::Repository(CoreError::from(e))),
    }
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
        assert!(matches!(result, Err(GetEntityError::Validation(_))));
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

        assert!(matches!(
            result,
            Err(GetEntityError::Repository(CoreError::Memory(_)))
        ));
    }
}
