use mm_git::GitServiceTrait;
use mm_memory::{LabelMatchMode, MemoryEntity};
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
pub async fn list_projects<R, G>(
    ports: &Ports<R, G>,
    command: ListProjectsCommand,
) -> CoreResult<ListProjectsResult, R::Error>
where
    R: mm_memory::MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
    G: GitServiceTrait + Send + Sync,
{
    // Find all projects
    let mut projects = ports
        .memory_service
        .find_entities_by_labels(&["Project".to_string()], LabelMatchMode::All, None)
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
            labels: vec!["Memory".to_string(), "Project".to_string()],
            observations: vec!["A project for managing memory".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        let project2 = MemoryEntity {
            name: "andoriyu:project:flakes".to_string(),
            labels: vec!["Memory".to_string(), "Project".to_string()],
            observations: vec!["A project for managing Nix flakes".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        mock_repo
            .expect_find_entities_by_labels()
            .with(
                eq(vec!["Project".to_string()]),
                eq(LabelMatchMode::All),
                always(),
            )
            .times(1)
            .returning(move |_, _, _| Ok(vec![project1.clone(), project2.clone()]));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service), Arc::new(mm_git::NoopGitService));

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
            labels: vec!["Memory".to_string(), "Project".to_string()],
            observations: vec!["A project for managing memory".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        let project2 = MemoryEntity {
            name: "andoriyu:project:flakes".to_string(),
            labels: vec!["Memory".to_string(), "Project".to_string()],
            observations: vec!["A project for managing Nix flakes".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        mock_repo
            .expect_find_entities_by_labels()
            .with(
                eq(vec!["Project".to_string()]),
                eq(LabelMatchMode::All),
                always(),
            )
            .times(1)
            .returning(move |_, _, _| Ok(vec![project1.clone(), project2.clone()]));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service), Arc::new(mm_git::NoopGitService));

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
            .expect_find_entities_by_labels()
            .with(
                eq(vec!["Project".to_string()]),
                eq(LabelMatchMode::All),
                always(),
            )
            .times(1)
            .returning(move |_, _, _| Ok(vec![]));

        let service = MemoryService::new(mock_repo, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service), Arc::new(mm_git::NoopGitService));

        let command = ListProjectsCommand { name_filter: None };

        let result = list_projects(&ports, command).await.unwrap();

        assert_eq!(result.projects.len(), 0);
    }
}
