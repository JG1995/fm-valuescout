#!/usr/bin/env bash
# Contract checks for repository-defined Cursor agents (.cursor/agents/*.md).
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
agents_dir="$repo_root/.cursor/agents"
cursor_readme="$repo_root/.cursor/README.md"
docs_review="$repo_root/.cursor/commands/docs-review.md"

if [[ ! -d "$agents_dir" ]]; then
  printf 'Missing agents directory: %s\n' "$agents_dir" >&2
  exit 1
fi

if [[ ! -f "$cursor_readme" ]]; then
  printf 'Missing Cursor workflow readme: %s\n' "$cursor_readme" >&2
  exit 1
fi

if [[ ! -f "$docs_review" ]]; then
  printf 'Missing documentation-review command: %s\n' "$docs_review" >&2
  exit 1
fi

validation_output="$(
  python3 - "$agents_dir" <<'PY'
import pathlib
import re
import sys

agents_dir = pathlib.Path(sys.argv[1])

EXPECTED_AGENTS = {"reviewer", "documentation-steward"}
EXPECTED_MODELS = {
    "reviewer": "grok-4.5[effort=high,fast=false]",
    "documentation-steward": "composer-2.5[fast=false]",
}
READ_ONLY_AGENTS = {"reviewer"}
DOCUMENTATION_WRITER = "documentation-steward"
WRITE_TOOLS = {"edit", "write"}

errors = []

agent_files = sorted(agents_dir.glob("*.md"))
found_names = set()

for f in agent_files:
    name = f.stem
    found_names.add(name)
    content = f.read_text(encoding="utf-8")

    fm_match = re.match(r'^---\s*\n(.*?)\n---', content, re.DOTALL)
    if not fm_match:
        errors.append(f"{name}: missing or malformed frontmatter (no --- delimiters)")
        continue

    fm = {}
    for line in fm_match.group(1).splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        colon = line.find(":")
        if colon == -1:
            continue
        key = line[:colon].strip()
        value = line[colon + 1:].strip()
        if (value.startswith('"') and value.endswith('"')) or \
           (value.startswith("'") and value.endswith("'")):
            value = value[1:-1]
        fm[key] = value

    for field in ("name", "description", "model"):
        if field not in fm:
            errors.append(f"{name}: missing required frontmatter field '{field}'")

    expected_model = EXPECTED_MODELS.get(name)
    if expected_model and fm.get("model") != expected_model:
        errors.append(
            f"{name}: model must be {expected_model!r}; got {fm.get('model')!r}"
        )

    agent_name = fm.get("name", "")
    if agent_name and agent_name != name:
        errors.append(f"{name}: frontmatter name must match file stem; got {agent_name!r}")

    if name in READ_ONLY_AGENTS:
        if "You do NOT edit files" in content or "PROHIBITED from" in content:
            pass
        elif "without changing files" in content or "read-only" in content.lower():
            pass
        else:
            errors.append(f"{name}: read-only agent must state edit prohibition in body")

    if name == DOCUMENTATION_WRITER:
        required_contract = {
            "documentation-only scope": "You may create, edit, move, or remove documentation",
            "implementation prohibition": "Do not modify implementation",
            "Cursor configuration prohibition": "Cursor configuration",
            "agent-definition prohibition": "agent definitions",
            "command-template prohibition": "command templates",
            "Git prohibition": "Do not stage, unstage, commit",
        }
        for requirement, text in required_contract.items():
            if text not in content:
                errors.append(f"{name}: missing {requirement} contract")

missing = EXPECTED_AGENTS - found_names
extra = found_names - EXPECTED_AGENTS
if missing:
    errors.append(f"missing required agent definition(s): {sorted(missing)}")
if extra:
    errors.append(f"unexpected project agent definition(s): {sorted(extra)}")

if errors:
    print("ERRORS:")
    for e in errors:
        print(f"  - {e}")
else:
    print("OK")
PY
)"

if echo "$validation_output" | grep -q "^ERRORS:"; then
  printf 'Agent definition validation failed:\n%s\n' "$validation_output" >&2
  exit 1
fi

for required_text in \
  'reviewer' \
  'documentation-steward' \
  '/plan-feature' \
  'must not contain credentials'; do
  if ! grep -Fq -- "$required_text" "$cursor_readme"; then
    printf 'Cursor workflow readme must mention %q.\n' "$required_text" >&2
    exit 1
  fi
done

for required_text in \
  'documentation-steward' \
  'Do not run it in the background' \
  'implementation, tests, scripts, CI'; do
  if ! grep -Fq -- "$required_text" "$docs_review"; then
    printf 'Documentation-review command must include %q.\n' "$required_text" >&2
    exit 1
  fi
done

printf 'Cursor agent definition contract tests passed.\n'
