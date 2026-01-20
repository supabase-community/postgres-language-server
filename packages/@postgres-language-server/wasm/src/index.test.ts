/**
 * Tests for the Postgres Language Server WASM bindings.
 */

import { expect, test, describe, beforeAll } from "bun:test";
import { createWorkspace, Workspace } from "./index";

describe("WASM Workspace", () => {
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
		workspace.insertFile("/hover.sql", "SELECT * FROM users;");
		const hover = workspace.hover("/hover.sql", 14); // Over "users"
		// Without schema loaded, hover should return null
		expect(hover).toBeNull();
	});

	test("clearSchema does not throw", () => {
		// clearSchema should work even without a schema set
		expect(() => workspace.clearSchema()).not.toThrow();
	});

	// Note: setSchema test is skipped because it requires exact field matching
	// with the Rust SchemaCache struct. In practice, schema JSON would be
	// exported from a real database using `postgres-language-server schema-export`.

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
