use mm_core::{
    Ports, GetEntityCommand, get_entity,
};
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use rust_mcp_sdk::schema::CallToolResult;
use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json;
use tracing::{error};

use crate::mcp::error::ToolError;

/// MCP tool for retrieving entities
#[mcp_tool(
    name = "get_entity",
    description = "Get an entity from the memory graph by name"
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetEntityTool {
    /// Name of the entity to retrieve
    pub name: String,
}

impl GetEntityTool {
    /// Execute the tool with the given ports
    pub async fn call_tool(&self, ports: &Ports) -> Result<CallToolResult, CallToolError> {
        // Create command from tool parameters
        let command = GetEntityCommand {
            name: self.name.clone(),
        };
        
        // Execute the operation
        match get_entity(ports, command).await {
            Ok(Some(entity)) => {
                match serde_json::to_value(entity) {
                    Ok(json) => Ok(CallToolResult::text_content(json.to_string(), None)),
                    Err(e) => {
                        error!("Failed to serialize entity: {:#?}", e);
                        let tool_error = ToolError::from(e);
                        Err(CallToolError::new(tool_error))
                    }
                }
            },
            Ok(None) => {
                Ok(CallToolResult::text_content(
                    format!("Entity '{}' not found", self.name),
                    None
                ))
            },
            Err(e) => {
                // Log the detailed error
                error!("Failed to get entity: {:#?}", e);
                // Return a simplified error for the MCP protocol
                let tool_error = ToolError::from(e);
                Err(CallToolError::new(tool_error))
            },
        }
    }
}
