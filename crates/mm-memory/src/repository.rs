use async_trait::async_trait;
use std::error::Error as StdError;

use crate::entity::MemoryEntity;
use crate::error::MemoryResult;
use crate::relationship::MemoryRelationship;
use crate::relationship_direction::RelationshipDirection;

#[cfg_attr(any(test, feature = "mock"), mockall::automock(type Error = std::convert::Infallible;))]
#[async_trait]
pub trait MemoryRepository {
    type Error: StdError + Send + Sync + 'static;

    async fn create_entities(&self, entities: &[MemoryEntity]) -> MemoryResult<(), Self::Error>;
    async fn find_entity_by_name(
        &self,
        name: &str,
    ) -> MemoryResult<Option<MemoryEntity>, Self::Error>;

    async fn set_observations(
        &self,
        name: &str,
        observations: &[String],
    ) -> MemoryResult<(), Self::Error>;

    async fn add_observations(
        &self,
        name: &str,
        observations: &[String],
    ) -> MemoryResult<(), Self::Error>;

    async fn remove_all_observations(&self, name: &str) -> MemoryResult<(), Self::Error>;

    async fn remove_observations(
        &self,
        name: &str,
        observations: &[String],
    ) -> MemoryResult<(), Self::Error>;

    async fn create_relationships(
        &self,
        relationships: &[MemoryRelationship],
    ) -> MemoryResult<(), Self::Error>;

    async fn find_related_entities(
        &self,
        name: &str,
        relationship_type: Option<String>,
        direction: Option<RelationshipDirection>,
        depth: u32,
    ) -> MemoryResult<Vec<MemoryEntity>, Self::Error>;
}
