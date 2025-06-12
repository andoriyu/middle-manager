# Middle Manager

Middle Manager is a Model Context Protocol (MCP) server that provides tools for interacting with a Neo4j memory graph. It uses a hexagonal architecture to separate domain logic from external protocols.

## Project Structure

The project is organized as a Rust workspace with the following crates:

- **mm-cli**: Command-line interface for running the MCP server
- **mm-core**: Core domain logic and operations
- **mm-server**: MCP server implementation
- **mm-memory**: Memory graph repository and service

All workspace crates reside in the `crates/` directory to keep the repository root tidy.

### Code Structure Diagram

```mermaid
graph TD
    %% mm-cli details
    subgraph "mm-cli"
        main_fn["main()"]
        run_server_fn["run_server()"]
        main_fn --> run_server_fn
        run_server_fn --> create_handler
    end

    %% mm-server details
    subgraph "mm-server"
        create_handler["create_handler()"]
        subgraph "mcp"
            create_tool["CreateEntityTool::call_tool"]
            get_tool["GetEntityTool::call_tool"]
        end
        create_handler --> create_tool
        create_handler --> get_tool
    end

    %% mm-core details
    subgraph "mm-core"
        create_op["create_entity"]
        get_op["get_entity"]
        create_op --> memory_service["MemoryService"]
        get_op --> memory_service["MemoryService"]
    end

    %% mm-memory details
    subgraph "mm-memory"
        memory_service["MemoryService"]
        repository_trait["MemoryRepository"]
        neo4j_repo["Neo4jRepository"]
        neo4j_db[(Neo4j)]
        memory_service --> repository_trait
        repository_trait --> neo4j_repo
        neo4j_repo --> neo4j_db
    end

    %% Flow connections
    create_tool --> create_op
    get_tool --> get_op
```

## Features

- **MCP Server**: Implements the Model Context Protocol for AI assistant integration
- **Memory Graph**: Stores and retrieves knowledge from a Neo4j graph database
- **Entity Management**: Create and retrieve entities with labels, observations, and properties
- **Configurable Logging**: Control log level and file output

## Building

Run `cargo build` from the repository root to compile all crates.

### Using Nix

If you have [Nix](https://nixos.org/) installed you can build the CLI package with:

```bash
nix build .#middle_manager
```

The [Determinate Systems installer](https://install.determinate.systems/nix) provides a fast way to install Nix:

```bash
curl -L https://install.determinate.systems/nix | sh -s -- --no-confirm
```

## Running

Execute `cargo run -p mm-cli` to build and run the CLI with default settings.

### CLI Options

```
USAGE:
    mm-cli [OPTIONS]

OPTIONS:
    -l, --log-level <LOG_LEVEL>    Log level [default: info] [possible values: error, warn, info, debug, trace]
    -f, --logfile <FILE>           Path to log file (required if log level is specified)
    -r, --rotate-logs              Rotate logs (clear log file if it exists) [default: true]
    -c, --config <FILE>            Path to config file
    -h, --help                     Print help
    -V, --version                  Print version
```

### Configuration

The server looks for configuration files in the following order:
1. Custom config file specified with `-c` or `--config`
2. `config/default.toml`
3. `config/local.toml` (gitignored for local overrides)

Example configuration:

```toml
[neo4j]
uri = "neo4j://localhost:7687"
username = "neo4j"
password = "password"
```

When using the provided `docker-compose.yml` file, Neo4j is exposed on host
port `7688` rather than the default `7687`. Update `config/local.toml` or set
the environment variable `MM_NEO4J__URI` to `neo4j://localhost:7688` when
running with Docker.

## Development

### Using Just

The project includes a `justfile` with common development tasks:

```bash
# List available commands
just

# Run the MCP inspector with mm-cli
just inspect

# Run with debug logging
just inspect-debug

# Clean Neo4j volumes
just clean-neo4j
```

### Using MCP Inspector

To test the MCP server with the inspector:

```bash
npx @modelcontextprotocol/inspector cargo run -p mm-cli
```

### Using Nix

This repository provides a Nix flake. Enter the development environment with:

```bash
nix develop
```

Build the workspace via:

```bash
nix build
```

Run the flake's checks:

```bash
nix flake check
```

The flake uses [naersk](https://github.com/nix-community/naersk) together
with the [rust overlay](https://github.com/oxalica/rust-overlay) to build the
Rust workspace.

## Architecture

The project follows a hexagonal architecture (ports and adapters) pattern:

- **Core Domain**: Business logic independent of external protocols
- **Ports**: Interfaces for external dependencies
- **Adapters**: Implementations of ports for specific technologies
- **MCP Protocol**: External interface for AI assistants

This architecture ensures that the core domain logic is isolated from external concerns, making it more maintainable and testable.

### Ports & Adapters Diagram

```mermaid
graph TD
    %% Color legend for crates
    classDef mm-core fill:#D5F5E3,stroke:#333;
    classDef mm-memory fill:#D6EAF8,stroke:#333;
    classDef mm-server fill:#FAD7A0,stroke:#333;

    %% Ports defined in mm-core
    subgraph "mm-core"
        core_service_trait["MemoryService trait"]
    end

    %% Ports and adapters in mm-memory
    subgraph "mm-memory"
        repository_trait["MemoryRepository trait"]
        memory_service_impl["MemoryService struct"]
        neo4j_repo["Neo4jRepository"]
    end

    %% Adapter in mm-server
    subgraph "mm-server"
        server_handler["MiddleManagerHandler"]
    end

    %% Relationships with labels
    server_handler --"depends on"--> core_service_trait
    core_service_trait --"implemented in"--> memory_service_impl
    memory_service_impl --"uses"--> repository_trait
    repository_trait --"implemented in"--> neo4j_repo

    %% Apply crate colors
    class core_service_trait mm-core;
    class repository_trait mm-memory;
    class memory_service_impl mm-memory;
    class neo4j_repo mm-memory;
    class server_handler mm-server;

    %% Legend showing colors for crates
    subgraph "Legend"
        legend_core["mm-core"]
        legend_memory["mm-memory"]
        legend_server["mm-server"]
    end
    class legend_core mm-core;
    class legend_memory mm-memory;
    class legend_server mm-server;
```
