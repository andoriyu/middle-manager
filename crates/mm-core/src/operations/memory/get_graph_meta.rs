use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use mm_git::GitRepository;
use mm_memory::{MemoryEntity, MemoryRepository, RelationshipDirection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// Name of the root memory graph entity
pub const GRAPH_ROOT: &str = "tech:tool:memory_graph";
/// Default traversal depth
const DEPTH: u32 = 5;

/// Command for retrieving graph metadata
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetGraphMetaCommand {
    /// Optional relationship type filter
    pub relationship: Option<String>,
}

/// Result containing entities related to the memory graph root
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetGraphMetaResult {
    /// Entities found
    pub entities: Vec<MemoryEntity>,
}

pub type GetGraphMetaResultType<E> = CoreResult<GetGraphMetaResult, E>;

#[instrument(skip(ports), err)]
pub async fn get_graph_meta<M, G>(
    ports: &Ports<M, G>,
    command: GetGraphMetaCommand,
) -> GetGraphMetaResultType<M::Error>
where
    M: MemoryRepository + Send + Sync,
    G: GitRepository + Send + Sync,
    M::Error: std::error::Error + Send + Sync + 'static,
    G::Error: std::error::Error + Send + Sync + 'static,
{
    let entities = ports
        .memory_service
        .find_related_entities(
            GRAPH_ROOT,
            command.relationship.clone(),
            Some(RelationshipDirection::Outgoing),
            DEPTH,
        )
        .await
        .map_err(CoreError::from)?;

    Ok(GetGraphMetaResult { entities })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::Ports;
    use mm_memory::{MemoryConfig, MemoryEntity, MemoryError, MemoryService, MockMemoryRepository};
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_graph_meta_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_related_entities()
            .with(
                eq(GRAPH_ROOT),
                eq(Some("rel".to_string())),
                eq(Some(RelationshipDirection::Outgoing)),
                eq(DEPTH),
            )
            .returning(|_, _, _, _| Ok(vec![MemoryEntity::default()]));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        let cmd = GetGraphMetaCommand {
            relationship: Some("rel".to_string()),
        };
        let result = get_graph_meta(&ports, cmd).await.unwrap();
        assert_eq!(result.entities.len(), 1);
    }

    #[tokio::test]
    async fn test_get_graph_meta_repo_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_related_entities()
            .returning(|_, _, _, _| Err(MemoryError::query_error("fail")));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        let cmd = GetGraphMetaCommand { relationship: None };
        let res = get_graph_meta(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::Memory(_))));
    }
}
