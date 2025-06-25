use rust_mcp_sdk::schema::{CallToolResult, schema_utils::CallToolError};
use serde::Serialize;
use crate::mcp::error::ToolError;

/// Convert a serializable value to a text content result
pub fn to_json_result<T: Serialize>(value: T) -> Result<CallToolResult, CallToolError> {
    serde_json::to_value(value)
        .map(|json| CallToolResult::text_content(json.to_string(), None))
        .map_err(|e| CallToolError::new(ToolError::from(e)))
}

/// Handle an optional result, returning the value as JSON if present,
/// or a not found message if None
pub fn optional_result<T: Serialize>(
    result: Option<T>, 
    entity_type: &str,
    name: &str
) -> Result<CallToolResult, CallToolError> {
    match result {
        Some(value) => to_json_result(value),
        None => Ok(CallToolResult::text_content(
            format!("{} '{}' not found", entity_type, name),
            None,
        )),
    }
}

/// Default result handler that intelligently processes different result types
pub fn default_result_handler<T: Serialize>(
    result: T,
    entity_type: Option<&str>,
    name: Option<&str>
) -> Result<CallToolResult, CallToolError> {
    // This is a placeholder - the actual implementation would be more complex
    // to handle different types of results
    to_json_result(result)
}
