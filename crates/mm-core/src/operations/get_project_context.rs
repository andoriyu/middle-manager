use mm_memory::{LabelMatchMode, MemoryEntity, MemoryError, ProjectContext, RelationshipDirection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;

/// Filter for finding a project
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProjectFilter {
    /// Find project by its name (e.g., "andoriyu:project:middle_manager")
    Name(String),

    /// Find project by repository name (e.g., "andoriyu/middle-manager")
    Repository(String),
}

/// Command for retrieving project context
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetProjectContextCommand {
    /// Filter to use for finding the project
    pub filter: ProjectFilter,
}

/// Result of retrieving project context
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetProjectContextResult {
    /// Project context
    pub context: ProjectContext,
}

/// Get project context by name or repository
#[instrument(skip(ports), err)]
pub async fn get_project_context<R>(
    ports: &Ports<R>,
    command: GetProjectContextCommand,
) -> CoreResult<GetProjectContextResult, R::Error>
where
    R: mm_memory::MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    match command.filter {
        ProjectFilter::Name(name) => {
            // Try to find the project by name
            let project_entity = ports
                .memory_service
                .find_entity_by_name(&name)
                .await
                .map_err(CoreError::from)?;

            match project_entity {
                Some(entity) => {
                    if entity.labels.contains(&"Project".to_string()) {
                        let context = build_project_context(ports, entity).await?;
                        Ok(GetProjectContextResult { context })
                    } else {
                        Err(CoreError::Memory(MemoryError::entity_not_found(format!(
                            "Entity '{}' exists but is not a project",
                            name
                        ))))
                    }
                }
                None => Err(CoreError::Memory(MemoryError::entity_not_found(format!(
                    "No project found with name '{}'",
                    name
                )))),
            }
        }
        ProjectFilter::Repository(repo) => {
            // Try to find a GitRepository entity with this name
            let repo_entities = ports
                .memory_service
                .find_entities_by_labels(&["GitRepository".to_string()], LabelMatchMode::All, None)
                .await
                .map_err(CoreError::from)?
                .into_iter()
                .filter(|e| {
                    e.name.contains(&repo) || e.observations.iter().any(|o| o.contains(&repo))
                })
                .collect::<Vec<_>>();

            if repo_entities.is_empty() {
                return Err(CoreError::Memory(MemoryError::entity_not_found(format!(
                    "No repository found with name '{}'",
                    repo
                ))));
            }

            // For each repository, find related projects
            for repo_entity in repo_entities {
                let related_entities = ports
                    .memory_service
                    .find_related_entities(
                        &repo_entity.name,
                        Some("contains".to_string()),
                        Some(RelationshipDirection::Outgoing),
                        1,
                    )
                    .await
                    .map_err(CoreError::from)?;

                let projects = related_entities
                    .into_iter()
                    .filter(|e| e.labels.contains(&"Project".to_string()))
                    .collect::<Vec<_>>();

                if !projects.is_empty() {
                    // Use the first project found
                    let context = build_project_context(ports, projects[0].clone()).await?;
                    return Ok(GetProjectContextResult { context });
                }
            }

            Err(CoreError::Memory(MemoryError::entity_not_found(format!(
                "No project found related to repository '{}'",
                repo
            ))))
        }
    }
}

