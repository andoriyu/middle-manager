use super::types::TaskProperties;
use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use mm_git::GitRepository;
use mm_memory::{MemoryEntity, MemoryRepository, RelationshipDirection, labels::TASK_LABEL};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// Command for listing tasks
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListTasksCommand {
    /// Optional project name to list tasks for
    pub project_name: Option<String>,
    /// Optional lifecycle label to filter tasks
    pub lifecycle: Option<String>,
}

/// Result of listing tasks
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListTasksResult {
    /// Tasks matching the query
    pub tasks: Vec<MemoryEntity<TaskProperties>>,
}

/// List tasks for a project optionally filtered by lifecycle label
#[instrument(skip(ports), err)]
pub async fn list_tasks<M, G>(
    ports: &Ports<M, G>,
    command: ListTasksCommand,
) -> CoreResult<ListTasksResult, M::Error>
where
    M: MemoryRepository + Send + Sync,
    G: GitRepository + Send + Sync,
    M::Error: std::error::Error + Send + Sync + 'static,
    G::Error: std::error::Error + Send + Sync + 'static,
{
    let project_name = match command
        .project_name
        .or_else(|| ports.memory_service.memory_config().default_project.clone())
    {
        Some(p) => p,
        None => return Err(CoreError::MissingProject),
    };

    let mut tasks = ports
        .memory_service
        .find_related_entities_typed::<TaskProperties>(
            &project_name,
            Some("contains".to_string()),
            Some(RelationshipDirection::Outgoing),
            1,
        )
        .await
        .map_err(CoreError::from)?
        .into_iter()
        .filter(|t| t.labels.contains(&TASK_LABEL.to_string()))
        .collect::<Vec<_>>();

    if let Some(label) = command.lifecycle {
        tasks.retain(|t| t.labels.contains(&label));
    }

    Ok(ListTasksResult { tasks })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::Ports;
    use mm_memory::{
        MemoryConfig, MemoryEntity, MemoryService, MockMemoryRepository, RelationshipDirection,
        labels::{ACTIVE_LABEL, TASK_LABEL},
        value::MemoryValue,
    };
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_list_tasks_no_filter() {
        let props: std::collections::HashMap<String, MemoryValue> =
            TaskProperties::default().into();
        let task1 = MemoryEntity {
            name: "task:1".into(),
            labels: vec![TASK_LABEL.to_string(), ACTIVE_LABEL.to_string()],
            observations: vec![],
            properties: props.clone(),
            relationships: vec![],
        };
        let task2 = MemoryEntity {
            name: "task:2".into(),
            labels: vec![TASK_LABEL.to_string()],
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
            .returning(move |_, _, _, _| Ok(vec![task1.clone(), task2.clone()]));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_project: Some("proj".into()),
                ..MemoryConfig::default()
            },
        );
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        let cmd = ListTasksCommand {
            project_name: None,
            lifecycle: None,
        };
        let result = list_tasks(&ports, cmd).await.unwrap();
        assert_eq!(result.tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_list_tasks_with_lifecycle() {
        let props: std::collections::HashMap<String, MemoryValue> =
            TaskProperties::default().into();
        let task1 = MemoryEntity {
            name: "task:1".into(),
            labels: vec![TASK_LABEL.to_string(), ACTIVE_LABEL.to_string()],
            observations: vec![],
            properties: props.clone(),
            relationships: vec![],
        };
        let task2 = MemoryEntity {
            name: "task:2".into(),
            labels: vec![TASK_LABEL.to_string()],
            observations: vec![],
            properties: props.clone(),
            relationships: vec![],
        };
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_related_entities()
            .returning(move |_, _, _, _| Ok(vec![task1.clone(), task2.clone()]));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_project: Some("proj".into()),
                ..MemoryConfig::default()
            },
        );
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));
        let cmd = ListTasksCommand {
            project_name: None,
            lifecycle: Some(ACTIVE_LABEL.to_string()),
        };
        let result = list_tasks(&ports, cmd).await.unwrap();
        assert_eq!(result.tasks.len(), 1);
        assert_eq!(result.tasks[0].name, "task:1");
    }

    #[tokio::test]
    async fn test_list_tasks_missing_project() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_related_entities().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));
        let cmd = ListTasksCommand {
            project_name: None,
            lifecycle: None,
        };
        let res = list_tasks(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::MissingProject)));
    }
}
