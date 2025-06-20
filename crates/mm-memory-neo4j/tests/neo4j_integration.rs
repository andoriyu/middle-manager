use mm_memory::test_suite::run_memory_service_test_suite;
use mm_memory::{MemoryRelationship, MemoryValue, RelationshipDirection};
use mm_memory_neo4j::LabelMatchMode;
use mm_memory_neo4j::{
    MemoryConfig, MemoryEntity, MemoryError, Neo4jConfig, Neo4jRepository, create_neo4j_service,
};
use std::collections::HashMap;

#[tokio::test]
async fn test_connection_error() {
    // Use an invalid scheme so repository creation fails immediately
    let config = Neo4jConfig {
        uri: "invalid://localhost:7687".to_string(),
        username: "neo4j".to_string(),
        password: "wrong".to_string(),
    };

    let result = create_neo4j_service(config, MemoryConfig::default()).await;
    assert!(matches!(result, Err(MemoryError::ConnectionError { .. })));
}

#[tokio::test]
async fn test_find_nonexistent_entity() {
    let config = Neo4jConfig {
        uri: "neo4j://localhost:7688".to_string(),
        username: "neo4j".to_string(),
        password: "password".to_string(),
    };

    let service = create_neo4j_service(
        config,
        MemoryConfig {
            default_label: Some("TestFindNone".to_string()),
            default_relationships: true,
            additional_relationships: std::collections::HashSet::default(),
            default_labels: true,
            additional_labels: std::iter::once("Example".to_string()).collect(),
        },
    )
    .await
    .unwrap();

    // Test that finding a non-existent entity returns None
    let result = service
        .find_entity_by_name("non:existent:entity")
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_create_and_find_entity() {
    let config = Neo4jConfig {
        uri: "neo4j://localhost:7688".to_string(),
        username: "neo4j".to_string(),
        password: "password".to_string(),
    };

    let service = create_neo4j_service(
        config,
        MemoryConfig {
            default_label: Some("TestCreate".to_string()),
            default_relationships: true,
            additional_relationships: std::collections::HashSet::default(),
            default_labels: true,
            additional_labels: std::iter::once("Example".to_string()).collect(),
        },
    )
    .await
    .unwrap();

    let mut props = HashMap::new();
    props.insert("rating".to_string(), MemoryValue::Integer(5));
    let entity = MemoryEntity {
        name: "test:entity:create".to_string(),
        labels: vec!["Example".to_string()],
        observations: vec!["This is a test entity for creation".to_string()],
        properties: props,
        ..Default::default()
    };

    // Test that entity creation doesn't error
    service
        .create_entities(std::slice::from_ref(&entity))
        .await
        .unwrap();

    // Test that we can find the entity after creation
    let found = service
        .find_entity_by_name("test:entity:create")
        .await
        .unwrap();
    assert!(found.is_some());

    let found_entity = found.unwrap();
    assert_eq!(found_entity.name, entity.name);
    assert_eq!(found_entity.observations, entity.observations);
    assert_eq!(
        found_entity.properties.get("rating"),
        Some(&MemoryValue::Integer(5))
    );

    // Check that labels contain the expected values
    assert!(found_entity.labels.contains(&"Example".to_string()));
    assert!(found_entity.labels.contains(&"TestCreate".to_string()));
}

#[tokio::test]
async fn test_validation_errors() {
    let config = Neo4jConfig {
        uri: "neo4j://localhost:7688".to_string(),
        username: "neo4j".to_string(),
        password: "password".to_string(),
    };

    let service = create_neo4j_service(
        config,
        MemoryConfig {
            default_label: Some("TestValidation".to_string()),
            default_relationships: true,
            additional_relationships: std::collections::HashSet::default(),
            default_labels: true,
            additional_labels: std::iter::once("Example".to_string()).collect(),
        },
    )
    .await
    .unwrap();

    // Test empty entity name
    let result = service.find_entity_by_name("").await;
    assert!(result.is_err());

    // Test entity with no labels
    let entity = MemoryEntity {
        name: "test:entity:no_labels".to_string(),
        observations: vec!["This entity has no labels".to_string()],
        ..Default::default()
    };

    let result = service.create_entities(std::slice::from_ref(&entity)).await;
    assert!(result.is_ok());

    // Ensure the default tag was applied
    let found = service
        .find_entity_by_name("test:entity:no_labels")
        .await
        .unwrap()
        .unwrap();
    assert!(found.labels.contains(&"TestValidation".to_string()));
}

#[tokio::test]
async fn test_set_observations() {
    let config = Neo4jConfig {
        uri: "neo4j://localhost:7688".to_string(),
        username: "neo4j".to_string(),
        password: "password".to_string(),
    };

    let service = create_neo4j_service(
        config,
        MemoryConfig {
            default_label: Some("TestSet".to_string()),
            default_relationships: true,
            additional_relationships: std::collections::HashSet::default(),
            default_labels: true,
            additional_labels: std::iter::once("Example".to_string()).collect(),
        },
    )
    .await
    .unwrap();

    let entity_name = "test:entity:set";
    let entity = MemoryEntity {
        name: entity_name.to_string(),
        labels: vec!["Example".to_string()],
        observations: vec!["initial".to_string()],
        ..Default::default()
    };

    service
        .create_entities(std::slice::from_ref(&entity))
        .await
        .unwrap();

    service
        .set_observations(entity_name, &["replaced".to_string()])
        .await
        .unwrap();

    let updated = service
        .find_entity_by_name(entity_name)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.observations, vec!["replaced".to_string()]);
}

