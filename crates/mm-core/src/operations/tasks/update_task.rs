use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use crate::validate_name;
use mm_memory::{EntityUpdate, MemoryRepository};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct UpdateTaskCommand {
    pub name: String,
    pub update: EntityUpdate,
}

pub type UpdateTaskResult<E> = CoreResult<(), E>;

#[instrument(skip(ports), fields(name = %command.name))]
pub async fn update_task<R>(
    ports: &Ports<R>,
    command: UpdateTaskCommand,
) -> UpdateTaskResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
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
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_update_task_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_update_entity()
            .withf(|n, _| n == "task:1")
            .returning(|_, _| Ok(()));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let cmd = UpdateTaskCommand {
            name: "task:1".into(),
            update: EntityUpdate::default(),
        };
        let res = update_task(&ports, cmd).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_update_task_empty_name() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_update_entity().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let cmd = UpdateTaskCommand {
            name: String::new(),
            update: EntityUpdate::default(),
        };
        let res = update_task(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::Validation(_))));
    }
}
