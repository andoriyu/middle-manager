use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents the status of a Git repository
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct GitStatus {
    /// Current branch name
    pub branch: String,
}
