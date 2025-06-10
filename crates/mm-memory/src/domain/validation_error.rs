use thiserror::Error;

/// Validation errors for memory operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ValidationError {
    #[error("Entity name cannot be empty")]
    EmptyEntityName,
    
    #[error("Entity '{0}' must have at least one label")]
    NoLabels(String),
    
    #[error("Relationship type '{0}' is not in snake_case format")]
    InvalidRelationshipFormat(String),
}
