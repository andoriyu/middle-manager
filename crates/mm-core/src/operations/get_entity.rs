use crate::ports::Ports;
use crate::MemoryEntity;
use crate::error::CoreError;
use crate::neo4rs;
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
        return Err(GetEntityError::Validation("Entity name cannot be empty".to_string()));
    }
    
    // Find entity using the memory service
    match ports.memory_service.find_entity_by_name(&command.name).await {
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
    async fn test_get_entity_validation() {
        let ports = Ports::new(Arc::new(MockMemoryService::<neo4rs::Error>::new()));
        let cmd = GetEntityCommand { name: "".into() };
        let err = get_entity(&ports, cmd).await.unwrap_err();
        matches!(err, GetEntityError::Validation(_));
    }

    #[tokio::test]
    async fn test_get_entity_calls_service() {
        let mut mock = MockMemoryService::<neo4rs::Error>::new();
        let expected = MemoryEntity {
            name: "exists".into(),
            labels: vec!["L".into()],
            observations: vec!["o".into()],
            properties: HashMap::new(),
        };
        mock.expect_find_entity_by_name()
            .with(eq("exists"))
            .returning(move |_| Ok(Some(expected.clone())));

        let ports = Ports::new(Arc::new(mock));
        let cmd = GetEntityCommand { name: "exists".into() };
        let result = get_entity(&ports, cmd).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "exists");
    }
}
