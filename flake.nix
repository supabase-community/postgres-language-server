{
  description = "PostgreSQL Language Server Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Read rust-toolchain.toml to get the exact Rust version
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        # Development dependencies
        buildInputs = with pkgs; [
          # Rust toolchain
          rustToolchain
          
          # Node.js ecosystem
          bun
          nodejs_20
          
          # Python for additional tooling
          python3
          python3Packages.pip
          
          # System dependencies
          pkg-config
          openssl
          
          # Build tools
          just
          git
          
          # LSP and development tools
          rust-analyzer
          
          # Additional tools that might be needed
          cmake
          gcc
          libiconv
        ];
        
        # Environment variables
        env = {
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };

      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs;
          
          shellHook = ''
            echo "PostgreSQL Language Server Development Environment"
            echo "Available tools:"
            echo "  • Rust $(rustc --version)"
            echo "  • Node.js $(node --version)"
            echo "  • Bun $(bun --version)"
            echo "  • Just $(just --version)"
            echo ""
            echo "Development Commands:"
            echo "  • just --list   # Show tasks"
            echo "  • cargo check   # Check Rust"
            echo "  • bun install   # Install deps"
            echo ""
            echo "Use Docker for database:"
            echo "  • docker-compose up -d"
            echo ""
            
            # Set environment variables
            ${pkgs.lib.concatStringsSep "\n" 
              (pkgs.lib.mapAttrsToList (name: value: "export ${name}=\"${value}\"") env)}
          '';
        };

        # Formatter for nix files
        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}