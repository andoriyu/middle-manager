#![warn(clippy::all)]
use clap::{Parser, ValueEnum};
use std::fs::{File, OpenOptions};
use std::io;
use std::path::PathBuf;
use tracing::{Level, instrument};
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::{EnvFilter, Registry, fmt, prelude::*};

use mm_server as mm_server_lib;

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
#[instrument(fields(path = ?path, rotate))]
fn create_file_writer(path: &PathBuf, rotate: bool) -> io::Result<File> {
    if rotate {
        // Create or truncate the file
        File::create(path)
    } else {
        // Open for appending
        OpenOptions::new().create(true).append(true).open(path)
    }
}

#[instrument(skip(args))]
async fn run(args: Args) -> anyhow::Result<()> {
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
    mm_server_lib::run_server(&config_paths).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    run(args).await
}
