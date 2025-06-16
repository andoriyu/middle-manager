use mm_core::{Ports, RemoveObservationsCommand, remove_observations};
use mm_memory::MemoryRepository;
use mm_memory_neo4j::neo4rs;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::CallToolResult;
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::mcp::error::ToolError;

#[mcp_tool(
    name = "remove_observations",
    description = "Remove specific observations from an entity"
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RemoveObservationsTool {
    pub name: String,
    pub observations: Vec<String>,
}

impl RemoveObservationsTool {
    pub async fn call_tool<R>(&self, ports: &Ports<R>) -> Result<CallToolResult, CallToolError>
    where
        R: MemoryRepository<Error = neo4rs::Error> + Send + Sync,
    {
        let command = RemoveObservationsCommand {
            name: self.name.clone(),
            observations: self.observations.clone(),
        };

        match remove_observations(ports, command).await {
            Ok(_) => Ok(CallToolResult::text_content(
                format!("Observations removed from '{}'", self.name),
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
