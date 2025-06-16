use crate::error::CoreError;
use crate::ports::Ports;
use mm_memory::MemoryRepository;
use thiserror::Error;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SetObservationsCommand {
    pub name: String,
    pub observations: Vec<String>,
}

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum SetObservationsError<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    #[error("Repository error: {0}")]
    Repository(#[from] CoreError<E>),

    #[error("Validation error: {0}")]
    Validation(String),
}

#[allow(dead_code)]
pub type SetObservationsResult<E> = Result<(), SetObservationsError<E>>;

#[allow(dead_code)]
pub async fn set_observations<R>(
    ports: &Ports<R>,
    command: SetObservationsCommand,
) -> SetObservationsResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    if command.name.is_empty() {
        return Err(SetObservationsError::Validation(
            "Entity name cannot be empty".to_string(),
        ));
    }

    match ports
        .memory_service
        .set_observations(&command.name, &command.observations)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(SetObservationsError::Repository(CoreError::from(e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_set_observations_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_set_observations()
            .withf(|name, obs| name == "test:entity" && obs.len() == 1)
            .returning(|_, _| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let command = SetObservationsCommand {
            name: "test:entity".to_string(),
            observations: vec!["obs".to_string()],
        };
        let result = set_observations(&ports, command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_observations_empty_name() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_set_observations().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let command = SetObservationsCommand {
            name: "".to_string(),
            observations: vec![],
        };
        let result = set_observations(&ports, command).await;
        assert!(matches!(result, Err(SetObservationsError::Validation(_))));
    }
}
