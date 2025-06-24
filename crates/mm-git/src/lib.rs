#![warn(clippy::all)]

pub mod error;
pub mod repository;
pub mod service;
pub mod status;

pub use error::{GitError, GitResult};
pub use repository::GitRepository;
pub use service::GitService;
pub use status::GitStatus;
