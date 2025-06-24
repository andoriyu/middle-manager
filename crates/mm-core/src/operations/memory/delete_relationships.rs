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
pub async fn delete_relationships<MR, GR>(
    ports: &Ports<MR, GR>,
    command: DeleteRelationshipsCommand,
) -> DeleteRelationshipsResult<MR::Error>
where
    MR: MemoryRepository + Send + Sync,
    MR::Error: std::error::Error + Send + Sync + 'static,
    GR: GitRepository + Send + Sync,
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
