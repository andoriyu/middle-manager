use mm_core::operations::memory::{CreateEntitiesCommand, create_entities};
use mm_memory::MemoryEntity;
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "create_entities",
    description = "Create new entities in the memory graph"
)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateEntitiesTool {
    /// Entities to create
    pub entities: Vec<MemoryEntity>,
}

impl CreateEntitiesTool {
    generate_call_tool!(
        self,
        CreateEntitiesCommand {
            entities => self.entities.clone()
        },
        create_entities
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
        mock.expect_create_entities()
            .withf(|ents| ents.len() == 1 && ents[0].name == "test:entity")
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                allow_default_labels: false,
                agent_name: "test".to_string(),
                ..MemoryConfig::default()
            },
        );
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        let tool = CreateEntitiesTool {
            entities: vec![MemoryEntity {
                name: "test:entity".to_string(),
                labels: vec!["Test".to_string()],
                ..Default::default()
            }],
        };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        // With our new macro, we're returning null
        assert_eq!(text, "null");
    }

    #[tokio::test]
    async fn test_call_tool_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities()
            .returning(|_| Err(MemoryError::runtime_error("fail")));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        let tool = CreateEntitiesTool {
            entities: vec![MemoryEntity {
                name: "test:entity".to_string(),
                labels: vec!["Test".to_string()],
                ..Default::default()
            }],
        };

        let result = tool.call_tool(&ports).await;
        assert!(result.is_err());
    }
}
#[cfg(test)]
mod schema_tests {
    use super::*;
    use crate::mcp::tests::assert_no_defs;

    #[test]
    fn test_schema_has_no_refs() {
        assert_no_defs::<CreateEntitiesTool>();
    }
}
