#![warn(clippy::all)]
pub mod config;
pub mod entity;
pub mod error;
pub mod label_match_mode;
pub mod project_context;
pub mod relationship;
pub mod relationship_direction;
pub mod repository;
pub mod service;
pub mod validation_error;

pub use config::{DEFAULT_LABELS, DEFAULT_RELATIONSHIPS};
pub use config::{DEFAULT_MEMORY_LABEL, MemoryConfig};
pub use entity::MemoryEntity;
pub use error::{MemoryError, MemoryResult};
pub use label_match_mode::LabelMatchMode;
pub use project_context::ProjectContext;
pub use relationship::MemoryRelationship;
pub use relationship_direction::RelationshipDirection;
pub use repository::MemoryRepository;
#[cfg(any(test, feature = "mock"))]
pub use repository::MockMemoryRepository;
pub use service::MemoryService;
pub use validation_error::{ValidationError, ValidationErrorKind};

#[cfg(test)]
pub mod test_helpers;

#[cfg(any(test, feature = "test-suite"))]
pub mod test_suite;
