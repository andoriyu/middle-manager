use async_trait::async_trait;
use std::error::Error as StdError;
use std::path::Path;

use crate::error::GitResult;
use crate::status::GitStatus;

#[cfg_attr(any(test, feature = "mock"), mockall::automock(type Error = std::convert::Infallible;))]
#[async_trait]
pub trait GitRepository {
    type Error: StdError + Send + Sync + 'static;

    /// Get the status of a Git repository.
    ///
    /// The returned [`GitStatus`] includes the current branch name, upstream
    /// branch information, ahead/behind counts, and detailed lists of files in
    /// different states (staged, modified, untracked, conflicted). It also
    /// reports whether the working tree is dirty and how many stashes exist.
    async fn get_status(&self, path: &Path) -> GitResult<GitStatus, Self::Error>;
}
