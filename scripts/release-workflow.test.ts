import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

const checkWorkflow = readFileSync(".github/workflows/check.yml", "utf8");
const releaseWorkflow = readFileSync(".github/workflows/release.yml", "utf8");

describe("verified-main release workflow", () => {
  it("only follows successful Check runs from pushes to main", () => {
    expect(releaseWorkflow).toContain("workflow_run:");
    expect(releaseWorkflow).toContain('workflows: ["Check"]');
    expect(releaseWorkflow).toContain(
      "github.event.workflow_run.conclusion == 'success'",
    );
    expect(releaseWorkflow).toContain(
      "github.event.workflow_run.event == 'push'",
    );
    expect(releaseWorkflow).toContain(
      "github.event.workflow_run.head_branch == 'main'",
    );
    expect(releaseWorkflow).toContain(
      `ref: \${{ github.event.workflow_run.head_sha }}`,
    );
  });

  it("keeps release validation and candidate packaging read-only", () => {
    expect(releaseWorkflow).toContain("permissions:\n  contents: read");
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

  it("requires metadata validation and the same Windows candidate command in Check", () => {
    expect(checkWorkflow).toContain("release:");
    expect(checkWorkflow).toContain("release-candidate:");
    expect(checkWorkflow).toContain(
      "& bash ./scripts/dev release-metadata $latestTag",
    );
    expect(
      checkWorkflow.match(
        /& bash \.\/scripts\/dev release-metadata \$latestTag/g,
      ),
    ).toHaveLength(1);
    expect(checkWorkflow).toContain("./scripts/dev package-windows");
    expect(checkWorkflow).toContain("RELEASE_CANDIDATE:");
    expect(checkWorkflow).toContain("include-hidden-files: true");
    expect(checkWorkflow).toContain(
      "dorny/paths-filter@ceb8a2b8f2d89434be7ff52d3de7ec3738c5cc9d",
    );
  });

  it("stages a checked release before publishing one Windows prerelease", () => {
    expect(releaseWorkflow).toContain("draft = $true");
    expect(releaseWorkflow).toContain("prerelease = $true");
    expect(releaseWorkflow).toContain("draft = $false");
    expect(releaseWorkflow).toContain("Get-FileHash");
    expect(releaseWorkflow).toContain("Invoke-Gh release download");
    expect(releaseWorkflow).not.toContain("matrix:");
    expect(releaseWorkflow).not.toContain("tauri-apps/tauri-action");
  });
});
