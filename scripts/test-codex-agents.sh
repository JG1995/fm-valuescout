#!/usr/bin/env bash
# Contract tests for project-scoped Codex specialist agents (.codex/agents/).
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
agents_dir="$repo_root/.codex/agents"

if [[ ! -d "$agents_dir" ]]; then
  printf 'Missing Codex agents directory: %s\n' "$agents_dir" >&2
  exit 1
fi

python3 - "$repo_root" "$agents_dir" <<'PY'
import pathlib
import sys
import tomllib

repo_root = pathlib.Path(sys.argv[1])
agents_dir = pathlib.Path(sys.argv[2])
expected = {
    "reviewer": {
        "model": "gpt-5.6-terra",
        "model_reasoning_effort": "xhigh",
        "sandbox_mode": "read-only",
        "instruction_markers": (
            "review without changing files",
            "Do not edit, write, stage, unstage, commit, or push",
        ),
    },
    "documentation-steward": {
        "model": "gpt-5.6-terra",
        "model_reasoning_effort": "medium",
        "sandbox_mode": "workspace-write",
        "instruction_markers": (
            ".wiki/**/*.md",
            "README.md",
            "AGENTS.md",
            "Do not modify implementation, tests, schemas, executable scripts, CI workflows, Codex configuration, agent definitions, command templates, or other runtime configuration",
            "Do not stage, unstage, commit, push, or rewrite Git history",
        ),
    },
}
errors = []

for name, requirements in expected.items():
    path = agents_dir / f"{name}.toml"
    if not path.is_file():
        errors.append(f"missing agent definition: {path.relative_to(agents_dir)}")
        continue

    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        errors.append(f"{name}: invalid TOML: {error}")
        continue

    for field in ("name", "description", "developer_instructions"):
        if not isinstance(data.get(field), str) or not data[field].strip():
            errors.append(f"{name}: missing required {field!r}")

    if data.get("name") != name:
        errors.append(f"{name}: name must match the agent identifier")
    if data.get("sandbox_mode") != requirements["sandbox_mode"]:
        errors.append(f"{name}: sandbox_mode must be {requirements['sandbox_mode']!r}")
    for field in ("model", "model_reasoning_effort"):
        if data.get(field) != requirements[field]:
            errors.append(f"{name}: {field} must be {requirements[field]!r}")

    instructions = data.get("developer_instructions", "")
    if isinstance(instructions, str):
        for marker in requirements["instruction_markers"]:
            if marker not in instructions:
                errors.append(f"{name}: missing scope instruction {marker!r}")

workflow_markers = {
    "workflow-checkpoint": ("named `reviewer` Codex agent",),
    "workflow-build-loop": ("named `reviewer` Codex agent",),
    "workflow-review": ("named `reviewer` Codex agent",),
    "workflow-docs-review": ("named `documentation-steward` Codex agent",),
    "workflow-finish-feature": (
        "named `reviewer` Codex agent",
        "named `documentation-steward` Codex agent",
    ),
}
for skill, markers in workflow_markers.items():
    path = repo_root / ".agents" / "skills" / skill / "SKILL.md"
    if not path.is_file():
        errors.append(f"missing workflow skill: {skill}")
        continue
    content = path.read_text(encoding="utf-8")
    for marker in markers:
        if marker not in content:
            errors.append(f"{skill}: does not dispatch {marker}")

if errors:
    print("Codex agent contract validation failed:")
    for error in errors:
        print(f"  - {error}")
    raise SystemExit(1)
PY

printf 'Codex agent contract tests passed.\n'
