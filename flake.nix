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

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Read rust-toolchain.toml to get the exact Rust version
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        # Nightly toolchain for rustfmt (used by codegen)
        rustNightly = pkgs.rust-bin.nightly.latest.minimal.override {
          extensions = [ "rustfmt" ];
        };

        # Extract just nightly rustfmt (to avoid nightly rustc taking precedence)
        nightlyRustfmtOnly = pkgs.runCommand "nightly-rustfmt" { } ''
          mkdir -p $out/bin
          ln -s ${rustNightly}/bin/rustfmt $out/bin/rustfmt
        '';

        # Development dependencies
        buildInputs = with pkgs; [
          # Nightly rustfmt (for codegen) - must come before stable toolchain
          nightlyRustfmtOnly
          # Rust toolchain (stable from rust-toolchain.toml)
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
          taplo

          # Docker
          docker-compose

          # LSP and development tools
          rust-analyzer

          # Additional tools that might be needed
          cmake
          gcc
          libiconv
          llvmPackages.clang
          llvmPackages.libclang

          # WebAssembly toolchain
          emscripten

          # Database tools
          sqlx-cli
        ];

        # Environment variables
        env = {
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          # Emscripten SDK path for WASM builds
          EMSDK = "${pkgs.emscripten}";
        };

      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs;
          hardeningDisable = [ "fortify" ];
          shellHook = ''
            echo "Postgres Language Server Development Environment"

            # Set environment variables
            ${pkgs.lib.concatStringsSep "\n" (
              pkgs.lib.mapAttrsToList (name: value: "export ${name}=\"${value}\"") env
            )}
          '';
        };

        # Formatter for nix files
        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
