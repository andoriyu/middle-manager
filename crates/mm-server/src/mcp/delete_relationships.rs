use mm_core::{DeleteRelationshipsCommand, delete_relationships};
use mm_memory::relationship::RelationshipRef;
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "delete_relationships",
    description = "Delete relationships from the memory graph"
)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeleteRelationshipsTool {
    pub relationships: Vec<RelationshipRef>,
}

impl DeleteRelationshipsTool {
    generate_call_tool!(
        self,
        DeleteRelationshipsCommand { relationships => self.relationships.clone() },
        delete_relationships,
        |_cmd, _res| {
            Ok(rust_mcp_sdk::schema::CallToolResult::text_content(
                "Relationships deleted".to_string(),
                None,
            ))
        }
    );
}
