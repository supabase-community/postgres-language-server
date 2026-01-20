/**
 * TypeScript type definitions for the Postgres Language Server WASM API.
 */

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
 * Schema cache representing database metadata.
 * This structure mirrors the Rust SchemaCache type.
 */
export interface SchemaCache {
	schemas: Schema[];
	tables: Table[];
	functions: Function[];
	types: Type[];
	// Add more as needed based on pgls_schema_cache
}

export interface Schema {
	name: string;
	owner?: string;
}

export interface Table {
	name: string;
	schema: string;
	columns: Column[];
}

export interface Column {
	name: string;
	dataType: string;
	isNullable: boolean;
	defaultValue?: string;
}

export interface Function {
	name: string;
	schema: string;
	returnType: string;
	arguments: string[];
}

export interface Type {
	name: string;
	schema: string;
	kind: string;
}

/**
 * Options for initializing the workspace.
 */
export interface WorkspaceOptions {
	/** Optional schema cache to preload */
	schema?: SchemaCache | string;
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
	_pgls_complete(pathPtr: number, offset: number): number;
	_pgls_hover(pathPtr: number, offset: number): number;
	_pgls_parse(sqlPtr: number): number;
	_pgls_version(): number;

	// Emscripten runtime methods
	UTF8ToString(ptr: number): string;
	stringToUTF8(str: string, ptr: number, maxLength: number): void;
	lengthBytesUTF8(str: string): number;
}
