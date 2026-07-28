#!/usr/bin/env bash
# Structural contract test for the multi-OS release workflow (not a golden-diff).
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
workflow="$repo_root/.github/workflows/release.yml"

fail() {
  printf 'FAIL: %s\n' "$1" >&2
  exit 1
}

[[ -f "$workflow" ]] || fail "missing GitHub workflow: $workflow"

workflow_text="$(<"$workflow")"

grep -Eq '^[[:space:]]*tags:' <<<"$workflow_text" \
  || fail "release workflow must trigger on version tags"

grep -Eq "windows-latest" <<<"$workflow_text" \
  || fail "release matrix must include windows-latest"

grep -Eq "ubuntu-(latest|[0-9]+\.[0-9]+)" <<<"$workflow_text" \
  || fail "release matrix must include an Ubuntu runner"

grep -Eq "macos-latest" <<<"$workflow_text" \
  || fail "release matrix must include macos-latest"

grep -Fq "aarch64-apple-darwin" <<<"$workflow_text" \
  || fail "release matrix must target aarch64-apple-darwin"

grep -Fq "x86_64-apple-darwin" <<<"$workflow_text" \
  || fail "release matrix must target x86_64-apple-darwin"

grep -Eq 'tauri-apps/tauri-action@v1' <<<"$workflow_text" \
  || fail "release workflow must use tauri-apps/tauri-action@v1"

grep -Eq 'releaseDraft:[[:space:]]*true' <<<"$workflow_text" \
  || fail "release workflow must create a draft GitHub Release"

grep -Eq 'dtolnay/rust-toolchain@stable' <<<"$workflow_text" \
  || fail "release workflow must install a Rust toolchain"

grep -Eq 'swatinem/rust-cache@v2' <<<"$workflow_text" \
  || fail "release workflow must cache Rust artifacts"

grep -Eq 'pnpm/action-setup@' <<<"$workflow_text" \
  || fail "release workflow must set up pnpm"

grep -Eq 'actions/setup-node@' <<<"$workflow_text" \
  || fail "release workflow must set up Node.js"

grep -Eq 'libwebkit2gtk' <<<"$workflow_text" \
  || fail "release workflow must install Tauri Linux WebKit dependencies"

grep -Eiq 'code.?sign|notariz|unsigned' <<<"$workflow_text" \
  || fail "release workflow must document that signing/notarization is not configured"

printf 'Release workflow contract tests passed.\n'
