use thiserror::Error;

/// Validation errors for memory operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Error when an entity name is empty
    #[error("Entity name cannot be empty")]
    EmptyEntityName,

    /// Error when an entity has no labels
    #[error("Entity '{0}' must have at least one label")]
    NoLabels(String),

    /// Error when a relationship type is not in snake_case format
    #[error("Relationship type '{0}' is not in snake_case format")]
    InvalidRelationshipFormat(String),

    /// Error when a relationship type is not allowed
    #[error("Relationship type '{0}' is not allowed")]
    UnknownRelationship(String),
}
