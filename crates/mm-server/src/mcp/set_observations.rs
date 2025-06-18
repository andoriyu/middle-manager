use mm_core::{SetObservationsCommand, set_observations};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "set_observations",
    description = "Replace all observations for an entity"
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SetObservationsTool {
    pub name: String,
    pub observations: Vec<String>,
}

use arbitrary::{Arbitrary, Unstructured};
use mm_utils::prop::NonEmptyName;

impl<'a> Arbitrary<'a> for SetObservationsTool {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let NonEmptyName(name) = NonEmptyName::arbitrary(u)?;
        let observations = <Vec<String>>::arbitrary(u)?;
        Ok(Self { name, observations })
    }
}

impl SetObservationsTool {
    generate_call_tool!(
        self,
        SetObservationsCommand { name, observations },
        "Observations for '{}' replaced",
        set_observations
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
        mock.expect_set_observations()
            .withf(|name, obs| name == "test:entity" && obs.len() == 1)
            .returning(|_, _| Ok(()));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = SetObservationsTool {
            name: "test:entity".to_string(),
            observations: vec!["obs".to_string()],
        };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert_eq!(text, "Observations for 'test:entity' replaced");
    }

    #[tokio::test]
    async fn test_call_tool_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_set_observations()
            .returning(|_, _| Err(MemoryError::runtime_error("fail")));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = SetObservationsTool {
            name: "test:entity".to_string(),
            observations: vec!["obs".to_string()],
        };

        let result = tool.call_tool(&ports).await;
        assert!(result.is_err());
    }
}
