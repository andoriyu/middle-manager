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
            let branch_name = head
                .shorthand()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "HEAD".to_string());
            let upstream_branch = repo
                .find_branch(&branch_name, git2::BranchType::Local)
                .ok()
                .and_then(|branch| {
                    branch
                        .upstream()
                        .ok()
                        .and_then(|u| u.name().ok().flatten().map(|s| s.to_string()))
                });

            let mut opts = git2::StatusOptions::new();
            opts.include_untracked(true).recurse_untracked_dirs(true);
            let statuses = repo.statuses(Some(&mut opts))?;
            let is_dirty = !statuses.is_empty();
            let mut staged_files = Vec::new();
            let mut modified_files = Vec::new();
            let mut untracked_files = Vec::new();
            let mut conflicted_files = Vec::new();

            for entry in statuses.iter() {
                if let Some(path) = entry.path() {
                    let path = path.to_string();
                    let status = entry.status();
                    if status.contains(git2::Status::CONFLICTED) {
                        conflicted_files.push(path.clone());
                    }
                    if status.intersects(
                        git2::Status::INDEX_NEW
                            | git2::Status::INDEX_MODIFIED
                            | git2::Status::INDEX_DELETED
                            | git2::Status::INDEX_RENAMED
                            | git2::Status::INDEX_TYPECHANGE,
                    ) {
                        staged_files.push(path.clone());
                    }
                    if status.intersects(
                        git2::Status::WT_MODIFIED
                            | git2::Status::WT_DELETED
                            | git2::Status::WT_RENAMED
                            | git2::Status::WT_TYPECHANGE,
                    ) {
                        modified_files.push(path.clone());
                    }
                    if status.contains(git2::Status::WT_NEW) {
                        untracked_files.push(path.clone());
                    }
                }
            }

            let mut stash_count = 0u32;
            repo.stash_foreach(|_, _, _| {
                stash_count += 1;
                true
            })?;

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
                upstream_branch,
                is_dirty,
                ahead_by: ahead as u32,
                behind_by: behind as u32,
                staged_files,
                modified_files,
                untracked_files,
                conflicted_files,
                stash_count,
            })
        })
        .await
        .map_err(|e| GitError::repository_error(format!("Task join error: {e}")))?;

        res.map_err(|e| GitError::repository_error_with_source("Git operation failed", e))
    }
}
