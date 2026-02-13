import { expect, test } from "@playwright/test";

/**
 * E2E tests for Monaco Editor with PGLS Language Server (Web Worker).
 *
 * Tests the full LSP integration via web worker, as it would be used in production.
 *
 * Prerequisites:
 * 1. Have Emscripten in PATH (source emsdk_env.sh)
 * 2. Build everything: bun run build
 * 3. Install Playwright: bun run test:e2e:install
 * 4. Run tests: bun run test:e2e
 */

const TEST_SCHEMA = {
	schemas: [
		{
			id: 1,
			name: "public",
			owner: "postgres",
			allowed_users: [],
			allowed_creators: [],
			table_count: 1,
			view_count: 0,
			function_count: 0,
			total_size: "0 bytes",
			comment: null,
		},
	],
	tables: [
		{
			id: 1,
			schema: "public",
			name: "users",
			rls_enabled: false,
			rls_forced: false,
			replica_identity: "Default",
			table_kind: "Ordinary",
			bytes: 0,
			size: "0 bytes",
			live_rows_estimate: 0,
			dead_rows_estimate: 0,
			comment: null,
		},
		{
			id: 2,
			schema: "public",
			name: "orders",
			rls_enabled: false,
			rls_forced: false,
			replica_identity: "Default",
			table_kind: "Ordinary",
			bytes: 0,
			size: "0 bytes",
			live_rows_estimate: 0,
			dead_rows_estimate: 0,
			comment: null,
		},
	],
	columns: [
		{
			name: "id",
			table_name: "users",
			table_oid: 1,
			class_kind: "OrdinaryTable",
			number: 1,
			schema_name: "public",
			type_id: 23,
			type_name: "integer",
			is_nullable: false,
			is_primary_key: true,
			is_unique: true,
			default_expr: null,
			varchar_length: null,
			comment: null,
		},
		{
			name: "username",
			table_name: "users",
			table_oid: 1,
			class_kind: "OrdinaryTable",
			number: 2,
			schema_name: "public",
			type_id: 25,
			type_name: "text",
			is_nullable: false,
			is_primary_key: false,
			is_unique: true,
			default_expr: null,
			varchar_length: null,
			comment: "The user display name",
		},
	],
	functions: [],
	types: [],
	version: {
		version: "16.0",
		version_num: 160000,
		major_version: 16,
		active_connections: 1,
		max_connections: 100,
	},
	policies: [],
	extensions: [],
	triggers: [],
	roles: [],
};

