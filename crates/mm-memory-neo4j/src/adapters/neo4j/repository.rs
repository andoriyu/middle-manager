use std::collections::HashMap;

use async_trait::async_trait;
use neo4rs::{self, Graph, Node, Query};
use tracing::instrument;

use super::config::Neo4jConfig;
use super::helpers::memory_entity_from_node;
use crate::adapters::conversions::{bolt_to_memory_value, memory_value_to_bolt};
use mm_memory::{
    EntityUpdate, LabelMatchMode, MemoryEntity, MemoryError, MemoryRelationship, MemoryRepository,
    MemoryResult, PropertiesUpdate, RelationshipDirection, RelationshipUpdate, ValidationError,
    ValidationErrorKind, relationship::RelationshipRef,
};

pub struct Neo4jRepository {
    graph: Graph,
}

impl Neo4jRepository {
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

    #[instrument(skip(self, params, update))]
    async fn apply_property_update(
        &self,
        match_clause: &str,
        identifier: &str,
        params: &[(&str, String)],
        update: &PropertiesUpdate,
        preserve: Option<&[&str]>,
        context: &str,
    ) -> MemoryResult<(), neo4rs::Error> {
        if let Some(add) = &update.add {
            let mut map: HashMap<String, neo4rs::BoltType> = HashMap::new();
            for (k, v) in add {
                map.insert(k.clone(), memory_value_to_bolt(v)?);
            }
            let qstr = format!("{} SET {} += $props", match_clause, identifier);
            let mut query = Query::new(qstr).param("props", map);
            for (k, v) in params {
                query = query.param(k, v.clone());
            }
            self.graph.run(query).await.map_err(|e| {
                MemoryError::query_error_with_source(format!("Failed to add {}", context), e)
            })?;
        } else if let Some(remove) = &update.remove {
            if !remove.is_empty() {
                let fields = remove
                    .iter()
                    .map(|k| format!("{}.`{}`", identifier, k))
                    .collect::<Vec<_>>()
                    .join(", ");
                let qstr = format!("{} REMOVE {}", match_clause, fields);
                let mut query = Query::new(qstr);
                for (k, v) in params {
                    query = query.param(k, v.clone());
                }
                self.graph.run(query).await.map_err(|e| {
                    MemoryError::query_error_with_source(format!("Failed to remove {}", context), e)
                })?;
            }
        } else if let Some(set_map) = &update.set {
            let mut map: HashMap<String, neo4rs::BoltType> = HashMap::new();
            for (k, v) in set_map {
                map.insert(k.clone(), memory_value_to_bolt(v)?);
            }
            let qstr = if let Some(keep) = preserve {
                let cond = keep
                    .iter()
                    .map(|k| format!("x <> '{}'", k))
                    .collect::<Vec<_>>()
                    .join(" AND ");
                format!(
                    "{match_clause} WITH {id}, keys({id}) AS k UNWIND [x IN k WHERE {cond}] AS key REMOVE {id}[key] WITH {id} SET {id} += $props",
                    match_clause = match_clause,
                    id = identifier,
                    cond = cond
                )
            } else {
                format!("{} SET {} = $props", match_clause, identifier)
            };
            let mut query = Query::new(qstr).param("props", map);
            for (k, v) in params {
                query = query.param(k, v.clone());
            }
            self.graph.run(query).await.map_err(|e| {
                MemoryError::query_error_with_source(format!("Failed to set {}", context), e)
            })?;
        }
        Ok(())
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
        if name.is_empty() {
            return Err(ValidationError::from(ValidationErrorKind::EmptyEntityName).into());
        }

        let query = Query::new(
            "MATCH (n {name: $name}) \n \
             OPTIONAL MATCH (n)-[r]-() \n \
             WITH n, collect(CASE WHEN r IS NOT NULL THEN {from: startNode(r).name, to: endNode(r).name, name: type(r), properties: properties(r)} END) as rels\n \
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
            let node = match row.get::<Node>("n") {
                Ok(n) => n,
                Err(e) => {
                    return Err(MemoryError::runtime_error_with_source(
                        format!("Failed to get node from result for entity {}", name),
                        e,
                    ));
                }
            };

            let rels_bolt = row.get::<neo4rs::BoltType>("rels").map_err(|e| {
                MemoryError::runtime_error_with_source(
                    "Failed to decode relationships".to_string(),
                    e,
                )
            })?;

            let entity = memory_entity_from_node(&node, rels_bolt)?;

            Ok(Some(entity))
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
            "MATCH (start {{name: $name}}) MATCH (start){}(n)\n \
             WITH DISTINCT n\n \
             OPTIONAL MATCH (n)-[r]-()\n \
             WITH n, collect(CASE WHEN r IS NOT NULL THEN {{from: startNode(r).name, to: endNode(r).name, name: type(r), properties: properties(r)}} END) as rels\n \
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

            let rels_bolt = row.get::<neo4rs::BoltType>("rels").map_err(|e| {
                MemoryError::runtime_error_with_source(
                    "Failed to decode relationships".to_string(),
                    e,
                )
            })?;

            let entity = memory_entity_from_node(&node, rels_bolt)?;

            entities.push(entity);
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
            "MATCH (n) {where_clause}\n \
             OPTIONAL MATCH (n)-[r]-()\n \
             WITH n, collect(CASE WHEN r IS NOT NULL THEN {{from: startNode(r).name, to: endNode(r).name, name: type(r), properties: properties(r)}} END) as rels\n \
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

            let rels_bolt = row.get::<neo4rs::BoltType>("rels").map_err(|e| {
                MemoryError::runtime_error_with_source(
                    "Failed to decode relationships".to_string(),
                    e,
                )
            })?;

            let entity = memory_entity_from_node(&node, rels_bolt)?;

            entities.push(entity);
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
            let params = [("name", name.to_string())];
            self.apply_property_update(
                "MATCH (n {name: $name})",
                "n",
                &params,
                props,
                Some(&["name", "observations"]),
                &format!("properties for {}", name),
            )
            .await?;
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
            let match_clause = format!(
                "MATCH (a {{name: $from}})-[r:`{}`]->(b {{name: $to}})",
                name
            );
            let params = [("from", from.to_string()), ("to", to.to_string())];
            self.apply_property_update(
                &match_clause,
                "r",
                &params,
                props,
                None,
                "relationship properties",
            )
            .await?;
        }
        Ok(())
    }

