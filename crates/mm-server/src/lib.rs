use std::sync::Arc;
use async_trait::async_trait;

use mm_core::{
    Config, CreateEntityTool, GetEntityTool, MemoryService, MemoryServiceImpl,
    Neo4jRepository, create_neo4j_service, neo4rs
};
use rust_mcp_sdk::{
    mcp_server::{server_runtime, ServerHandler, ServerRuntime},
    schema::{
        schema_utils::CallToolError,
        CallToolRequest, CallToolResult, ListToolsRequest, ListToolsResult,
        Implementation, InitializeResult, ServerCapabilities, ServerCapabilitiesTools,
        RpcError, LATEST_PROTOCOL_VERSION,
    },
    StdioTransport, TransportOptions, McpServer,
};
use std::path::Path;
use tracing::{debug, info};

/// Middle Manager MCP server handler
pub struct MiddleManagerHandler<S>
where
    S: MemoryService<neo4rs::Error> + Send + Sync + 'static,
{
    service: Arc<S>,
}

impl<S> MiddleManagerHandler<S>
where
    S: MemoryService<neo4rs::Error> + Send + Sync + 'static,
{
    /// Create a new Middle Manager MCP server handler
    pub fn new(service: S) -> Self {
        Self {
            service: Arc::new(service),
        }
    }
}

#[async_trait]
impl ServerHandler for MiddleManagerHandler<MemoryServiceImpl<Neo4jRepository, neo4rs::Error>> {
    async fn handle_list_tools_request(
        &self,
        _request: ListToolsRequest,
        _runtime: &dyn McpServer,
    ) -> Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            tools: vec![
                CreateEntityTool::tool(),
                GetEntityTool::tool(),
            ],
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        request: CallToolRequest,
        _runtime: &dyn McpServer,
    ) -> Result<CallToolResult, CallToolError> {
        debug!("Handling tool request: {}", request.tool_name());
        
        match request.tool_name() {
            name if name == CreateEntityTool::tool_name() => {
                let params: CreateEntityTool = serde_json::from_value(serde_json::to_value(request.params).unwrap())
                    .map_err(|e| CallToolError::new(e))?;
                
                match params.execute(self.service.clone()).await {
                    Ok(response) => {
                        let json = serde_json::to_value(response)
                            .map_err(|e| CallToolError::new(e))?;
                        Ok(CallToolResult::text_content(json.to_string(), None))
                    },
                    Err(e) => Err(CallToolError::new(e)),
                }
            },
            name if name == GetEntityTool::tool_name() => {
                let params: GetEntityTool = serde_json::from_value(serde_json::to_value(request.params).unwrap())
                    .map_err(|e| CallToolError::new(e))?;
                
                match params.execute(self.service.clone()).await {
                    Ok(response) => {
                        let json = serde_json::to_value(response)
                            .map_err(|e| CallToolError::new(e))?;
                        Ok(CallToolResult::text_content(json.to_string(), None))
                    },
                    Err(e) => Err(CallToolError::new(e)),
                }
            },
            _ => Err(CallToolError::unknown_tool(request.tool_name().to_string())),
        }
    }
}

/// Run the Middle Manager MCP server
pub async fn run_server<P: AsRef<Path>>(config_paths: &[P]) -> anyhow::Result<()> {
    // Load configuration
    let config = Config::load(config_paths)
        .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;
    
    info!("Starting Middle Manager MCP server");
    debug!("Using Neo4j URI: {}", config.neo4j.uri);
    
    // Create memory service
    let neo4j_config = config.neo4j.into();
    let memory_service = create_neo4j_service(neo4j_config).await
        .map_err(|e| anyhow::anyhow!("Failed to create Neo4j memory service: {}", e))?;
    
    // Create server handler
    let handler = MiddleManagerHandler::new(memory_service);
    
    // Create server details
    let server_details = InitializeResult {
        server_info: Implementation {
            name: "Middle Manager MCP Server".to_string(),
            version: "0.1.0".to_string(),
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        meta: None,
        instructions: Some("Middle Manager MCP Server provides tools for interacting with the memory graph.".to_string()),
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
    };

    // Create transport
    let transport = StdioTransport::new(TransportOptions::default())
        .map_err(|e| anyhow::anyhow!("Failed to create stdio transport: {}", e))?;
    
    // Create and start server
    let server: ServerRuntime = server_runtime::create_server(server_details, transport, handler);
    info!("Server initialized, starting...");
    server.start().await
        .map_err(|e| anyhow::anyhow!("Server failed to start or run: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::MockMemoryService;
    use std::collections::HashMap;
    use serde_json::Value;
    use rust_mcp_sdk::error::SdkResult;

    #[tokio::test]
    async fn test_list_tools() {
        let mock = MockMemoryService::<neo4rs::Error>::new();
        let handler = MiddleManagerHandler::new(mock);
        
        let result = handler.handle_list_tools_request(
            ListToolsRequest { cursor: None, meta: None },
            &MockServer {},
        ).await;
        
        assert!(result.is_ok());
        let tools = result.unwrap().tools;
        assert_eq!(tools.len(), 2);
        
        let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
        assert!(tool_names.contains(&"create_entity".to_string()));
        assert!(tool_names.contains(&"get_entity".to_string()));
    }
    
    struct MockServer;
    
    impl McpServer for MockServer {
        fn send_notification(&self, _method: &str, _params: Value) -> SdkResult<()> {
            Ok(())
        }
    }
}
