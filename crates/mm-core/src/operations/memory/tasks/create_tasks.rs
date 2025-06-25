use super::super::common::handle_batch_result;
use super::types::TaskProperties;
use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use mm_git::GitRepository;
use mm_memory::MemoryRepository;
use mm_memory::{MemoryEntity, MemoryRelationship};
use std::collections::HashMap;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct CreateTasksCommand {
    pub tasks: Vec<MemoryEntity<TaskProperties>>,
    pub project_name: Option<String>,
    pub depends_on: Vec<String>,
}

pub type CreateTasksResult<E> = CoreResult<(), E>;

#[instrument(skip(ports), fields(tasks_count = command.tasks.len()))]
pub async fn create_tasks<M, G>(
    ports: &Ports<M, G>,
    command: CreateTasksCommand,
) -> CreateTasksResult<M::Error>
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

    let tasks = command.tasks;
    let task_names: Vec<String> = tasks.iter().map(|t| t.name.clone()).collect();

    // Create the task entity
    handle_batch_result(|| ports.memory_service.create_entities_typed(&tasks)).await?;

    let relationships: Vec<MemoryRelationship> = task_names
        .into_iter()
        .map(|task_name| MemoryRelationship {
            from: project_name.clone(),
            to: task_name,
            name: "contains".to_string(),
            properties: HashMap::default(),
        })
        .collect();

    handle_batch_result(|| ports.memory_service.create_relationships(&relationships)).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_git::repository::MockGitRepository;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository, ValidationErrorKind};
    use std::collections::HashSet;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create_tasks_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities()
            .withf(|ents| ents.len() == 1 && ents[0].name == "task:1")
            .returning(|_| Ok(()));
        mock.expect_create_relationships()
            .withf(|rels| rels.len() == 1 && rels[0].name == "contains")
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
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));

        let cmd = CreateTasksCommand {
            tasks: vec![MemoryEntity::<TaskProperties> {
                name: "task:1".into(),
                labels: vec!["Task".into()],
                ..Default::default()
            }],
            project_name: None,
            depends_on: Vec::new(),
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
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));

        let cmd = CreateTasksCommand {
            tasks: vec![MemoryEntity::<TaskProperties> {
                name: "task:1".into(),
                labels: vec!["Task".into()],
                ..Default::default()
            }],
            project_name: None,
            depends_on: Vec::new(),
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
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));

        let cmd = CreateTasksCommand {
            tasks: vec![MemoryEntity::<TaskProperties> {
                name: String::new(),
                labels: vec!["Task".into()],
                ..Default::default()
            }],
            project_name: None,
            depends_on: Vec::new(),
        };

        let res = create_tasks(&ports, cmd).await;
        assert!(matches!(
            res,
            Err(CoreError::BatchValidation(ref errs))
                if errs.iter().any(|(_, e)| e.0.contains(&ValidationErrorKind::EmptyEntityName))
        ));
    }
}
