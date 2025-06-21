use crate::{
    MemoryConfig, MemoryEntity, MemoryRelationship, MemoryRepository, MemoryService, MemoryValue,
};
use chrono::Utc;
use std::collections::{HashMap, HashSet};

/// Run a comprehensive test suite against a `MemoryRepository` implementation.
///
/// This function creates a `MemoryService` with a fixed configuration and then
/// exercises all service methods to verify correct behaviour. It can be reused
/// with any repository implementation.
pub async fn run_memory_service_test_suite<R>(
    repository: R,
) -> Result<(), Box<dyn std::error::Error>>
where
    R: MemoryRepository + Send + Sync + 'static,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    // Fixed configuration used for all tests
    let config = MemoryConfig {
        default_label: Some("TestSuite".to_string()),
        default_relationships: true,
        additional_relationships: HashSet::default(),
        default_labels: true,
        additional_labels: std::iter::once("Example".to_string()).collect(),
    };

    let service = MemoryService::new(repository, config);

    // Generate unique names so repeated runs don't conflict
    let unique = Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let name_a = format!("test:suite:a:{unique}");
    let name_b = format!("test:suite:b:{unique}");

    // --- Entity creation (batch) ---
    let mut props = HashMap::new();
    props.insert("k".to_string(), MemoryValue::String("v".to_string()));
    let entity_a = MemoryEntity {
        name: name_a.clone(),
        labels: vec!["Example".to_string()],
        observations: vec!["first".to_string()],
        properties: props,
        ..Default::default()
    };
    let entity_b = MemoryEntity {
        name: name_b.clone(),
        labels: vec!["Example".to_string()],
        ..Default::default()
    };

    let errs = service
        .create_entities(&[entity_a.clone(), entity_b.clone()])
        .await?;
    assert!(errs.is_empty());

    // Verify retrieval of entity and default label
    let fetched_a = service
        .find_entity_by_name(&name_a)
        .await?
        .expect("entity a should exist");
    assert!(fetched_a.labels.contains(&"TestSuite".to_string()));
    assert!(fetched_a.labels.contains(&"Example".to_string()));
    assert_eq!(fetched_a.observations, ["first".to_string()]);
    assert_eq!(
        fetched_a.properties.get("k"),
        Some(&MemoryValue::String("v".to_string()))
    );

    // --- Relationship creation ---
    let rel = MemoryRelationship {
        from: name_a.clone(),
        to: name_b.clone(),
        name: "relates_to".to_string(),
        properties: HashMap::default(),
    };
    service.create_relationships(&[rel.clone()]).await?;

    let fetched_a = service.find_entity_by_name(&name_a).await?.unwrap();
    assert!(
        fetched_a
            .relationships
            .iter()
            .any(|r| r.from == name_a && r.to == name_b && r.name == "relates_to")
    );

    // --- Observation management ---
    service
        .set_observations(&name_a, &["one".to_string(), "two".to_string()])
        .await?;
    let after_set = service.find_entity_by_name(&name_a).await?.unwrap();
    assert_eq!(after_set.observations, ["one", "two"]);

    service
        .add_observations(&name_a, &["three".to_string()])
        .await?;
    let after_add = service.find_entity_by_name(&name_a).await?.unwrap();
    assert_eq!(after_add.observations, ["one", "two", "three"]);

    service
        .remove_observations(&name_a, &["two".to_string()])
        .await?;
    let after_remove = service.find_entity_by_name(&name_a).await?.unwrap();
    assert_eq!(after_remove.observations, ["one", "three"]);

    service.remove_all_observations(&name_a).await?;
    let cleared = service.find_entity_by_name(&name_a).await?.unwrap();
    assert!(cleared.observations.is_empty());

    // --- Validation and error handling ---
    let invalid = MemoryEntity::default();
    let errs = service.create_entities(&[invalid]).await?;
    assert!(!errs.is_empty());
    assert!(service.find_entity_by_name("").await.is_err());

    let bad_rel = MemoryRelationship {
        from: name_a.clone(),
        to: name_b.clone(),
        name: "BadRel".to_string(),
        properties: HashMap::default(),
    };
    let rel_errors = service.create_relationships(&[bad_rel]).await?;
    assert!(!rel_errors.is_empty());

    // --- Large batch create to exercise path ---
    let extra: Vec<_> = (0..10)
        .map(|i| MemoryEntity {
            name: format!("test:suite:extra:{unique}:{i}"),
            labels: vec!["Example".to_string()],
            ..Default::default()
        })
        .collect();
    service.create_entities(&extra).await?;

    // --- Find entities by labels ---
    use crate::label_match_mode::LabelMatchMode;
    let by_example = service
        .find_entities_by_labels(&["Example".to_string()], LabelMatchMode::Any, None)
        .await?;
    assert!(by_example.iter().any(|e| e.name == name_a));
    assert!(by_example.iter().any(|e| e.name == name_b));

    let by_all = service
        .find_entities_by_labels(
            &["Example".to_string(), "TestSuite".to_string()],
            LabelMatchMode::All,
            None,
        )
        .await?;
    assert!(by_all.iter().any(|e| e.name == name_a));
    assert!(by_all.iter().any(|e| e.name == name_b));

    let required_only = service
        .find_entities_by_labels(&[], LabelMatchMode::Any, Some("TestSuite".to_string()))
        .await?;
    assert!(required_only.iter().any(|e| e.name == name_a));
    assert!(required_only.iter().any(|e| e.name == name_b));

    Ok(())
}
