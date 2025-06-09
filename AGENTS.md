# Repository Guidelines

- Install Nix in your sandbox using the Determinate Systems installer:

  ```bash
  curl -L https://install.determinate.systems/nix | sh -s -- install
  ```

- A Nix flake is provided. You can enter the dev environment with `nix develop` and build with `nix build`.
- Use `nix develop` (or `nix run`) when invoking `cargo` and `cargo nextest` commands.
- Run tests with `cargo nextest run --workspace --all-targets --profile ci`.
- `cargo fmt --check`, `cargo clippy -- -D warnings`, and `nix flake check` must all pass before work is ready.

