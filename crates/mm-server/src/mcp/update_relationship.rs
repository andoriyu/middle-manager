use mm_core::operations::memory::{UpdateRelationshipCommand, update_relationship};
use mm_memory::RelationshipUpdate;
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "update_relationship",
    description = "Update properties of a relationship"
)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdateRelationshipTool {
    /// Source entity name
    pub from: String,
    /// Target entity name
    pub to: String,
    /// Relationship type
    pub name: String,
    /// Property modifications
    pub update: RelationshipUpdate,
}

impl UpdateRelationshipTool {
    generate_call_tool!(
        self,
        UpdateRelationshipCommand {
            from,
            to,
            name,
            update
        },
        update_relationship,
        |_cmd, _res| {
            Ok(rust_mcp_sdk::schema::CallToolResult::text_content(
                "Relationship updated".to_string(),
                None,
            ))
        }
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::Ports;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_call_tool_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_update_relationship()
            .withf(|f, t, n, _| f == "a" && t == "b" && n == "rel")
            .returning(|_, _, _, _| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));
        let tool = UpdateRelationshipTool {
            from: "a".into(),
            to: "b".into(),
            name: "rel".into(),
            update: RelationshipUpdate::default(),
        };
        let result = tool.call_tool(&ports).await.unwrap();
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert_eq!(text, "Relationship updated");
    }
}
