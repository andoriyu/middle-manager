/// Generate the `call_tool` method for a MCP tool implementation.
///
/// The macro constructs a command from the tool's fields, invokes a core
/// operation, and converts the result into a `CallToolResult`.
///
/// ### Parameters
/// * `$self_ident` - identifier for the tool instance (usually `self`).
/// * `$command` - command type to instantiate.
/// * `{ $( $field $(=> $value )? ),* }` - mapping of the tool fields to the
///   command fields; optionally override a field with an expression using
///   `field => expr`.
/// * `$operation` - path to the async core operation to call.
/// * Optional `$success_msg` - static message to return on success.
///
/// ### Examples
/// ```no_run
/// // Automatically serialize result to JSON
/// impl ExampleTool {
///     generate_call_tool!(
///         self,
///         ExampleCommand { field1, field2 => transform(&self.field2) },
///         example_operation
///     );
/// }
///
/// // Return static success message
/// impl ExampleTool {
///     generate_call_tool!(
///         self,
///         ExampleCommand { field1, field2 => transform(&self.field2) },
///         example_operation,
///         "Operation completed successfully"
///     );
/// }
/// ```
macro_rules! generate_call_tool {
    (@value $self_field:expr $(, $value:expr)? ) => {
        generate_call_tool!(@inner $self_field $(, $value)? )
    };
    (@inner $self_field:expr, $value:expr) => { $value };
    (@inner $self_field:expr) => { $self_field.clone() };

    // Default version that automatically serializes the result
    ($self_ident:ident, $command:ident { $( $field:ident $(=> $value:expr)? ),* $(,)? }, $operation:path) => {
        pub async fn call_tool<M, G>(&$self_ident, ports: &mm_core::Ports<M, G>) -> Result<rust_mcp_sdk::schema::CallToolResult, rust_mcp_sdk::schema::schema_utils::CallToolError>
        where
            M: mm_memory::MemoryRepository + Send + Sync,
            G: mm_git::GitRepository + Send + Sync,
            M::Error: std::error::Error + Send + Sync + 'static,
            G::Error: std::error::Error + Send + Sync + 'static,
        {
            use tracing::Instrument;

            let command = $command {
                $(
                    $field: generate_call_tool!(@value $self_ident.$field $(, $value)? ),
                )*
            };

            let span = tracing::info_span!("call_tool");
            async move {
                // Convert core errors into CallToolError using anyhow
                let result = $operation(ports, command.clone()).await
                    .map_err(crate::mcp::error::into_call_tool_error)?;

                // Use ? again for JSON serialization errors
                let json = serde_json::to_value(result)
                    .map_err(crate::mcp::error::into_call_tool_error)?;

                // Return the final result
                Ok(rust_mcp_sdk::schema::CallToolResult::text_content(json.to_string(), None))
            }
            .instrument(span)
            .await
        }
    };

    // Simple message version
    ($self_ident:ident, $command:ident { $( $field:ident $(=> $value:expr)? ),* $(,)? }, $operation:path, $success_msg:expr) => {
        pub async fn call_tool<M, G>(&$self_ident, ports: &mm_core::Ports<M, G>) -> Result<rust_mcp_sdk::schema::CallToolResult, rust_mcp_sdk::schema::schema_utils::CallToolError>
        where
            M: mm_memory::MemoryRepository + Send + Sync,
            G: mm_git::GitRepository + Send + Sync,
            M::Error: std::error::Error + Send + Sync + 'static,
            G::Error: std::error::Error + Send + Sync + 'static,
        {
            use tracing::Instrument;

            let command = $command {
                $(
                    $field: generate_call_tool!(@value $self_ident.$field $(, $value)? ),
                )*
            };

            let span = tracing::info_span!("call_tool");
            async move {
                // Convert core errors into CallToolError using anyhow
                $operation(ports, command).await
                    .map_err(crate::mcp::error::into_call_tool_error)?;

                // Return the success message
                Ok(rust_mcp_sdk::schema::CallToolResult::text_content($success_msg.to_string(), None))
            }
            .instrument(span)
            .await
        }
    };
}
