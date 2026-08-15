import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { describe, expect, it } from "vitest";

import { evaluatePublicationState } from "./release-publication-policy.mjs";

const version = "0.1.0-alpha.1";
const tag = `v${version}`;
const sha = "a".repeat(40);
const notes = "## [0.1.0-alpha.1] - 2026-08-14\n\n- Initial release.";
const expected = {
  version,
  tag,
  title: `FM ValueScout ${tag}`,
  notes,
  verifiedSha: sha,
};

function metadata(releaseRequired: boolean) {
  return {
    version,
    tag,
    releaseRequired,
    releaseNotes: releaseRequired ? notes : "",
  };
}

function release(overrides: Record<string, unknown> = {}) {
  return {
    tag_name: tag,
    target_commitish: sha,
    name: expected.title,
    body: notes,
    draft: false,
    prerelease: true,
    ...overrides,
  };
}

describe("release publication policy", () => {
  it("exposes the state decision through its read-only JSON CLI", () => {
    const temporaryDirectory = mkdtempSync(
      join(tmpdir(), "fm-valuescout-release-policy-"),
    );
    const inputPath = join(temporaryDirectory, "input.json");
    writeFileSync(
      inputPath,
      JSON.stringify({
        expected,
        metadata: metadata(false),
        existingRelease: release(),
        tagSha: sha,
      }),
    );

    try {
      const result = spawnSync(
        process.execPath,
        [
          join(process.cwd(), "scripts/release-publication-policy.mjs"),
          inputPath,
        ],
        { encoding: "utf8" },
      );

      expect(result.status).toBe(0);
      expect(JSON.parse(result.stdout)).toEqual({ mode: "no-op" });
    } finally {
      rmSync(temporaryDirectory, { force: true, recursive: true });
    }
  });

  it("makes an exact published retry a no-op", () => {
    expect(
      evaluatePublicationState({
        expected,
        metadata: metadata(false),
        existingRelease: release(),
        tagSha: sha,
      }),
    ).toEqual({ mode: "no-op" });
  });

  it("creates a newer version only when its tag is absent", () => {
    expect(
      evaluatePublicationState({
        expected,
        metadata: metadata(true),
        existingRelease: null,
        tagSha: null,
        releaseSourcePrepared: true,
      }),
    ).toEqual({ mode: "create" });
  });

  it("defers a later same-version SHA without a new release preparation", () => {
    expect(
      evaluatePublicationState({
        expected: { ...expected, verifiedSha: "b".repeat(40) },
        metadata: metadata(true),
        existingRelease: null,
        tagSha: null,
        releaseSourcePrepared: false,
      }),
    ).toEqual({ mode: "defer" });
  });

  it.each([
    ["orphaned tag", null, sha],
    ["temporary draft", release({ draft: true }), null],
  ])(
    "rejects a deferred release with a %s",
    (_name, existingRelease, tagSha) => {
      expect(() =>
        evaluatePublicationState({
          expected: { ...expected, verifiedSha: "b".repeat(40) },
          metadata: metadata(true),
          existingRelease,
          tagSha,
          releaseSourcePrepared: false,
        }),
      ).toThrow();
    },
  );

  it("repairs only a temporary draft for the same version and SHA", () => {
    expect(
      evaluatePublicationState({
        expected,
        metadata: metadata(true),
        existingRelease: release({ draft: true, body: "failed attempt" }),
        tagSha: null,
        releaseSourcePrepared: true,
      }),
    ).toEqual({ mode: "repair" });
  });

  it.each([
    [
      "mismatched draft",
      release({ draft: true, target_commitish: "b".repeat(40) }),
      null,
    ],
    ["mismatched published release", release({ body: "wrong notes" }), sha],
    ["orphaned tag", null, sha],
  ])("rejects a %s before mutation", (_name, existingRelease, tagSha) => {
    expect(() =>
      evaluatePublicationState({
        expected,
        metadata: metadata(true),
        existingRelease,
        tagSha,
        releaseSourcePrepared: true,
      }),
    ).toThrow();
  });
});
