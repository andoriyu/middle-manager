use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use mm_memory::{MemoryRelationship, MemoryRepository};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CreateRelationshipCommand {
    pub from: String,
    pub to: String,
    pub name: String,
    pub properties: HashMap<String, String>,
}

pub type CreateRelationshipResult<E> = CoreResult<(), E>;

pub async fn create_relationship<R>(
    ports: &Ports<R>,
    command: CreateRelationshipCommand,
) -> CreateRelationshipResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    let rel = MemoryRelationship {
        from: command.from,
        to: command.to,
        name: command.name,
        properties: command.properties,
    };

    ports
        .memory_service
        .create_relationship(&rel)
        .await
        .map_err(CoreError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create_relationship_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationship().returning(|_| Ok(()));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = CreateRelationshipCommand {
            from: "a".to_string(),
            to: "b".to_string(),
            name: "related_to".to_string(),
            properties: HashMap::new(),
        };

        let result = create_relationship(&ports, command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_relationship_empty_name() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationship().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = CreateRelationshipCommand {
            from: "a".to_string(),
            to: "b".to_string(),
            name: "".to_string(),
            properties: HashMap::new(),
        };

        let result = create_relationship(&ports, command).await;
        assert!(matches!(result, Err(CoreError::Memory(_))));
    }

    #[tokio::test]
    async fn test_create_relationship_invalid_format() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationship().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = CreateRelationshipCommand {
            from: "a".to_string(),
            to: "b".to_string(),
            name: "InvalidFormat".to_string(),
            properties: HashMap::new(),
        };

        let result = create_relationship(&ports, command).await;
        assert!(matches!(
            result,
            Err(CoreError::Memory(mm_memory::MemoryError::ValidationError(ref e)))
                if e.0.contains(&mm_memory::ValidationErrorKind::InvalidRelationshipFormat("InvalidFormat".to_string()))
        ));
    }

    #[tokio::test]
    async fn test_create_relationship_unknown_relationship() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationship().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = CreateRelationshipCommand {
            from: "a".to_string(),
            to: "b".to_string(),
            name: "custom_rel".to_string(),
            properties: HashMap::new(),
        };

        let result = create_relationship(&ports, command).await;
        assert!(matches!(
            result,
            Err(CoreError::Memory(mm_memory::MemoryError::ValidationError(ref e)))
                if e.0.contains(&mm_memory::ValidationErrorKind::UnknownRelationship("custom_rel".to_string()))
        ));
    }
}
