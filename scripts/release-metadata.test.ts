import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import {
  calculateExpectedVersion,
  compareSemver,
  readReleaseMetadata,
} from "./release-metadata.mjs";

const temporaryDirectories: string[] = [];
const initialVersion = "0.1.0-alpha.1";
const initialSection = `## [${initialVersion}] - 2026-08-14

### Added

- Initial super-early alpha release.`;

function createFixture(overrides: Record<string, string> = {}) {
  const rootDir = mkdtempSync(
    join(tmpdir(), "fm-valuescout-release-metadata-"),
  );
  temporaryDirectories.push(rootDir);

  const files = {
    "package.json": JSON.stringify({ version: initialVersion }),
    "src-tauri/Cargo.toml": `[package]\nversion = "${initialVersion}"\n`,
    "src-tauri/Cargo.lock": `version = 3\n\n[[package]]\nname = "app"\nversion = "${initialVersion}"\n`,
    "src-tauri/tauri.conf.json": JSON.stringify({ version: initialVersion }),
    "bridge/FmDataBridge.csproj": `<Project><PropertyGroup><Version>${initialVersion}</Version></PropertyGroup></Project>`,
    "CHANGELOG.md": `# Changelog

All notable changes to this project are documented in this file.

## [Unreleased]

${initialSection}

## [0.0.1] - 2026-08-01

### Added

- Historical entry.`,
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

describe("release metadata", () => {
  it("accepts the initial alpha identity and emits its exact dated section", () => {
    const rootDir = createFixture();

    expect(readReleaseMetadata(rootDir, null, "minor")).toEqual({
      version: initialVersion,
      tag: `v${initialVersion}`,
      releaseRequired: true,
      releaseNotes: initialSection,
    });
  });

  it("orders alpha prereleases and calculates patch and minor identities", () => {
    expect(compareSemver("0.1.0-alpha.2", "0.1.0-alpha.1")).toBeGreaterThan(0);
    expect(calculateExpectedVersion("0.1.0-alpha.1", "patch")).toBe(
      "0.1.0-alpha.2",
    );
    expect(calculateExpectedVersion("0.1.0-alpha.1", "minor")).toBe(
      "0.2.0-alpha.1",
    );
  });

  it("follows the SemVer prerelease precedence sequence", () => {
    const versions = [
      "1.0.0-alpha",
      "1.0.0-alpha.1",
      "1.0.0-alpha.beta",
      "1.0.0-beta",
      "1.0.0-beta.2",
      "1.0.0-beta.11",
      "1.0.0-rc.1",
      "1.0.0",
    ];

    for (let index = 1; index < versions.length; index += 1) {
      expect(
        compareSemver(versions[index], versions[index - 1]),
      ).toBeGreaterThan(0);
    }
  });

  it("returns a no-op when none leaves a released identity unchanged", () => {
    const rootDir = createFixture();

    expect(readReleaseMetadata(rootDir, `v${initialVersion}`, "none")).toEqual({
      version: initialVersion,
      tag: `v${initialVersion}`,
      releaseRequired: false,
      releaseNotes: "",
    });
  });

  it("treats a one-argument none as the no-tag sentinel, not an intent", () => {
    const rootDir = createFixture();
    const result = spawnSync(
      process.execPath,
      [join(process.cwd(), "scripts/release-metadata.mjs"), "none"],
      { cwd: rootDir, encoding: "utf8" },
    );

    expect(result.status).toBe(0);
    expect(JSON.parse(result.stdout)).toMatchObject({
      version: initialVersion,
      releaseRequired: true,
      releaseNotes: initialSection,
    });
  });

  it.each([
    [
      "mismatched version owners",
      { "src-tauri/Cargo.toml": '[package]\nversion = "0.1.0-alpha.2"\n' },
      null,
      "minor",
      "Release version mismatch",
    ],
    [
      "a missing dated changelog section",
      {
        "CHANGELOG.md": "# Changelog\n\n## [Unreleased]\n",
      },
      null,
      "minor",
      "must immediately follow Unreleased",
    ],
    [
      "duplicate dated changelog sections",
      {
        "CHANGELOG.md": `# Changelog\n\n## [Unreleased]\n\n${initialSection}\n\n${initialSection}`,
      },
      null,
      "minor",
      "duplicate dated sections",
    ],
    [
      "an invalid prerelease identifier",
      {
        "package.json": JSON.stringify({ version: "0.1.0-alpha.01" }),
        "src-tauri/Cargo.toml": '[package]\nversion = "0.1.0-alpha.01"\n',
        "src-tauri/Cargo.lock":
          '[[package]]\nname = "app"\nversion = "0.1.0-alpha.01"\n',
        "src-tauri/tauri.conf.json": JSON.stringify({
          version: "0.1.0-alpha.01",
        }),
        "bridge/FmDataBridge.csproj": "<Version>0.1.0-alpha.01</Version>",
      },
      null,
      "minor",
      "not SemVer",
    ],
    [
      "a non-increasing patch release",
      {},
      `v${initialVersion}`,
      "patch",
      "does not match expected",
    ],
  ])("rejects %s", (_name, overrides, latestTag, intent, message) => {
    const rootDir = createFixture(overrides);

    expect(() => readReleaseMetadata(rootDir, latestTag, intent)).toThrow(
      message,
    );
  });

  it("stops for a major compatibility decision", () => {
    expect(() => calculateExpectedVersion(initialVersion, "major")).toThrow(
      "requires a maintainer decision",
    );
  });

  it("rejects a version lower than the latest tag without an intent", () => {
    const rootDir = createFixture();

    expect(() => readReleaseMetadata(rootDir, "v0.1.0-alpha.2")).toThrow(
      "must be greater",
    );
  });

  it("rejects a target section that is not immediately after Unreleased", () => {
    const rootDir = createFixture({
      "CHANGELOG.md": `# Changelog

## [Unreleased]

## [0.1.0-alpha.2] - 2026-08-15

### Added

- Newer prepared release.

${initialSection}`,
    });

    expect(() => readReleaseMetadata(rootDir, null, "minor")).toThrow(
      "must immediately follow Unreleased",
    );
  });

  it("rejects a malformed H2 section between Unreleased and the target", () => {
    const rootDir = createFixture({
      "CHANGELOG.md": `# Changelog

## [Unreleased]

## [0.1.0-alpha.2] - invalid-date

### Added

- Malformed newer release.

${initialSection}`,
    });

    expect(() => readReleaseMetadata(rootDir, null, "minor")).toThrow(
      "must immediately follow Unreleased",
    );
  });
});
