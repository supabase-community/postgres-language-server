/**
 * Web Worker for the Postgres Language Server.
 *
 * This worker handles LSP JSON-RPC messages and communicates with the main thread.
 */

let lsp = null;
let initialized = false;

// Import the LSP module
async function init() {
	try {
		const { createLanguageServer } = await import("/dist/lsp.js");
		lsp = await createLanguageServer();
		initialized = true;
		self.postMessage({ type: "ready" });
	} catch (err) {
		self.postMessage({ type: "error", message: err.message });
	}
}

// Handle messages from main thread
self.onmessage = async (event) => {
	const { type, message, id } = event.data;

	if (type === "init") {
		await init();
		return;
	}

	if (type === "message" && lsp) {
		try {
			const responses = lsp.handleMessage(message);
			// Send all responses back
			for (const response of responses) {
				self.postMessage({ type: "response", response, requestId: id });
			}
		} catch (err) {
			self.postMessage({
				type: "error",
				message: err.message,
				requestId: id,
			});
		}
	}
};
