# Middle Manager

A Rust workspace containing the `mm-cli` binary and `mm-core` library crates.
All workspace crates reside in the `crates/` directory to keep the repository root tidy.

## Building

Run `cargo build` from the repository root to compile all crates.

## Running

Execute `cargo run -p mm-cli` to build and run the CLI.

## Using Nix

First, install Nix using the Determinate Systems installer:

```bash
curl -L https://install.determinate.systems/nix | sh -s -- install
```

This repository provides a Nix flake. Enter the development environment with

```bash
nix develop
```

and build the workspace via

```bash
nix build
```

To run the flake's checks, use

```bash
nix flake check
```

The flake uses [naersk](https://github.com/nix-community/naersk) together
with the [rust overlay](https://github.com/oxalica/rust-overlay) to build the
Rust workspace.
