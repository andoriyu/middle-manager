use anyhow::{Context, Error};
use mm_core::CoreError;
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use std::error::Error as StdError;
use tracing::error;

/// Create an [`anyhow::Error`] with a custom message and source error.
pub fn error_with_source<S, E>(message: S, source: E) -> Error
where
    S: Into<String>,
    E: StdError + Send + Sync + 'static,
{
    Err::<(), E>(source).context(message.into()).unwrap_err()
}

/// Convert a [`CoreError`] into an [`anyhow::Error`] with a helpful message.
pub fn core_error_to_anyhow<E>(error: CoreError<E>) -> Error
where
    E: StdError + Send + Sync + 'static,
{
    let message = match &error {
        CoreError::Memory(e) => e.to_string(),
        CoreError::Git(e) => e.to_string(),
        CoreError::Serialization(e) => e.to_string(),
        CoreError::Validation(e) => e.to_string(),
        CoreError::BatchValidation(v) => v
            .iter()
            .map(|(name, err)| format!("{}: {}", name, err))
            .collect::<Vec<_>>()
            .join("; "),
        CoreError::MissingProject => "No project specified".to_string(),
    };

    error_with_source(message, error)
}

/// Convert any error into a [`CallToolError`], logging it in the process.
pub fn into_call_tool_error<E>(err: E) -> CallToolError
where
    E: StdError + Send + Sync + 'static,
{
    error!("Tool call failed: {:#?}", err);
    let err = Error::from(err).into_boxed_dyn_error();
    CallToolError(err)
}

/// Map a result into a [`CallToolError`] using [`into_call_tool_error`].
pub fn map_result<R, E>(res: Result<R, E>) -> Result<R, CallToolError>
where
    E: StdError + Send + Sync + 'static,
{
    res.map_err(into_call_tool_error)
}
