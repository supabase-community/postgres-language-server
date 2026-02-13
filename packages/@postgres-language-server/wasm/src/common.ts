/**
 * Common utilities shared between Workspace and LanguageServer APIs.
 */

import type { PGLSModule } from "./types";

// The WASM module will be loaded dynamically
let wasmModule: PGLSModule | null = null;

/**
 * Detect if we're running in Node.js/Bun (vs browser)
 */
function isNode(): boolean {
	return (
		typeof process !== "undefined" &&
		process.versions != null &&
		(process.versions.node != null || process.versions.bun != null)
	);
}

/**
 * Load the WASM module.
 * This is called automatically by createWorkspace/createLanguageServer,
 * but can be called manually for preloading.
 */
export async function loadWasm(): Promise<PGLSModule> {
	if (wasmModule) {
		return wasmModule;
	}

	// Dynamic import of the Emscripten-generated module
	// @ts-expect-error - Generated JS file without type declarations
	const createPGLS = (await import("../wasm/pgls.js")).default as (
		options?: object,
	) => Promise<PGLSModule>;

	// Build options for Emscripten module initialization
	const moduleOptions: Record<string, unknown> = {};

	if (isNode()) {
		// In Node.js/Bun, read the WASM file directly
		const { readFileSync } = await import("node:fs");
		const { fileURLToPath } = await import("node:url");
		const { dirname, join } = await import("node:path");

		const __filename = fileURLToPath(import.meta.url);
		const __dirname = dirname(__filename);
		const wasmPath = join(__dirname, "..", "wasm", "pgls.wasm");

		moduleOptions.wasmBinary = readFileSync(wasmPath);
	} else {
		// In browser, use locateFile to help find the .wasm file
		moduleOptions.locateFile = (path: string) => {
			if (path.endsWith(".wasm")) {
				return new URL("./pgls.wasm", import.meta.url).href;
			}
			return path;
		};
	}

	// Initialize the Emscripten module
	const module = await createPGLS(moduleOptions);

	// Initialize the workspace
	const result = module._pgls_init();
	if (result !== 0) {
		throw new Error(`Failed to initialize PGLS: error code ${result}`);
	}

	wasmModule = module;
	return module;
}

/**
 * Allocate a string in WASM memory.
 */
export function allocateString(module: PGLSModule, str: string): number {
	const length = module.lengthBytesUTF8(str) + 1;
	const ptr = module._malloc(length);
	module.stringToUTF8(str, ptr, length);
	return ptr;
}

/**
 * Read and free a string from WASM memory.
 */
export function readAndFreeString(
	module: PGLSModule,
	ptr: number,
): string | null {
	if (ptr === 0) {
		return null;
	}
	const str = module.UTF8ToString(ptr);
	module._pgls_free_string(ptr);
	return str;
}

/**
 * Parse a result string that may be an error.
 */
export function parseResult<T>(str: string | null): T {
	if (str === null) {
		return null as T;
	}
	if (str.startsWith("ERROR:")) {
		throw new Error(str.substring(7).trim());
	}
	return JSON.parse(str) as T;
}
