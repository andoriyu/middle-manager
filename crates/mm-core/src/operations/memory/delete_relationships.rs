use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use mm_git::GitRepository;
use mm_memory::{MemoryRepository, relationship::RelationshipRef};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct DeleteRelationshipsCommand {
    pub relationships: Vec<RelationshipRef>,
}

pub type DeleteRelationshipsResult<E> = CoreResult<(), E>;

#[instrument(skip(ports), fields(rel_count = command.relationships.len()))]
pub async fn delete_relationships<M, G>(
    ports: &Ports<M, G>,
    command: DeleteRelationshipsCommand,
) -> DeleteRelationshipsResult<M::Error>
where
    M: MemoryRepository + Send + Sync,
    G: GitRepository + Send + Sync,
    M::Error: std::error::Error + Send + Sync + 'static,
    G::Error: std::error::Error + Send + Sync + 'static,
{
    let errors = ports
        .memory_service
        .delete_relationships(&command.relationships)
        .await
        .map_err(CoreError::from)?;

    if !errors.is_empty() {
        return Err(CoreError::BatchValidation(errors));
    }

    Ok(())
}
