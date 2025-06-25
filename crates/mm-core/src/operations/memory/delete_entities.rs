use super::common::handle_batch_result;
use crate::error::CoreResult;
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
pub async fn delete_entities<M, G>(
    ports: &Ports<M, G>,
    command: DeleteEntitiesCommand,
) -> DeleteEntitiesResult<M::Error>
where
    M: MemoryRepository + Send + Sync,
    G: GitRepository + Send + Sync,
    M::Error: std::error::Error + Send + Sync + 'static,
    G::Error: std::error::Error + Send + Sync + 'static,
{
    handle_batch_result(|| ports.memory_service.delete_entities(&command.names)).await
}
