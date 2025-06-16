use mm_core::{Ports, SetObservationsCommand, set_observations};
use mm_memory::MemoryRepository;
use mm_memory_neo4j::neo4rs;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::CallToolResult;
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::mcp::error::ToolError;

#[mcp_tool(
    name = "set_observations",
    description = "Replace all observations for an entity"
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SetObservationsTool {
    pub name: String,
    pub observations: Vec<String>,
}

impl SetObservationsTool {
    pub async fn call_tool<R>(&self, ports: &Ports<R>) -> Result<CallToolResult, CallToolError>
    where
        R: MemoryRepository<Error = neo4rs::Error> + Send + Sync,
    {
        let command = SetObservationsCommand {
            name: self.name.clone(),
            observations: self.observations.clone(),
        };

        match set_observations(ports, command).await {
            Ok(_) => Ok(CallToolResult::text_content(
                format!("Observations for '{}' replaced", self.name),
                None,
            )),
            Err(e) => {
                error!("Failed to set observations: {:#?}", e);
                let tool_error = ToolError::from(e);
                Err(CallToolError::new(tool_error))
            }
        }
    }
}