/// Build a project context for a specific project entity
async fn build_project_context<R>(
    ports: &Ports<R>,
    project: MemoryEntity,
) -> CoreResult<ProjectContext, R::Error>
where
    R: mm_memory::MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    let mut context = ProjectContext::new(project.clone());

    // Find all related entities
    let related_entities = ports
        .memory_service
        .find_related_entities(&project.name, None, None, 2)
        .await
        .map_err(CoreError::from)?;

    debug!(
        "Found {} related entities for project '{}'",
        related_entities.len(),
        project.name
    );

    // Categorize related entities
    for entity in related_entities {
        if entity.name != project.name {
            context.add_related_entity(entity);
        }
    }

    Ok(context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use mockall::predicate::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_project_context_by_name() {
        let mut mock_repo = MockMemoryRepository::new();

        // Setup mock for find_entity_by_name
        let project_entity = MemoryEntity {
            name: "andoriyu:project:middle_manager".to_string(),
            labels: vec!["Memory".to_string(), "Project".to_string()],
            observations: vec!["A project for managing memory".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        mock_repo
            .expect_find_entity_by_name()
            .with(eq("andoriyu:project:middle_manager"))
            .times(1)
            .returning(move |_| Ok(Some(project_entity.clone())));

        // Setup mock for find_related_entities
        let project_entity2 = MemoryEntity {
            name: "andoriyu:project:middle_manager".to_string(),
            labels: vec!["Memory".to_string(), "Project".to_string()],
            observations: vec!["A project for managing memory".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        let related_entity = MemoryEntity {
            name: "tech:language:rust".to_string(),
            labels: vec![
                "Memory".to_string(),
                "Technology".to_string(),
                "Language".to_string(),
            ],
            observations: vec!["A systems programming language".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        mock_repo
            .expect_find_related_entities()
            .with(
                eq("andoriyu:project:middle_manager"),
                always(),
                always(),
                always(),
            )
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![project_entity2.clone(), related_entity.clone()]));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = GetProjectContextCommand {
            filter: ProjectFilter::Name("andoriyu:project:middle_manager".to_string()),
        };

        let result = get_project_context(&ports, command).await.unwrap();

        assert_eq!(
            result.context.project.name,
            "andoriyu:project:middle_manager"
        );
        assert_eq!(result.context.technologies.len(), 1);
        assert_eq!(result.context.technologies[0].name, "tech:language:rust");
    }

    #[tokio::test]
    async fn test_get_project_context_by_repository() {
        let mut mock_repo = MockMemoryRepository::new();

        // Setup mock for find_entities_by_labels
        let repo_entity = MemoryEntity {
            name: "tech:git:repo:andoriyu/middle-manager".to_string(),
            labels: vec!["Memory".to_string(), "GitRepository".to_string()],
            observations: vec!["A git repository".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        mock_repo
            .expect_find_entities_by_labels()
            .with(
                eq(vec!["GitRepository".to_string()]),
                eq(LabelMatchMode::All),
                always(),
            )
            .times(1)
            .returning(move |_, _, _| Ok(vec![repo_entity.clone()]));

        // Setup mock for find_related_entities for repository
        let project_entity = MemoryEntity {
            name: "andoriyu:project:middle_manager".to_string(),
            labels: vec!["Memory".to_string(), "Project".to_string()],
            observations: vec!["A project for managing memory".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        mock_repo
            .expect_find_related_entities()
            .with(
                eq("tech:git:repo:andoriyu/middle-manager"),
                eq(Some("contains".to_string())),
                eq(Some(RelationshipDirection::Outgoing)),
                eq(1),
            )
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![project_entity.clone()]));

        // Setup mock for find_related_entities for project
        let project_entity2 = MemoryEntity {
            name: "andoriyu:project:middle_manager".to_string(),
            labels: vec!["Memory".to_string(), "Project".to_string()],
            observations: vec!["A project for managing memory".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        let related_entity = MemoryEntity {
            name: "tech:language:rust".to_string(),
            labels: vec![
                "Memory".to_string(),
                "Technology".to_string(),
                "Language".to_string(),
            ],
            observations: vec!["A systems programming language".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        mock_repo
            .expect_find_related_entities()
            .with(
                eq("andoriyu:project:middle_manager"),
                always(),
                always(),
                always(),
            )
            .times(1)
            .returning(move |_, _, _, _| Ok(vec![project_entity2.clone(), related_entity.clone()]));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = GetProjectContextCommand {
            filter: ProjectFilter::Repository("andoriyu/middle-manager".to_string()),
        };

        let result = get_project_context(&ports, command).await.unwrap();

        assert_eq!(
            result.context.project.name,
            "andoriyu:project:middle_manager"
        );
        assert_eq!(result.context.technologies.len(), 1);
        assert_eq!(result.context.technologies[0].name, "tech:language:rust");
    }

    #[tokio::test]
    async fn test_get_project_context_not_found() {
        let mut mock_repo = MockMemoryRepository::new();

        mock_repo
            .expect_find_entity_by_name()
            .with(eq("nonexistent:project"))
            .times(1)
            .returning(|_| Ok(None));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = GetProjectContextCommand {
            filter: ProjectFilter::Name("nonexistent:project".to_string()),
        };

        let result = get_project_context(&ports, command).await;
        assert!(result.is_err());

        match result {
            Err(CoreError::Memory(MemoryError::EntityNotFound(_))) => {
                // This is the expected error
            }
            _ => panic!("Expected EntityNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_project_context_not_a_project() {
        let mut mock_repo = MockMemoryRepository::new();

        // Entity exists but is not a project
        let entity = MemoryEntity {
            name: "tech:language:rust".to_string(),
            labels: vec!["Memory".to_string(), "Technology".to_string()],
            observations: vec!["A systems programming language".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        mock_repo
            .expect_find_entity_by_name()
            .with(eq("tech:language:rust"))
            .times(1)
            .returning(move |_| Ok(Some(entity.clone())));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = GetProjectContextCommand {
            filter: ProjectFilter::Name("tech:language:rust".to_string()),
        };

        let result = get_project_context(&ports, command).await;
        assert!(result.is_err());

        match result {
            Err(CoreError::Memory(MemoryError::EntityNotFound(_))) => {
                // This is the expected error
            }
            _ => panic!("Expected EntityNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_project_context_repository_not_found() {
        let mut mock_repo = MockMemoryRepository::new();

        mock_repo
            .expect_find_entities_by_labels()
            .with(
                eq(vec!["GitRepository".to_string()]),
                eq(LabelMatchMode::All),
                always(),
            )
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = GetProjectContextCommand {
            filter: ProjectFilter::Repository("nonexistent/repo".to_string()),
        };

        let result = get_project_context(&ports, command).await;
        assert!(result.is_err());

        match result {
            Err(CoreError::Memory(MemoryError::EntityNotFound(_))) => {
                // This is the expected error
            }
            _ => panic!("Expected EntityNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_get_project_context_no_project_for_repository() {
        let mut mock_repo = MockMemoryRepository::new();

        // Repository exists but has no related project
        let repo_entity = MemoryEntity {
            name: "tech:git:repo:andoriyu/no-project".to_string(),
            labels: vec!["Memory".to_string(), "GitRepository".to_string()],
            observations: vec!["A git repository with no project".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        mock_repo
            .expect_find_entities_by_labels()
            .with(
                eq(vec!["GitRepository".to_string()]),
                eq(LabelMatchMode::All),
                always(),
            )
            .times(1)
            .returning(move |_, _, _| Ok(vec![repo_entity.clone()]));

        mock_repo
            .expect_find_related_entities()
            .with(
                eq("tech:git:repo:andoriyu/no-project"),
                eq(Some("contains".to_string())),
                eq(Some(RelationshipDirection::Outgoing)),
                eq(1),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(vec![]));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let command = GetProjectContextCommand {
            filter: ProjectFilter::Repository("andoriyu/no-project".to_string()),
        };

        let result = get_project_context(&ports, command).await;
        assert!(result.is_err());

        match result {
            Err(CoreError::Memory(MemoryError::EntityNotFound(_))) => {
                // This is the expected error
            }
            _ => panic!("Expected EntityNotFound error"),
        }
    }
}
