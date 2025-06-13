use std::error::Error as StdError;
use thiserror::Error;

use crate::domain::validation_error::ValidationError;

/// Errors that can occur when interacting with the memory store
///
/// This is a generic error type that can wrap adapter-specific errors.
/// It provides different error variants for different types of errors:
///
/// - `ConnectionError`: Errors related to connecting to the memory store
/// - `QueryError`: Errors related to executing queries
/// - `RuntimeError`: Runtime errors that can wrap any error type
/// - `SerializationError`: Errors related to serializing or deserializing data
/// - `ValidationError`: Domain validation errors
///
/// The generic parameter `E` allows for adapter-specific error types.
#[derive(Error, Debug)]
pub enum MemoryError<E>
where
    E: StdError + Send + Sync + 'static,
{
    /// Error connecting to the memory store
    #[error("Database connection error: {message}")]
    ConnectionError {
        /// Error message
        message: String,
        /// Original error that caused the connection error
        #[source]
        source: Option<E>,
    },

    /// Error executing a query
    #[error("Query execution error: {message}")]
    QueryError {
        /// Error message
        message: String,
        /// Original error that caused the query error
        #[source]
        source: Option<E>,
    },

    /// Runtime error that can wrap any error type
    #[error("Runtime error: {message}")]
    RuntimeError {
        /// Error message
        message: String,
        /// Original error that caused the runtime error
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },

    /// Error serializing or deserializing data
    #[error("Serialization error")]
    SerializationError(#[from] serde_json::Error),

    /// Domain validation error
    #[error("Validation error")]
    ValidationError(#[from] ValidationError),
}

impl<E> MemoryError<E>
where
    E: StdError + Send + Sync + 'static,
{
    /// Create a new connection error without a source
    ///
    /// # Arguments
    ///
    /// * `message` - Error message
    ///
    /// # Returns
    ///
    /// A new `MemoryError::ConnectionError` without a source
    pub fn connection_error<S: Into<String>>(message: S) -> Self {
        Self::ConnectionError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new connection error with a source
    ///
    /// # Arguments
    ///
    /// * `message` - Error message
    /// * `source` - Original error that caused the connection error
    ///
    /// # Returns
    ///
    /// A new `MemoryError::ConnectionError` with the given source
    pub fn connection_error_with_source<S: Into<String>>(message: S, source: E) -> Self {
        Self::ConnectionError {
            message: message.into(),
            source: Some(source),
        }
    }

    /// Create a new query error without a source
    ///
    /// # Arguments
    ///
    /// * `message` - Error message
    ///
    /// # Returns
    ///
    /// A new `MemoryError::QueryError` without a source
    pub fn query_error<S: Into<String>>(message: S) -> Self {
        Self::QueryError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new query error with a source
    ///
    /// # Arguments
    ///
    /// * `message` - Error message
    /// * `source` - Original error that caused the query error
    ///
    /// # Returns
    ///
    /// A new `MemoryError::QueryError` with the given source
    pub fn query_error_with_source<S: Into<String>>(message: S, source: E) -> Self {
        Self::QueryError {
            message: message.into(),
            source: Some(source),
        }
    }

    /// Create a new runtime error without a source
    ///
    /// # Arguments
    ///
    /// * `message` - Error message
    ///
    /// # Returns
    ///
    /// A new `MemoryError::RuntimeError` without a source
    pub fn runtime_error<S: Into<String>>(message: S) -> Self {
        Self::RuntimeError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new runtime error with a source
    ///
    /// This method can wrap any error type that implements `StdError + Send + Sync + 'static`.
    ///
    /// # Arguments
    ///
    /// * `message` - Error message
    /// * `source` - Original error that caused the runtime error
    ///
    /// # Returns
    ///
    /// A new `MemoryError::RuntimeError` with the given source
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
///
/// This is a type alias for `Result<T, MemoryError<E>>`.
pub type MemoryResult<T, E> = Result<T, MemoryError<E>>;
