use mm_core::operations::memory::{CreateTaskCommand, TaskProperties, create_task};
use mm_memory::MemoryEntity;
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "create_task",
    description = "Create a task and associate it with a project"
)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateTaskTool {
    /// Unique task name
    pub task_name: String,
    /// Labels for the task
    pub labels: Vec<String>,
    /// Observations describing the task
    #[serde(default)]
    pub observations: Vec<String>,
    /// Task properties
    #[serde(default)]
    pub properties: Option<TaskProperties>,
    /// Project to associate with
    pub project_name: Option<String>,
}

impl CreateTaskTool {
    generate_call_tool!(
        self,
        CreateTaskCommand {
            task => MemoryEntity::<TaskProperties> {
                name: self.task_name.clone(),
                labels: self.labels.clone(),
                observations: self.observations.clone(),
                properties: self.properties.clone().unwrap_or_default(),
                relationships: Vec::new(),
            },
            project_name
        },
        create_task,
        |command, _result| {
            Ok(rust_mcp_sdk::schema::CallToolResult::text_content(
                command.task.name,
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
        mock.expect_create_entities()
            .withf(|ents| ents.len() == 1 && ents[0].name == "task:1")
            .returning(|_| Ok(()));
        mock.expect_create_relationships()
            .withf(|rels| rels.len() == 1 && rels[0].from == "proj" && rels[0].to == "task:1")
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_project: Some("proj".into()),
                ..MemoryConfig::default()
            },
        );
        let ports = Ports::new(Arc::new(service));

        let tool = CreateTaskTool {
            task_name: "task:1".into(),
            labels: vec!["Task".into()],
            observations: vec![],
            properties: None,
            project_name: None,
        };

        let result = tool.call_tool(&ports).await.unwrap();
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert_eq!(text, "task:1");
    }
}
