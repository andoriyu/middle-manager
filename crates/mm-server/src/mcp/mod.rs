#[macro_use]
mod macros;
pub mod add_observations; // deprecated
pub mod create_entities;
pub mod create_relationships;
pub mod delete_entities;
pub mod delete_relationships;
pub mod error;
pub mod find_entities_by_labels;
pub mod find_relationships;
pub mod get_entity;
pub mod get_project_context;
pub mod list_projects;
pub mod remove_all_observations;
pub mod remove_observations;
pub mod set_observations;
pub mod update_entity;
pub mod update_relationship;

use rust_mcp_sdk::tool_box;

pub use add_observations::AddObservationsTool;
pub use create_entities::CreateEntitiesTool;
pub use create_relationships::CreateRelationshipsTool;
pub use delete_entities::DeleteEntitiesTool;
pub use delete_relationships::DeleteRelationshipsTool;
pub use find_entities_by_labels::FindEntitiesByLabelsTool;
pub use find_relationships::FindRelationshipsTool;
pub use get_entity::GetEntityTool;
pub use get_project_context::GetProjectContextTool;
pub use list_projects::ListProjectsTool;
pub use remove_all_observations::RemoveAllObservationsTool; // deprecated
pub use remove_observations::RemoveObservationsTool; // deprecated
pub use set_observations::SetObservationsTool; // deprecated
pub use update_entity::UpdateEntityTool;
pub use update_relationship::UpdateRelationshipTool;

// Generate an enum with all tools
tool_box!(
    MemoryTools,
    [
        CreateEntitiesTool,
        CreateRelationshipsTool,
        DeleteEntitiesTool,
        DeleteRelationshipsTool,
        FindEntitiesByLabelsTool,
        FindRelationshipsTool,
        GetEntityTool,
        // Deprecated observation tools below
        SetObservationsTool,
        AddObservationsTool,
        RemoveAllObservationsTool,
        RemoveObservationsTool,
        GetProjectContextTool,
        ListProjectsTool,
        UpdateEntityTool,
        UpdateRelationshipTool
    ]
);
