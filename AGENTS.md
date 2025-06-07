# Repository Guidelines

- Install Nix in your sandbox using the Determinate Systems installer:

  ```bash
  curl -L https://install.determinate.systems/nix | sh -s -- install
  ```

- A Nix flake is provided. You can enter the dev environment with `nix develop` and build with `nix build`.
- Run tests with `cargo nextest run --workspace --all-targets --profile ci`.

