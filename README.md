# Middle Manager

Middle Manager provides a Model Context Protocol (MCP) interface for a Neo4j backed memory graph. The server exposes tools for creating and querying entities, managing tasks, and inspecting Git repositories. It follows a hexagonal architecture so that domain logic stays independent of the protocol implementation.

## Features

- Store entities, relationships and observations in Neo4j
- Manage project tasks as entities in the graph
- Query repository status and project context from Git
- Interact through an MCP server or the CLI

## Running

Compile and start the server using the CLI:

```bash
cargo run -p mm-cli -- --config config/default.toml,config/local.toml
```

List available tools:

```bash
cargo run -p mm-cli -- tools list --config config/default.toml,config/local.toml
```

The CLI also supports commands for validating configuration and viewing tasks.

## Workspace Layout

All crates are under `crates/`:

- **mm-cli** – command line interface and entry point
- **mm-core** – domain operations
- **mm-memory** – data model and repository traits
- **mm-memory-neo4j** – Neo4j implementation
- **mm-git** and **mm-git-git2** – Git abstractions
- **mm-server** – MCP integration
- **mm-utils** – shared utilities

## Development

Use the `justfile` for common tasks:

```bash
just validate   # clippy, fmt and unit tests
just inspect    # run the server with the MCP inspector
```

## License

This project is licensed under the [Mozilla Public License 2.0](LICENSE).

