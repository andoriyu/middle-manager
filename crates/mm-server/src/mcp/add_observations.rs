use crate::generate_call_tool;
use mm_core::{AddObservationsCommand, add_observations};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "add_observations",
    description = "Add observations to an entity"
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddObservationsTool {
    pub name: String,
    pub observations: Vec<String>,
}

impl AddObservationsTool {
    generate_call_tool!(
        self,
        AddObservationsCommand { name, observations },
        "Observations added to '{}'",
        add_observations
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
        mock.expect_add_observations()
            .withf(|name, obs| name == "test:entity" && obs.len() == 1)
            .returning(|_, _| Ok(()));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = AddObservationsTool {
            name: "test:entity".to_string(),
            observations: vec!["obs".to_string()],
        };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert_eq!(text, "Observations added to 'test:entity'");
    }

    #[tokio::test]
    async fn test_call_tool_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_add_observations()
            .returning(|_, _| Err(MemoryError::runtime_error("fail")));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = AddObservationsTool {
            name: "test:entity".to_string(),
            observations: vec!["obs".to_string()],
        };

        let result = tool.call_tool(&ports).await;
        assert!(result.is_err());
    }
}
