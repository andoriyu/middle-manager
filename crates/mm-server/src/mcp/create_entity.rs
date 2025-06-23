use mm_core::{CreateEntityCommand, MemoryEntity, create_entity};
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "create_entity",
    description = "Create a new entity in the memory graph"
)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateEntityTool {
    /// Entities to create
    pub entities: Vec<MemoryEntity>,
}

impl CreateEntityTool {
    generate_call_tool!(
        self,
        CreateEntityCommand {
            entities => self.entities.clone()
        },
        create_entity,
        |command, _result| {
            serde_json::to_value(command.entities)
                .map(|json| rust_mcp_sdk::schema::CallToolResult::text_content(json.to_string(), None))
                .map_err(|e| {
                    rust_mcp_sdk::schema::schema_utils::CallToolError::new(
                        crate::mcp::error::ToolError::from(e),
                    )
                })
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
        mock.expect_create_entities()
            .withf(|ents| ents.len() == 1 && ents[0].name == "test:entity")
            .returning(|_| Ok(()));

        let service = MemoryService::new(
            mock,
            MemoryConfig {
                default_labels: false,
                ..MemoryConfig::default()
            },
        );
        let ports = Ports::new(Arc::new(service));

        let tool = CreateEntityTool {
            entities: vec![MemoryEntity {
                name: "test:entity".to_string(),
                labels: vec!["Test".to_string()],
                ..Default::default()
            }],
        };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        let expected = serde_json::json!([
            {
                "name": "test:entity",
                "labels": ["Test"],
                "observations": [],
                "properties": {},
                "relationships": []
            }
        ]);
        assert_eq!(text, expected.to_string());
    }

    #[tokio::test]
    async fn test_call_tool_repository_error() {
        let mut mock = MockMemoryRepository::new();
        mock.expect_create_entities()
            .returning(|_| Err(MemoryError::runtime_error("fail")));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = CreateEntityTool {
            entities: vec![MemoryEntity {
                name: "test:entity".to_string(),
                labels: vec!["Test".to_string()],
                ..Default::default()
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
    use serde_json::Value;

    #[test]
    fn test_schema_has_no_refs() {
        // Generate the schema for CreateEntityTool
        let schema = CreateEntityTool::json_schema();

        // Convert to a Value for easier inspection
        let schema_value =
            serde_json::to_value(&schema).expect("Failed to convert schema to Value");

        // Convert to a string to check for $defs
        let schema_str =
            serde_json::to_string(&schema_value).expect("Failed to convert schema to string");

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
