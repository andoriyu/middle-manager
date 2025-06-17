use crate::generate_call_tool;
use mm_core::{GetEntityCommand, get_entity};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use serde::{Deserialize, Serialize};

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
    generate_call_tool!(
        self,
        GetEntityCommand { name },
        "Entity '{}' retrieved",
        get_entity
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::Ports;
    use mm_memory::{MemoryConfig, MemoryEntity, MemoryError, MemoryService, MockMemoryRepository};
    use mockall::predicate::*;
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
        assert_eq!(text, "Entity 'test:entity' retrieved");
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

    #[tokio::test]
    async fn test_call_tool_not_found() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entity_by_name()
            .with(eq("missing"))
            .returning(|_| Ok(None));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = GetEntityTool {
            name: "missing".to_string(),
        };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert_eq!(text, "Entity 'missing' retrieved");
    }
}
