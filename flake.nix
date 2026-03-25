{
  description = "Jakewray.dev Personal Portfolio";

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
          name = "jakewray-dev";
          
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain
            cargo-leptos
            wasm-bindgen-cli
            
            # Build dependencies
            pkg-config
            openssl
            
            # Database
            sqlite
            sqlx-cli
            
            # Styling
            sass
            dart-sass
            
            # Web compilation (optional)
            nodejs
            
            # Development tools
            git
            just
          ];

          shellHook = ''
            export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}
            export RUST_LOG=info
            export DATABASE_URL="sqlite://jakewray.db"
            
            echo "🚀 jakewray.dev development environment loaded"
            echo "   Rust: $(rustc --version)"
            echo "   Cargo: $(cargo --version)"
            echo "   Database: SQLite"
            echo ""
            echo "📚 Quick commands:"
            echo "   cargo leptos watch      - Start dev server"
            echo "   ./scripts/setup-dev.sh  - Setup local database"
          '';
        };
      }
    );
}
