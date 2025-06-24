#![warn(clippy::all)]

mod repository;

pub use repository::Git2Repository;

use mm_git::GitService;

/// Create a new GitService with a Git2Repository
pub fn create_git_service() -> GitService<Git2Repository> {
    GitService::new(Git2Repository::new())
}
