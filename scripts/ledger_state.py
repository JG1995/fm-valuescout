#!/usr/bin/env python3
"""Validate and classify a schema-4 feature implementation ledger."""

from __future__ import annotations

import hashlib
import json
import re
import sys
import unicodedata
from dataclasses import dataclass, field
from pathlib import Path

ID = r"[a-z0-9]+(?:-[a-z0-9]+)*"
PR_HEADING = re.compile(rf"^### (PR `({ID})`\s+[—-]\s+.+)$")
PACKAGE_HEADING = re.compile(rf"^#### (Package `({ID})`\s+[—-]\s+.+)$")
NUMBERED_PACKAGE_HEADING = re.compile(
    rf"^Package `({ID})`\s+[—-]\s+Commit ([1-9][0-9]*):\s+\S.*$"
)
IMMUTABLE_GIT_OID = re.compile(r"^(?:[0-9a-f]{40}|[0-9a-f]{64})$")
FEATURE_STATUSES = {"Accepted", "Blocked", "Completed", "Abandoned"}
PACKAGE_STATUSES = {"Planned", "Integrated"}
REQUIRED_PR_FIELDS = (
    "Depends on",
    "PR ref",
    "Merge ref",
    "Branch",
    "Base branch",
    "Publication provider",
    "PR template",
    "Merge method",
    "Required checks",
    "Feature close-out",
    "Provisional PR title",
    "Purpose",
)
REQUIRED_PACKAGE_FIELDS = (
    "Provisional commit",
    "Work",
    "Implementation packet",
    "Files and responsibilities",
    "Tests and proof",
    "Validation",
    "Stop conditions",
    "Review mandate",
)
PROHIBITED_MODEL_FIELDS = {
    "Build-feature-loop profile",
    "Implementation profile",
    "Review profile",
    "Feature review profile",
}


@dataclass
class Package:
    name: str
    package_id: str
    status: str | None = None
    fields: set[str] = field(default_factory=set)
    authority_lines: list[str] = field(default_factory=list)
    wave: int | None = None
    depends_on: list[str] = field(default_factory=list)
    write_scope: list[str] = field(default_factory=list)


@dataclass
class PullRequest:
    name: str
    pr_id: str
    status: str | None = None
    fields: set[str] = field(default_factory=set)
    pr_ref: str | None = None
    merge_ref: str | None = None
    branch: str | None = None
    base_branch: str | None = None
    provider: str | None = None
    template: str | None = None
    merge_method: str | None = None
    required_checks: str | None = None
    feature_closeout: str | None = None
    title: str | None = None
    purpose: str | None = None
    depends_on: list[str] = field(default_factory=list)
    packages: list[Package] = field(default_factory=list)


@dataclass
class Ledger:
    schema: int | None
    feature_status: str | None
    pull_requests: list[PullRequest]
    abandonment: dict[str, str]
    errors: list[str]


def canonical_file_path(value: object) -> bool:
    if not isinstance(value, str) or not value or value.startswith("/"):
        return False
    if any(unicodedata.category(char).startswith("C") for char in value):
        return False
    if any(char in value for char in "\\*?[]:"):
        return False
    return all(part not in {"", ".", "..", ".git"} for part in value.split("/"))


def coordinator_owned_path(value: str) -> bool:
    return (
        value == ".work"
        or value.startswith(".work/")
        or value.endswith(("-DESIGN.md", "-IMPLEMENTATION.md"))
        or Path(value).name in {"TODO.md", "BACKLOG.md"}
    )


def removed(status: str | None) -> bool:
    return bool(status and status.startswith("Removed — "))


def immutable_git_oid(value: str | None) -> bool:
    return bool(value and IMMUTABLE_GIT_OID.fullmatch(value))


def _json_field(
    line: str,
    label: str,
    owner: str,
    errors: list[str],
) -> object | None:
    if not line.startswith(f"**{label}:** "):
        return None
    try:
        return json.loads(line.split(":** ", 1)[1])
    except (ValueError, TypeError):
        errors.append(f"{owner} has malformed {label}")
        return None


