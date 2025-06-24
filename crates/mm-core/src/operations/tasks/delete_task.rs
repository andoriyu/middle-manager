use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use crate::validate_name;
use mm_memory::MemoryRepository;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct DeleteTaskCommand {
    pub name: String,
}

pub type DeleteTaskResult<E> = CoreResult<(), E>;

#[instrument(skip(ports), fields(name = %command.name))]
pub async fn delete_task<R>(
    ports: &Ports<R>,
    command: DeleteTaskCommand,
) -> DeleteTaskResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    validate_name!(command.name);

    let errors = ports
        .memory_service
        .delete_entities(std::slice::from_ref(&command.name))
        .await
        .map_err(CoreError::from)?;

    if !errors.is_empty() {
        return Err(CoreError::BatchValidation(errors));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_delete_task_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_delete_entities()
            .withf(|n| n.len() == 1 && n[0] == "task:1")
            .returning(|_| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let cmd = DeleteTaskCommand {
            name: "task:1".into(),
        };
        let res = delete_task(&ports, cmd).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_delete_task_empty_name() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_delete_entities().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let cmd = DeleteTaskCommand {
            name: String::new(),
        };
        let res = delete_task(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::Validation(_))));
    }
}
