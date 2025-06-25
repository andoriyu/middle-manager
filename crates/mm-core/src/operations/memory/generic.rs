use crate::error::{CoreError, CoreResult};
use crate::ports::Ports;
use crate::validate_name;
use mm_git::GitRepository;
use mm_memory::{EntityUpdate, MemoryEntity, MemoryRepository, value::MemoryValue};
use schemars::JsonSchema;
use std::collections::HashMap;
use tracing::instrument;

pub type UpdateEntityGenericResult<E> = CoreResult<(), E>;

#[instrument(skip(ports), fields(name = %name))]
pub async fn update_entity_generic<M, G>(
    ports: &Ports<M, G>,
    name: &str,
    update: &EntityUpdate,
) -> UpdateEntityGenericResult<M::Error>
where
    M: MemoryRepository + Send + Sync,
    G: GitRepository + Send + Sync,
    M::Error: std::error::Error + Send + Sync + 'static,
    G::Error: std::error::Error + Send + Sync + 'static,
{
    validate_name!(name);

    ports
        .memory_service
        .update_entity(name, update)
        .await
        .map_err(CoreError::from)
}

pub type GetEntityGenericResult<P, E> = CoreResult<Option<MemoryEntity<P>>, E>;

#[instrument(skip(ports), fields(name = %name))]
pub async fn get_entity_generic<M, G, P>(
    ports: &Ports<M, G>,
    name: &str,
) -> GetEntityGenericResult<P, M::Error>
where
    M: MemoryRepository + Send + Sync,
    G: GitRepository + Send + Sync,
    M::Error: std::error::Error + Send + Sync + 'static,
    G::Error: std::error::Error + Send + Sync + 'static,
    P: JsonSchema
        + From<HashMap<String, MemoryValue>>
        + Into<HashMap<String, MemoryValue>>
        + Clone
        + std::fmt::Debug
        + Default,
{
    validate_name!(name);

    ports
        .memory_service
        .find_entity_by_name_typed::<P>(name)
        .await
        .map_err(CoreError::from)
}
