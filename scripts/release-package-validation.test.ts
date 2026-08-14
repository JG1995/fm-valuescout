import { createHash } from "node:crypto";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import {
  validateBridgeDll,
  validateReleaseConfig,
  validateReleaseIdentity,
  writeSha256,
} from "./release-package-validation.mjs";

const temporaryDirectories: string[] = [];

function createReleaseFixture(overrides: Record<string, string> = {}) {
  const rootDir = mkdtempSync(join(tmpdir(), "fm-valuescout-release-"));
  temporaryDirectories.push(rootDir);

  mkdirSync(join(rootDir, "src-tauri"), { recursive: true });
  mkdirSync(join(rootDir, "bridge"), { recursive: true });

  const files = {
    "package.json": JSON.stringify({ version: "0.1.0" }),
    "src-tauri/Cargo.toml": '[package]\nversion = "0.1.0"\n',
    "src-tauri/Cargo.lock":
      'version = 3\n\n[[package]]\nname = "app"\nversion = "0.1.0"\n',
    "src-tauri/tauri.conf.json": JSON.stringify({ version: "0.1.0" }),
    "bridge/FmDataBridge.csproj":
      "<Project><PropertyGroup><Version>0.1.0</Version></PropertyGroup></Project>",
    "src-tauri/tauri.release.conf.json": JSON.stringify({
      bundle: {
        targets: ["nsis"],
        resources: {
          "../bridge/bin/Release/net6.0/FmDataBridge.dll":
            "resources/FmDataBridge.dll",
        },
      },
    }),
    ...overrides,
  };

  for (const [relativePath, contents] of Object.entries(files)) {
    const filePath = join(rootDir, relativePath);
    mkdirSync(dirname(filePath), { recursive: true });
    writeFileSync(filePath, contents);
  }

  return rootDir;
}

afterEach(() => {
  for (const directory of temporaryDirectories.splice(0)) {
    rmSync(directory, { force: true, recursive: true });
  }
});

describe("release package validation", () => {
  it("accepts one matching SemVer identity and the isolated NSIS mapping", () => {
    const rootDir = createReleaseFixture();

    expect(validateReleaseIdentity(rootDir)).toBe("0.1.0");
    expect(validateReleaseConfig(rootDir)).toEqual({
      bridgeSource: "../bridge/bin/Release/net6.0/FmDataBridge.dll",
      bridgeDestination: "resources/FmDataBridge.dll",
    });
  });

  it("rejects a version mismatch before a package command can invoke Tauri", () => {
    const rootDir = createReleaseFixture({
      "src-tauri/Cargo.toml": '[package]\nversion = "0.1.1"\n',
    });

    expect(() => validateReleaseIdentity(rootDir)).toThrow("version mismatch");
  });

  it("rejects a Cargo lock entry that differs from the release version", () => {
    const rootDir = createReleaseFixture({
      "src-tauri/Cargo.lock":
        'version = 3\n\n[[package]]\nname = "app"\nversion = "0.1.1"\n',
    });

    expect(() => validateReleaseIdentity(rootDir)).toThrow("version mismatch");
  });

  it("rejects routing the release through the tracked placeholder resource", () => {
    const rootDir = createReleaseFixture({
      "src-tauri/tauri.release.conf.json": JSON.stringify({
        bundle: {
          targets: ["nsis"],
          resources: {
            "resources/FmDataBridge.dll": "resources/FmDataBridge.dll",
          },
        },
      }),
    });

    expect(() => validateReleaseConfig(rootDir)).toThrow("source-built bridge");
  });

  it("runs release validation before starting the Tauri bundle", () => {
    const command = readFileSync("scripts/dev", "utf8");
    const tauriBuild = command.indexOf("pnpm tauri build");
    const validations = [
      command.indexOf("release-package-validation.mjs identity"),
      command.indexOf("release-package-validation.mjs config"),
      command.indexOf("release-package-validation.mjs bridge"),
    ];

    expect(tauriBuild).toBeGreaterThan(-1);
    for (const validation of validations) {
      expect(validation).toBeGreaterThan(-1);
      expect(validation).toBeLessThan(tauriBuild);
    }
  });

  it("uses locked Cargo resolution and version-scoped release output", () => {
    const command = readFileSync("scripts/dev", "utf8");

    expect(command).toContain(
      'target_dir="$repo_root/.release/tauri-target/$version"',
    );
    expect(command).toContain('rm -rf -- "$target_dir"');
    expect(command).toContain(
      'CARGO_TARGET_DIR="$target_dir" pnpm tauri build',
    );
    expect(command).toContain("-- --locked");
    expect(command).toContain(
      'artifact_dir="$repo_root/.release/windows/$version"',
    );
    expect(command).toContain('rm -rf -- "$artifact_dir"');
  });

  it("rejects the current text placeholder as a bridge DLL", () => {
    expect(() => {
      validateBridgeDll("src-tauri/resources/FmDataBridge.dll");
    }).toThrow("managed Windows DLL");
  });

  it("rejects an MZ file that has no managed assembly metadata", () => {
    const rootDir = createReleaseFixture();
    const fakeDllPath = join(rootDir, "not-managed.dll");
    writeFileSync(fakeDllPath, Buffer.from("MZ not a DLL"));

    expect(() => validateBridgeDll(fakeDllPath)).toThrow("managed Windows DLL");
  });

  it("accepts a PE DLL with a CLR metadata directory", () => {
    const rootDir = createReleaseFixture();
    const bridgeDllPath = join(rootDir, "FmDataBridge.dll");
    const dll = Buffer.alloc(0x200);
    const peOffset = 0x80;
    const optionalHeaderOffset = peOffset + 24;
    const clrDirectoryOffset = optionalHeaderOffset + 96 + 14 * 8;
    dll.write("MZ");
    dll.writeUInt32LE(peOffset, 0x3c);
    dll.write("PE\0\0", peOffset);
    dll.writeUInt16LE(0x10b, optionalHeaderOffset);
    dll.writeUInt32LE(0x2000, clrDirectoryOffset);
    dll.writeUInt32LE(72, clrDirectoryOffset + 4);
    writeFileSync(bridgeDllPath, dll);

    expect(() => validateBridgeDll(bridgeDllPath)).not.toThrow();
  });

  it("writes a SHA-256 sidecar for the exact installer bytes", () => {
    const rootDir = createReleaseFixture();
    const installerPath = join(rootDir, "FM-ValueScout_0.1.0_x64-setup.exe");
    const installer = Buffer.from("installer-bytes");
    writeFileSync(installerPath, installer);

    const checksumPath = writeSha256(installerPath);
    const expectedHash = createHash("sha256").update(installer).digest("hex");

    expect(checksumPath).toBe(`${installerPath}.sha256`);
    expect(readFileSync(checksumPath, "utf8")).toBe(
      `${expectedHash} *FM-ValueScout_0.1.0_x64-setup.exe\n`,
    );
  });
});
