use crate::MemoryEntity;
use crate::error::CoreError;
use crate::neo4rs;
use crate::ports::Ports;
use thiserror::Error;

/// Command to retrieve an entity by name
#[derive(Debug, Clone)]
pub struct GetEntityCommand {
    pub name: String,
}

/// Error types that can occur when getting an entity
#[derive(Debug, Error)]
pub enum GetEntityError {
    #[error("Repository error: {0}")]
    Repository(#[from] CoreError<neo4rs::Error>),

    #[error("Validation error: {0}")]
    Validation(String),
}

/// Result type for the get_entity operation
pub type GetEntityResult = Result<Option<MemoryEntity>, GetEntityError>;

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
pub async fn get_entity(ports: &Ports, command: GetEntityCommand) -> GetEntityResult {
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
        Err(e) => Err(GetEntityError::Repository(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::MockMemoryService;
    use mockall::predicate::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_entity_success() {
        let mut mock = MockMemoryService::<neo4rs::Error>::new();
        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec![],
            properties: HashMap::new(),
        };

        mock.expect_find_entity_by_name()
            .with(eq("test:entity"))
            .returning(move |_| Ok(Some(entity.clone())));

        let ports = Ports::new(Arc::new(mock));
        let command = GetEntityCommand {
            name: "test:entity".to_string(),
        };

        let result = get_entity(&ports, command).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "test:entity");
    }

    #[tokio::test]
    async fn test_get_entity_empty_name() {
        let mut mock = MockMemoryService::<neo4rs::Error>::new();
        mock.expect_find_entity_by_name().never();
        let ports = Ports::new(Arc::new(mock));

        let command = GetEntityCommand {
            name: "".to_string(),
        };

        let result = get_entity(&ports, command).await;
        assert!(matches!(result, Err(GetEntityError::Validation(_))));
    }
}
