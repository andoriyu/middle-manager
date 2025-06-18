use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use crate::validate_name;
use mm_memory::MemoryRepository;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct AddObservationsCommand {
    pub name: String,
    pub observations: Vec<String>,
}

pub type AddObservationsResult<E> = CoreResult<(), E>;

#[instrument(skip(ports), fields(name = %command.name, observations_count = command.observations.len()))]
pub async fn add_observations<R>(
    ports: &Ports<R>,
    command: AddObservationsCommand,
) -> AddObservationsResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    validate_name!(command.name);

    ports
        .memory_service
        .add_observations(&command.name, &command.observations)
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

    use crate::test_utils::prop::{NonEmptyName, async_arbtest};
    use arbitrary::Arbitrary;

    #[test]
    fn prop_add_observations_success() {
        async_arbtest(|rt, u| {
            let NonEmptyName(name) = NonEmptyName::arbitrary(u)?;
            let observations = <Vec<String>>::arbitrary(u)?;
            let mut mock = MockMemoryRepository::new();
            let name_clone = name.clone();
            let obs_clone = observations.clone();
            mock.expect_add_observations()
                .withf(move |n, o| n == &name_clone && o == &obs_clone)
                .returning(|_, _| Ok(()));
            let service = MemoryService::new(mock, MemoryConfig::default());
            let ports = Ports::new(Arc::new(service));
            let command = AddObservationsCommand { name, observations };
            let result = rt.block_on(add_observations(&ports, command));
            assert!(result.is_ok());
            Ok(())
        });
    }

    #[test]
    fn prop_add_observations_empty_name() {
        async_arbtest(|rt, u| {
            let observations = <Vec<String>>::arbitrary(u)?;
            let mut mock = MockMemoryRepository::new();
            mock.expect_add_observations().never();
            let service = MemoryService::new(mock, MemoryConfig::default());
            let ports = Ports::new(Arc::new(service));
            let command = AddObservationsCommand {
                name: String::new(),
                observations,
            };
            let result = rt.block_on(add_observations(&ports, command));
            assert!(matches!(result, Err(CoreError::Validation(_))));
            Ok(())
        });
    }
}
