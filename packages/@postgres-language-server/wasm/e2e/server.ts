/**
 * Simple HTTP server for E2E tests.
 * Serves the test HTML page and WASM files.
 */

import { existsSync, readFileSync } from "node:fs";
import { extname, join } from "node:path";

const PORT = 3000;
const ROOT = join(import.meta.dir, "..");

const MIME_TYPES: Record<string, string> = {
	".html": "text/html",
	".js": "application/javascript",
	".mjs": "application/javascript",
	".ts": "application/javascript",
	".css": "text/css",
	".wasm": "application/wasm",
	".json": "application/json",
};

function getMimeType(path: string): string {
	const ext = extname(path);
	return MIME_TYPES[ext] || "application/octet-stream";
}

const server = Bun.serve({
	port: PORT,
	fetch(req) {
		const url = new URL(req.url);
		let pathname = url.pathname;

		// Default to index.html
		if (pathname === "/") {
			pathname = "/e2e/index.html";
		}

		// Map paths
		let filePath: string;
		if (pathname.startsWith("/wasm/")) {
			filePath = join(ROOT, pathname);
		} else if (pathname.startsWith("/dist/")) {
			filePath = join(ROOT, pathname);
		} else if (pathname.startsWith("/e2e/")) {
			filePath = join(ROOT, pathname);
		} else {
			filePath = join(ROOT, "e2e", pathname);
		}

		if (!existsSync(filePath)) {
			console.log(`404: ${pathname} (${filePath})`);
			return new Response("Not Found", { status: 404 });
		}

		const content = readFileSync(filePath);
		const mimeType = getMimeType(filePath);

		console.log(`200: ${pathname}`);
		return new Response(content, {
			headers: {
				"Content-Type": mimeType,
				"Cross-Origin-Opener-Policy": "same-origin",
				"Cross-Origin-Embedder-Policy": "require-corp",
			},
		});
	},
});

console.log(`E2E test server running at http://localhost:${PORT}`);
