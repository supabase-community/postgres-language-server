/**
 * Web Worker for running the Postgres Language Server in the browser.
 *
 * This worker implements the LSP message handling pattern compatible with
 * monaco-languageclient's BrowserMessageReader/BrowserMessageWriter.
 *
 * @example
 * ```typescript
 * // In your main application:
 * import { MonacoLanguageClient } from 'monaco-languageclient';
 * import { BrowserMessageReader, BrowserMessageWriter } from 'vscode-languageserver-protocol/browser';
 *
 * // Create worker
 * const worker = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
 *
 * // Wait for worker to be ready
 * await new Promise<void>((resolve) => {
 *   worker.onmessage = (e) => {
 *     if (e.data.type === 'ready') resolve();
 *   };
 * });
 *
 * // Standard monaco-languageclient setup
 * const reader = new BrowserMessageReader(worker);
 * const writer = new BrowserMessageWriter(worker);
 *
 * const languageClient = new MonacoLanguageClient({
 *   name: 'PGLS',
 *   clientOptions: {
 *     documentSelector: [{ language: 'sql' }],
 *   },
 *   messageTransports: { reader, writer },
 * });
 *
 * languageClient.start();
 * ```
 */

import { createWorkspace, type Workspace, type JsonRpcMessage } from "./index";

let workspace: Workspace | null = null;

/**
 * Initialize the workspace.
 */
async function initialize(): Promise<void> {
	if (!workspace) {
		workspace = await createWorkspace();
	}
}

/**
 * Handle incoming messages from the main thread.
 */
self.onmessage = async (event: MessageEvent) => {
	// Ensure workspace is initialized
	if (!workspace) {
		await initialize();
	}

	const data = event.data;

	// Handle special control messages
	if (typeof data === "object" && data.type === "setSchema") {
		workspace!.setSchema(data.schema);
		return;
	}

	// Handle LSP JSON-RPC messages
	// The message can be a string (raw JSON) or an object
	const message: string | JsonRpcMessage = data;

	// Process the message and get array of outgoing messages
	const outgoing = workspace!.handleMessage(message);

	// Send EACH message separately via postMessage
	// This is required by BrowserMessageReader which expects
	// individual messages, not arrays
	for (const msg of outgoing) {
		self.postMessage(msg);
	}
};

// Initialize immediately and signal readiness
initialize().then(() => {
	self.postMessage({ type: "ready" });
});

// Export for type checking (worker doesn't actually export)
export type {};
