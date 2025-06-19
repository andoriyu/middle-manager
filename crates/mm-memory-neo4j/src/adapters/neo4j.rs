use crate::adapters::conversions::{bolt_to_memory_value, memory_value_to_bolt};
use async_trait::async_trait;
use neo4rs::{self, Graph, Node, Query};
use serde_json;
use std::collections::HashMap;
use tracing::instrument;

use mm_memory::{
    MemoryEntity, MemoryError, MemoryRelationship, MemoryRepository, MemoryResult, MemoryValue,
    ValidationError, ValidationErrorKind,
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
    #[instrument(skip(config), fields(uri = %config.uri))]
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

    /// Parse relationships from Neo4j BoltType
    ///
    /// This function converts a Neo4j BoltType (typically a List of Maps) into a Vec of MemoryRelationship.
    /// Each map in the list should contain 'from', 'to', 'name', and optionally 'properties'.
    fn parse_relationships_from_bolt(
        bolt: neo4rs::BoltType,
    ) -> MemoryResult<Vec<MemoryRelationship>, neo4rs::Error> {
        let mut relationships = Vec::new();

        // Handle empty list or null
        if let neo4rs::BoltType::Null(_) = bolt {
            return Ok(relationships);
        }

        if let neo4rs::BoltType::List(rel_list) = bolt {
            // If the list is empty, return an empty vector
            if rel_list.is_empty() {
                return Ok(relationships);
            }

            for rel_item in rel_list {
                if let neo4rs::BoltType::Map(rel_map) = rel_item {
                    // Skip null entries (empty relationships)
                    if rel_map.value.is_empty() {
                        continue;
                    }

                    // Extract from
                    let from = match rel_map.get("from") {
                        Ok(neo4rs::BoltType::String(s)) => s.to_string(),
                        Ok(neo4rs::BoltType::Null(_)) => {
                            // Skip null values for required fields
                            continue;
                        },
                        Err(e) => {
                            tracing::error!("Failed to get 'from' field from relationship: {}", e);
                            continue;
                        },
                        Ok(other) => {
                            tracing::error!("Expected string for 'from' field, got: {:?}", other);
                            continue;
                        }
                    };
                    
                    // Extract to
                    let to = match rel_map.get("to") {
                        Ok(neo4rs::BoltType::String(s)) => s.to_string(),
                        Ok(neo4rs::BoltType::Null(_)) => {
                            // Skip null values for required fields
                            continue;
                        },
                        Err(e) => {
                            tracing::error!("Failed to get 'to' field from relationship: {}", e);
                            continue;
                        },
                        Ok(other) => {
                            tracing::error!("Expected string for 'to' field, got: {:?}", other);
                            continue;
                        }
                    };
                    
                    // Extract name
                    let name = match rel_map.get("name") {
                        Ok(neo4rs::BoltType::String(s)) => s.to_string(),
                        Ok(neo4rs::BoltType::Null(_)) => {
                            // Skip null values for required fields
                            continue;
                        },
                        Err(e) => {
                            tracing::error!("Failed to get 'name' field from relationship: {}", e);
                            continue;
                        },
                        Ok(other) => {
                            tracing::error!("Expected string for 'name' field, got: {:?}", other);
                            continue;
                        }
                    };
                    
                    // Extract properties
                    let mut properties = HashMap::new();
                    if let Ok(neo4rs::BoltType::Map(props_map)) = rel_map.get("properties") {
                        for (key, value) in &props_map.value {
                            match bolt_to_memory_value(value.clone()) {
                                Ok(memory_value) => {
                                    properties.insert(key.to_string(), memory_value);
                                },
                                Err(e) => {
                                    tracing::error!(
                                        "Failed to convert property '{}' in relationship {}-[{}]->{}: {}", 
                                        key, from, name, to, e
                                    );
                                }
                            }
                        }
                    } else {
                        tracing::debug!("No properties found for relationship {}-[{}]->{}", from, name, to);
                    }
                    
                    relationships.push(MemoryRelationship {
                        from,
                        to,
                        name,
                        properties,
                    });
                } else if let neo4rs::BoltType::Null(_) = rel_item {
                    // Skip null entries
                    continue;
                } else {
                    tracing::error!("Expected Map for relationship, got: {:?}", rel_item);
                }
            }
        } else {
            tracing::error!("Expected List for relationships, got: {:?}", bolt);
        }
        
        Ok(relationships)
    }
}

