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
/// * success block `|$cmd_ident, $res_ident| { ... }` - code executed after the
///   operation, expected to return `Result<CallToolResult, CallToolError>`.
///
/// ### Example
/// ```rust
/// impl ExampleTool {
///     generate_call_tool!(
///         self,
///         ExampleCommand { field1, field2 => transform(&self.field2) },
///         example_operation,
///         |command, result| { Ok(CallToolResult::text_content("ok", None)) }
///     );
/// }
/// ```
macro_rules! generate_call_tool {
    (@value $self_field:expr $(, $value:expr)? ) => {
        generate_call_tool!(@inner $self_field $(, $value)? )
    };
    (@inner $self_field:expr, $value:expr) => { $value };
    (@inner $self_field:expr) => { $self_field.clone() };

    // Custom success block version that provides both the command and the result
    ($self_ident:ident, $command:ident { $( $field:ident $(=> $value:expr)? ),* $(,)? }, $operation:path, |$cmd_ident:ident, $res_ident:ident| $success_block:block) => {
        pub async fn call_tool<R, G>(&$self_ident, ports: &mm_core::Ports<R, G>) -> Result<rust_mcp_sdk::schema::CallToolResult, rust_mcp_sdk::schema::schema_utils::CallToolError>
        where
            R: mm_memory::MemoryRepository + Send + Sync,
            R::Error: std::error::Error + Send + Sync + 'static,
            G: mm_git::GitServiceTrait + Send + Sync,
        {
            use tracing::Instrument;

            let $cmd_ident = $command {
                $(
                    $field: generate_call_tool!(@value $self_ident.$field $(, $value)? ),
                )*
            };

            let span = tracing::info_span!("call_tool");
            async move {
                let $res_ident = $crate::mcp::error::map_result($operation(ports, $cmd_ident.clone()).await)?;
                $success_block
            }
            .instrument(span)
            .await
        }
    };

    // Backwards-compatible custom success block that only receives the command
    ($self_ident:ident, $command:ident { $( $field:ident $(=> $value:expr)? ),* $(,)? }, $operation:path, |$cmd_ident:ident| $success_block:block) => {
        generate_call_tool!(
            $self_ident,
            $command { $( $field $(=> $value)? ),* },
            $operation,
            |$cmd_ident, _result| $success_block
        );
    };

    // Default success message version
    ($self_ident:ident, $command:ident { $( $field:ident $(=> $value:expr)? ),* $(,)? }, $success_msg:expr, $operation:path) => {
        generate_call_tool!(
            $self_ident,
            $command { $( $field $(=> $value)? ),* },
            $operation,
            |cmd, _result| {
                let _ = cmd;
                Ok(rust_mcp_sdk::schema::CallToolResult::text_content(
                    format!($success_msg, $self_ident.name),
                    None,
                ))
            }
        );
    };
}
