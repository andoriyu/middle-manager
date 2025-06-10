use neo4rs::{Graph, Node, Query};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur when interacting with the memory store
#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Database connection error: {0}")]
    ConnectionError(String),
    
    #[error("Query execution error: {0}")]
    QueryError(String),
    
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
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
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
        params.insert("name".to_string(), entity.name.clone());
        params.insert("observations".to_string(), serde_json::to_string(&entity.observations)
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
        let query = Query::new("MATCH (n {name: $name}) RETURN n".to_string())
            .param("name", name.to_string());
        
        let mut result = self.graph.execute(query).await
            .map_err(|e| MemoryError::QueryError(e.to_string()))?;
        
        if let Some(row) = result.next().await
            .map_err(|e| MemoryError::QueryError(e.to_string()))? {
            
            let node: Node = row.get("n")
                .map_err(|e| MemoryError::QueryError(e.to_string()))?;
            
            let labels: Vec<String> = node.labels().iter().map(|s| s.to_string()).collect();
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
                        properties.insert(key.to_string(), value);
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
}
