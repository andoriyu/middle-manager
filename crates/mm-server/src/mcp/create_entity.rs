use mm_core::{create_entity, CreateEntityCommand, Ports};
use mm_memory_neo4j::neo4rs;
use mm_memory::MemoryRepository;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::CallToolResult;
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::error;

use crate::mcp::error::ToolError;

/// MCP tool for creating entities
#[mcp_tool(
    name = "create_entity",
    description = "Create a new entity in the memory graph"
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateEntityTool {
    /// Name of the entity
    pub name: String,

    /// Labels for the entity
    pub labels: Vec<String>,

    /// Observations about the entity
    pub observations: Vec<String>,

    /// Additional properties for the entity
    #[serde(default)]
    pub properties: Option<HashMap<String, String>>,
}

impl CreateEntityTool {
    /// Execute the tool with the given ports
    pub async fn call_tool<R>(&self, ports: &Ports<R>) -> Result<CallToolResult, CallToolError>
    where
        R: MemoryRepository<Error = neo4rs::Error> + Send + Sync,
    {
        // Create command from tool parameters
        let command = CreateEntityCommand {
            name: self.name.clone(),
            labels: self.labels.clone(),
            observations: self.observations.clone(),
            properties: self.properties.clone().unwrap_or_default(),
        };

        // Execute the operation
        match create_entity(ports, command).await {
            Ok(_) => Ok(CallToolResult::text_content(
                format!("Entity '{}' created successfully", self.name),
                None,
            )),
            Err(e) => {
                // Log the detailed error
                error!("Failed to create entity: {:#?}", e);
                // Return a simplified error for the MCP protocol
                let tool_error = ToolError::from(e);
                Err(CallToolError::new(tool_error))
            }
        }
    }
}
