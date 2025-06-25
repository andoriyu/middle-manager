use crate::adapters::conversions::bolt_to_memory_value;
use mm_memory::{MemoryEntity, MemoryError, MemoryRelationship, MemoryResult, MemoryValue};
use neo4rs::{self, Node};
use std::collections::HashMap;

pub(super) fn extract_observations_from_bolt(
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

pub(super) fn extract_observations_from_node(
    node: &Node,
) -> MemoryResult<Vec<String>, neo4rs::Error> {
    let observations_bolt = node.get::<neo4rs::BoltType>("observations").map_err(|e| {
        MemoryError::runtime_error_with_source(
            "Failed to get observations property from node".to_string(),
            e,
        )
    })?;

    extract_observations_from_bolt(observations_bolt)
}

pub(super) fn parse_relationships_from_bolt(
    bolt: neo4rs::BoltType,
) -> MemoryResult<Vec<MemoryRelationship>, neo4rs::Error> {
    let mut relationships = Vec::new();

    if let neo4rs::BoltType::Null(_) = bolt {
        return Ok(relationships);
    }

    if let neo4rs::BoltType::List(rel_list) = bolt {
        if rel_list.is_empty() {
            return Ok(relationships);
        }

        for rel_item in rel_list {
            if let neo4rs::BoltType::Map(rel_map) = rel_item {
                if rel_map.value.is_empty() {
                    continue;
                }

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

                let mut properties = HashMap::new();
                if let Ok(neo4rs::BoltType::Map(props_map)) = rel_map.get("properties") {
                    for (key, value) in &props_map.value {
                        match bolt_to_memory_value(value.clone()) {
                            Ok(memory_value) => {
                                properties.insert(key.to_string(), memory_value);
                            }
                            Err(e) => {
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

pub(super) fn memory_entity_from_node(
    node: &Node,
    rels_bolt: neo4rs::BoltType,
) -> MemoryResult<MemoryEntity, neo4rs::Error> {
    let entity_name = node.get::<String>("name").map_err(|e| {
        MemoryError::runtime_error_with_source(
            "Failed to get name property from node".to_string(),
            e,
        )
    })?;

    let observations = extract_observations_from_node(node)?;

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

    let relationships = parse_relationships_from_bolt(rels_bolt)?;

    Ok(MemoryEntity {
        name: entity_name,
        labels,
        observations,
        properties,
        relationships,
    })
}
