use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use crate::validate_name;
use mm_memory::MemoryRepository;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct RemoveObservationsCommand {
    pub name: String,
    pub observations: Vec<String>,
}

pub type RemoveObservationsResult<E> = CoreResult<(), E>;

#[instrument(skip(ports), fields(name = %command.name, observations_count = command.observations.len()))]
pub async fn remove_observations<R>(
    ports: &Ports<R>,
    command: RemoveObservationsCommand,
) -> RemoveObservationsResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    validate_name!(command.name);

    ports
        .memory_service
        .remove_observations(&command.name, &command.observations)
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
    async fn test_remove_observations_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_remove_observations()
            .withf(|name, obs| name == "test:entity" && obs.len() == 1)
            .returning(|_, _| Ok(()));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let command = RemoveObservationsCommand {
            name: "test:entity".to_string(),
            observations: vec!["obs".to_string()],
        };
        let result = remove_observations(&ports, command).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_observations_empty_name() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_remove_observations().never();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let command = RemoveObservationsCommand {
            name: "".to_string(),
            observations: vec![],
        };
        let result = remove_observations(&ports, command).await;
        assert!(matches!(
            result,
            Err(CoreError::Validation(ref e)) if e.0.contains(&ValidationErrorKind::EmptyEntityName)
        ));
    }

    #[tokio::test]
    async fn test_remove_observations_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_remove_observations()
            .withf(|name, _| name == "test:entity")
            .returning(|_, _| Err(MemoryError::runtime_error("fail")));
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));
        let command = RemoveObservationsCommand {
            name: "test:entity".to_string(),
            observations: vec!["obs".to_string()],
        };
        let result = remove_observations(&ports, command).await;
        assert!(matches!(result, Err(CoreError::Memory(_))));
    }

    use arbitrary::Arbitrary;
    use mm_utils::prop::{NonEmptyName, async_arbtest};

    #[test]
    fn prop_remove_observations_success() {
        async_arbtest(|rt, u| {
            let NonEmptyName(name) = NonEmptyName::arbitrary(u)?;
            let observations = <Vec<String>>::arbitrary(u)?;
            let mut mock = MockMemoryRepository::new();
            let name_clone = name.clone();
            let obs_clone = observations.clone();
            mock.expect_remove_observations()
                .withf(move |n, o| n == name_clone && o == obs_clone.as_slice())
                .returning(|_, _| Ok(()));
            let service = MemoryService::new(mock, MemoryConfig::default());
            let ports = Ports::new(Arc::new(service));
            let command = RemoveObservationsCommand { name, observations };
            let result = rt.block_on(remove_observations(&ports, command));
            assert!(result.is_ok());
            Ok(())
        });
    }

    #[test]
    fn prop_remove_observations_empty_name() {
        async_arbtest(|rt, u| {
            let observations = <Vec<String>>::arbitrary(u)?;
            let mut mock = MockMemoryRepository::new();
            mock.expect_remove_observations().never();
            let service = MemoryService::new(mock, MemoryConfig::default());
            let ports = Ports::new(Arc::new(service));
            let command = RemoveObservationsCommand {
                name: String::default(),
                observations,
            };
            let result = rt.block_on(remove_observations(&ports, command));
            assert!(matches!(result, Err(CoreError::Validation(_))));
            Ok(())
        });
    }
}
