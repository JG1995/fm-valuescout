#!/usr/bin/env bash
# Contract tests for Codex project configuration (.codex/config.toml).
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
config="$repo_root/.codex/config.toml"

if [[ ! -f "$config" ]]; then
  printf 'Missing Codex project configuration: %s\n' "$config" >&2
  exit 1
fi

python3 - "$config" <<'PY'
import pathlib
import sys
import tomllib

path = pathlib.Path(sys.argv[1])
try:
    data = tomllib.loads(path.read_text(encoding="utf-8"))
except (OSError, tomllib.TOMLDecodeError) as error:
    raise SystemExit(f"invalid .codex/config.toml: {error}")

servers = data.get("mcp_servers")
if not isinstance(servers, dict):
    raise SystemExit("missing [mcp_servers] configuration")

required_urls = {
    "recallium": "http://10.189.1.195:8001/mcp",
    "context7": "https://mcp.context7.com/mcp",
}
for name, expected_url in required_urls.items():
    server = servers.get(name)
    if not isinstance(server, dict):
        raise SystemExit(f"missing MCP server: {name}")
    if server.get("url") != expected_url:
        raise SystemExit(f"{name}: expected URL {expected_url!r}")
PY

printf 'Codex project configuration contract tests passed.\n'
