#!/usr/bin/env bash
# Contract tests for repository Codex skills (.agents/skills/).
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
skills_dir="$repo_root/.agents/skills"

if [[ ! -d "$skills_dir" ]]; then
  printf 'Missing Codex skills directory: %s\n' "$skills_dir" >&2
  exit 1
fi

python3 - "$repo_root" "$skills_dir" <<'PY'
import pathlib
import re
import sys

repo_root = pathlib.Path(sys.argv[1])
skills_dir = pathlib.Path(sys.argv[2])

domain_skills = {
    "coding-standards",
    "conventional-commits",
    "debug",
    "minimalism",
    "project-strategy",
    "recallium-usage",
    "security-audit",
    "semantic-versioning",
    "technical-writing",
    "ui-design",
}
workflow_skills = {
    "workflow-build",
    "workflow-build-loop",
    "workflow-checkpoint",
    "workflow-docs-review",
    "workflow-finish-feature",
    "workflow-fix",
    "workflow-plan-feature",
    "workflow-review",
    "workflow-roadmap",
    "workflow-security-audit",
    "workflow-spike",
    "workflow-stack",
}
expected_skills = domain_skills | workflow_skills
found_skills = {path.name for path in skills_dir.iterdir() if path.is_dir()}
errors = []

missing = expected_skills - found_skills
extra = found_skills - expected_skills
if missing:
    errors.append(f"missing skill directories: {sorted(missing)}")
if extra:
    errors.append(f"unexpected skill directories: {sorted(extra)}")

for name in sorted(expected_skills & found_skills):
    path = skills_dir / name / "SKILL.md"
    if not path.is_file():
        errors.append(f"{name}: missing SKILL.md")
        continue

    content = path.read_text(encoding="utf-8")
    frontmatter = re.match(r"^---\s*\n(.*?)\n---", content, re.DOTALL)
    if not frontmatter:
        errors.append(f"{name}: missing or malformed frontmatter")
        continue

    fields = {}
    for line in frontmatter.group(1).splitlines():
        key, separator, value = line.partition(":")
        if separator:
            fields[key.strip()] = value.strip().strip('"\'')

    if fields.get("name") != name:
        errors.append(f"{name}: frontmatter name must match directory")
    if not fields.get("description"):
        errors.append(f"{name}: missing frontmatter description")
    if ".cursor/" in content:
        errors.append(f"{name}: contains a Cursor path")

for name in sorted(workflow_skills & found_skills):
    content = (skills_dir / name / "SKILL.md").read_text(encoding="utf-8")
    for forbidden in ("${ARGUMENTS", "Task", "subagent_type:", "Cursor"):
        if forbidden in content:
            errors.append(f"{name}: contains unsupported migrated workflow syntax {forbidden!r}")

scope_markers = {
    "workflow-build": "developer clearly requests",
    "workflow-build-loop": "developer-supplied commit scope",
    "workflow-checkpoint": "developer-supplied commit scope",
    "workflow-docs-review": "developer-supplied implemented change",
    "workflow-finish-feature": "developer-supplied feature or comparison base",
    "workflow-plan-feature": "feature named by the developer",
    "workflow-roadmap": "developer-supplied constraints",
    "workflow-security-audit": "developer-supplied scope",
    "workflow-spike": "question supplied by the developer",
    "workflow-stack": "developer-supplied constraints",
}
for name, marker in scope_markers.items():
    content = (skills_dir / name / "SKILL.md").read_text(encoding="utf-8")
    if marker not in content:
        errors.append(f"{name}: does not preserve developer-supplied scope")

for name in sorted(domain_skills & found_skills):
    source = repo_root / ".cursor" / "skills" / name
    target = skills_dir / name
    for source_file in source.rglob("*"):
        if not source_file.is_file():
            continue
        target_file = target / source_file.relative_to(source)
        if not target_file.is_file():
            errors.append(f"{name}: missing copied file {target_file.relative_to(target)}")

if errors:
    print("Codex skill contract validation failed:")
    for error in errors:
        print(f"  - {error}")
    raise SystemExit(1)
PY

printf 'Codex skill contract tests passed.\n'
