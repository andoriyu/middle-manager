macro_rules! generate_call_tool {
    (@value $self_field:expr $(, $value:expr)? ) => {
        generate_call_tool!(@inner $self_field $(, $value)? )
    };
    (@inner $self_field:expr, $value:expr) => { $value };
    (@inner $self_field:expr) => { $self_field.clone() };

    // Custom success block version that provides both the command and the result
    ($self_ident:ident, $command:ident { $( $field:ident $(=> $value:expr)? ),* $(,)? }, $operation:path, |$cmd_ident:ident, $res_ident:ident| $success_block:block) => {
        pub async fn call_tool<R>(&$self_ident, ports: &mm_core::Ports<R>) -> Result<rust_mcp_sdk::schema::CallToolResult, rust_mcp_sdk::schema::schema_utils::CallToolError>
        where
            R: mm_memory::MemoryRepository + Send + Sync,
            R::Error: std::error::Error + Send + Sync + 'static,
        {
            let $cmd_ident = $command {
                $(
                    $field: generate_call_tool!(@value $self_ident.$field $(, $value)? ),
                )*
            };

            let $res_ident = $crate::mcp::error::map_result($operation(ports, $cmd_ident.clone()).await)?;
            $success_block
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
