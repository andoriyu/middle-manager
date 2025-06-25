#[cfg(test)]
use crate::error::CoreError;
#[cfg(test)]
use mm_memory::EntityUpdate;

generate_update_wrapper!(UpdateEntityCommand, update_entity, UpdateEntityResult);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::Ports;
    use mm_git::repository::MockGitRepository;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_update_entity_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_update_entity()
            .withf(|n, _| n == "test:entity")
            .returning(|_, _| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));
        let cmd = UpdateEntityCommand {
            name: "test:entity".into(),
            update: EntityUpdate::default(),
        };
        let res = update_entity(&ports, cmd).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_update_entity_empty_name() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_update_entity().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));
        let cmd = UpdateEntityCommand {
            name: "".into(),
            update: EntityUpdate::default(),
        };
        let res = update_entity(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::Validation(_))));
    }
}
