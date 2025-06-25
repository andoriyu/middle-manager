use super::common::handle_batch_result;
use crate::error::CoreResult;
use crate::ports::Ports;
use mm_git::GitRepository;
use mm_memory::MemoryEntity;
use mm_memory::MemoryRepository;
use tracing::instrument;

/// Command to create new entities
#[derive(Debug, Clone)]
pub struct CreateEntitiesCommand {
    pub entities: Vec<MemoryEntity>,
}

/// Result type for the create_entities operation
pub type CreateEntitiesResult<E> = CoreResult<(), E>;

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
pub async fn create_entities<M, G>(
    ports: &Ports<M, G>,
    command: CreateEntitiesCommand,
) -> CreateEntitiesResult<M::Error>
where
    M: MemoryRepository + Send + Sync,
    G: GitRepository + Send + Sync,
    M::Error: std::error::Error + Send + Sync + 'static,
    G::Error: std::error::Error + Send + Sync + 'static,
{
    handle_batch_result(|| ports.memory_service.create_entities(&command.entities)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CoreError;
    use mm_git::repository::MockGitRepository;
    use mm_memory::ValidationErrorKind;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create_entities_success() {
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
                agent_name: "test".to_string(),
                ..MemoryConfig::default()
            },
        );
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));

        let command = CreateEntitiesCommand {
            entities: vec![MemoryEntity {
                name: "test:entity".to_string(),
                labels: vec!["Test".to_string()],
                ..Default::default()
            }],
        };

        let result = create_entities(&ports, command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_entities_empty_name() {
        let mut mock_repo = MockMemoryRepository::new();
        mock_repo.expect_create_entities().never();

        let service = MemoryService::new(
            mock_repo,
            MemoryConfig {
                default_label: None,
                default_labels: false,
                agent_name: "test".to_string(),
                ..MemoryConfig::default()
            },
        );
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));

        let command = CreateEntitiesCommand {
            entities: vec![MemoryEntity {
                name: "".to_string(),
                labels: vec!["Test".to_string()],
                ..Default::default()
            }],
        };

        let result = create_entities(&ports, command).await;
        assert!(matches!(
            result,
            Err(CoreError::BatchValidation(ref errs)) if errs.iter().any(|(n, e)| n.is_empty() && e.0.contains(&ValidationErrorKind::EmptyEntityName))
        ));
    }

    #[tokio::test]
    async fn test_create_entities_repository_error() {
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
                agent_name: "test".to_string(),
                ..MemoryConfig::default()
            },
        );
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));

        let command = CreateEntitiesCommand {
            entities: vec![MemoryEntity {
                name: "test:entity".to_string(),
                labels: vec!["Test".to_string()],
                ..Default::default()
            }],
        };

        let result = create_entities(&ports, command).await;

        assert!(matches!(result, Err(CoreError::Memory(_))));
    }

    #[tokio::test]
    async fn test_create_entities_multiple_errors() {
        let mut mock_repo = MockMemoryRepository::new();
        mock_repo.expect_create_entities().never();

        let service = MemoryService::new(
            mock_repo,
            MemoryConfig {
                default_label: None,
                default_labels: false,
                agent_name: "test".to_string(),
                ..MemoryConfig::default()
            },
        );
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));

        let command = CreateEntitiesCommand {
            entities: vec![
                MemoryEntity {
                    name: "".to_string(),
                    ..Default::default()
                },
                MemoryEntity {
                    name: "valid:entity".to_string(),
                    ..Default::default()
                },
            ],
        };

        let result = create_entities(&ports, command).await;

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
            unreachable!("Expected batch validation error");
        }
    }
}
