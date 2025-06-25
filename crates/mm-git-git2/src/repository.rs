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
            let repo = Repository::open(path)?;
            let head = repo.head()?;
            let branch_name = head
                .shorthand()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "HEAD".to_string());

            let mut opts = git2::StatusOptions::new();
            opts.include_untracked(true).recurse_untracked_dirs(true);
            let statuses = repo.statuses(Some(&mut opts))?;
            let is_dirty = !statuses.is_empty();
            let changed_files = statuses
                .iter()
                .filter_map(|e| e.path().map(|p| p.to_string()))
                .collect::<Vec<_>>();

            let (ahead, behind) =
                if let Ok(branch) = repo.find_branch(&branch_name, git2::BranchType::Local) {
                    if let Ok(upstream) = branch.upstream() {
                        let local_oid = branch.get().target().unwrap_or_else(git2::Oid::zero);
                        let upstream_oid = upstream.get().target().unwrap_or_else(git2::Oid::zero);
                        repo.graph_ahead_behind(local_oid, upstream_oid)?
                    } else {
                        (0, 0)
                    }
                } else {
                    (0, 0)
                };

            Ok(GitStatus {
                branch: branch_name,
                is_dirty,
                ahead_by: ahead as u32,
                behind_by: behind as u32,
                changed_files,
            })
        })
        .await
        .map_err(|e| GitError::repository_error(format!("Task join error: {e}")))?;

        res.map_err(|e| GitError::repository_error_with_source("Git operation failed", e))
    }
}
