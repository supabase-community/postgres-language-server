# WASM Binding for Postgres Language Server

## Key Discoveries

1. **pgls_query/build.rs already supports Emscripten** - detects `wasm32-unknown-emscripten`, uses `emcc`/`emar`
2. **Biome patterns to adopt**: Workspace struct, MemoryFileSystem, error handling, TypeScript generation
3. **Features without live DB**: completions, hover, lint (all use SchemaCache snapshot)
4. **SchemaCache lacks serde** - needs derives added for JSON import

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  pgls_wasm (single WASM via Emscripten)                     │
├─────────────────────────────────────────────────────────────┤
│  Exported Structs (like Biome):                             │
│  ┌─────────────────────┐  ┌─────────────────────────────┐   │
│  │  MemoryFileSystem   │  │  Workspace                  │   │
│  │  • insert(path,data)│  │  • lint(path) → Diagnostic[]│   │
│  │  • remove(path)     │  │  • complete(path,pos)       │   │
│  └─────────────────────┘  │  • hover(path,pos)          │   │
│                           │  • updateSettings(config)    │   │
│                           │  • handleMessage(jsonrpc)    │   │
│                           └─────────────────────────────────┘   │
│  Internal:                                                  │
│  • pgls_query (libpg_query C → emcc)                        │
│  • pgls_treesitter_grammar (tree-sitter C → emcc)           │
│  • pgls_completions, pgls_hover, pgls_analyser              │
│  • pgls_schema_cache (JSON import)                          │
└─────────────────────────────────────────────────────────────┘
```

## API Surface

### Direct Methods (Primary - like Biome)
```typescript
// Setup
const fs = new MemoryFileSystem();
fs.insert('/project/query.sql', 'SELECT * FROM users');

const workspace = new Workspace(fs);
workspace.setSchema(schemaJson);  // Optional: enables completions

