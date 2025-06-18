use rust_mcp_sdk::macros::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Unstructured};
#[cfg(feature = "arbitrary")]
use mm_utils::is_snake_case;

/// Memory relationship representing an edge between entities
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct MemoryRelationship {
    /// Name of the source entity
    pub from: String,
    /// Name of the target entity
    pub to: String,
    /// Relationship type in snake_case
    pub name: String,
    /// Additional key-value properties
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

#[cfg(feature = "arbitrary")]
impl<'a> Arbitrary<'a> for MemoryRelationship {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let str_len = |rng: std::ops::RangeInclusive<usize>,
                       u: &mut Unstructured<'a>|
         -> arbitrary::Result<String> {
            let len = u.int_in_range(rng)?;
            let mut s = String::new();
            for _ in 0..len {
                s.push((b'a' + u.int_in_range::<u8>(0..=25)?) as char);
            }
            Ok(s)
        };

        let from = str_len(1..=8, u)?;
        let to = str_len(1..=8, u)?;

        let valid_name = u.arbitrary::<bool>()?;
        let name = if valid_name {
            str_len(3..=10, u)?
        } else {
            let mut s = str_len(3..=10, u)?;
            if is_snake_case(&s) {
                // introduce an uppercase character to break snake_case
                let pos = u.int_in_range::<usize>(0..=s.len() - 1)?;
                let ch = (b'A' + u.int_in_range::<u8>(0..=25)?) as char;
                s.replace_range(pos..=pos, &ch.to_string());
            }
            s
        };

        let prop_count = u.int_in_range::<usize>(0..=2)?;
        let mut properties = HashMap::new();
        for _ in 0..prop_count {
            let key = str_len(1..=5, u)?;
            let val = str_len(1..=5, u)?;
            properties.insert(key, val);
        }

        Ok(Self {
            from,
            to,
            name,
            properties,
        })
    }
}
