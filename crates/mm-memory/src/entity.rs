use rust_mcp_sdk::macros::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "arbitrary")]
use crate::DEFAULT_LABELS;
#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Unstructured};

/// Memory entity representing a node in the knowledge graph
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct MemoryEntity {
    /// Unique name of the entity
    pub name: String,
    /// Labels for categorizing the entity
    pub labels: Vec<String>,
    /// Facts or notes about the entity
    pub observations: Vec<String>,
    /// Additional key-value properties
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

#[cfg(feature = "arbitrary")]
impl<'a> Arbitrary<'a> for MemoryEntity {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let name_len = u.int_in_range::<usize>(1..=10)?;
        let mut name = String::new();
        for _ in 0..name_len {
            let ch = (b'a' + u.int_in_range::<u8>(0..=25)?) as char;
            name.push(ch);
        }

        let has_labels = u.arbitrary::<bool>()?;
        let label_count = if has_labels {
            u.int_in_range::<usize>(1..=3)?
        } else {
            0
        };
        let mut labels = Vec::new();
        for _ in 0..label_count {
            let idx = u.int_in_range::<usize>(0..=DEFAULT_LABELS.len() - 1)?;
            labels.push(DEFAULT_LABELS[idx].to_string());
        }

        let obs_count = u.int_in_range::<usize>(0..=3)?;
        let mut observations = Vec::new();
        for _ in 0..obs_count {
            let len = u.int_in_range::<usize>(1..=10)?;
            let mut s = String::new();
            for _ in 0..len {
                let ch = (b'a' + u.int_in_range::<u8>(0..=25)?) as char;
                s.push(ch);
            }
            observations.push(s);
        }

        let prop_count = u.int_in_range::<usize>(0..=2)?;
        let mut properties = HashMap::new();
        for _ in 0..prop_count {
            let key_len = u.int_in_range::<usize>(1..=5)?;
            let mut key = String::new();
            for _ in 0..key_len {
                key.push((b'a' + u.int_in_range::<u8>(0..=25)?) as char);
            }
            let val_len = u.int_in_range::<usize>(1..=5)?;
            let mut val = String::new();
            for _ in 0..val_len {
                val.push((b'a' + u.int_in_range::<u8>(0..=25)?) as char);
            }
            properties.insert(key, val);
        }

        Ok(Self {
            name,
            labels,
            observations,
            properties,
        })
    }
}
