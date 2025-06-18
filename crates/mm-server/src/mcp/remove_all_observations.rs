use mm_core::{RemoveAllObservationsCommand, remove_all_observations};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "remove_all_observations",
    description = "Remove all observations from an entity"
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RemoveAllObservationsTool {
    pub name: String,
}

use arbitrary::{Arbitrary, Unstructured};
use mm_utils::prop::NonEmptyName;

impl<'a> Arbitrary<'a> for RemoveAllObservationsTool {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let NonEmptyName(name) = NonEmptyName::arbitrary(u)?;
        Ok(Self { name })
    }
}

impl RemoveAllObservationsTool {
    generate_call_tool!(
        self,
        RemoveAllObservationsCommand { name },
        "All observations removed from '{}'",
        remove_all_observations
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
