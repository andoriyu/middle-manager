use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents the status of a Git repository
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct GitStatus {
    /// Current branch name
    pub branch: String,
    /// Whether the working tree has uncommitted changes
    pub is_dirty: bool,
    /// Number of commits the local branch is ahead of its upstream
    pub ahead_by: u32,
    /// Number of commits the local branch is behind its upstream
    pub behind_by: u32,
    /// Paths of files that have been modified
    pub changed_files: Vec<String>,
}
