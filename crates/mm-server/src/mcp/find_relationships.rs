use mm_core::operations::memory::{FindRelationshipsCommand, find_relationships};
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "find_relationships",
    description = "Find relationships between entities"
)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FindRelationshipsTool {
    pub from: Option<String>,
    pub to: Option<String>,
    pub name: Option<String>,
}

impl FindRelationshipsTool {
    generate_call_tool!(
        self,
        FindRelationshipsCommand {
            from => self.from.clone(),
            to => self.to.clone(),
            name => self.name.clone()
        },
        find_relationships
    );
}
