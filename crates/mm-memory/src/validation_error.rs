use thiserror::Error;

/// Individual validation error types
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ValidationErrorKind {
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

    /// Error when a label is not allowed
    #[error("Label '{0}' is not allowed")]
    UnknownLabel(String),

    /// Error when traversal depth is invalid
    #[error("Traversal depth '{0}' is out of range (1-5)")]
    InvalidDepth(u32),

    /// Error when multiple operations are specified for the same field
    #[error("Conflicting operations for {0}")]
    ConflictingOperations(&'static str),
}

/// Collection of validation errors
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError(pub Vec<ValidationErrorKind>);

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msgs: Vec<String> = self.0.iter().map(|e| e.to_string()).collect();
        write!(f, "{}", msgs.join("; "))
    }
}

impl std::error::Error for ValidationError {}

impl From<ValidationErrorKind> for ValidationError {
    fn from(kind: ValidationErrorKind) -> Self {
        Self(vec![kind])
    }
}
