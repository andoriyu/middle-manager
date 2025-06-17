use mm_core::{Ports, RemoveAllObservationsCommand, remove_all_observations};
use mm_memory::MemoryRepository;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::CallToolResult;
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::mcp::error::ToolError;

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
        R: MemoryRepository + Send + Sync,
        R::Error: std::error::Error + Send + Sync + 'static,
    {
        let command = RemoveAllObservationsCommand {
            name: self.name.clone(),
        };

        match remove_all_observations(ports, command).await {
            Ok(_) => Ok(CallToolResult::text_content(
                format!("All observations removed from '{}'", self.name),
                None,
            )),
            Err(e) => {
                error!("Failed to remove observations: {:#?}", e);
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
        mock.expect_remove_all_observations()
            .withf(|name| name == "test:entity")
            .returning(|_| Ok(()));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = RemoveAllObservationsTool {
            name: "test:entity".to_string(),
        };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert_eq!(text, "All observations removed from 'test:entity'");
    }

    #[tokio::test]
    async fn test_call_tool_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_remove_all_observations()
            .withf(|name| name == "test:entity")
            .returning(|_| Err(MemoryError::runtime_error("fail")));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = RemoveAllObservationsTool {
            name: "test:entity".to_string(),
        };

        let result = tool.call_tool(&ports).await;
        assert!(result.is_err());
    }
}
