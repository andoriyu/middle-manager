use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use crate::validate_name;
use mm_git::GitRepository;
use mm_memory::{MemoryEntity, MemoryRepository, RelationshipDirection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct FindRelatedEntitiesCommand {
    pub name: String,
    pub relationship: Option<String>,
    pub direction: Option<RelationshipDirection>,
    pub depth: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct FindRelatedEntitiesResult {
    pub entities: Vec<MemoryEntity>,
}

pub type FindRelatedEntitiesResultType<E> = CoreResult<FindRelatedEntitiesResult, E>;

#[instrument(skip(ports), fields(name = %command.name, depth = command.depth))]
pub async fn find_related_entities<M, G>(
    ports: &Ports<M, G>,
    command: FindRelatedEntitiesCommand,
) -> FindRelatedEntitiesResultType<M::Error>
where
    M: MemoryRepository + Send + Sync,
    G: GitRepository + Send + Sync,
    M::Error: std::error::Error + Send + Sync + 'static,
    G::Error: std::error::Error + Send + Sync + 'static,
{
    validate_name!(command.name);

    let entities = ports
        .memory_service
        .find_related_entities(
            &command.name,
            command.relationship.clone(),
            command.direction,
            command.depth,
        )
        .await
        .map_err(CoreError::from)?;

    Ok(FindRelatedEntitiesResult { entities })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::Ports;
    use mm_memory::{MemoryConfig, MemoryError, MemoryService, MockMemoryRepository};
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_find_related_entities_success() {
        let mut mock = MockMemoryRepository::new();
        let expected = vec![MemoryEntity {
            name: "b".into(),
            ..Default::default()
        }];
        mock.expect_find_related_entities()
            .with(
                eq("a"),
                eq(Some("rel".to_string())),
                eq(Some(RelationshipDirection::Outgoing)),
                eq(2u32),
            )
            .returning(move |_, _, _, _| Ok(expected.clone()));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        let cmd = FindRelatedEntitiesCommand {
            name: "a".into(),
            relationship: Some("rel".into()),
            direction: Some(RelationshipDirection::Outgoing),
            depth: 2,
        };

        let res = find_related_entities(&ports, cmd).await.unwrap();
        assert_eq!(res.entities.len(), 1);
    }

    #[tokio::test]
    async fn test_find_related_entities_empty_name() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_related_entities().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        let cmd = FindRelatedEntitiesCommand {
            name: "".into(),
            relationship: None,
            direction: None,
            depth: 1,
        };

        let res = find_related_entities(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::Validation(_))));
    }

    #[tokio::test]
    async fn test_find_related_entities_repo_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_related_entities()
            .returning(|_, _, _, _| Err(MemoryError::query_error("fail")));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        let cmd = FindRelatedEntitiesCommand {
            name: "a".into(),
            relationship: None,
            direction: None,
            depth: 1,
        };

        let res = find_related_entities(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::Memory(_))));
    }
}
