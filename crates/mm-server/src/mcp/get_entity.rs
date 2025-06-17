use crate::mcp::error::{ToolError, map_result};
use mm_core::{GetEntityCommand, Ports, get_entity};
use mm_memory::MemoryRepository;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::CallToolResult;
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use serde::{Deserialize, Serialize};
use serde_json;

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
    pub async fn call_tool<R>(&self, ports: &Ports<R>) -> Result<CallToolResult, CallToolError>
    where
        R: MemoryRepository + Send + Sync,
        R::Error: std::error::Error + Send + Sync + 'static,
    {
        // Create command from tool parameters
        let command = GetEntityCommand {
            name: self.name.clone(),
        };

        // Execute the operation
        map_result(get_entity(ports, command).await).and_then(|maybe_entity| match maybe_entity {
            Some(entity) => serde_json::to_value(entity)
                .map(|json| CallToolResult::text_content(json.to_string(), None))
                .map_err(|e| CallToolError::new(ToolError::from(e))),
            None => Ok(CallToolResult::text_content(
                format!("Entity '{}' not found", self.name),
                None,
            )),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::Ports;
    use mm_memory::{MemoryConfig, MemoryEntity, MemoryError, MemoryService, MockMemoryRepository};
    use mockall::predicate::*;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_call_tool_success() {
        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec![],
            properties: HashMap::new(),
        };

        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entity_by_name()
            .with(eq("test:entity"))
            .returning(move |_| Ok(Some(entity.clone())));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = GetEntityTool {
            name: "test:entity".to_string(),
        };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        let value: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["name"], "test:entity");
    }

    #[tokio::test]
    async fn test_call_tool_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entity_by_name()
            .returning(|_| Err(MemoryError::query_error("fail")));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = GetEntityTool {
            name: "test:entity".to_string(),
        };

        let result = tool.call_tool(&ports).await;
        assert!(result.is_err());
    }
}
