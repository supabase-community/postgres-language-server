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

| API                | Use Case                                    | Import Path                                |
| ------------------ | ------------------------------------------- | ------------------------------------------ |
| **Workspace**      | Direct parse, lint, format, complete, hover | `@postgres-language-server/wasm/workspace` |
| **LanguageServer** | Full LSP JSON-RPC protocol                  | `@postgres-language-server/wasm/lsp`       |

Each API manages its own workspace independently. Use one or the other, not both.

## Workspace API

Use this for custom editor integrations, build-time SQL linting, or simple tooling that doesn't need full LSP.

```typescript
import { createWorkspace } from "@postgres-language-server/wasm/workspace";

const workspace = await createWorkspace();

// Parse SQL and get errors
const errors = workspace.parse("SELECT * FROM users;");
console.log(errors); // []

// Split SQL into individual statements
const statements = workspace.splitStatements("SELECT 1; SELECT 2;");
// [{ sql: "SELECT 1;", start: 0, end: 9 }, { sql: "SELECT 2;", start: 10, end: 19 }]

// Insert a file and lint it
workspace.insertFile("/query.sql", "SELECT * FROM users;");
const diagnostics = workspace.lint("/query.sql");

// Get completions
const completions = workspace.complete("/query.sql", 14); // position after "FROM "

// Get hover info
const hover = workspace.hover("/query.sql", 14); // position over "users"
```

### Formatting

Format SQL in-process. Options mirror the `format` section of
`postgres-language-server.jsonc` (same names and defaults, see `FormatOptions`).
Formatting is always enabled when you call a format function, regardless of `enabled`.

```typescript
import { createWorkspace, format } from "@postgres-language-server/wasm";

// One-off: loads the WASM module on first use and returns the formatted SQL
const sql = await format('CREATE TABLE "public"."t" ("id" "uuid" NOT NULL)', {
  keywordCase: "upper",
});
// "CREATE TABLE public.t (id uuid NOT NULL);"

// With a workspace: synchronous, and also returns diagnostics
const workspace = await createWorkspace();
const result = workspace.formatSql("select 1;\n\nSELEC 2;", { lineWidth: 80 });
result.formatted; // statements that cannot be formatted are kept verbatim
result.diagnostics; // one diagnostic per statement that could not be formatted

// Format a file that was added with insertFile
workspace.insertFile("/query.sql", "select * from users");
const { formatted } = workspace.format("/query.sql", { indentStyle: "tabs" });
```

### With Schema

For schema-aware completions and hover, provide your database schema.
TypeScript types exported as `SchemaCache`:

```typescript
import type { SchemaCache } from '@postgres-language-server/wasm';

const workspace = await createWorkspace();

// Set schema
workspace.setSchema(JSON.stringify({
  schemas: [{ id: 1, name: 'public', owner: 'postgres', ... }],
  tables: [{ id: 1, schema: 'public', name: 'users', ... }],
  columns: [{ name: 'id', table_name: 'users', type_name: 'integer', ... }],
  functions: [],
  types: [],
  // ...
} satisfies SchemaCache));

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

| Method                      | Description                                 |
| --------------------------- | ------------------------------------------- |
| `parse(sql: string)`        | Parse SQL, returns array of error messages  |
| `splitStatements(sql)`      | Split SQL into statements with byte offsets |
| `insertFile(path, content)` | Add or update a file in the workspace       |
| `removeFile(path)`          | Remove a file from the workspace            |
| `lint(path)`                | Get diagnostics for a file                  |
| `format(path, options?)`    | Format a file, returns `FormatResult`       |
| `formatSql(sql, options?)`  | Format a SQL string, returns `FormatResult` |
| `complete(path, offset)`    | Get completions at position                 |
| `hover(path, offset)`       | Get hover info at position                  |
| `setSchema(json)`           | Set database schema                         |
| `clearSchema()`             | Clear the current schema                    |
| `version()`                 | Get library version                         |

The top-level `format(sql, options?)` function returns a `Promise<string>` with the formatted SQL.

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
