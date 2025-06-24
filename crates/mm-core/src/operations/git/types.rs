use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Properties for GitRepository entities
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct GitRepositoryProperties {
    /// Repository URL (e.g., "https://github.com/andoriyu/middle-manager")
    pub url: String,

    /// Default branch name
    pub default_branch: String,
}
