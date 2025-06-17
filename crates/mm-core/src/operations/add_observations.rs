use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use mm_memory::{MemoryRepository, ValidationError, ValidationErrorKind};

#[derive(Debug, Clone)]
pub struct AddObservationsCommand {
    pub name: String,
    pub observations: Vec<String>,
}

pub type AddObservationsResult<E> = CoreResult<(), E>;

pub async fn add_observations<R>(
    ports: &Ports<R>,
    command: AddObservationsCommand,
) -> AddObservationsResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    if command.name.is_empty() {
        return Err(CoreError::Validation(ValidationError(vec![
            ValidationErrorKind::EmptyEntityName,
        ])));
    }

    ports
        .memory_service
        .add_observations(&command.name, &command.observations)
        .await
        .map_err(CoreError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_memory::{MemoryConfig, MemoryError, MemoryService, MockMemoryRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_add_observations_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_add_observations()
            .withf(|name, obs| name == "test:entity" && obs.len() == 1)
            .returning(|_, _| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let command = AddObservationsCommand {
            name: "test:entity".to_string(),
            observations: vec!["obs".to_string()],
        };
        let result = add_observations(&ports, command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_observations_empty_name() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_add_observations().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let command = AddObservationsCommand {
            name: "".to_string(),
            observations: vec![],
        };
        let result = add_observations(&ports, command).await;
        assert!(matches!(
            result,
            Err(CoreError::Validation(ref e)) if e.0.contains(&ValidationErrorKind::EmptyEntityName)
        ));
    }

    #[tokio::test]
    async fn test_add_observations_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_add_observations()
            .withf(|name, _| name == "test:entity")
            .returning(|_, _| Err(MemoryError::runtime_error("fail")));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let command = AddObservationsCommand {
            name: "test:entity".to_string(),
            observations: vec!["obs".to_string()],
        };
        let result = add_observations(&ports, command).await;
        assert!(matches!(result, Err(CoreError::Memory(_))));
    }
}
