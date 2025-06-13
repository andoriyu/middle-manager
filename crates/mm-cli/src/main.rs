use clap::{Parser, ValueEnum};
use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use tracing::{Level, debug, info};
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::{EnvFilter, Registry, fmt, prelude::*};

use mm_core::{Config, create_neo4j_service, neo4rs};
use rust_mcp_sdk::{
    McpServer, StdioTransport, TransportOptions,
    mcp_server::server_runtime_core,
    schema::{
        Implementation, InitializeResult, LATEST_PROTOCOL_VERSION, ServerCapabilities,
        ServerCapabilitiesTools,
    },
};

/// Middle Manager CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Log level
    #[arg(short, long, value_enum, default_value_t = LogLevel::Info)]
    log_level: LogLevel,

    /// Path to log file (required if log level is specified)
    #[arg(
        short = 'f',
        long,
        value_name = "FILE",
        required_if_eq("log_level", "error"),
        required_if_eq("log_level", "warn"),
        required_if_eq("log_level", "info"),
        required_if_eq("log_level", "debug"),
        required_if_eq("log_level", "trace")
    )]
    logfile: Option<PathBuf>,

    /// Rotate logs (clear log file if it exists)
    #[arg(short = 'r', long, default_value_t = true)]
    rotate_logs: bool,

    /// Path to config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

/// Log level for the application
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, ValueEnum)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => Level::ERROR,
            LogLevel::Warn => Level::WARN,
            LogLevel::Info => Level::INFO,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Trace => Level::TRACE,
        }
    }
}

/// Create a file writer for logging
fn create_file_writer(path: &PathBuf, rotate: bool) -> io::Result<File> {
    if rotate {
        // Create or truncate the file
        File::create(path)
    } else {
        // Open for appending
        OpenOptions::new().create(true).append(true).open(path)
    }
}

/// Run the Middle Manager MCP server
async fn run_server<P: AsRef<Path>>(config_paths: &[P]) -> anyhow::Result<()> {
    // Load configuration
    let config = Config::load(config_paths)
        .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;

    info!("Starting Middle Manager MCP server");
    debug!("Using Neo4j URI: {}", config.neo4j.uri);

    // Create memory service
    let neo4j_config = config.neo4j.into();
    let memory_service = create_neo4j_service(neo4j_config, config.memory.into())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create Neo4j memory service: {}", e))?;

    // Create server handler
    let handler = mm_server::create_handler::<_, neo4rs::Error>(memory_service);

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
    info!("Server initialized, starting...");
    server
        .start()
        .await
        .map_err(|e| anyhow::anyhow!("Server failed to start or run: {}", e))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let level: Level = args.log_level.into();
    let filter = EnvFilter::from_default_env().add_directive(level.into());

    // Set up logging
    let subscriber = Registry::default().with(filter);

    if let Some(logfile_path) = &args.logfile {
        // Create log file writer
        let file = create_file_writer(logfile_path, args.rotate_logs)?;

        // Set up file logging only (no console)
        let file_layer = fmt::layer().with_writer(file.with_max_level(level));

        // Register only the file layer
        subscriber.with(file_layer).init();
    } else {
        // No logging output at all if no logfile is specified
        subscriber.init();
    }

    // Determine config paths
    let config_paths: Vec<PathBuf> = if let Some(config_path) = args.config {
        vec![config_path]
    } else {
        // Default config paths
        vec![
            PathBuf::from("config/default.toml"),
            PathBuf::from("config/local.toml"),
        ]
    };

    // Run the server
    run_server(&config_paths).await?;

    Ok(())
}
