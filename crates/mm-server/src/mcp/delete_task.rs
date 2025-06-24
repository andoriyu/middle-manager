use mm_core::operations::memory::{DeleteTaskCommand, delete_task};
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

#[mcp_tool(name = "delete_task", description = "Delete a task")]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeleteTaskTool {
    /// Task name
    pub task_name: String,
    /// Optional project name (unused)
    pub project_name: Option<String>,
}

impl DeleteTaskTool {
    generate_call_tool!(
        self,
        DeleteTaskCommand { name => self.task_name.clone() },
        delete_task,
        |command, _res| {
            Ok(rust_mcp_sdk::schema::CallToolResult::text_content(command.name, None))
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
        mock.expect_delete_entities()
            .withf(|names| names.len() == 1 && names[0] == "task:1")
            .returning(|_| Ok(()));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = DeleteTaskTool {
            task_name: "task:1".into(),
            project_name: None,
        };

        let result = tool.call_tool(&ports).await.unwrap();
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert_eq!(text, "task:1");
    }
}
