macro_rules! validate_name {
    ($name:expr) => {
        if $name.is_empty() {
            return Err($crate::error::CoreError::Validation(
                mm_memory::ValidationError(vec![mm_memory::ValidationErrorKind::EmptyEntityName]),
            ));
        }
    };
}

use crate::error::{CoreError, CoreResult};
use mm_memory::{MemoryError, ValidationError};

/// Handle the result of a batch memory service call.
///
/// This utility executes a future returned by `MemoryService` methods that
/// produce a list of `(name, ValidationError)` tuples on success. If the list
/// is empty the operation succeeded, otherwise a `CoreError::BatchValidation`
/// is returned.
pub async fn handle_batch_result<F, Fut, E>(fut: F) -> CoreResult<(), E>
where
    Fut: std::future::Future<Output = Result<Vec<(String, ValidationError)>, MemoryError<E>>>,
    F: FnOnce() -> Fut,
    E: std::error::Error + Send + Sync + 'static,
{
    let errors = fut().await.map_err(CoreError::from)?;
    if errors.is_empty() {
        Ok(())
    } else {
        Err(CoreError::BatchValidation(errors))
    }
}

/// Generate a simple update wrapper around `update_entity_generic`.
///
/// The macro defines a command struct, result type alias and async function
/// that forwards to `update_entity_generic`.
#[macro_export]
macro_rules! generate_update_wrapper {
    ($command:ident, $func:ident, $result:ident) => {
        #[derive(Debug, Clone)]
        pub struct $command {
            pub name: String,
            pub update: mm_memory::EntityUpdate,
        }

        pub type $result<E> = $crate::error::CoreResult<(), E>;

        #[tracing::instrument(skip(ports), fields(name = %command.name))]
        pub async fn $func<M, G>(
            ports: &$crate::ports::Ports<M, G>,
            command: $command,
        ) -> $result<M::Error>
        where
            M: mm_memory::MemoryRepository + Send + Sync,
            G: mm_git::GitRepository + Send + Sync,
            M::Error: std::error::Error + Send + Sync + 'static,
            G::Error: std::error::Error + Send + Sync + 'static,
        {
            $crate::operations::memory::generic::update_entity_generic(
                ports,
                &command.name,
                &command.update,
            )
            .await
        }
    };
}

/// Generate a simple get wrapper around `get_entity_generic`.
#[macro_export]
macro_rules! generate_get_wrapper {
    ($command:ident, $func:ident, $result:ident, $props:ty) => {
        #[derive(Debug, Clone)]
        pub struct $command {
            pub name: String,
        }

        pub type $result<E> = $crate::error::CoreResult<Option<mm_memory::MemoryEntity<$props>>, E>;

        #[tracing::instrument(skip(ports), fields(name = %command.name))]
        pub async fn $func<M, G>(
            ports: &$crate::ports::Ports<M, G>,
            command: $command,
        ) -> $result<M::Error>
        where
            M: mm_memory::MemoryRepository + Send + Sync,
            G: mm_git::GitRepository + Send + Sync,
            M::Error: std::error::Error + Send + Sync + 'static,
            G::Error: std::error::Error + Send + Sync + 'static,
        {
            $crate::operations::memory::generic::get_entity_generic::<M, G, $props>(
                ports,
                &command.name,
            )
            .await
        }
    };
}

/// Generate a simple delete wrapper that forwards to `delete_entities`.
#[macro_export]
macro_rules! generate_delete_wrapper {
    ($command:ident, $func:ident, $result:ident) => {
        #[derive(Debug, Clone)]
        pub struct $command {
            pub name: String,
        }

        pub type $result<E> = $crate::error::CoreResult<(), E>;

        #[tracing::instrument(skip(ports), fields(name = %command.name))]
        pub async fn $func<M, G>(
            ports: &$crate::ports::Ports<M, G>,
            command: $command,
        ) -> $result<M::Error>
        where
            M: mm_memory::MemoryRepository + Send + Sync,
            G: mm_git::GitRepository + Send + Sync,
            M::Error: std::error::Error + Send + Sync + 'static,
            G::Error: std::error::Error + Send + Sync + 'static,
        {
            validate_name!(command.name);
            $crate::operations::memory::delete_entities(
                ports,
                $crate::operations::memory::DeleteEntitiesCommand {
                    names: vec![command.name],
                },
            )
            .await
        }
    };
}
