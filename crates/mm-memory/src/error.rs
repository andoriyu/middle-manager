use std::error::Error as StdError;
use thiserror::Error;

use crate::validation_error::ValidationError;

/// Errors that can occur when interacting with the memory store
#[derive(Error, Debug)]
pub enum MemoryError<E>
where
    E: StdError + Send + Sync + 'static,
{
    /// Error connecting to the memory store
    #[error("Database connection error: {message}")]
    ConnectionError {
        message: String,
        #[source]
        source: Option<E>,
    },

    /// Error executing a query
    #[error("Query execution error: {message}")]
    QueryError {
        message: String,
        #[source]
        source: Option<E>,
    },

    /// Runtime error that can wrap any error type
    #[error("Runtime error: {message}")]
    RuntimeError {
        message: String,
        #[source]
        source: Option<Box<dyn StdError + Send + Sync>>,
    },

    /// Error serializing or deserializing data
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Domain validation error
    #[error("Validation error")]
    ValidationError(#[from] ValidationError),

    /// Error when an entity is not found
    #[error("Entity not found: {0}")]
    EntityNotFound(String),
}

impl<E> MemoryError<E>
where
    E: StdError + Send + Sync + 'static,
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

    pub fn entity_not_found<S: Into<String>>(entity_name: S) -> Self {
        Self::EntityNotFound(entity_name.into())
    }
}

pub type MemoryResult<T, E> = Result<T, MemoryError<E>>;
