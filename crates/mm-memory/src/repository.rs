use async_trait::async_trait;
use std::error::Error as StdError;

use crate::entity::MemoryEntity;
use crate::error::MemoryResult;
use crate::label_match_mode::LabelMatchMode;
use crate::relationship::MemoryRelationship;
use crate::relationship_direction::RelationshipDirection;
use crate::update::{EntityUpdate, RelationshipUpdate};

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

    async fn delete_entities(&self, names: &[String]) -> MemoryResult<(), Self::Error>;

    async fn delete_relationships(
        &self,
        relationships: &[crate::relationship::RelationshipRef],
    ) -> MemoryResult<(), Self::Error>;

    async fn find_relationships(
        &self,
        from: Option<&str>,
        to: Option<&str>,
        name: Option<&str>,
    ) -> MemoryResult<Vec<MemoryRelationship>, Self::Error>;

    async fn find_entities_by_labels(
        &self,
        labels: &[String],
        match_mode: LabelMatchMode,
        required_label: Option<String>,
    ) -> MemoryResult<Vec<MemoryEntity>, Self::Error>;

    async fn find_related_entities(
        &self,
        name: &str,
        relationship_type: Option<String>,
        direction: Option<RelationshipDirection>,
        depth: u32,
    ) -> MemoryResult<Vec<MemoryEntity>, Self::Error>;

    async fn update_entity(
        &self,
        name: &str,
        update: &EntityUpdate,
    ) -> MemoryResult<(), Self::Error>;

    async fn update_relationship(
        &self,
        from: &str,
        to: &str,
        name: &str,
        update: &RelationshipUpdate,
    ) -> MemoryResult<(), Self::Error>;
}
