use mm_core::operations::memory::{GetGraphMetaCommand, get_graph_meta};
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

/// MCP tool for retrieving metadata about the memory graph root
#[mcp_tool(
    name = "get_graph_meta",
    description = "Return entities related to the memory graph root"
)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetGraphMetaTool {
    /// Optional relationship type filter
    pub relationship: Option<String>,
}

impl GetGraphMetaTool {
    generate_call_tool!(self, GetGraphMetaCommand { relationship }, get_graph_meta);
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::Ports;
    use mm_memory::{
        MemoryConfig, MemoryEntity, MemoryService, MockMemoryRepository, RelationshipDirection,
    };
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_call_tool_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_related_entities()
            .with(
                eq(mm_core::operations::memory::GRAPH_ROOT),
                eq(None),
                eq(Some(RelationshipDirection::Outgoing)),
                eq(5u32),
            )
            .returning(|_, _, _, _| Ok(vec![MemoryEntity::default()]));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        let tool = GetGraphMetaTool { relationship: None };
        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert!(text.contains("entities"));
    }

    #[test]
    fn test_schema_has_no_defs() {
        use crate::mcp::tests::assert_no_defs;
        assert_no_defs::<GetGraphMetaTool>();
    }
}
