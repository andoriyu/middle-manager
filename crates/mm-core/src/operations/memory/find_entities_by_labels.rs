use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use mm_git::GitRepository;
use mm_memory::{LabelMatchMode, MemoryEntity, MemoryRepository};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct FindEntitiesByLabelsCommand {
    pub labels: Vec<String>,
    pub match_mode: LabelMatchMode,
    pub required_label: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct FindEntitiesByLabelsResult {
    pub entities: Vec<MemoryEntity>,
}

pub type FindEntitiesByLabelsResultType<E> = CoreResult<FindEntitiesByLabelsResult, E>;

#[instrument(skip(ports), fields(label_count = command.labels.len()))]
pub async fn find_entities_by_labels<M, G>(
    ports: &Ports<M, G>,
    command: FindEntitiesByLabelsCommand,
) -> FindEntitiesByLabelsResultType<M::Error>
where
    M: MemoryRepository + Send + Sync,
    G: GitRepository + Send + Sync,
    M::Error: std::error::Error + Send + Sync + 'static,
    G::Error: std::error::Error + Send + Sync + 'static,
{
    let entities = ports
        .memory_service
        .find_entities_by_labels(
            &command.labels,
            command.match_mode,
            command.required_label.clone(),
        )
        .await
        .map_err(CoreError::from)?;
    Ok(FindEntitiesByLabelsResult { entities })
}
