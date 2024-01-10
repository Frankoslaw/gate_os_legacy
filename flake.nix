{
  description = "A devShell example";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { 
          inherit system; 
          overlays = [
            (import rust-overlay)
          ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            (pkgs.rust-bin.nightly.latest.default.override {
                  extensions = [ "rust-src" "cargo" "rustc" "llvm-tools-preview" "rust-std" "rust-analyzer" ];
                  targets = ["x86_64-unknown-linux-gnu" "x86_64-unknown-none"];
            })
          ];

          buildInputs = with pkgs; [
            pre-commit
            gitleaks
            cz-cli

            cargo-make
            cargo-deny
            pkg-config
            clippy

            gdb
            lldb
          ];

          RUST_SRC_PATH = "${pkgs.rust-bin.nightly.latest.default.override {
              extensions = [ "rust-src" "cargo" "rustc" "llvm-tools-preview" "rust-std" ];
              targets = ["x86_64-unknown-linux-gnu" "x86_64-unknown-none"];
          }}/lib/rustlib/src/rust/library";
        };
      }
    );
}
