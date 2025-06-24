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
    /// Tasks that this task depends on
    #[serde(default)]
    pub depends_on: Vec<String>,
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
            project_name,
            depends_on
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
    use mm_memory::{MemoryConfig, MemoryEntity, MemoryService, MockMemoryRepository};
    use mockall::predicate::*;
    use std::collections::HashSet;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_call_tool_success() {
        let mut mock = MockMemoryRepository::new();

        mock.expect_find_entity_by_name()
            .with(eq("task:dep"))
            .returning(|_| Ok(Some(MemoryEntity::default())));
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
                additional_relationships: std::iter::once("depends_on".to_string())
                    .collect::<HashSet<_>>(),
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
            depends_on: Vec::new(),
        };

        let result = tool.call_tool(&ports).await.unwrap();
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert_eq!(text, "task:1");
    }

    #[tokio::test]
    async fn test_call_tool_with_dependencies() {
        let mut mock = MockMemoryRepository::new();

        mock.expect_find_entity_by_name()
            .with(eq("task:dep"))
            .returning(|_| Ok(Some(MemoryEntity::default())));

        mock.expect_create_entities()
            .withf(|ents| ents.len() == 1 && ents[0].name == "task:1")
            .returning(|_| Ok(()));

        mock.expect_create_relationships()
            .withf(|rels| {
                rels.len() == 2
                    && rels[0].name == "contains"
                    && rels[1].name == "depends_on"
                    && rels[1].to == "task:dep"
                    && rels[1].from == "task:1"
            })
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_project: Some("proj".into()),
                additional_relationships: std::iter::once("depends_on".to_string())
                    .collect::<HashSet<_>>(),
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
            depends_on: vec!["task:dep".into()],
        };

        let result = tool.call_tool(&ports).await.unwrap();
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert_eq!(text, "task:1");
    }

    #[tokio::test]
    async fn test_call_tool_dependency_missing() {
        let mut mock = MockMemoryRepository::new();

        mock.expect_find_entity_by_name()
            .with(eq("missing"))
            .returning(|_| Ok(None));
        mock.expect_create_entities().never();
        mock.expect_create_relationships().never();

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
            depends_on: vec!["missing".into()],
        };

        let result = tool.call_tool(&ports).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_call_tool_self_dependency() {
        let mut mock = MockMemoryRepository::new();

        mock.expect_create_entities().never();
        mock.expect_create_relationships().never();
        mock.expect_find_entity_by_name().never();

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_project: Some("proj".into()),
                additional_relationships: std::iter::once("depends_on".to_string())
                    .collect::<HashSet<_>>(),
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
            depends_on: vec!["task:1".into()],
        };

        let result = tool.call_tool(&ports).await;
        assert!(result.is_err());
    }
}
