//! Additional property helpers for generating memory data live in
//! `mm_memory::test_helpers`.
//!
//! ```
//! use arbitrary::Unstructured;
//! use mm_memory::test_helpers::{prop_random_entity, prop_random_relationship};
//!
//! let mut u = Unstructured::new(&[1, 2, 3, 4]);
//! let _e = prop_random_entity(&mut u, None).unwrap();
//! let _r = prop_random_relationship(&mut u, None).unwrap();
//! ```

use arbitrary::{Arbitrary, Unstructured};
use arbtest::arbtest;
use std::collections::HashMap;

#[derive(Debug)]
pub struct NonEmptyName(pub String);

impl<'a> Arbitrary<'a> for NonEmptyName {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let s: String = Arbitrary::arbitrary(u)?;
        if s.is_empty() {
            Err(arbitrary::Error::IncorrectFormat)
        } else {
            Ok(Self(s))
        }
    }
}

pub fn async_arbtest<F>(mut f: F)
where
    F: for<'a> FnMut(&tokio::runtime::Runtime, &mut Unstructured<'a>) -> arbitrary::Result<()>,
{
    arbtest(|u| {
        let rt = tokio::runtime::Runtime::new().expect("Failed to initialize Tokio runtime");
        f(&rt, u)
    });
}

/// Generate a short lowercase ASCII string.
///
/// This helper limits the produced string to eight characters so tests remain
/// efficient even with arbitrarily large inputs.
pub fn small_string(u: &mut Unstructured<'_>) -> arbitrary::Result<String> {
    let len = usize::min(u.len(), 8);
    let mut s = String::with_capacity(len);
    for _ in 0..len {
        let b = u.arbitrary::<u8>()?;
        s.push((b % 26 + b'a') as char);
    }
    Ok(s)
}

/// Generate a vector of up to `max` short strings.
pub fn small_string_vec(u: &mut Unstructured<'_>, max: usize) -> arbitrary::Result<Vec<String>> {
    let count = u.int_in_range::<usize>(0..=max)?;
    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        items.push(small_string(u)?);
    }
    Ok(items)
}

/// Generate a map of up to `max` key/value pairs of short strings.
pub fn small_string_map(
    u: &mut Unstructured<'_>,
    max: usize,
) -> arbitrary::Result<HashMap<String, String>> {
    let count = u.int_in_range::<usize>(0..=max)?;
    let mut items = HashMap::with_capacity(count);
    for _ in 0..count {
        items.insert(small_string(u)?, small_string(u)?);
    }
    Ok(items)
}
