use std::collections::HashMap;
use neo4rs::{self, Graph, Node, Query};
use serde_json;

use crate::domain::entity::MemoryEntity;
use crate::domain::error::{MemoryError, MemoryResult};
use crate::domain::validation_error::ValidationError;
use crate::ports::repository::MemoryRepository;

/// Configuration for connecting to Neo4j
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Neo4jConfig {
    pub uri: String,
    pub username: String,
    pub password: String,
}

/// Neo4j implementation of the MemoryRepository
pub struct Neo4jRepository {
    graph: Graph,
}

impl Neo4jRepository {
    /// Create a new Neo4j repository with the given configuration
    pub async fn new(config: Neo4jConfig) -> Result<Self, MemoryError<neo4rs::Error>> {
        let graph = Graph::new(&config.uri, &config.username, &config.password)
            .await
            .map_err(|e| MemoryError::connection_error_with_source(
                format!("Failed to connect to Neo4j at {}", config.uri),
                e
            ))?;
        
        Ok(Self { graph })
    }
}

impl MemoryRepository<neo4rs::Error> for Neo4jRepository {
    async fn create_entity(&self, entity: &MemoryEntity) -> MemoryResult<(), neo4rs::Error> {
        // Validate entity
        if entity.name.is_empty() {
            return Err(ValidationError::EmptyEntityName.into());
        }
        
        if entity.labels.is_empty() {
            return Err(ValidationError::NoLabels(entity.name.clone()).into());
        }
        
        let labels = entity.labels.join(":");
        let mut params = HashMap::new();
        params.insert("name".to_string(), entity.name.clone());
        
        // Serialize observations
        let observations_json = serde_json::to_string(&entity.observations)?;
        params.insert("observations".to_string(), observations_json);
        
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
            .map_err(|e| MemoryError::query_error_with_source(
                format!("Failed to create entity {}", entity.name),
                e
            ))?;
        
        Ok(())
    }
    
    async fn find_entity_by_name(&self, name: &str) -> MemoryResult<Option<MemoryEntity>, neo4rs::Error> {
        // Validate name
        if name.is_empty() {
            return Err(ValidationError::EmptyEntityName.into());
        }
        
        let query = Query::new("MATCH (n {name: $name}) RETURN n".to_string())
            .param("name", name.to_string());
        
        let mut result = self.graph.execute(query).await
            .map_err(|e| MemoryError::query_error_with_source(
                format!("Failed to execute query to find entity {}", name),
                e
            ))?;
        
        if let Some(row) = result.next().await
            .map_err(|e| MemoryError::query_error_with_source(
                format!("Failed to retrieve result for entity {}", name),
                e
            ))? {
            
            // Get the node from the result
            let node = match row.get::<Node>("n") {
                Ok(n) => n,
                Err(e) => {
                    return Err(MemoryError::runtime_error_with_source(
                        format!("Failed to get node from result for entity {}", name),
                        e
                    ));
                }
            };
            
            // Get the name property
            let entity_name = match node.get::<String>("name") {
                Ok(n) => n,
                Err(e) => {
                    return Err(MemoryError::runtime_error_with_source(
                        "Failed to get name property from node".to_string(),
                        e
                    ));
                }
            };
            
            // Get the observations property
            let observations_json = match node.get::<String>("observations") {
                Ok(o) => o,
                Err(e) => {
                    return Err(MemoryError::runtime_error_with_source(
                        "Failed to get observations property from node".to_string(),
                        e
                    ));
                }
            };
            
            // Deserialize observations
            let observations: Vec<String> = serde_json::from_str(&observations_json)?;
            
            // Extract labels
            let labels: Vec<String> = node.labels().iter().map(|s| s.to_string()).collect();
            
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
                name: entity_name,
                labels,
                observations,
                properties,
            }))
        } else {
            Ok(None)
        }
    }
}
