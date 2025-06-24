# Middle Manager

Middle Manager is a Model Context Protocol (MCP) server for a Neo4j memory graph. It uses a hexagonal architecture to keep domain logic independent of external protocols.

### Memory

#### Resources

| URI | Description | Example |
| --- | ----------- | ------- |
| `memory://{name}` | Read a memory entity by name | `memory://tech:language:rust` |

The `memory://` scheme is dynamic: any entity name can be requested. The server exposes this through a single template from `list_resource_templates`.

#### Tools

| Name | Purpose |
| ---- | ------- |
| `create_entity` | Create one or more entities |
| `create_relationship` | Create relationships between entities |
| `get_entity` | Retrieve an entity by name |
| `set_observations` | Replace all observations for an entity |
| `add_observations` | Append observations to an entity |
| `remove_observations` | Remove specific observations from an entity |
| `remove_all_observations` | Delete all observations from an entity |
| `git_status` | Retrieve repository status information |

## Project Structure

The project is organized as a Rust workspace with the following crates:

- **mm-cli**: Command-line interface for running the MCP server
- **mm-core**: Core domain operations that depend on the `MemoryService` from `mm-memory`
- **mm-memory**: Memory domain types including the `MemoryService` struct and `MemoryRepository` trait
- **mm-memory-neo4j**: Neo4j-backed memory repository implementation
- **mm-server**: MCP server implementation
- **mm-utils**: Shared utility helpers

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
            create_rel_tool["CreateRelationshipTool::call_tool"]
            set_obs_tool["SetObservationsTool::call_tool"]
            add_obs_tool["AddObservationsTool::call_tool"]
            remove_obs_tool["RemoveObservationsTool::call_tool"]
            remove_all_obs_tool["RemoveAllObservationsTool::call_tool"]
            get_tool["GetEntityTool::call_tool"]
        end
        subgraph "resources"
            list_res["list_resources()"]
            list_tmpl["list_resource_templates()"]
            read_res["read_resource()"]
        end
        create_handler --> create_tool
        create_handler --> create_rel_tool
        create_handler --> set_obs_tool
        create_handler --> add_obs_tool
        create_handler --> remove_obs_tool
        create_handler --> remove_all_obs_tool
        create_handler --> get_tool
        create_handler --> list_res
        create_handler --> list_tmpl
        create_handler --> read_res
    end

    %% mm-core details
    subgraph "mm-core"
        create_op["create_entity"]
        create_rel_op["create_relationship"]
        set_obs_op["set_observations"]
        add_obs_op["add_observations"]
        remove_obs_op["remove_observations"]
        remove_all_obs_op["remove_all_observations"]
        get_op["get_entity"]
        create_op --> memory_service["MemoryService Struct"]
        create_rel_op --> memory_service
        set_obs_op --> memory_service
        add_obs_op --> memory_service
        remove_obs_op --> memory_service
        remove_all_obs_op --> memory_service
        get_op --> memory_service
    end

    %% mm-memory details
    subgraph "mm-memory"
        memory_service["MemoryService Struct"]
        repository_trait["MemoryRepository Trait"]
        memory_service -->|uses| repository_trait
    end

    %% mm-memory-neo4j details
    subgraph "mm-memory-neo4j"
        neo4j_repo["Neo4jRepository Struct"] -->|implements| repository_trait
        neo4j_repo --> neo4j_db[(Neo4j)]
    end

    %% Flow connections
    create_tool --> create_op
    create_rel_tool --> create_rel_op
    set_obs_tool --> set_obs_op
    add_obs_tool --> add_obs_op
    remove_obs_tool --> remove_obs_op
    remove_all_obs_tool --> remove_all_obs_op
    get_tool --> get_op
    read_res --> get_op
```

## Features

### Memory

- Stores and retrieves data in Neo4j
- Create and retrieve entities with labels, observations, and properties
- Set, add, remove, or clear observations
- Create relationships between entities

- Fetch any entity with `memory://{name}`; `list_resource_templates` advertises this
- Configurable logging

### Git

- Retrieve Git repository status with `git_status`

## Building

Run `cargo build` to compile all crates.

The workspace uses Rust 2024 as pinned in [`rust-toolchain.toml`](./rust-toolchain.toml).

### Using Nix

With [Nix](https://nixos.org/), you can build the CLI package:

```bash
nix build .#middle_manager
```

Install Nix with the [Determinate Systems installer](https://install.determinate.systems/nix):

```bash
curl -L https://install.determinate.systems/nix | sh -s -- --no-confirm
```

## Running

Run `cargo run -p mm-cli -- --config config/default.toml,config/local.toml` to start the CLI.
Use the `tools` subcommand to interact with MCP tools.

### CLI Options

```
USAGE:
    mm-cli [OPTIONS] [COMMAND]

OPTIONS:
    -l, --log-level <LOG_LEVEL>    Log level [default: info] [possible values: error, warn, info, debug, trace]
    -f, --logfile <FILE>           Path to log file (required if log level is specified)
    -r, --rotate-logs              Rotate logs (clear log file if it exists) [default: true]
    -c, --config <FILE>            Paths to config files (comma-separated, required)
    -h, --help                     Print help
    -V, --version                  Print version

COMMANDS:
    server    Start the MCP server (default)
    tools     Call server tools from the CLI
```

### Configuration

Configuration is loaded from the files specified with `-c` or `--config`.
Multiple paths can be provided separated by commas, allowing layered
configuration (for example `-c config/default.toml,config/local.toml`).

Example configuration:

```toml
[neo4j]
uri = "neo4j://localhost:7687"
username = "neo4j"
password = "password"
```

With `docker-compose.yml`, Neo4j runs on port `7688`. Update `config/local.toml` or set `MM_NEO4J__URI` to `neo4j://localhost:7688`.

### Using Tools

List available tools:

```bash
cargo run -p mm-cli -- tools list --config config/default.toml,config/local.toml --log-level debug
```

Call a tool (example adds an observation):

```bash
cargo run -p mm-cli -- tools call add_observations '{"name":"example","observations":["demo"]}' --config config/default.toml,config/local.toml
```

Call the built-in `tools/list` operation:

```bash
cargo run -p mm-cli -- tools call tools/list '{}' --config config/default.toml,config/local.toml
```

View the JSON schema for a tool:

```bash
cargo run -p mm-cli -- tools schema MemoryTools add_observations --config config/default.toml,config/local.toml
```


## Development

### Using Just

Use the `justfile` for common tasks:

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

This repository provides a Nix flake. Enter with `nix develop`, build with `nix build`, and run checks with `nix flake check`.

The flake builds the workspace with [naersk](https://github.com/nix-community/naersk) and the [rust overlay](https://github.com/oxalica/rust-overlay).

## Architecture

The project follows a hexagonal architecture (ports and adapters) pattern:

- **Core Domain**: Business logic independent of external protocols
- **Ports**: Interfaces for external dependencies
- **Adapters**: Implementations of ports for specific technologies
- **MCP Protocol**: External interface for AI assistants

This keeps the core domain isolated and testable.

## License

This project is licensed under the [Mozilla Public License 2.0](LICENSE).
