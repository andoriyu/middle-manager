use git2::{Repository, Status, StatusOptions};
use mm_git::{GitError, GitRepository, RepositoryStatus};
use std::path::{Path, PathBuf};

pub struct LibGit2Repository;

impl Default for LibGit2Repository {
    fn default() -> Self {
        Self
    }
}

impl LibGit2Repository {
    pub fn new() -> Self {
        Self
    }
}

impl GitRepository for LibGit2Repository {
    fn get_status(&self, path: &Path) -> Result<RepositoryStatus, GitError> {
        let repo = Repository::open(path).map_err(|e| GitError::RepositoryError {
            message: e.message().to_string(),
            source: None,
        })?;
        let head = repo.head().ok();
        let branch = head
            .as_ref()
            .and_then(|h| h.shorthand())
            .unwrap_or("HEAD")
            .to_string();

        let remote_url = repo
            .find_remote("origin")
            .ok()
            .and_then(|r| r.url().map(|u| u.to_string()));

        let mut opts = StatusOptions::new();
        opts.include_untracked(true).recurse_untracked_dirs(true);
        let statuses = repo
            .statuses(Some(&mut opts))
            .map_err(|e| GitError::RepositoryError {
                message: e.message().to_string(),
                source: None,
            })?;

        let mut untracked = Vec::new();
        let mut modified = Vec::new();
        let mut staged = Vec::new();
        let mut is_dirty = false;

        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry.path().map(PathBuf::from).unwrap_or_default();
            if status.contains(Status::WT_NEW) {
                is_dirty = true;
                untracked.push(path.clone());
            }
            if status.intersects(
                Status::WT_MODIFIED
                    | Status::WT_DELETED
                    | Status::WT_RENAMED
                    | Status::WT_TYPECHANGE,
            ) {
                is_dirty = true;
                modified.push(path.clone());
            }
            if status.intersects(
                Status::INDEX_MODIFIED
                    | Status::INDEX_NEW
                    | Status::INDEX_DELETED
                    | Status::INDEX_RENAMED
                    | Status::INDEX_TYPECHANGE,
            ) {
                is_dirty = true;
                staged.push(path);
            }
        }

        Ok(RepositoryStatus {
            current_branch: branch,
            remote_url,
            is_dirty,
            untracked_files: untracked,
            modified_files: modified,
            staged_files: staged,
        })
    }
}
