use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents the status of a Git repository
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct GitStatus {
    /// Current branch name
    pub branch: String,
    /// Upstream branch if configured
    pub upstream_branch: Option<String>,
    /// Whether the working tree has uncommitted changes
    pub is_dirty: bool,
    /// Number of commits the local branch is ahead of its upstream
    pub ahead_by: u32,
    /// Number of commits the local branch is behind its upstream
    pub behind_by: u32,
    /// Paths of files that have been staged
    pub staged_files: Vec<String>,
    /// Paths of tracked files modified but not staged
    pub modified_files: Vec<String>,
    /// Paths of untracked files
    pub untracked_files: Vec<String>,
    /// Files with merge conflicts
    pub conflicted_files: Vec<String>,
    /// Number of stashes in the repository
    pub stash_count: u32,
}
