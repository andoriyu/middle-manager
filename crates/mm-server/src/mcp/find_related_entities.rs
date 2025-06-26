use mm_core::operations::memory::{FindRelatedEntitiesCommand, find_related_entities};
use mm_memory::RelationshipDirection;
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "find_related_entities",
    description = "Find entities related to a given entity"
)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FindRelatedEntitiesTool {
    pub name: String,
    pub relationship: Option<String>,
    pub direction: Option<RelationshipDirection>,
    pub depth: u32,
}

impl FindRelatedEntitiesTool {
    generate_call_tool!(
        self,
        FindRelatedEntitiesCommand {
            name,
            relationship,
            direction,
            depth
        },
        find_related_entities
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::Ports;
    use mm_memory::{MemoryConfig, MemoryEntity, MemoryService, MockMemoryRepository};
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_call_tool_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_related_entities()
            .with(
                eq("a"),
                eq(Some("rel".to_string())),
                eq(Some(RelationshipDirection::Outgoing)),
                eq(2u32),
            )
            .returning(|_, _, _, _| Ok(vec![MemoryEntity::default()]));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        let tool = FindRelatedEntitiesTool {
            name: "a".into(),
            relationship: Some("rel".into()),
            direction: Some(RelationshipDirection::Outgoing),
            depth: 2,
        };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert!(text.contains("entities"));
    }

    #[test]
    fn test_schema_has_no_refs() {
        use crate::mcp::tests::assert_no_defs;
        assert_no_defs::<FindRelatedEntitiesTool>();
    }
}