#[tokio::test]
async fn test_add_and_remove_observations() {
    let config = Neo4jConfig {
        uri: "neo4j://localhost:7688".to_string(),
        username: "neo4j".to_string(),
        password: "password".to_string(),
    };

    let service = create_neo4j_service(
        config,
        MemoryConfig {
            default_label: Some("TestAddRemove".to_string()),
            default_relationships: true,
            additional_relationships: std::collections::HashSet::default(),
            default_labels: true,
            additional_labels: std::iter::once("Example".to_string()).collect(),
        },
    )
    .await
    .unwrap();

    let entity_name = "test:entity:modify";
    let entity = MemoryEntity {
        name: entity_name.to_string(),
        labels: vec!["Example".to_string()],
        observations: vec!["obs1".to_string(), "obs2".to_string()],
        ..Default::default()
    };

    service
        .create_entities(std::slice::from_ref(&entity))
        .await
        .unwrap();

    service
        .add_observations(entity_name, &["obs3".to_string()])
        .await
        .unwrap();

    let after_add = service
        .find_entity_by_name(entity_name)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(after_add.observations, vec!["obs1", "obs2", "obs3"]);

    service
        .remove_observations(entity_name, &["obs2".to_string()])
        .await
        .unwrap();

    let after_remove = service
        .find_entity_by_name(entity_name)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(after_remove.observations, vec!["obs1", "obs3"]);

    service.remove_all_observations(entity_name).await.unwrap();

    let cleared = service
        .find_entity_by_name(entity_name)
        .await
        .unwrap()
        .unwrap();
    assert!(cleared.observations.is_empty());
}

#[tokio::test]
async fn test_create_relationship() {
    let config = Neo4jConfig {
        uri: "neo4j://localhost:7688".to_string(),
        username: "neo4j".to_string(),
        password: "password".to_string(),
    };

    let service = create_neo4j_service(
        config,
        MemoryConfig {
            default_label: Some("RelationshipTest".to_string()),
            default_relationships: true,
            additional_relationships: std::collections::HashSet::default(),
            default_labels: true,
            additional_labels: std::iter::once("Example".to_string()).collect(),
        },
    )
    .await
    .unwrap();

    let a = MemoryEntity {
        name: "rel:a".to_string(),
        labels: vec!["Example".to_string()],
        ..Default::default()
    };
    let b = MemoryEntity {
        name: "rel:b".to_string(),
        labels: vec!["Example".to_string()],
        ..Default::default()
    };

    service
        .create_entities(std::slice::from_ref(&a))
        .await
        .unwrap();
    service
        .create_entities(std::slice::from_ref(&b))
        .await
        .unwrap();

    let rel = MemoryRelationship {
        from: a.name.clone(),
        to: b.name.clone(),
        name: "relates_to".to_string(),
        properties: HashMap::default(),
    };

    service
        .create_relationships(std::slice::from_ref(&rel))
        .await
        .unwrap();

    let fetched_a = service.find_entity_by_name("rel:a").await.unwrap().unwrap();
    assert!(
        fetched_a
            .relationships
            .iter()
            .any(|r| r.from == "rel:a" && r.to == "rel:b" && r.name == "relates_to")
    );

    let fetched_b = service.find_entity_by_name("rel:b").await.unwrap().unwrap();
    assert!(
        fetched_b
            .relationships
            .iter()
            .any(|r| r.from == "rel:a" && r.to == "rel:b" && r.name == "relates_to")
    );
}

