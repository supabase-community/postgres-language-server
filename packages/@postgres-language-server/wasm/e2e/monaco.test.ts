import { test, expect } from "@playwright/test";

/**
 * E2E tests for Monaco Editor with PGLS WASM.
 *
 * Prerequisites:
 * 1. Have Emscripten in PATH (source emsdk_env.sh)
 * 2. Build everything: bun run build
 * 3. Install Playwright: bun run test:e2e:install
 * 4. Run tests: bun run test:e2e
 */
test.describe("Monaco Editor with PGLS WASM", () => {
	test.beforeEach(async ({ page }) => {
		await page.goto("/");

		// Wait for the editor to be ready (or error)
		await page.waitForFunction(
			() => {
				const status = document.getElementById("status");
				return (
					status?.classList.contains("ready") ||
					status?.classList.contains("error")
				);
			},
			{ timeout: 30000 },
		);

		// Check if WASM loaded successfully
		const hasError = await page.locator("#status").evaluate((el) =>
			el.classList.contains("error"),
		);

		if (hasError) {
			const errorText = await page.locator("#status").textContent();
			// Skip test if WASM isn't built with required exports
			if (errorText?.includes("_pgls_handle_message is not a function")) {
				test.skip(
					true,
					"WASM needs to be rebuilt with handleMessage export. Run: cd crates/pgls_wasm && ./build-wasm.sh",
				);
			} else if (errorText?.includes("expected magic")) {
				test.skip(true, "WASM file is invalid or missing. Run build-wasm.sh");
			} else {
				throw new Error(`WASM failed to load: ${errorText}`);
			}
		}
	});

	test("loads Monaco editor and WASM module", async ({ page }) => {
		// Verify Monaco editor is loaded
		const editor = page.locator(".monaco-editor");
		await expect(editor).toBeVisible();

		// Verify status shows ready
		await expect(page.locator("#status")).toHaveText("Ready");

		// Verify workspace is available
		const hasWorkspace = await page.evaluate(() => {
			return typeof (window as any).pglsWorkspace !== "undefined";
		});
		expect(hasWorkspace).toBe(true);
	});

	test("shows no diagnostics for valid SQL", async ({ page }) => {
		// Clear editor and type valid SQL
		await page.evaluate(() => {
			(window as any).monacoEditor.setValue("SELECT 1;");
		});

		// Wait for diagnostics to update
		await page.waitForTimeout(500);

		// Check diagnostics panel
		const diagnosticsEl = page.locator('[data-testid="diagnostics"]');
		await expect(diagnosticsEl).toContainText("No diagnostics");
	});

	test("shows diagnostics for invalid SQL", async ({ page }) => {
		// Type invalid SQL
		await page.evaluate(() => {
			(window as any).monacoEditor.setValue("SELEC * FROM users;");
		});

		// Wait for diagnostics to update
		await page.waitForTimeout(500);

		// Check that diagnostics appear
		const diagnostics = await page.evaluate(() => {
			return (window as any).lastDiagnostics;
		});

		expect(diagnostics).toBeDefined();
		expect(diagnostics.length).toBeGreaterThan(0);

		// Check diagnostics panel shows error
		const diagnosticsEl = page.locator('[data-testid="diagnostics"]');
		await expect(diagnosticsEl).toContainText("error");
	});

	test("updates diagnostics on content change", async ({ page }) => {
		// Start with invalid SQL
		await page.evaluate(() => {
			(window as any).monacoEditor.setValue("SELEC 1;");
		});

		await page.waitForTimeout(500);

		// Verify error appears
		let diagnostics = await page.evaluate(() => {
			return (window as any).lastDiagnostics;
		});
		expect(diagnostics.length).toBeGreaterThan(0);

		// Fix the SQL
		await page.evaluate(() => {
			(window as any).monacoEditor.setValue("SELECT 1;");
		});

		await page.waitForTimeout(500);

		// Verify error is gone
		diagnostics = await page.evaluate(() => {
			return (window as any).lastDiagnostics;
		});
		expect(diagnostics.length).toBe(0);
	});

	test("handleMessage returns initialize response", async ({ page }) => {
		const response = await page.evaluate(() => {
			const workspace = (window as any).pglsWorkspace;
			return workspace.handleMessage({
				jsonrpc: "2.0",
				id: 1,
				method: "initialize",
				params: { capabilities: {} },
			});
		});

		expect(response).toHaveLength(1);
		expect(response[0].jsonrpc).toBe("2.0");
		expect(response[0].id).toBe(1);
		expect(response[0].result).toBeDefined();
		expect(response[0].result.capabilities).toBeDefined();
	});

	test("completion request returns results", async ({ page }) => {
		// Open a document first
		await page.evaluate(() => {
			const workspace = (window as any).pglsWorkspace;
			workspace.handleMessage({
				jsonrpc: "2.0",
				method: "textDocument/didOpen",
				params: {
					textDocument: {
						uri: "file:///completion-test.sql",
						languageId: "sql",
						version: 1,
						text: "SELECT ",
					},
				},
			});
		});

		// Request completions
		const response = await page.evaluate(() => {
			const workspace = (window as any).pglsWorkspace;
			return workspace.handleMessage({
				jsonrpc: "2.0",
				id: 2,
				method: "textDocument/completion",
				params: {
					textDocument: { uri: "file:///completion-test.sql" },
					position: { line: 0, character: 7 },
				},
			});
		});

		expect(response).toHaveLength(1);
		expect(response[0].id).toBe(2);
		expect(response[0].result).toBeDefined();
		// Result should be an array (possibly empty without schema)
		expect(Array.isArray(response[0].result)).toBe(true);
	});

	test("hover request works", async ({ page }) => {
		// Open a document first
		await page.evaluate(() => {
			const workspace = (window as any).pglsWorkspace;
			workspace.handleMessage({
				jsonrpc: "2.0",
				method: "textDocument/didOpen",
				params: {
					textDocument: {
						uri: "file:///hover-test.sql",
						languageId: "sql",
						version: 1,
						text: "SELECT * FROM users;",
					},
				},
			});
		});

		// Request hover
		const response = await page.evaluate(() => {
			const workspace = (window as any).pglsWorkspace;
			return workspace.handleMessage({
				jsonrpc: "2.0",
				id: 3,
				method: "textDocument/hover",
				params: {
					textDocument: { uri: "file:///hover-test.sql" },
					position: { line: 0, character: 14 },
				},
			});
		});

		expect(response).toHaveLength(1);
		expect(response[0].id).toBe(3);
		// Response should have either result (null or hover content) or error
		// Without schema, hover typically returns null result
		expect(response[0].jsonrpc).toBe("2.0");
	});

	test("didChange triggers publishDiagnostics", async ({ page }) => {
		// Open a document
		await page.evaluate(() => {
			const workspace = (window as any).pglsWorkspace;
			workspace.handleMessage({
				jsonrpc: "2.0",
				method: "textDocument/didOpen",
				params: {
					textDocument: {
						uri: "file:///change-test.sql",
						languageId: "sql",
						version: 1,
						text: "SELECT 1;",
					},
				},
			});
		});

		// Change the document to have an error
		const response = await page.evaluate(() => {
			const workspace = (window as any).pglsWorkspace;
			return workspace.handleMessage({
				jsonrpc: "2.0",
				method: "textDocument/didChange",
				params: {
					textDocument: {
						uri: "file:///change-test.sql",
						version: 2,
					},
					contentChanges: [{ text: "SELEC 1;" }],
				},
			});
		});

		// Should return publishDiagnostics notification
		expect(response.length).toBeGreaterThanOrEqual(1);
		const diagNotification = response.find(
			(m: any) => m.method === "textDocument/publishDiagnostics",
		);
		expect(diagNotification).toBeDefined();
		expect(diagNotification.params.diagnostics.length).toBeGreaterThan(0);
	});
});
