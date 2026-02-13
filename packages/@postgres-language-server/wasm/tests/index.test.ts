/**
 * Tests for the Postgres Language Server WASM bindings.
 */

import { beforeAll, beforeEach, describe, expect, test } from "bun:test";
import { type LanguageServer, createLanguageServer } from "../src/lsp";
import { type Workspace, createWorkspace } from "../src/workspace";

// =============================================================================
// Workspace API Tests
// =============================================================================

describe("Workspace API", () => {
	let workspace: Workspace;

	beforeAll(async () => {
		workspace = await createWorkspace();
	});

	test("version returns a string", () => {
		const version = workspace.version();
		expect(typeof version).toBe("string");
		expect(version).toBe("0.0.0");
	});

	test("parse valid SQL returns empty array", () => {
		const errors = workspace.parse("SELECT 1;");
		expect(errors).toBeArray();
		expect(errors.length).toBe(0);
	});

	test("parse invalid SQL returns errors", () => {
		const errors = workspace.parse("SELEC 1;");
		expect(errors).toBeArray();
		expect(errors.length).toBeGreaterThan(0);
	});

	test("parse multiple statements", () => {
		const errors = workspace.parse("SELECT 1; SELECT 2;");
		expect(errors).toBeArray();
		expect(errors.length).toBe(0);
	});

	test("insertFile and lint", () => {
		workspace.insertFile("/test.sql", "SELECT * FROM users;");
		const diagnostics = workspace.lint("/test.sql");
		expect(diagnostics).toBeArray();
		// Valid SQL should have no parse errors
		expect(diagnostics.length).toBe(0);
	});

	test("insertFile with invalid SQL and lint", () => {
		workspace.insertFile("/invalid.sql", "SELEC * FROM;");
		const diagnostics = workspace.lint("/invalid.sql");
		expect(diagnostics).toBeArray();
		// Invalid SQL should have at least one error
		expect(diagnostics.length).toBeGreaterThan(0);
	});

	test("removeFile", () => {
		workspace.insertFile("/to-remove.sql", "SELECT 1;");
		// Should not throw
		workspace.removeFile("/to-remove.sql");
		// Linting a removed file should throw or return error
		expect(() => workspace.lint("/to-remove.sql")).toThrow();
	});

	test("complete returns completion items", () => {
		workspace.insertFile("/complete.sql", "SELECT ");
		const completions = workspace.complete("/complete.sql", 7);
		expect(completions).toBeArray();
		// Without schema, may return empty or basic completions
	});

	test("hover returns null without schema", () => {
		// Explicitly clear schema to ensure we're testing without schema
		workspace.clearSchema();
		workspace.insertFile("/hover.sql", "SELECT * FROM users;");
		const hover = workspace.hover("/hover.sql", 14); // Over "users"
		// Without schema loaded, hover should return null
		expect(hover).toBeNull();
	});

	test("clearSchema does not throw", () => {
		// clearSchema should work even without a schema set
		expect(() => workspace.clearSchema()).not.toThrow();
	});

	test("setSchema with invalid JSON throws", () => {
		expect(() => workspace.setSchema("not valid json")).toThrow();
	});

	test("parse empty string", () => {
		const errors = workspace.parse("");
		expect(errors).toBeArray();
		expect(errors.length).toBe(0);
	});

	test("parse with comments", () => {
		const errors = workspace.parse("-- This is a comment\nSELECT 1;");
		expect(errors).toBeArray();
		expect(errors.length).toBe(0);
	});
});

// =============================================================================
// Language Server API Tests
// =============================================================================

describe("LanguageServer API", () => {
	let lsp: LanguageServer;

	beforeAll(async () => {
		lsp = await createLanguageServer();
	});

	test("handleMessage returns array", () => {
		const messages = lsp.handleMessage({
			jsonrpc: "2.0",
			id: 1,
			method: "initialize",
			params: {},
		});
		expect(messages).toBeArray();
		expect(messages.length).toBeGreaterThan(0);
	});

	test("initialize returns capabilities", () => {
		const messages = lsp.handleMessage({
			jsonrpc: "2.0",
			id: 1,
			method: "initialize",
			params: {},
		});

		expect(messages.length).toBe(1);
		const response = messages[0];
		expect(response.jsonrpc).toBe("2.0");
		expect(response.id).toBe(1);
		expect(response.result).toBeDefined();
		// @ts-expect-error - result is unknown type
		expect(response.result.capabilities).toBeDefined();
	});

	test("shutdown returns null", () => {
		const messages = lsp.handleMessage({
			jsonrpc: "2.0",
			id: 2,
			method: "shutdown",
			params: null,
		});

		expect(messages.length).toBe(1);
		const response = messages[0];
		expect(response.id).toBe(2);
		expect(response.result).toBeNull();
	});

	test("handleMessage accepts string input", () => {
		const messages = lsp.handleMessage(
			JSON.stringify({
				jsonrpc: "2.0",
				id: 3,
				method: "shutdown",
				params: null,
			}),
		);

		expect(messages.length).toBe(1);
		expect(messages[0].id).toBe(3);
	});

	test("didOpen returns publishDiagnostics notification", () => {
		const messages = lsp.handleMessage({
			jsonrpc: "2.0",
			method: "textDocument/didOpen",
			params: {
				textDocument: {
					uri: "file:///test-lsp.sql",
					languageId: "sql",
					version: 1,
					text: "SELECT * FROM users;",
				},
			},
		});

		// Should return at least one publishDiagnostics notification
		expect(messages.length).toBeGreaterThanOrEqual(1);
		const notification = messages.find(
			(m) => m.method === "textDocument/publishDiagnostics",
		);
		expect(notification).toBeDefined();
		expect(notification?.params).toBeDefined();
	});

	test("didOpen with invalid SQL returns diagnostics", () => {
		const messages = lsp.handleMessage({
			jsonrpc: "2.0",
			method: "textDocument/didOpen",
			params: {
				textDocument: {
					uri: "file:///invalid-lsp.sql",
					languageId: "sql",
					version: 1,
					text: "SELEC * FROM;",
				},
			},
		});

		const notification = messages.find(
			(m) => m.method === "textDocument/publishDiagnostics",
		);
		expect(notification).toBeDefined();
		// @ts-expect-error - params is unknown type
		expect(notification?.params?.diagnostics?.length).toBeGreaterThan(0);
	});

	test("unknown method returns error", () => {
		const messages = lsp.handleMessage({
			jsonrpc: "2.0",
			id: 99,
			method: "unknownMethod",
			params: {},
		});

		expect(messages.length).toBe(1);
		const response = messages[0];
		expect(response.error).toBeDefined();
		expect(response.error?.code).toBe(-32601); // Method not found
	});

	test("invalid JSON returns parse error", () => {
		const messages = lsp.handleMessage("not valid json");

		expect(messages.length).toBe(1);
		const response = messages[0];
		expect(response.error).toBeDefined();
		expect(response.error?.code).toBe(-32700); // Parse error
	});
});

