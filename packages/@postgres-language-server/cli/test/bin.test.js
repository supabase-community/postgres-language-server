import { describe, expect, it } from "bun:test";
import { spawn } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const binPath = join(__dirname, "../bin/postgres-language-server");
const testSqlPath = join(__dirname, "test.sql");

describe("postgres-language-server bin", () => {
	it("should check a SQL file successfully", async () => {
		const result = await new Promise((resolve) => {
			const proc = spawn(
				"node",
				[binPath, "check", "--disable-db", testSqlPath],
				{
					env: { ...process.env },
				},
			);

			let stdout = "";
			let stderr = "";

			proc.stdout.on("data", (data) => {
				stdout += data.toString();
			});

			proc.stderr.on("data", (data) => {
				stderr += data.toString();
			});

			proc.on("close", (code) => {
				resolve({ code, stdout, stderr });
			});
		});

		expect(result.code).toBe(0);
		expect(result.stderr).toBe("");
	});

	it("should fail when file doesn't exist", async () => {
		const result = await new Promise((resolve) => {
			const proc = spawn("node", [binPath, "check", "nonexistent.sql"], {
				env: { ...process.env },
			});

			let stdout = "";
			let stderr = "";

			proc.stdout.on("data", (data) => {
				stdout += data.toString();
			});

			proc.stderr.on("data", (data) => {
				stderr += data.toString();
			});

			proc.on("close", (code) => {
				resolve({ code, stdout, stderr });
			});
		});

		expect(result.code).not.toBe(0);
	});
});
