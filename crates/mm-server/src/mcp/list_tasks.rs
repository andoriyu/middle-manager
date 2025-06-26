use mm_core::operations::memory::{ListTasksCommand, list_tasks};
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

#[mcp_tool(name = "list_tasks", description = "List tasks for a project")]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListTasksTool {
    /// Optional project name
    pub project_name: Option<String>,
    /// Optional lifecycle label to filter by
    pub lifecycle: Option<String>,
}

impl ListTasksTool {
    generate_call_tool!(
        self,
        ListTasksCommand { project_name => self.project_name.clone(), lifecycle => self.lifecycle.clone() },
        list_tasks
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::operations::memory::TASK_LABEL;
    use mm_core::{Ports, operations::memory::TaskProperties};
    use mm_memory::{
        MemoryConfig, MemoryEntity, MemoryService, MockMemoryRepository, RelationshipDirection,
        labels::ACTIVE_LABEL, value::MemoryValue,
    };
    use mockall::predicate::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_call_tool_success() {
        let props: HashMap<String, MemoryValue> = TaskProperties::default().into();
        let task = MemoryEntity {
            name: "task:1".into(),
            labels: vec![TASK_LABEL.to_string(), ACTIVE_LABEL.to_string()],
            observations: vec![],
            properties: props.clone(),
            relationships: vec![],
        };
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_related_entities()
            .with(
                eq("proj"),
                eq(Some("contains".to_string())),
                eq(Some(RelationshipDirection::Outgoing)),
                eq(1u32),
            )
            .returning(move |_, _, _, _| Ok(vec![task.clone()]));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_project: Some("proj".into()),
                ..MemoryConfig::default()
            },
        );
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        let tool = ListTasksTool {
            project_name: None,
            lifecycle: None,
        };
        let result = tool.call_tool(&ports).await.unwrap();
        let text = result.content[0].as_text_content().unwrap().text.clone();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert!(value.get("tasks").unwrap().as_array().unwrap().len() == 1);
    }
}
