/**
 * Postgres Language Server WASM bindings.
 *
 * This module provides a high-level TypeScript API for using the Postgres
 * Language Server in the browser or Node.js via WebAssembly.
 *
 * @example
 * ```typescript
 * import { createWorkspace } from '@postgres-language-server/wasm';
 *
 * const workspace = await createWorkspace();
 *
 * // Insert a SQL file
 * workspace.insertFile('/query.sql', 'SELECT * FROM users;');
 *
 * // Get linting diagnostics
 * const diagnostics = workspace.lint('/query.sql');
 *
 * // Get completions at a position
 * const completions = workspace.complete('/query.sql', 14);
 * ```
 */

import type {
	Diagnostic,
	CompletionItem,
	SchemaCache,
	WorkspaceOptions,
	PGLSModule,
} from "./types";

export type {
	Diagnostic,
	CompletionItem,
	SchemaCache,
	WorkspaceOptions,
	PGLSModule,
};

// The WASM module will be loaded dynamically
let wasmModule: PGLSModule | null = null;

/**
 * Load the WASM module.
 * This is called automatically by createWorkspace, but can be called
 * manually for preloading.
 */
export async function loadWasm(): Promise<PGLSModule> {
	if (wasmModule) {
		return wasmModule;
	}

	// Dynamic import of the Emscripten-generated module
	// The actual path will depend on how the package is bundled
	const createPGLS = (await import("../wasm/pgls.js")).default;
	wasmModule = await createPGLS();

	// Initialize the workspace
	const result = wasmModule._pgls_init();
	if (result !== 0) {
		throw new Error(`Failed to initialize PGLS workspace: error code ${result}`);
	}

	return wasmModule;
}

/**
 * Helper to allocate a string in WASM memory.
 */
function allocateString(module: PGLSModule, str: string): number {
	const length = module.lengthBytesUTF8(str) + 1;
	const ptr = module._malloc(length);
	module.stringToUTF8(str, ptr, length);
	return ptr;
}

/**
 * Helper to read and free a string from WASM memory.
 */
function readAndFreeString(module: PGLSModule, ptr: number): string | null {
	if (ptr === 0) {
		return null;
	}
	const str = module.UTF8ToString(ptr);
	module._pgls_free_string(ptr);
	return str;
}

/**
 * Helper to parse a result string that may be an error.
 */
function parseResult<T>(str: string | null): T {
	if (str === null) {
		return null as T;
	}
	if (str.startsWith("ERROR:")) {
		throw new Error(str.substring(7).trim());
	}
	return JSON.parse(str) as T;
}

/**
 * The Workspace class provides a high-level API for interacting with
 * the Postgres Language Server.
 */
export class Workspace {
	private module: PGLSModule;

	constructor(module: PGLSModule) {
		this.module = module;
	}

	/**
	 * Set the database schema from a SchemaCache object or JSON string.
	 *
	 * @param schema - The schema cache object or JSON string
	 * @throws Error if the schema is invalid
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
	 *
	 * @param path - The virtual file path (e.g., "/query.sql")
	 * @param content - The file content
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
	 *
	 * @param path - The virtual file path to remove
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
	 *
	 * @param path - The virtual file path to lint
	 * @returns Array of diagnostic messages
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
	 *
	 * @param path - The virtual file path
	 * @param offset - The byte offset in the file
	 * @returns Array of completion items
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
	 *
	 * @param path - The virtual file path
	 * @param offset - The byte offset in the file
	 * @returns Hover text (markdown formatted), or null if no hover info
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
	 *
	 * @param sql - The SQL string to parse
	 * @returns Array of error messages (empty if parsing succeeded)
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
	 *
	 * @returns Version string
	 */
	version(): string {
		const resultPtr = this.module._pgls_version();
		return readAndFreeString(this.module, resultPtr) ?? "unknown";
	}
}

/**
 * Create a new Workspace instance.
 *
 * This loads the WASM module if not already loaded and initializes
 * a new workspace.
 *
 * @param options - Optional configuration
 * @returns A new Workspace instance
 *
 * @example
 * ```typescript
 * const workspace = await createWorkspace();
 * workspace.insertFile('/query.sql', 'SELECT * FROM users;');
 * const diagnostics = workspace.lint('/query.sql');
 * ```
 */
export async function createWorkspace(
	options?: WorkspaceOptions,
): Promise<Workspace> {
	const module = await loadWasm();
	const workspace = new Workspace(module);

	if (options?.schema) {
		workspace.setSchema(options.schema);
	}

	return workspace;
}

// Default export for convenience
export default createWorkspace;
