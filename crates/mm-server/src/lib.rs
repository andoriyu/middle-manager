use std::sync::Arc;
use std::error::Error as StdError;
use async_trait::async_trait;

use mm_core::{
    Ports, MemoryService, neo4rs,
};
use rust_mcp_sdk::schema::{
    schema_utils::{CallToolError, NotificationFromClient, RequestFromClient, ResultFromServer},
    ClientRequest, ListToolsResult, RpcError,
};
use rust_mcp_sdk::{
    mcp_server::{enforce_compatible_protocol_version, ServerHandlerCore},
    McpServer,
};
use tracing::{debug};

mod mcp;
use mcp::MemoryTools;

/// Middle Manager MCP server handler
pub struct MiddleManagerHandler<S, E>
where
    S: MemoryService<E> + Send + Sync + 'static,
    E: StdError + Send + Sync + 'static,
{
    service: Arc<S>,
    _phantom: std::marker::PhantomData<E>,
}

impl<S, E> MiddleManagerHandler<S, E>
where
    S: MemoryService<E> + Send + Sync + 'static,
    E: StdError + Send + Sync + 'static,
{
    /// Create a new Middle Manager MCP server handler
    pub fn new(service: S) -> Self {
        Self {
            service: Arc::new(service),
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Create a Middle Manager MCP server handler with the given memory service
pub fn create_handler<S, E>(memory_service: S) -> MiddleManagerHandler<S, E>
where
    S: MemoryService<E> + Send + Sync + 'static,
    E: StdError + Send + Sync + 'static,
{
    MiddleManagerHandler::new(memory_service)
}

#[async_trait]
impl<S> ServerHandlerCore for MiddleManagerHandler<S, neo4rs::Error>
where
    S: MemoryService<neo4rs::Error> + Send + Sync + 'static,
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
                    .map_err(|err| RpcError::internal_error().with_message(err.to_string()))? {
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
                },

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
                    };
                    Ok(result.into())
                }

                _ => Err(RpcError::method_not_found()
                    .with_message(format!("No handler is implemented for '{}'.", client_request.method()))),
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

