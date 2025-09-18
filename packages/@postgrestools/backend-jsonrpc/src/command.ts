import { execSync } from "node:child_process";

/**
 * Gets the path of the binary for the current platform
 *
 * @returns Filesystem path to the binary, or null if no prebuilt distribution exists for the current platform
 */
export function getCommand(): string | null {
  const { platform, arch } = process;

  const PLATFORMS: Partial<
    Record<
      NodeJS.Platform | "linux-musl",
      Partial<Record<NodeJS.Architecture, string>>
    >
  > = {
    win32: {
      x64: "@postgrestools/cli-x86_64-windows-msvc/postgrestools.exe",
      arm64: "@postgrestools/cli-aarch64-windows-msvc/postgrestools.exe",
    },
    darwin: {
      x64: "@postgrestools/cli-x86_64-apple-darwin/postgrestools",
      arm64: "@postgrestools/cli-aarch64-apple-darwin/postgrestools",
    },
    linux: {
      x64: "@postgrestools/cli-x86_64-linux-gnu/postgrestools",
      arm64: "@postgrestools/cli-aarch64-linux-gnu/postgrestools",
    },
    "linux-musl": {
      x64: "@postgrestools/cli-x86_64-linux-musl/postgrestools",
      // no arm64 build for musl
    },
  };

  function isMusl() {
    let stderr = "";
    try {
      stderr = execSync("ldd --version", {
        stdio: [
          "ignore", // stdin
          "pipe", // stdout – glibc systems print here
          "pipe", // stderr – musl systems print here
        ],
      }).toString();
    } catch (err: unknown) {
      if (hasStdErr(err)) {
        stderr = err.stderr;
      }
    }
    if (stderr.indexOf("musl") > -1) {
      return true;
    }
    return false;
  }

  function getPlatform(): NodeJS.Platform | "linux-musl" {
    if (platform === "linux") {
      return isMusl() ? "linux-musl" : "linux";
    }

    return platform;
  }

  const binPath = PLATFORMS?.[getPlatform()]?.[arch];
  if (!binPath) {
    return null;
  }

  return require.resolve(binPath);
}

function hasStdErr(err: unknown): err is { stderr: string } {
  return !!(err as any)?.stderr;
}
