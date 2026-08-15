import { appendFileSync } from "node:fs";

const allowedEcosystems = new Set(["cargo", "npm_and_yarn"]);
const stableSemver =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:\+(?:[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$/;

function parseStableSemver(version) {
  const match = stableSemver.exec(version);

  if (!match) {
    return null;
  }

  return {
    major: BigInt(match[1]),
    minor: BigInt(match[2]),
    patch: BigInt(match[3]),
  };
}

function isEligibleDependency(dependency, expectedEcosystem, expectedBaseRef) {
  if (!dependency || typeof dependency !== "object") {
    return false;
  }

  if (
    typeof dependency.dependencyName !== "string" ||
    dependency.dependencyName.length === 0 ||
    dependency.updateType !== "version-update:semver-patch" ||
    dependency.packageEcosystem !== expectedEcosystem ||
    dependency.targetBranch !== expectedBaseRef ||
    dependency.maintainerChanges !== false
  ) {
    return false;
  }

  const previous = parseStableSemver(dependency.prevVersion);
  const next = parseStableSemver(dependency.newVersion);

  if (previous === null || next === null) {
    return false;
  }

  // Cargo treats the leftmost non-zero component as its compatibility boundary.
  const hasCompatiblePatchLine =
    previous.major >= 1n ||
    (expectedEcosystem === "cargo" &&
      previous.major === 0n &&
      previous.minor > 0n);

  return (
    hasCompatiblePatchLine &&
    previous.major === next.major &&
    previous.minor === next.minor &&
    next.patch > previous.patch
  );
}

export function evaluateDependencyAutomergePolicy(input) {
  if (
    input.actor !== "dependabot[bot]" ||
    input.pullRequestAuthor !== "dependabot[bot]"
  ) {
    return { eligible: false, reason: "unverified-author" };
  }

  if (
    input.repository !== input.expectedRepository ||
    input.baseRepository !== input.expectedRepository
  ) {
    return { eligible: false, reason: "wrong-repository" };
  }

  if (
    input.baseRef !== input.expectedBaseRef ||
    input.targetBranch !== input.expectedBaseRef
  ) {
    return { eligible: false, reason: "wrong-base" };
  }

  if (
    !allowedEcosystems.has(input.packageEcosystem) ||
    input.updateType !== "version-update:semver-patch" ||
    input.maintainerChanges !== "false"
  ) {
    return { eligible: false, reason: "ineligible-summary" };
  }

  let dependencies;

  try {
    dependencies = JSON.parse(input.updatedDependenciesJson);
  } catch {
    return { eligible: false, reason: "invalid-metadata" };
  }

  if (
    !Array.isArray(dependencies) ||
    dependencies.length === 0 ||
    !dependencies.every((dependency) =>
      isEligibleDependency(
        dependency,
        input.packageEcosystem,
        input.expectedBaseRef,
      ),
    )
  ) {
    return { eligible: false, reason: "ineligible-dependency" };
  }

  return { eligible: true, reason: "eligible" };
}

function writeResult(result) {
  const outputPath = process.env.GITHUB_OUTPUT;

  if (outputPath) {
    appendFileSync(outputPath, `eligible=${result.eligible}\n`);
  }

  console.log(`Dependabot auto-merge policy: ${result.reason}`);

  if (process.env.DEPENDABOT_REQUIRE_ELIGIBLE === "true" && !result.eligible) {
    process.exitCode = 1;
  }
}

if (process.argv[1] === new URL(import.meta.url).pathname) {
  writeResult(
    evaluateDependencyAutomergePolicy({
      actor: process.env.DEPENDABOT_ACTOR,
      pullRequestAuthor: process.env.DEPENDABOT_PULL_REQUEST_AUTHOR,
      repository: process.env.DEPENDABOT_REPOSITORY,
      baseRepository: process.env.DEPENDABOT_BASE_REPOSITORY,
      baseRef: process.env.DEPENDABOT_BASE_REF,
      expectedRepository: process.env.DEPENDABOT_EXPECTED_REPOSITORY,
      expectedBaseRef: process.env.DEPENDABOT_EXPECTED_BASE_REF,
      packageEcosystem: process.env.DEPENDABOT_PACKAGE_ECOSYSTEM,
      targetBranch: process.env.DEPENDABOT_TARGET_BRANCH,
      updateType: process.env.DEPENDABOT_UPDATE_TYPE,
      maintainerChanges: process.env.DEPENDABOT_MAINTAINER_CHANGES,
      updatedDependenciesJson: process.env.DEPENDABOT_UPDATED_DEPENDENCIES,
    }),
  );
}
