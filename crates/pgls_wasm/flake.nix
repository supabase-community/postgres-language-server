{
  description = "WASM build environment for postgres-language-server";

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

        # Rust with wasm32-unknown-emscripten target
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-emscripten" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain with WASM target
            rustToolchain

            # Emscripten SDK
            emscripten

            # Build tools
            pkg-config
            cmake

            # For TypeScript package
            nodejs
            bun

            # Optional: for testing
            wabt  # WebAssembly Binary Toolkit
          ];

          shellHook = ''
            echo "WASM build environment for postgres-language-server"
            echo ""
            echo "Available commands:"
            echo "  just build-wasm          - Build debug WASM"
            echo "  just build-wasm-release  - Build release WASM"
            echo "  just check-wasm-prereqs  - Check prerequisites"
            echo ""
            echo "Emscripten version: $(emcc --version | head -1)"
            echo "Rust version: $(rustc --version)"
            echo ""
          '';

          # Set up Emscripten environment
          EMSDK = "${pkgs.emscripten}";
          EM_CACHE = "/tmp/emscripten-cache";
        };

        # Package for building WASM
        packages.default = pkgs.stdenv.mkDerivation {
          pname = "pgls-wasm";
          version = "0.0.0";

          src = ../..;

          nativeBuildInputs = with pkgs; [
            rustToolchain
            emscripten
            pkg-config
            cmake
          ];

          # Disable default configure phase (no CMakeLists.txt)
          dontConfigure = true;

          buildPhase = ''
            export HOME=$TMPDIR
            export EM_CACHE=$TMPDIR/emscripten-cache
            mkdir -p $EM_CACHE
            cd crates/pgls_wasm
            bash ./build-wasm.sh --release
          '';

          installPhase = ''
            mkdir -p $out
            cp -r crates/pgls_wasm/dist/* $out/
          '';

          # Emscripten environment
          EMSDK = "${pkgs.emscripten}";
        };
      }
    );
}
