use mm_core::{ListProjectsCommand, list_projects};
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

/// MCP tool for listing available projects
#[mcp_tool(name = "list_projects", description = "List all available projects")]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListProjectsTool {
    /// Optional name filter to narrow down results
    pub name_filter: Option<String>,
}

impl ListProjectsTool {
    generate_call_tool!(
        self,
        ListProjectsCommand {
            name_filter => self.name_filter.clone()
        },
        list_projects,
        |_command, result| {
            serde_json::to_value(result.projects)
                .map(|json| {
                    rust_mcp_sdk::schema::CallToolResult::text_content(json.to_string(), None)
                })
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
    use mm_memory::{MemoryConfig, MemoryEntity, MemoryService, MockMemoryRepository};
    use mockall::predicate::*;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_call_tool_success() {
        let project1 = MemoryEntity {
            name: "andoriyu:project:middle_manager".to_string(),
            labels: vec!["Memory".to_string(), "Project".to_string()],
            observations: vec!["A project for managing memory".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        let project2 = MemoryEntity {
            name: "andoriyu:project:flakes".to_string(),
            labels: vec!["Memory".to_string(), "Project".to_string()],
            observations: vec!["A project for managing Nix flakes".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entities_by_labels()
            .with(
                eq(vec!["Project".to_string()]),
                eq(mm_memory::LabelMatchMode::All),
                always(),
            )
            .returning(move |_, _, _| Ok(vec![project1.clone(), project2.clone()]));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = ListProjectsTool { name_filter: None };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        let value: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value.as_array().unwrap().len(), 2);
        assert_eq!(value[0]["name"], "andoriyu:project:middle_manager");
        assert_eq!(value[1]["name"], "andoriyu:project:flakes");
    }

    #[tokio::test]
    async fn test_call_tool_with_filter() {
        let project1 = MemoryEntity {
            name: "andoriyu:project:middle_manager".to_string(),
            labels: vec!["Memory".to_string(), "Project".to_string()],
            observations: vec!["A project for managing memory".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        let project2 = MemoryEntity {
            name: "andoriyu:project:flakes".to_string(),
            labels: vec!["Memory".to_string(), "Project".to_string()],
            observations: vec!["A project for managing Nix flakes".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        let mut mock = MockMemoryRepository::new();
        mock.expect_find_entities_by_labels()
            .with(
                eq(vec!["Project".to_string()]),
                eq(mm_memory::LabelMatchMode::All),
                always(),
            )
            .returning(move |_, _, _| Ok(vec![project1.clone(), project2.clone()]));

        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::new(Arc::new(service));

        let tool = ListProjectsTool {
            name_filter: Some("flakes".to_string()),
        };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        let value: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value.as_array().unwrap().len(), 1);
        assert_eq!(value[0]["name"], "andoriyu:project:flakes");
    }
}
