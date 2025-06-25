use mm_git::GitRepository;
use mm_memory::{
    MemoryEntity, MemoryError, MemoryRepository, ProjectContext, RelationshipDirection,
};
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
pub async fn get_project_context<M, G>(
    ports: &Ports<M, G>,
    command: GetProjectContextCommand,
) -> CoreResult<GetProjectContextResult, M::Error>
where
    M: MemoryRepository + Send + Sync,
    G: GitRepository + Send + Sync,
    M::Error: std::error::Error + Send + Sync + 'static,
    G::Error: std::error::Error + Send + Sync + 'static,
{
    match command.filter {
        ProjectFilter::Name(name) => {
            // Try to find the project by name
            let project_entity = ports
                .memory_service
                .find_entity_by_name(&name)
                .await
                .map_err(CoreError::from)?;

            if let Some(entity) = project_entity {
                let context = build_project_context(ports, entity).await?;
                Ok(GetProjectContextResult { context })
            } else {
                Err(CoreError::Memory(MemoryError::entity_not_found(name)))
            }
        }
        ProjectFilter::Repository(repo_name) => {
            // Try to find the project by repository name
            let repo_name = format!("tech:git:repo:{}", repo_name);
            let repo_entity = ports
                .memory_service
                .find_entity_by_name(&repo_name)
                .await
                .map_err(CoreError::from)?;

            if let Some(repo) = repo_entity {
                // Find projects contained by this repository
                let projects = ports
                    .memory_service
                    .find_related_entities(
                        &repo.name,
                        Some("contains".to_string()),
                        Some(RelationshipDirection::Outgoing),
                        1,
                    )
                    .await
                    .map_err(CoreError::from)?
                    .into_iter()
                    .filter(|e| e.labels.contains(&"Project".to_string()))
                    .collect::<Vec<_>>();

                if projects.is_empty() {
                    Err(CoreError::Memory(MemoryError::entity_not_found(format!(
                        "No projects found for repository {}",
                        repo_name
                    ))))
                } else if projects.len() > 1 {
                    debug!(
                        "Multiple projects found for repository {}, using first one",
                        repo_name
                    );
                    let context = build_project_context(ports, projects[0].clone()).await?;
                    Ok(GetProjectContextResult { context })
                } else {
                    let context = build_project_context(ports, projects[0].clone()).await?;
                    Ok(GetProjectContextResult { context })
                }
            } else {
                Err(CoreError::Memory(MemoryError::entity_not_found(repo_name)))
            }
        }
    }
}

/// Build project context from a project entity
async fn build_project_context<M, G>(
    ports: &Ports<M, G>,
    project: MemoryEntity,
) -> CoreResult<ProjectContext, M::Error>
where
    M: MemoryRepository + Send + Sync,
    G: GitRepository + Send + Sync,
    M::Error: std::error::Error + Send + Sync + 'static,
    G::Error: std::error::Error + Send + Sync + 'static,
{
    // Find tasks related to this project
    let tasks = ports
        .memory_service
        .find_related_entities(
            &project.name,
            Some("contains".to_string()),
            Some(RelationshipDirection::Outgoing),
            1,
        )
        .await
        .map_err(CoreError::from)?
        .into_iter()
        .filter(|e| e.labels.contains(&"Task".to_string()))
        .collect::<Vec<_>>();

    // Find notes related to this project
    let notes = ports
        .memory_service
        .find_related_entities(
            &project.name,
            Some("relates_to".to_string()),
            Some(RelationshipDirection::Incoming),
            1,
        )
        .await
        .map_err(CoreError::from)?
        .into_iter()
        .filter(|e| e.labels.contains(&"Note".to_string()))
        .collect::<Vec<_>>();

    // Find components related to this project
    let components = ports
        .memory_service
        .find_related_entities(
            &project.name,
            Some("contains".to_string()),
            Some(RelationshipDirection::Outgoing),
            1,
        )
        .await
        .map_err(CoreError::from)?
        .into_iter()
        .filter(|e| e.labels.contains(&"Component".to_string()))
        .collect::<Vec<_>>();

    // Find other entities related to this project
    let other_related = ports
        .memory_service
        .find_related_entities(&project.name, None, Some(RelationshipDirection::Both), 1)
        .await
        .map_err(CoreError::from)?
        .into_iter()
        .filter(|e| {
            !e.labels.contains(&"Task".to_string())
                && !e.labels.contains(&"Note".to_string())
                && !e.labels.contains(&"Component".to_string())
                && !e.labels.contains(&"Technology".to_string())
        })
        .collect();

    // Find technologies used by this project
    let technologies = ports
        .memory_service
        .find_related_entities(
            &project.name,
            Some("uses".to_string()),
            Some(RelationshipDirection::Outgoing),
            1,
        )
        .await
        .map_err(CoreError::from)?
        .into_iter()
        .filter(|e| e.labels.contains(&"Technology".to_string()))
        .collect();

    Ok(ProjectContext {
        project,
        tasks,
        notes,
        components,
        other_related_entities: other_related,
        technologies,
    })
}
