use mm_core::{RemoveObservationsCommand, remove_observations};
use rust_mcp_sdk::macros::mcp_tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "remove_observations",
    description = "Remove specific observations from an entity"
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RemoveObservationsTool {
    pub name: String,
    pub observations: Vec<String>,
}

impl RemoveObservationsTool {
    pub fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        serde_json::to_value(schemars::schema_for!(Self))
            .expect("schema serialization")
            .as_object()
            .cloned()
            .expect("schema object")
    }

    generate_call_tool!(
        self,
        RemoveObservationsCommand { name, observations },
        "Observations removed from '{}'",
        remove_observations
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
        mock.expect_remove_observations()
            .withf(|name, obs| name == "test:entity" && obs.len() == 1)
            .returning(|_, _| Ok(()));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = RemoveObservationsTool {
            name: "test:entity".to_string(),
            observations: vec!["obs".to_string()],
        };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert_eq!(text, "Observations removed from 'test:entity'");
    }

    #[tokio::test]
    async fn test_call_tool_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_remove_observations()
            .returning(|_, _| Err(MemoryError::runtime_error("fail")));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = RemoveObservationsTool {
            name: "test:entity".to_string(),
            observations: vec!["obs".to_string()],
        };

        let result = tool.call_tool(&ports).await;
        assert!(result.is_err());
    }
}
