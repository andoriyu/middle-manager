//! Utility helpers that are independent from the rest of the project.

#![warn(clippy::all)]
pub mod prop;

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
    use arbitrary::{Arbitrary, Unstructured};
    use arbtest::arbtest;

    #[derive(Debug)]
    struct SnakeCaseString(String);

    impl<'a> Arbitrary<'a> for SnakeCaseString {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let len = u.int_in_range::<usize>(0..=20)?;
            let mut s = String::new();
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
            let len = u.int_in_range::<usize>(1..=20)?;
            let invalid_pos = u.int_in_range::<usize>(0..len)?;
            let mut s = String::new();
            for i in 0..len {
                let c = if i == invalid_pos {
                    let choice = u.int_in_range::<u8>(0..=26)?;
                    match choice {
                        0..=25 => (b'A' + choice) as char,
                        _ => '-',
                    }
                } else {
                    let choice = u.int_in_range::<u8>(0..=36)?;
                    match choice {
                        0..=25 => (b'a' + choice) as char,
                        26 => '_',
                        _ => (b'0' + (choice - 27)) as char,
                    }
                };
                s.push(c);
            }
            Ok(NonSnakeCaseString(s))
        }
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
