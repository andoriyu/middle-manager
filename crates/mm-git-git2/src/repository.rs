use async_trait::async_trait;
use git2::Repository;
use mm_git::{GitError, GitRepository, GitResult, GitStatus};
use std::path::{Path, PathBuf};
use tokio::task;

pub struct Git2Repository;

impl Git2Repository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Git2Repository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GitRepository for Git2Repository {
    type Error = git2::Error;

    async fn get_status(&self, path: &Path) -> GitResult<GitStatus, Self::Error> {
        let path: PathBuf = path.to_path_buf();
        let res = task::spawn_blocking(move || -> Result<GitStatus, git2::Error> {
            let repo = Repository::discover(path)?;
            let head = repo.head()?;
            let branch = head
                .shorthand()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "HEAD".to_string());
            Ok(GitStatus { branch })
        })
        .await
        .map_err(|e| GitError::repository_error(format!("Task join error: {e}")))?;

        res.map_err(|e| GitError::repository_error_with_source("Git operation failed", e))
    }
}
