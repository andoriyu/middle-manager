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

# Run the MCP inspector with mm-cli
inspect:
    npx @modelcontextprotocol/inspector cargo run -p mm-cli

# Run the MCP inspector with mm-cli in debug mode
inspect-debug:
    npx @modelcontextprotocol/inspector cargo run -p mm-cli -- --log-level debug
