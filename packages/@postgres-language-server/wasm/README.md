# @postgres-language-server/wasm

WebAssembly bindings for the Postgres Language Server. This package provides two independent APIs for working with PostgreSQL SQL in the browser and Node.js.

## Installation

```bash
npm install @postgres-language-server/wasm
# or
bun add @postgres-language-server/wasm
```

## Two APIs

This package provides two separate APIs. Choose the one that fits your use case:

| API                | Use Case                            | Import Path                                |
| ------------------ | ----------------------------------- | ------------------------------------------ |
| **Workspace**      | Direct parse, lint, complete, hover | `@postgres-language-server/wasm/workspace` |
| **LanguageServer** | Full LSP JSON-RPC protocol          | `@postgres-language-server/wasm/lsp`       |

Each API manages its own workspace independently. Use one or the other, not both.

## Workspace API

Use this for custom editor integrations, build-time SQL linting, or simple tooling that doesn't need full LSP.

```typescript
import { createWorkspace } from "@postgres-language-server/wasm/workspace";

const workspace = await createWorkspace();

// Parse SQL and get errors
const errors = workspace.parse("SELECT * FROM users;");
console.log(errors); // []

// Insert a file and lint it
workspace.insertFile("/query.sql", "SELECT * FROM users;");
const diagnostics = workspace.lint("/query.sql");

// Get completions
const completions = workspace.complete("/query.sql", 14); // position after "FROM "

// Get hover info
const hover = workspace.hover("/query.sql", 14); // position over "users"
```

### With Schema

For schema-aware completions and hover, provide your database schema:

```typescript
const workspace = await createWorkspace();

// Set schema (matches pgls_schema_cache format)
workspace.setSchema(JSON.stringify({
  schemas: [{ id: 1, name: 'public', owner: 'postgres', ... }],
  tables: [{ id: 1, schema: 'public', name: 'users', ... }],
  columns: [{ name: 'id', table_name: 'users', type_name: 'integer', ... }],
  functions: [],
  types: [],
  // ...
}));

workspace.insertFile('/query.sql', 'SELECT * FROM ');
const completions = workspace.complete('/query.sql', 14);
// completions now include 'users' table
```

## LanguageServer API

Use this for Monaco editor integration with `monaco-languageclient` or any editor that speaks LSP protocol.

```typescript
import { createLanguageServer } from "@postgres-language-server/wasm/lsp";

const lsp = await createLanguageServer();

// Handle LSP messages
const responses = lsp.handleMessage({
  jsonrpc: "2.0",
  id: 1,
  method: "initialize",
  params: { capabilities: {} },
});

// responses is an array of outgoing messages
for (const msg of responses) {
  // Send to client...
}
```

### Web Worker Integration

For Monaco editor, run the language server in a web worker:

```typescript
// lsp-worker.js
import { createLanguageServer } from "@postgres-language-server/wasm/lsp";

let lsp = null;

self.onmessage = async (event) => {
  if (!lsp) {
    lsp = await createLanguageServer();
    self.postMessage({ type: "ready" });
  }

  const responses = lsp.handleMessage(event.data);
  for (const msg of responses) {
    self.postMessage(msg);
  }
};
```

### Setting Schema via LSP

Use the `pgls/setSchema` notification:

```typescript
lsp.handleMessage({
  jsonrpc: "2.0",
  method: "pgls/setSchema",
  params: { schema: JSON.stringify(schemaCache) },
});
```

## API Reference

### Workspace

| Method                      | Description                                |
| --------------------------- | ------------------------------------------ |
| `parse(sql: string)`        | Parse SQL, returns array of error messages |
| `insertFile(path, content)` | Add or update a file in the workspace      |
| `removeFile(path)`          | Remove a file from the workspace           |
| `lint(path)`                | Get diagnostics for a file                 |
| `complete(path, offset)`    | Get completions at position                |
| `hover(path, offset)`       | Get hover info at position                 |
| `setSchema(json)`           | Set database schema                        |
| `clearSchema()`             | Clear the current schema                   |
| `version()`                 | Get library version                        |

### LanguageServer

| Method               | Description                                              |
| -------------------- | -------------------------------------------------------- |
| `handleMessage(msg)` | Process LSP JSON-RPC message, returns array of responses |

### Supported LSP Methods

- `initialize`
- `shutdown`
- `textDocument/didOpen`
- `textDocument/didChange`
- `textDocument/didClose`
- `textDocument/completion`
- `textDocument/hover`
- `pgls/setSchema` (custom notification)

## Building

Requires Emscripten SDK:

```bash
# Build WASM
./crates/pgls_wasm/build-wasm.sh --release

# Build TypeScript
cd packages/@postgres-language-server/wasm
bun run build
```

## License

MIT
