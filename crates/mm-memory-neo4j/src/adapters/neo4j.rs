use async_trait::async_trait;
use neo4rs::{self, Graph, Node, Query};
use serde_json;
use std::collections::HashMap;

use mm_memory::{
    MemoryEntity, MemoryError, MemoryRelationship, MemoryRepository, MemoryResult, ValidationError,
    ValidationErrorKind,
};

/// Configuration for connecting to Neo4j
///
/// This struct contains the configuration parameters needed to connect to a Neo4j database.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Neo4jConfig {
    /// URI of the Neo4j server (e.g., "neo4j://localhost:7688")
    pub uri: String,

    /// Username for authentication
    pub username: String,

    /// Password for authentication
    pub password: String,
}

/// Neo4j implementation of the MemoryRepository
///
/// This adapter implements the `MemoryRepository` trait using Neo4j as the backend storage.
/// It handles the details of connecting to Neo4j, executing Cypher queries, and
/// converting between Neo4j nodes and domain entities.
pub struct Neo4jRepository {
    /// Neo4j graph connection
    graph: Graph,
}

impl Neo4jRepository {
    /// Create a new Neo4j repository with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for connecting to Neo4j
    ///
    /// # Returns
    ///
    /// A new `Neo4jRepository` if the connection was successful
    ///
    /// # Errors
    ///
    /// Returns a `MemoryError` if the connection to Neo4j fails
    pub async fn new(config: Neo4jConfig) -> Result<Self, MemoryError<neo4rs::Error>> {
        let graph = Graph::new(&config.uri, &config.username, &config.password)
            .await
            .map_err(|e| {
                MemoryError::connection_error_with_source(
                    format!("Failed to connect to Neo4j at {}", config.uri),
                    e,
                )
            })?;

        Ok(Self { graph })
    }
}

#[async_trait]
impl MemoryRepository for Neo4jRepository {
    type Error = neo4rs::Error;
    async fn create_entity(&self, entity: &MemoryEntity) -> MemoryResult<(), Self::Error> {
        // Validate entity
        if entity.name.is_empty() {
            return Err(ValidationError::from(ValidationErrorKind::EmptyEntityName).into());
        }

        if entity.labels.is_empty() {
            return Err(
                ValidationError::from(ValidationErrorKind::NoLabels(entity.name.clone())).into(),
            );
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
        self.graph.run(query).await.map_err(|e| {
            MemoryError::query_error_with_source(
                format!("Failed to create entity {}", entity.name),
                e,
            )
        })?;

        Ok(())
    }

    async fn find_entity_by_name(
        &self,
        name: &str,
    ) -> MemoryResult<Option<MemoryEntity>, Self::Error> {
        // Validate name
        if name.is_empty() {
            return Err(ValidationError::from(ValidationErrorKind::EmptyEntityName).into());
        }

        let query = Query::new("MATCH (n {name: $name}) RETURN n".to_string())
            .param("name", name.to_string());

        let mut result = self.graph.execute(query).await.map_err(|e| {
            MemoryError::query_error_with_source(
                format!("Failed to execute query to find entity {}", name),
                e,
            )
        })?;

        if let Some(row) = result.next().await.map_err(|e| {
            MemoryError::query_error_with_source(
                format!("Failed to retrieve result for entity {}", name),
                e,
            )
        })? {
            // Get the node from the result
            let node = match row.get::<Node>("n") {
                Ok(n) => n,
                Err(e) => {
                    return Err(MemoryError::runtime_error_with_source(
                        format!("Failed to get node from result for entity {}", name),
                        e,
                    ));
                }
            };

            // Get the name property
            let entity_name = match node.get::<String>("name") {
                Ok(n) => n,
                Err(e) => {
                    return Err(MemoryError::runtime_error_with_source(
                        "Failed to get name property from node".to_string(),
                        e,
                    ));
                }
            };

            // Get the observations property
            let observations_json = match node.get::<String>("observations") {
                Ok(o) => o,
                Err(e) => {
                    return Err(MemoryError::runtime_error_with_source(
                        "Failed to get observations property from node".to_string(),
                        e,
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
                    if let Ok(value) = node.get::<String>(key) {
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

    async fn set_observations(
        &self,
        name: &str,
        observations: &[String],
    ) -> MemoryResult<(), Self::Error> {
        if name.is_empty() {
            return Err(ValidationError::from(ValidationErrorKind::EmptyEntityName).into());
        }

        let observations_json = serde_json::to_string(observations)?;
        let query =
            Query::new("MATCH (n {name: $name}) SET n.observations = $observations".to_string())
                .param("name", name.to_string())
                .param("observations", observations_json);

        self.graph.run(query).await.map_err(|e| {
            MemoryError::query_error_with_source(
                format!("Failed to set observations for entity {}", name),
                e,
            )
        })?;

        Ok(())
    }

    async fn add_observations(
        &self,
        name: &str,
        observations: &[String],
    ) -> MemoryResult<(), Self::Error> {
        let mut current = match self.find_entity_by_name(name).await? {
            Some(e) => e.observations,
            None => Vec::new(),
        };
        current.extend_from_slice(observations);
        self.set_observations(name, &current).await
    }

    async fn remove_all_observations(&self, name: &str) -> MemoryResult<(), Self::Error> {
        self.set_observations(name, &[]).await
    }

    async fn remove_observations(
        &self,
        name: &str,
        observations: &[String],
    ) -> MemoryResult<(), Self::Error> {
        let mut current = match self.find_entity_by_name(name).await? {
            Some(e) => e.observations,
            None => Vec::new(),
        };
        current.retain(|o| !observations.contains(o));
        self.set_observations(name, &current).await
    }

    async fn create_relationship(
        &self,
        relationship: &MemoryRelationship,
    ) -> MemoryResult<(), Self::Error> {
        let mut params = HashMap::new();
        params.insert("from".to_string(), relationship.from.clone());
        params.insert("to".to_string(), relationship.to.clone());

        for (k, v) in &relationship.properties {
            params.insert(k.clone(), v.clone());
        }

        let props = if relationship.properties.is_empty() {
            String::new()
        } else {
            let pairs: Vec<String> = relationship
                .properties
                .keys()
                .map(|k| format!("{}: ${}", k, k))
                .collect();
            format!(" {{{}}}", pairs.join(", "))
        };

        let query_str = format!(
            "MATCH (a {{name: $from}}), (b {{name: $to}}) CREATE (a)-[:{}{}]->(b)",
            relationship.name, props
        );
        let query = Query::new(query_str).params(params);
        self.graph.run(query).await.map_err(|e| {
            MemoryError::query_error_with_source(
                format!("Failed to create relationship {}", relationship.name),
                e,
            )
        })?;

        Ok(())
    }
}
