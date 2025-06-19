use crate::MemoryEntity;
use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use mm_memory::MemoryRepository;
use tracing::instrument;

/// Command to create a new entity
#[derive(Debug, Clone)]
pub struct CreateEntityCommand {
    pub entities: Vec<MemoryEntity>,
}

/// Result type for the create_entity operation
pub type CreateEntityResult<E> = CoreResult<(), E>;

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
#[instrument(skip(ports), fields(entities_count = command.entities.len()))]
pub async fn create_entity<R>(
    ports: &Ports<R>,
    command: CreateEntityCommand,
) -> CreateEntityResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    let errors = ports
        .memory_service
        .create_entities(&command.entities)
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
    async fn test_create_entity_success() {
        let mut mock_repo = MockMemoryRepository::new();
        mock_repo
            .expect_create_entities()
            .withf(|entities| entities.len() == 1 && entities[0].name == "test:entity")
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock_repo,
            MemoryConfig {
                default_label: None,
                default_labels: false,
                ..MemoryConfig::default()
            },
        );
        let ports = Ports::new(Arc::new(service));

        let command = CreateEntityCommand {
            entities: vec![MemoryEntity {
                name: "test:entity".to_string(),
                labels: vec!["Test".to_string()],
                observations: vec![],
                properties: HashMap::default(),
                relationships: Vec::new(),
            }],
        };

        let result = create_entity(&ports, command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_entity_empty_name() {
        let mut mock_repo = MockMemoryRepository::new();
        mock_repo.expect_create_entities().never();

        let service = MemoryService::new(
            mock_repo,
            MemoryConfig {
                default_label: None,
                default_labels: false,
                ..MemoryConfig::default()
            },
        );
        let ports = Ports::new(Arc::new(service));

        let command = CreateEntityCommand {
            entities: vec![MemoryEntity {
                name: "".to_string(),
                labels: vec!["Test".to_string()],
                observations: vec![],
                properties: HashMap::default(),
                relationships: Vec::new(),
            }],
        };

        let result = create_entity(&ports, command).await;
        assert!(matches!(
            result,
            Err(CoreError::BatchValidation(ref errs)) if errs.iter().any(|(n, e)| n.is_empty() && e.0.contains(&ValidationErrorKind::EmptyEntityName))
        ));
    }

    #[tokio::test]
    async fn test_create_entity_repository_error() {
        use mm_memory::MemoryError;

        let mut mock_repo = MockMemoryRepository::new();
        mock_repo
            .expect_create_entities()
            .withf(|entities| entities.len() == 1 && entities[0].name == "test:entity")
            .returning(|_| Err(MemoryError::runtime_error("db error")));

        let service = MemoryService::new(
            mock_repo,
            MemoryConfig {
                default_label: None,
                default_labels: false,
                ..MemoryConfig::default()
            },
        );
        let ports = Ports::new(Arc::new(service));

        let command = CreateEntityCommand {
            entities: vec![MemoryEntity {
                name: "test:entity".to_string(),
                labels: vec!["Test".to_string()],
                observations: vec![],
                properties: HashMap::default(),
                relationships: Vec::new(),
            }],
        };

        let result = create_entity(&ports, command).await;

        assert!(matches!(result, Err(CoreError::Memory(_))));
    }

    #[tokio::test]
    async fn test_create_entity_multiple_errors() {
        let mut mock_repo = MockMemoryRepository::new();
        mock_repo.expect_create_entities().never();

        let service = MemoryService::new(
            mock_repo,
            MemoryConfig {
                default_label: None,
                default_labels: false,
                ..MemoryConfig::default()
            },
        );
        let ports = Ports::new(Arc::new(service));

        let command = CreateEntityCommand {
            entities: vec![
                MemoryEntity {
                    name: "".to_string(),
                    labels: vec![],
                    observations: vec![],
                    properties: HashMap::default(),
                    relationships: Vec::new(),
                },
                MemoryEntity {
                    name: "valid:entity".to_string(),
                    labels: vec![],
                    observations: vec![],
                    properties: HashMap::default(),
                    relationships: Vec::new(),
                },
            ],
        };

        let result = create_entity(&ports, command).await;

        if let Err(CoreError::BatchValidation(errs)) = result {
            assert_eq!(errs.len(), 2);
            assert!(errs.iter().any(|(n, e)| {
                n.is_empty()
                    && e.0.contains(&ValidationErrorKind::EmptyEntityName)
                    && e.0.contains(&ValidationErrorKind::NoLabels("".to_string()))
            }));
            assert!(errs.iter().any(|(n, e)| {
                n == "valid:entity"
                    && e.0
                        .contains(&ValidationErrorKind::NoLabels("valid:entity".to_string()))
            }));
        } else {
            panic!("Expected batch validation error");
        }
    }
}
