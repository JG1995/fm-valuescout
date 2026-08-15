import { createHash } from "node:crypto";
import { existsSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { basename, join } from "node:path";
import { fileURLToPath } from "node:url";

const semverPattern =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$/;
const bridgeSource = "../bridge/bin/Release/net6.0/FmDataBridge.dll";
const bridgeDestination = "resources/FmDataBridge.dll";

function readFile(path) {
  if (!existsSync(path)) {
    throw new Error(`Release input is missing: ${path}`);
  }

  return readFileSync(path, "utf8");
}

function readJsonVersion(path) {
  const value = JSON.parse(readFile(path)).version;

  if (typeof value !== "string") {
    throw new Error(`Release version is missing from ${path}`);
  }

  return value;
}

function readTomlVersion(path) {
  const match = readFile(path).match(/^version\s*=\s*"([^"]+)"\s*$/m);

  if (!match) {
    throw new Error(`Release version is missing from ${path}`);
  }

  return match[1];
}

function readProjectVersion(path) {
  const match = readFile(path).match(/<Version>([^<]+)<\/Version>/);

  if (!match) {
    throw new Error(`Release version is missing from ${path}`);
  }

  return match[1];
}

function readCargoLockVersion(path) {
  const packageBlock = readFile(path)
    .split("[[package]]")
    .find((block) => /^name\s*=\s*"app"\s*$/m.test(block));
  const match = packageBlock?.match(/^version\s*=\s*"([^"]+)"\s*$/m);

  if (!match) {
    throw new Error(`Release version is missing from ${path}`);
  }

  return match[1];
}

export function validateReleaseIdentity(rootDir) {
  const owners = {
    "package.json": readJsonVersion(join(rootDir, "package.json")),
    "src-tauri/Cargo.toml": readTomlVersion(
      join(rootDir, "src-tauri/Cargo.toml"),
    ),
    "src-tauri/Cargo.lock": readCargoLockVersion(
      join(rootDir, "src-tauri/Cargo.lock"),
    ),
    "src-tauri/tauri.conf.json": readJsonVersion(
      join(rootDir, "src-tauri/tauri.conf.json"),
    ),
    "bridge/FmDataBridge.csproj": readProjectVersion(
      join(rootDir, "bridge/FmDataBridge.csproj"),
    ),
  };
  const versions = new Set(Object.values(owners));

  if (versions.size !== 1) {
    throw new Error(
      `Release version mismatch: ${Object.entries(owners)
        .map(([path, version]) => `${path}=${version}`)
        .join(", ")}`,
    );
  }

  const version = versions.values().next().value;
  if (!semverPattern.test(version)) {
    throw new Error(`Release version is not SemVer: ${version}`);
  }

  return version;
}

export function validateReleaseConfig(rootDir) {
  const configPath = join(rootDir, "src-tauri/tauri.release.conf.json");
  const config = JSON.parse(readFile(configPath));
  const targets = config?.bundle?.targets;
  const resources = config?.bundle?.resources;

  if (
    !Array.isArray(targets) ||
    targets.length !== 1 ||
    targets[0] !== "nsis"
  ) {
    throw new Error("Release config must select only the NSIS bundle target");
  }

  if (
    !resources ||
    typeof resources !== "object" ||
    Array.isArray(resources) ||
    Object.keys(resources).length !== 1 ||
    resources[bridgeSource] !== bridgeDestination
  ) {
    throw new Error(
      "Release config must route the source-built bridge to resources/FmDataBridge.dll",
    );
  }

  return { bridgeSource, bridgeDestination };
}

export function validateBridgeDll(bridgeDllPath) {
  if (!existsSync(bridgeDllPath) || !statSync(bridgeDllPath).isFile()) {
    throw new Error(`Bridge DLL is missing: ${bridgeDllPath}`);
  }

  const contents = readFileSync(bridgeDllPath);
  if (contents.length < 0x40 || contents[0] !== 0x4d || contents[1] !== 0x5a) {
    throw new Error(
      `Bridge DLL is not a managed Windows DLL: ${bridgeDllPath}`,
    );
  }

  const peOffset = contents.readUInt32LE(0x3c);
  const optionalHeaderOffset = peOffset + 24;
  if (peOffset + 26 > contents.length) {
    throw new Error(
      `Bridge DLL is not a managed Windows DLL: ${bridgeDllPath}`,
    );
  }

  const optionalHeaderMagic = contents.readUInt16LE(optionalHeaderOffset);
  const dataDirectoryOffset =
    optionalHeaderOffset + (optionalHeaderMagic === 0x10b ? 96 : 112);
  const clrDirectoryOffset = dataDirectoryOffset + 14 * 8;

  if (
    contents.subarray(peOffset, peOffset + 4).toString() !== "PE\0\0" ||
    (optionalHeaderMagic !== 0x10b && optionalHeaderMagic !== 0x20b) ||
    clrDirectoryOffset + 8 > contents.length ||
    contents.readUInt32LE(clrDirectoryOffset) === 0 ||
    contents.readUInt32LE(clrDirectoryOffset + 4) === 0
  ) {
    throw new Error(
      `Bridge DLL is not a managed Windows DLL: ${bridgeDllPath}`,
    );
  }
}

export function writeSha256(installerPath) {
  if (!existsSync(installerPath) || !statSync(installerPath).isFile()) {
    throw new Error(`Installer is missing: ${installerPath}`);
  }

  const digest = createHash("sha256")
    .update(readFileSync(installerPath))
    .digest("hex");
  const checksumPath = `${installerPath}.sha256`;
  writeFileSync(checksumPath, `${digest} *${basename(installerPath)}\n`);

  return checksumPath;
}

function requireArguments(operation, args, expectedCount) {
  if (args.length !== expectedCount) {
    throw new Error(`${operation} expects ${expectedCount} argument(s)`);
  }
}

function main() {
  const [operation, ...args] = process.argv.slice(2);

  switch (operation) {
    case "identity":
      requireArguments(operation, args, 0);
      process.stdout.write(`${validateReleaseIdentity(process.cwd())}\n`);
      return;
    case "config":
      requireArguments(operation, args, 0);
      validateReleaseConfig(process.cwd());
      return;
    case "bridge":
      requireArguments(operation, args, 1);
      validateBridgeDll(args[0]);
      return;
    case "checksum":
      requireArguments(operation, args, 1);
      process.stdout.write(`${writeSha256(args[0])}\n`);
      return;
    default:
      throw new Error(
        "Usage: release-package-validation {identity|config|bridge|checksum} [path]",
      );
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main();
}
