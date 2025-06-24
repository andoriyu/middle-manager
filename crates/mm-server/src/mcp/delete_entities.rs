use mm_core::operations::memory::{DeleteEntitiesCommand, delete_entities};
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "delete_entities",
    description = "Delete entities from the memory graph"
)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeleteEntitiesTool {
    pub names: Vec<String>,
}

impl DeleteEntitiesTool {
    generate_call_tool!(
        self,
        DeleteEntitiesCommand { names => self.names.clone() },
        delete_entities,
        |_cmd, _res| {
            Ok(rust_mcp_sdk::schema::CallToolResult::text_content(
                "Entities deleted".to_string(),
                None,
            ))
        }
    );
}
