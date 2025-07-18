use mm_core::operations::memory::{GetTaskCommand, get_task};
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

#[mcp_tool(name = "get_task", description = "Retrieve a task by name")]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetTaskTool {
    /// Task name
    pub task_name: String,
    /// Optional project name (unused)
    pub project_name: Option<String>,
}

impl GetTaskTool {
    generate_call_tool!(
        self,
        GetTaskCommand { name => self.task_name.clone() },
        get_task
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::Ports;
    use mm_core::operations::memory::TASK_LABEL;
    use mm_memory::{MemoryConfig, MemoryEntity, MemoryService, MockMemoryRepository};
    use mockall::predicate::*;
    use serde_json::Value;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_call_tool_success() {
        let entity = MemoryEntity {
            name: "task:1".into(),
            labels: vec![TASK_LABEL.to_string()],
            ..Default::default()
        };
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entity_by_name()
            .with(eq("task:1"))
            .returning(move |_| Ok(Some(entity.clone())));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        let tool = GetTaskTool {
            task_name: "task:1".into(),
            project_name: None,
        };

        let result = tool.call_tool(&ports).await.unwrap();
        let text = result.content[0].as_text_content().unwrap().text.clone();
        let value: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["name"], "task:1");
    }
}
