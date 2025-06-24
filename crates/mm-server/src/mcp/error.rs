use mm_core::CoreError;

use rust_mcp_sdk::schema::schema_utils::CallToolError;
use std::error::Error as StdError;
use std::fmt;
use tracing::error;

/// Error type for MCP tools
#[derive(Debug)]
pub struct ToolError {
    message: String,
    source: Option<Box<dyn StdError + Send + Sync>>,
}

impl ToolError {
    /// Create a new tool error with a message and source
    pub fn with_source<S, E>(message: S, source: E) -> Self
    where
        S: Into<String>,
        E: StdError + Send + Sync + 'static,
    {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StdError for ToolError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|s| s.as_ref() as &(dyn StdError + 'static))
    }
}

impl<E> From<CoreError<E>> for ToolError
where
    E: StdError + Send + Sync + 'static,
{
    fn from(error: CoreError<E>) -> Self {
        let message = match &error {
            CoreError::Memory(e) => e.to_string(),
            CoreError::Serialization(e) => e.to_string(),
            CoreError::Validation(e) => e.to_string(),
            CoreError::BatchValidation(v) => v
                .iter()
                .map(|(name, err)| format!("{}: {}", name, err))
                .collect::<Vec<_>>()
                .join("; "),
            CoreError::Git(e) => e.to_string(),
            CoreError::GitNotConfigured => "Git service not configured".to_string(),
        };
        Self::with_source(message, error)
    }
}

impl From<serde_json::Error> for ToolError {
    fn from(error: serde_json::Error) -> Self {
        Self::with_source(format!("Serialization error: {:#?}", error), error)
    }
}

/// Map a core result into a MCP tool result by converting any error into a
/// [`CallToolError`].
///
/// The caller is expected to map the successful result into a
/// [`CallToolResult`] before invoking this helper.
pub fn map_result<R, E>(res: Result<R, E>) -> Result<R, CallToolError>
where
    E: Into<ToolError> + StdError + Send + Sync + 'static,
{
    res.map_err(|e| {
        error!("Tool call failed: {:#?}", e);
        let tool_error: ToolError = e.into();
        CallToolError::new(tool_error)
    })
}
