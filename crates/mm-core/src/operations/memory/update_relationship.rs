use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use crate::validate_name;
use mm_git::GitRepository;
use mm_memory::{MemoryRepository, RelationshipUpdate};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct UpdateRelationshipCommand {
    pub from: String,
    pub to: String,
    pub name: String,
    pub update: RelationshipUpdate,
}

pub type UpdateRelationshipResult<E> = CoreResult<(), E>;

#[instrument(skip(ports), fields(from = %command.from, to = %command.to, name = %command.name))]
pub async fn update_relationship<M, G>(
    ports: &Ports<M, G>,
    command: UpdateRelationshipCommand,
) -> UpdateRelationshipResult<M::Error>
where
    M: MemoryRepository + Send + Sync,
    G: GitRepository + Send + Sync,
    M::Error: std::error::Error + Send + Sync + 'static,
    G::Error: std::error::Error + Send + Sync + 'static,
{
    validate_name!(command.from);
    validate_name!(command.to);

    ports
        .memory_service
        .update_relationship(&command.from, &command.to, &command.name, &command.update)
        .await
        .map_err(CoreError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_git::repository::MockGitRepository;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_update_relationship_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_update_relationship()
            .withf(|f, t, n, _| f == "a" && t == "b" && n == "rel")
            .returning(|_, _, _, _| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));
        let cmd = UpdateRelationshipCommand {
            from: "a".into(),
            to: "b".into(),
            name: "rel".into(),
            update: RelationshipUpdate::default(),
        };
        let res = update_relationship(&ports, cmd).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_update_relationship_empty_from() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_update_relationship().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));
        let cmd = UpdateRelationshipCommand {
            from: "".into(),
            to: "b".into(),
            name: "rel".into(),
            update: RelationshipUpdate::default(),
        };
        let res = update_relationship(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::Validation(_))));
    }
}
