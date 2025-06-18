use mm_memory::MemoryRelationship;
use mm_memory_neo4j::{MemoryConfig, MemoryEntity, MemoryError, Neo4jConfig, create_neo4j_service};
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
            default_tag: Some("TestFindNone".to_string()),
            default_relationships: true,
            additional_relationships: std::collections::HashSet::new(),
            default_labels: true,
            additional_labels: std::collections::HashSet::new(),
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
            default_tag: Some("TestCreate".to_string()),
            default_relationships: true,
            additional_relationships: std::collections::HashSet::new(),
            default_labels: true,
            additional_labels: std::collections::HashSet::new(),
        },
    )
    .await
    .unwrap();

    let entity = MemoryEntity {
        name: "test:entity:create".to_string(),
        labels: vec!["Example".to_string()],
        observations: vec!["This is a test entity for creation".to_string()],
        properties: HashMap::new(),
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
            default_tag: Some("TestValidation".to_string()),
            default_relationships: true,
            additional_relationships: std::collections::HashSet::new(),
            default_labels: true,
            additional_labels: std::collections::HashSet::new(),
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
        labels: vec![],
        observations: vec!["This entity has no labels".to_string()],
        properties: HashMap::new(),
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
            default_tag: Some("TestSet".to_string()),
            default_relationships: true,
            additional_relationships: std::collections::HashSet::new(),
            default_labels: true,
            additional_labels: std::collections::HashSet::new(),
        },
    )
    .await
    .unwrap();

    let entity_name = "test:entity:set";
    let entity = MemoryEntity {
        name: entity_name.to_string(),
        labels: vec!["Example".to_string()],
        observations: vec!["initial".to_string()],
        properties: HashMap::new(),
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
            default_tag: Some("TestAddRemove".to_string()),
            default_relationships: true,
            additional_relationships: std::collections::HashSet::new(),
            default_labels: true,
            additional_labels: std::collections::HashSet::new(),
        },
    )
    .await
    .unwrap();

    let entity_name = "test:entity:modify";
    let entity = MemoryEntity {
        name: entity_name.to_string(),
        labels: vec!["Example".to_string()],
        observations: vec!["obs1".to_string(), "obs2".to_string()],
        properties: HashMap::new(),
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
            default_tag: Some("RelationshipTest".to_string()),
            default_relationships: true,
            additional_relationships: std::collections::HashSet::new(),
            default_labels: true,
            additional_labels: std::collections::HashSet::new(),
        },
    )
    .await
    .unwrap();

    let a = MemoryEntity {
        name: "rel:a".to_string(),
        labels: vec!["Example".to_string()],
        observations: vec![],
        properties: HashMap::new(),
    };
    let b = MemoryEntity {
        name: "rel:b".to_string(),
        labels: vec!["Example".to_string()],
        observations: vec![],
        properties: HashMap::new(),
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
        properties: HashMap::new(),
    };

    service
        .create_relationships(std::slice::from_ref(&rel))
        .await
        .unwrap();
}