    async fn delete_entities(&self, names: &[String]) -> MemoryResult<(), Self::Error> {
        if names.is_empty() {
            return Ok(());
        }
        let query = Query::new("MATCH (n) WHERE n.name IN $names DETACH DELETE n".to_string())
            .param("names", names.to_vec());
        self.graph.run(query).await.map_err(|e| {
            MemoryError::query_error_with_source("Failed to delete entities".to_string(), e)
        })?;
        Ok(())
    }

    async fn delete_relationships(
        &self,
        relationships: &[RelationshipRef],
    ) -> MemoryResult<(), Self::Error> {
        if relationships.is_empty() {
            return Ok(());
        }

        let rows: Vec<HashMap<String, neo4rs::BoltType>> = relationships
            .iter()
            .map(|rel| {
                let mut row = HashMap::new();
                row.insert("from".to_string(), rel.from.clone().into());
                row.insert("to".to_string(), rel.to.clone().into());
                row.insert("name".to_string(), rel.name.clone().into());
                row
            })
            .collect();

        let query = Query::new(
            "UNWIND $rows AS row MATCH (a {name: row.from})-[r]->(b {name: row.to}) \
             WHERE type(r) = row.name DELETE r"
                .to_string(),
        )
        .param("rows", rows);

        self.graph.run(query).await.map_err(|e| {
            MemoryError::query_error_with_source("Failed to delete relationships".to_string(), e)
        })?;
        Ok(())
    }

    async fn find_relationships(
        &self,
        from: Option<String>,
        to: Option<String>,
        name: Option<String>,
    ) -> MemoryResult<Vec<MemoryRelationship>, Self::Error> {
        let mut query_str = String::from("MATCH (a)-[r]->(b)");
        let mut conditions = Vec::new();
        if from.is_some() {
            conditions.push("a.name = $from".to_string());
        }
        if to.is_some() {
            conditions.push("b.name = $to".to_string());
        }
        if name.is_some() {
            conditions.push("type(r) = $type".to_string());
        }
        if !conditions.is_empty() {
            query_str.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
        }
        query_str.push_str(
            " RETURN a.name as from, b.name as to, type(r) as name, properties(r) as props",
        );

        let mut query = Query::new(query_str);
        if let Some(f) = from {
            query = query.param("from", f.to_string());
        }
        if let Some(t) = to {
            query = query.param("to", t.to_string());
        }
        if let Some(n) = name {
            query = query.param("type", n.to_string());
        }

        let mut result = self.graph.execute(query).await.map_err(|e| {
            MemoryError::query_error_with_source("Failed to query relationships".to_string(), e)
        })?;

        let mut rels = Vec::new();
        while let Some(row) = result.next().await.map_err(|e| {
            MemoryError::query_error_with_source("Failed to fetch relationships".to_string(), e)
        })? {
            let from = row.get::<String>("from").map_err(|e| {
                MemoryError::runtime_error_with_source("Failed to get from".to_string(), e)
            })?;
            let to = row.get::<String>("to").map_err(|e| {
                MemoryError::runtime_error_with_source("Failed to get to".to_string(), e)
            })?;
            let name = row.get::<String>("name").map_err(|e| {
                MemoryError::runtime_error_with_source("Failed to get name".to_string(), e)
            })?;
            let props_bolt = row.get::<neo4rs::BoltType>("props").map_err(|e| {
                MemoryError::runtime_error_with_source("Failed to decode props".to_string(), e)
            })?;
            let mut properties = HashMap::new();
            if let neo4rs::BoltType::Map(map) = props_bolt {
                for (k, v) in &map.value {
                    let mv = bolt_to_memory_value(v.clone())?;
                    properties.insert(k.to_string(), mv);
                }
            }
            rels.push(MemoryRelationship {
                from,
                to,
                name,
                properties,
            });
        }
        Ok(rels)
    }
}
