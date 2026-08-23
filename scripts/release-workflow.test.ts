import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

const checkWorkflow = readFileSync(".github/workflows/check.yml", "utf8");
const releaseWorkflow = readFileSync(".github/workflows/release.yml", "utf8");

describe("explicit release workflow", () => {
  it("does not run release-only validation from Check", () => {
    expect(checkWorkflow).not.toContain("release-validation:");
    expect(checkWorkflow).not.toContain("RELEASE_VALIDATION:");
    expect(checkWorkflow).not.toContain("./scripts/dev package-windows");
    expect(checkWorkflow).not.toContain("./scripts/dev release-metadata");
    expect(checkWorkflow).not.toContain("release:");
    expect(checkWorkflow).toContain(
      "dorny/paths-filter@ceb8a2b8f2d89434be7ff52d3de7ec3738c5cc9d",
    );
  });

  it("starts only when an explicit release preparation reaches main", () => {
    const trigger = releaseWorkflow.slice(
      releaseWorkflow.indexOf("on:\n"),
      releaseWorkflow.indexOf("\nconcurrency:"),
    );

    expect(trigger).toBe(
      "on:\n  push:\n    branches: [main]\n    paths:\n      - release-preparation.json\n",
    );
  });

  it("waits for the exact main Check before it validates or packages", () => {
    expect(releaseWorkflow).toContain("actions: read");
    const prepareWorkflow = releaseWorkflow.slice(
      releaseWorkflow.indexOf("  prepare:"),
      releaseWorkflow.indexOf("  publish:"),
    );
    const publishWorkflow = releaseWorkflow.slice(
      releaseWorkflow.indexOf("  publish:"),
    );
    const waitInvocation = "\n          Wait-ForVerifiedCheck\n";
    const waitIndex = prepareWorkflow.indexOf(waitInvocation);

    expect(prepareWorkflow).toContain(
      "Invoke-Gh run list --workflow Check --branch main --event push --commit $env:VERIFIED_SHA",
    );
    const waitFunction = prepareWorkflow.slice(
      prepareWorkflow.indexOf("function Wait-ForVerifiedCheck"),
      prepareWorkflow.indexOf("function Get-Releases"),
    );

    expect(waitFunction).toContain(
      'if ($run.conclusion -cne "success") {\n                throw "Exact main Check did not succeed"',
    );
    expect(
      waitFunction.indexOf("Exact main Check did not succeed"),
    ).toBeLessThan(waitFunction.lastIndexOf("return"));
    expect(prepareWorkflow).toContain("VERIFIED_SHA: $" + "{{ github.sha }}");
    expect(prepareWorkflow).toContain(`ref: \${{ github.sha }}`);
    expect(waitIndex).toBeGreaterThan(-1);
    expect(prepareWorkflow.indexOf(waitInvocation, waitIndex + 1)).toBe(-1);
    expect(waitIndex).toBeLessThan(
      prepareWorkflow.indexOf(
        "& bash ./scripts/dev release-metadata $latestTag",
      ),
    );
    expect(publishWorkflow).toContain("needs: prepare");
    expect(publishWorkflow).toContain("./scripts/dev package-windows");
  });

  it("keeps release validation and packaging read-only until publication", () => {
    expect(releaseWorkflow).toContain(
      "permissions:\n  actions: read\n  contents: read",
    );
    expect(releaseWorkflow.match(/contents: write/g)).toHaveLength(1);
    expect(releaseWorkflow).toContain("publish:");
    expect(releaseWorkflow).toContain(
      "& bash ./scripts/dev release-metadata $latestTag",
    );
    expect(
      releaseWorkflow.match(
        /& bash \.\/scripts\/dev release-metadata \$latestTag/g,
      ),
    ).toHaveLength(2);
    expect(releaseWorkflow).toContain(
      "node scripts/release-publication-policy.mjs",
    );
    const publishWorkflow = releaseWorkflow.slice(
      releaseWorkflow.indexOf("  publish:"),
    );
    expect(
      publishWorkflow.match(/\$decision = Get-PublicationDecision/g),
    ).toHaveLength(2);
    expect(
      publishWorkflow.indexOf("$decision = Get-PublicationDecision"),
    ).toBeLessThan(
      publishWorkflow.indexOf("& bash ./scripts/dev package-windows"),
    );
    expect(
      publishWorkflow.indexOf("foreach ($asset in @($existing.assets))"),
    ).toBeGreaterThan(
      publishWorkflow.lastIndexOf("$decision = Get-PublicationDecision"),
    );
    expect(releaseWorkflow).toContain("./scripts/dev package-windows");
  });

  it("preserves absent GitHub SHAs as JSON null in both release jobs", () => {
    expect(
      releaseWorkflow.match(
        /\[AllowNull\(\)\]\[object\]\$TagSha, \[AllowNull\(\)\]\[object\]\$VerifiedSha/g,
      ),
    ).toHaveLength(2);
    expect(releaseWorkflow).not.toContain("[AllowNull()][string]$TagSha");
  });

  it("stages a checked release before publishing one Windows release", () => {
    expect(releaseWorkflow).toContain("draft = $true");
    expect(releaseWorkflow).toContain(
      '$expectedPrerelease = $expectedVersion.Contains("-")',
    );
    expect(releaseWorkflow).toContain(
      '$makeLatest = if ($expectedPrerelease) { "false" } else { "true" }',
    );
    expect(releaseWorkflow).toContain("prerelease = $expectedPrerelease");
    expect(releaseWorkflow).not.toContain("prerelease = $true");
    expect(releaseWorkflow).toContain(
      'draft = $true\n            prerelease = $expectedPrerelease\n            make_latest = "false"',
    );
    expect(releaseWorkflow).toContain(
      "draft = $false\n            prerelease = $expectedPrerelease\n            make_latest = $makeLatest",
    );
    expect(releaseWorkflow).toContain("draft = $false");
    expect(releaseWorkflow).toContain("Get-FileHash");
    expect(releaseWorkflow).toContain("Invoke-Gh release download");
    expect(releaseWorkflow).not.toContain("matrix:");
    expect(releaseWorkflow).not.toContain("tauri-apps/tauri-action");
  });
});
