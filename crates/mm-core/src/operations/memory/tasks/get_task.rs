use super::types::TaskProperties;
#[cfg(test)]
use crate::error::CoreError;
#[cfg(test)]
use mm_memory::MemoryEntity;

generate_get_wrapper!(GetTaskCommand, get_task, GetTaskResult, TaskProperties);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::Ports;
    use mm_memory::labels::TASK_LABEL;
    use mm_memory::{MemoryConfig, MemoryService, MockMemoryRepository};
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_task_success() {
        let mut mock = MockMemoryRepository::new();
        let entity = MemoryEntity {
            name: "task:1".into(),
            labels: vec![TASK_LABEL.to_string()],
            ..Default::default()
        };
        mock.expect_find_entity_by_name()
            .with(eq("task:1"))
            .returning(move |_| Ok(Some(entity.clone())));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| {
            p.memory_service = Arc::new(service);
        });

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
        let ports = Ports::noop().with(|p| {
            p.memory_service = Arc::new(service);
        });

        let cmd = GetTaskCommand {
            name: String::new(),
        };
        let res = get_task(&ports, cmd).await;
        assert!(matches!(res, Err(CoreError::Validation(_))));
    }
}
