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
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_get_status() {
        let mut mock = MockGitRepository::new();
        let expected = GitStatus {
            branch: "main".to_string(),
            is_dirty: false,
            ahead_by: 0,
            behind_by: 0,
            changed_files: vec![],
        };
        mock.expect_get_status()
            .withf(|p| p == Path::new("/tmp/repo"))
            .returning(|_| {
                Ok(GitStatus {
                    branch: "main".to_string(),
                    is_dirty: false,
                    ahead_by: 0,
                    behind_by: 0,
                    changed_files: vec![],
                })
            });

        let service = GitService::new(mock);
        let path = PathBuf::from("/tmp/repo");
        let status = service.get_status(&path).await.unwrap();
        assert_eq!(status.branch, expected.branch);
        assert_eq!(status.is_dirty, expected.is_dirty);
        assert_eq!(status.ahead_by, expected.ahead_by);
        assert_eq!(status.behind_by, expected.behind_by);
        assert_eq!(status.changed_files, expected.changed_files);
    }
}
