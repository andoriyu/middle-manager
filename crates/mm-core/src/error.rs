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
    Memory(#[from] mm_memory_neo4j::MemoryError<E>),

    #[error("Serialization error")]
    Serialization(#[from] serde_json::Error),

    #[error("Validation error")]
    Validation(#[from] mm_memory_neo4j::ValidationError),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("MCP error: {0}")]
    Mcp(String),
}

/// Result type for mm-core
pub type CoreResult<T, E> = std::result::Result<T, CoreError<E>>;

// Type aliases for common error types
// Use the Error type that mm-memory-neo4j re-exports
pub type Error = CoreError<mm_memory_neo4j::Error>;
pub type Result<T> = CoreResult<T, mm_memory_neo4j::Error>;
