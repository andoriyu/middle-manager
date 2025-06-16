#![allow(unused_imports)]
pub mod add_observations;
pub mod create_entity;
pub mod error;
pub mod get_entity;
pub mod remove_all_observations;
pub mod remove_observations;
pub mod set_observations;

use rust_mcp_sdk::tool_box;

pub use add_observations::AddObservationsTool;
pub use create_entity::CreateEntityTool;
pub use get_entity::GetEntityTool;
pub use remove_all_observations::RemoveAllObservationsTool;
pub use remove_observations::RemoveObservationsTool;
pub use set_observations::SetObservationsTool;

// Generate an enum with all tools
tool_box!(
    MemoryTools,
    [
        CreateEntityTool,
        GetEntityTool,
        SetObservationsTool,
        AddObservationsTool,
        RemoveAllObservationsTool,
        RemoveObservationsTool
    ]
);
