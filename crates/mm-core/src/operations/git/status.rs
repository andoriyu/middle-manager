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

pub async fn get_git_status<M, G>(
    ports: &Ports<M, G>,
    command: GetGitStatusCommand,
) -> GetGitStatusResult<G::Error>
where
    M: MemoryRepository + Send + Sync,
    G: GitRepository + Send + Sync,
    G::Error: std::error::Error + Send + Sync + 'static,
{
    // Use the git_service from ports to get the status
    ports
        .git_service
        .get_status(&command.path)
        .await
        .map_err(CoreError::Git)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_git::{GitError, repository::MockGitRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_git_status_success() {
        // Create a mock git repository with expectations
        let mut git_repo = MockGitRepository::new();
        git_repo.expect_get_status().returning(|_| {
            Ok(GitStatus {
                branch: "main".to_string(),
                is_dirty: false,
                ahead_by: 0,
                behind_by: 0,
                changed_files: vec![],
            })
        });

        // Create a Ports instance with the configured mock
        let ports = Ports::noop().with(|ports| {
            ports.git_service = Arc::new(mm_git::GitService::new(git_repo));
        });

        // Call the function under test
        let command = GetGitStatusCommand {
            path: PathBuf::from("/fake/path"),
        };
        let result = get_git_status(&ports, command).await;

        // Assert the result
        assert!(result.is_ok());
        let status = result.unwrap();
        assert_eq!(status.branch, "main");
        assert!(!status.is_dirty);
        assert_eq!(status.ahead_by, 0);
        assert_eq!(status.behind_by, 0);
        assert!(status.changed_files.is_empty());
    }

    #[tokio::test]
    async fn test_get_git_status_error() {
        // Create a mock git repository with expectations
        let mut git_repo = MockGitRepository::new();
        git_repo
            .expect_get_status()
            .returning(|_| Err(GitError::repository_error("Repository not found")));

        // Create a Ports instance with the configured mock
        let ports = Ports::noop().with(|ports| {
            ports.git_service = Arc::new(mm_git::GitService::new(git_repo));
        });

        // Call the function under test
        let command = GetGitStatusCommand {
            path: PathBuf::from("/fake/path"),
        };
        let result = get_git_status(&ports, command).await;

        // Assert the result
        assert!(
            matches!(result, Err(CoreError::Git(_))),
            "Expected Git error"
        );
    }
}
