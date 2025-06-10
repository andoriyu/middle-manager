use async_trait::async_trait;
use neo4rs::{Graph, Node, Query, Relation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, error, info};

/// Errors that can occur when interacting with the memory store
#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Database connection error: {0}")]
    ConnectionError(String),
    
    #[error("Query execution error: {0}")]
    QueryError(String),
    
    #[error("Entity not found: {0}")]
    NotFound(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Result type for memory operations
pub type MemoryResult<T> = Result<T, MemoryError>;

/// Configuration for connecting to Neo4j
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Neo4jConfig {
    pub uri: String,
    pub username: String,
    pub password: String,
}

/// Memory entity representing a node in the knowledge graph
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemoryEntity {
    pub name: String,
    pub labels: Vec<String>,
    pub observations: Vec<String>,
    pub properties: HashMap<String, String>,
}

/// Memory store for interacting with Neo4j
pub struct MemoryStore {
    graph: Graph,
}

impl MemoryStore {
    /// Create a new memory store with the given configuration
    pub async fn new(config: Neo4jConfig) -> MemoryResult<Self> {
        let graph = Graph::new(&config.uri, &config.username, &config.password)
            .await
            .map_err(|e| MemoryError::ConnectionError(e.to_string()))?;
        
        Ok(Self { graph })
    }
    
    /// Create a new entity in the memory graph
    pub async fn create_entity(&self, entity: &MemoryEntity) -> MemoryResult<()> {
        let labels = entity.labels.join(":");
        let mut params = HashMap::new();
        params.insert("name", entity.name.clone());
        params.insert("observations", serde_json::to_string(&entity.observations)
            .map_err(|e| MemoryError::SerializationError(e.to_string()))?);
        
        // Add all custom properties
        for (key, value) in &entity.properties {
            params.insert(key.clone(), value.clone());
        }
        
        let query_str = format!(
            "CREATE (n:{} {{name: $name, observations: $observations}})",
            labels
        );
        
        let query = Query::new(query_str).params(params);
        self.graph.run(query).await
            .map_err(|e| MemoryError::QueryError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Find an entity by name
    pub async fn find_entity_by_name(&self, name: &str) -> MemoryResult<Option<MemoryEntity>> {
        let query = Query::new("MATCH (n {name: $name}) RETURN n")
            .param("name", name);
        
        let mut result = self.graph.execute(query).await
            .map_err(|e| MemoryError::QueryError(e.to_string()))?;
        
        if let Some(row) = result.next().await
            .map_err(|e| MemoryError::QueryError(e.to_string()))? {
            
            let node: Node = row.get("n")
                .map_err(|e| MemoryError::QueryError(e.to_string()))?;
            
            let labels = node.labels();
            let name = node.get::<String>("name")
                .map_err(|e| MemoryError::QueryError(e.to_string()))?;
            
            let observations_json = node.get::<String>("observations")
                .map_err(|e| MemoryError::QueryError(e.to_string()))?;
            
            let observations: Vec<String> = serde_json::from_str(&observations_json)
                .map_err(|e| MemoryError::SerializationError(e.to_string()))?;
            
            // Extract all other properties
            let mut properties = HashMap::new();
            for key in node.keys() {
                if key != "name" && key != "observations" {
                    if let Ok(value) = node.get::<String>(&key) {
                        properties.insert(key, value);
                    }
                }
            }
            
            Ok(Some(MemoryEntity {
                name,
                labels,
                observations,
                properties,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Create a relationship between two entities
    pub async fn create_relationship(
        &self,
        from_name: &str,
        to_name: &str,
        relationship_type: &str,
        properties: HashMap<String, String>,
    ) -> MemoryResult<()> {
        let mut params = HashMap::new();
        params.insert("from_name", from_name.to_string());
        params.insert("to_name", to_name.to_string());
        
        // Add relationship properties
        for (key, value) in &properties {
            params.insert(key.clone(), value.clone());
        }
        
        let props_str = if properties.is_empty() {
            String::new()
        } else {
            let props = properties.keys()
                .map(|k| format!("{}: ${}", k, k))
                .collect::<Vec<_>>()
                .join(", ");
            format!(" {{{}}}", props)
        };
        
        let query_str = format!(
            "MATCH (a {{name: $from_name}}), (b {{name: $to_name}}) 
             CREATE (a)-[r:{}{}]->(b)",
            relationship_type, props_str
        );
        
        let query = Query::new(query_str).params(params);
        self.graph.run(query).await
            .map_err(|e| MemoryError::QueryError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Execute a custom Cypher query
    pub async fn execute_query<T: for<'de> Deserialize<'de>>(
        &self,
        query_str: &str,
        params: HashMap<&str, String>,
    ) -> MemoryResult<Vec<T>> {
        let query = Query::new(query_str).params(params);
        let mut result = self.graph.execute(query).await
            .map_err(|e| MemoryError::QueryError(e.to_string()))?;
        
        let mut items = Vec::new();
        while let Some(row) = result.next().await
            .map_err(|e| MemoryError::QueryError(e.to_string()))? {
            
            let json = serde_json::to_value(&row)
                .map_err(|e| MemoryError::SerializationError(e.to_string()))?;
            
            let item = serde_json::from_value(json)
                .map_err(|e| MemoryError::SerializationError(e.to_string()))?;
            
            items.push(item);
        }
        
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // These tests require a running Neo4j instance
    // They are ignored by default and can be run with:
    // cargo test -- --ignored
    
    #[tokio::test]
    #[ignore]
    async fn test_create_and_find_entity() {
        let config = Neo4jConfig {
            uri: "neo4j://localhost:7687".to_string(),
            username: "neo4j".to_string(),
            password: "password".to_string(),
        };
        
        let store = MemoryStore::new(config).await.unwrap();
        
        let entity = MemoryEntity {
            name: "test:entity:1".to_string(),
            labels: vec!["Memory".to_string(), "Test".to_string()],
            observations: vec!["This is a test entity".to_string()],
            properties: HashMap::new(),
        };
        
        store.create_entity(&entity).await.unwrap();
        
        let found = store.find_entity_by_name("test:entity:1").await.unwrap();
        assert!(found.is_some());
        
        let found = found.unwrap();
        assert_eq!(found.name, "test:entity:1");
        assert!(found.labels.contains(&"Memory".to_string()));
        assert!(found.labels.contains(&"Test".to_string()));
        assert_eq!(found.observations.len(), 1);
        assert_eq!(found.observations[0], "This is a test entity");
    }
}
