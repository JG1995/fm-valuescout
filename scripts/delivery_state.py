#!/usr/bin/env python3
"""Validate one schema-4 ledger's native feature-delivery authority."""

from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any

from ledger_state import inspect_ledger, parse_ledger, removed

PLAN_SECTIONS = [
    "## Intent",
    "## User-visible behavior",
    "## Invariants",
    "## Non-goals",
    "## Current-state map",
    "## Feature architecture",
    "## Uncertainty register",
    "## Walking skeleton",
    "## Final validation",
]
MANUAL_PROVIDERS = {
    "GitLab", "Gitea", "Forgejo", "Bitbucket", "Codeberg", "SourceHut",
    "Azure DevOps",
}
MANUAL_HOST = re.compile(r"^[a-z0-9]+(?:[.-][a-z0-9]+)*\.[a-z]{2,}$", re.IGNORECASE)
MANUAL_LABEL = re.compile(r"^[A-Za-z][A-Za-z-]*(?: [A-Za-z][A-Za-z-]*)+$")


def valid_publication_provider(value: str | None) -> bool:
    return bool(
        value in {"GitHub", "Local"}
        or value in MANUAL_PROVIDERS
        or (isinstance(value, str) and (
            MANUAL_HOST.fullmatch(value) or MANUAL_LABEL.fullmatch(value)
        ))
    )


REQUIRED_PR_FIELDS = {
    "Branch": "branch",
    "Base branch": "base_branch",
    "Publication provider": "provider",
    "PR template": "template",
    "Merge method": "merge_method",
    "Required checks": "required_checks",
    "Provisional PR title": "title",
    "Purpose": "purpose",
}


def section_text(lines: list[str], heading: str) -> str:
    try:
        start = lines.index(heading) + 1
    except ValueError:
        return ""
    end = next(
        (index for index in range(start, len(lines)) if lines[index].startswith("## ")),
        len(lines),
    )
    return "\n".join(lines[start:end]).strip()


def native_design(
    ledger: Path, repository_root: Path, lines: list[str], errors: list[str]
) -> str:
    root = repository_root.resolve()
    if ledger.is_symlink() or any(parent.is_symlink() for parent in ledger.parents):
        errors.append("native feature path must not traverse symlinks")
    try:
        relative = ledger.resolve().relative_to(root)
    except ValueError:
        errors.append("feature ledger escapes repository")
        return ""
    parts = relative.parts
    if (
        len(parts) != 4
        or parts[:2] != (".wiki", "features")
        or not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", parts[2])
        or parts[2] in {"template", "templates"}
        or parts[3] != f"{parts[2].upper()}-IMPLEMENTATION.md"
    ):
        errors.append(
            "ledger must be a native non-template .wiki/features/<slug>/<SLUG>-IMPLEMENTATION.md"
        )
        return ""
    design = ledger.resolve().with_name(f"{parts[2].upper()}-DESIGN.md")
    links = [
        line.removeprefix("**Design:** ").strip()
        for line in lines
        if line.startswith("**Design:** ")
    ]
    target = (
        re.fullmatch(r"\[[^\]]+\]\(([^)]+)\)", links[0]) if len(links) == 1 else None
    )
    if target is None or target.group(1) != design.name:
        errors.append("Design link must identify the retained sibling DESIGN file")
    if design.is_symlink() or not design.is_file():
        errors.append("retained DESIGN file is missing or a symlink")
        return ""
    return design.read_text(encoding="utf-8")


