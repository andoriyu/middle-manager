use std::path::PathBuf;

use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use mm_git::{GitRepository, GitStatus};
use mm_memory::MemoryRepository;

#[derive(Debug, Clone)]
pub struct GetGitStatusCommand {
    pub path: PathBuf,
}

pub type GetGitStatusResult<E> = CoreResult<GitStatus, E>;

pub async fn get_git_status<MR, GR>(
    ports: &Ports<MR, GR>,
    command: GetGitStatusCommand,
) -> GetGitStatusResult<GR::Error>
where
    MR: MemoryRepository + Send + Sync,
    GR: GitRepository + Send + Sync,
    GR::Error: std::error::Error + Send + Sync + 'static,
{
    let path = &command.path;

    ports
        .git_service
        .get_status(path)
        .await
        .map_err(CoreError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_git::GitService;
    use mm_git::repository::MockGitRepository;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_git_status_success() {
        let mut mock = MockGitRepository::new();
        mock.expect_get_status().returning(|_| {
            Ok(GitStatus {
                branch: "main".to_string(),
            })
        });
        let git_service = Arc::new(GitService::new(mock));
        let memory_service = Arc::new(MemoryService::new(
            MockMemoryRepository::new(),
            MemoryConfig::default(),
        ));
        let ports = Ports::new(memory_service, git_service.clone());

        let command = GetGitStatusCommand {
            path: PathBuf::from("/tmp"),
        };

        let status = get_git_status(&ports, command).await.unwrap();
        assert_eq!(status.branch, "main");
    }
}
