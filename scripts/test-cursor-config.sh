#!/usr/bin/env bash
# Contract tests for project Cursor configuration (.cursor/mcp.json, commands).
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
mcp_config="$repo_root/.cursor/mcp.json"
commands_dir="$repo_root/.cursor/commands"
failures=0

fail() {
  printf 'FAIL: %s\n' "$1" >&2
  failures=$((failures + 1))
}

echo "=== Test: MCP configuration ==="
if [[ ! -f "$mcp_config" ]]; then
  fail "missing .cursor/mcp.json"
else
  if ! python3 - "$mcp_config" <<'PY'; then
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
data = json.loads(path.read_text(encoding="utf-8"))
servers = data.get("mcpServers", {})
required = {"recallium", "context7"}
missing = required - set(servers.keys())
if missing:
    raise SystemExit(f"missing MCP servers: {sorted(missing)}")
for name in required:
    url = servers[name].get("url", "")
    if not url:
        raise SystemExit(f"{name}: missing url")
PY
    fail "invalid .cursor/mcp.json"
  fi
  printf '  MCP servers verified.\n'
fi

echo "=== Test: Workflow commands ==="
for cmd in stack roadmap plan-feature build fix checkpoint review docs-review finish-feature spike security-audit; do
  if [[ ! -f "$commands_dir/$cmd.md" ]]; then
    fail "missing command: $cmd.md"
  fi
  if ! grep -Fq '## Recallium' "$commands_dir/$cmd.md"; then
    fail "command $cmd.md must include ## Recallium section"
  fi
done
defined_count=$(find "$commands_dir" -maxdepth 1 -name '*.md' | wc -l)
if [[ "$defined_count" -ne 11 ]]; then
  fail "expected exactly 11 workflow commands, found $defined_count"
fi
printf '  Found %d workflow command(s).\n' "$defined_count"

recallium_rule="$repo_root/.cursor/rules/recallium.mdc"
if [[ ! -f "$recallium_rule" ]]; then
  fail "missing .cursor/rules/recallium.mdc"
else
  printf '  Recallium rule present.\n'
fi

for agent in reviewer documentation-steward; do
  agent_file="$repo_root/.cursor/agents/$agent.md"
  if ! grep -Fq '## Recallium' "$agent_file"; then
    fail "agent $agent.md must include ## Recallium section"
  fi
done

if [[ "$failures" -ne 0 ]]; then
  printf '\n%s Cursor configuration test(s) failed.\n' "$failures" >&2
  exit 1
fi

printf '\nAll Cursor configuration contract tests passed.\n'