def parse_ledger(path: Path) -> Ledger:
    lines = path.read_text(encoding="utf-8").splitlines()
    errors: list[str] = []
    schema: int | None = None
    feature_status: str | None = None
    pull_requests: list[PullRequest] = []
    abandonment: dict[str, str] = {}
    current_pr: PullRequest | None = None
    current_package: Package | None = None
    seen_sections: set[str] = set()
    seen_fields: set[str] = set()
    seen_globals: set[str] = set()
    current_section: str | None = None
    commit_count: str | None = None
    commit_count_seen = False
    in_abandonment = False

    for index, line in enumerate(lines):
        if (
            line.startswith("## ")
            or PR_HEADING.match(line)
            or PACKAGE_HEADING.match(line)
        ):
            seen_fields = set()
        if line.startswith("### PR ") and not PR_HEADING.match(line):
            errors.append(f"malformed PR heading: {line}")
        if line.startswith("#### Package ") and not PACKAGE_HEADING.match(line):
            errors.append(f"malformed package heading: {line}")

        field_match = re.match(r"^\*\*([^*]+):\*\*(?:\s|$)", line)
        if field_match:
            label = field_match.group(1)
            if label in seen_fields:
                errors.append(f"duplicate field: {label}")
            seen_fields.add(label)
            if label in {"Ledger schema", "Design"}:
                if label in seen_globals:
                    errors.append(f"duplicate global field: {label}")
                seen_globals.add(label)
            if label in PROHIBITED_MODEL_FIELDS:
                errors.append(f"ledger contains prohibited {label}")

        if line.startswith("## "):
            current_section = line
            if line in seen_sections:
                errors.append(f"duplicate ledger section: {line}")
            seen_sections.add(line)
            in_abandonment = line == "## Abandonment record"
            current_pr = None
            current_package = None

        if line.startswith("**Ledger schema:** "):
            raw = line.removeprefix("**Ledger schema:** ").strip()
            try:
                schema = int(raw)
            except ValueError:
                errors.append(f"invalid ledger schema: {raw!r}")
            continue

        if line.startswith("**Commit packages:**"):
            if commit_count_seen:
                errors.append("duplicate Commit packages")
            commit_count_seen = True
            if current_section != "## Delivery plan" or current_pr is not None:
                errors.append("Commit packages must precede the first PR in Delivery plan")
            match = re.fullmatch(r"\*\*Commit packages:\*\* ([1-9][0-9]*)", line)
            if match is None:
                errors.append("invalid Commit packages total")
            else:
                commit_count = match.group(1)
            continue

        if line == "## Status":
            feature_status = next(
                (
                    candidate.strip()
                    for candidate in lines[index + 1 :]
                    if candidate.strip()
                ),
                None,
            )
            continue

        if in_abandonment and line.startswith("**") and ":** " in line:
            name, value = line[2:].split(":** ", maxsplit=1)
            abandonment[name] = value.strip()
            continue

        pr_match = PR_HEADING.match(line)
        if pr_match:
            current_pr = PullRequest(pr_match.group(1), pr_match.group(2))
            pull_requests.append(current_pr)
            current_package = None
            continue

        package_match = PACKAGE_HEADING.match(line)
        if package_match and current_pr:
            current_package = Package(
                package_match.group(1),
                package_match.group(2),
                authority_lines=[line],
            )
            current_pr.packages.append(current_package)
            continue

        if not current_pr:
            continue

        if line.startswith("**Status:** "):
            value = line.removeprefix("**Status:** ").strip()
            if current_package:
                current_package.status = value
            else:
                current_pr.status = value
            continue

        if current_package:
            current_package.authority_lines.append(line)
            for structural_field in ("Wave", "Depends on", "Write scope"):
                if line.startswith(f"**{structural_field}:** "):
                    current_package.fields.add(structural_field)
            value = _json_field(line, "Wave", current_package.name, errors)
            if value is not None:
                if type(value) is int and value > 0:
                    current_package.wave = value
                else:
                    errors.append(f"{current_package.name} has invalid Wave")
            value = _json_field(line, "Depends on", current_package.name, errors)
            if value is not None:
                if (
                    isinstance(value, list)
                    and len(value) == len(set(value))
                    and all(
                        isinstance(item, str) and re.fullmatch(ID, item)
                        for item in value
                    )
                ):
                    current_package.depends_on = value
                else:
                    errors.append(f"{current_package.name} has invalid Depends on")
            value = _json_field(line, "Write scope", current_package.name, errors)
            if value is not None:
                if (
                    isinstance(value, list)
                    and bool(value)
                    and len(value) == len(set(value))
                    and all(canonical_file_path(item) for item in value)
                ):
                    current_package.write_scope = value
                else:
                    errors.append(f"{current_package.name} has invalid Write scope")
            for required in REQUIRED_PACKAGE_FIELDS:
                if line.startswith(f"**{required}:**"):
                    current_package.fields.add(required)
                    break
            continue

        fields = {
            "PR ref": "pr_ref",
            "Merge ref": "merge_ref",
            "Branch": "branch",
            "Base branch": "base_branch",
            "Publication provider": "provider",
            "PR template": "template",
            "Merge method": "merge_method",
            "Required checks": "required_checks",
            "Feature close-out": "feature_closeout",
            "Provisional PR title": "title",
            "Purpose": "purpose",
        }
        for label, attribute in fields.items():
            if line.startswith(f"**{label}:** "):
                current_pr.fields.add(label)
                setattr(
                    current_pr, attribute, line.split(":** ", 1)[1].strip().strip("`")
                )
                break
        value = _json_field(line, "Depends on", current_pr.name, errors)
        if line.startswith("**Depends on:** "):
            current_pr.fields.add("Depends on")
        if value is not None:
            if (
                isinstance(value, list)
                and len(value) == len(set(value))
                and all(
                    isinstance(item, str) and re.fullmatch(ID, item) for item in value
                )
            ):
                current_pr.depends_on = value
            else:
                errors.append(f"{current_pr.name} has invalid Depends on")

    _validate_structure(schema, feature_status, pull_requests, abandonment, commit_count, errors)
    return Ledger(schema, feature_status, pull_requests, abandonment, errors)


