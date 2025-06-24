use std::path::Path;

use crate::error::GitResult;
use crate::repository::GitRepository;
use crate::status::GitStatus;

/// Service for Git operations
pub struct GitService<R>
where
    R: GitRepository,
{
    /// The repository used to perform Git operations
    repository: R,
}

impl<R> GitService<R>
where
    R: GitRepository + Sync,
{
    /// Create a new Git service with the given repository
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Get the status of a Git repository
    pub async fn get_status(&self, path: &Path) -> GitResult<GitStatus, R::Error> {
        self.repository.get_status(path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::MockGitRepository;

    #[tokio::test]
    async fn test_get_status() {
        let mut mock = MockGitRepository::new();
        mock.expect_get_status()
            .withf(|p| p.to_str() == Some("/tmp"))
            .returning(|_| {
                Ok(GitStatus {
                    branch: "main".to_string(),
                })
            });

        let service = GitService::new(mock);
        let status = service.get_status(Path::new("/tmp")).await.unwrap();
        assert_eq!(status.branch, "main");
    }
}
