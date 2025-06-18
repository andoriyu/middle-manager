#[cfg(test)]
pub mod prop {
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
            let rt = tokio::runtime::Runtime::new().unwrap();
            f(&rt, u)
        });
    }
}
