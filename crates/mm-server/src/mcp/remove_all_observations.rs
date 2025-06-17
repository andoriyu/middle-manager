use crate::mcp::error::map_result;
use mm_core::{Ports, RemoveAllObservationsCommand, remove_all_observations};
use mm_memory::MemoryRepository;
use mm_memory_neo4j::neo4rs;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::CallToolResult;
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use serde::{Deserialize, Serialize};

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

        map_result(remove_all_observations(ports, command).await).map(|_| {
            CallToolResult::text_content(
                format!("All observations removed from '{}'", self.name),
                None,
            )
        })
    }
}
