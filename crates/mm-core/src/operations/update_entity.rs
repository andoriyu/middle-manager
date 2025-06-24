use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use crate::validate_name;
use mm_git::GitServiceTrait;
use mm_memory::{EntityUpdate, MemoryRepository};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct UpdateEntityCommand {
    pub name: String,
    pub update: EntityUpdate,
}

pub type UpdateEntityResult<E> = CoreResult<(), E>;

#[instrument(skip(ports), fields(name = %command.name))]
pub async fn update_entity<R, G>(
    ports: &Ports<R, G>,
    command: UpdateEntityCommand,
) -> UpdateEntityResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
    G: GitServiceTrait + Send + Sync,
{
    validate_name!(command.name);

    ports
        .memory_service
        .update_entity(&command.name, &command.update)
        .await
        .map_err(CoreError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_update_entity_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_update_entity()
            .withf(|n, _| n == "test:entity")
            .returning(|_, _| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service), Arc::new(mm_git::NoopGitService));
        let cmd = UpdateEntityCommand {
            name: "test:entity".into(),
            update: EntityUpdate::default(),
        };
        let res = update_entity(&ports, cmd).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_update_entity_empty_name() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_update_entity().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service), Arc::new(mm_git::NoopGitService));
        let cmd = UpdateEntityCommand {
            name: "".into(),
            update: EntityUpdate::default(),
        };
        let res = update_entity(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::Validation(_))));
    }
}
