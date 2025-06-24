use crate::MemoryEntity;
use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use crate::validate_name;
use mm_git::GitServiceTrait;
use mm_memory::MemoryRepository;
use tracing::instrument;

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
#[instrument(skip(ports), fields(name = %command.name))]
pub async fn get_entity<R, G>(
    ports: &Ports<R, G>,
    command: GetEntityCommand,
) -> GetEntityResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
    G: GitServiceTrait + Send + Sync,
{
    // Validate command
    validate_name!(command.name);

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
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository, ValidationErrorKind};
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_entity_success() {
        let mut mock_repo = MockMemoryRepository::new();
        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            ..Default::default()
        };

        mock_repo
            .expect_find_entity_by_name()
            .with(eq("test:entity"))
            .returning(move |_| Ok(Some(entity.clone())));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service), Arc::new(mm_git::NoopGitService));
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
        let ports = Ports::new(Arc::new(service), Arc::new(mm_git::NoopGitService));

        let command = GetEntityCommand {
            name: "".to_string(),
        };

        let result = get_entity(&ports, command).await;
        assert!(matches!(
            result,
            Err(CoreError::Validation(ref e)) if e.0.contains(&ValidationErrorKind::EmptyEntityName)
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
        let ports = Ports::new(Arc::new(service), Arc::new(mm_git::NoopGitService));

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
        let ports = Ports::new(Arc::new(service), Arc::new(mm_git::NoopGitService));

        let command = GetEntityCommand {
            name: "missing:entity".to_string(),
        };

        let result = get_entity(&ports, command).await.unwrap();
        assert!(result.is_none());
    }

    use arbitrary::Arbitrary;
    use mm_utils::prop::{NonEmptyName, async_arbtest};

    #[test]
    fn prop_get_entity_success() {
        async_arbtest(|rt, u| {
            let NonEmptyName(name) = NonEmptyName::arbitrary(u)?;
            let mut mock_repo = MockMemoryRepository::new();
            let name_clone = name.clone();
            mock_repo
                .expect_find_entity_by_name()
                .withf(move |n| n == name_clone)
                .returning(|_| Ok(None));
            let service = MemoryService::new(mock_repo, MemoryConfig::default());
            let ports = Ports::new(Arc::new(service), Arc::new(mm_git::NoopGitService));
            let command = GetEntityCommand { name };
            let result = rt.block_on(get_entity(&ports, command));
            assert!(result.is_ok());
            Ok(())
        });
    }

    #[test]
    fn prop_get_entity_empty_name() {
        async_arbtest(|rt, _| {
            let mut mock_repo = MockMemoryRepository::new();
            mock_repo.expect_find_entity_by_name().never();
            let service = MemoryService::new(mock_repo, MemoryConfig::default());
            let ports = Ports::new(Arc::new(service), Arc::new(mm_git::NoopGitService));
            let command = GetEntityCommand {
                name: String::default(),
            };
            let result = rt.block_on(get_entity(&ports, command));
            assert!(matches!(result, Err(CoreError::Validation(_))));
            Ok(())
        });
    }
}
