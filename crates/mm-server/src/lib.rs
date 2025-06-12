use async_trait::async_trait;
use std::error::Error as StdError;
use std::sync::Arc;

use mm_core::{MemoryService, Ports, neo4rs};
use rust_mcp_sdk::schema::{
    ClientRequest, ListToolsResult, RpcError,
    schema_utils::{CallToolError, NotificationFromClient, RequestFromClient, ResultFromServer},
};
use rust_mcp_sdk::{
    McpServer,
    mcp_server::{ServerHandlerCore, enforce_compatible_protocol_version},
};
use tracing::debug;

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

#[cfg(test)]
mod tests {
    use super::*;
    use mm_core::MemoryServiceImpl;
    use rust_mcp_sdk::error::McpSdkError;
    use rust_mcp_sdk::error::SdkResult;
    use rust_mcp_sdk::schema::ClientMessage;
    use rust_mcp_sdk::schema::InitializeRequestParams;
    use rust_mcp_sdk::schema::{
        Implementation, InitializeResult, LATEST_PROTOCOL_VERSION, ServerCapabilities,
        schema_utils::NotificationFromServer,
    };
    use rust_mcp_sdk::transport::McpDispatch;
    use rust_mcp_sdk::transport::MessageDispatcher;
    use serde_json::Value;
    use std::future::Future;
    use std::option::Option;
    use std::pin::Pin;
    use tokio::sync::RwLock;

    struct MockServer;

    #[async_trait]
    impl McpServer for MockServer {
        async fn start(&self) -> SdkResult<()> {
            Ok(())
        }

        async fn send_notification(&self, _notification: NotificationFromServer) -> SdkResult<()> {
            Ok(())
        }

        fn server_info(&self) -> &InitializeResult {
            static SERVER_INFO: std::sync::OnceLock<InitializeResult> = std::sync::OnceLock::new();
            SERVER_INFO.get_or_init(|| InitializeResult {
                server_info: Implementation {
                    name: "Mock Server".to_string(),
                    version: "0.1.0".to_string(),
                },
                capabilities: ServerCapabilities::default(),
                meta: None,
                instructions: None,
                protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
            })
        }

        fn set_client_details(&self, _params: InitializeRequestParams) -> SdkResult<()> {
            Ok(())
        }

        fn client_info(&self) -> Option<InitializeRequestParams> {
            None
        }

        async fn sender(&self) -> &RwLock<Option<MessageDispatcher<ClientMessage>>> {
            static SENDER: std::sync::OnceLock<RwLock<Option<MessageDispatcher<ClientMessage>>>> =
                std::sync::OnceLock::new();
            SENDER.get_or_init(|| RwLock::new(None))
        }

        async fn stderr_message(&self, _message: String) -> SdkResult<()> {
            Ok(())
        }
    }
}