def _validate_structure(
    schema: int | None,
    feature_status: str | None,
    pull_requests: list[PullRequest],
    abandonment: dict[str, str],
    commit_count: str | None,
    errors: list[str],
) -> None:
    packages = [package for pr in pull_requests for package in pr.packages]
    if commit_count is not None and commit_count != str(len(packages)):
        errors.append("Commit packages total must match all package headings")
    for number, package in enumerate(packages, 1):
        match = NUMBERED_PACKAGE_HEADING.fullmatch(package.name)
        if commit_count is not None:
            if match is None or match.group(2) != str(number):
                errors.append(f"{package.name} must use Commit {number}: in its heading")
        elif re.search(r"\s+[—-]\s+Commit [0-9]+:", package.name):
            errors.append(f"{package.name} numbered headings require Commit packages")
    if schema != 4:
        errors.append("Executable ledger requires schema 4")
    if feature_status not in FEATURE_STATUSES:
        errors.append(f"invalid or missing feature status: {feature_status!r}")
    if not pull_requests:
        errors.append("ledger must contain at least one PR")

    seen_prs: set[str] = set()
    seen_packages: dict[str, tuple[PullRequest, Package]] = {}
    for pr in pull_requests:
        if pr.pr_id in seen_prs:
            errors.append(f"duplicate PR id: {pr.pr_id}")
        if pr.pr_id in seen_packages:
            errors.append(f"PR id collides with package id: {pr.pr_id}")
        seen_prs.add(pr.pr_id)
        if pr.status != "Planned" and not removed(pr.status):
            errors.append(f"{pr.name} has invalid or missing status: {pr.status!r}")
        if not pr.packages and not removed(pr.status):
            errors.append(f"{pr.name} must contain at least one package")
        if pr.branch and pr.base_branch and pr.branch == pr.base_branch:
            errors.append(f"{pr.name} branch must differ from its base branch")
        if pr.merge_ref not in {None, "Not merged"} and not immutable_git_oid(
            pr.merge_ref
        ):
            errors.append(f"{pr.name} has invalid Merge ref")
        for required in REQUIRED_PR_FIELDS:
            if required not in pr.fields:
                errors.append(f"{pr.name} is missing {required}")
        for label, value in (
            ("Branch", pr.branch),
            ("Base branch", pr.base_branch),
            ("Publication provider", pr.provider),
            ("PR template", pr.template),
            ("Merge method", pr.merge_method),
            ("Required checks", pr.required_checks),
            ("Provisional PR title", pr.title),
            ("Purpose", pr.purpose),
        ):
            if not value or value.startswith("<"):
                errors.append(f"{pr.name} has incomplete {label}")
        if pr.feature_closeout not in {
            "Not required",
            "Not run",
            "Current",
            "Required after publication correction",
        }:
            errors.append(f"{pr.name} has invalid Feature close-out")
        for dependency in pr.depends_on:
            if dependency not in seen_prs or dependency == pr.pr_id:
                errors.append(
                    f"{pr.name} dependency {dependency} must identify an earlier PR"
                )

        previous_wave = 1
        planned_seen = False
        for package_index, package in enumerate(pr.packages):
            if removed(pr.status) and package.status == "Planned":
                errors.append(
                    f"removed {pr.name} requires {package.name} to be Integrated or Removed"
                )
            if immutable_git_oid(pr.merge_ref) and package.status == "Planned":
                errors.append(f"merged {pr.name} cannot contain planned {package.name}")
            if package.package_id in seen_packages:
                errors.append(f"duplicate package id: {package.package_id}")
            if package.package_id in seen_prs:
                errors.append(f"package id collides with PR id: {package.package_id}")
            seen_packages[package.package_id] = (pr, package)
            if package.status not in PACKAGE_STATUSES and not removed(package.status):
                errors.append(
                    f"{pr.name} / {package.name} has invalid or missing status: {package.status!r}"
                )
            if package.status == "Planned":
                planned_seen = True
            elif package.status == "Integrated" and planned_seen:
                errors.append(
                    f"{pr.name} Integrated packages must form an ordered prefix"
                )
            if package.wave is None or package.wave not in {
                previous_wave,
                previous_wave + 1,
            }:
                errors.append(
                    f"{package.name} waves must begin at 1 and be contiguous/nondecreasing"
                )
            if package_index == 0 and package.wave != 1:
                errors.append(f"{pr.name} waves must begin at 1")
            if package.wave is not None:
                previous_wave = package.wave
            for label in ("Wave", "Depends on", "Write scope"):
                if label not in package.fields:
                    errors.append(f"{package.name} is missing {label}")
            if not removed(package.status):
                for required in REQUIRED_PACKAGE_FIELDS:
                    if required not in package.fields:
                        errors.append(
                            f"{pr.name} / {package.name} is missing {required}"
                        )
            for path in package.write_scope:
                if coordinator_owned_path(path):
                    errors.append(
                        f"{package.name} has coordinator-owned Write scope: {path}"
                    )

    for pr_index, pr in enumerate(pull_requests):
        for package in pr.packages:
            for dependency in package.depends_on:
                target = seen_packages.get(dependency)
                if target is None:
                    errors.append(
                        f"{package.name} dependency {dependency} does not exist"
                    )
                    continue
                target_pr, target_package = target
                target_pr_index = pull_requests.index(target_pr)
                if removed(target_package.status) and not removed(package.status):
                    errors.append(
                        f"{package.name} depends on removed package {dependency}"
                    )
                if target_pr_index > pr_index or (
                    target_pr is pr
                    and (
                        target_package.wave is None
                        or package.wave is None
                        or target_package.wave >= package.wave
                    )
                ):
                    errors.append(
                        f"{package.name} dependency {dependency} must belong to an earlier PR or earlier wave"
                    )
        for left_index, left in enumerate(pr.packages):
            for right in pr.packages[left_index + 1 :]:
                if (
                    removed(left.status)
                    or removed(right.status)
                    or left.wave != right.wave
                ):
                    continue
                for first in left.write_scope:
                    for second in right.write_scope:
                        if (
                            first == second
                            or first.startswith(second + "/")
                            or second.startswith(first + "/")
                        ):
                            errors.append(
                                f"same-wave Write scope overlap: {first} / {second}"
                            )

    if feature_status == "Abandoned":
        required = {
            "Developer approval",
            "Decision",
            "Cleanup",
            "Planning disposition",
            "Resume rule",
        }
        for field_name in required:
            value = abandonment.get(field_name)
            if not value or value == "..." or value.startswith("<"):
                errors.append(f"abandoned feature is missing {field_name}")
        if abandonment.get("Resume rule") != "Fresh plan required":
            errors.append("abandoned feature Resume rule must be 'Fresh plan required'")
        for pr in pull_requests:
            if pr.merge_ref == "Not merged" and not removed(pr.status):
                errors.append(
                    f"abandoned feature requires unfinished {pr.name} to be Removed"
                )
            for package in pr.packages:
                if package.status == "Planned":
                    errors.append(
                        f"abandoned feature requires {package.name} to be Integrated or Removed"
                    )


