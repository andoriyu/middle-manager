use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use crate::validate_name;
use mm_memory::MemoryRepository;

#[derive(Debug, Clone)]
pub struct RemoveAllObservationsCommand {
    pub name: String,
}

pub type RemoveAllObservationsResult<E> = CoreResult<(), E>;

pub async fn remove_all_observations<R>(
    ports: &Ports<R>,
    command: RemoveAllObservationsCommand,
) -> RemoveAllObservationsResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    validate_name!(command.name);

    ports
        .memory_service
        .remove_all_observations(&command.name)
        .await
        .map_err(CoreError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_memory::{
        MemoryConfig, MemoryError, MemoryService, MockMemoryRepository, ValidationErrorKind,
    };
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
            Err(CoreError::Validation(ref e)) if e.0.contains(&ValidationErrorKind::EmptyEntityName)
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
        assert!(matches!(result, Err(CoreError::Memory(_))));
    }
}
