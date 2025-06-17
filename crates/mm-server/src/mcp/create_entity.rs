use mm_core::{CreateEntityCommand, MemoryEntity, create_entity};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "create_entity",
    description = "Create a new entity in the memory graph"
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateEntityTool {
    /// Entities to create
    pub entities: Vec<MemoryEntity>,
}

impl CreateEntityTool {
    #[tracing::instrument(skip(self, ports), fields(entities_count = self.entities.len()))]
    pub async fn call_tool<R>(
        &self,
        ports: &mm_core::Ports<R>,
    ) -> Result<
        rust_mcp_sdk::schema::CallToolResult,
        rust_mcp_sdk::schema::schema_utils::CallToolError,
    >
    where
        R: mm_memory::MemoryRepository + Send + Sync,
        R::Error: std::error::Error + Send + Sync + 'static,
    {
        let command = CreateEntityCommand {
            entities: self.entities.clone(),
        };

        crate::mcp::error::map_result(create_entity(ports, command.clone()).await)?;
        let json = serde_json::to_value(command.entities)
            .map(|json| rust_mcp_sdk::schema::CallToolResult::text_content(json.to_string(), None))
            .map_err(|e| {
                rust_mcp_sdk::schema::schema_utils::CallToolError::new(
                    crate::mcp::error::ToolError::from(e),
                )
            })?;
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::Ports;
    use mm_memory::{MemoryConfig, MemoryError, MemoryService, MockMemoryRepository};
    use std::collections::HashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_call_tool_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities()
            .withf(|ents| ents.len() == 1 && ents[0].name == "test:entity")
            .returning(|_| Ok(()));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = CreateEntityTool {
            entities: vec![MemoryEntity {
                name: "test:entity".to_string(),
                labels: vec!["Test".to_string()],
                observations: vec![],
                properties: HashMap::new(),
            }],
        };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        let expected = serde_json::json!([
            {
                "name": "test:entity",
                "labels": ["Test"],
                "observations": [],
                "properties": {}
            }
        ]);
        assert_eq!(text, expected.to_string());
    }

    #[tokio::test]
    async fn test_call_tool_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities()
            .returning(|_| Err(MemoryError::runtime_error("fail")));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = CreateEntityTool {
            entities: vec![MemoryEntity {
                name: "test:entity".to_string(),
                labels: vec!["Test".to_string()],
                observations: vec![],
                properties: HashMap::new(),
            }],
        };

        let result = tool.call_tool(&ports).await;
        assert!(result.is_err());
    }
}
