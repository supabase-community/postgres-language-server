#!/bin/bash
# Build script for pgls_wasm with Emscripten
#
# This builds Rust as a static library, then uses emcc to link and generate
# the final WASM + JS with proper Emscripten runtime support.
#
# Prerequisites:
#   - Emscripten SDK (https://emscripten.org/docs/getting_started/downloads.html)
#   - Rust target: rustup target add wasm32-unknown-emscripten
#
# Usage:
#   ./build-wasm.sh [--release]

set -e

# Check for Emscripten
if ! command -v emcc &> /dev/null; then
    echo "Error: Emscripten (emcc) not found in PATH"
    echo "Please install Emscripten: https://emscripten.org/docs/getting_started/downloads.html"
    echo "And run: source /path/to/emsdk/emsdk_env.sh"
    exit 1
fi

# Parse arguments
RELEASE=""
BUILD_TYPE="debug"
CARGO_PROFILE="dev"
if [ "$1" = "--release" ]; then
    RELEASE="--release"
    BUILD_TYPE="release"
    CARGO_PROFILE="release"
fi

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUT_DIR="$ROOT_DIR/target/wasm32-unknown-emscripten/$BUILD_TYPE"
DIST_DIR="$SCRIPT_DIR/dist"

echo "Building pgls_wasm for wasm32-unknown-emscripten ($BUILD_TYPE)..."

# Add the Emscripten target if not present
rustup target add wasm32-unknown-emscripten 2>/dev/null || true

# Set up cross-compilation environment for Emscripten
export CC_wasm32_unknown_emscripten="emcc"
export CXX_wasm32_unknown_emscripten="em++"
export AR_wasm32_unknown_emscripten="emar"
export CARGO_TARGET_WASM32_UNKNOWN_EMSCRIPTEN_LINKER="emcc"

# Ensure emscripten cache is set up
export EM_CACHE="${EM_CACHE:-$HOME/.emscripten_cache}"
mkdir -p "$EM_CACHE"

# Build with cargo - produces a static library (.a file)
cd "$ROOT_DIR"
cargo build -p pgls_wasm --target wasm32-unknown-emscripten $RELEASE

# Create dist directory
mkdir -p "$DIST_DIR"

# Find the static library
STATIC_LIB="$OUT_DIR/libpgls_wasm.a"

if [ ! -f "$STATIC_LIB" ]; then
    echo "Error: Static library not found at $STATIC_LIB"
    echo "Available files:"
    ls -la "$OUT_DIR" 2>/dev/null || echo "  Directory does not exist"
    exit 1
fi

echo "Linking with emcc..."

# Exported functions (C ABI functions from ffi.rs)
EXPORTED_FUNCTIONS="[
  '_malloc',
  '_free',
  '_pgls_init',
  '_pgls_free_string',
  '_pgls_set_schema',
  '_pgls_clear_schema',
  '_pgls_insert_file',
  '_pgls_remove_file',
  '_pgls_lint',
  '_pgls_complete',
  '_pgls_hover',
  '_pgls_parse',
  '_pgls_version',
  '_pgls_handle_message'
]"

# Exported runtime methods for string handling
EXPORTED_RUNTIME="[
  'UTF8ToString',
  'stringToUTF8',
  'lengthBytesUTF8',
  'getValue',
  'setValue'
]"

# Optimization flags based on build type
# Note: We use -O2 instead of -O3 for release builds because:
# - -flto causes LTO bitcode mismatch errors in CI environments
# - -O3 triggers wasm-opt post-processing issues in some Emscripten versions
if [ "$BUILD_TYPE" = "release" ]; then
    OPT_FLAGS="-O2"
else
    OPT_FLAGS="-O0 -g"
fi

# Link with emcc to produce WASM + JS
emcc "$STATIC_LIB" \
    $OPT_FLAGS \
    -s WASM=1 \
    -s MODULARIZE=1 \
    -s EXPORT_NAME="createPGLS" \
    -s EXPORTED_FUNCTIONS="$EXPORTED_FUNCTIONS" \
    -s EXPORTED_RUNTIME_METHODS="$EXPORTED_RUNTIME" \
    -s ALLOW_MEMORY_GROWTH=1 \
    -s INITIAL_MEMORY=16777216 \
    -s STACK_SIZE=1048576 \
    -s NO_EXIT_RUNTIME=1 \
    -s FILESYSTEM=0 \
    -s ENVIRONMENT='web,worker,node' \
    -s EXPORT_ES6=1 \
    -s USE_ES6_IMPORT_META=1 \
    -s DYNAMIC_EXECUTION=0 \
    -s DISABLE_EXCEPTION_CATCHING=0 \
    -o "$DIST_DIR/pgls.js"

echo ""
echo "Build complete!"
echo "Output files:"
echo "  $DIST_DIR/pgls.js   (Emscripten JS glue)"
echo "  $DIST_DIR/pgls.wasm (WebAssembly module)"
echo ""
echo "Usage in JavaScript:"
echo "  import createPGLS from './pgls.js';"
echo "  const pgls = await createPGLS();"
echo "  pgls._pgls_init();"
