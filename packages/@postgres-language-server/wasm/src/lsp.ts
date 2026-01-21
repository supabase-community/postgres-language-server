/**
 * Language Server API - Full LSP JSON-RPC message handling.
 *
 * Use this for:
 * - Monaco editor with monaco-languageclient
 * - Any editor that speaks LSP protocol
 *
 * The language server manages its own workspace internally.
 * Use the `pgls/setSchema` notification to set the database schema:
 * ```json
 * {"jsonrpc":"2.0","method":"pgls/setSchema","params":{"schema":"..."}}
 * ```
 *
 * @example
 * ```typescript
 * import { createLanguageServer } from '@postgres-language-server/wasm/lsp';
 *
 * const lsp = await createLanguageServer();
 *
 * // In a web worker
 * self.onmessage = (event) => {
 *   const responses = lsp.handleMessage(event.data);
 *   for (const msg of responses) {
 *     self.postMessage(msg);
 *   }
 * };
 * ```
 */

import type { PGLSModule, JsonRpcMessage } from "./types.js";

import { loadWasm, allocateString, readAndFreeString } from "./common.js";

export type { PGLSModule, JsonRpcMessage };

/**
 * The LanguageServer class provides a full LSP JSON-RPC message handler.
 */
export class LanguageServer {
	private module: PGLSModule;

	constructor(module: PGLSModule) {
		this.module = module;
	}

	/**
	 * Handle an LSP JSON-RPC message.
	 *
	 * Processes an incoming LSP message and returns an array of outgoing
	 * messages (response + any notifications like publishDiagnostics).
	 *
	 * @param message - The JSON-RPC message as a string or object
	 * @returns Array of outgoing JSON-RPC messages
	 */
	handleMessage(message: string | JsonRpcMessage): JsonRpcMessage[] {
		const messageStr =
			typeof message === "string" ? message : JSON.stringify(message);
		const messagePtr = allocateString(this.module, messageStr);
		try {
			const resultPtr = this.module._pgls_lsp_handle_message(messagePtr);
			const result = readAndFreeString(this.module, resultPtr) ?? "[]";
			return JSON.parse(result) as JsonRpcMessage[];
		} finally {
			this.module._free(messagePtr);
		}
	}
}

/**
 * Create a new LanguageServer instance.
 */
export async function createLanguageServer(): Promise<LanguageServer> {
	const module = await loadWasm();
	return new LanguageServer(module);
}

export default createLanguageServer;
