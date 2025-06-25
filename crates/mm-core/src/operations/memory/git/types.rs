use mm_memory::value::MemoryValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Properties for GitRepository entities
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct GitRepositoryProperties {
    /// Repository URL (e.g., "https://github.com/andoriyu/middle-manager")
    pub url: String,

    /// Default branch name
    pub default_branch: String,
}

impl From<HashMap<String, MemoryValue>> for GitRepositoryProperties {
    fn from(mut map: HashMap<String, MemoryValue>) -> Self {
        let url = match map.remove("url") {
            Some(MemoryValue::String(s)) => s,
            _ => String::new(),
        };
        let default_branch = match map.remove("default_branch") {
            Some(MemoryValue::String(s)) => s,
            _ => String::new(),
        };
        GitRepositoryProperties {
            url,
            default_branch,
        }
    }
}

impl From<GitRepositoryProperties> for HashMap<String, MemoryValue> {
    fn from(props: GitRepositoryProperties) -> Self {
        let mut map = HashMap::new();
        map.insert("url".to_string(), MemoryValue::String(props.url));
        map.insert(
            "default_branch".to_string(),
            MemoryValue::String(props.default_branch),
        );
        map
    }
}
