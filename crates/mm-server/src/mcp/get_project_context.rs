use mm_core::operations::memory::{GetProjectContextCommand, ProjectFilter, get_project_context};
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

/// MCP tool for retrieving project context
#[mcp_tool(
    name = "get_project_context",
    description = "Get comprehensive context information about a project"
)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetProjectContextTool {
    /// Project name to look up (e.g., "andoriyu:project:middle_manager")
    pub project_name: Option<String>,

    /// Repository name to look up (e.g., "andoriyu/middle-manager")
    pub repository_name: Option<String>,
}

impl GetProjectContextTool {
    generate_call_tool!(
        self,
        GetProjectContextCommand {
            filter => match (self.project_name.clone(), self.repository_name.clone()) {
                (Some(name), _) => ProjectFilter::Name(name),
                (None, Some(repo)) => ProjectFilter::Repository(repo),
                (None, None) => {
                    return Err(rust_mcp_sdk::schema::schema_utils::CallToolError(
                        crate::mcp::error::error_with_source(
                            "Either project_name or repository_name must be provided",
                            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Missing required parameter")
                        )
                        .into_boxed_dyn_error()
                    ));
                }
            }
        },
        get_project_context
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::Ports;
    use mm_core::operations::memory::{PROJECT_LABEL, TECHNOLOGY_LABEL};
    use mm_memory::{MemoryConfig, MemoryEntity, MemoryService, MockMemoryRepository};
    use mockall::predicate::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_call_tool_by_name_success() {
        // Create test entities
        let project_entity = MemoryEntity {
            name: "andoriyu:project:middle_manager".to_string(),
            labels: vec!["Memory".to_string(), PROJECT_LABEL.to_string()],
            observations: vec!["A project for managing memory".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        let related_entity = MemoryEntity {
            name: "tech:language:rust".to_string(),
            labels: vec![
                "Memory".to_string(),
                TECHNOLOGY_LABEL.to_string(),
                "Language".to_string(),
            ],
            observations: vec!["A systems programming language".to_string()],
            properties: HashMap::new(),
            relationships: Vec::new(),
        };

        // Setup mock repository
        let mut mock = MockMemoryRepository::new();

        // Clone entities for each closure to avoid moved value errors
        let project_entity_clone1 = project_entity.clone();
        mock.expect_find_entity_by_name()
            .with(eq("andoriyu:project:middle_manager"))
            .returning(move |_| Ok(Some(project_entity_clone1.clone())));

        let project_entity_clone2 = project_entity.clone();
        let related_entity_clone = related_entity.clone();
        mock.expect_find_related_entities()
            .with(
                eq("andoriyu:project:middle_manager"),
                always(),
                always(),
                always(),
            )
            .returning(move |_, _, _, _| {
                Ok(vec![
                    project_entity_clone2.clone(),
                    related_entity_clone.clone(),
                ])
            });

        // Create service and ports
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        // Create and call tool
        let tool = GetProjectContextTool {
            project_name: Some("andoriyu:project:middle_manager".to_string()),
            repository_name: None,
        };

        let result = tool.call_tool(&ports).await.expect("tool should succeed");
        let text = result.content[0].as_text_content().unwrap().text.clone();
        // With our new macro, we're returning the full context object
        assert!(text.contains("context"));
        assert!(text.contains("andoriyu:project:middle_manager"));
    }

    #[tokio::test]
    async fn test_call_tool_missing_parameters() {
        let mock = MockMemoryRepository::new();
        let service = MemoryService::new(mock, MemoryConfig::default());
        let ports = Ports::noop().with(|p| p.memory_service = Arc::new(service));

        let tool = GetProjectContextTool {
            project_name: None,
            repository_name: None,
        };

        let result = tool.call_tool(&ports).await;
        assert!(result.is_err());
    }
}
