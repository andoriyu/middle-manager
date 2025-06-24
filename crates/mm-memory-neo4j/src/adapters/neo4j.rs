use crate::adapters::conversions::{bolt_to_memory_value, memory_value_to_bolt};
use async_trait::async_trait;
use neo4rs::{self, Graph, Node, Query};
use std::collections::HashMap;
use tracing::instrument;

use mm_memory::{
    EntityUpdate, LabelMatchMode, MemoryEntity, MemoryError, MemoryRelationship, MemoryRepository,
    MemoryResult, MemoryValue, RelationshipDirection, RelationshipUpdate, ValidationError,
    ValidationErrorKind,
};

/// Configuration for connecting to Neo4j
///
/// This struct contains the configuration parameters needed to connect to a Neo4j database.
#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Neo4jConfig {
    /// URI of the Neo4j server (e.g., "neo4j://localhost:7688")
    pub uri: String,

    /// Username for authentication
    pub username: String,

    /// Password for authentication
    #[serde(skip_serializing)]
    pub password: String,
}

impl std::fmt::Debug for Neo4jConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Neo4jConfig")
            .field("uri", &self.uri)
            .field("username", &self.username)
            .field("password", &"***")
            .finish()
    }
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

    /// Extract observations from a Neo4j BoltType
    ///
    /// This helper method converts a Neo4j BoltType (expected to be a List of Strings)
    /// into a Vec<String> for observations.
    fn extract_observations_from_bolt(
        &self,
        bolt: neo4rs::BoltType,
    ) -> MemoryResult<Vec<String>, neo4rs::Error> {
        match bolt {
            neo4rs::BoltType::List(items) => {
                let result: Result<Vec<String>, _> = items
                    .into_iter()
                    .map(|item| {
                        if let neo4rs::BoltType::String(s) = item {
                            Ok(s.to_string())
                        } else {
                            Err(MemoryError::runtime_error(format!(
                                "Expected string in observations list, got {:?}",
                                item
                            )))
                        }
                    })
                    .collect();
                result
            }
            neo4rs::BoltType::Null(_) => Ok(Vec::new()),
            _ => Err(MemoryError::runtime_error(format!(
                "Expected observations to be a list, got {:?}",
                bolt
            ))),
        }
    }

    /// Extract observations from a Neo4j Node
    ///
    /// This helper method extracts the observations property from a Neo4j Node
    /// and converts it to a Vec<String>.
    fn extract_observations_from_node(
        &self,
        node: &neo4rs::Node,
    ) -> MemoryResult<Vec<String>, neo4rs::Error> {
        let observations_bolt = node.get::<neo4rs::BoltType>("observations").map_err(|e| {
            MemoryError::runtime_error_with_source(
                "Failed to get observations property from node".to_string(),
                e,
            )
        })?;

        self.extract_observations_from_bolt(observations_bolt)
    }

    /// Parse relationships from Neo4j BoltType
    ///
    /// This function converts a Neo4j BoltType (typically a List of Maps) into a Vec of MemoryRelationship.
    /// Each map in the list should contain 'from', 'to', 'name', and optionally 'properties'.
    fn parse_relationships_from_bolt(
        bolt: neo4rs::BoltType,
    ) -> MemoryResult<Vec<MemoryRelationship>, neo4rs::Error> {
        let mut relationships = Vec::new();

        // Handle empty list or null (no relationships case)
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
                    // Skip empty maps (no data)
                    if rel_map.value.is_empty() {
                        continue;
                    }

                    // Extract from (required field)
                    let from = match rel_map.get("from") {
                        Ok(neo4rs::BoltType::String(s)) => s.to_string(),
                        Ok(neo4rs::BoltType::Null(_)) => {
                            return Err(MemoryError::runtime_error(
                                "Required field 'from' is null in relationship".to_string(),
                            ));
                        }
                        Err(e) => {
                            return Err(MemoryError::runtime_error_with_source(
                                "Failed to get required 'from' field from relationship".to_string(),
                                e,
                            ));
                        }
                        Ok(other) => {
                            return Err(MemoryError::runtime_error(format!(
                                "Expected string for required 'from' field, got: {:?}",
                                other
                            )));
                        }
                    };

                    // Extract to (required field)
                    let to = match rel_map.get("to") {
                        Ok(neo4rs::BoltType::String(s)) => s.to_string(),
                        Ok(neo4rs::BoltType::Null(_)) => {
                            return Err(MemoryError::runtime_error(
                                "Required field 'to' is null in relationship".to_string(),
                            ));
                        }
                        Err(e) => {
                            return Err(MemoryError::runtime_error_with_source(
                                "Failed to get required 'to' field from relationship".to_string(),
                                e,
                            ));
                        }
                        Ok(other) => {
                            return Err(MemoryError::runtime_error(format!(
                                "Expected string for required 'to' field, got: {:?}",
                                other
                            )));
                        }
                    };

                    // Extract name (required field)
                    let name = match rel_map.get("name") {
                        Ok(neo4rs::BoltType::String(s)) => s.to_string(),
                        Ok(neo4rs::BoltType::Null(_)) => {
                            return Err(MemoryError::runtime_error(
                                "Required field 'name' is null in relationship".to_string(),
                            ));
                        }
                        Err(e) => {
                            return Err(MemoryError::runtime_error_with_source(
                                "Failed to get required 'name' field from relationship".to_string(),
                                e,
                            ));
                        }
                        Ok(other) => {
                            return Err(MemoryError::runtime_error(format!(
                                "Expected string for required 'name' field, got: {:?}",
                                other
                            )));
                        }
                    };

                    // Extract properties (optional)
                    let mut properties = HashMap::new();
                    if let Ok(neo4rs::BoltType::Map(props_map)) = rel_map.get("properties") {
                        for (key, value) in &props_map.value {
                            match bolt_to_memory_value(value.clone()) {
                                Ok(memory_value) => {
                                    properties.insert(key.to_string(), memory_value);
                                }
                                Err(e) => {
                                    // Property conversion errors are still logged but don't fail the whole operation
                                    tracing::error!(
                                        "Failed to convert property '{}' in relationship {}-[{}]->{}: {}",
                                        key,
                                        from,
                                        name,
                                        to,
                                        e
                                    );
                                }
                            }
                        }
                    } else {
                        tracing::debug!(
                            "No properties found for relationship {}-[{}]->{}",
                            from,
                            name,
                            to
                        );
                    }

                    relationships.push(MemoryRelationship {
                        from,
                        to,
                        name,
                        properties,
                    });
                } else if let neo4rs::BoltType::Null(_) = rel_item {
                    // Skip null entries in the list
                    continue;
                } else {
                    return Err(MemoryError::runtime_error(format!(
                        "Expected Map for relationship, got: {:?}",
                        rel_item
                    )));
                }
            }
        } else {
            return Err(MemoryError::runtime_error(format!(
                "Expected List for relationships, got: {:?}",
                bolt
            )));
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

            // Store observations as a native Neo4j array/sequence
            props.insert(
                "observations".to_string(),
                entity.observations.clone().into(),
            );

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
             WITH n, collect(CASE WHEN r IS NOT NULL THEN {from: startNode(r).name, to: endNode(r).name, name: type(r), properties: properties(r)} END) as rels \n\
             RETURN n, [x IN rels WHERE x IS NOT NULL] as rels"
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

            // Get the observations using our helper method
            let observations = self.extract_observations_from_node(&node)?;

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

        // Store observations as a native Neo4j array/sequence
        let query =
            Query::new("MATCH (n {name: $name}) SET n.observations = $observations".to_string())
                .param("name", name.to_string())
                .param("observations", observations.to_vec());

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

    #[instrument(skip(self), fields(name = %name, depth))]
    async fn find_related_entities(
        &self,
        name: &str,
        relationship_type: Option<String>,
        direction: Option<RelationshipDirection>,
        depth: u32,
    ) -> MemoryResult<Vec<MemoryEntity>, Self::Error> {
        if name.is_empty() {
            return Err(ValidationError::from(ValidationErrorKind::EmptyEntityName).into());
        }

        let dir = direction.unwrap_or(RelationshipDirection::Both);
        let rel_type = relationship_type
            .as_deref()
            .map(|t| format!(":{}", t))
            .unwrap_or_default();
        let pattern = match dir {
            RelationshipDirection::Outgoing => format!("-[r{}*1..{}]->", rel_type, depth),
            RelationshipDirection::Incoming => format!("<-[r{}*1..{}]-", rel_type, depth),
            RelationshipDirection::Both => format!("-[r{}*1..{}]-", rel_type, depth),
        };

        let query_str = format!(
            "MATCH (start {{name: $name}}) MATCH (start){}(n)\n\
             WITH DISTINCT n\n\
             OPTIONAL MATCH (n)-[r]-()\n\
             WITH n, collect(CASE WHEN r IS NOT NULL THEN {{from: startNode(r).name, to: endNode(r).name, name: type(r), properties: properties(r)}} END) as rels\n\
             RETURN n, [x IN rels WHERE x IS NOT NULL] as rels",
            pattern
        );

        let query = Query::new(query_str).param("name", name.to_string());
        let mut result = self.graph.execute(query).await.map_err(|e| {
            MemoryError::query_error_with_source(
                format!("Failed to execute related entity query for {}", name),
                e,
            )
        })?;

        let mut entities = Vec::new();
        while let Some(row) = result.next().await.map_err(|e| {
            MemoryError::query_error_with_source(
                format!("Failed to retrieve related entity results for {}", name),
                e,
            )
        })? {
            let node = row.get::<Node>("n").map_err(|e| {
                MemoryError::runtime_error_with_source(
                    "Failed to get node from result".to_string(),
                    e,
                )
            })?;

            let entity_name = node.get::<String>("name").map_err(|e| {
                MemoryError::runtime_error_with_source("Failed to get name property".to_string(), e)
            })?;

            // Get observations using our helper method
            let observations = self.extract_observations_from_node(&node)?;

            let labels: Vec<String> = node.labels().iter().map(|s| s.to_string()).collect();

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

            let rels_bolt = row.get::<neo4rs::BoltType>("rels").map_err(|e| {
                MemoryError::runtime_error_with_source(
                    "Failed to decode relationships".to_string(),
                    e,
                )
            })?;
            let relationships = Self::parse_relationships_from_bolt(rels_bolt)?;

            entities.push(MemoryEntity {
                name: entity_name,
                labels,
                observations,
                properties,
                relationships,
            });
        }

        Ok(entities)
    }

    #[instrument(skip(self, labels), fields(labels_count = labels.len()))]
    async fn find_entities_by_labels(
        &self,
        labels: &[String],
        match_mode: LabelMatchMode,
        required_label: Option<String>,
    ) -> MemoryResult<Vec<MemoryEntity>, Self::Error> {
        let mut conditions = Vec::new();
        if required_label.is_some() {
            conditions.push("$required IN labels(n)".to_string());
        }
        if !labels.is_empty() {
            let expr = match match_mode {
                LabelMatchMode::Any => "ANY(l IN $labels WHERE l IN labels(n))".to_string(),
                LabelMatchMode::All => "ALL(l IN $labels WHERE l IN labels(n))".to_string(),
            };
            conditions.push(expr);
        }
        let where_clause = if conditions.is_empty() {
            String::default()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let query_str = format!(
            "MATCH (n) {where_clause}\n\
             OPTIONAL MATCH (n)-[r]-()\n\
             WITH n, collect(CASE WHEN r IS NOT NULL THEN {{from: startNode(r).name, to: endNode(r).name, name: type(r), properties: properties(r)}} END) as rels\n\
             RETURN n, [x IN rels WHERE x IS NOT NULL] as rels",
            where_clause = where_clause
        );

        tracing::debug!("Executing Neo4j query: {}", query_str);
        tracing::debug!(
            "Query parameters: labels={:?}, required={:?}",
            labels,
            required_label
        );

        let mut query = Query::new(query_str).param("labels", labels.to_vec());
        if let Some(lbl) = required_label {
            query = query.param("required", lbl);
        }

        let mut result = self.graph.execute(query).await.map_err(|e| {
            MemoryError::query_error_with_source("Failed to execute label query".to_string(), e)
        })?;

        let mut entities = Vec::new();
        while let Some(row) = result.next().await.map_err(|e| {
            MemoryError::query_error_with_source(
                "Failed to retrieve label query results".to_string(),
                e,
            )
        })? {
            let node = row.get::<Node>("n").map_err(|e| {
                MemoryError::runtime_error_with_source(
                    "Failed to get node from result".to_string(),
                    e,
                )
            })?;

            let entity_name = node.get::<String>("name").map_err(|e| {
                MemoryError::runtime_error_with_source("Failed to get name property".to_string(), e)
            })?;

            // Get observations using our helper method
            let observations = self.extract_observations_from_node(&node)?;

            let labels_vec: Vec<String> = node.labels().iter().map(|s| s.to_string()).collect();

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

            let rels_bolt = row.get::<neo4rs::BoltType>("rels").map_err(|e| {
                MemoryError::runtime_error_with_source(
                    "Failed to decode relationships".to_string(),
                    e,
                )
            })?;
            let relationships = Self::parse_relationships_from_bolt(rels_bolt)?;

            entities.push(MemoryEntity {
                name: entity_name,
                labels: labels_vec,
                observations,
                properties,
                relationships,
            });
        }

        Ok(entities)
    }

    async fn update_entity(
        &self,
        name: &str,
        update: &EntityUpdate,
    ) -> MemoryResult<(), Self::Error> {
        if let Some(obs) = &update.observations {
            if let Some(set) = &obs.set {
                self.set_observations(name, set).await?;
            } else if let Some(add) = &obs.add {
                self.add_observations(name, add).await?;
            } else if let Some(remove) = &obs.remove {
                self.remove_observations(name, remove).await?;
            }
        }

        if let Some(props) = &update.properties {
            if let Some(add) = &props.add {
                let mut map: HashMap<String, neo4rs::BoltType> = HashMap::new();
                for (k, v) in add {
                    map.insert(k.clone(), memory_value_to_bolt(v)?);
                }
                let query = Query::new("MATCH (n {name: $name}) SET n += $props".to_string())
                    .param("name", name.to_string())
                    .param("props", map);
                self.graph.run(query).await.map_err(|e| {
                    MemoryError::query_error_with_source(
                        format!("Failed to add properties for {}", name),
                        e,
                    )
                })?;
            } else if let Some(remove) = &props.remove {
                if !remove.is_empty() {
                    let fields = remove
                        .iter()
                        .map(|k| format!("n.`{}`", k))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let qstr = format!("MATCH (n {{name: $name}}) REMOVE {}", fields);
                    let query = Query::new(qstr).param("name", name.to_string());
                    self.graph.run(query).await.map_err(|e| {
                        MemoryError::query_error_with_source(
                            format!("Failed to remove properties for {}", name),
                            e,
                        )
                    })?;
                }
            } else if let Some(set_map) = &props.set {
                // remove existing props except name and observations
                let mut map: HashMap<String, neo4rs::BoltType> = HashMap::new();
                for (k, v) in set_map {
                    map.insert(k.clone(), memory_value_to_bolt(v)?);
                }
                let query = Query::new(
                    "MATCH (n {name: $name}) WITH n, keys(n) AS k UNWIND [x IN k WHERE x <> 'name' AND x <> 'observations'] AS key REMOVE n[key] WITH n SET n += $props".to_string(),
                )
                .param("name", name.to_string())
                .param("props", map);
                self.graph.run(query).await.map_err(|e| {
                    MemoryError::query_error_with_source(
                        format!("Failed to set properties for {}", name),
                        e,
                    )
                })?;
            }
        }

        if let Some(labels) = &update.labels {
            if let Some(add) = &labels.add {
                if !add.is_empty() {
                    let label_str = add.iter().map(|l| format!(":`{}`", l)).collect::<String>();
                    let query_str = format!("MATCH (n {{name: $name}}) SET n{}", label_str);
                    let query = Query::new(query_str).param("name", name.to_string());
                    self.graph.run(query).await.map_err(|e| {
                        MemoryError::query_error_with_source(
                            format!("Failed to add labels for {}", name),
                            e,
                        )
                    })?;
                }
            } else if let Some(remove) = &labels.remove {
                if !remove.is_empty() {
                    let label_str = remove
                        .iter()
                        .map(|l| format!(":`{}`", l))
                        .collect::<String>();
                    let query_str = format!("MATCH (n {{name: $name}}) REMOVE n{}", label_str);
                    let query = Query::new(query_str).param("name", name.to_string());
                    self.graph.run(query).await.map_err(|e| {
                        MemoryError::query_error_with_source(
                            format!("Failed to remove labels for {}", name),
                            e,
                        )
                    })?;
                }
            }
        }

        Ok(())
    }

    async fn update_relationship(
        &self,
        from: &str,
        to: &str,
        name: &str,
        update: &RelationshipUpdate,
    ) -> MemoryResult<(), Self::Error> {
        if let Some(props) = &update.properties {
            if let Some(add) = &props.add {
                let mut map: HashMap<String, neo4rs::BoltType> = HashMap::new();
                for (k, v) in add {
                    map.insert(k.clone(), memory_value_to_bolt(v)?);
                }
                let query = Query::new(
                    "MATCH (a {name: $from})-[r:`".to_owned()
                        + name
                        + "`]->(b {name: $to}) SET r += $props",
                )
                .param("from", from.to_string())
                .param("to", to.to_string())
                .param("props", map);
                self.graph.run(query).await.map_err(|e| {
                    MemoryError::query_error_with_source(
                        "Failed to add relationship properties".to_string(),
                        e,
                    )
                })?;
            } else if let Some(remove) = &props.remove {
                if !remove.is_empty() {
                    let fields = remove
                        .iter()
                        .map(|k| format!("r.`{}`", k))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let qstr = format!(
                        "MATCH (a {{name: $from}})-[r:`{}`]->(b {{name: $to}}) REMOVE {}",
                        name, fields
                    );
                    let query = Query::new(qstr)
                        .param("from", from.to_string())
                        .param("to", to.to_string());
                    self.graph.run(query).await.map_err(|e| {
                        MemoryError::query_error_with_source(
                            "Failed to remove relationship properties".to_string(),
                            e,
                        )
                    })?;
                }
            } else if let Some(set_map) = &props.set {
                let mut map: HashMap<String, neo4rs::BoltType> = HashMap::new();
                for (k, v) in set_map {
                    map.insert(k.clone(), memory_value_to_bolt(v)?);
                }
                let qstr = format!(
                    "MATCH (a {{name: $from}})-[r:`{}`]->(b {{name: $to}}) SET r = $props",
                    name
                );
                let query = Query::new(qstr)
                    .param("from", from.to_string())
                    .param("to", to.to_string())
                    .param("props", map);
                self.graph.run(query).await.map_err(|e| {
                    MemoryError::query_error_with_source(
                        "Failed to set relationship properties".to_string(),
                        e,
                    )
                })?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Neo4jConfig;

    #[test]
    fn debug_redacts_password() {
        let cfg = Neo4jConfig {
            uri: "neo4j://localhost:7687".to_string(),
            username: "user".to_string(),
            password: "secret".to_string(),
        };

        let dbg = format!("{cfg:?}");
        assert!(!dbg.contains("secret"));
        assert!(dbg.contains("***"));
    }
}
