#!/usr/bin/env bash
# Contract tests for the scripts/dev dispatcher.
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
dev="$repo_root/scripts/dev"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

failures=0
last_status=0
last_output=""

run() {
  local output_file="$tmp_dir/output-$RANDOM"

  if "$@" >"$output_file" 2>&1; then
    last_status=0
  else
    last_status=$?
  fi
  last_output="$(<"$output_file")"
}

assert_status() {
  local expected="$1"
  local description="$2"

  if [[ "$last_status" -ne "$expected" ]]; then
    printf 'FAIL: %s (expected exit %s, got %s)\n%s\n' \
      "$description" "$expected" "$last_status" "$last_output" >&2
    failures=$((failures + 1))
  fi
}

assert_output_contains() {
  local expected="$1"
  local description="$2"

  if [[ "$last_output" != *"$expected"* ]]; then
    printf 'FAIL: %s (missing %q)\n%s\n' \
      "$description" "$expected" "$last_output" >&2
    failures=$((failures + 1))
  fi
}

assert_file_equals() {
  local file="$1"
  local expected="$2"
  local description="$3"
  local actual

  if [[ ! -f "$file" ]]; then
    printf 'FAIL: %s (expected output file was not created)\n' "$description" >&2
    failures=$((failures + 1))
    return
  fi

  actual="$(<"$file")"
  if [[ "$actual" != "$expected" ]]; then
    printf 'FAIL: %s\nExpected:\n%s\nActual:\n%s\n' \
      "$description" "$expected" "$actual" >&2
    failures=$((failures + 1))
  fi
}

forwarder="$tmp_dir/forwarder"
cat >"$forwarder" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$@" >"$DEV_TEST_LOG"
EOF
chmod +x "$forwarder"

failing_delegate="$tmp_dir/failing-delegate"
cat >"$failing_delegate" <<'EOF'
#!/usr/bin/env bash
printf 'delegated failure\n' >&2
exit 23
EOF
chmod +x "$failing_delegate"

prepare_isolated_secrets_repo() {
  local checkout="$1"

  mkdir -p "$checkout/scripts"
  cp "$repo_root/package.json" \
    "$repo_root/.secretlintrc.json" \
    "$repo_root/.secretlintignore" \
    "$checkout/"
  cp "$repo_root/scripts/dev" "$checkout/scripts/"
  ln -sf "$repo_root/node_modules" "$checkout/node_modules"
  git init -q "$checkout"
  (
    cd "$checkout"
    git add .secretlintrc.json .secretlintignore
    git -c user.name=secretlint-contract \
      -c user.email=contract@localhost \
      commit -q -m "init"
  )
}

run "$dev" test src/app/app-shell-routing.test.tsx
assert_status 0 "default test runs vitest with forwarded file pattern"
assert_output_contains "passed" "vitest reports passing tests"

argument_log="$tmp_dir/arguments"
run env DEV_TEST_COMMAND="$forwarder" DEV_TEST_LOG="$argument_log" \
  "$dev" test "unit/api test" --focused
assert_status 0 "configured test delegate succeeds"
assert_file_equals "$argument_log" $'unit/api test\n--focused' \
  "test target arguments are forwarded unchanged"

run env DEV_TEST_COMMAND="$failing_delegate" "$dev" test failing-target
assert_status 23 "failing delegated test status propagates"
assert_output_contains "delegated failure" "failing delegated test output is preserved"

run "$dev" check-fast
assert_status 0 "check-fast runs Biome, TypeScript, and staged secretlint"

run "$dev" smoke
assert_status 0 "smoke runs Playwright tests"

run "$dev" format
assert_status 0 "format runs biome check --write and cargo fmt"

run "$dev" secrets
assert_status 0 "secrets runs secretlint on the repository"

run "$dev" secrets --staged extra
assert_status 2 "secrets rejects unknown arguments"
assert_output_contains "secrets accepts only optional --staged" \
  "secrets unknown arguments show guidance"

secretlint_probe_value="$(printf '%s' \
  'eG94Yi0xMjM0NTY3ODkwLWFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3eHl6' | base64 -d)"

isolated_secrets_repo="$tmp_dir/isolated-secrets-repo"
prepare_isolated_secrets_repo "$isolated_secrets_repo"
isolated_dev="$isolated_secrets_repo/scripts/dev"

run "$isolated_dev" secrets --staged
assert_status 0 \
  "secrets --staged exits 0 when isolated repo staged index is empty"

printf '%s\n' "$secretlint_probe_value" >"$isolated_secrets_repo/credential-probe.txt"
(
  cd "$isolated_secrets_repo"
  git add credential-probe.txt
)

run "$isolated_dev" secrets --staged
assert_status 1 \
  "secrets --staged rejects staged credential patterns in isolated repo"

tree_probe="$repo_root/src/testing/secretlint-tree-probe.txt"
printf '%s\n' "$secretlint_probe_value" >"$tree_probe"

run "$dev" secrets
assert_status 1 "secrets rejects credential patterns in the tree"

rm -f "$tree_probe"

run "$dev" mutate subject
assert_status 69 "mutate reports unsupported status"
assert_output_contains "not configured" "mutate explains its unsupported status"

run "$dev" mutate
assert_status 2 "mutate requires a target"
assert_output_contains "requires a target" "missing mutation target has precise guidance"

run "$dev" unknown
assert_status 2 "unknown commands fail"
assert_output_contains "Usage:" "unknown commands show usage"

run "$dev"
assert_status 2 "missing command shows usage"
assert_output_contains "bridge-install" "usage documents bridge-install"

# Without a resolvable plugins dir (and no default Steam mount), fail closed with guidance.
# Skip when the developer's WSL Steam path exists — that is a valid default destination.
unset FM_BRIDGE_PLUGINS FM_STEAM_ROOT || true
if [[ ! -d "/mnt/c/Program Files (x86)/Steam/steamapps/common/Football Manager 26/BepInEx/plugins" ]]; then
  run "$dev" bridge-install
  assert_status 66 "bridge-install fails when plugins path is unresolved"
  assert_output_contains "FM_STEAM_ROOT" "bridge-install explains how to set the path"
fi

if [[ "$failures" -ne 0 ]]; then
  printf '%s dispatcher contract test(s) failed.\n' "$failures" >&2
  exit 1
fi

printf 'scripts/dev dispatcher contract tests passed.\n'
