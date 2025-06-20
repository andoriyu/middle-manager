use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Mode for matching entity labels in queries
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum LabelMatchMode {
    /// Entity must have ANY of the specified labels
    Any,
    /// Entity must have ALL of the specified labels
    All,
}
