/**
 * Postgres Language Server WASM bindings.
 *
 * This package provides two independent APIs:
 *
 * 1. **Workspace API** - Direct access to parse, lint, complete, hover
 *    ```ts
 *    import { createWorkspace } from '@postgres-language-server/wasm/workspace';
 *    ```
 *
 * 2. **Language Server API** - Full LSP JSON-RPC message handling
 *    ```ts
 *    import { createLanguageServer } from '@postgres-language-server/wasm/lsp';
 *    ```
 *
 * Each API manages its own workspace independently. Choose one based on your use case.
 */

// Re-export everything from both APIs
export * from "./workspace.js";
export * from "./lsp.js";
export { loadWasm } from "./common.js";

// Re-export types
export type {
  Diagnostic,
  CompletionItem,
  SchemaCache,
  WorkspaceOptions,
  PGLSModule,
  JsonRpcMessage,
} from "./types.js";

// Default export for convenience
export { createWorkspace as default } from "./workspace.js";
