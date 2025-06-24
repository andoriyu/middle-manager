use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use mm_memory::{MemoryRepository, relationship::RelationshipRef};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct DeleteRelationshipsCommand {
    pub relationships: Vec<RelationshipRef>,
}

pub type DeleteRelationshipsResult<E> = CoreResult<(), E>;

#[instrument(skip(ports), fields(rel_count = command.relationships.len()))]
pub async fn delete_relationships<R>(
    ports: &Ports<R>,
    command: DeleteRelationshipsCommand,
) -> DeleteRelationshipsResult<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
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
