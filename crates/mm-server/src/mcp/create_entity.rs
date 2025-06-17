use crate::generate_call_tool;
use mm_core::{CreateEntityCommand, create_entity};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    generate_call_tool!(
        self,
        CreateEntityCommand {
            name,
            labels,
            observations,
            properties => self.properties.clone().unwrap_or_default()
        },
        create_entity,
        |command| {
            let entity = mm_core::MemoryEntity {
                name: command.name,
                labels: command.labels,
                observations: command.observations,
                properties: command.properties,
            };
            serde_json::to_value(entity)
                .map(|json| rust_mcp_sdk::schema::CallToolResult::text_content(json.to_string(), None))
                .map_err(|e| rust_mcp_sdk::schema::schema_utils::CallToolError::new(crate::mcp::error::ToolError::from(e)))
        }
    );
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
        let expected = serde_json::json!({
            "name": "test:entity",
            "labels": ["Test"],
            "observations": [],
            "properties": {}
        });
        assert_eq!(text, expected.to_string());
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
