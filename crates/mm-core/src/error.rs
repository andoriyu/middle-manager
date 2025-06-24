use std::error::Error as StdError;
use thiserror::Error;

/// Error type for mm-core
#[derive(Error, Debug)]
pub enum CoreError<E>
where
    E: StdError + Send + Sync + 'static,
{
    #[error("Memory error")]
    Memory(#[from] mm_memory::MemoryError<E>),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Validation error")]
    Validation(#[from] mm_memory::ValidationError),

    #[error("Git error")]
    Git(#[from] mm_git::GitError),

    #[error("Git service not configured")]
    GitNotConfigured,

    /// Multiple validation errors grouped by name
    #[error("Batch validation error")]
    BatchValidation(Vec<(String, mm_memory::ValidationError)>),
}

/// Result type for mm-core
pub type CoreResult<T, E> = std::result::Result<T, CoreError<E>>;

impl<E> CoreError<E>
where
    E: StdError + Send + Sync + 'static,
{
    pub fn git_not_configured() -> Self {
        CoreError::GitNotConfigured
    }
}
