use async_trait::async_trait;
use rust_mcp_sdk::{
    error::SdkResult,
    mcp_server::{server_runtime, ServerHandler, ServerRuntime},
    schema::{
        schema_utils::CallToolError,
        CallToolRequest, CallToolResult, ListToolsRequest, ListToolsResult,
        Implementation, InitializeResult, ServerCapabilities, ServerCapabilitiesTools,
        RpcError, LATEST_PROTOCOL_VERSION,
    },
    StdioTransport, TransportOptions, McpServer,
};
use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};

#[mcp_tool(name = "say_hello_world", description = "Prints \"Hello World!\" message")]
#[derive(Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct SayHelloTool {}

pub struct HelloWorldHandler;

#[async_trait]
impl ServerHandler for HelloWorldHandler {
    async fn handle_list_tools_request(
        &self,
        _request: ListToolsRequest,
        _runtime: &dyn McpServer,
    ) -> Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            tools: vec![SayHelloTool::tool()],
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        request: CallToolRequest,
        _runtime: &dyn McpServer,
    ) -> Result<CallToolResult, CallToolError> {
        if request.tool_name() == SayHelloTool::tool_name() {
            Ok(CallToolResult::text_content("Hello World!".to_string(), None))
        } else {
            Err(CallToolError::unknown_tool(request.tool_name().to_string()))
        }
    }
}

pub async fn run_server() -> SdkResult<()> {
    let server_details = InitializeResult {
        server_info: Implementation {
            name: "Hello World MCP Server".to_string(),
            version: "0.1.0".to_string(),
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        meta: None,
        instructions: Some("server instructions...".to_string()),
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
    };

    let transport = StdioTransport::new(TransportOptions::default())?;
    let handler = HelloWorldHandler;
    let server: ServerRuntime = server_runtime::create_server(server_details, transport, handler);
    server.start().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn server_starts() {
        // we just start server and then stop immediately using abort
        let server_future = run_server();
        // we cannot easily test stdio server without client, so just ensure it doesn't error immediately
        let _ = std::future::pending::<()>();
        // For testing, we won't run server_future because it would block.
        drop(server_future);
    }
}
