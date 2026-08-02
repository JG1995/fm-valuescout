import { spawnSync } from "node:child_process";
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { resolveUiAgentForwardConsole } from "./ui-agent-vite";

const repoRoot = path.resolve(import.meta.dirname, "..");
const launcherPath = path.join(repoRoot, "scripts", "ui-agent");

let testRoot: string;
let temporaryProfilesRoot: string;
let captureRoot: string;
let fakeBinRoot: string;

function installPnpmStub() {
  const stubPath = path.join(fakeBinRoot, "pnpm");
  writeFileSync(
    stubPath,
    [
      "#!/usr/bin/env bash",
      "set -euo pipefail",
      'printf "%s\\n" "$*" > "$UI_AGENT_TEST_CAPTURE_ROOT/arguments"',
      'printf "%s\\n" "$FM_VALUESCOUT_UI_AGENT" > "$UI_AGENT_TEST_CAPTURE_ROOT/mode"',
      `printf "%s\\n" "\${FM_VALUESCOUT_UI_AGENT_WSL:-}" > "$UI_AGENT_TEST_CAPTURE_ROOT/wsl"`,
      'printf "%s\\n" "$FM_VALUESCOUT_UI_AGENT_DATA_DIR" > "$UI_AGENT_TEST_CAPTURE_ROOT/profile"',
      `printf "%s\\n" "\${FM_VALUESCOUT_UI_AGENT_DUMP:-}" > "$UI_AGENT_TEST_CAPTURE_ROOT/dump"`,
      'printf "temporary mutation\\n" > "$FM_VALUESCOUT_UI_AGENT_DATA_DIR/app.db"',
      `exit "\${UI_AGENT_TEST_EXIT_CODE:-0}"`,
    ].join("\n"),
  );
  chmodSync(stubPath, 0o755);
}

function runLauncher(args: string[], exitCode = 0) {
  return spawnSync(launcherPath, args, {
    cwd: repoRoot,
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: `${fakeBinRoot}${path.delimiter}${process.env.PATH ?? ""}`,
      TMPDIR: temporaryProfilesRoot,
      UI_AGENT_TEST_CAPTURE_ROOT: captureRoot,
      UI_AGENT_TEST_EXIT_CODE: String(exitCode),
    },
  });
}

beforeEach(() => {
  testRoot = mkdtempSync(
    path.join(os.tmpdir(), "fm-valuescout-ui-agent-test."),
  );
  temporaryProfilesRoot = path.join(testRoot, "profiles");
  captureRoot = path.join(testRoot, "capture");
  fakeBinRoot = path.join(testRoot, "bin");
  mkdirSync(temporaryProfilesRoot);
  mkdirSync(captureRoot);
  mkdirSync(fakeBinRoot);
  installPnpmStub();
});

afterEach(() => {
  rmSync(testRoot, { recursive: true, force: true });
});

describe("UI-agent launcher", () => {
  it("passes an absolute dump into a unique temporary profile and removes the profile", () => {
    const dumpPath = path.join(testRoot, "dump.json");
    const originalDump = '{"schemaVersion":5}\n';
    writeFileSync(dumpPath, originalDump);

    const result = runLauncher(["--dump", dumpPath]);

    expect(result.error).toBeUndefined();
    expect(result.status).toBe(0);
    expect(
      readFileSync(path.join(captureRoot, "arguments"), "utf8").trim(),
    ).toBe(
      "tauri dev --features ui-agent --config src-tauri/tauri.ui-agent.conf.json",
    );
    expect(readFileSync(path.join(captureRoot, "mode"), "utf8").trim()).toBe(
      "1",
    );
    const isWsl =
      existsSync("/proc/sys/kernel/osrelease") &&
      readFileSync("/proc/sys/kernel/osrelease", "utf8")
        .toLowerCase()
        .includes("microsoft");
    expect(readFileSync(path.join(captureRoot, "wsl"), "utf8").trim()).toBe(
      isWsl ? "1" : "",
    );
    expect(readFileSync(path.join(captureRoot, "dump"), "utf8").trim()).toBe(
      dumpPath,
    );

    const profilePath = readFileSync(
      path.join(captureRoot, "profile"),
      "utf8",
    ).trim();
    expect(path.dirname(profilePath)).toBe(temporaryProfilesRoot);
    expect(path.basename(profilePath)).toMatch(/^fm-valuescout-ui-agent\./);
    expect(existsSync(profilePath)).toBe(false);
    expect(readFileSync(dumpPath, "utf8")).toBe(originalDump);
  });

  it("creates an empty temporary profile when no dump is supplied", () => {
    const result = runLauncher([]);

    expect(result.error).toBeUndefined();
    expect(result.status).toBe(0);
    expect(readFileSync(path.join(captureRoot, "dump"), "utf8").trim()).toBe(
      "",
    );

    const profilePath = readFileSync(
      path.join(captureRoot, "profile"),
      "utf8",
    ).trim();
    expect(path.dirname(profilePath)).toBe(temporaryProfilesRoot);
    expect(existsSync(profilePath)).toBe(false);
  });

  it("removes the temporary profile when Tauri startup fails", () => {
    const result = runLauncher([], 17);

    expect(result.error).toBeUndefined();
    expect(result.status).toBe(17);
    const profilePath = readFileSync(
      path.join(captureRoot, "profile"),
      "utf8",
    ).trim();
    expect(existsSync(profilePath)).toBe(false);
  });

  it.each([
    [["--dump"], "requires an absolute dump path"],
    [["--dump", "relative/dump.json"], "requires an absolute dump path"],
    [
      ["--dump", "/definitely/missing/dump.json"],
      "requires a readable dump file",
    ],
    [["--unknown"], "accepts only --dump"],
    [["--dump", "/tmp/dump.json", "extra"], "accepts only --dump"],
  ])("rejects invalid arguments: %s", (args, expectedMessage) => {
    const result = runLauncher(args);

    expect(result.error).toBeUndefined();
    expect(result.status).toBe(2);
    expect(result.stderr).toContain(expectedMessage);
    expect(existsSync(path.join(captureRoot, "arguments"))).toBe(false);
  });
});

