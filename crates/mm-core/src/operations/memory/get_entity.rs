#[cfg(test)]
use crate::error::CoreError;
#[cfg(test)]
use mm_memory::MemoryEntity;

generate_get_wrapper!(
    GetEntityCommand,
    get_entity,
    GetEntityResult,
    mm_memory::BasicEntityProperties
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::Ports;
    use mm_git::repository::MockGitRepository;
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
            .expect_find_entity_by_name_typed::<mm_memory::BasicEntityProperties>()
            .with(eq("test:entity"))
            .returning(move |_| Ok(Some(entity.clone())));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));
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
        mock_repo
            .expect_find_entity_by_name_typed::<mm_memory::BasicEntityProperties>()
            .never();
        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));

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
            .expect_find_entity_by_name_typed::<mm_memory::BasicEntityProperties>()
            .with(eq("test:entity"))
            .returning(|_| Err(MemoryError::query_error("db error")));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));

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
            .expect_find_entity_by_name_typed::<mm_memory::BasicEntityProperties>()
            .with(eq("missing:entity"))
            .returning(|_| Ok(None));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));

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
                .expect_find_entity_by_name_typed::<mm_memory::BasicEntityProperties>()
                .withf(move |n| n == name_clone)
                .returning(|_| Ok(None));
            let service = MemoryService::new(mock_repo, MemoryConfig::default());
            let git_repo = MockGitRepository::new();
            let git_service = mm_git::GitService::new(git_repo);
            let ports = Ports::new(Arc::new(service), Arc::new(git_service));
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
            mock_repo
                .expect_find_entity_by_name_typed::<mm_memory::BasicEntityProperties>()
                .never();
            let service = MemoryService::new(mock_repo, MemoryConfig::default());
            let git_repo = MockGitRepository::new();
            let git_service = mm_git::GitService::new(git_repo);
            let ports = Ports::new(Arc::new(service), Arc::new(git_service));
            let command = GetEntityCommand {
                name: String::default(),
            };
            let result = rt.block_on(get_entity(&ports, command));
            assert!(matches!(result, Err(CoreError::Validation(_))));
            Ok(())
        });
    }
}
