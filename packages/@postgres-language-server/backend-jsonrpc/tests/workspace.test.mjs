import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

import { createWorkspaceWithBinary } from "../src";

describe("Workspace API", () => {
	it("should process remote requests", async () => {
		const extension = process.platform === "win32" ? ".exe" : "";
		const command = resolve(
			fileURLToPath(import.meta.url),
			"../../../../..",
			`target/release/postgres-language-server${extension}`,
		);

		const workspace = await createWorkspaceWithBinary(command);
		workspace.registerProjectFolder({
			setAsCurrentWorkspace: true,
		});
		await workspace.openFile({
			path: {
				path: "test.sql",
				was_written: false,
				kind: ["Handleable"],
			},
			content: "select 1 from",
			version: 0,
		});

		const { diagnostics } = await workspace.pullDiagnostics({
			only: [],
			skip: [],
			max_diagnostics: 100,
			categories: [],
			path: {
				path: "test.sql",
				was_written: false,
				kind: ["Handleable"],
			},
		});

		expect(diagnostics).toHaveLength(1);
		expect(diagnostics[0].description).toBe(
			"Invalid statement: syntax error at end of input",
		);

		await workspace.closeFile({
			path: {
				path: "test.sql",
				was_written: false,
				kind: ["Handleable"],
			},
		});

		workspace.destroy();
	});
});
