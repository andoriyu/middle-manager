use mm_memory::{MemoryEntity, MemoryStore, Neo4jConfig};
use std::collections::HashMap;

#[tokio::test]
async fn test_find_nonexistent_entity() {
    let config = Neo4jConfig {
        uri: "neo4j://localhost:7688".to_string(),
        username: "neo4j".to_string(),
        password: "password".to_string(),
    };
    
    let store = MemoryStore::new(config).await.unwrap();
    
    // Test that finding a non-existent entity returns None
    let result = store.find_entity_by_name("non:existent:entity").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_create_and_find_entity() {
    let config = Neo4jConfig {
        uri: "neo4j://localhost:7688".to_string(),
        username: "neo4j".to_string(),
        password: "password".to_string(),
    };
    
    let store = MemoryStore::new(config).await.unwrap();
    
    let entity = MemoryEntity {
        name: "test:entity:create".to_string(),
        labels: vec!["Memory".to_string(), "Test".to_string()],
        observations: vec!["This is a test entity for creation".to_string()],
        properties: HashMap::new(),
    };
    
    // Test that entity creation doesn't error
    store.create_entity(&entity).await.unwrap();
    
    // Test that we can find the entity after creation
    let found = store.find_entity_by_name("test:entity:create").await.unwrap();
    assert!(found.is_some());
    
    let found_entity = found.unwrap();
    assert_eq!(found_entity.name, entity.name);
    assert_eq!(found_entity.observations, entity.observations);
    
    // Check that labels contain the expected values
    assert!(found_entity.labels.contains(&"Memory".to_string()));
    assert!(found_entity.labels.contains(&"Test".to_string()));
}
