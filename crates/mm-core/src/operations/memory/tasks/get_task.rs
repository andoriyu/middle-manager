use super::types::TaskProperties;
use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use crate::validate_name;
use mm_memory::{MemoryEntity, MemoryRepository};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct GetTaskCommand {
    pub name: String,
}

pub type GetTaskResult<E> = CoreResult<Option<MemoryEntity<TaskProperties>>, E>;

#[instrument(skip(ports), fields(name = %command.name))]
pub async fn get_task<R>(ports: &Ports<R>, command: GetTaskCommand) -> GetTaskResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    validate_name!(command.name);

    ports
        .memory_service
        .find_entity_by_name_typed::<TaskProperties>(&command.name)
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
    async fn test_get_task_success() {
        let mut mock = MockMemoryRepository::new();
        let entity = MemoryEntity {
            name: "task:1".into(),
            labels: vec!["Task".into()],
            ..Default::default()
        };
        mock.expect_find_entity_by_name()
            .with(eq("task:1"))
            .returning(move |_| Ok(Some(entity.clone())));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let cmd = GetTaskCommand {
            name: "task:1".into(),
        };
        let res = get_task(&ports, cmd).await.unwrap();
        assert!(res.is_some());
    }

    #[tokio::test]
    async fn test_get_task_empty_name() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entity_by_name().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let cmd = GetTaskCommand {
            name: String::new(),
        };
        let res = get_task(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::Validation(_))));
    }
}
