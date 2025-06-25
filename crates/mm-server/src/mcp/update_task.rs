use mm_core::operations::memory::{TaskProperties, UpdateTaskCommand, update_task};
use mm_memory::{EntityUpdate, ObservationsUpdate, PropertiesUpdate};
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "update_task",
    description = "Update a task's observations or properties"
)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdateTaskTool {
    /// Task name
    pub task_name: String,
    /// Optional project name (unused)
    pub project_name: Option<String>,
    /// Replace observations
    #[serde(default)]
    pub observations: Option<Vec<String>>,
    /// Replace properties
    #[serde(default)]
    pub properties: Option<TaskProperties>,
}

impl UpdateTaskTool {
    generate_call_tool!(
        self,
        UpdateTaskCommand {
            name => self.task_name.clone(),
            update => {
                let mut update = EntityUpdate::default();
                if let Some(obs) = self.observations.clone() {
                    update.observations = Some(ObservationsUpdate { add: None, remove: None, set: Some(obs) });
                }
                if let Some(props) = self.properties.clone() {
                    update.properties = Some(PropertiesUpdate { add: None, remove: None, set: Some(props.into()) });
                }
                update
            }
        },
        update_task,
        "Task updated"
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
        mock.expect_update_entity()
            .withf(|n, _| n == "task:1")
            .returning(|_, _| Ok(()));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        let tool = UpdateTaskTool {
            task_name: "task:1".into(),
            project_name: None,
            observations: Some(vec!["done".into()]),
            properties: None,
        };

        let result = tool.call_tool(&ports).await.unwrap();
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert_eq!(text, "Task updated");
    }
}
