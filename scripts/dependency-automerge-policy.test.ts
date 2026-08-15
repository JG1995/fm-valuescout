import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { evaluateDependencyAutomergePolicy } from "./dependency-automerge-policy.mjs";

const dependency = {
  dependencyName: "nanoid",
  updateType: "version-update:semver-patch",
  packageEcosystem: "npm_and_yarn",
  targetBranch: "main",
  prevVersion: "3.3.17",
  newVersion: "3.3.18",
  maintainerChanges: false,
};

const eligibleInput = {
  actor: "dependabot[bot]",
  pullRequestAuthor: "dependabot[bot]",
  repository: "JG1995/fm-valuescout",
  baseRepository: "JG1995/fm-valuescout",
  baseRef: "main",
  expectedRepository: "JG1995/fm-valuescout",
  expectedBaseRef: "main",
  packageEcosystem: "npm_and_yarn",
  targetBranch: "main",
  updateType: "version-update:semver-patch",
  maintainerChanges: "false",
  updatedDependenciesJson: JSON.stringify([dependency]),
};

function evaluate(overrides: Record<string, unknown> = {}) {
  return evaluateDependencyAutomergePolicy({ ...eligibleInput, ...overrides });
}

function cargoDependency(
  dependencyName: string,
  prevVersion: string,
  newVersion: string,
) {
  return {
    ...dependency,
    dependencyName,
    packageEcosystem: "cargo",
    prevVersion,
    newVersion,
  };
}

