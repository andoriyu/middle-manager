use mm_core::operations::memory::{CreateRelationshipsCommand, create_relationships};
use mm_memory::MemoryRelationship;
use mm_memory::MemoryValue;
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RelationshipInput {
    pub from: String,
    pub to: String,
    pub name: String,
    #[serde(default)]
    pub properties: Option<HashMap<String, MemoryValue>>,
}

impl RelationshipInput {
    fn to_memory_relationship(&self) -> MemoryRelationship {
        MemoryRelationship {
            from: self.from.clone(),
            to: self.to.clone(),
            name: self.name.clone(),
            properties: self.properties.clone().unwrap_or_default(),
        }
    }
}

#[mcp_tool(
    name = "create_relationships",
    description = "Create relationships between entities"
)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateRelationshipsTool {
    pub relationships: Vec<RelationshipInput>,
}

impl CreateRelationshipsTool {
    generate_call_tool!(
        self,
        CreateRelationshipsCommand {
            relationships => self
                .relationships
                .iter()
                .map(RelationshipInput::to_memory_relationship)
                .collect(),
        },
        create_relationships,
        |_cmd, _res| {
            Ok(rust_mcp_sdk::schema::CallToolResult::text_content(
                "Relationships created".to_string(),
                None,
            ))
        }
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::Ports;
    use mm_memory::{MemoryConfig, MemoryError, MemoryService, MockMemoryRepository};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_call_tool_success() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationships()
            .withf(|rels| rels.len() == 1 && rels[0].name == "relates_to")
            .returning(|_| Ok(()));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = CreateRelationshipsTool {
            relationships: vec![RelationshipInput {
                from: "a".to_string(),
                to: "b".to_string(),
                name: "relates_to".to_string(),
                properties: Some(HashMap::default()),
            }],
        };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        assert_eq!(text, "Relationships created");
    }

    #[tokio::test]
    async fn test_call_tool_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_relationships()
            .returning(|_| Err(MemoryError::runtime_error("fail")));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = CreateRelationshipsTool {
            relationships: vec![RelationshipInput {
                from: "a".to_string(),
                to: "b".to_string(),
                name: "relates_to".to_string(),
                properties: Some(HashMap::default()),
            }],
        };

        let result = tool.call_tool(&ports).await;
        assert!(result.is_err());
    }
}
#[cfg(test)]
mod schema_tests {
    use super::*;
    use mm_utils::IntoJsonSchema;

    #[test]
    fn test_schema_has_no_refs() {
        // Generate the schema for CreateRelationshipTool
        let schema = CreateRelationshipsTool::json_schema();

        // Convert to a string to check for $defs
        let schema_str =
            serde_json::to_string(&schema).expect("Failed to convert schema to string");

        // Verify that the schema doesn't contain $defs
        assert!(
            !schema_str.contains("\"$defs\""),
            "Schema should not contain $defs section"
        );

        // Verify that the schema doesn't contain any $ref that points to $defs
        assert!(
            !schema_str.contains("\"$ref\":\"#/$defs/"),
            "Schema should not contain references to $defs"
        );
    }
}
