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
    const projectKey = await workspace.registerProjectFolder({});
    await workspace.openFile({
      project_key: projectKey,
      path: {
        path: "test.sql",
        was_written: false,
        kind: ["Handleable"],
      },
      content: "select 1 from",
      version: 0,
    });

    const { diagnostics } = await workspace.pullFileDiagnostics({
      only: [],
      skip: [],
      max_diagnostics: 100,
      categories: [],
      project_key: projectKey,
      path: {
        path: "test.sql",
        was_written: false,
        kind: ["Handleable"],
      },
    });

    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0].description).toBe("Invalid statement: syntax error at end of input");

    await workspace.closeFile({
      project_key: projectKey,
      path: {
        path: "test.sql",
        was_written: false,
        kind: ["Handleable"],
      },
    });

    workspace.destroy();
  });
});
