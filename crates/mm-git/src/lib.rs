use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    /// Generic repository error
    #[error("Repository error: {message}")]
    RepositoryError {
        message: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
}

pub type GitResult<T> = Result<T, GitError>;

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryStatus {
    pub current_branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_url: Option<String>,
    pub is_dirty: bool,
    pub untracked_files: Vec<PathBuf>,
    pub modified_files: Vec<PathBuf>,
    pub staged_files: Vec<PathBuf>,
}

pub trait GitRepository: Send + Sync {
    fn get_status(&self, path: &Path) -> GitResult<RepositoryStatus>;
}

pub struct GitService<R: GitRepository> {
    repository: R,
}

impl<R: GitRepository> GitService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn get_status(&self, path: &Path) -> GitResult<RepositoryStatus> {
        self.repository.get_status(path)
    }
}

pub trait GitServiceTrait: Send + Sync {
    fn get_status(&self, path: &Path) -> GitResult<RepositoryStatus>;
}

impl<R: GitRepository> GitServiceTrait for GitService<R> {
    fn get_status(&self, path: &Path) -> GitResult<RepositoryStatus> {
        self.get_status(path)
    }
}

/// No-op Git service used when no real Git integration is configured.
#[derive(Debug, Clone, Default)]
pub struct NoopGitService;

impl GitServiceTrait for NoopGitService {
    fn get_status(&self, _path: &Path) -> GitResult<RepositoryStatus> {
        Err(GitError::RepositoryError {
            message: "Git service not configured".into(),
            source: None,
        })
    }
}