describe("evaluateDependencyAutomergePolicy", () => {
  it("accepts stable patch-only pnpm and Cargo updates", () => {
    expect(evaluate()).toEqual({ eligible: true, reason: "eligible" });
    expect(
      evaluate({
        packageEcosystem: "cargo",
        updatedDependenciesJson: JSON.stringify([
          { ...dependency, packageEcosystem: "cargo" },
        ]),
      }),
    ).toEqual({ eligible: true, reason: "eligible" });
  });

  it("accepts grouped Cargo patches on compatible zero-major lines", () => {
    expect(
      evaluate({
        packageEcosystem: "cargo",
        updatedDependenciesJson: JSON.stringify([
          cargoDependency("camino", "1.2.4", "1.2.5"),
          cargoDependency("displaydoc", "0.2.6", "0.2.7"),
          cargoDependency("tao-macros", "0.1.3", "0.1.4"),
          cargoDependency(
            "toml_parser",
            "1.1.2+spec-1.1.0",
            "1.1.3+spec-1.1.0",
          ),
        ]),
      }),
    ).toEqual({ eligible: true, reason: "eligible" });
  });

  it.each([
    ["minor", { updateType: "version-update:semver-minor" }],
    ["major", { updateType: "version-update:semver-major" }],
    [
      "pre-1.0.0 pnpm dependency",
      {
        updatedDependenciesJson: JSON.stringify([
          { ...dependency, prevVersion: "0.9.0", newVersion: "0.9.1" },
        ]),
      },
    ],
    [
      "Cargo 0.0 patch",
      {
        packageEcosystem: "cargo",
        updatedDependenciesJson: JSON.stringify([
          cargoDependency("zerocopy", "0.0.3", "0.0.4"),
        ]),
      },
    ],
    [
      "Cargo zero-major minor change disguised as a patch",
      {
        packageEcosystem: "cargo",
        updatedDependenciesJson: JSON.stringify([
          cargoDependency("tao-macros", "0.1.3", "0.2.0"),
        ]),
      },
    ],
    [
      "Cargo oversized minor change disguised as a patch",
      {
        packageEcosystem: "cargo",
        updatedDependenciesJson: JSON.stringify([
          cargoDependency(
            "tao-macros",
            "0.9007199254740992.1",
            "0.9007199254740993.2",
          ),
        ]),
      },
    ],
    [
      "prerelease",
      {
        updatedDependenciesJson: JSON.stringify([
          { ...dependency, newVersion: "3.3.18-rc.1" },
        ]),
      },
    ],
    [
      "malformed version",
      {
        updatedDependenciesJson: JSON.stringify([
          { ...dependency, newVersion: "3.3" },
        ]),
      },
    ],
    ["malformed metadata", { updatedDependenciesJson: "{" }],
    [
      "malformed dependency",
      {
        updatedDependenciesJson: JSON.stringify([
          { ...dependency, dependencyName: "" },
        ]),
      },
    ],
    [
      "patch downgrade",
      {
        updatedDependenciesJson: JSON.stringify([
          { ...dependency, newVersion: "3.3.16" },
        ]),
      },
    ],
    [
      "GitHub Actions",
      {
        packageEcosystem: "github-actions",
        updatedDependenciesJson: JSON.stringify([
          { ...dependency, packageEcosystem: "github-actions" },
        ]),
      },
    ],
    [
      "NuGet",
      {
        packageEcosystem: "nuget",
        updatedDependenciesJson: JSON.stringify([
          { ...dependency, packageEcosystem: "nuget" },
        ]),
      },
    ],
    ["non-Dependabot actor", { actor: "octocat" }],
    ["non-Dependabot author", { pullRequestAuthor: "octocat" }],
    ["wrong repository", { repository: "octocat/fork" }],
    ["wrong base repository", { baseRepository: "octocat/fork" }],
    ["wrong base", { baseRef: "release" }],
    ["wrong metadata base", { targetBranch: "release" }],
    ["maintainer changes", { maintainerChanges: "true" }],
  ])("rejects %s metadata", (_name, overrides) => {
    expect(evaluate(overrides)).toMatchObject({ eligible: false });
  });

  it("rejects a grouped update when the highest update type hides a minor entry", () => {
    expect(
      evaluate({
        updatedDependenciesJson: JSON.stringify([
          dependency,
          {
            ...dependency,
            dependencyName: "vite",
            updateType: "version-update:semver-minor",
            prevVersion: "8.1.5",
            newVersion: "8.2.0",
          },
        ]),
      }),
    ).toMatchObject({ eligible: false });
  });

  it("rejects mixed ecosystems and per-dependency maintainer changes", () => {
    expect(
      evaluate({
        updatedDependenciesJson: JSON.stringify([
          dependency,
          { ...dependency, dependencyName: "serde", packageEcosystem: "cargo" },
        ]),
      }),
    ).toMatchObject({ eligible: false });

    expect(
      evaluate({
        updatedDependenciesJson: JSON.stringify([
          { ...dependency, maintainerChanges: true },
        ]),
      }),
    ).toMatchObject({ eligible: false });
  });

  it("revokes prior auto-merge when Dependabot metadata becomes ineligible", () => {
    const workflow = readFileSync(
      ".github/workflows/dependabot-automerge.yml",
      "utf8",
    );

    expect(workflow).toContain(
      "types: [opened, reopened, synchronize, edited]",
    );
    expect(workflow).not.toContain("github.actor == 'dependabot[bot]'");
    expect(workflow).toContain("continue-on-error: true");
    expect(workflow).toContain("always() &&");
    expect(workflow).toContain("gh pr merge --disable-auto");
  });

  it("fails the required check for ineligible metadata", () => {
    const result = spawnSync(
      process.execPath,
      ["scripts/dependency-automerge-policy.mjs"],
      {
        cwd: process.cwd(),
        encoding: "utf8",
        env: {
          ...process.env,
          DEPENDABOT_REQUIRE_ELIGIBLE: "true",
          DEPENDABOT_ACTOR: "octocat",
        },
      },
    );

    expect(result.status).toBe(1);
  });

  it("makes Dependabot policy part of the required Check workflow", () => {
    const checkWorkflow = readFileSync(".github/workflows/check.yml", "utf8");

    expect(checkWorkflow).toContain("- 'scripts/**'");
    expect(checkWorkflow).toContain("dependabot-policy:");
    expect(checkWorkflow).toContain("DEPENDABOT_REQUIRE_ELIGIBLE: true");
    expect(checkWorkflow).toContain("DEPENDABOT_POLICY:");
  });
});