// =============================================================================
// Schema-based Workspace Tests
// =============================================================================

/**
 * Sample schema for testing completions and hover.
 * This matches the Rust SchemaCache struct format.
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
			comment: "User accounts table",
		},
		{
			id: 2,
			schema: "public",
			name: "posts",
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
			default_expr: "nextval('users_id_seq'::regclass)",
			varchar_length: null,
			comment: null,
		},
		{
			name: "email",
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
			comment: "User email address",
		},
		{
			name: "name",
			table_name: "users",
			table_oid: 1,
			class_kind: "OrdinaryTable",
			number: 3,
			schema_name: "public",
			type_id: 1043,
			type_name: "character varying",
			is_nullable: true,
			is_primary_key: false,
			is_unique: false,
			default_expr: null,
			varchar_length: 255,
			comment: null,
		},
		{
			name: "id",
			table_name: "posts",
			table_oid: 2,
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
			name: "user_id",
			table_name: "posts",
			table_oid: 2,
			class_kind: "OrdinaryTable",
			number: 2,
			schema_name: "public",
			type_id: 23,
			type_name: "integer",
			is_nullable: false,
			is_primary_key: false,
			is_unique: false,
			default_expr: null,
			varchar_length: null,
			comment: null,
		},
		{
			name: "title",
			table_name: "posts",
			table_oid: 2,
			class_kind: "OrdinaryTable",
			number: 3,
			schema_name: "public",
			type_id: 25,
			type_name: "text",
			is_nullable: false,
			is_primary_key: false,
			is_unique: false,
			default_expr: null,
			varchar_length: null,
			comment: null,
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

describe("Schema-based Workspace completions and hover", () => {
	let workspace: Workspace;

	beforeAll(async () => {
		workspace = await createWorkspace();
	});

	// Ensure schema is loaded before each test
	beforeEach(() => {
		workspace.setSchema(JSON.stringify(TEST_SCHEMA));
	});

	test("setSchema works with valid schema", () => {
		// Schema was set in beforeEach
		expect(true).toBe(true);
	});

	test("complete returns table names in FROM clause", () => {
		workspace.insertFile("/from-complete.sql", "SELECT * FROM ");
		const completions = workspace.complete("/from-complete.sql", 14);
		expect(completions).toBeArray();
		// Should contain table names from schema
		const tableNames = completions.map((c: any) => c.label);
		expect(tableNames).toContain("users");
		expect(tableNames).toContain("posts");
	});

	test("complete returns column names after table reference", () => {
		workspace.insertFile("/col-complete.sql", "SELECT  FROM users");
		// Position cursor after SELECT (position 7)
		const completions = workspace.complete("/col-complete.sql", 7);
		expect(completions).toBeArray();
		// Should contain column names from users table
		const columnNames = completions.map((c: any) => c.label);
		expect(columnNames).toContain("id");
		expect(columnNames).toContain("email");
		expect(columnNames).toContain("name");
	});

	test("hover on table name shows table info", () => {
		workspace.insertFile("/hover-table.sql", "SELECT * FROM users;");
		// Position over "users" (around character 14)
		const hover = workspace.hover("/hover-table.sql", 14);
		// With schema loaded, hover should return info (a markdown string)
		expect(hover).not.toBeNull();
		expect(typeof hover).toBe("string");
		// The hover text should mention the table
		expect(hover?.toLowerCase()).toContain("users");
	});

	test("hover on column name shows column type", () => {
		workspace.insertFile("/hover-col.sql", "SELECT email FROM users;");
		// Position over "email" (around character 7)
		const hover = workspace.hover("/hover-col.sql", 8);
		// With schema loaded, hover should return type info (a markdown string)
		expect(hover).not.toBeNull();
		expect(typeof hover).toBe("string");
		// The hover text should mention the type
		expect(hover?.toLowerCase()).toContain("text");
	});

	test("clearSchema removes schema and hover returns null", () => {
		// First verify hover works with schema (set by beforeEach)
		workspace.insertFile("/with-schema.sql", "SELECT * FROM users;");
		const hoverWithSchema = workspace.hover("/with-schema.sql", 14);
		expect(hoverWithSchema).not.toBeNull();

		// Now clear schema and verify hover returns null
		workspace.clearSchema();
		workspace.insertFile("/no-schema.sql", "SELECT * FROM users;");
		const hoverWithoutSchema = workspace.hover("/no-schema.sql", 14);
		expect(hoverWithoutSchema).toBeNull();
	});
});