// Use
const diagnostics = workspace.lint('/project/query.sql');
const completions = workspace.complete('/project/query.sql', 16);
const hover = workspace.hover('/project/query.sql', 14);
```

### LSP Protocol (Secondary - for LSP clients)
```typescript
// Full LSP compatibility via JSON-RPC
const response = workspace.handleMessage(JSON.stringify({
  jsonrpc: '2.0',
  id: 1,
  method: 'textDocument/completion',
  params: { textDocument: { uri: 'file:///query.sql' }, position: { line: 0, character: 16 } }
}));
```

## Implementation Steps

### 1. Add serde to SchemaCache
`crates/pgls_schema_cache/src/*.rs` - add `#[derive(Serialize, Deserialize)]` to all types

### 2. Update tree-sitter grammar for Emscripten
`crates/pgls_treesitter_grammar/build.rs` - add Emscripten detection like pgls_query

### 3. Create pgls_wasm crate (following Biome patterns)

```toml
# crates/pgls_wasm/Cargo.toml
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
pgls_query = { path = "../pgls_query" }
pgls_treesitter_grammar = { path = "../pgls_treesitter_grammar" }
pgls_schema_cache = { path = "../pgls_schema_cache", features = ["serde"] }
pgls_analyser = { path = "../pgls_analyser" }
pgls_completions = { path = "../pgls_completions" }
pgls_hover = { path = "../pgls_hover" }
pgls_lsp = { path = "../pgls_lsp" }  # For handleMessage
pgls_fs = { path = "../pgls_fs" }    # MemoryFileSystem

serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### 4. Implement Rust structs

```rust
// crates/pgls_wasm/src/lib.rs

/// In-memory file system (like Biome's MemoryFileSystem)
pub struct MemoryFileSystem {
    inner: pgls_fs::MemoryFileSystem,
}

impl MemoryFileSystem {
    pub fn new() -> Self { ... }
    pub fn insert(&self, path: &str, data: &[u8]) { ... }
    pub fn remove(&self, path: &str) { ... }
}

/// Main workspace API (like Biome's Workspace)
pub struct Workspace {
    fs: Arc<MemoryFileSystem>,
    schema: Option<SchemaCache>,
}

impl Workspace {
    pub fn new(fs: &MemoryFileSystem) -> Self { ... }

    // Direct methods
    pub fn set_schema(&mut self, json: &str) -> Result<(), Error> { ... }
    pub fn lint(&self, path: &str) -> Result<Vec<Diagnostic>, Error> { ... }
    pub fn complete(&self, path: &str, offset: u32) -> Result<Vec<CompletionItem>, Error> { ... }
    pub fn hover(&self, path: &str, offset: u32) -> Result<Option<String>, Error> { ... }

    // LSP protocol
    pub fn handle_message(&self, json_rpc: &str) -> Result<String, Error> { ... }
}

// Error conversion (Biome pattern)
fn into_error<E: Display>(err: E) -> Error {
    Error::new(&err.to_string())
}
```

### 5. C ABI wrapper for Emscripten

```rust
// crates/pgls_wasm/src/ffi.rs
// Thin C wrappers around the Rust structs

#[no_mangle]
pub extern "C" fn pgls_fs_new() -> *mut MemoryFileSystem { ... }

#[no_mangle]
pub extern "C" fn pgls_fs_insert(fs: *mut MemoryFileSystem, path: *const c_char, data: *const u8, len: usize) { ... }

#[no_mangle]
pub extern "C" fn pgls_workspace_new(fs: *mut MemoryFileSystem) -> *mut Workspace { ... }

#[no_mangle]
pub extern "C" fn pgls_workspace_lint(ws: *mut Workspace, path: *const c_char) -> *mut c_char { ... }

// ... etc
```

### 6. Build with Emscripten

```bash
emsdk install latest && emsdk activate latest
cargo build -p pgls_wasm --target wasm32-unknown-emscripten --release

# Emscripten flags (in build.rs or .cargo/config.toml)
# -sEXPORT_ES6=1 -sMODULARIZE=1 -sEXPORTED_FUNCTIONS=[...]
```

### 7. TypeScript wrapper

`packages/@postgres-language-server/wasm/`

```typescript
// Wraps Emscripten C ABI with clean TypeScript API
import Module from './pgls_wasm.js';

export class MemoryFileSystem {
  private ptr: number;
  constructor() { this.ptr = Module._pgls_fs_new(); }
  insert(path: string, data: Uint8Array) { ... }
  remove(path: string) { ... }
}

export class Workspace {
  private ptr: number;
  constructor(fs: MemoryFileSystem) { ... }

  setSchema(schema: SchemaCache) { ... }
  lint(path: string): Diagnostic[] { ... }
  complete(path: string, offset: number): CompletionItem[] { ... }
  hover(path: string, offset: number): string | null { ... }

  // LSP protocol
  handleMessage(jsonRpc: string): string { ... }
}
```

### 8. CLI schema export

```bash
postgres-language-server schema export --output schema.json
# Connects to DB, dumps SchemaCache as JSON
```

## Files to Create/Modify

| File | Action |
|------|--------|
| `crates/pgls_schema_cache/src/*.rs` | Add serde derives |
| `crates/pgls_treesitter_grammar/build.rs` | Add Emscripten support |
| `crates/pgls_wasm/` | New crate |
| `crates/pgls_wasm/src/lib.rs` | Rust structs (MemoryFileSystem, Workspace) |
| `crates/pgls_wasm/src/ffi.rs` | C ABI wrappers |
| `packages/@postgres-language-server/wasm/` | New TS package |
| `crates/pgls_cli/` | Add `schema export` command |

## Open Questions

1. **Bundle size** - ~5MB estimate acceptable?
2. **Tokio/async deps** - need to stub or remove for WASM (Emscripten has limited async)
3. **LSP message routing** - implement from scratch or adapt tower-lsp internals?
4. **TypeScript generation** - manual or build.rs like Biome?

## References

- Biome WASM: `biome-main/crates/biome_wasm/`
- Biome LSP: `biome-main/crates/biome_lsp/`
- Supabase pg-parser: https://github.com/supabase-community/pg-parser
