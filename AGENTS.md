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
│   │   │   ├── operations/  # Business operations (create_entity, create_relationship, get_entity, set_observations, add_observations, remove_observations, remove_all_observations)
│   │   │   ├── ports.rs     # Dependency injection container
│   ├── mm-server/      # MCP server implementation
│   │   ├── src/
│   │   │   ├── mcp/        # MCP tool implementations
│   │   │   ├── lib.rs      # Server handler implementation
│   ├── mm-memory/      # Memory domain types and schema
│   ├── mm-memory-neo4j/      # Memory graph repository and service
├── config/
│   ├── default.toml    # Default configuration
│   ├── local.toml      # Local overrides (gitignored)
├── justfile            # Common development tasks
```

The `mm-memory` crate defines the memory data model. Its schema is stored in
`crates/mm-memory/schema.json`.

## Available MCP Resources

Middle Manager exposes memory entities via URIs with the `memory://` scheme. A URI takes the form `memory://{name}` where `{name}` is the entity identifier. For example, `memory://tech:language:rust` retrieves the `tech:language:rust` entity.

## Available MCP Tools

Middle Manager exposes the following tools through the MCP protocol:

### 1. `create_entity`

Create new entities in the memory graph.

**Parameters:**
- `entities`: Array of objects each with `name`, `labels`, `observations`, and optional `properties` fields

**Example:**
```json
{
  "entities": [
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
  ]
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

### 3. `set_observations`

Replace all observations for an entity.

**Parameters:**
- `name`: String - Entity to modify
- `observations`: Array of String - New observations

### 4. `add_observations`

Add observations to an entity without removing existing ones.

**Parameters:**
- `name`: String - Entity to modify
- `observations`: Array of String - Observations to add

### 5. `remove_observations`

Remove specific observations from an entity.

**Parameters:**
- `name`: String - Entity to modify
- `observations`: Array of String - Observations to remove

### 6. `remove_all_observations`

Delete all observations from an entity.

**Parameters:**
- `name`: String - Entity to modify

### 7. `create_relationship`

Create relationships between entities.

**Parameters:**
- `relationships`: Array of objects with `from`, `to`, `name`, and optional `properties` fields

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

Agents running in restricted environments (such as Codex) should only run unit tests using the command above or `just validate`.

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

To validate code before committing, run:
```bash
just validate
```
This command runs clippy, formats the code, and executes unit tests without integration tests. Ensure it passes. If `just` is missing, install it using the official installer script:
```bash
curl -sL https://just.systems/install.sh | bash -s -- --to ~/.cargo/bin
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

## Property Testing

The project uses property-based tests to validate many operations. These tests
live alongside unit tests in the `mm-core` and `mm-utils` crates. Property tests
use helper utilities from `mm-utils::prop` such as `NonEmptyName` for generating
valid entity names and `async_arbtest` for running asynchronous checks. Random
inputs are derived with the `arbitrary` crate and executed using the `arbtest`
framework.

To run the property tests, execute:
```bash
cargo test
# or
just validate
```
