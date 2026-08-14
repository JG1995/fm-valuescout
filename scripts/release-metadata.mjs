import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const semverPattern =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$/;
const releaseIntents = new Set(["none", "patch", "minor", "major"]);

function readFile(path) {
  if (!existsSync(path)) {
    throw new Error(`Release input is missing: ${path}`);
  }

  return readFileSync(path, "utf8");
}

function readJsonVersion(path) {
  const version = JSON.parse(readFile(path)).version;

  if (typeof version !== "string") {
    throw new Error(`Release version is missing from ${path}`);
  }

  return version;
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

function parseSemver(version) {
  const match = semverPattern.exec(version);

  if (!match) {
    throw new Error(`Release version is not SemVer: ${version}`);
  }

  return {
    major: match[1],
    minor: match[2],
    patch: match[3],
    prerelease: match[4]?.split(".") ?? [],
  };
}

function compareNumericIdentifiers(left, right) {
  const leftNumber = BigInt(left);
  const rightNumber = BigInt(right);

  return leftNumber === rightNumber ? 0 : leftNumber > rightNumber ? 1 : -1;
}

export function compareSemver(leftVersion, rightVersion) {
  const left = parseSemver(leftVersion);
  const right = parseSemver(rightVersion);

  for (const field of ["major", "minor", "patch"]) {
    const comparison = compareNumericIdentifiers(left[field], right[field]);

    if (comparison !== 0) {
      return comparison;
    }
  }

  if (left.prerelease.length === 0 || right.prerelease.length === 0) {
    if (left.prerelease.length === right.prerelease.length) {
      return 0;
    }

    return left.prerelease.length === 0 ? 1 : -1;
  }

  const identifiers = Math.max(left.prerelease.length, right.prerelease.length);

  for (let index = 0; index < identifiers; index += 1) {
    const leftIdentifier = left.prerelease[index];
    const rightIdentifier = right.prerelease[index];

    if (leftIdentifier === undefined || rightIdentifier === undefined) {
      return leftIdentifier === undefined ? -1 : 1;
    }

    const leftNumeric = /^\d+$/.test(leftIdentifier);
    const rightNumeric = /^\d+$/.test(rightIdentifier);

    if (leftNumeric && rightNumeric) {
      const comparison = compareNumericIdentifiers(
        leftIdentifier,
        rightIdentifier,
      );

      if (comparison !== 0) {
        return comparison;
      }
      continue;
    }

    if (leftNumeric !== rightNumeric) {
      return leftNumeric ? -1 : 1;
    }

    if (leftIdentifier !== rightIdentifier) {
      return leftIdentifier > rightIdentifier ? 1 : -1;
    }
  }

  return 0;
}

function increment(identifier) {
  return (BigInt(identifier) + 1n).toString();
}

export function calculateExpectedVersion(latestVersion, intent) {
  if (!releaseIntents.has(intent)) {
    throw new Error(`Release intent is invalid: ${intent}`);
  }

  if (intent === "major") {
    throw new Error("A major release requires a maintainer decision");
  }

  if (intent === "none") {
    return latestVersion;
  }

  if (latestVersion === null) {
    if (intent !== "minor") {
      throw new Error("The initial prerelease requires the minor intent");
    }

    return "0.1.0-alpha.1";
  }

  const latest = parseSemver(latestVersion);

  if (intent === "minor") {
    return `${latest.major}.${increment(latest.minor)}.0-alpha.1`;
  }

  if (
    latest.prerelease.length === 2 &&
    latest.prerelease[0] === "alpha" &&
    /^\d+$/.test(latest.prerelease[1])
  ) {
    return `${latest.major}.${latest.minor}.${latest.patch}-alpha.${increment(
      latest.prerelease[1],
    )}`;
  }

  return `${latest.major}.${latest.minor}.${increment(latest.patch)}-alpha.1`;
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
  parseSemver(version);
  return version;
}

function extractDatedSection(changelog, version) {
  const headings = [...changelog.matchAll(/^## .+$/gm)];
  const unreleased = headings.filter(
    (heading) => heading[0].trimEnd() === "## [Unreleased]",
  );

  if (unreleased.length !== 1) {
    throw new Error("CHANGELOG.md must contain one Unreleased section");
  }

  const unreleasedIndex = headings.indexOf(unreleased[0]);
  const match = headings[unreleasedIndex + 1];
  const escapedVersion = version.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const targetHeading = new RegExp(
    `^## \\[${escapedVersion}\\] - \\d{4}-\\d{2}-\\d{2}\\s*$`,
  );

  if (match === undefined || !targetHeading.test(match[0])) {
    throw new Error(
      `CHANGELOG.md dated section for ${version} must immediately follow Unreleased`,
    );
  }

  if (
    headings.filter((heading) => targetHeading.test(heading[0])).length !== 1
  ) {
    throw new Error(`CHANGELOG.md has duplicate dated sections for ${version}`);
  }

  const end = headings[unreleasedIndex + 2]?.index ?? changelog.length;

  return changelog.slice(match.index, end).trimEnd();
}

function normalizeLatestTag(latestTag) {
  if (latestTag === null) {
    return null;
  }

  if (typeof latestTag !== "string" || !latestTag.startsWith("v")) {
    throw new Error("Latest release tag must use the v<version> form");
  }

  const version = latestTag.slice(1);
  parseSemver(version);
  return version;
}

export function readReleaseMetadata(rootDir, latestTag = null, intent) {
  const version = validateReleaseIdentity(rootDir);
  const latestVersion = normalizeLatestTag(latestTag);

  if (intent !== undefined && !releaseIntents.has(intent)) {
    throw new Error(`Release intent is invalid: ${intent}`);
  }

  if (intent === "none") {
    if (latestVersion !== null && compareSemver(version, latestVersion) !== 0) {
      throw new Error(
        "Release intent none requires an unchanged release version",
      );
    }

    return {
      version,
      tag: `v${version}`,
      releaseRequired: false,
      releaseNotes: "",
    };
  }

  if (intent !== undefined) {
    const expected = calculateExpectedVersion(latestVersion, intent);

    if (version !== expected) {
      throw new Error(
        `Release version ${version} does not match expected ${expected} for ${intent}`,
      );
    }
  }

  if (latestVersion !== null) {
    const precedence = compareSemver(version, latestVersion);

    if (precedence < 0) {
      throw new Error(
        `Release version ${version} must be greater than latest tag ${latestTag}`,
      );
    }

    if (precedence === 0) {
      return {
        version,
        tag: `v${version}`,
        releaseRequired: false,
        releaseNotes: "",
      };
    }
  }

  return {
    version,
    tag: `v${version}`,
    releaseRequired: true,
    releaseNotes: extractDatedSection(
      readFile(join(rootDir, "CHANGELOG.md")),
      version,
    ),
  };
}

function parseCliArguments(args) {
  if (args.length === 0) {
    return { latestTag: null, intent: undefined };
  }

  if (args.length === 1) {
    return {
      latestTag: args[0] === "none" ? null : args[0],
      intent: undefined,
    };
  }

  if (args.length === 2) {
    return {
      latestTag: args[0] === "none" ? null : args[0],
      intent: args[1],
    };
  }

  throw new Error("Usage: release-metadata [latest-tag|none] [release-intent]");
}

function main() {
  const { latestTag, intent } = parseCliArguments(process.argv.slice(2));
  const metadata = readReleaseMetadata(process.cwd(), latestTag, intent);
  process.stdout.write(`${JSON.stringify(metadata)}\n`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main();
}
