#!/usr/bin/env python3
"""Read-only validation of an isolated worker's same-base package result."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

from delivery_state import inspect_delivery
from ledger_state import (
    canonical_file_path,
    coordinator_owned_path,
    immutable_git_oid,
    inspect_ledger,
)


def git(root: Path, *arguments: str) -> bytes:
    environment = {
        key: value for key, value in os.environ.items() if not key.startswith("GIT_")
    }
    environment.update(
        GIT_CONFIG_NOSYSTEM="1",
        GIT_CONFIG_GLOBAL=os.devnull,
        GIT_NO_REPLACE_OBJECTS="1",
    )
    process = subprocess.run(
        ["git", "--no-pager", "-C", str(root), *arguments],
        capture_output=True,
        check=False,
        env=environment,
    )
    if process.returncode:
        raise ValueError(
            f"Git {' '.join(arguments[:1])} failed: {process.stderr.decode(errors='replace').strip()}"
        )
    return process.stdout


def inspect_result(
    ledger: Path,
    repository_root: Path,
    base: str,
    package_id: str,
    result_sha: str,
    *,
    staged: bool = False,
    expected_plan_digest: str | None = None,
    expected_package_digest: str | None = None,
    expected_head: str | None = None,
) -> dict[str, object]:
    expectations = (expected_plan_digest, expected_package_digest, expected_head)
    guarded = any(value is not None for value in expectations)
    result: dict[str, object] = {
        "state": "invalid",
        "packet": None,
        "package_digest": None,
        "plan_digest": None,
        "head_sha": None,
        "authority_matches": False if guarded else None,
        "base_sha": base,
        "result_sha": result_sha,
        "changed_paths": [],
        "patch_sha256": None,
        "staged_matches_result": False if staged else None,
        "errors": [],
    }
    errors: list[str] = []
    result["errors"] = errors
    try:
        state = inspect_ledger(ledger)
        if state["state"] != "runnable":
            raise ValueError(f"ledger must be runnable schema 4: {state['errors']}")
        root = repository_root.resolve(strict=True)
        if (
            Path(
                os.fsdecode(git(root, "rev-parse", "--show-toplevel")).strip()
            ).resolve()
            != root
        ):
            raise ValueError("repository-root must be the Git worktree root")
        ledger_relative = ledger.resolve(strict=True).relative_to(root).as_posix()
        packets = state["wave_packages"]
        assert isinstance(packets, list)
        packet = next((item for item in packets if item["id"] == package_id), None)
        if packet is None:
            raise ValueError("package must be a Planned member of the current PR/wave")
        result["packet"] = packet
        digests = state["package_digests"]
        assert isinstance(digests, list)
        result["package_digest"] = next(item["sha256"] for item in digests if item["package"] == package_id)
        if guarded:
            if any(value is None for value in expectations):
                raise ValueError("expected plan digest, package digest and HEAD must be supplied together")
            if not immutable_git_oid(expected_head):
                raise ValueError("expected HEAD must be an exact immutable full SHA id")
            delivery = inspect_delivery(ledger, root)
            if delivery["state"] == "invalid":
                raise ValueError(f"invalid delivery authority: {delivery['errors']}")
            result["plan_digest"] = delivery["plan_digest"]
            result["head_sha"] = os.fsdecode(git(root, "rev-parse", "--verify", "HEAD")).strip()
            if result["plan_digest"] != expected_plan_digest:
                raise ValueError("accepted plan digest changed; reconcile authority before integration")
            if result["package_digest"] != expected_package_digest:
                raise ValueError("accepted package digest changed; reconcile authority before integration")
            if result["head_sha"] != expected_head:
                raise ValueError("integration HEAD changed; reconcile progress before integration")
            result["authority_matches"] = True
        for oid in (base, result_sha):
            if not immutable_git_oid(oid):
                raise ValueError("base and result must be exact immutable full SHA ids")
            if git(root, "cat-file", "-t", oid).strip() != b"commit":
                raise ValueError("base and result must identify commit objects")
            if git(root, "rev-parse", "--verify", oid).strip().decode() != oid:
                raise ValueError("SHA must match the repository object format")
        git(root, "merge-base", "--is-ancestor", base, result_sha)
        changed = [
            os.fsdecode(path)
            for path in git(
                root,
                "diff",
                "--no-ext-diff",
                "--no-textconv",
                "--no-renames",
                "--name-only",
                "-z",
                base,
                result_sha,
                "--",
            ).split(b"\0")
            if path
        ]
        result["changed_paths"] = sorted(changed)
        if not changed:
            raise ValueError("result contains no changes")
        for path in changed:
            if not canonical_file_path(path) or path not in packet["write_scope"]:
                raise ValueError(f"changed path outside exact Write scope: {path}")
            if path == ledger_relative or coordinator_owned_path(path):
                raise ValueError(f"coordinator-owned path: {path}")
            candidate = root / path
            candidate.resolve().relative_to(root)
            if any(
                parent.is_symlink()
                for parent in (candidate, *candidate.parents)
                if parent != root and root in parent.parents
            ):
                raise ValueError(f"symlink traversal: {path}")
            if candidate.is_dir():
                raise ValueError(f"Write scope must identify files: {path}")
            for oid in (base, result_sha):
                entry = git(root, "ls-tree", "-z", oid, "--", path)
                if entry and not entry.startswith((b"100644 ", b"100755 ")):
                    raise ValueError(f"non-regular Git path: {path}")
        diff_options = ("--binary", "--full-index", "--no-ext-diff", "--no-textconv", "--no-renames")
        patch = git(root, "diff", *diff_options, base, result_sha, "--")
        result["patch_sha256"] = hashlib.sha256(patch).hexdigest()
        if staged:
            if state["active_package_id"] != package_id:
                raise ValueError("staged result must be the active serial integration package")
            staged_paths = {
                os.fsdecode(path) for path in git(
                    root, "diff", "--cached", "--name-only", "--no-renames", "-z", "--",
                ).split(b"\0") if path
            }
            if staged_paths - set(packet["write_scope"]) - {ledger_relative}:
                raise ValueError("staged paths outside package scope and coordinator ledger")
            if git(root, "diff", "--name-only", "-z", "--"):
                raise ValueError("worktree has unstaged changes")
            if git(root, "ls-files", "--others", "--exclude-standard", "-z"):
                raise ValueError("worktree has untracked non-ignored files")
            actual = git(root, "diff", "--cached", *diff_options, "--", *packet["write_scope"])
            if actual != patch:
                raise ValueError("staged package differs from frozen result; fresh review required")
            result["staged_matches_result"] = True
        result["state"] = "valid"
    except (OSError, UnicodeError, ValueError, RuntimeError) as error:
        errors.append(str(error))
    return result


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("ledger", type=Path)
    parser.add_argument("repository_root", type=Path)
    parser.add_argument("wave_base_sha")
    parser.add_argument("package_id")
    parser.add_argument("result_sha")
    parser.add_argument("--staged", action="store_true")
    parser.add_argument("--expected-plan-digest", help="accepted plan digest captured before dispatch")
    parser.add_argument("--expected-package-digest", help="accepted package digest captured before dispatch")
    parser.add_argument("--expected-head", help="expected integration HEAD from verified run evidence")
    args = parser.parse_args(argv[1:])
    result = inspect_result(
        args.ledger, args.repository_root, args.wave_base_sha, args.package_id,
        args.result_sha, staged=args.staged,
        expected_plan_digest=args.expected_plan_digest,
        expected_package_digest=args.expected_package_digest,
        expected_head=args.expected_head,
    )
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["state"] == "valid" else 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
