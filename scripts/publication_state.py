#!/usr/bin/env python3
"""Classify a schema-4 boundary for GitHub, manual handoff, or local Git."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from urllib.parse import urlsplit

from delivery_state import valid_publication_provider
from ledger_state import inspect_ledger, parse_ledger, removed

GITHUB_PR_URL = re.compile(r"^https://github\.com/[^/]+/[^/]+/pull/\d+$")
MERGE_METHODS = {"merge", "squash", "rebase"}


def manual_ref(value: str) -> bool:
    try:
        parsed = urlsplit(value)
        return (
            parsed.scheme == "https"
            and bool(parsed.hostname)
            and bool(parsed.path.strip("/"))
            and not parsed.username
            and not parsed.password
            and not parsed.fragment
        )
    except ValueError:
        return False


def unwrap_markdown_autolink(value: str | None) -> str | None:
    if value and value.startswith("<") and value.endswith(">"):
        return value[1:-1]
    return value


def inspect_publication(ledger_path: Path, repository_root: Path) -> dict[str, object]:
    classified = inspect_ledger(ledger_path)
    classified_errors = classified.get("errors")
    errors = (
        [str(error) for error in classified_errors]
        if isinstance(classified_errors, list)
        else []
    )
    ledger = parse_ledger(ledger_path)
    boundary_id = classified.get("boundary_pr_id")
    pull_request = next(
        (pr for pr in ledger.pull_requests if pr.pr_id == boundary_id),
        None,
    )
    result: dict[str, object] = {
        "ledger": str(ledger_path),
        "state": "invalid",
        "pr": None,
        "pr_id": None,
        "branch": None,
        "base_branch": None,
        "provider": None,
        "template": None,
        "merge_method": None,
        "required_checks": None,
        "title": None,
        "pr_url": None,
        "is_final_pr": None,
        "feature_closeout": None,
        "errors": errors,
    }
    if classified.get("state") not in {"pr_boundary", "implementation_complete"}:
        errors.append("ledger must be at a PR publication boundary")
        return result
    if pull_request is None:
        errors.append("publication boundary does not identify a PR")
        return result

    index = ledger.pull_requests.index(pull_request)
    is_final = not any(
        not removed(successor.status) for successor in ledger.pull_requests[index + 1 :]
    )
    result.update(
        pr=pull_request.name,
        pr_id=pull_request.pr_id,
        branch=pull_request.branch,
        base_branch=pull_request.base_branch,
        provider=pull_request.provider,
        template=pull_request.template,
        merge_method=pull_request.merge_method,
        required_checks=pull_request.required_checks,
        title=pull_request.title,
        is_final_pr=is_final,
        feature_closeout=pull_request.feature_closeout,
    )

    required = {
        "Branch": pull_request.branch,
        "Base branch": pull_request.base_branch,
        "Publication provider": pull_request.provider,
        "PR template": pull_request.template,
        "Merge method": pull_request.merge_method,
        "Required checks": pull_request.required_checks,
        "Feature close-out": pull_request.feature_closeout,
        "Provisional PR title": pull_request.title,
        "PR ref": pull_request.pr_ref,
        "Merge ref": pull_request.merge_ref,
    }
    for field_name, value in required.items():
        if not value:
            errors.append(f"{pull_request.name} is missing {field_name}")
    if not valid_publication_provider(pull_request.provider):
        errors.append(f"unsupported publication provider: {pull_request.provider}")
    if pull_request.provider == "Local":
        if pull_request.merge_method != "ff-only":
            errors.append("local integration requires ff-only")
        if pull_request.template != "Not applicable":
            errors.append("local integration has no PR template")
    elif pull_request.merge_method and pull_request.merge_method not in MERGE_METHODS:
        errors.append(f"unsupported merge method: {pull_request.merge_method}")
    if pull_request.branch and pull_request.branch == pull_request.base_branch:
        errors.append("publication branch must differ from the base branch")
    if pull_request.provider == "GitHub" and pull_request.template == "Not applicable":
        errors.append("GitHub publication requires a PR template")
    if pull_request.template and pull_request.template != "Not applicable":
        root = repository_root.resolve()
        template_path = (root / pull_request.template).resolve()
        try:
            template_path.relative_to(root)
        except ValueError:
            errors.append(f"PR template escapes repository: {pull_request.template}")
        else:
            if not template_path.is_file():
                errors.append(f"PR template does not exist: {pull_request.template}")
    if pull_request.merge_ref and pull_request.merge_ref != "Not merged":
        errors.append(f"{pull_request.name} is already merged")

    pr_ref = unwrap_markdown_autolink(pull_request.pr_ref)
    if pull_request.provider == "Local":
        state = "ready_to_integrate" if pr_ref == "Not applicable" else "invalid"
    elif pr_ref == "Not published":
        state = "ready_to_publish" if pull_request.provider == "GitHub" else "manual_publish"
    elif pr_ref and pull_request.provider == "GitHub" and GITHUB_PR_URL.fullmatch(pr_ref):
        state = "published"
        result["pr_url"] = pr_ref
    elif pr_ref and pull_request.provider != "GitHub" and manual_ref(pr_ref):
        state = "manual_publish"
        result["pr_url"] = pr_ref
    else:
        state = "invalid"
    if state == "invalid":
        errors.append(f"invalid {pull_request.provider} PR ref: {pr_ref}")

    if not errors:
        if is_final and pull_request.feature_closeout != "Current":
            result["state"] = "closeout_required"
        else:
            result["state"] = state
    return result


def main(argv: list[str]) -> int:
    if len(argv) != 3:
        print(
            "usage: publication_state.py <feature-ledger.md> <repository-root>",
            file=sys.stderr,
        )
        return 2
    try:
        result = inspect_publication(Path(argv[1]), Path(argv[2]))
    except (OSError, UnicodeError) as error:
        result = {"ledger": argv[1], "state": "invalid", "errors": [str(error)]}
    print(json.dumps(result, indent=2, sort_keys=True))
    return 2 if result["state"] == "invalid" else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
