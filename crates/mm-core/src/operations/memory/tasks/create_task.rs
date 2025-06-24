use super::types::TaskProperties;
use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use crate::validate_name;
use mm_memory::MemoryRepository;
use mm_memory::{MemoryEntity, MemoryRelationship};
use std::collections::HashMap;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct CreateTaskCommand {
    pub task: MemoryEntity<TaskProperties>,
    pub project_name: Option<String>,
}

pub type CreateTaskResult<E> = CoreResult<(), E>;

#[instrument(skip(ports), fields(task_name = %command.task.name))]
pub async fn create_task<R>(
    ports: &Ports<R>,
    command: CreateTaskCommand,
) -> CreateTaskResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    validate_name!(command.task.name);

    let project_name = match command
        .project_name
        .or_else(|| ports.memory_service.memory_config().default_project.clone())
    {
        Some(p) => p,
        None => return Err(CoreError::MissingProject),
    };

    let task = command.task;
    let task_name = task.name.clone();

    let errors = ports
        .memory_service
        .create_entities_typed(std::slice::from_ref(&task))
        .await
        .map_err(CoreError::from)?;

    if !errors.is_empty() {
        return Err(CoreError::BatchValidation(errors));
    }

    let relationship = MemoryRelationship {
        from: project_name,
        to: task_name,
        name: "contains".to_string(),
        properties: HashMap::default(),
    };

    let errors = ports
        .memory_service
        .create_relationships(std::slice::from_ref(&relationship))
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
    async fn test_create_task_success() {
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

        let cmd = CreateTaskCommand {
            task: MemoryEntity::<TaskProperties> {
                name: "task:1".into(),
                labels: vec!["Task".into()],
                ..Default::default()
            },
            project_name: None,
        };

        let res = create_task(&ports, cmd).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_create_task_missing_project() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities().never();
        mock.expect_create_relationships().never();

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let cmd = CreateTaskCommand {
            task: MemoryEntity::<TaskProperties> {
                name: "task:1".into(),
                labels: vec!["Task".into()],
                ..Default::default()
            },
            project_name: None,
        };

        let res = create_task(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::MissingProject)));
    }

    #[tokio::test]
    async fn test_create_task_empty_name() {
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

        let cmd = CreateTaskCommand {
            task: MemoryEntity::<TaskProperties> {
                name: String::new(),
                labels: vec!["Task".into()],
                ..Default::default()
            },
            project_name: None,
        };

        let res = create_task(&ports, cmd).await;
        assert!(
            matches!(res, Err(CoreError::Validation(ref e)) if e.0.contains(&ValidationErrorKind::EmptyEntityName))
        );
    }
}
