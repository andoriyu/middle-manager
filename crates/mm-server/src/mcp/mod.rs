#[macro_use]
mod macros;
pub mod create_entities;
pub mod create_relationships;
pub mod create_tasks;
pub mod delete_entities;
pub mod delete_relationships;
pub mod delete_task;
pub mod error;
pub mod find_entities_by_labels;
pub mod find_relationships;
pub mod get_entity;
pub mod get_git_status;
pub mod get_project_context;
pub mod get_task;
pub mod list_projects;
#[cfg(test)]
pub mod tests;
pub mod update_entity;
pub mod update_relationship;
pub mod update_task;

use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::tool_box;
use serde_json::{Map, Value};

pub use create_entities::CreateEntitiesTool;
pub use create_relationships::CreateRelationshipsTool;
pub use create_tasks::CreateTasksTool;
pub use delete_entities::DeleteEntitiesTool;
pub use delete_relationships::DeleteRelationshipsTool;
pub use delete_task::DeleteTaskTool;
pub use find_entities_by_labels::FindEntitiesByLabelsTool;
pub use find_relationships::FindRelationshipsTool;
pub use get_entity::GetEntityTool;
pub use get_git_status::GetGitStatusTool;
pub use get_project_context::GetProjectContextTool;
pub use get_task::GetTaskTool;
pub use list_projects::ListProjectsTool;
pub use update_entity::UpdateEntityTool;
pub use update_relationship::UpdateRelationshipTool;
pub use update_task::UpdateTaskTool;

// Generate an enum with all tools
tool_box!(
    MMTools,
    [
        CreateEntitiesTool,
        CreateRelationshipsTool,
        DeleteEntitiesTool,
        DeleteRelationshipsTool,
        FindEntitiesByLabelsTool,
        FindRelationshipsTool,
        CreateTasksTool,
        GetTaskTool,
        UpdateTaskTool,
        DeleteTaskTool,
        GetEntityTool,
        GetGitStatusTool,
        GetProjectContextTool,
        ListProjectsTool,
        UpdateEntityTool,
        UpdateRelationshipTool
    ]
);

impl MMTools {
    /// Execute the contained tool using the provided ports.
    pub async fn execute<M, G>(
        self,
        ports: &mm_core::Ports<M, G>,
    ) -> Result<
        rust_mcp_sdk::schema::CallToolResult,
        rust_mcp_sdk::schema::schema_utils::CallToolError,
    >
    where
        M: mm_memory::MemoryRepository + Send + Sync,
        G: mm_git::GitRepository + Send + Sync,
        M::Error: std::error::Error + Send + Sync + 'static,
        G::Error: std::error::Error + Send + Sync + 'static,
    {
        match self {
            MMTools::CreateEntitiesTool(tool) => tool.call_tool(ports).await,
            MMTools::CreateRelationshipsTool(tool) => tool.call_tool(ports).await,
            MMTools::DeleteEntitiesTool(tool) => tool.call_tool(ports).await,
            MMTools::DeleteRelationshipsTool(tool) => tool.call_tool(ports).await,
            MMTools::FindEntitiesByLabelsTool(tool) => tool.call_tool(ports).await,
            MMTools::FindRelationshipsTool(tool) => tool.call_tool(ports).await,
            MMTools::CreateTasksTool(tool) => tool.call_tool(ports).await,
            MMTools::GetTaskTool(tool) => tool.call_tool(ports).await,
            MMTools::UpdateTaskTool(tool) => tool.call_tool(ports).await,
            MMTools::DeleteTaskTool(tool) => tool.call_tool(ports).await,
            MMTools::GetEntityTool(tool) => tool.call_tool(ports).await,
            MMTools::GetGitStatusTool(tool) => tool.call_tool(ports).await,
            MMTools::GetProjectContextTool(tool) => tool.call_tool(ports).await,
            MMTools::ListProjectsTool(tool) => tool.call_tool(ports).await,
            MMTools::UpdateEntityTool(tool) => tool.call_tool(ports).await,
            MMTools::UpdateRelationshipTool(tool) => tool.call_tool(ports).await,
        }
    }

    /// Return the JSON schema for the contained tool.
    pub fn schema(&self) -> Map<String, Value> {
        match self {
            MMTools::CreateEntitiesTool(_) => CreateEntitiesTool::json_schema(),
            MMTools::CreateRelationshipsTool(_) => CreateRelationshipsTool::json_schema(),
            MMTools::DeleteEntitiesTool(_) => DeleteEntitiesTool::json_schema(),
            MMTools::DeleteRelationshipsTool(_) => DeleteRelationshipsTool::json_schema(),
            MMTools::FindEntitiesByLabelsTool(_) => FindEntitiesByLabelsTool::json_schema(),
            MMTools::FindRelationshipsTool(_) => FindRelationshipsTool::json_schema(),
            MMTools::CreateTasksTool(_) => CreateTasksTool::json_schema(),
            MMTools::GetTaskTool(_) => GetTaskTool::json_schema(),
            MMTools::UpdateTaskTool(_) => UpdateTaskTool::json_schema(),
            MMTools::DeleteTaskTool(_) => DeleteTaskTool::json_schema(),
            MMTools::GetEntityTool(_) => GetEntityTool::json_schema(),
            MMTools::GetGitStatusTool(_) => GetGitStatusTool::json_schema(),
            MMTools::GetProjectContextTool(_) => GetProjectContextTool::json_schema(),
            MMTools::ListProjectsTool(_) => ListProjectsTool::json_schema(),
            MMTools::UpdateEntityTool(_) => UpdateEntityTool::json_schema(),
            MMTools::UpdateRelationshipTool(_) => UpdateRelationshipTool::json_schema(),
        }
    }
}
