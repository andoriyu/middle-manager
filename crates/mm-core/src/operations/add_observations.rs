use crate::error::CoreError;
use crate::ports::Ports;
use mm_memory::MemoryRepository;
use thiserror::Error;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AddObservationsCommand {
    pub name: String,
    pub observations: Vec<String>,
}

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum AddObservationsError<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    #[error("Repository error: {0}")]
    Repository(#[from] CoreError<E>),

    #[error("Validation error: {0}")]
    Validation(String),
}

#[allow(dead_code)]
pub type AddObservationsResult<E> = Result<(), AddObservationsError<E>>;

#[allow(dead_code)]
pub async fn add_observations<R>(
    ports: &Ports<R>,
    command: AddObservationsCommand,
) -> AddObservationsResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    if command.name.is_empty() {
        return Err(AddObservationsError::Validation(
            "Entity name cannot be empty".to_string(),
        ));
    }

    match ports
        .memory_service
        .add_observations(&command.name, &command.observations)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(AddObservationsError::Repository(CoreError::from(e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
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
        assert!(matches!(result, Err(AddObservationsError::Validation(_))));
    }
}
