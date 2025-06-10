use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Memory entity representing a node in the knowledge graph
///
/// A memory entity is a node in the knowledge graph that represents a piece of knowledge.
/// It has a name, labels, observations, and properties.
///
/// # Name Format
///
/// Entity names should follow a colon-separated format to indicate their type and context:
/// - `project:entity:name`
/// - `tech:language:rust`
/// - `agent:task:update_packages`
///
/// # Labels
///
/// Labels are used to categorize entities and should follow CamelCase format:
/// - `Memory`
/// - `Project`
/// - `Technology`
/// - `Task`
///
/// # Observations
///
/// Observations are facts or notes about the entity, stored as a list of strings.
///
/// # Properties
///
/// Properties are additional key-value pairs that provide more structured information about the entity.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MemoryEntity {
    /// Unique name of the entity, following a colon-separated format
    pub name: String,
    
    /// Labels for categorizing the entity, following CamelCase format
    pub labels: Vec<String>,
    
    /// Facts or notes about the entity
    pub observations: Vec<String>,
    
    /// Additional key-value properties
    pub properties: HashMap<String, String>,
}

/// Check if a string is in snake_case format
///
/// This function checks if a string follows the snake_case naming convention,
/// which means all lowercase letters with underscores separating words.
///
/// # Arguments
///
/// * `s` - The string to check
///
/// # Returns
///
/// `true` if the string is in snake_case format, `false` otherwise
///
/// # Examples
///
/// ```
/// use mm_memory::domain::entity::is_snake_case;
///
/// assert!(is_snake_case("hello_world"));
/// assert!(is_snake_case("hello"));
/// assert!(!is_snake_case("HelloWorld"));
/// assert!(!is_snake_case("Hello_World"));
/// ```
pub fn is_snake_case(s: &str) -> bool {
    s.chars().all(|c| c.is_lowercase() || c == '_' || c.is_numeric())
}
