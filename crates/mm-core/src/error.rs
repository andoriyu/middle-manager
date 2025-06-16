use std::error::Error as StdError;
use thiserror::Error;

/// Error type for mm-core
#[derive(Error, Debug)]
pub enum CoreError<E>
where
    E: StdError + Send + Sync + 'static,
{
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Memory error")]
    Memory(#[from] mm_memory::MemoryError<E>),

    #[error("Serialization error")]
    Serialization(#[from] serde_json::Error),

    #[error("Validation error")]
    Validation(#[from] mm_memory::ValidationError),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("MCP error: {0}")]
    Mcp(String),
}

/// Result type for mm-core
pub type CoreResult<T, E> = std::result::Result<T, CoreError<E>>;
