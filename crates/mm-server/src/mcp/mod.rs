pub mod create_entity;
pub mod error;
pub mod get_entity;

use rust_mcp_sdk::tool_box;

pub use create_entity::CreateEntityTool;
pub use get_entity::GetEntityTool;

// Generate an enum with all tools
tool_box!(MemoryTools, [CreateEntityTool, GetEntityTool]);
