use mm_git::{GitServiceTrait, RepositoryStatus};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;

/// Command for retrieving git status
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetGitStatusCommand {
    /// Path to the repository (optional, defaults to current directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

pub type GetGitStatusResult<E> = CoreResult<RepositoryStatus, E>;

/// Get git repository status
#[instrument(skip(ports), err)]
pub async fn get_git_status<R, G>(
    ports: &Ports<R, G>,
    command: GetGitStatusCommand,
) -> GetGitStatusResult<R::Error>
where
    R: mm_memory::MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
    G: GitServiceTrait + Send + Sync,
{
    let service = &ports.git_service;

    let path = command
        .path
        .as_deref()
        .map(std::path::Path::new)
        .unwrap_or_else(|| std::path::Path::new("."));

    service.get_status(path).map_err(CoreError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_git::{GitError, GitRepository, GitService, RepositoryStatus};
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use std::path::Path;
    use std::sync::Arc;

    struct MockGitRepo;
    impl GitRepository for MockGitRepo {
        fn get_status(&self, _path: &Path) -> Result<RepositoryStatus, GitError> {
            Ok(RepositoryStatus::default())
        }
    }

    #[tokio::test]
    async fn test_get_git_status() {
        let mock_repo = MockMemoryRepository::new();
        let memory_service = MemoryService::new(mock_repo, MemoryConfig::default());
        let git_service = GitService::new(MockGitRepo);
        let ports = Ports::new(Arc::new(memory_service), Arc::new(git_service));

        let cmd = GetGitStatusCommand { path: None };
        let res = get_git_status(&ports, cmd).await;
        assert!(res.is_ok());
    }
}
