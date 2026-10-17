/**
 * TypeScript type definitions for the Postgres Language Server WASM API.
 */

import type { Snapshot } from "./snapshot";

/**
 * A diagnostic message from the linter.
 */
export interface Diagnostic {
  /** The category/rule name (e.g., "lint/safety/banDropColumn") */
  category: string;
  /** Start byte offset in the file */
  start: number;
  /** End byte offset in the file */
  end: number;
  /** The diagnostic message */
  message: string;
  /** Severity: "error", "warning", "info", "hint", or "fatal" */
  severity: "error" | "warning" | "info" | "hint" | "fatal";
}

/**
 * A completion item suggestion.
 */
export interface CompletionItem {
  /** The label of the completion item */
  label: string;
  /** The kind of completion (e.g., "table", "column", "function") */
  kind: string;
  /** Optional detail text */
  detail?: string;
  /** Optional documentation */
  documentation?: string;
  /** The text to insert */
  insertText?: string;
}

/**
 * Database snapshot type generated from Rust `Snapshot` via schemars.
 */
export type { Snapshot } from "./snapshot";

/**
 * Options for initializing the workspace.
 */
export interface WorkspaceOptions {
  /** Optional database snapshot to preload */
  schema?: Snapshot | string;
}

/**
 * A single SQL statement extracted from a multi-statement source.
 */
export interface Statement {
  /** The SQL text of the statement */
  sql: string;
  /** Start byte offset in the original SQL string */
  start: number;
  /** End byte offset in the original SQL string */
  end: number;
}

/**
 * Formatter options.
 *
 * Mirrors the `format` section of `postgres-language-server.jsonc` (same names
 * and defaults). Omitted options use their defaults. Formatting is always
 * enabled when calling a format function explicitly.
 */
export interface FormatOptions {
  /** Maximum line width before breaking. Default: 100. */
  lineWidth?: number;
  /** Number of spaces (or tab width) for indentation. Default: 2. */
  indentSize?: number;
  /** Indentation style. Default: "spaces". */
  indentStyle?: "spaces" | "tabs";
  /** Keyword casing. Default: "lower". */
  keywordCase?: "upper" | "lower";
  /** Constant casing (NULL, TRUE, FALSE). Default: "lower". */
  constantCase?: "upper" | "lower";
  /** Data type casing (text, varchar, int). Default: "lower". */
  typeCase?: "upper" | "lower";
  /** Where a comma sits when a list breaks. Default: "trailing". */
  commaStyle?: "trailing" | "leading";
  /** Where a boolean operator sits when a condition breaks. Default: "trailing". */
  logicalOperatorPlacement?: "trailing" | "leading";
  /**
   * How a statement is laid out: "fit" breaks only when a line would exceed the
   * line width, "expanded" always breaks between clauses. Default: "fit".
   */
  layout?: "fit" | "expanded";
  /** How an explicit cast is spelled: `CAST(x AS t)` or `x::t`. Default: "cast". */
  castStyle?: "cast" | "operator";
  /**
   * Where the body of a clause starts: "break" for a new line, "compact" to keep
   * the first element on the keyword line. Default: "break".
   */
  clauseBodyStyle?: "break" | "compact";
  /**
   * If `true`, the terminating semicolon goes on its own line when the statement
   * spans several lines. Default: `false`.
   */
  isolateSemicolon?: boolean;
  /**
   * Function names mapped to the number of adjacent arguments in one logical group
   * (1-255). `json_build_object` and `jsonb_build_object` are grouped in pairs by
   * default; configure a size of `1` to opt out.
   */
  functionArgumentGroups?: Record<string, number>;
  /** If `true`, keep SQL function bodies verbatim. Default: `false`. */
  skipFnBodies?: boolean;
}

/**
 * The result of formatting SQL.
 */
export interface FormatResult {
  /** The formatted SQL. Statements that could not be formatted are kept verbatim. */
  formatted: string;
  /** Diagnostics for statements that could not be formatted. */
  diagnostics: Diagnostic[];
}

/**
 * A JSON-RPC message (request, response, or notification).
 * This is the standard LSP message format.
 */
export interface JsonRpcMessage {
  jsonrpc: "2.0";
  id?: number | string | null;
  method?: string;
  params?: unknown;
  result?: unknown;
  error?: {
    code: number;
    message: string;
    data?: unknown;
  };
}

/**
 * The Emscripten module interface.
 * This is the raw interface exposed by the compiled WASM.
 */
export interface PGLSModule {
  // Memory management
  _malloc(size: number): number;
  _free(ptr: number): void;

  // FFI functions
  _pgls_init(): number;
  _pgls_free_string(ptr: number): void;
  _pgls_set_schema(jsonPtr: number): number;
  _pgls_clear_schema(): void;
  _pgls_insert_file(pathPtr: number, contentPtr: number): number;
  _pgls_remove_file(pathPtr: number): void;
  _pgls_lint(pathPtr: number): number;
  _pgls_format(pathPtr: number, optionsPtr: number): number;
  _pgls_format_sql(sqlPtr: number, optionsPtr: number): number;
  _pgls_complete(pathPtr: number, offset: number): number;
  _pgls_hover(pathPtr: number, offset: number): number;
  _pgls_parse(sqlPtr: number): number;
  _pgls_split_statements(sqlPtr: number): number;
  _pgls_version(): number;

  // Language Server API
  _pgls_lsp_handle_message(messagePtr: number): number;

  // Emscripten runtime methods
  UTF8ToString(ptr: number): string;
  stringToUTF8(str: string, ptr: number, maxLength: number): void;
  lengthBytesUTF8(str: string): number;
}
