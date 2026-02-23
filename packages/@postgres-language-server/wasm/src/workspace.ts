/**
 * Workspace API - Direct access to parse, lint, complete, hover.
 *
 * Use this for:
 * - Custom editor integrations
 * - Build-time SQL linting
 * - Simple tooling that doesn't need full LSP
 *
 * @example
 * ```typescript
 * import { createWorkspace } from '@postgres-language-server/wasm/workspace';
 *
 * const workspace = await createWorkspace();
 * workspace.insertFile('/query.sql', 'SELECT * FROM users;');
 * const diagnostics = workspace.lint('/query.sql');
 * ```
 */

import { allocateString, loadWasm, parseResult, readAndFreeString } from "./common.js";
import type {
  CompletionItem,
  Diagnostic,
  PGLSModule,
  SchemaCache,
  WorkspaceOptions,
} from "./types.js";

export type { Diagnostic, CompletionItem, SchemaCache, WorkspaceOptions, PGLSModule };

/**
 * The Workspace class provides a direct API for SQL parsing, linting,
 * completions, and hover information.
 */
export class Workspace {
  private module: PGLSModule;

  constructor(module: PGLSModule) {
    this.module = module;
  }

  /**
   * Set the database schema from a SchemaCache object or JSON string.
   */
  setSchema(schema: SchemaCache | string): void {
    const json = typeof schema === "string" ? schema : JSON.stringify(schema);
    const jsonPtr = allocateString(this.module, json);
    try {
      const resultPtr = this.module._pgls_set_schema(jsonPtr);
      const result = readAndFreeString(this.module, resultPtr);
      if (result !== null) {
        throw new Error(result);
      }
    } finally {
      this.module._free(jsonPtr);
    }
  }

  /**
   * Clear the current schema.
   */
  clearSchema(): void {
    this.module._pgls_clear_schema();
  }

  /**
   * Insert or update a file in the workspace.
   */
  insertFile(path: string, content: string): void {
    const pathPtr = allocateString(this.module, path);
    const contentPtr = allocateString(this.module, content);
    try {
      const resultPtr = this.module._pgls_insert_file(pathPtr, contentPtr);
      const result = readAndFreeString(this.module, resultPtr);
      if (result !== null) {
        throw new Error(result);
      }
    } finally {
      this.module._free(pathPtr);
      this.module._free(contentPtr);
    }
  }

  /**
   * Remove a file from the workspace.
   */
  removeFile(path: string): void {
    const pathPtr = allocateString(this.module, path);
    try {
      this.module._pgls_remove_file(pathPtr);
    } finally {
      this.module._free(pathPtr);
    }
  }

  /**
   * Lint a file and return diagnostics.
   */
  lint(path: string): Diagnostic[] {
    const pathPtr = allocateString(this.module, path);
    try {
      const resultPtr = this.module._pgls_lint(pathPtr);
      const result = readAndFreeString(this.module, resultPtr);
      return parseResult<Diagnostic[]>(result) ?? [];
    } finally {
      this.module._free(pathPtr);
    }
  }

  /**
   * Get completions at a position in a file.
   */
  complete(path: string, offset: number): CompletionItem[] {
    const pathPtr = allocateString(this.module, path);
    try {
      const resultPtr = this.module._pgls_complete(pathPtr, offset);
      const result = readAndFreeString(this.module, resultPtr);
      return parseResult<CompletionItem[]>(result) ?? [];
    } finally {
      this.module._free(pathPtr);
    }
  }

  /**
   * Get hover information at a position in a file.
   */
  hover(path: string, offset: number): string | null {
    const pathPtr = allocateString(this.module, path);
    try {
      const resultPtr = this.module._pgls_hover(pathPtr, offset);
      const result = readAndFreeString(this.module, resultPtr);
      if (result === null) {
        return null;
      }
      if (result.startsWith("ERROR:")) {
        throw new Error(result.substring(7).trim());
      }
      return result;
    } finally {
      this.module._free(pathPtr);
    }
  }

  /**
   * Parse SQL and return any parse errors.
   */
  parse(sql: string): string[] {
    const sqlPtr = allocateString(this.module, sql);
    try {
      const resultPtr = this.module._pgls_parse(sqlPtr);
      const result = readAndFreeString(this.module, resultPtr);
      return parseResult<string[]>(result) ?? [];
    } finally {
      this.module._free(sqlPtr);
    }
  }

  /**
   * Get the version of the library.
   */
  version(): string {
    const resultPtr = this.module._pgls_version();
    return readAndFreeString(this.module, resultPtr) ?? "unknown";
  }
}

/**
 * Create a new Workspace instance.
 */
export async function createWorkspace(options?: WorkspaceOptions): Promise<Workspace> {
  const module = await loadWasm();
  const workspace = new Workspace(module);

  if (options?.schema) {
    workspace.setSchema(options.schema);
  }

  return workspace;
}

export default createWorkspace;
