use crate::mcp::error::map_result;
use mm_core::{CreateRelationshipCommand, Ports, create_relationship};
use mm_memory::MemoryRepository;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::CallToolResult;
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[mcp_tool(
    name = "create_relationship",
    description = "Create a relationship between two entities"
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateRelationshipTool {
    pub from: String,
    pub to: String,
    pub name: String,
    #[serde(default)]
    pub properties: Option<HashMap<String, String>>,
}

impl CreateRelationshipTool {
    pub async fn call_tool<R>(&self, ports: &Ports<R>) -> Result<CallToolResult, CallToolError>
    where
        R: MemoryRepository + Send + Sync,
        R::Error: std::error::Error + Send + Sync + 'static,
    {
        let command = CreateRelationshipCommand {
            from: self.from.clone(),
            to: self.to.clone(),
            name: self.name.clone(),
            properties: self.properties.clone().unwrap_or_default(),
        };

        map_result(create_relationship(ports, command).await).map(|_| {
            CallToolResult::text_content(format!("Relationship '{}' created", self.name), None)
        })
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
        mock.expect_create_relationship().returning(|_| Ok(()));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = CreateRelationshipTool {
            from: "a".to_string(),
            to: "b".to_string(),
            name: "related_to".to_string(),
            properties: None,
        };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert_eq!(text, "Relationship 'related_to' created");
    }

    #[tokio::test]
    async fn test_call_tool_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationship()
            .returning(|_| Err(MemoryError::runtime_error("fail")));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = CreateRelationshipTool {
            from: "a".to_string(),
            to: "b".to_string(),
            name: "related_to".to_string(),
            properties: None,
        };

        let result = tool.call_tool(&ports).await;
        assert!(result.is_err());
    }
}
