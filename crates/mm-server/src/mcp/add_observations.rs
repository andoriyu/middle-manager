use mm_core::{AddObservationsCommand, Ports, add_observations};
use mm_memory::MemoryRepository;
use mm_memory_neo4j::neo4rs;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::CallToolResult;
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::mcp::error::ToolError;

#[mcp_tool(
    name = "add_observations",
    description = "Add observations to an entity"
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddObservationsTool {
    pub name: String,
    pub observations: Vec<String>,
}

impl AddObservationsTool {
    pub async fn call_tool<R>(&self, ports: &Ports<R>) -> Result<CallToolResult, CallToolError>
    where
        R: MemoryRepository<Error = neo4rs::Error> + Send + Sync,
    {
        let command = AddObservationsCommand {
            name: self.name.clone(),
            observations: self.observations.clone(),
        };

        match add_observations(ports, command).await {
            Ok(_) => Ok(CallToolResult::text_content(
                format!("Observations added to '{}'", self.name),
                None,
            )),
            Err(e) => {
                error!("Failed to add observations: {:#?}", e);
                let tool_error = ToolError::from(e);
                Err(CallToolError::new(tool_error))
            }
        }
    }
}
