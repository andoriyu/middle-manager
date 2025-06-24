use std::error::Error as StdError;
use thiserror::Error;

/// Errors that can occur when interacting with Git repositories
#[derive(Error, Debug)]
pub enum GitError<E>
where
    E: StdError + Send + Sync + 'static,
{
    /// Error accessing the Git repository
    #[error("Repository error: {message}")]
    RepositoryError {
        message: String,
        #[source]
        source: Option<E>,
    },
}

impl<E> GitError<E>
where
    E: StdError + Send + Sync + 'static,
{
    pub fn repository_error<S: Into<String>>(message: S) -> Self {
        Self::RepositoryError {
            message: message.into(),
            source: None,
        }
    }

    pub fn repository_error_with_source<S: Into<String>>(message: S, source: E) -> Self {
        Self::RepositoryError {
            message: message.into(),
            source: Some(source),
        }
    }
}

pub type GitResult<T, E> = Result<T, GitError<E>>;