def pull_request_authority(
    ledger_path: Path, repository_root: Path, errors: list[str]
) -> list[dict[str, Any]]:
    authority: list[dict[str, Any]] = []
    root = repository_root.resolve()
    for pull_request in parse_ledger(ledger_path).pull_requests:
        if removed(pull_request.status):
            continue
        record: dict[str, Any] = {
            "id": pull_request.pr_id,
            "depends_on": pull_request.depends_on,
        }
        for label, attribute in REQUIRED_PR_FIELDS.items():
            value = getattr(pull_request, attribute)
            record[attribute] = value
            if not value:
                errors.append(f"{pull_request.name} is missing {label}")
        if not valid_publication_provider(pull_request.provider):
            errors.append(
                f"{pull_request.name} has unsupported publication provider: "
                f"{pull_request.provider}"
            )
        if pull_request.provider == "Local":
            if pull_request.merge_method != "ff-only":
                errors.append(f"{pull_request.name} local integration requires ff-only")
            if pull_request.template != "Not applicable":
                errors.append(f"{pull_request.name} local integration has no PR template")
            if pull_request.pr_ref != "Not applicable":
                errors.append(f"{pull_request.name} local integration has no PR ref")
        elif pull_request.merge_method not in {"merge", "squash", "rebase"}:
            errors.append(
                f"{pull_request.name} has unsupported merge method: {pull_request.merge_method}"
            )
        if pull_request.provider == "GitHub" and pull_request.template == "Not applicable":
            errors.append(f"{pull_request.name} GitHub publication requires a PR template")
        if pull_request.template and pull_request.template != "Not applicable":
            template = (root / pull_request.template).resolve()
            try:
                template.relative_to(root)
            except ValueError:
                errors.append(f"{pull_request.name} PR template escapes the repository")
            else:
                if not template.is_file():
                    errors.append(
                        f"{pull_request.name} PR template does not exist: {pull_request.template}"
                    )
        authority.append(record)
    return authority


def next_action(classifier_state: str, provider: str | None) -> str:
    if classifier_state == "pr_boundary":
        if provider == "GitHub":
            return "publish"
        return "integrate_local" if provider == "Local" else "manual_publish"
    return {
        "runnable": "build",
        "implementation_complete": "finish",
        "abandoned": "complete",
        "completed": "complete",
    }.get(classifier_state, "stop")


def inspect_delivery(ledger_path: Path, repository_root: Path) -> dict[str, Any]:
    errors: list[str] = []
    lines = ledger_path.read_text(encoding="utf-8").splitlines()
    classified = inspect_ledger(ledger_path)
    classified_errors = classified.get("errors")
    if isinstance(classified_errors, list):
        errors.extend(str(error) for error in classified_errors)
    if classified.get("ledger_schema") != 4:
        errors.append("End-to-end delivery requires ledger schema 4")
    design_content = native_design(ledger_path, repository_root, lines, errors)
    pull_requests = pull_request_authority(ledger_path, repository_root, errors)
    feature_plan = {heading: section_text(lines, heading) for heading in PLAN_SECTIONS}
    for heading, content in feature_plan.items():
        if not content:
            errors.append(f"ledger is missing required plan section content: {heading}")
    authority = {
        "design_content": design_content,
        "feature_plan": feature_plan,
        "pull_requests": pull_requests,
        "packages": classified.get("package_digests", []),
    }
    plan_digest = hashlib.sha256(
        json.dumps(authority, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()
    classifier_state = str(classified.get("state"))
    boundary = next(
        (pr for pr in pull_requests if pr["id"] == classified.get("boundary_pr_id")),
        None,
    )
    return {
        "ledger": str(ledger_path),
        "state": "invalid" if errors else classifier_state,
        "classifier_state": classifier_state,
        "next_action": (
            "stop" if errors else next_action(classifier_state, boundary["provider"] if boundary else None)
        ),
        "plan_digest": plan_digest,
        "package_digests": classified.get("package_digests", []),
        "active_pr": classified.get("active_pr"),
        "active_pr_id": classified.get("active_pr_id"),
        "active_branch": classified.get("active_branch"),
        "active_base_branch": classified.get("active_base_branch"),
        "active_pr_dependency_merge_refs": classified.get(
            "active_pr_dependency_merge_refs", []
        ),
        "active_wave": classified.get("active_wave"),
        "active_package": classified.get("active_package"),
        "active_package_id": classified.get("active_package_id"),
        "wave_packages": classified.get("wave_packages", []),
        "boundary_pr": classified.get("boundary_pr"),
        "boundary_pr_id": classified.get("boundary_pr_id"),
        "remaining_packages": classified.get("remaining_packages"),
        "pull_requests": pull_requests,
        "errors": errors,
    }


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print(
            "usage: delivery_state.py <feature-ledger.md> <repository-root>",
            file=sys.stderr,
        )
        return 2
    try:
        result = inspect_delivery(Path(argv[1]), Path(argv[2]))
    except (OSError, UnicodeError) as error:
        result = {"ledger": argv[1], "state": "invalid", "errors": [str(error)]}
    print(json.dumps(result, indent=2, sort_keys=True))
    return 2 if result["state"] == "invalid" else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
