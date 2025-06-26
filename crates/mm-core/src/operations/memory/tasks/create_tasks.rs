use super::super::common::handle_batch_result;
use super::types::TaskProperties;
use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use mm_git::GitRepository;
use mm_memory::MemoryRepository;
use mm_memory::{MemoryEntity, MemoryRelationship, ValidationError, ValidationErrorKind};
use std::collections::HashMap;
use tracing::instrument;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TaskInput {
    pub task: MemoryEntity<TaskProperties>,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CreateTasksCommand {
    pub tasks: Vec<TaskInput>,
    pub project_name: Option<String>,
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
    let new_names: std::collections::HashSet<String> =
        tasks.iter().map(|t| t.task.name.clone()).collect();

    // Validate dependencies
    let mut validation_errors = Vec::new();
    for task in &tasks {
        if task.depends_on.iter().any(|d| d == &task.task.name) {
            validation_errors.push((
                task.task.name.clone(),
                ValidationError(vec![ValidationErrorKind::SelfDependency(
                    task.task.name.clone(),
                )]),
            ));
        }
        for dep in &task.depends_on {
            if dep != &task.task.name
                && !new_names.contains(dep)
                && ports
                    .memory_service
                    .find_entity_by_name(dep)
                    .await?
                    .is_none()
            {
                validation_errors.push((
                    task.task.name.clone(),
                    ValidationError(vec![ValidationErrorKind::DependencyNotFound(dep.clone())]),
                ));
            }
        }
    }
    if !validation_errors.is_empty() {
        return Err(CoreError::BatchValidation(validation_errors));
    }

    // Create the task entities
    let entities: Vec<MemoryEntity<TaskProperties>> =
        tasks.iter().map(|t| t.task.clone()).collect();
    handle_batch_result(|| ports.memory_service.create_entities_typed(&entities)).await?;

    let mut relationships: Vec<MemoryRelationship> = Vec::new();
    for task in &tasks {
        relationships.push(MemoryRelationship {
            from: project_name.clone(),
            to: task.task.name.clone(),
            name: "contains".to_string(),
            properties: HashMap::default(),
        });

        for dependency in &task.depends_on {
            relationships.push(MemoryRelationship {
                from: task.task.name.clone(),
                to: dependency.clone(),
                name: "depends_on".to_string(),
                properties: HashMap::default(),
            });
        }
    }

    handle_batch_result(|| ports.memory_service.create_relationships(&relationships)).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_memory::labels::TASK_LABEL;
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
        let ports = Ports::noop().with(|p| {
            p.memory_service = Arc::new(service);
        });

        let cmd = CreateTasksCommand {
            tasks: vec![TaskInput {
                task: MemoryEntity::<TaskProperties> {
                    name: "task:1".into(),
                    labels: vec![TASK_LABEL.to_string()],
                    ..Default::default()
                },
                depends_on: Vec::new(),
            }],
            project_name: None,
        };

        let res = create_tasks(&ports, cmd).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_create_tasks_with_dependencies() {
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
                additional_relationships: std::iter::once("depends_on".to_string())
                    .collect::<HashSet<_>>(),
                ..MemoryConfig::default()
            },
        );
        let ports = Ports::noop().with(|p| {
            p.memory_service = Arc::new(service);
        });

        let cmd = CreateTasksCommand {
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

        let res = create_tasks(&ports, cmd).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_create_tasks_missing_project() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities().never();
        mock.expect_create_relationships().never();

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| {
            p.memory_service = Arc::new(service);
        });

        let cmd = CreateTasksCommand {
            tasks: vec![TaskInput {
                task: MemoryEntity::<TaskProperties> {
                    name: "task:1".into(),
                    labels: vec![TASK_LABEL.to_string()],
                    ..Default::default()
                },
                depends_on: Vec::new(),
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
        let ports = Ports::noop().with(|p| {
            p.memory_service = Arc::new(service);
        });

        let cmd = CreateTasksCommand {
            tasks: vec![TaskInput {
                task: MemoryEntity::<TaskProperties> {
                    name: String::new(),
                    labels: vec![TASK_LABEL.to_string()],
                    ..Default::default()
                },
                depends_on: Vec::new(),
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

    #[tokio::test]
    async fn test_create_tasks_self_dependency() {
        let mut mock = MockMemoryRepository::new();
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
        let ports = Ports::noop().with(|p| {
            p.memory_service = Arc::new(service);
        });

        let cmd = CreateTasksCommand {
            tasks: vec![TaskInput {
                task: MemoryEntity::<TaskProperties> {
                    name: "task:1".into(),
                    labels: vec![TASK_LABEL.to_string()],
                    ..Default::default()
                },
                depends_on: vec!["task:1".into()],
            }],
            project_name: None,
        };

        let res = create_tasks(&ports, cmd).await;
        assert!(
            matches!(res, Err(CoreError::BatchValidation(ref errs)) if errs
            .iter()
            .any(|(n, e)| {
                n == "task:1" && e.0.iter().any(|k| matches!(k, ValidationErrorKind::SelfDependency(_)))
            }))
        );
    }

    #[tokio::test]
    async fn test_task_specific_dependencies() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entity_by_name()
            .with(mockall::predicate::eq("other:task"))
            .returning(|_| {
                Ok(Some(MemoryEntity {
                    name: "other:task".into(),
                    labels: vec![TASK_LABEL.to_string()],
                    ..Default::default()
                }))
            });
        mock.expect_create_entities()
            .withf(|ents| {
                ents.len() == 2
                    && ents.iter().any(|e| e.name == "task:1")
                    && ents.iter().any(|e| e.name == "task:2")
            })
            .returning(|_| Ok(()));
        mock.expect_create_relationships()
            .withf(|rels| {
                rels.len() == 5
                    && rels
                        .iter()
                        .any(|r| r.from == "proj" && r.to == "task:1" && r.name == "contains")
                    && rels
                        .iter()
                        .any(|r| r.from == "proj" && r.to == "task:2" && r.name == "contains")
                    && rels
                        .iter()
                        .any(|r| r.from == "task:1" && r.to == "task:2" && r.name == "depends_on")
                    && rels.iter().any(|r| {
                        r.from == "task:1" && r.to == "other:task" && r.name == "depends_on"
                    })
                    && rels.iter().any(|r| {
                        r.from == "task:2" && r.to == "other:task" && r.name == "depends_on"
                    })
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
        let ports = Ports::noop().with(|p| {
            p.memory_service = Arc::new(service);
        });

        let cmd = CreateTasksCommand {
            tasks: vec![
                TaskInput {
                    task: MemoryEntity::<TaskProperties> {
                        name: "task:1".into(),
                        labels: vec![TASK_LABEL.to_string()],
                        ..Default::default()
                    },
                    depends_on: vec!["task:2".into(), "other:task".into()],
                },
                TaskInput {
                    task: MemoryEntity::<TaskProperties> {
                        name: "task:2".into(),
                        labels: vec![TASK_LABEL.to_string()],
                        ..Default::default()
                    },
                    depends_on: vec!["other:task".into()],
                },
            ],
            project_name: None,
        };

        let res = create_tasks(&ports, cmd).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_dependency_must_exist() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entity_by_name()
            .with(mockall::predicate::eq("missing:task"))
            .returning(|_| Ok(None));
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
        let ports = Ports::noop().with(|p| {
            p.memory_service = Arc::new(service);
        });

        let cmd = CreateTasksCommand {
            tasks: vec![TaskInput {
                task: MemoryEntity::<TaskProperties> {
                    name: "task:1".into(),
                    labels: vec![TASK_LABEL.to_string()],
                    ..Default::default()
                },
                depends_on: vec!["missing:task".into()],
            }],
            project_name: None,
        };

        let res = create_tasks(&ports, cmd).await;
        assert!(matches!(
            res,
            Err(CoreError::BatchValidation(ref errs))
                if errs.iter().any(|(n, e)| {
                    n == "task:1" && e.0.contains(&ValidationErrorKind::DependencyNotFound("missing:task".into()))
                })
        ));
    }
}
