use mm_core::{Ports, RemoveAllObservationsCommand, remove_all_observations};
use mm_memory::MemoryRepository;
use mm_memory_neo4j::neo4rs;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::CallToolResult;
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::mcp::error::ToolError;

#[mcp_tool(
    name = "remove_all_observations",
    description = "Remove all observations from an entity"
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RemoveAllObservationsTool {
    pub name: String,
}

impl RemoveAllObservationsTool {
    pub async fn call_tool<R>(&self, ports: &Ports<R>) -> Result<CallToolResult, CallToolError>
    where
        R: MemoryRepository<Error = neo4rs::Error> + Send + Sync,
    {
        let command = RemoveAllObservationsCommand {
            name: self.name.clone(),
        };

        match remove_all_observations(ports, command).await {
            Ok(_) => Ok(CallToolResult::text_content(
                format!("All observations removed from '{}'", self.name),
                None,
            )),
            Err(e) => {
                error!("Failed to remove observations: {:#?}", e);
                let tool_error = ToolError::from(e);
                Err(CallToolError::new(tool_error))
            }
        }
    }
}
