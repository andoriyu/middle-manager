#[cfg(test)]
use crate::error::CoreError;

generate_delete_wrapper!(DeleteTaskCommand, delete_task, DeleteTaskResult);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::Ports;
    use mm_git::repository::MockGitRepository;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_delete_task_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_delete_entities()
            .withf(|n| n.len() == 1 && n[0] == "task:1")
            .returning(|_| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));
        let cmd = DeleteTaskCommand {
            name: "task:1".into(),
        };
        let res = delete_task(&ports, cmd).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_delete_task_empty_name() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_delete_entities().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));
        let cmd = DeleteTaskCommand {
            name: String::new(),
        };
        let res = delete_task(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::Validation(_))));
    }
}
