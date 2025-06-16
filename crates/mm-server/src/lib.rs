use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result as AnyResult;

use mm_core::{MemoryService, Ports};
use mm_memory::MemoryRepository;
use mm_memory_neo4j::{create_neo4j_service, neo4rs};

mod config;
pub use config::Config;

use rust_mcp_sdk::schema::{
    ClientRequest, ListToolsResult, RpcError,
    schema_utils::{CallToolError, NotificationFromClient, RequestFromClient, ResultFromServer},
};
use rust_mcp_sdk::{
    McpServer, StdioTransport, TransportOptions,
    mcp_server::{ServerHandlerCore, enforce_compatible_protocol_version, server_runtime_core},
    schema::{
        Implementation, InitializeResult, LATEST_PROTOCOL_VERSION, ServerCapabilities,
        ServerCapabilitiesTools,
    },
};
use tracing::debug;

mod mcp;
use mcp::MemoryTools;

/// Middle Manager MCP server handler
pub struct MiddleManagerHandler<R>
where
    R: MemoryRepository<Error = neo4rs::Error> + Send + Sync + 'static,
{
    service: Arc<MemoryService<R>>,
}

impl<R> MiddleManagerHandler<R>
where
    R: MemoryRepository<Error = neo4rs::Error> + Send + Sync + 'static,
{
    /// Create a new Middle Manager MCP server handler
    pub fn new(service: MemoryService<R>) -> Self {
        Self {
            service: Arc::new(service),
        }
    }
}

/// Create a Middle Manager MCP server handler with the given memory service
pub fn create_handler<R>(memory_service: MemoryService<R>) -> MiddleManagerHandler<R>
where
    R: MemoryRepository<Error = neo4rs::Error> + Send + Sync + 'static,
{
    MiddleManagerHandler::new(memory_service)
}

#[async_trait]
impl<R> ServerHandlerCore for MiddleManagerHandler<R>
where
    R: MemoryRepository<Error = neo4rs::Error> + Send + Sync + 'static,
{
    async fn handle_request(
        &self,
        request: RequestFromClient,
        runtime: &dyn McpServer,
    ) -> std::result::Result<ResultFromServer, RpcError> {
        match request {
            RequestFromClient::ClientRequest(client_request) => match client_request {
                ClientRequest::InitializeRequest(initialize_request) => {
                    let mut server_info = runtime.server_info().to_owned();

                    if let Some(_updated_protocol_version) = enforce_compatible_protocol_version(
                        &initialize_request.params.protocol_version,
                        &server_info.protocol_version,
                    )
                    .map_err(|err| RpcError::internal_error().with_message(err.to_string()))?
                    {
                        server_info.protocol_version = initialize_request.params.protocol_version;
                    }

                    return Ok(server_info.into());
                }

                ClientRequest::ListToolsRequest(_) => {
                    debug!("Handling list tools request");
                    Ok(ListToolsResult {
                        meta: None,
                        next_cursor: None,
                        tools: MemoryTools::tools(),
                    }
                    .into())
                }

                ClientRequest::CallToolRequest(request) => {
                    let tool_name = request.tool_name().to_string();
                    debug!("Handling call tool request: {}", tool_name);

                    // Create ports with the memory service
                    let ports = Ports::new(self.service.clone());

                    // Attempt to convert request parameters into MemoryTools enum
                    let tool_params = MemoryTools::try_from(request.params)
                        .map_err(|_| CallToolError::unknown_tool(tool_name.clone()))?;

                    // Match the tool variant and execute its corresponding logic
                    let result = match tool_params {
                        MemoryTools::CreateEntityTool(create_entity_tool) => {
                            create_entity_tool.call_tool(&ports).await.map_err(|err| {
                                RpcError::internal_error().with_message(err.to_string())
                            })?
                        }
                        MemoryTools::GetEntityTool(get_entity_tool) => {
                            get_entity_tool.call_tool(&ports).await.map_err(|err| {
                                RpcError::internal_error().with_message(err.to_string())
                            })?
                        }
                        MemoryTools::SetObservationsTool(tool) => {
                            tool.call_tool(&ports).await.map_err(|err| {
                                RpcError::internal_error().with_message(err.to_string())
                            })?
                        }
                        MemoryTools::AddObservationsTool(tool) => {
                            tool.call_tool(&ports).await.map_err(|err| {
                                RpcError::internal_error().with_message(err.to_string())
                            })?
                        }
                        MemoryTools::RemoveAllObservationsTool(tool) => {
                            tool.call_tool(&ports).await.map_err(|err| {
                                RpcError::internal_error().with_message(err.to_string())
                            })?
                        }
                        MemoryTools::RemoveObservationsTool(tool) => {
                            tool.call_tool(&ports).await.map_err(|err| {
                                RpcError::internal_error().with_message(err.to_string())
                            })?
                        }
                    };
                    Ok(result.into())
                }

                _ => Err(RpcError::method_not_found().with_message(format!(
                    "No handler is implemented for '{}'.",
                    client_request.method()
                ))),
            },
            RequestFromClient::CustomRequest(_) => Err(RpcError::method_not_found()
                .with_message("No handler is implemented for custom requests.".to_string())),
        }
    }

    async fn handle_notification(
        &self,
        _notification: NotificationFromClient,
        _: &dyn McpServer,
    ) -> std::result::Result<(), RpcError> {
        Ok(())
    }

    async fn handle_error(
        &self,
        _error: RpcError,
        _: &dyn McpServer,
    ) -> std::result::Result<(), RpcError> {
        Ok(())
    }
}

/// Run the Middle Manager MCP server
pub async fn run_server<P: AsRef<Path>>(config_paths: &[P]) -> AnyResult<()> {
    // Load configuration
    let config = Config::load(config_paths)
        .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;

    tracing::info!("Starting Middle Manager MCP server");
    tracing::debug!("Using Neo4j URI: {}", config.neo4j.uri);

    // Create memory service
    let memory_service = create_neo4j_service(config.neo4j, config.memory)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create Neo4j memory service: {}", e))?;

    // Create server handler
    let handler = create_handler(memory_service);

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
        instructions: Some(
            "Middle Manager MCP Server provides tools for interacting with the memory graph."
                .to_string(),
        ),
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
    };

    // Create transport
    let transport = StdioTransport::new(TransportOptions::default())
        .map_err(|e| anyhow::anyhow!("Failed to create stdio transport: {}", e))?;

    // Create and start server
    let server = server_runtime_core::create_server(server_details, transport, handler);
    tracing::info!("Server initialized, starting...");
    server
        .start()
        .await
        .map_err(|e| anyhow::anyhow!("Server failed to start or run: {}", e))
}
