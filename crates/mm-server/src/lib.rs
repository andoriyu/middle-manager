//! MCP server adapter for the Middle Manager project.
//!
//! This crate is an outer adapter in the hexagonal architecture. It
//! exposes the memory graph operations defined in `mm-core` through the
//! Model Context Protocol (MCP) so external clients can interact with
//! the system. The server orchestrates requests, translating protocol
//! calls into core domain operations.
#![warn(clippy::all)]
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result as AnyResult;

use mm_core::Ports;
use mm_git::{GitRepository, GitService};
use mm_git_git2::create_git_service;
use mm_memory::{MemoryRepository, MemoryService};
use mm_memory_neo4j::{create_neo4j_service, neo4rs};

mod config;
pub use config::Config;

use rust_mcp_sdk::schema::{
    ListToolsResult, Result as McpResult, RpcError, schema_utils::CallToolError,
};
use rust_mcp_sdk::{
    McpServer, StdioTransport, TransportOptions,
    mcp_server::{ServerHandler, server_runtime},
    schema::{
        Implementation, InitializeResult, LATEST_PROTOCOL_VERSION, ServerCapabilities,
        ServerCapabilitiesResources, ServerCapabilitiesTools,
    },
};
use tracing::{debug, error};

pub mod mcp;
use mcp::MMTools;
mod resources;
mod roots;

use clap::Subcommand;
use mm_utils::IntoJsonSchema;
use rust_mcp_sdk::schema::{ListResourceTemplatesResult, ListResourcesResult};
use serde_json::Value;

/// Subcommands for interacting with tools via the CLI
#[derive(Subcommand, Debug, Clone)]
pub enum ToolsCommand {
    /// List available tools
    List,
    /// Call a tool with JSON input
    Call {
        /// Name of the tool to call
        tool_name: String,
        /// JSON input for the tool
        tool_input: String,
    },
    /// Print the JSON schema for a tool
    Schema {
        /// Name of the toolbox (e.g., "MMTools")
        toolbox: String,
        /// Name of the tool
        tool_name: String,
    },
}

/// Middle Manager MCP server handler
pub struct MiddleManagerHandler<M, G>
where
    M: MemoryRepository + Send + Sync + 'static,
    G: GitRepository + Send + Sync + 'static,
    M::Error: From<neo4rs::Error> + Send + Sync + 'static,
{
    ports: Arc<Ports<M, G>>,
}

impl<M, G> MiddleManagerHandler<M, G>
where
    M: MemoryRepository + Send + Sync + 'static,
    G: GitRepository + Send + Sync + 'static,
    M::Error: From<neo4rs::Error> + Send + Sync + 'static,
{
    /// Create a new Middle Manager MCP server handler
    pub fn new(memory_service: MemoryService<M>, git_service: GitService<G>) -> Self {
        let ports = Arc::new(Ports::new(Arc::new(memory_service), Arc::new(git_service)));
        Self { ports }
    }

    /// Request the client's roots and store them if supported.
    async fn update_client_roots(&self, runtime: &dyn McpServer) {
        if runtime.client_supports_root_list().unwrap_or(false) {
            match runtime.list_roots(None).await {
                Ok(result) => {
                    let roots = result
                        .roots
                        .into_iter()
                        .map(roots::from_sdk_root)
                        .collect::<Vec<_>>();
                    let mut collection = self.ports.roots.write().await;
                    collection.set_roots(roots);
                }
                Err(err) => {
                    error!("Failed to list client roots and update the roots collection: {err}");
                }
            }
        }
    }

    async fn handle_list_tools_request(
        &self,
        _request: rust_mcp_sdk::schema::ListToolsRequest,
        _runtime: &dyn McpServer,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        debug!("Handling list tools request");
        Ok(ListToolsResult {
            meta: None,
            next_cursor: None,
            tools: MMTools::tools(),
        })
    }

    async fn handle_list_resources_request(
        &self,
        _request: rust_mcp_sdk::schema::ListResourcesRequest,
        _runtime: &dyn McpServer,
    ) -> std::result::Result<ListResourcesResult, RpcError> {
        debug!("Handling list resources request");
        Ok(resources::list_resources())
    }

    async fn handle_list_resource_templates_request(
        &self,
        _request: rust_mcp_sdk::schema::ListResourceTemplatesRequest,
        _runtime: &dyn McpServer,
    ) -> std::result::Result<ListResourceTemplatesResult, RpcError> {
        debug!("Handling list resource templates request");
        Ok(resources::list_resource_templates())
    }

    async fn handle_ping_request(
        &self,
        _request: rust_mcp_sdk::schema::PingRequest,
        _runtime: &dyn McpServer,
    ) -> std::result::Result<McpResult, RpcError> {
        debug!("Handling ping request");
        Ok(McpResult::default())
    }

    async fn handle_read_resource_request(
        &self,
        request: rust_mcp_sdk::schema::ReadResourceRequest,
        _runtime: &dyn McpServer,
    ) -> std::result::Result<rust_mcp_sdk::schema::ReadResourceResult, RpcError> {
        debug!("Handling read resource request: {}", request.params.uri);
        let result = resources::read_resource(&self.ports, &request.params.uri)
            .await
            .map_err(|err| RpcError::internal_error().with_message(err.to_string()))?;
        Ok(result)
    }

    async fn handle_call_tool_request(
        &self,
        request: rust_mcp_sdk::schema::CallToolRequest,
        _runtime: &dyn McpServer,
    ) -> std::result::Result<rust_mcp_sdk::schema::CallToolResult, CallToolError> {
        let tool_name = request.tool_name().to_string();
        debug!("Handling call tool request: {}", tool_name);

        // Attempt to convert request parameters into MMTools enum
        let tool_params = MMTools::try_from(request.params)
            .map_err(|_| CallToolError::unknown_tool(tool_name.clone()))?;

        tool_params.execute(&self.ports).await
    }
}

