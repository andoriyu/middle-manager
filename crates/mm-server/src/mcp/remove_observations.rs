use crate::mcp::error::map_result;
use mm_core::{Ports, RemoveObservationsCommand, remove_observations};
use mm_memory::MemoryRepository;
use mm_memory_neo4j::neo4rs;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::CallToolResult;
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use serde::{Deserialize, Serialize};

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

        map_result(remove_observations(ports, command).await).map(|_| {
            CallToolResult::text_content(format!("Observations removed from '{}'", self.name), None)
        })
    }
}
