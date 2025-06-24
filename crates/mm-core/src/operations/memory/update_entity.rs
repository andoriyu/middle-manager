use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use crate::validate_name;
use mm_git::GitRepository;
use mm_memory::{EntityUpdate, MemoryRepository};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct UpdateEntityCommand {
    pub name: String,
    pub update: EntityUpdate,
}

pub type UpdateEntityResult<E> = CoreResult<(), E>;

#[instrument(skip(ports), fields(name = %command.name))]
pub async fn update_entity<MR, GR>(
    ports: &Ports<MR, GR>,
    command: UpdateEntityCommand,
) -> UpdateEntityResult<MR::Error>
where
    MR: MemoryRepository + Send + Sync,
    MR::Error: std::error::Error + Send + Sync + 'static,
    GR: GitRepository + Send + Sync,
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
    use mm_git::GitService;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_update_entity_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_update_entity()
            .withf(|n, _| n == "test:entity")
            .returning(|_, _| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports {
            memory_service: Arc::new(service),
            ..Ports::new_noop()
        };
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
        let ports = Ports {
            memory_service: Arc::new(service),
            ..Ports::new_noop()
        };
        let cmd = UpdateEntityCommand {
            name: "".into(),
            update: EntityUpdate::default(),
        };
        let res = update_entity(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::Validation(_))));
    }
}
