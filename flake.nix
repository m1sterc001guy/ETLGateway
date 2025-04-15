{
  description = "Dev shell with nightly Rust and edition2024 support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };

        rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" ];
        });
      in {
        devShells.default = pkgs.mkShell {
          packages = [
            rust
            pkgs.sqlite
            pkgs.just
          ];

          shellHook = ''
            echo "🚀 Rust nightly with edition2024 and rust-src enabled!"
            export RUSTC="${rust}/bin/rustc"
            export CARGO="${rust}/bin/cargo"
          '';
        };
      }
    );
}