#[tokio::test]
async fn test_find_related_entities() {
    let config = Neo4jConfig {
        uri: "neo4j://localhost:7688".to_string(),
        username: "neo4j".to_string(),
        password: "password".to_string(),
    };

    let service = create_neo4j_service(
        config,
        MemoryConfig {
            default_label: Some("RelatedTest".to_string()),
            default_relationships: true,
            additional_relationships: std::collections::HashSet::default(),
            default_labels: true,
            additional_labels: std::iter::once("Example".to_string()).collect(),
        },
    )
    .await
    .unwrap();

    let a = MemoryEntity {
        name: "related:a".to_string(),
        labels: vec!["Example".to_string()],
        ..Default::default()
    };
    let b = MemoryEntity {
        name: "related:b".to_string(),
        labels: vec!["Example".to_string()],
        ..Default::default()
    };
    let c = MemoryEntity {
        name: "related:c".to_string(),
        labels: vec!["Example".to_string()],
        ..Default::default()
    };

    service
        .create_entities(&[a.clone(), b.clone(), c.clone()])
        .await
        .unwrap();

    let rel_ab = MemoryRelationship {
        from: a.name.clone(),
        to: b.name.clone(),
        name: "relates_to".to_string(),
        properties: HashMap::default(),
    };
    let rel_bc = MemoryRelationship {
        from: b.name.clone(),
        to: c.name.clone(),
        name: "relates_to".to_string(),
        properties: HashMap::default(),
    };
    service
        .create_relationships(&[rel_ab, rel_bc])
        .await
        .unwrap();

    let related = service
        .find_related_entities(
            &a.name,
            Some("relates_to".to_string()),
            Some(RelationshipDirection::Outgoing),
            2,
        )
        .await
        .unwrap();
    assert!(related.iter().any(|e| e.name == b.name));
    assert!(related.iter().any(|e| e.name == c.name));
}

#[tokio::test]
async fn test_find_entities_by_labels() {
    let config = Neo4jConfig {
        uri: "neo4j://localhost:7688".to_string(),
        username: "neo4j".to_string(),
        password: "password".to_string(),
    };

    let service = create_neo4j_service(
        config,
        MemoryConfig {
            default_label: Some("LabelTest".to_string()),
            default_relationships: true,
            additional_relationships: std::collections::HashSet::default(),
            default_labels: true,
            additional_labels: ["Example".to_string(), "Extra".to_string()]
                .into_iter()
                .collect(),
        },
    )
    .await
    .unwrap();

    let a = MemoryEntity {
        name: "label:a".to_string(),
        labels: vec!["Example".to_string()],
        ..Default::default()
    };
    let b = MemoryEntity {
        name: "label:b".to_string(),
        labels: vec!["Example".to_string(), "Extra".to_string()],
        ..Default::default()
    };

    service
        .create_entities(&[a.clone(), b.clone()])
        .await
        .unwrap();

    let any = service
        .find_entities_by_labels(&["Extra".to_string()], LabelMatchMode::Any, None)
        .await
        .unwrap();
    assert!(any.iter().any(|e| e.name == b.name));
    assert!(!any.iter().any(|e| e.name == a.name));

    let all = service
        .find_entities_by_labels(
            &["Example".to_string(), "Extra".to_string()],
            LabelMatchMode::All,
            None,
        )
        .await
        .unwrap();
    assert!(all.iter().any(|e| e.name == b.name));

    let req = service
        .find_entities_by_labels(&[], LabelMatchMode::Any, Some("Extra".to_string()))
        .await
        .unwrap();
    assert!(req.iter().all(|e| e.labels.contains(&"Extra".to_string())));
}

#[tokio::test]
async fn test_run_memory_service_suite() {
    let config = Neo4jConfig {
        uri: "neo4j://localhost:7688".to_string(),
        username: "neo4j".to_string(),
        password: "password".to_string(),
    };

    let repo = Neo4jRepository::new(config).await.unwrap();
    run_memory_service_test_suite(repo).await.unwrap();
}
