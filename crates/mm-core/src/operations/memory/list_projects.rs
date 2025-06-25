use mm_git::GitRepository;
use mm_memory::labels::PROJECT_LABEL;
use mm_memory::{BasicEntityProperties, LabelMatchMode, MemoryEntity, MemoryRepository};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;

/// Command for listing projects
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListProjectsCommand {
    /// Optional name filter to narrow down results
    pub name_filter: Option<String>,
}

/// Result of listing projects
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListProjectsResult {
    /// List of available projects
    pub projects: Vec<MemoryEntity>,
}

/// List all available projects
#[instrument(skip(ports), err)]
pub async fn list_projects<M, G>(
    ports: &Ports<M, G>,
    command: ListProjectsCommand,
) -> CoreResult<ListProjectsResult, M::Error>
where
    M: MemoryRepository + Send + Sync,
    G: GitRepository + Send + Sync,
    M::Error: std::error::Error + Send + Sync + 'static,
    G::Error: std::error::Error + Send + Sync + 'static,
{
    // Find all projects
    let mut projects = ports
        .memory_service
        .find_entities_by_labels_typed::<BasicEntityProperties>(
            &[PROJECT_LABEL.to_string()],
            LabelMatchMode::All,
            None,
        )
        .await
        .map_err(CoreError::from)?;

    // Apply name filter if provided
    if let Some(filter) = command.name_filter {
        projects.retain(|p| {
            p.name.contains(&filter) || p.observations.iter().any(|o| o.contains(&filter))
        });
    }

    // Return the list of projects (may be empty)
    Ok(ListProjectsResult { projects })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_git::repository::MockGitRepository;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use mockall::predicate::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_list_projects_no_filter() {
        let mut mock_repo = MockMemoryRepository::new();

        // Setup mock for find_entities_by_labels
        let project1 = MemoryEntity {
            name: "andoriyu:project:middle_manager".to_string(),
            labels: vec!["Memory".to_string(), PROJECT_LABEL.to_string()],
            observations: vec!["A project for managing memory".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        let project2 = MemoryEntity {
            name: "andoriyu:project:flakes".to_string(),
            labels: vec!["Memory".to_string(), PROJECT_LABEL.to_string()],
            observations: vec!["A project for managing Nix flakes".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        mock_repo
            .expect_find_entities_by_labels_typed::<BasicEntityProperties>()
            .with(
                eq(vec![PROJECT_LABEL.to_string()]),
                eq(LabelMatchMode::All),
                always(),
            )
            .times(1)
            .returning(move |_, _, _| Ok(vec![project1.clone(), project2.clone()]));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));

        let command = ListProjectsCommand { name_filter: None };

        let result = list_projects(&ports, command).await.unwrap();

        assert_eq!(result.projects.len(), 2);
        assert_eq!(result.projects[0].name, "andoriyu:project:middle_manager");
        assert_eq!(result.projects[1].name, "andoriyu:project:flakes");
    }

    #[tokio::test]
    async fn test_list_projects_with_filter() {
        let mut mock_repo = MockMemoryRepository::new();

        // Setup mock for find_entities_by_labels
        let project1 = MemoryEntity {
            name: "andoriyu:project:middle_manager".to_string(),
            labels: vec!["Memory".to_string(), PROJECT_LABEL.to_string()],
            observations: vec!["A project for managing memory".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        let project2 = MemoryEntity {
            name: "andoriyu:project:flakes".to_string(),
            labels: vec!["Memory".to_string(), PROJECT_LABEL.to_string()],
            observations: vec!["A project for managing Nix flakes".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        mock_repo
            .expect_find_entities_by_labels_typed::<BasicEntityProperties>()
            .with(
                eq(vec![PROJECT_LABEL.to_string()]),
                eq(LabelMatchMode::All),
                always(),
            )
            .times(1)
            .returning(move |_, _, _| Ok(vec![project1.clone(), project2.clone()]));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));

        let command = ListProjectsCommand {
            name_filter: Some("flakes".to_string()),
        };

        let result = list_projects(&ports, command).await.unwrap();

        assert_eq!(result.projects.len(), 1);
        assert_eq!(result.projects[0].name, "andoriyu:project:flakes");
    }

    #[tokio::test]
    async fn test_list_projects_empty_result() {
        let mut mock_repo = MockMemoryRepository::new();

        // Setup mock for find_entities_by_labels
        mock_repo
            .expect_find_entities_by_labels_typed::<BasicEntityProperties>()
            .with(
                eq(vec![PROJECT_LABEL.to_string()]),
                eq(LabelMatchMode::All),
                always(),
            )
            .times(1)
            .returning(move |_, _, _| Ok(vec![]));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let git_repo = MockGitRepository::new();
        let git_service = mm_git::GitService::new(git_repo);
        let ports = Ports::new(Arc::new(service), Arc::new(git_service));

        let command = ListProjectsCommand { name_filter: None };

        let result = list_projects(&ports, command).await.unwrap();

        assert_eq!(result.projects.len(), 0);
    }
}
