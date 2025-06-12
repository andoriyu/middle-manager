use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use mm_server::run_server;

/// Middle Manager CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Log level
    #[arg(short, long, value_enum, default_value_t = LogLevel::Info)]
    log_level: LogLevel,

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    // Initialize tracing
    let level: Level = args.log_level.into();
    let filter = EnvFilter::from_default_env()
        .add_directive(level.into());
    
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();
    
    info!("Starting Middle Manager CLI");
    
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
