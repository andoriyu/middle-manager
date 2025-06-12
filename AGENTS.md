# Middle Manager: Guide for AI Agents

This document provides guidance for AI agents interacting with the Middle Manager project. It outlines the project structure, key concepts, and how to effectively assist users with this codebase.

## Project Overview

Middle Manager is a Model Context Protocol (MCP) server that provides tools for interacting with a Neo4j memory graph. It follows a hexagonal architecture pattern to maintain clean separation between domain logic and external protocols.

## Key Concepts

### Memory Graph

The memory graph is a Neo4j graph database that stores knowledge as entities with:
- **Name**: Unique identifier for the entity
- **Labels**: Categories or types assigned to the entity
- **Observations**: Facts or information about the entity
- **Properties**: Additional key-value metadata

### Model Context Protocol (MCP)

MCP is an open protocol that standardizes how applications provide context to LLMs. Middle Manager implements this protocol to expose tools for interacting with the memory graph.

### Hexagonal Architecture

The project uses a ports and adapters pattern:
- **Core Domain**: Business logic in mm-core, independent of external protocols
- **Ports**: Interfaces for external dependencies
- **Adapters**: Implementations of ports for specific technologies
- **MCP Protocol**: External interface in mm-server

## Project Structure

```
middle-manager/
├── crates/
│   ├── mm-cli/         # Command-line interface
│   ├── mm-core/        # Core domain logic and operations
│   │   ├── src/
│   │   │   ├── operations/  # Business operations (get_entity, create_entity)
│   │   │   ├── ports.rs     # Dependency injection container
│   ├── mm-server/      # MCP server implementation
│   │   ├── src/
│   │   │   ├── mcp/        # MCP tool implementations
│   │   │   ├── lib.rs      # Server handler implementation
│   ├── mm-memory/      # Memory graph repository and service
├── config/
│   ├── default.toml    # Default configuration
│   ├── local.toml      # Local overrides (gitignored)
├── justfile            # Common development tasks
```

## Available MCP Tools

Middle Manager exposes the following tools through the MCP protocol:

### 1. `create_entity`

Creates a new entity in the memory graph.

**Parameters:**
- `name`: String - Unique identifier for the entity
- `labels`: Array of String - Categories or types for the entity
- `observations`: Array of String - Facts or information about the entity
- `properties`: Object (optional) - Additional key-value metadata

**Example:**
```json
{
  "name": "tech:language:rust",
  "labels": ["Memory", "Technology", "Language"],
  "observations": [
    "Rust is a systems programming language",
    "Rust emphasizes safety, especially memory safety"
  ],
  "properties": {
    "website": "https://www.rust-lang.org/",
    "created_year": "2010"
  }
}
```

### 2. `get_entity`

Retrieves an entity from the memory graph by name.

**Parameters:**
- `name`: String - Name of the entity to retrieve

**Example:**
```json
{
  "name": "tech:language:rust"
}
```

## Testing

### Unit Testing

The project uses Rust's built-in testing framework. Run tests with:

```bash
cargo test
```

To run tests for a specific crate:

```bash
cargo test -p mm-core
```

### Integration Testing

Integration tests can be run with:

```bash
cargo test --test '*'
```

Integration tests require a running Neo4j database started via Docker Compose.
If the database isn't available, skip them with:

```bash
cargo test --workspace --lib
```

To run all tests, start Neo4j first:

```bash
docker compose up -d
cargo test --workspace
```

### Testing with Mock Services

The project uses `mockall` for mocking services in tests. Example of testing with mocks:

```rust
#[tokio::test]
async fn test_get_entity_success() {
    let mut mock = MockMemoryService::<neo4rs::Error>::new();
    
    // Set up the mock expectation
    mock.expect_find_entity_by_name()
        .with(eq("test:entity"))
        .returning(|_| Ok(Some(MemoryEntity {
            name: "test:entity".to_string(),
            labels: vec!["Test".to_string()],
            observations: vec!["Test observation".to_string()],
            properties: HashMap::new(),
        })));
    
    let ports = Ports::new(Arc::new(mock));
    
    let command = GetEntityCommand {
        name: "test:entity".to_string(),
    };
    
    let result = get_entity(&ports, command).await;
    
    assert!(result.is_ok());
    let entity = result.unwrap().unwrap();
    assert_eq!(entity.name, "test:entity");
}
```

## Assisting with Common Tasks

### 1. Running the Server

Recommend users run the server with:
```bash
cargo run -p mm-cli
```

Or with custom configuration:
```bash
cargo run -p mm-cli -- --config path/to/config.toml
```

### 2. Debugging Issues

For debugging, suggest enabling debug logs:
```bash
cargo run -p mm-cli -- --log-level debug --logfile debug.log
```

### 3. Using Just Commands

Recommend the `just` command for common tasks:
```bash
just inspect      # Run with MCP inspector
just inspect-debug # Run with debug logging
```

### 4. Working with Neo4j

If users need to clean Neo4j data:
```bash
just clean-neo4j
```

### 5. Code Structure Guidance

When helping with code changes:
- Keep domain logic in mm-core independent of MCP
- Use the Ports struct for dependency injection
- Implement MCP-specific code only in mm-server
- Follow the hexagonal architecture pattern

## Naming Conventions

### Entity Names

Entity names follow the format: `domain:type:name[:subtype]`

Examples:
- `tech:language:rust`
- `project:component:auth_service`
- `user:preference:theme:dark`

### Labels

Labels use PascalCase and categorize entities:
- `Memory` - Base label for all entities
- `Technology` - Technology-related entities
- `Project` - Project-related entities
- `Component` - Software components
- `User` - User-related entities

## Best Practices for AI Assistance

1. **Respect Architecture**: When suggesting code changes, maintain the separation between domain logic and MCP protocol
2. **Use Ports Pattern**: Encourage dependency injection through the Ports struct
3. **Follow Naming Conventions**: Suggest entity names and labels that follow the established patterns
4. **Understand Graph Structure**: Consider relationships between entities when suggesting queries
5. **Leverage Just Commands**: Recommend appropriate just commands for common tasks

By understanding these concepts and patterns, you can provide more effective assistance to users working with the Middle Manager project.
