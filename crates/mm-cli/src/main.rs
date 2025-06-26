#![warn(clippy::all)]
use clap::{Parser, Subcommand, ValueEnum};
use std::fs::{File, OpenOptions};
use std::io;
use std::path::PathBuf;
use tracing::{Level, instrument};
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::{EnvFilter, Registry, fmt, prelude::*};

use mm_cli::{format_task_detail, format_tasks_table};
use mm_server as mm_server_lib;
use mm_server_lib::mcp::{GetTaskTool, ListTasksTool};
use mm_server_lib::{ToolsCommand, create_ports_from_config};

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
        required_if_eq_any([
            ("log_level", "error"),
            ("log_level", "warn"),
            ("log_level", "info"),
            ("log_level", "debug"),
            ("log_level", "trace"),
        ])
    )]
    logfile: Option<PathBuf>,

    /// Rotate logs (clear log file if it exists)
    #[arg(short = 'r', long, default_value_t = true)]
    rotate_logs: bool,

    /// Path to config file (can be specified multiple times)
    #[arg(short, long, value_name = "FILE", required = true, action = clap::ArgAction::Append)]
    config: Vec<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
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

#[derive(Subcommand, Debug)]
enum Command {
    /// Start the MCP server
    Server,
    /// Interact with tools
    Tools(ToolsSubcommand),
    /// Configuration related commands
    Config(ConfigSubcommand),
    /// Task management commands
    Tasks(TasksSubcommand),
}

#[derive(Parser, Debug)]
struct ToolsSubcommand {
    #[command(subcommand)]
    command: ToolsSubcommandType,
}

#[derive(Parser, Debug)]
struct ConfigSubcommand {
    #[command(subcommand)]
    command: ConfigSubcommandType,
}

#[derive(Subcommand, Debug)]
enum ConfigSubcommandType {
    /// Validate configuration
    Validate {
        /// Show loaded configuration if valid
        #[arg(long)]
        show: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ToolsSubcommandType {
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
        /// Name of the tool
        tool_name: String,
    },
}

#[derive(Parser, Debug)]
struct TasksSubcommand {
    #[command(subcommand)]
    command: TasksSubcommandType,
}

#[derive(Subcommand, Debug)]
enum TasksSubcommandType {
    /// List tasks for a project
    List {
        /// Project name to list tasks for
        #[arg(long)]
        project: Option<String>,
        /// Labels to filter by
        #[arg(long, value_delimiter = ',', num_args = 1..)]
        labels: Vec<String>,
        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },
    /// View a single task
    View {
        /// Name of the task to view
        name: String,
        /// Output result in JSON format
        #[arg(long)]
        json: bool,
    },
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

#[instrument(skip(paths))]
async fn run_config_validate<P: AsRef<std::path::Path>>(
    paths: &[P],
    show: bool,
) -> anyhow::Result<()> {
    match create_ports_from_config(paths).await {
        Ok((config, _)) => {
            if show {
                println!("{}", serde_json::to_string_pretty(&config)?);
            } else {
                println!("Ok");
            }
            Ok(())
        }
        Err(err) => {
            println!("{:#?}", err);
            Err(err)
        }
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

    // Use the specified config paths directly
    let config_paths: Vec<PathBuf> = args.config;

    match args.command.unwrap_or(Command::Server) {
        Command::Server => mm_server_lib::run_server(&config_paths).await?,
        Command::Tools(tools_subcommand) => {
            match tools_subcommand.command {
                ToolsSubcommandType::List => {
                    mm_server_lib::run_tools(ToolsCommand::List, &config_paths).await?
                }
                ToolsSubcommandType::Call {
                    tool_name,
                    tool_input,
                } => {
                    mm_server_lib::run_tools(
                        ToolsCommand::Call {
                            tool_name,
                            tool_input,
                        },
                        &config_paths,
                    )
                    .await?
                }
                ToolsSubcommandType::Schema { tool_name } => {
                    // Always use "MMTools" as the toolbox name (hardcoded)
                    mm_server_lib::run_tools(
                        ToolsCommand::Schema {
                            toolbox: "MMTools".to_string(),
                            tool_name,
                        },
                        &config_paths,
                    )
                    .await?
                }
            }
        }
        Command::Config(config_subcommand) => match config_subcommand.command {
            ConfigSubcommandType::Validate { show } => {
                run_config_validate(&config_paths, show).await?;
            }
        },
        Command::Tasks(tasks_subcommand) => {
            let (_, ports) = create_ports_from_config(&config_paths).await?;
            match tasks_subcommand.command {
                TasksSubcommandType::List {
                    project,
                    labels,
                    json,
                } => {
                    let tool = ListTasksTool {
                        project_name: project,
                        labels,
                    };
                    let result = tool
                        .call_tool(&ports)
                        .await
                        .map_err(|e| anyhow::anyhow!(format!("{e:?}")))?;
                    let value = mm_cli::parse_json_result(&result)?;
                    let tasks = value["tasks"].as_array().cloned().unwrap_or_default();
                    if json {
                        println!("{}", serde_json::to_string_pretty(&tasks)?);
                    } else if tasks.is_empty() {
                        println!("No tasks found");
                    } else {
                        print!("{}", format_tasks_table(&tasks));
                    }
                }
                TasksSubcommandType::View { name, json } => {
                    let tool = GetTaskTool {
                        task_name: name,
                        project_name: None,
                    };
                    let result = tool
                        .call_tool(&ports)
                        .await
                        .map_err(|e| anyhow::anyhow!(format!("{e:?}")))?;
                    let task = mm_cli::parse_json_result(&result)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&task)?);
                    } else {
                        print!("{}", format_task_detail(&task));
                    }
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    run(args).await
}
