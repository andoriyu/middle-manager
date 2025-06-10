use std::error::Error as StdError;
use thiserror::Error;

use crate::domain::validation_error::ValidationError;

/// Errors that can occur when interacting with the memory store
#[derive(Error, Debug)]
pub enum MemoryError<E> 
where 
    E: StdError + Send + Sync + 'static
{
    #[error("Database connection error: {message}")]
    ConnectionError {
        message: String,
        #[source]
        source: Option<E>,
    },
    
    #[error("Query execution error: {message}")]
    QueryError {
        message: String,
        #[source]
        source: Option<E>,
    },
    
    #[error("Runtime error: {message}")]
    RuntimeError {
        message: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },
    
    #[error("Serialization error")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Validation error")]
    ValidationError(#[from] ValidationError),
}

impl<E> MemoryError<E> 
where 
    E: StdError + Send + Sync + 'static
{
    pub fn connection_error<S: Into<String>>(message: S) -> Self {
        Self::ConnectionError {
            message: message.into(),
            source: None,
        }
    }
    
    pub fn connection_error_with_source<S: Into<String>>(message: S, source: E) -> Self {
        Self::ConnectionError {
            message: message.into(),
            source: Some(source),
        }
    }
    
    pub fn query_error<S: Into<String>>(message: S) -> Self {
        Self::QueryError {
            message: message.into(),
            source: None,
        }
    }
    
    pub fn query_error_with_source<S: Into<String>>(message: S, source: E) -> Self {
        Self::QueryError {
            message: message.into(),
            source: Some(source),
        }
    }
    
    pub fn runtime_error<S: Into<String>>(message: S) -> Self {
        Self::RuntimeError {
            message: message.into(),
            source: None,
        }
    }
    
    pub fn runtime_error_with_source<S, T>(message: S, source: T) -> Self
    where
        S: Into<String>,
        T: StdError + Send + Sync + 'static,
    {
        Self::RuntimeError {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

/// Result type for memory operations
pub type MemoryResult<T, E> = Result<T, MemoryError<E>>;
