#[cfg(test)]
pub mod prop {
    use arbitrary::{Arbitrary, Unstructured};
    use arbtest::arbtest;

    #[derive(Debug)]
    pub struct NonEmptyName(pub String);

    impl<'a> Arbitrary<'a> for NonEmptyName {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let len = u.int_in_range::<usize>(1..=20)?;
            let mut s = String::new();
            for _ in 0..len {
                let choice = u.int_in_range::<u8>(0..=25)?;
                s.push((b'a' + choice) as char);
            }
            Ok(Self(s))
        }
    }

    #[derive(Debug)]
    pub struct NonEmptySnakeCase(pub String);

    impl<'a> Arbitrary<'a> for NonEmptySnakeCase {
        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let len = u.int_in_range::<usize>(1..=20)?;
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
            Ok(Self(s))
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
