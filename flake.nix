{
  description = "Rust WASM build deps";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
        with pkgs; {
          devShells.default = mkShell {
            buildInputs = [
              openssl
              pkg-config
              wasm-bindgen-cli
              wasm-pack
              # Version suggested by: https://github.com/RReverser/wasm-bindgen-rayon?tab=readme-ov-file#using-config-files
              (rust-bin.nightly."2024-08-02".default.override {
                targets = ["wasm32-unknown-unknown"];
                extensions = ["rust-src"];
              })
            ];

            # shellHook = ''
            #   export RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals'
            # '';
          };
        }
    );
}