describe("UI-agent configuration isolation", () => {
  it("disables Vite console forwarding only for WSL UI-agent sessions", () => {
    expect(resolveUiAgentForwardConsole({})).toBeUndefined();
    expect(
      resolveUiAgentForwardConsole({
        FM_VALUESCOUT_UI_AGENT: "1",
        FM_VALUESCOUT_UI_AGENT_WSL: "1",
      }),
    ).toBe(false);
    expect(
      resolveUiAgentForwardConsole({ FM_VALUESCOUT_UI_AGENT: "1" }),
    ).toBeUndefined();
  });

  it("keeps the bridge capability and global Tauri API in the dedicated overlay", () => {
    const ordinaryConfig = JSON.parse(
      readFileSync(path.join(repoRoot, "src-tauri", "tauri.conf.json"), "utf8"),
    );
    const uiAgentConfig = JSON.parse(
      readFileSync(
        path.join(repoRoot, "src-tauri", "tauri.ui-agent.conf.json"),
        "utf8",
      ),
    );

    expect(ordinaryConfig.app.withGlobalTauri).toBeUndefined();
    expect(ordinaryConfig.app.security.capabilities).toBeUndefined();
    expect(uiAgentConfig.app.withGlobalTauri).toBe(true);
    expect(uiAgentConfig.app.security.capabilities).toEqual([
      "default",
      {
        identifier: "ui-agent",
        description:
          "Disposable loopback control for trusted UI-polish sessions",
        windows: ["main"],
        permissions: ["mcp-bridge:default"],
      },
    ]);
  });

  it("pins the upstream bridge and CLI without registering a Codex MCP server", () => {
    const packageJson = JSON.parse(
      readFileSync(path.join(repoRoot, "package.json"), "utf8"),
    );
    const cargoToml = readFileSync(
      path.join(repoRoot, "src-tauri", "Cargo.toml"),
      "utf8",
    );
    const codexConfig = readFileSync(
      path.join(repoRoot, ".codex", "config.toml"),
      "utf8",
    );
    const workflowSkill = readFileSync(
      path.join(
        repoRoot,
        ".agents",
        "skills",
        "workflow-ui-polish",
        "SKILL.md",
      ),
      "utf8",
    );

    expect(packageJson.devDependencies["@hypothesi/tauri-mcp-cli"]).toBe(
      "0.12.0",
    );
    expect(
      packageJson.devDependencies["@hypothesi/tauri-mcp-server"],
    ).toBeUndefined();
    expect(cargoToml).toContain(
      'tauri-plugin-mcp-bridge = { version = "=0.12.0", optional = true }',
    );
    expect(codexConfig).not.toContain("[mcp_servers.tauri]");
    expect(workflowSkill).toContain("pnpm exec tauri-mcp");
    expect(workflowSkill).toContain(
      "node_modules/@hypothesi/tauri-mcp-cli/skills/tauri-mcp-cli/SKILL.md",
    );
    expect(workflowSkill).toContain("driver-session");
    expect(workflowSkill).toContain("connected: true");
    expect(workflowSkill).toContain(
      "driver-session stop --app-identifier app.fmvaluescout",
    );
    expect(workflowSkill).toContain("Never issue an unscoped");
    expect(workflowSkill).toContain("--json");
  });
});