/// Create a Middle Manager MCP server handler with the given memory service
pub fn create_handler<M, G>(
    memory_service: MemoryService<M>,
    git_service: GitService<G>,
) -> MiddleManagerHandler<M, G>
where
    M: MemoryRepository + Send + Sync + 'static,
    G: GitRepository + Send + Sync + 'static,
    M::Error: From<neo4rs::Error> + Send + Sync + 'static,
{
    MiddleManagerHandler::new(memory_service, git_service)
}

#[async_trait]
impl<M, G> ServerHandler for MiddleManagerHandler<M, G>
where
    M: MemoryRepository + Send + Sync + 'static,
    G: GitRepository + Send + Sync + 'static,
    M::Error: From<neo4rs::Error> + Send + Sync + 'static,
{
    async fn on_initialized(&self, runtime: &dyn McpServer) {
        self.update_client_roots(runtime).await;
    }

    async fn handle_list_tools_request(
        &self,
        request: rust_mcp_sdk::schema::ListToolsRequest,
        runtime: &dyn McpServer,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        MiddleManagerHandler::handle_list_tools_request(self, request, runtime).await
    }

    async fn handle_list_resources_request(
        &self,
        request: rust_mcp_sdk::schema::ListResourcesRequest,
        runtime: &dyn McpServer,
    ) -> std::result::Result<ListResourcesResult, RpcError> {
        MiddleManagerHandler::handle_list_resources_request(self, request, runtime).await
    }

    async fn handle_list_resource_templates_request(
        &self,
        request: rust_mcp_sdk::schema::ListResourceTemplatesRequest,
        runtime: &dyn McpServer,
    ) -> std::result::Result<ListResourceTemplatesResult, RpcError> {
        MiddleManagerHandler::handle_list_resource_templates_request(self, request, runtime).await
    }

    async fn handle_ping_request(
        &self,
        request: rust_mcp_sdk::schema::PingRequest,
        runtime: &dyn McpServer,
    ) -> std::result::Result<McpResult, RpcError> {
        MiddleManagerHandler::handle_ping_request(self, request, runtime).await
    }

    async fn handle_read_resource_request(
        &self,
        request: rust_mcp_sdk::schema::ReadResourceRequest,
        runtime: &dyn McpServer,
    ) -> std::result::Result<rust_mcp_sdk::schema::ReadResourceResult, RpcError> {
        MiddleManagerHandler::handle_read_resource_request(self, request, runtime).await
    }

    async fn handle_call_tool_request(
        &self,
        request: rust_mcp_sdk::schema::CallToolRequest,
        runtime: &dyn McpServer,
    ) -> std::result::Result<rust_mcp_sdk::schema::CallToolResult, CallToolError> {
        MiddleManagerHandler::handle_call_tool_request(self, request, runtime).await
    }
}

