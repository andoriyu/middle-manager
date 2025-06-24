use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use mm_memory::MemoryRepository;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct DeleteEntitiesCommand {
    pub names: Vec<String>,
}

pub type DeleteEntitiesResult<E> = CoreResult<(), E>;

#[instrument(skip(ports), fields(names_count = command.names.len()))]
pub async fn delete_entities<R>(
    ports: &Ports<R>,
    command: DeleteEntitiesCommand,
) -> DeleteEntitiesResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    let errors = ports
        .memory_service
        .delete_entities(&command.names)
        .await
        .map_err(CoreError::from)?;

    if !errors.is_empty() {
        return Err(CoreError::BatchValidation(errors));
    }

    Ok(())
}
