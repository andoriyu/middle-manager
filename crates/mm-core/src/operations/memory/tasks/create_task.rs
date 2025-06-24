use super::types::TaskProperties;
use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use crate::validate_name;
use futures::future::try_join_all;
use mm_memory::MemoryRepository;
use mm_memory::{MemoryEntity, MemoryRelationship, ValidationError, ValidationErrorKind};
use std::collections::HashMap;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct CreateTaskCommand {
    pub task: MemoryEntity<TaskProperties>,
    pub project_name: Option<String>,
    pub depends_on: Vec<String>,
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

    // Validate dependency names and ensure they exist
    if command.depends_on.iter().any(|d| d == &command.task.name) {
        return Err(CoreError::Validation(ValidationError(vec![
            ValidationErrorKind::ConflictingOperations("depends_on"),
        ])));
    }

    for dep in &command.depends_on {
        validate_name!(dep);
    }

    let lookups = command
        .depends_on
        .iter()
        .map(|name| ports.memory_service.find_entity_by_name(name));
    let results = try_join_all(lookups).await.map_err(CoreError::from)?;

    for (dep, found) in command.depends_on.iter().zip(results) {
        if found.is_none() {
            return Err(CoreError::Memory(mm_memory::MemoryError::entity_not_found(
                dep.clone(),
            )));
        }
    }

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

    let mut relationships = vec![MemoryRelationship {
        from: project_name,
        to: task_name.clone(),
        name: "contains".to_string(),
        properties: HashMap::default(),
    }];

    for dep in &command.depends_on {
        relationships.push(MemoryRelationship {
            from: task_name.clone(),
            to: dep.clone(),
            name: "depends_on".to_string(),
            properties: HashMap::default(),
        });
    }

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
    use mm_memory::{
        MemoryConfig, MemoryError, MemoryService, MockMemoryRepository, ValidationErrorKind,
    };
    use mockall::predicate::*;
    use std::collections::HashSet;
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
                additional_relationships: std::iter::once("depends_on".to_string())
                    .collect::<HashSet<_>>(),
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
            depends_on: Vec::new(),
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
            depends_on: Vec::new(),
        };

        let res = create_task(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::MissingProject)));
    }

    #[tokio::test]
    async fn test_create_task_empty_name() {
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

        let cmd = CreateTaskCommand {
            task: MemoryEntity::<TaskProperties> {
                name: String::new(),
                labels: vec!["Task".into()],
                ..Default::default()
            },
            project_name: None,
            depends_on: Vec::new(),
        };

        let res = create_task(&ports, cmd).await;
        assert!(
            matches!(res, Err(CoreError::Validation(ref e)) if e.0.contains(&ValidationErrorKind::EmptyEntityName))
        );
    }

    #[tokio::test]
    async fn test_create_task_with_dependencies() {
        let mut mock = MockMemoryRepository::new();

        mock.expect_find_entity_by_name()
            .with(eq("task:dep"))
            .times(1)
            .returning(|_| Ok(Some(MemoryEntity::default())));

        mock.expect_create_entities()
            .withf(|e| e.len() == 1 && e[0].name == "task:1")
            .returning(|_| Ok(()));

        mock.expect_create_relationships()
            .withf(|r| {
                r.len() == 2
                    && r[0].name == "contains"
                    && r[1].name == "depends_on"
                    && r[1].to == "task:dep"
                    && r[1].from == "task:1"
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

        let cmd = CreateTaskCommand {
            task: MemoryEntity::<TaskProperties> {
                name: "task:1".into(),
                labels: vec!["Task".into()],
                ..Default::default()
            },
            project_name: None,
            depends_on: vec!["task:dep".into()],
        };

        let res = create_task(&ports, cmd).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_create_task_dependency_missing() {
        let mut mock = MockMemoryRepository::new();

        mock.expect_find_entity_by_name()
            .with(eq("task:missing"))
            .times(1)
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

        let cmd = CreateTaskCommand {
            task: MemoryEntity::<TaskProperties> {
                name: "task:1".into(),
                labels: vec!["Task".into()],
                ..Default::default()
            },
            project_name: None,
            depends_on: vec!["task:missing".into()],
        };

        let res = create_task(&ports, cmd).await;
        assert!(matches!(
            res,
            Err(CoreError::Memory(MemoryError::EntityNotFound(_)))
        ));
    }

    #[tokio::test]
    async fn test_create_task_self_dependency() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entity_by_name().never();
        mock.expect_create_entities().never();
        mock.expect_create_relationships().never();

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

        let cmd = CreateTaskCommand {
            task: MemoryEntity::<TaskProperties> {
                name: "task:1".into(),
                labels: vec!["Task".into()],
                ..Default::default()
            },
            project_name: None,
            depends_on: vec!["task:1".into()],
        };

        let res = create_task(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::Validation(_))));
    }
}
