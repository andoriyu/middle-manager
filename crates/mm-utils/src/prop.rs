use arbitrary::{Arbitrary, Unstructured};
use arbtest::arbtest;

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
