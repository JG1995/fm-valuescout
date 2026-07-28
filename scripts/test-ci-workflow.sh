#!/usr/bin/env bash
# Contract test for the GitHub workflow that mirrors the local full gate (CI).
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
workflow="$repo_root/.github/workflows/check.yml"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

fail() {
  printf 'FAIL: %s\n' "$1" >&2
  exit 1
}

[[ -f "$workflow" ]] || fail "missing GitHub workflow: $workflow"

# Nested controlled-checkout checks set CI_WORKFLOW_GATE_PROBE_DEPTH; unset in normal shells.
if [[ -n "${CI_WORKFLOW_GATE_PROBE_DEPTH:-}" ]]; then
  printf 'GitHub Actions workflow contract tests passed.\n'
  exit 0
fi

expected_workflow="$tmp_dir/expected-check.yml"
cat >"$expected_workflow" <<'EOF'
name: Check

on:
  pull_request:
  push:
    branches:
      - main

permissions:
  contents: read

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: swatinem/rust-cache@v2
        with:
          workspaces: |
            src-tauri -> target
      - name: Install Tauri Linux dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev \
            patchelf
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with:
          node-version: "24"
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - name: Install Playwright Chromium
        run: pnpm exec playwright install --with-deps chromium
      - name: Run project gate
        run: ./scripts/dev check
      - name: Run tests
        run: ./scripts/dev test
      - name: Production build
        run: pnpm build
EOF

if ! diff -u "$expected_workflow" "$workflow"; then
  fail "workflow must retain the validated GitHub Actions structure"
fi

ensure_contract_test_deps() {
  if [[ ! -d "$repo_root/node_modules" ]]; then
    fail "node_modules missing — run pnpm install before CI workflow contract tests"
  fi

  export PLAYWRIGHT_BROWSERS_PATH="${PLAYWRIGHT_BROWSERS_PATH:-${HOME}/.cache/ms-playwright}"
  mkdir -p "$PLAYWRIGHT_BROWSERS_PATH"

  (
    cd "$repo_root"
    pnpm exec playwright install chromium
  )
}

prepare_controlled_checkout() {
  local checkout="$1"

  mkdir "$checkout"
  cp -R "$repo_root/scripts" "$repo_root/.cursor" "$repo_root/.github" \
    "$checkout"
  cp "$repo_root/package.json" "$repo_root/pnpm-lock.yaml" \
    "$repo_root/pnpm-workspace.yaml" "$repo_root/biome.json" \
    "$repo_root/tsconfig.json" "$repo_root/tsconfig.app.json" \
    "$repo_root/tsconfig.node.json" "$repo_root/vite.config.ts" \
    "$repo_root/playwright.config.ts" "$repo_root/index.html" \
    "$repo_root/.gitignore" "$repo_root/.secretlintrc.json" \
    "$repo_root/.secretlintignore" "$checkout/"
  cp -R "$repo_root/src" "$repo_root/public" "$repo_root/e2e" "$checkout/"
  rsync -a --exclude target "$repo_root/src-tauri/" "$checkout/src-tauri/"

  (
    cd "$checkout"
    CI=true pnpm install --frozen-lockfile
  )
}

ensure_contract_test_deps

if [[ "${CI_WORKFLOW_CONTROLLED_FAILURE:-}" != "1" ]]; then
  tsc_checkout="$tmp_dir/tsc-checkout"
  prepare_controlled_checkout "$tsc_checkout"
  printf 'export const tscGateProbe: string = 1;\n' \
    >"$tsc_checkout/src/testing/tsc-gate-probe.ts"

  if CI_WORKFLOW_GATE_PROBE_DEPTH=1 "$tsc_checkout/scripts/dev" check \
    >"$tmp_dir/tsc-output" 2>&1; then
    fail "check must fail when TypeScript errors exist"
  fi

  if ! grep -Eiq 'tsc|TypeScript|TS[0-9]+' "$tmp_dir/tsc-output"; then
    fail "TypeScript gate failure must preserve tsc cause"
  fi

  biome_checkout="$tmp_dir/biome-checkout"
  prepare_controlled_checkout "$biome_checkout"
  printf 'export const biomeGateProbe = ;\n' \
    >"$biome_checkout/src/testing/biome-gate-probe.ts"

  if CI_WORKFLOW_GATE_PROBE_DEPTH=1 "$biome_checkout/scripts/dev" check \
    >"$tmp_dir/biome-output" 2>&1; then
    fail "check must fail when Biome errors exist"
  fi

  if ! grep -Eiq 'biome|parse' "$tmp_dir/biome-output"; then
    fail "Biome gate failure must preserve biome cause"
  fi

  secretlint_checkout="$tmp_dir/secretlint-checkout"
  prepare_controlled_checkout "$secretlint_checkout"
  secretlint_probe="$(printf '%s' \
    'eG94Yi0xMjM0NTY3ODkwLWFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3eHl6' | base64 -d)"
  printf '%s\n' "$secretlint_probe" \
    >"$secretlint_checkout/src/testing/secretlint-gate-probe.txt"

  if CI_WORKFLOW_GATE_PROBE_DEPTH=1 "$secretlint_checkout/scripts/dev" check \
    >"$tmp_dir/secretlint-output" 2>&1; then
    fail "check must fail when secretlint finds credentials"
  fi

  if ! grep -Eiq 'secretlint|secret' "$tmp_dir/secretlint-output"; then
    fail "secretlint gate failure must preserve secretlint cause"
  fi

  rust_checkout="$tmp_dir/rust-checkout"
  prepare_controlled_checkout "$rust_checkout"
  printf 'pub fn rust_gate_probe() { let x = ; }\n' \
    >>"$rust_checkout/src-tauri/src/features/health/commands.rs"

  if CI_WORKFLOW_GATE_PROBE_DEPTH=1 "$rust_checkout/scripts/dev" check \
    >"$tmp_dir/rust-output" 2>&1; then
    fail "check must fail when Rust fmt/clippy errors exist"
  fi

  if ! grep -Eiq 'expected expression|cargo fmt|rustfmt|clippy|error:' \
    "$tmp_dir/rust-output"; then
    fail "Rust gate failure must preserve cargo cause"
  fi

  controlled_checkout="$tmp_dir/controlled-checkout"
  prepare_controlled_checkout "$controlled_checkout"

  python3 - "$controlled_checkout/.cursor/agents/reviewer.md" <<'PY'
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
content = path.read_text(encoding="utf-8")
content = re.sub(
    r'^description:.*\n',
    '',
    content,
    count=1,
    flags=re.MULTILINE,
)
path.write_text(content, encoding="utf-8")
PY

  if CI_WORKFLOW_CONTROLLED_FAILURE=1 CI_WORKFLOW_GATE_PROBE_DEPTH=1 \
    "$controlled_checkout/scripts/dev" check \
      >"$tmp_dir/controlled-output" 2>&1; then
    fail "a controlled local fast-gate failure must return non-zero"
  fi

  if ! grep -Fq 'missing required frontmatter field' "$tmp_dir/controlled-output"; then
    fail "controlled fast-gate failure must preserve its cause"
  fi
fi

printf 'GitHub Actions workflow contract tests passed.\n'
