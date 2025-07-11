use mm_core::operations::memory::{CreateTasksCommand, TaskInput, create_tasks};
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
    pub tasks: Vec<TaskInput>,
    /// Project to associate with
    pub project_name: Option<String>,
}

impl CreateTasksTool {
    generate_call_tool!(
        self,
        CreateTasksCommand { tasks => self.tasks.clone(), project_name },
        create_tasks
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::Ports;
    use mm_core::operations::memory::TASK_LABEL;
    use mm_core::operations::memory::TaskProperties;
    use mm_git::repository::MockGitRepository;
    use mm_memory::{MemoryConfig, MemoryEntity, MemoryService, MockMemoryRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_call_tool_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entity_by_name().never();
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
            tasks: vec![TaskInput {
                task: MemoryEntity::<TaskProperties> {
                    name: "task:1".into(),
                    labels: vec![TASK_LABEL.to_string()],
                    ..Default::default()
                },
                depends_on: vec![],
            }],
            project_name: None,
        };

        let result = tool.call_tool(&ports).await.unwrap();
        let text = result.content[0].as_text_content().unwrap().text.clone();
        // With our new macro, we're returning null
        assert_eq!(text, "null");
    }

    #[tokio::test]
    async fn test_call_tool_with_dependencies() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entity_by_name()
            .with(mockall::predicate::eq("task:1"))
            .return_once(|_| {
                Ok(Some(MemoryEntity {
                    name: "task:1".into(),
                    labels: vec![TASK_LABEL.to_string()],
                    ..Default::default()
                }))
            });
        mock.expect_create_entities()
            .withf(|ents| ents.len() == 1 && ents[0].name == "task:2")
            .returning(|_| Ok(()));
        mock.expect_create_relationships()
            .withf(|rels| {
                rels.len() == 2
                    && rels
                        .iter()
                        .any(|r| r.from == "proj" && r.to == "task:2" && r.name == "contains")
                    && rels
                        .iter()
                        .any(|r| r.from == "task:2" && r.to == "task:1" && r.name == "depends_on")
            })
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_project: Some("proj".into()),
                allowed_relationships: std::iter::once("depends_on".to_string()).collect(),
                ..MemoryConfig::default()
            },
        );
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));

        let tool = CreateTasksTool {
            tasks: vec![TaskInput {
                task: MemoryEntity::<TaskProperties> {
                    name: "task:2".into(),
                    labels: vec![TASK_LABEL.to_string()],
                    ..Default::default()
                },
                depends_on: vec!["task:1".into()],
            }],
            project_name: None,
        };

        let result = tool.call_tool(&ports).await.unwrap();
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert_eq!(text, "null");
    }
}
