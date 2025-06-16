use mm_core::CoreError;
use mm_core::{
    AddObservationsError, CreateEntityError, GetEntityError, RemoveAllObservationsError,
    RemoveObservationsError, SetObservationsError,
};
use std::error::Error as StdError;
use std::fmt;

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
        Self::with_source(format!("{:#?}", error), error)
    }
}

impl<E> From<CreateEntityError<E>> for ToolError
where
    E: StdError + Send + Sync + 'static,
{
    fn from(error: CreateEntityError<E>) -> Self {
        Self::with_source(format!("Create entity error: {:#?}", error), error)
    }
}

impl<E> From<GetEntityError<E>> for ToolError
where
    E: StdError + Send + Sync + 'static,
{
    fn from(error: GetEntityError<E>) -> Self {
        Self::with_source(format!("Get entity error: {:#?}", error), error)
    }
}

impl<E> From<SetObservationsError<E>> for ToolError
where
    E: StdError + Send + Sync + 'static,
{
    fn from(error: SetObservationsError<E>) -> Self {
        Self::with_source(format!("Set observations error: {:#?}", error), error)
    }
}

impl<E> From<AddObservationsError<E>> for ToolError
where
    E: StdError + Send + Sync + 'static,
{
    fn from(error: AddObservationsError<E>) -> Self {
        Self::with_source(format!("Add observations error: {:#?}", error), error)
    }
}

impl<E> From<RemoveAllObservationsError<E>> for ToolError
where
    E: StdError + Send + Sync + 'static,
{
    fn from(error: RemoveAllObservationsError<E>) -> Self {
        Self::with_source(
            format!("Remove all observations error: {:#?}", error),
            error,
        )
    }
}

impl<E> From<RemoveObservationsError<E>> for ToolError
where
    E: StdError + Send + Sync + 'static,
{
    fn from(error: RemoveObservationsError<E>) -> Self {
        Self::with_source(format!("Remove observations error: {:#?}", error), error)
    }
}

impl From<serde_json::Error> for ToolError {
    fn from(error: serde_json::Error) -> Self {
        Self::with_source(format!("Serialization error: {:#?}", error), error)
    }
}
