use crate::{MemoryEntity, MemoryRelationship, MemoryValue};
use arbitrary::{Arbitrary, Unstructured};
use mm_utils::prop::{NonEmptyName, small_string, small_string_map, small_string_vec};

/// Generate a random `MemoryEntity` for property tests.
///
/// The entity will always have a non-empty name and at least one label.
/// A specific label can be supplied to ensure it matches allowed values.
///
/// # Examples
///
/// ```
/// use arbitrary::Unstructured;
/// use mm_memory::test_helpers::prop_random_entity;
///
/// let mut u = Unstructured::new(&[1, 2, 3, 4]);
/// let entity = prop_random_entity(&mut u, None).unwrap();
/// assert!(!entity.name.is_empty());
/// ```
pub fn prop_random_entity(
    u: &mut Unstructured<'_>,
    label: Option<String>,
) -> arbitrary::Result<MemoryEntity> {
    let NonEmptyName(name) = NonEmptyName::arbitrary(u)?;
    let mut labels = if let Some(l) = label {
        vec![l]
    } else {
        small_string_vec(u, 3)?
    };
    if labels.is_empty() {
        labels.push(small_string(u)?);
    }
    let observations = small_string_vec(u, 3)?;
    let properties = small_string_map(u, 3)?
        .into_iter()
        .map(|(k, v)| (k, MemoryValue::String(v)))
        .collect();
    Ok(MemoryEntity {
        name,
        labels,
        observations,
        properties,
    })
}

/// Generate a random `MemoryRelationship` for property tests.
///
/// By default the relationship name is random lowercase ASCII. A specific name
/// can be provided when testing validation of known relationship types.
///
/// # Examples
///
/// ```
/// use arbitrary::Unstructured;
/// use mm_memory::test_helpers::prop_random_relationship;
///
/// let mut u = Unstructured::new(&[5, 6, 7, 8]);
/// let rel = prop_random_relationship(&mut u, None).unwrap();
/// assert!(!rel.from.is_empty() && !rel.to.is_empty());
/// ```
pub fn prop_random_relationship(
    u: &mut Unstructured<'_>,
    name: Option<String>,
) -> arbitrary::Result<MemoryRelationship> {
    let NonEmptyName(from) = NonEmptyName::arbitrary(u)?;
    let NonEmptyName(to) = NonEmptyName::arbitrary(u)?;
    let rel_name = match name {
        Some(n) => n,
        None => small_string(u)?,
    };
    let properties = small_string_map(u, 3)?
        .into_iter()
        .map(|(k, v)| (k, MemoryValue::String(v)))
        .collect();
    Ok(MemoryRelationship {
        from,
        to,
        name: rel_name,
        properties,
    })
}
