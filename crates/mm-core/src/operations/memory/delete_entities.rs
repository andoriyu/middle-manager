use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use mm_git::GitRepository;
use mm_memory::MemoryRepository;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct DeleteEntitiesCommand {
    pub names: Vec<String>,
}

pub type DeleteEntitiesResult<E> = CoreResult<(), E>;

#[instrument(skip(ports), fields(names_count = command.names.len()))]
pub async fn delete_entities<MR, GR>(
    ports: &Ports<MR, GR>,
    command: DeleteEntitiesCommand,
) -> DeleteEntitiesResult<MR::Error>
where
    MR: MemoryRepository + Send + Sync,
    MR::Error: std::error::Error + Send + Sync + 'static,
    GR: GitRepository + Send + Sync,
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
