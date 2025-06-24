use mm_core::{GetGitStatusCommand, get_git_status};
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};

/// MCP tool for retrieving git repository status
#[mcp_tool(name = "git_status", description = "Get status of a Git repository")]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GitStatusTool {
    /// Path to the repository (optional, defaults to current directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl GitStatusTool {
    generate_call_tool!(
        self,
        GetGitStatusCommand { path => self.path.clone() },
        get_git_status,
        |_cmd, status| {
            serde_json::to_value(status).map(|json| {
                rust_mcp_sdk::schema::CallToolResult::text_content(json.to_string(), None)
            }).map_err(|e| rust_mcp_sdk::schema::schema_utils::CallToolError::new(crate::mcp::error::ToolError::from(e)))
        }
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::{MemoryService, Ports};
    use mm_git::{GitError, GitRepository, GitService, RepositoryStatus};
    use mm_memory::{MemoryConfig, MockMemoryRepository};
    use std::path::Path;
    use std::sync::Arc;

    struct MockGitRepo;
    impl GitRepository for MockGitRepo {
        fn get_status(&self, _path: &Path) -> Result<RepositoryStatus, GitError> {
            Ok(RepositoryStatus::default())
        }
    }

    #[tokio::test]
    async fn test_call_tool() {
        let mut mock_memory = MockMemoryRepository::new();
        mock_memory.expect_create_entities().returning(|_| Ok(()));
        let memory_service = MemoryService::new(mock_memory, MemoryConfig::default());
        let git_service = GitService::new(MockGitRepo);
        let ports = Ports::new(Arc::new(memory_service), Arc::new(git_service));

        let tool = GitStatusTool { path: None };
        let res = tool.call_tool(&ports).await;
        assert!(res.is_ok());
    }
}