def package_digest(package: Package) -> str:
    """Hash static package authority while ignoring lifecycle status."""
    payload = "\n".join(
        line
        for line in package.authority_lines
        if line not in {"**Status:** Planned", "**Status:** Integrated"}
    ).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def inspect_ledger(path: Path) -> dict[str, object]:
    ledger = parse_ledger(path)
    errors = list(ledger.errors)
    remaining = [
        (pr, package)
        for pr in ledger.pull_requests
        if not removed(pr.status)
        for package in pr.packages
        if package.status == "Planned"
    ]
    result: dict[str, object] = {
        "ledger": str(path),
        "state": "invalid",
        "ledger_schema": ledger.schema,
        "feature_status": ledger.feature_status,
        "active_pr": None,
        "active_pr_id": None,
        "active_branch": None,
        "active_base_branch": None,
        "active_pr_dependency_merge_refs": [],
        "active_package": None,
        "active_package_id": None,
        "active_wave": None,
        "active_packages": [],
        "wave_packages": [],
        "boundary_pr": None,
        "boundary_pr_id": None,
        "package_digests": [
            {
                "pr": pr.pr_id,
                "package": package.package_id,
                "sha256": package_digest(package),
            }
            for pr in ledger.pull_requests
            for package in pr.packages
        ],
        "remaining_packages": len(remaining),
        "errors": errors,
    }
    if errors:
        return result
    if ledger.feature_status == "Abandoned":
        result["state"] = "abandoned"
        return result

    retained_prs = [pr for pr in ledger.pull_requests if not removed(pr.status)]
    live_prs = [pr for pr in retained_prs if pr.merge_ref == "Not merged"]
    if ledger.feature_status == "Completed" and (
        not retained_prs
        or any(not immutable_git_oid(pr.merge_ref) for pr in retained_prs)
    ):
        errors.append(
            "Completed feature requires every retained PR to have an immutable Merge ref"
        )
        return result
    if not live_prs:
        if ledger.feature_status == "Completed":
            result["state"] = "completed"
        else:
            errors.append(
                "all retained PRs are merged but feature status is not Completed"
            )
        return result

    active_pr = live_prs[0]
    active_index = ledger.pull_requests.index(active_pr)
    for predecessor in ledger.pull_requests[:active_index]:
        if not removed(predecessor.status) and not immutable_git_oid(
            predecessor.merge_ref
        ):
            errors.append(
                f"{active_pr.name} cannot start before {predecessor.name} is merged"
            )
    for successor in ledger.pull_requests[active_index + 1 :]:
        if immutable_git_oid(successor.merge_ref):
            errors.append(f"{successor.name} cannot be merged before {active_pr.name}")
        if any(package.status == "Integrated" for package in successor.packages):
            errors.append(
                f"{successor.name} cannot contain integrated packages before {active_pr.name} is merged"
            )
    if errors:
        return result

    result.update(
        active_pr=active_pr.name,
        active_pr_id=active_pr.pr_id,
        active_branch=active_pr.branch,
        active_base_branch=active_pr.base_branch,
        active_pr_dependency_merge_refs=[
            pr.merge_ref
            for pr in ledger.pull_requests[:active_index]
            if immutable_git_oid(pr.merge_ref)
        ],
    )
    planned = [package for package in active_pr.packages if package.status == "Planned"]
    if ledger.feature_status == "Blocked":
        result["state"] = "blocked"
        if planned:
            result["active_package"] = planned[0].name
            result["active_package_id"] = planned[0].package_id
        return result

    if planned:
        if ledger.feature_status != "Accepted":
            errors.append("runnable work requires feature status Accepted")
            return result
        wave = min(package.wave for package in planned if package.wave is not None)
        wave_packages = [
            package
            for package in active_pr.packages
            if package.wave == wave and package.status == "Planned"
        ]
        integrated_ids = {
            package.package_id
            for pr in ledger.pull_requests
            for package in pr.packages
            if package.status == "Integrated"
        }
        for package in wave_packages:
            missing = [
                dependency
                for dependency in package.depends_on
                if dependency not in integrated_ids
            ]
            if missing:
                errors.append(
                    f"{package.name} has unfinished dependencies: {', '.join(missing)}"
                )
        if errors:
            return result
        first = wave_packages[0]
        result.update(
            state="runnable",
            active_package=first.name,
            active_package_id=first.package_id,
            active_wave=wave,
            active_packages=[package.package_id for package in wave_packages],
            wave_packages=[
                {
                    "id": package.package_id,
                    "name": package.name,
                    "status": package.status,
                    "wave": package.wave,
                    "depends_on": package.depends_on,
                    "write_scope": package.write_scope,
                }
                for package in wave_packages
            ],
        )
        return result

    result["boundary_pr"] = active_pr.name
    result["boundary_pr_id"] = active_pr.pr_id
    is_final = not any(
        not removed(pr.status) for pr in ledger.pull_requests[active_index + 1 :]
    )
    if is_final and active_pr.feature_closeout != "Current":
        result["state"] = "implementation_complete"
    else:
        result["state"] = "pr_boundary"
    return result


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: ledger_state.py <feature-ledger.md>", file=sys.stderr)
        return 2
    try:
        result = inspect_ledger(Path(argv[1]))
    except (OSError, UnicodeError) as error:
        result = {"ledger": argv[1], "state": "invalid", "errors": [str(error)]}
    print(json.dumps(result, indent=2, sort_keys=True))
    return 2 if result["state"] == "invalid" else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
