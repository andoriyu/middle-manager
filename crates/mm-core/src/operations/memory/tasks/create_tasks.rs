use super::types::TaskProperties;
use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use mm_memory::MemoryRepository;
use mm_memory::{MemoryEntity, MemoryRelationship};
use std::collections::HashMap;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct CreateTasksCommand {
    pub tasks: Vec<MemoryEntity<TaskProperties>>,
    pub project_name: Option<String>,
}

pub type CreateTasksResult<E> = CoreResult<(), E>;

#[instrument(skip(ports), fields(tasks_count = command.tasks.len()))]
pub async fn create_tasks<R>(
    ports: &Ports<R>,
    command: CreateTasksCommand,
) -> CreateTasksResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    let project_name = match command
        .project_name
        .or_else(|| ports.memory_service.memory_config().default_project.clone())
    {
        Some(p) => p,
        None => return Err(CoreError::MissingProject),
    };

    let tasks = command.tasks;
    let task_names: Vec<String> = tasks.iter().map(|t| t.name.clone()).collect();

    let errors = ports
        .memory_service
        .create_entities_typed(&tasks)
        .await
        .map_err(CoreError::from)?;

    if !errors.is_empty() {
        return Err(CoreError::BatchValidation(errors));
    }

    let relationships: Vec<MemoryRelationship> = task_names
        .into_iter()
        .map(|task_name| MemoryRelationship {
            from: project_name.clone(),
            to: task_name,
            name: "contains".to_string(),
            properties: HashMap::default(),
        })
        .collect();

    let errors = ports
        .memory_service
        .create_relationships(&relationships)
        .await
        .map_err(CoreError::from)?;

    if !errors.is_empty() {
        return Err(CoreError::BatchValidation(errors));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository, ValidationErrorKind};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create_tasks_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities()
            .withf(|e| e.len() == 1 && e[0].name == "task:1")
            .returning(|_| Ok(()));
        mock.expect_create_relationships()
            .withf(|r| {
                r.len() == 1
                    && r[0].from == "proj"
                    && r[0].to == "task:1"
                    && r[0].name == "contains"
            })
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_project: Some("proj".into()),
                ..MemoryConfig::default()
            },
        );
        let ports = Ports::new(Arc::new(service));

        let cmd = CreateTasksCommand {
            tasks: vec![MemoryEntity::<TaskProperties> {
                name: "task:1".into(),
                labels: vec!["Task".into()],
                ..Default::default()
            }],
            project_name: None,
        };

        let res = create_tasks(&ports, cmd).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_create_tasks_missing_project() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities().never();
        mock.expect_create_relationships().never();

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let cmd = CreateTasksCommand {
            tasks: vec![MemoryEntity::<TaskProperties> {
                name: "task:1".into(),
                labels: vec!["Task".into()],
                ..Default::default()
            }],
            project_name: None,
        };

        let res = create_tasks(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::MissingProject)));
    }

    #[tokio::test]
    async fn test_create_tasks_empty_name() {
        let mut mock = MockMemoryRepository::new();
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

        let cmd = CreateTasksCommand {
            tasks: vec![MemoryEntity::<TaskProperties> {
                name: String::new(),
                labels: vec!["Task".into()],
                ..Default::default()
            }],
            project_name: None,
        };

        let res = create_tasks(&ports, cmd).await;
        assert!(matches!(
            res,
            Err(CoreError::BatchValidation(ref errs))
                if errs.iter().any(|(_, e)| e.0.contains(&ValidationErrorKind::EmptyEntityName))
        ));
    }
}
