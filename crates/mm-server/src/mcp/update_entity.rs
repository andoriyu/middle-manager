use mm_core::operations::memory::{UpdateEntityCommand, update_entity};
use mm_memory::EntityUpdate;
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

#[mcp_tool(name = "update_entity", description = "Update fields of an entity")]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdateEntityTool {
    /// Entity name to update
    pub name: String,
    /// Changes to apply
    pub update: EntityUpdate,
}

impl UpdateEntityTool {
    generate_call_tool!(
        self,
        UpdateEntityCommand { name, update },
        "Entity '{}' updated",
        update_entity
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::Ports;
    use mm_core::mm_git::GitService;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_call_tool_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_update_entity()
            .withf(|n, _| n == "e")
            .returning(|_, _| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service), Arc::new(GitService::new(())));
        let tool = UpdateEntityTool {
            name: "e".into(),
            update: EntityUpdate::default(),
        };
        let result = tool.call_tool(&ports).await.unwrap();
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert_eq!(text, "Entity 'e' updated");
    }
}
