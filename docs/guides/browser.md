# Use in the browser

The [`@postgres-language-server/wasm`](https://www.npmjs.com/package/@postgres-language-server/wasm) package compiles the language server to WebAssembly, so it runs entirely in the browser, without a server process. It provides two APIs:

| API | Use case | Import path |
| --- | --- | --- |
| Workspace | Call parse, lint, format, complete and hover directly | `@postgres-language-server/wasm/workspace` |
| LanguageServer | Handle LSP JSON-RPC messages, for editors with an LSP client | `@postgres-language-server/wasm/lsp` |

Each API manages its own workspace. Use one or the other, not both. This guide uses the LanguageServer API to connect an editor. The [package README](https://github.com/supabase-community/postgres-language-server/tree/main/packages/@postgres-language-server/wasm) documents the Workspace API.

```bash
npm install @postgres-language-server/wasm
```

The browser build cannot connect to a database. Schema-aware features, such as completing table and column names, use a schema that you export once and pass to the language server (see [Providing the database schema](#providing-the-database-schema)).

The LanguageServer API supports these LSP methods:

- `initialize`, `initialized`, `shutdown`
- `textDocument/didOpen`, `textDocument/didChange` (full document sync), `textDocument/didClose`
- `textDocument/completion`
- `textDocument/hover`
- `textDocument/publishDiagnostics`, sent after a document is opened or changed, and for all open documents after the schema is set
- `pgls/setSchema` and `pgls/clearSchema` (custom notifications)

For formatting, use the `format` function of the package (see [Formatting](../features/formatting.md#javascript-api)).

## Run the language server in a web worker

Run the language server in a web worker, so that parsing and linting do not block the editor. `handleMessage` takes one JSON-RPC message, as an object or a string, and returns the outgoing messages: the response, if any, and notifications such as `textDocument/publishDiagnostics`. Post each of them back separately:

```typescript
// pgls-worker.ts
import { createLanguageServer } from "@postgres-language-server/wasm/lsp";

const languageServer = createLanguageServer();

self.onmessage = async (event: MessageEvent) => {
  const lsp = await languageServer;
  for (const message of lsp.handleMessage(event.data)) {
    self.postMessage(message);
  }
};

// Signal the main thread that the WASM module is loaded
languageServer.then(() => self.postMessage({ type: "ready" }));
```

In the browser, the package loads `pgls.wasm` from `new URL("./pgls.wasm", import.meta.url)`, next to its own modules. Make sure your bundler or server serves the file there.

## Monaco

[`monaco-languageclient`](https://github.com/TypeFox/monaco-languageclient) can talk to the worker through `BrowserMessageReader` and `BrowserMessageWriter`, which exchange one message per `postMessage`:

```typescript
import { MonacoLanguageClient } from "monaco-languageclient";
import {
  BrowserMessageReader,
  BrowserMessageWriter,
} from "vscode-languageserver-protocol/browser";

const worker = new Worker(new URL("./pgls-worker.ts", import.meta.url), { type: "module" });

// Wait until the WASM module is loaded
await new Promise<void>((resolve) => {
  worker.onmessage = (event) => {
    if (event.data.type === "ready") resolve();
  };
});

const reader = new BrowserMessageReader(worker);
const writer = new BrowserMessageWriter(worker);

const languageClient = new MonacoLanguageClient({
  name: "Postgres Language Server",
  clientOptions: {
    documentSelector: [{ language: "sql" }],
  },
  messageTransports: { reader, writer },
});

languageClient.start();
```

How Monaco and its services are initialised depends on the version of `monaco-languageclient`; follow its documentation for that part.

## CodeMirror

Any LSP client for CodeMirror works with the same worker. [`@codemirror/lsp-client`](https://codemirror.net/docs/ref/#lsp-client) talks to the server through a `Transport` that sends and receives messages as strings:

```typescript
import { LSPClient, languageServerExtensions, type Transport } from "@codemirror/lsp-client";
import { basicSetup, EditorView } from "codemirror";
import { sql } from "@codemirror/lang-sql";

function workerTransport(worker: Worker): Transport {
  let handlers: ((message: string) => void)[] = [];
  worker.addEventListener("message", (event) => {
    // Skip the ready signal, forward the JSON-RPC messages
    if (event.data.jsonrpc !== "2.0") return;
    const message = JSON.stringify(event.data);
    for (const handler of handlers) handler(message);
  });
  return {
    send(message) {
      worker.postMessage(message);
    },
    subscribe(handler) {
      handlers.push(handler);
    },
    unsubscribe(handler) {
      handlers = handlers.filter((h) => h !== handler);
    },
  };
}

const worker = new Worker(new URL("./pgls-worker.ts", import.meta.url), { type: "module" });
const client = new LSPClient({ extensions: languageServerExtensions() }).connect(
  workerTransport(worker),
);

new EditorView({
  extensions: [basicSetup, sql(), client.plugin("file:///query.sql")],
  parent: document.body,
});
```

## Providing the database schema

Export the schema of your database to JSON with the CLI:

```bash
postgres-language-server schema-export \
  --connection-string postgres://postgres:postgres@localhost:5432/postgres \
  --output schema.json
```

Then send its content to the language server with the `pgls/setSchema` notification. The `schema` parameter is the JSON as a string:

```typescript
const schema = await fetch("/schema.json").then((response) => response.text());

// monaco-languageclient
await languageClient.sendNotification("pgls/setSchema", { schema });

// @codemirror/lsp-client
client.notification("pgls/setSchema", { schema });
```

The `pgls/clearSchema` notification removes the schema again. The TypeScript type of the schema is exported as `SchemaCache` from `@postgres-language-server/wasm`.
