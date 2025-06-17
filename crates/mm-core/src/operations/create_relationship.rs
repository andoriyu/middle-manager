use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use mm_memory::{MemoryRelationship, MemoryRepository};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct CreateRelationshipCommand {
    pub relationships: Vec<MemoryRelationship>,
}

pub type CreateRelationshipResult<E> = CoreResult<(), E>;

#[instrument(skip(ports), fields(relationships_count = command.relationships.len()))]
pub async fn create_relationship<R>(
    ports: &Ports<R>,
    command: CreateRelationshipCommand,
) -> CreateRelationshipResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    let errors = ports
        .memory_service
        .create_relationships(&command.relationships)
        .await
        .map_err(CoreError::from)?;

    if !errors.is_empty() {
        return Err(CoreError::BatchValidation(errors));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_memory::ValidationErrorKind;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use std::collections::HashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create_relationship_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationships()
            .withf(|rels| rels.len() == 1 && rels[0].name == "related_to")
            .returning(|_| Ok(()));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = CreateRelationshipCommand {
            relationships: vec![MemoryRelationship {
                from: "a".to_string(),
                to: "b".to_string(),
                name: "related_to".to_string(),
                properties: HashMap::new(),
            }],
        };

        let result = create_relationship(&ports, command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_relationship_empty_name() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationships().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = CreateRelationshipCommand {
            relationships: vec![MemoryRelationship {
                from: "a".to_string(),
                to: "b".to_string(),
                name: "".to_string(),
                properties: HashMap::new(),
            }],
        };

        let result = create_relationship(&ports, command).await;
        assert!(matches!(
            result,
            Err(CoreError::BatchValidation(ref errs)) if errs.iter().any(|(n, e)| n.is_empty() && e.0.contains(&ValidationErrorKind::UnknownRelationship("".to_string())))
        ));
    }

    #[tokio::test]
    async fn test_create_relationship_invalid_format() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationships().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = CreateRelationshipCommand {
            relationships: vec![MemoryRelationship {
                from: "a".to_string(),
                to: "b".to_string(),
                name: "InvalidFormat".to_string(),
                properties: HashMap::new(),
            }],
        };

        let result = create_relationship(&ports, command).await;
        assert!(matches!(
            result,
            Err(CoreError::BatchValidation(ref errs)) if errs.iter().any(|(n, e)| n == "InvalidFormat" && e.0.contains(&ValidationErrorKind::InvalidRelationshipFormat("InvalidFormat".to_string())))
        ));
    }

    #[tokio::test]
    async fn test_create_relationship_unknown_relationship() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationships().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = CreateRelationshipCommand {
            relationships: vec![MemoryRelationship {
                from: "a".to_string(),
                to: "b".to_string(),
                name: "custom_rel".to_string(),
                properties: HashMap::new(),
            }],
        };

        let result = create_relationship(&ports, command).await;
        assert!(matches!(
            result,
            Err(CoreError::BatchValidation(ref errs)) if errs.iter().any(|(n, e)| n == "custom_rel" && e.0.contains(&ValidationErrorKind::UnknownRelationship("custom_rel".to_string())))
        ));
    }

    #[tokio::test]
    async fn test_create_relationship_multiple_errors() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationships().never();

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = CreateRelationshipCommand {
            relationships: vec![
                MemoryRelationship {
                    from: "a".to_string(),
                    to: "b".to_string(),
                    name: "Invalid".to_string(),
                    properties: HashMap::new(),
                },
                MemoryRelationship {
                    from: "c".to_string(),
                    to: "d".to_string(),
                    name: "".to_string(),
                    properties: HashMap::new(),
                },
            ],
        };

        let result = create_relationship(&ports, command).await;

        if let Err(CoreError::BatchValidation(errs)) = result {
            assert_eq!(errs.len(), 2);
            assert!(errs.iter().any(|(n, e)| {
                n == "Invalid"
                    && e.0
                        .contains(&ValidationErrorKind::InvalidRelationshipFormat(
                            "Invalid".to_string(),
                        ))
            }));
            assert!(errs.iter().any(|(n, e)| {
                n.is_empty()
                    && e.0
                        .contains(&ValidationErrorKind::UnknownRelationship("".to_string()))
            }));
        } else {
            panic!("Expected batch validation error");
        }
    }
}
