use mm_core::operations::git::{GetGitStatusCommand, get_git_status};
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::macros::mcp_tool;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// MCP tool for retrieving Git repository status
#[mcp_tool(
    name = "get_git_status",
    description = "Get the status of a Git repository"
)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetGitStatusTool {
    /// Path to the Git repository
    pub path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetGitStatusResponse {
    /// Current branch name
    pub branch: String,
    /// Upstream branch if configured
    pub upstream_branch: Option<String>,
    /// Whether the working tree has uncommitted changes
    pub is_dirty: bool,
    /// Commits ahead of the upstream branch
    pub ahead_by: u32,
    /// Commits behind the upstream branch
    pub behind_by: u32,
    /// Paths of files that have been staged
    pub staged_files: Vec<String>,
    /// Paths of tracked files modified but not staged
    pub modified_files: Vec<String>,
    /// Paths of untracked files
    pub untracked_files: Vec<String>,
    /// Files with merge conflicts
    pub conflicted_files: Vec<String>,
    /// Number of stashes in the repository
    pub stash_count: u32,
}

impl GetGitStatusTool {
    generate_call_tool!(self, GetGitStatusCommand { path }, get_git_status);
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::Ports;
    use mm_git::{GitStatus, repository::MockGitRepository};
    use mm_memory::MockMemoryRepository;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_call_tool_success() {
        // Create a mock git repository with expectations
        let mut git_repo = MockGitRepository::new();
        git_repo.expect_get_status().returning(|_| {
            Ok(GitStatus {
                branch: "main".to_string(),
                upstream_branch: Some("origin/main".to_string()),
                is_dirty: false,
                ahead_by: 0,
                behind_by: 0,
                staged_files: vec![],
                modified_files: vec![],
                untracked_files: vec![],
                conflicted_files: vec![],
                stash_count: 0,
            })
        });

        // Create a git service with the configured mock
        let git_service = Arc::new(mm_git::GitService::new(git_repo));

        // Create a Ports instance with the git service
        let memory_repo = MockMemoryRepository::new();
        let memory_service = Arc::new(mm_memory::MemoryService::new(
            memory_repo,
            mm_memory::MemoryConfig::default(),
        ));
        let ports = Ports::new(memory_service, git_service);

        // Call the tool
        let tool = GetGitStatusTool {
            path: PathBuf::from("/fake/path"),
        };
        let result = tool.call_tool(&ports).await.unwrap();

        // Assert the result contains the branch name
        let text = result.content[0].as_text_content().unwrap().text.clone();
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        let branch = json.get("branch").unwrap().as_str().unwrap();
        assert_eq!(branch, "main");
        assert_eq!(
            json.get("upstream_branch").unwrap().as_str().unwrap(),
            "origin/main"
        );
        assert!(!json.get("is_dirty").unwrap().as_bool().unwrap());
        assert_eq!(json.get("ahead_by").unwrap().as_u64().unwrap(), 0);
        assert_eq!(json.get("behind_by").unwrap().as_u64().unwrap(), 0);
        for key in [
            "staged_files",
            "modified_files",
            "untracked_files",
            "conflicted_files",
        ] {
            assert!(json.get(key).unwrap().as_array().unwrap().is_empty());
        }
        assert_eq!(json.get("stash_count").unwrap().as_u64().unwrap(), 0);
    }

    #[tokio::test]
    async fn test_call_tool_error() {
        // Create a mock git repository with expectations
        let mut git_repo = MockGitRepository::new();
        git_repo
            .expect_get_status()
            .returning(|_| Err(mm_git::GitError::repository_error("Repository not found")));

        // Create a git service with the configured mock
        let git_service = Arc::new(mm_git::GitService::new(git_repo));

        // Create a Ports instance with the git service
        let memory_repo = MockMemoryRepository::new();
        let memory_service = Arc::new(mm_memory::MemoryService::new(
            memory_repo,
            mm_memory::MemoryConfig::default(),
        ));
        let ports = Ports::new(memory_service, git_service);

        // Call the tool
        let tool = GetGitStatusTool {
            path: PathBuf::from("/fake/path"),
        };
        let result = tool.call_tool(&ports).await;

        // Assert the result
        assert!(result.is_err());
    }
}
