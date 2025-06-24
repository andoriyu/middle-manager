use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use mm_memory::{MemoryRelationship, MemoryRepository};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct FindRelationshipsCommand {
    pub from: Option<String>,
    pub to: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct FindRelationshipsResult {
    pub relationships: Vec<MemoryRelationship>,
}

pub type FindRelationshipsResultType<E> = CoreResult<FindRelationshipsResult, E>;

#[instrument(skip(ports))]
pub async fn find_relationships<R>(
    ports: &Ports<R>,
    command: FindRelationshipsCommand,
) -> FindRelationshipsResultType<R::Error>
where
    R: MemoryRepository + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    let rels = ports
        .memory_service
        .find_relationships(
            command.from.clone(),
            command.to.clone(),
            command.name.clone(),
        )
        .await
        .map_err(CoreError::from)?;
    Ok(FindRelationshipsResult {
        relationships: rels,
    })
}
