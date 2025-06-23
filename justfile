# Middle Manager justfile

# Default recipe to display help information
default:
    @just --list

# Build the project
build:
    cargo build

# Run tests
test:
    cargo test

# Check code formatting
fmt-check:
    cargo fmt --all -- --check

# Format code
fmt:
    cargo fmt --all

# Run clippy lints
clippy:
    cargo clippy -- -D warnings

# Check the project
check:
    cargo check

# Run the MCP inspector with mm-cli using local config
inspect:
    npx @modelcontextprotocol/inspector cargo "run -p mm-cli -- --config config/local.toml --logfile log.log --log-rotate"

# Run the MCP inspector with mm-cli in debug mode using local config
inspect-debug:
    npx @modelcontextprotocol/inspector cargo "run -p mm-cli -- --config config/local.toml --log-level debug --logfile log.log --log-rotate"

# Delete Neo4j data and logs volumes
clean-neo4j:
    docker compose down
    docker volume rm middle-manager_neo4j_data middle-manager_neo4j_logs

# Validate code with lints, formatting, and unit tests
validate:
    just fmt
    cargo clean
    just clippy
    cargo test --workspace --lib
    cargo check --workspace --tests
