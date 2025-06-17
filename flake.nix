{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs";
  inputs.naersk.url = "github:nix-community/naersk";
  inputs.naersk.inputs.nixpkgs.follows = "nixpkgs";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

  outputs = { self, nixpkgs, naersk, rust-overlay }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; overlays = [ rust-overlay.overlays.default ]; };
          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          naersk-lib = pkgs.callPackage naersk { cargo = rustToolchain; rustc = rustToolchain; };
        in
        rec {
          middle_manager = naersk-lib.buildPackage {
            pname = "middle_manager";
            src = self;
            cargoLock = ./Cargo.lock;
            nativeBuildInputs = with pkgs; [
              cmake
              pkg-config
            ];
            buildInputs = with pkgs; [
              openssl
              openssl.dev
            ];
            overrideMain = old: {
              preConfigure = (old.preConfigure or "") + ''
                cargo_build_options="$cargo_build_options -p mm-cli"
              '';
            };
            postInstall = ''
              mv $out/bin/mm-cli $out/bin/middle_manager
            '';
          };

          default = middle_manager;
        }
      );

      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; overlays = [ rust-overlay.overlays.default ]; };
          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        in
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rustToolchain
              rust-analyzer
              clippy
              rustfmt
              cmake
              pkg-config
              openssl
              openssl.dev
              nodejs
              cargo-nextest
            ];
            
            # Set environment variables for OpenSSL
            OPENSSL_DIR = "${pkgs.openssl.dev}";
            OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
            OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
          };
        }
      );
    };
}
