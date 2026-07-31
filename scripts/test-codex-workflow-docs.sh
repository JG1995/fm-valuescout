#!/usr/bin/env bash
# Contract tests for the documented Codex workflow.
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"

python3 - "$repo_root" <<'PY'
import pathlib
import sys

repo_root = pathlib.Path(sys.argv[1])
required = {
    ".codex/README.md": (
        ".agents/skills/",
        ".codex/agents/",
        ".codex/config.toml",
        "workflow-build",
        "workflow-checkpoint",
        "workflow-finish-feature",
    ),
    "AGENTS.md": (
        ".agents/skills/",
        ".codex/agents/",
        ".codex/config.toml",
        "./scripts/dev check",
        "read-only reviewer",
    ),
    "README.md": (
        ".codex/README.md",
        "workflow-plan-feature",
        "workflow-build",
    ),
    "CONTRIBUTING.md": (
        ".codex/README.md",
        "workflow-checkpoint",
        ".agents/skills/conventional-commits/SKILL.md",
    ),
    ".wiki/INDEX.md": (
        ".work/",
        "workflow-plan-feature",
        "workflow-finish-feature",
    ),
    ".wiki/ARCHITECTURE.md": (
        ".agents/skills/",
        ".codex/",
        "workflow-stack",
    ),
    ".wiki/features/active/README.md": ("workflow-plan-feature",),
}
forbidden = {
    "AGENTS.md": (".cursor/", "Cursor commands", "via Task"),
    "README.md": ("Cursor workflow", ".cursor/README.md", "Cursor MCP"),
    "CONTRIBUTING.md": (".cursor/", "Cursor workflow", "Cursor commands"),
    ".wiki/INDEX.md": (".cursor/work/", "`/roadmap`", "`/plan-feature`", "`/finish-feature`"),
    ".wiki/features/planned/README.md": ("`/roadmap`", "`/plan-feature`"),
    ".wiki/features/completed/README.md": ("`/finish-feature`", "`/docs-review`"),
    ".wiki/ARCHITECTURE.md": (".cursor/", "with a Cursor workflow", "via `/stack`"),
    ".wiki/features/active/README.md": ("`/plan-feature`",),
}
errors = []
for relative_path, markers in required.items():
    path = repo_root / relative_path
    if not path.is_file():
        errors.append(f"missing required workflow document: {relative_path}")
        continue
    content = path.read_text(encoding="utf-8")
    for marker in markers:
        if marker not in content:
            errors.append(f"{relative_path}: missing Codex workflow marker {marker!r}")
for relative_path, markers in forbidden.items():
    path = repo_root / relative_path
    if not path.is_file():
        errors.append(f"missing checked workflow document: {relative_path}")
        continue
    content = path.read_text(encoding="utf-8")
    for marker in markers:
        if marker in content:
            errors.append(f"{relative_path}: contains stale Cursor workflow marker {marker!r}")
if errors:
    print("Codex workflow documentation contract validation failed:")
    for error in errors:
        print(f"  - {error}")
    raise SystemExit(1)
PY

printf 'Codex workflow documentation contract tests passed.\n'
