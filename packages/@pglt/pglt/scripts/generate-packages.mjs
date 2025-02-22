import assert from "node:assert";
import * as fs from "node:fs";
import { pipeline } from "node:stream";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
const streamPipeline = promisify(pipeline);

const CLI_ROOT = resolve(fileURLToPath(import.meta.url), "../..");
const PACKAGES_PGLT_ROOT = resolve(CLI_ROOT, "..");
const PGLT_ROOT = resolve(PACKAGES_PGLT_ROOT, "../..");
const MANIFEST_PATH = resolve(CLI_ROOT, "package.json");
const SUPPORTED_PLATFORMS = [
  "pc-windows-msvc",
  "apple-darwin",
  "unknown-linux-gnu",
];
const SUPPORTED_ARCHITECTURES = ["x86_64", "aarch64"];

async function downloadSchema(releaseTag, githubToken) {
  const assetUrl = `https://github.com/supabase-community/postgres_lsp/releases/download/${releaseTag}/schema.json`;

  const response = await fetch(assetUrl.trim(), {
    headers: {
      Authorization: `token ${githubToken}`,
      Accept: `application/octet-stream`,
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to Fetch Asset from ${assetUrl}`);
  }

  // download to root.
  const fileStream = fs.createWriteStream(resolve(PGLT_ROOT, "schema.json"));

  await streamPipeline(response.body, fileStream);

  console.log(`Downloaded schema for ${releaseTag}`);
}

async function downloadBinary(platform, arch, os, releaseTag, githubToken) {
  const buildName = getBuildName(platform, arch);

  // const assetUrl = `https://github.com/supabase-community/postgres_lsp/releases/download/${releaseTag}/${buildName}`;
  const assetUrl = `https://github.com/juleswritescode/pglsp_publish_test/releases/download/${releaseTag}/${buildName}`;
  // https://github.com/juleswritescode/pglsp_publish_test/releases/download/0.1.0/pglt_x86_64-pc-windows-msvc
  // https://github.com/juleswritescode/pglsp_publish_test/releases/download/0.1.0/pglt_x86_64-pc-windows-msvc

  const response = await fetch(assetUrl.trim(), {
    headers: {
      Authorization: `token ${githubToken}`,
      Accept: `application/octet-stream`,
    },
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(
      `Failed to Fetch Asset from ${assetUrl} (Reason: ${error})`
    );
  }

  // just download to root.
  const fileStream = fs.createWriteStream(getBinarySource(platform, arch, os));

  await streamPipeline(response.body, fileStream);

  console.log(`Downloaded asset for ${buildName} (v${releaseTag})`);
}

async function overwriteManifestVersions(releaseTag, isPrerelease) {
  const version = getVersion(releaseTag, isPrerelease);

  const manifestClone = structuredClone(rootManifest());

  manifestClone.version = version;
  for (const key in manifestClone.optionalDependencies) {
    manifestClone.optionalDependencies[key] = version;
  }

  /**
   * writeFileSync seemed to not work reliably?
   */
  await new Promise((res, rej) => {
    fs.writeFile(MANIFEST_PATH, JSON.stringify(manifestClone, null, 2), (e) =>
      e ? rej(e) : res()
    );
  });
}

async function makePackageDir(platform, arch) {
  const buildName = getBuildName(platform, arch);
  const packageRoot = resolve(PACKAGES_PGLT_ROOT, buildName);

  await new Promise((res, rej) => {
    fs.mkdir(packageRoot, {}, (e) => (e ? rej(e) : res()));
  });
}

function copyBinaryToNativePackage(platform, arch, os) {
  // Update the package.json manifest
  const buildName = getBuildName(platform, arch);
  const packageRoot = resolve(PACKAGES_PGLT_ROOT, buildName);
  const packageName = getPackageName(platform, arch);

  const { version, license, repository, engines } = rootManifest();

  /**
   * We need to map rust triplets to NPM-known values.
   * Otherwise, npm will abort the package installation.
   */
  const npm_arch = arch === "aarch64" ? "arm64" : "x64";
  let libc = undefined;
  let npm_os = undefined;

  switch (os) {
    case "linux": {
      libc = "gnu";
      npm_os = "linux";
      break;
    }
    case "windows": {
      libc = "msvc";
      npm_os = "win32";
      break;
    }
    case "darwin": {
      libc = undefined;
      npm_os = "darwin";
      break;
    }
    default: {
      throw new Error(`Unsupported os: ${os}`);
    }
  }

  const manifest = JSON.stringify(
    {
      name: packageName,
      version,
      license,
      repository,
      engines,
      os: [npm_os],
      cpu: [npm_arch],
      libc,
    },
    null,
    2
  );

  const ext = getBinaryExt(os);
  const manifestPath = resolve(packageRoot, "package.json");
  console.info(`Update manifest ${manifestPath}`);
  fs.writeFileSync(manifestPath, manifest);

  // Copy the CLI binary
  const binarySource = getBinarySource(platform, arch, os);
  const binaryTarget = resolve(packageRoot, `pglt${ext}`);

  if (!fs.existsSync(binarySource)) {
    console.error(
      `Source for binary for ${buildName} not found at: ${binarySource}`
    );
    process.exit(1);
  }

  console.info(`Copy binary ${binaryTarget}`);
  fs.copyFileSync(binarySource, binaryTarget);
  fs.chmodSync(binaryTarget, 0o755);
}

function copySchemaToNativePackage(platform, arch) {
  const buildName = getBuildName(platform, arch);
  const packageRoot = resolve(PACKAGES_PGLT_ROOT, buildName);

  const schemaSrc = resolve(PGLT_ROOT, `schema.json`);
  const schemaTarget = resolve(packageRoot, `schema.json`);

  if (!fs.existsSync(schemaSrc)) {
    console.error(`schema.json not found at: ${schemaSrc}`);
    process.exit(1);
  }

  console.info(`Copying schema.json`);
  fs.copyFileSync(schemaSrc, schemaTarget);
  fs.chmodSync(schemaTarget, 0o666);
}

const rootManifest = () =>
  JSON.parse(fs.readFileSync(MANIFEST_PATH).toString("utf-8"));

function getBinaryExt(os) {
  return os === "windows" ? ".exe" : "";
}

function getBinarySource(platform, arch, os) {
  const ext = getBinaryExt(os);
  return resolve(PGLT_ROOT, `${getBuildName(platform, arch)}${ext}`);
}

function getBuildName(platform, arch) {
  return `pglt_${arch}-${platform}`;
}

function getPackageName(platform, arch) {
  // trim the "unknown" from linux and the "pc" from windows
  const name = platform.split("-").slice(-2).join("-");
  return `pglt-testrelease-nightly-2-cli-${arch}-${name}`;
}

function getOs(platform) {
  return platform.split("-").find((_, idx) => idx === 1);
}

function getVersion(releaseTag, isPrerelease) {
  return releaseTag + (isPrerelease ? "-rc" : "");
}

(async function main() {
  const githubToken = process.env.GITHUB_TOKEN;
  let releaseTag = process.env.RELEASE_TAG;
  assert(githubToken, "GITHUB_TOKEN not defined!");
  assert(releaseTag, "RELEASE_TAG not defined!");

  const isPrerelease = process.env.PRERELEASE === "true";

  await downloadSchema(releaseTag, githubToken);
  await overwriteManifestVersions(releaseTag, isPrerelease);

  for (const platform of SUPPORTED_PLATFORMS) {
    const os = getOs(platform);

    for (const arch of SUPPORTED_ARCHITECTURES) {
      await makePackageDir(platform, arch);
      await downloadBinary(platform, arch, os, releaseTag, githubToken);
      copyBinaryToNativePackage(platform, arch, os);
      copySchemaToNativePackage(platform, arch);
    }
  }

  process.exit(0);
})();
