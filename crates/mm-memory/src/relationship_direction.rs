use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Direction to traverse relationships when querying related entities
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum RelationshipDirection {
    /// From source to target
    Outgoing,
    /// From target to source
    Incoming,
    /// Either direction
    Both,
}
