#[macro_export]
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
