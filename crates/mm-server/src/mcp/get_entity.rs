use mm_core::operations::memory::{GetEntityCommand, get_entity};
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

/// MCP tool for retrieving entities
#[mcp_tool(
    name = "get_entity",
    description = "Get an entity from the memory graph by name"
)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetEntityTool {
    /// Name of the entity to retrieve
    pub name: String,
}

impl GetEntityTool {
    generate_call_tool!(self, GetEntityCommand { name }, get_entity);
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::Ports;
    use mm_memory::{MemoryConfig, MemoryEntity, MemoryError, MemoryService, MockMemoryRepository};
    use mockall::predicate::*;
    use serde_json::Value;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_call_tool_success() {
        let entity = MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            ..Default::default()
        };

        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entity_by_name()
            .with(eq("test:entity"))
            .returning(move |_| Ok(Some(entity.clone())));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        let tool = GetEntityTool {
            name: "test:entity".to_string(),
        };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        let value: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["name"], "test:entity");
        assert!(value["relationships"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_call_tool_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entity_by_name()
            .returning(|_| Err(MemoryError::query_error("fail")));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        let tool = GetEntityTool {
            name: "test:entity".to_string(),
        };

        let result = tool.call_tool(&ports).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_call_tool_not_found() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entity_by_name()
            .with(eq("missing"))
            .returning(|_| Ok(None));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        let tool = GetEntityTool {
            name: "missing".to_string(),
        };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert_eq!(text, "null");
    }
}