#[async_trait]
impl MemoryRepository for Neo4jRepository {
    type Error = neo4rs::Error;
    #[instrument(skip(self, entities), fields(count = entities.len()))]
    async fn create_entities(&self, entities: &[MemoryEntity]) -> MemoryResult<(), Self::Error> {
        if entities.is_empty() {
            return Ok(());
        }

        let mut batch: Vec<HashMap<String, neo4rs::BoltType>> = Vec::default();
        for entity in entities {
            let mut props: HashMap<String, neo4rs::BoltType> = HashMap::default();
            props.insert("name".to_string(), entity.name.clone().into());
            let observations_json = serde_json::to_string(&entity.observations)?;
            props.insert("observations".to_string(), observations_json.into());
            for (k, v) in &entity.properties {
                let bolt = memory_value_to_bolt(v)?;
                props.insert(k.clone(), bolt);
            }

            let mut row: HashMap<String, neo4rs::BoltType> = HashMap::default();
            row.insert("labels".to_string(), entity.labels.clone().into());
            row.insert("props".to_string(), props.into());
            batch.push(row);
        }

        let query = Query::new(
            "UNWIND $rows AS row CALL apoc.create.node(row.labels, row.props) YIELD node RETURN count(node)"
                .to_string(),
        )
        .param("rows", batch);

        self.graph.run(query).await.map_err(|e| {
            MemoryError::query_error_with_source("Failed to create entities".to_string(), e)
        })?;

        Ok(())
    }

    #[instrument(skip(self), fields(name = %name))]
    async fn find_entity_by_name(
        &self,
        name: &str,
    ) -> MemoryResult<Option<MemoryEntity>, Self::Error> {
        // Validate name
        if name.is_empty() {
            return Err(ValidationError::from(ValidationErrorKind::EmptyEntityName).into());
        }

        let query = Query::new(
            "MATCH (n {name: $name}) \n\
             OPTIONAL MATCH (n)-[r]-() \n\
             WITH n, collect({from: startNode(r).name, to: endNode(r).name, name: type(r), properties: properties(r)}) as rels \n\
             RETURN n, rels"
                .to_string(),
        )
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
            let mut properties: HashMap<String, MemoryValue> = HashMap::default();
            for key in node.keys() {
                if key != "name" && key != "observations" {
                    let bolt: neo4rs::BoltType = node.get(key).map_err(|e| {
                        MemoryError::runtime_error_with_source(
                            "Failed to decode node properties".to_string(),
                            e,
                        )
                    })?;
                    let mv = bolt_to_memory_value(bolt)?;
                    properties.insert(key.to_string(), mv);
                }
            }

            // Parse relationships
            let rels_bolt = row.get::<neo4rs::BoltType>("rels").map_err(|e| {
                MemoryError::runtime_error_with_source(
                    "Failed to decode relationships".to_string(),
                    e,
                )
            })?;

            let relationships = Self::parse_relationships_from_bolt(rels_bolt)?;

            Ok(Some(MemoryEntity {
                name: entity_name,
                labels,
                observations,
                properties,
                relationships,
            }))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self, observations), fields(name = %name))]
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

    #[instrument(skip(self, observations), fields(name = %name))]
    async fn add_observations(
        &self,
        name: &str,
        observations: &[String],
    ) -> MemoryResult<(), Self::Error> {
        let mut current = match self.find_entity_by_name(name).await? {
            Some(e) => e.observations,
            None => Vec::default(),
        };
        current.extend_from_slice(observations);
        self.set_observations(name, &current).await
    }

    #[instrument(skip(self), fields(name = %name))]
    async fn remove_all_observations(&self, name: &str) -> MemoryResult<(), Self::Error> {
        self.set_observations(name, &[]).await
    }

    #[instrument(skip(self, observations), fields(name = %name))]
    async fn remove_observations(
        &self,
        name: &str,
        observations: &[String],
    ) -> MemoryResult<(), Self::Error> {
        let mut current = match self.find_entity_by_name(name).await? {
            Some(e) => e.observations,
            None => Vec::default(),
        };
        current.retain(|o| !observations.contains(o));
        self.set_observations(name, &current).await
    }

    #[instrument(skip(self, relationships), fields(count = relationships.len()))]
    async fn create_relationships(
        &self,
        relationships: &[MemoryRelationship],
    ) -> MemoryResult<(), Self::Error> {
        if relationships.is_empty() {
            return Ok(());
        }

        let mut rows: Vec<HashMap<String, neo4rs::BoltType>> = Vec::default();
        for rel in relationships {
            let mut props: HashMap<String, neo4rs::BoltType> = HashMap::default();
            for (k, v) in &rel.properties {
                let bolt = memory_value_to_bolt(v)?;
                props.insert(k.clone(), bolt);
            }

            let mut row: HashMap<String, neo4rs::BoltType> = HashMap::default();
            row.insert("from".to_string(), rel.from.clone().into());
            row.insert("to".to_string(), rel.to.clone().into());
            row.insert("name".to_string(), rel.name.clone().into());
            row.insert("props".to_string(), props.into());
            rows.push(row);
        }

        let query = Query::new(
            "UNWIND $rows AS row MATCH (a {name: row.from}), (b {name: row.to}) CALL apoc.create.relationship(a, row.name, row.props, b) YIELD rel RETURN count(rel)"
                .to_string(),
        )
        .param("rows", rows);

        self.graph.run(query).await.map_err(|e| {
            MemoryError::query_error_with_source("Failed to create relationships".to_string(), e)
        })?;

        Ok(())
    }
}
