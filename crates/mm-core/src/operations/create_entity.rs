use crate::MemoryEntity;
use crate::error::CoreError;
use crate::neo4rs;
use crate::ports::Ports;
use std::collections::HashMap;
use thiserror::Error;

/// Command to create a new entity
#[derive(Debug, Clone)]
pub struct CreateEntityCommand {
    pub name: String,
    pub labels: Vec<String>,
    pub observations: Vec<String>,
    pub properties: HashMap<String, String>,
}

/// Error types that can occur when creating an entity
#[derive(Debug, Error)]
pub enum CreateEntityError {
    #[error("Repository error: {0}")]
    Repository(#[from] CoreError<neo4rs::Error>),

    #[error("Validation error: {0}")]
    Validation(String),
}

/// Result type for the create_entity operation
pub type CreateEntityResult = Result<(), CreateEntityError>;

/// Create a new entity
///
/// # Arguments
///
/// * `ports` - The ports containing required services
/// * `command` - The command containing the entity details
///
/// # Returns
///
/// Ok(()) if the entity was created successfully, or an error
pub async fn create_entity(ports: &Ports, command: CreateEntityCommand) -> CreateEntityResult {
    // Validate command
    if command.name.is_empty() {
        return Err(CreateEntityError::Validation(
            "Entity name cannot be empty".to_string(),
        ));
    }

    if command.labels.is_empty() {
        return Err(CreateEntityError::Validation(
            "Entity must have at least one label".to_string(),
        ));
    }

    // Create entity using the memory service
    let entity = MemoryEntity {
        name: command.name,
        labels: command.labels,
        observations: command.observations,
        properties: command.properties,
    };

    match ports.memory_service.create_entity(&entity).await {
        Ok(_) => Ok(()),
        Err(e) => Err(CreateEntityError::Repository(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::MockMemoryService;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create_entity_success() {
        let mut mock = MockMemoryService::<neo4rs::Error>::new();
        mock.expect_create_entity()
            .withf(|e| e.name == "test:entity")
            .returning(|_| Ok(()));

        let ports = Ports::new(Arc::new(mock));

        let command = CreateEntityCommand {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec![],
            properties: HashMap::new(),
        };

        let result = create_entity(&ports, command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_entity_empty_name() {
        let mut mock = MockMemoryService::<neo4rs::Error>::new();
        mock.expect_create_entity().never();

        let ports = Ports::new(Arc::new(mock));

        let command = CreateEntityCommand {
            name: "".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec![],
            properties: HashMap::new(),
        };

        let result = create_entity(&ports, command).await;
        assert!(matches!(result, Err(CreateEntityError::Validation(_))));
    }
}
