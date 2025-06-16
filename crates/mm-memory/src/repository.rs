use async_trait::async_trait;
use std::error::Error as StdError;

use crate::entity::MemoryEntity;
use crate::error::MemoryResult;

#[cfg_attr(any(test, feature = "mock"), mockall::automock(type Error = std::convert::Infallible;))]
#[async_trait]
pub trait MemoryRepository {
    type Error: StdError + Send + Sync + 'static;

    async fn create_entity(&self, entity: &MemoryEntity) -> MemoryResult<(), Self::Error>;
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
}
