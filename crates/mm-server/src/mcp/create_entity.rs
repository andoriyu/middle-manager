use mm_core::{CreateEntityCommand, Ports, create_entity};
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
        R: MemoryRepository + Send + Sync,
        R::Error: std::error::Error + Send + Sync + 'static,
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

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::Ports;
    use mm_memory::{MemoryConfig, MemoryError, MemoryService, MockMemoryRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_call_tool_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entity()
            .withf(|e| e.name == "test:entity")
            .returning(|_| Ok(()));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = CreateEntityTool {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec![],
            properties: None,
        };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert_eq!(text, "Entity 'test:entity' created successfully");
    }

    #[tokio::test]
    async fn test_call_tool_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entity()
            .returning(|_| Err(MemoryError::runtime_error("fail")));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = CreateEntityTool {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec![],
            properties: None,
        };

        let result = tool.call_tool(&ports).await;
        assert!(result.is_err());
    }
}
