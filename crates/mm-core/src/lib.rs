//! Core domain logic for the Middle Manager project.
//!
//! This crate represents the application "core" in the hexagonal
//! architecture. It defines operations on the memory graph and the
//! ports (interfaces) that adapters must implement. The code here is
//! completely independent of external protocols or infrastructure and
//! focuses purely on business rules.
#![warn(clippy::all)]
pub mod error;
pub mod operations;
mod ports;
mod root;

pub use error::{CoreError, CoreResult};
pub use ports::Ports;
pub use root::{Root, RootCollection};

pub(crate) use operations::memory::validate_name;

// Re-export the mm-memory crate for easy access to memory types and services
pub use mm_memory;

// Re-export the mm-git crate for easy access to git types and services
pub use mm_git;
