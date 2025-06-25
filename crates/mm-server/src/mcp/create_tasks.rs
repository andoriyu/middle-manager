use mm_core::operations::memory::{CreateTasksCommand, TaskProperties, create_tasks};
use mm_memory::MemoryEntity;
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "create_tasks",
    description = "Create tasks and associate them with a project"
)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateTasksTool {
    /// Tasks to create
    pub tasks: Vec<MemoryEntity<TaskProperties>>,
    /// Project to associate with
    pub project_name: Option<String>,
    /// Tasks this task depends on
    #[serde(default)]
    pub depends_on: Vec<String>,
}

impl CreateTasksTool {
    generate_call_tool!(
        self,
        CreateTasksCommand { tasks => self.tasks.clone(), project_name, depends_on },
        create_tasks,
        |command, _result| {
            serde_json::to_value(command.tasks)
                .map(|json| rust_mcp_sdk::schema::CallToolResult::text_content(json.to_string(), None))
                .map_err(|e| {
                    rust_mcp_sdk::schema::schema_utils::CallToolError::new(
                        crate::mcp::error::ToolError::from(e),
                    )
                })
        }
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::Ports;
    use mm_git::repository::MockGitRepository;
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
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));

        let tool = CreateTasksTool {
            tasks: vec![MemoryEntity::<TaskProperties> {
                name: "task:1".into(),
                labels: vec!["Task".into()],
                ..Default::default()
            }],
            project_name: None,
            depends_on: vec![],
        };

        let result = tool.call_tool(&ports).await.unwrap();
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert!(text.contains("task:1"));
    }
}
