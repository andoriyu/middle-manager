//! Utility helpers that are independent from the rest of the project.

/// Check if a string is in snake_case format.
///
/// This function verifies that all characters are lowercase, digits, or
/// underscores. It can be used when validating identifiers or any string that
/// must follow the snake_case convention.
///
/// # Examples
///
/// ```
/// use mm_utils::is_snake_case;
///
/// assert!(is_snake_case("hello_world"));
/// assert!(is_snake_case("hello"));
/// assert!(!is_snake_case("HelloWorld"));
/// assert!(!is_snake_case("Hello_World"));
/// ```
pub fn is_snake_case(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_lowercase() || c == '_' || c.is_numeric())
}

#[cfg(test)]
mod tests {
    use super::is_snake_case;

    #[test]
    fn valid_snake_case_strings() {
        assert!(is_snake_case("hello_world"));
        assert!(is_snake_case("hello"));
        assert!(is_snake_case("hello_world123"));
    }

    #[test]
    fn invalid_snake_case_strings() {
        assert!(!is_snake_case("HelloWorld"));
        assert!(!is_snake_case("Hello_World"));
        assert!(!is_snake_case("hello-world"));
    }
}
