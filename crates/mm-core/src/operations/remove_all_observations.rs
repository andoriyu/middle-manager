use crate::error::CoreError;
use crate::ports::Ports;
use mm_memory::MemoryRepository;
use thiserror::Error;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RemoveAllObservationsCommand {
    pub name: String,
}

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum RemoveAllObservationsError<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    #[error("Repository error: {0}")]
    Repository(#[from] CoreError<E>),

    #[error("Validation error: {0}")]
    Validation(String),
}

#[allow(dead_code)]
pub type RemoveAllObservationsResult<E> = Result<(), RemoveAllObservationsError<E>>;

#[allow(dead_code)]
pub async fn remove_all_observations<R>(
    ports: &Ports<R>,
    command: RemoveAllObservationsCommand,
) -> RemoveAllObservationsResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    if command.name.is_empty() {
        return Err(RemoveAllObservationsError::Validation(
            "Entity name cannot be empty".to_string(),
        ));
    }

    match ports
        .memory_service
        .remove_all_observations(&command.name)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(RemoveAllObservationsError::Repository(CoreError::from(e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_memory::{MemoryConfig, MemoryError, MemoryService, MockMemoryRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_remove_all_observations_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_remove_all_observations()
            .withf(|name| name == "test:entity")
            .returning(|_| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let command = RemoveAllObservationsCommand {
            name: "test:entity".to_string(),
        };
        let result = remove_all_observations(&ports, command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_all_observations_empty_name() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_remove_all_observations().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let command = RemoveAllObservationsCommand {
            name: "".to_string(),
        };
        let result = remove_all_observations(&ports, command).await;
        assert!(matches!(
            result,
            Err(RemoveAllObservationsError::Validation(_))
        ));
    }

    #[tokio::test]
    async fn test_remove_all_observations_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_remove_all_observations()
            .withf(|name| name == "test:entity")
            .returning(|_| Err(MemoryError::runtime_error("fail")));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let command = RemoveAllObservationsCommand {
            name: "test:entity".to_string(),
        };
        let result = remove_all_observations(&ports, command).await;
        assert!(matches!(
            result,
            Err(RemoveAllObservationsError::Repository(CoreError::Memory(_)))
        ));
    }
}
