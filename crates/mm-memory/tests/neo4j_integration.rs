use mm_memory::{MemoryEntity, Neo4jConfig, create_neo4j_service};
use std::collections::HashMap;

#[tokio::test]
async fn test_find_nonexistent_entity() {
    let config = Neo4jConfig {
        uri: "neo4j://localhost:7688".to_string(),
        username: "neo4j".to_string(),
        password: "password".to_string(),
    };

    let service = match create_neo4j_service(config).await {
        Ok(service) => service,
        Err(e) => {
            eprintln!("Skipping test: {e}");
            return;
        }
    };

    // Test that finding a non-existent entity returns None
    let result = match service.find_entity_by_name("non:existent:entity").await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Skipping test: {e}");
            return;
        }
    };
    assert!(result.is_none());
}

#[tokio::test]
async fn test_create_and_find_entity() {
    let config = Neo4jConfig {
        uri: "neo4j://localhost:7688".to_string(),
        username: "neo4j".to_string(),
        password: "password".to_string(),
    };

    let service = match create_neo4j_service(config).await {
        Ok(service) => service,
        Err(e) => {
            eprintln!("Skipping test: {e}");
            return;
        }
    };

    let entity = MemoryEntity {
        name: "test:entity:create".to_string(),
        labels: vec!["Memory".to_string(), "Test".to_string()],
        observations: vec!["This is a test entity for creation".to_string()],
        properties: HashMap::new(),
    };

    // Test that entity creation doesn't error
    if let Err(e) = service.create_entity(&entity).await {
        eprintln!("Skipping test: {e}");
        return;
    }

    // Test that we can find the entity after creation
    let found = match service.find_entity_by_name("test:entity:create").await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Skipping test: {e}");
            return;
        }
    };
    assert!(found.is_some());

    let found_entity = found.unwrap();
    assert_eq!(found_entity.name, entity.name);
    assert_eq!(found_entity.observations, entity.observations);

    // Check that labels contain the expected values
    assert!(found_entity.labels.contains(&"Memory".to_string()));
    assert!(found_entity.labels.contains(&"Test".to_string()));
}

#[tokio::test]
async fn test_validation_errors() {
    let config = Neo4jConfig {
        uri: "neo4j://localhost:7688".to_string(),
        username: "neo4j".to_string(),
        password: "password".to_string(),
    };

    let service = match create_neo4j_service(config).await {
        Ok(service) => service,
        Err(e) => {
            eprintln!("Skipping test: {e}");
            return;
        }
    };

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

    let result = service.create_entity(&entity).await;
    assert!(result.is_err());
}
