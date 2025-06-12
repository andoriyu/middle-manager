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
          naersk-lib = pkgs.callPackage naersk { };
        in
        {
          default = naersk-lib.buildPackage {
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
          };
        }
      );

      devShells = forAllSystems (system:
        let pkgs = import nixpkgs { inherit system; overlays = [ rust-overlay.overlays.default ]; }; in
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rust-bin.stable.latest.default
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
