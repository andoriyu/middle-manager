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

    #[error("Git error")]
    Git(#[from] mm_git::GitError<E>),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Validation error")]
    Validation(#[from] mm_memory::ValidationError),

    /// Multiple validation errors grouped by name
    #[error("Batch validation error")]
    BatchValidation(Vec<(String, mm_memory::ValidationError)>),

    /// Error when a project name is required but not provided
    #[error("No project specified")]
    MissingProject,
}

/// Result type for mm-core
pub type CoreResult<T, E> = std::result::Result<T, CoreError<E>>;