/// Run the Middle Manager MCP server
#[tracing::instrument(skip(config_paths), fields(paths = config_paths.len()))]
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

    // Create git service
    let git_service = create_git_service();

    // Create server handler
    let handler = create_handler(memory_service, git_service);

    // Create server details
    let server_details = InitializeResult {
        server_info: Implementation {
            name: "Middle Manager MCP Server".to_string(),
            version: "0.1.0".to_string(),
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            resources: Some(ServerCapabilitiesResources::default()),
            ..ServerCapabilities::default()
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
    let server = server_runtime::create_server(server_details, transport, handler);
    tracing::info!("Server initialized, starting...");
    server
        .start()
        .await
        .map_err(|e| anyhow::anyhow!("Server failed to start or run: {}", e))
}

/// Execute tool-related commands from the CLI
#[tracing::instrument(skip(config_paths), fields(paths = config_paths.len()))]
pub async fn run_tools<P: AsRef<Path>>(command: ToolsCommand, config_paths: &[P]) -> AnyResult<()> {
    // Load configuration
    let config = Config::load(config_paths)?;
    let memory_service = create_neo4j_service(config.neo4j, config.memory).await?;
    let git_service = create_git_service();
    let ports = Ports::new(Arc::new(memory_service), Arc::new(git_service));

    match command {
        ToolsCommand::List => {
            println!("MMTools:");
            for tool in MMTools::tools() {
                let desc = tool.description.unwrap_or_default();
                println!("  {} - {}", tool.name, desc);
            }
        }
        ToolsCommand::Call {
            tool_name,
            tool_input,
        } => match tool_name.as_str() {
            "tools/list" | "list_tools" => {
                let result = ListToolsResult {
                    meta: None,
                    next_cursor: None,
                    tools: MMTools::tools(),
                };
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            "resources/list" | "list_resources" => {
                let result: ListResourcesResult = resources::list_resources();
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            "resource_templates/list" | "list_resource_templates" => {
                let result: ListResourceTemplatesResult = resources::list_resource_templates();
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            _ => {
                let value: Value = serde_json::from_str(&tool_input)?;
                let map = value
                    .as_object()
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("tool input must be an object"))?;
                let params = rust_mcp_sdk::schema::CallToolRequestParams {
                    name: tool_name.clone(),
                    arguments: Some(map),
                };
                let tool =
                    MMTools::try_from(params).map_err(|e| anyhow::anyhow!(format!("{e:?}")))?;
                let result = tool
                    .execute(&ports)
                    .await
                    .map_err(|e| anyhow::anyhow!(format!("{e:?}")))?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        },
        ToolsCommand::Schema { toolbox, tool_name } => {
            if toolbox != "MMTools" {
                anyhow::bail!("Unknown toolbox: {}", toolbox);
            }
            let schema = match tool_name.as_str() {
                "create_entities" => mcp::CreateEntitiesTool::json_schema(),
                "create_relationships" => mcp::CreateRelationshipsTool::json_schema(),
                "delete_entities" => mcp::DeleteEntitiesTool::json_schema(),
                "delete_relationships" => mcp::DeleteRelationshipsTool::json_schema(),
                "find_entities_by_labels" => mcp::FindEntitiesByLabelsTool::json_schema(),
                "find_relationships" => mcp::FindRelationshipsTool::json_schema(),
                "create_tasks" => mcp::CreateTasksTool::json_schema(),
                "get_task" => mcp::GetTaskTool::json_schema(),
                "update_task" => mcp::UpdateTaskTool::json_schema(),
                "delete_task" => mcp::DeleteTaskTool::json_schema(),
                "get_entity" => mcp::GetEntityTool::json_schema(),
                "get_git_status" => mcp::GetGitStatusTool::json_schema(),
                "get_project_context" => mcp::GetProjectContextTool::json_schema(),
                "list_projects" => mcp::ListProjectsTool::json_schema(),
                "update_entity" => mcp::UpdateEntityTool::json_schema(),
                "update_relationship" => mcp::UpdateRelationshipTool::json_schema(),
                _ => anyhow::bail!("Unknown tool: {}", tool_name),
            };
            println!("{}", serde_json::to_string_pretty(&schema)?);
        }
    }

    Ok(())
}
