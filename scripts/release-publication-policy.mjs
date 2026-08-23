import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

function requireString(value, name) {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`Release publication ${name} is missing`);
  }
}

function requireExpected(expected) {
  requireString(expected?.version, "version");
  requireString(expected?.tag, "tag");
  requireString(expected?.title, "title");
  requireString(expected?.notes, "notes");

  if (expected.tag !== `v${expected.version}`) {
    throw new Error("Release publication tag does not match its version");
  }
}

function requireMetadata(expected, metadata) {
  if (
    !metadata ||
    metadata.version !== expected.version ||
    metadata.tag !== expected.tag ||
    typeof metadata.releaseRequired !== "boolean"
  ) {
    throw new Error("Release metadata changed after the verified Check run");
  }

  if (metadata.releaseRequired && metadata.releaseNotes !== expected.notes) {
    throw new Error("Release notes changed after the verified Check run");
  }
}

function expectedPrerelease(expected) {
  return expected.version.includes("-");
}

function requireReleaseIdentity(expected, release) {
  if (
    !release ||
    release.tag_name !== expected.tag ||
    release.name !== expected.title ||
    release.body !== expected.notes ||
    release.prerelease !== expectedPrerelease(expected)
  ) {
    throw new Error(
      "Existing release metadata does not match the checked source",
    );
  }
}

/**
 * Chooses the only allowed publication action from read-only GitHub state.
 * The workflow must call this before it packages, removes draft assets, or
 * changes a release.
 */
export function evaluatePublicationState({
  expected,
  metadata,
  existingRelease,
  tagSha,
}) {
  requireExpected(expected);
  requireMetadata(expected, metadata);

  if (!metadata.releaseRequired) {
    requireReleaseIdentity(expected, existingRelease);
    if (existingRelease.draft || tagSha !== existingRelease.target_commitish) {
      throw new Error("Unchanged release is not a matching published tag");
    }
    if (
      expected.verifiedSha !== null &&
      expected.verifiedSha !== undefined &&
      existingRelease.target_commitish !== expected.verifiedSha
    ) {
      throw new Error("Published retry does not target the verified commit");
    }
    return { mode: "no-op" };
  }

  requireString(expected.verifiedSha, "verified SHA");

  if (existingRelease === null) {
    if (tagSha !== null) {
      throw new Error("Git tag exists without a matching draft release");
    }
    return { mode: "create" };
  }

  if (
    existingRelease.draft !== true ||
    existingRelease.tag_name !== expected.tag ||
    existingRelease.target_commitish !== expected.verifiedSha ||
    (tagSha !== null && tagSha !== expected.verifiedSha)
  ) {
    throw new Error(
      "Existing release cannot be repaired for this verified commit",
    );
  }

  return { mode: "repair" };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [inputPath] = process.argv.slice(2);

  if (inputPath === undefined || process.argv.length !== 3) {
    throw new Error("Usage: release-publication-policy <input.json>");
  }

  let input;
  try {
    input = JSON.parse(readFileSync(inputPath, "utf8"));
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`Release publication input is invalid: ${message}`);
  }
  process.stdout.write(`${JSON.stringify(evaluatePublicationState(input))}\n`);
}
