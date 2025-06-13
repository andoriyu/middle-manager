use async_trait::async_trait;
use std::error::Error as StdError;

use crate::entity::MemoryEntity;
use crate::error::MemoryResult;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait MemoryRepository<E>
where
    E: StdError + Send + Sync + 'static,
{
    async fn create_entity(&self, entity: &MemoryEntity) -> MemoryResult<(), E>;
    async fn find_entity_by_name(&self, name: &str) -> MemoryResult<Option<MemoryEntity>, E>;
}
