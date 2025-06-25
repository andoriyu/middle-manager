use super::types::TaskProperties;
#[cfg(test)]
use crate::error::CoreError;
use crate::error::CoreResult;
use crate::operations::memory::generic::get_entity_generic;
use crate::ports::Ports;
use mm_git::GitRepository;
use mm_memory::{MemoryEntity, MemoryRepository};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct GetTaskCommand {
    pub name: String,
}

pub type GetTaskResult<E> = CoreResult<Option<MemoryEntity<TaskProperties>>, E>;

#[instrument(skip(ports), fields(name = %command.name))]
pub async fn get_task<M, G>(ports: &Ports<M, G>, command: GetTaskCommand) -> GetTaskResult<M::Error>
where
    M: MemoryRepository + Send + Sync,
    G: GitRepository + Send + Sync,
    M::Error: std::error::Error + Send + Sync + 'static,
    G::Error: std::error::Error + Send + Sync + 'static,
{
    get_entity_generic::<M, G, TaskProperties>(ports, &command.name).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_git::repository::MockGitRepository;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_task_success() {
        let mut mock = MockMemoryRepository::new();
        let entity = MemoryEntity {
            name: "task:1".into(),
            labels: vec!["Task".into()],
            ..Default::default()
        };
        mock.expect_find_entity_by_name()
            .with(eq("task:1"))
            .returning(move |_| Ok(Some(entity.clone())));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));

        let cmd = GetTaskCommand {
            name: "task:1".into(),
        };
        let res = get_task(&ports, cmd).await.unwrap();
        assert!(res.is_some());
    }

    #[tokio::test]
    async fn test_get_task_empty_name() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entity_by_name().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));

        let cmd = GetTaskCommand {
            name: String::new(),
        };
        let res = get_task(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::Validation(_))));
    }
}
