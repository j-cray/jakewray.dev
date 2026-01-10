{
  description = "Jakewray.ca Personal Portfolio";

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

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = ["rust-src" "rust-analyzer"];
          targets = ["wasm32-unknown-unknown"];
        };
      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            pkg-config
            openssl
            postgresql # for libpq and psql
            sqlx-cli
            cargo-leptos
            sass
            docker
            docker-compose
            podman
            podman-compose
            trunk # alternative to cargo-leptos, good to have
            nodejs # for npm/npx if needed for tailwind or other js tools (though avoided per user request)
          ];

          shellHook = ''
            export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}
            echo "Environment prepared for jakewray.ca development"
          '';
        };
      }
    );
}
