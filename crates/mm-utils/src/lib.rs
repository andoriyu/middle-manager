//! Utility helpers that are independent from the rest of the project.

#![warn(clippy::all)]
pub mod json_schema;
pub mod prop;

pub use json_schema::IntoJsonSchema;

/// Check if a string is in snake_case format.
///
/// This function verifies that all characters are ASCII lowercase letters,
/// digits, or underscores. It can be used when validating identifiers or any
/// string that must follow the snake_case convention.
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
    s.chars().all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_'))
}

#[cfg(test)]
mod tests {
    use super::is_snake_case;
    use arbitrary::{Arbitrary, Unstructured};
    use arbtest::arbtest;
    use std::ops::ControlFlow;

    #[derive(Debug)]
    struct SnakeCaseString(String);

    impl<'a> Arbitrary<'a> for SnakeCaseString {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let len = u.arbitrary_len::<u8>()?;
            let mut s = String::with_capacity(len);
            for _ in 0..len {
                let choice = u.int_in_range::<u8>(0..=36)?;
                let c = match choice {
                    0..=25 => (b'a' + choice) as char,
                    26 => '_',
                    _ => (b'0' + (choice - 27)) as char,
                };
                s.push(c);
            }
            Ok(SnakeCaseString(s))
        }
    }

    #[derive(Debug)]
    struct NonSnakeCaseString(String);

    impl<'a> Arbitrary<'a> for NonSnakeCaseString {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let len = u.arbitrary_len::<u8>()?.max(20);
            let invalid_pos = u.choose_index(len)?;
            let mut s = String::with_capacity(len);
            u.arbitrary_loop(Some(len as u32), Some(len as u32), |u| {
                let c = if s.len() == invalid_pos {
                    generate_invalid_char(u)?
                } else {
                    generate_valid_char(u)?
                };
                s.push(c);
                Ok(ControlFlow::Continue(()))
            })?;
            Ok(NonSnakeCaseString(s))
        }
    }

    fn generate_invalid_char(u: &mut Unstructured<'_>) -> arbitrary::Result<char> {
        let choice = u.int_in_range::<u8>(0..=26)?;
        Ok(match choice {
            0..=25 => (b'A' + choice) as char,
            _ => '-',
        })
    }

    fn generate_valid_char(u: &mut Unstructured<'_>) -> arbitrary::Result<char> {
        let choice = u.int_in_range::<u8>(0..=36)?;
        Ok(match choice {
            0..=25 => (b'a' + choice) as char,
            26 => '_',
            _ => (b'0' + (choice - 27)) as char,
        })
    }

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
        assert!(!is_snake_case("héllo"));
        assert!(!is_snake_case("hello１23"));
    }

    #[test]
    fn arbitrary_valid_strings() {
        arbtest(|u| {
            if let Ok(SnakeCaseString(s)) = SnakeCaseString::arbitrary(u) {
                assert!(is_snake_case(&s), "{}", s);
            }
            Ok(())
        });
    }

    #[test]
    fn arbitrary_invalid_strings() {
        arbtest(|u| {
            if let Ok(NonSnakeCaseString(s)) = NonSnakeCaseString::arbitrary(u) {
                assert!(!is_snake_case(&s), "{}", s);
            }
            Ok(())
        });
    }
}