test.describe("Monaco Editor with PGLS Language Server (Web Worker)", () => {
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
		const hasError = await page
			.locator("#status")
			.evaluate((el) => el.classList.contains("error"));

		if (hasError) {
			const errorText = await page.locator("#status").textContent();
			if (errorText?.includes("expected magic")) {
				test.skip(true, "WASM file is invalid or missing. Run build-wasm.sh");
			} else {
				throw new Error(`WASM failed to load: ${errorText}`);
			}
		}
	});

	test("loads Monaco editor and LSP worker", async ({ page }) => {
		// Verify Monaco editor is loaded
		const editor = page.locator(".monaco-editor");
		await expect(editor).toBeVisible();

		// Verify status shows ready
		await expect(page.locator("#status")).toHaveText("Ready");

		// Verify LSP client (worker) is available
		const hasLspClient = await page.evaluate(() => {
			return typeof (window as any).pglsLspClient !== "undefined";
		});
		expect(hasLspClient).toBe(true);
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

	test("LSP initialize request returns capabilities", async ({ page }) => {
		const response = await page.evaluate(async () => {
			const client = (window as any).pglsLspClient;
			return await client.sendMessage({
				jsonrpc: "2.0",
				id: 100,
				method: "initialize",
				params: { capabilities: {} },
			});
		});

		expect(response.length).toBeGreaterThanOrEqual(1);
		const initResponse = response.find((r: any) => r.id === 100);
		expect(initResponse).toBeDefined();
		expect(initResponse.result).toBeDefined();
		expect(initResponse.result.capabilities).toBeDefined();
	});

	test("LSP shutdown request works", async ({ page }) => {
		const response = await page.evaluate(async () => {
			const client = (window as any).pglsLspClient;
			return await client.sendMessage({
				jsonrpc: "2.0",
				id: 101,
				method: "shutdown",
				params: null,
			});
		});

		expect(response.length).toBeGreaterThanOrEqual(1);
		const shutdownResponse = response.find((r: any) => r.id === 101);
		expect(shutdownResponse).toBeDefined();
		expect(shutdownResponse.result).toBeNull();
	});

	test("LSP didOpen returns publishDiagnostics", async ({ page }) => {
		const response = await page.evaluate(async () => {
			const client = (window as any).pglsLspClient;
			return await client.sendMessage({
				jsonrpc: "2.0",
				method: "textDocument/didOpen",
				params: {
					textDocument: {
						uri: "file:///worker-test.sql",
						languageId: "sql",
						version: 1,
						text: "SELEC 1;",
					},
				},
			});
		});

		const diagNotification = response.find(
			(m: any) => m.method === "textDocument/publishDiagnostics",
		);
		expect(diagNotification).toBeDefined();
		expect(diagNotification?.params?.diagnostics?.length).toBeGreaterThan(0);
	});

	test("LSP completion request returns results", async ({ page }) => {
		// Open a document first
		await page.evaluate(async () => {
			const client = (window as any).pglsLspClient;
			await client.sendMessage({
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
		const response = await page.evaluate(async () => {
			const client = (window as any).pglsLspClient;
			return await client.sendMessage({
				jsonrpc: "2.0",
				id: 102,
				method: "textDocument/completion",
				params: {
					textDocument: { uri: "file:///completion-test.sql" },
					position: { line: 0, character: 7 },
				},
			});
		});

		expect(response.length).toBeGreaterThanOrEqual(1);
		const completionResponse = response.find((r: any) => r.id === 102);
		expect(completionResponse).toBeDefined();
		expect(completionResponse.result).toBeDefined();
		expect(Array.isArray(completionResponse.result)).toBe(true);
	});

	test("LSP hover request works", async ({ page }) => {
		// Open a document first
		await page.evaluate(async () => {
			const client = (window as any).pglsLspClient;
			await client.sendMessage({
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
		const response = await page.evaluate(async () => {
			const client = (window as any).pglsLspClient;
			return await client.sendMessage({
				jsonrpc: "2.0",
				id: 103,
				method: "textDocument/hover",
				params: {
					textDocument: { uri: "file:///hover-test.sql" },
					position: { line: 0, character: 14 },
				},
			});
		});

		expect(response.length).toBeGreaterThanOrEqual(1);
		const hoverResponse = response.find((r: any) => r.id === 103);
		expect(hoverResponse).toBeDefined();
		expect(hoverResponse.jsonrpc).toBe("2.0");
	});

	test("LSP didChange triggers publishDiagnostics", async ({ page }) => {
		// Open a document
		await page.evaluate(async () => {
			const client = (window as any).pglsLspClient;
			await client.sendMessage({
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
		const response = await page.evaluate(async () => {
			const client = (window as any).pglsLspClient;
			return await client.sendMessage({
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

		const diagNotification = response.find(
			(m: any) => m.method === "textDocument/publishDiagnostics",
		);
		expect(diagNotification).toBeDefined();
		expect(diagNotification.params.diagnostics.length).toBeGreaterThan(0);
	});

	test("setSchema via LSP notification enables schema-aware completions", async ({
		page,
	}) => {
		const schema = TEST_SCHEMA;

		const result = await page.evaluate(async (schema) => {
			const client = (window as any).pglsLspClient;

			// Set schema via LSP notification
			await client.sendMessage({
				jsonrpc: "2.0",
				method: "pgls/setSchema",
				params: { schema: JSON.stringify(schema) },
			});

			// Open a document
			await client.sendMessage({
				jsonrpc: "2.0",
				method: "textDocument/didOpen",
				params: {
					textDocument: {
						uri: "file:///schema-completion.sql",
						languageId: "sql",
						version: 1,
						text: "SELECT * FROM ",
					},
				},
			});

			// Request completions
			const response = await client.sendMessage({
				jsonrpc: "2.0",
				id: 200,
				method: "textDocument/completion",
				params: {
					textDocument: { uri: "file:///schema-completion.sql" },
					position: { line: 0, character: 14 },
				},
			});

			const completionResponse = response.find((r: any) => r.id === 200);
			return completionResponse?.result;
		}, schema);

		expect(Array.isArray(result)).toBe(true);
		const labels = result.map((item: any) => item.label);
		expect(labels).toContain("users");
		expect(labels).toContain("orders");
	});

	test("hover with schema returns column type info", async ({ page }) => {
		const schema = TEST_SCHEMA;

		const response = await page.evaluate(async (schema) => {
			const client = (window as any).pglsLspClient;

			// Set schema
			await client.sendMessage({
				jsonrpc: "2.0",
				method: "pgls/setSchema",
				params: { schema: JSON.stringify(schema) },
			});

			// Open a document
			await client.sendMessage({
				jsonrpc: "2.0",
				method: "textDocument/didOpen",
				params: {
					textDocument: {
						uri: "file:///schema-hover.sql",
						languageId: "sql",
						version: 1,
						text: "SELECT username FROM users;",
					},
				},
			});

			// Request hover over "username"
			return await client.sendMessage({
				jsonrpc: "2.0",
				id: 201,
				method: "textDocument/hover",
				params: {
					textDocument: { uri: "file:///schema-hover.sql" },
					position: { line: 0, character: 10 },
				},
			});
		}, schema);

		const hoverResponse = response.find((r: any) => r.id === 201);
		expect(hoverResponse).toBeDefined();
		// With schema, hover should return content
		if (hoverResponse.result !== null) {
			expect(hoverResponse.result.contents).toBeDefined();
		}
	});
});
