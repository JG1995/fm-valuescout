#!/usr/bin/env python3
"""Local-only delivery authority for an accepted ignored schema-4 plan.

This is the local Git delivery seam for plans whose accepted DESIGN and
IMPLEMENTATION ledger live outside Git history under the ignored .wiki tree.
A deterministic plan digest plus immutable snapshot copies under the ignored
.work/<slug>/ directory supply acceptance authority, and every accepted
code commit is anchored to a verified HEAD. Local mode rejects any .wiki/
or .work/ path in a package Write scope, staged set, or committed result —
even force-added — and reads the ledger and DESIGN only from working-tree
files, never from Git objects.

Commands are fail-closed. A pending marker records each verified staged
set, index tree, and expected HEAD before the code-only content commit;
`record` advances the journal only against exact Git and ledger evidence.
Any interruption between precommit, commit, ledger edit, and record leaves
a pending marker, so `verify` rejects instead of inferring. Resuming after
an interruption is a coordinator decision: `record` completes a verified
flow only when the exact evidence exists. After a verified publication
boundary, `record-boundary` anchors the next code HEAD; hosted provider
checks remain independent evidence, not something local Git can prove.

Git cannot restore ignored authority, so every saved state change also
copies the canonical .wiki/ and .work/<slug>/ to a new timestamped backup
outside the repository; `backup` makes one on demand.
"""

from __future__ import annotations

import argparse
import contextlib
import hashlib
import json
import os
import re
import shutil
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from delivery_state import (
    PLAN_SECTIONS,
    inspect_delivery,
    pull_request_authority,
    section_text,
)
from ledger_state import (
    canonical_file_path,
    coordinator_owned_path,
    immutable_git_oid,
    inspect_ledger,
    parse_ledger,
    removed,
)
from publication_state import GITHUB_PR_URL, manual_ref, unwrap_markdown_autolink
from wave_result import git

STATE_SCHEMA = 1
MANIFEST_SCHEMA = 1
BACKUP_KEEP = 10
SLUG = r"[a-z0-9]+(?:-[a-z0-9]+)*"


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def atomic_write(path: Path, data: str) -> None:
    """Write text via temp file + fsync + rename in the target directory."""
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, tmp = tempfile.mkstemp(prefix=f".{path.name}.", dir=str(path.parent))
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(tmp, path)
    except BaseException:
        with contextlib.suppress(OSError):
            os.unlink(tmp)
        raise


def ignored_authority_path(path: str) -> bool:
    """True for any .wiki/ or .work/ root path that local mode keeps out of Git."""
    return path in {".wiki", ".work"} or path.startswith((".wiki/", ".work/"))


def resolve_ledger(ledger: Path, root: Path) -> tuple[Path, str, str]:
    """Return (resolved root, feature slug, repo-relative ledger path)."""
    root = root.resolve(strict=True)
    if ledger.is_symlink() or any(parent.is_symlink() for parent in ledger.parents):
        raise ValueError("feature ledger path must not traverse symlinks")
    try:
        relative = ledger.resolve(strict=True).relative_to(root)
    except (OSError, ValueError):
        raise ValueError(
            "feature ledger must live inside the repository root"
        ) from None
    parts = relative.parts
    if (
        len(parts) != 4
        or parts[:2] != (".wiki", "features")
        or not re.fullmatch(SLUG, parts[2])
        or parts[3] != f"{parts[2].upper()}-IMPLEMENTATION.md"
    ):
        raise ValueError(
            "ledger must be .wiki/features/<slug>/<SLUG>-IMPLEMENTATION.md"
        )
    design = ledger.with_name(f"{parts[2].upper()}-DESIGN.md")
    if design.is_symlink() or not design.is_file():
        raise ValueError("retained DESIGN file is missing or a symlink")
    tracked = git(
        root, "ls-files", "--cached", "-z", "--",
        relative.as_posix(), relative.with_name(design.name).as_posix(),
    )
    if tracked:
        raise ValueError("local delivery requires DESIGN and ledger excluded from Git")
    return root, parts[2], relative.as_posix()


def work_dir(root: Path, slug: str) -> Path:
    return root / ".work" / slug


def state_path(root: Path, slug: str) -> Path:
    return work_dir(root, slug) / "state.json"


def snapshot_paths(run_dir: Path) -> dict[str, Path]:
    return {
        "DESIGN": run_dir / "DESIGN.snapshot.md",
        "IMPLEMENTATION": run_dir / "IMPLEMENTATION.snapshot.md",
    }


def read_state(root: Path, slug: str) -> dict[str, Any]:
    path = state_path(root, slug)
    if not path.is_file():
        raise ValueError(f"no accepted delivery state at {path}; run accept first")
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except ValueError as error:
        raise ValueError(f"corrupt delivery state at {path}: {error}") from error
    if not isinstance(data, dict) or data.get("state_schema") != STATE_SCHEMA:
        raise ValueError("unsupported delivery state schema")
    if (
        data.get("slug") != slug
        or not isinstance(data.get("base_head"), str)
        or not immutable_git_oid(data["base_head"])
        or not isinstance(data.get("expected_head"), str)
        or not immutable_git_oid(data["expected_head"])
        or not isinstance(data.get("plan_digest"), str)
        or not isinstance(data.get("ledger"), str)
        or not isinstance(data.get("package_sha256"), dict)
        or not isinstance(data.get("snapshot_sha256"), dict)
        or not isinstance(data.get("integrated"), dict)
        or not isinstance(data.get("journal"), list)
        or any(not isinstance(entry, dict) for entry in data["journal"])
        or (data.get("pending") is not None and not isinstance(data["pending"], dict))
    ):
        raise ValueError("corrupt delivery state structure")
    return data


def backup_dir(root: Path, slug: str) -> Path:
    """Per-checkout, per-feature backup directory under the XDG state home."""
    state_home = os.environ.get("XDG_STATE_HOME") or Path.home() / ".local/state"
    checkout = f"{root.name}-{sha256_bytes(os.fsencode(root))[:12]}"
    return Path(state_home) / "jay-claude" / "backups" / checkout / slug


def write_backup(root: Path, slug: str) -> Path:
    """Copy .wiki/ and .work/<slug>/ to a new backup; keep the newest BACKUP_KEEP."""
    base = backup_dir(root, slug)
    if base.resolve().is_relative_to(root):
        raise ValueError(f"backup directory {base} must be outside the repository")
    base.mkdir(parents=True, exist_ok=True)
    # Copy into a hidden staging directory so a listed backup is always complete.
    staging = Path(tempfile.mkdtemp(prefix=".partial-", dir=base))
    try:
        shutil.copytree(root / ".wiki", staging / "wiki", symlinks=True)
        shutil.copytree(work_dir(root, slug), staging / "work", symlinks=True)
        target = base / datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
        os.rename(staging, target)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise
    backups = sorted(path for path in base.iterdir() if not path.name.startswith("."))
    for old in backups[:-BACKUP_KEEP]:
        shutil.rmtree(old)
    return target


def save_state(root: Path, slug: str, state: dict[str, Any]) -> str:
    """Write state.json, then back up the ignored authority it belongs to."""
    atomic_write(
        state_path(root, slug), json.dumps(state, indent=2, sort_keys=True) + "\n"
    )
    try:
        return str(write_backup(root, slug))
    except (OSError, ValueError) as error:
        raise ValueError(
            f"delivery state saved, but its backup failed: {error}; fix the cause "
            "and run backup instead of repeating this command"
        ) from error


def require_git_root(root: Path) -> Path:
    top = Path(
        os.fsdecode(git(root, "rev-parse", "--show-toplevel")).strip()
    ).resolve()
    if top != root.resolve():
        raise ValueError("repository-root must be the Git worktree root")
    return top


def absolute_head(root: Path) -> str:
    oid = os.fsdecode(git(root, "rev-parse", "HEAD")).strip()
    if not immutable_git_oid(oid):
        raise ValueError(f"invalid Git HEAD: {oid!r}")
    return oid


def require_clean_tracked(root: Path) -> None:
    status = os.fsdecode(git(root, "status", "--porcelain"))
    if status.strip():
        raise ValueError(
            "tracked worktree is not clean; commit or revert unrelated changes"
        )


def staged_paths(root: Path) -> list[str]:
    raw = git(root, "diff", "--cached", "--name-only", "-z", "--")
    return [os.fsdecode(path) for path in raw.split(b"\0") if path]


def index_mode(root: Path, path: str) -> str | None:
    raw = git(root, "ls-files", "--stage", "-z", "--", path)
    if not raw:
        return None
    return os.fsdecode(raw.split(b"\0", 1)[0]).split(" ", 1)[0]


def tree_mode(root: Path, rev: str, path: str) -> str | None:
    raw = git(root, "ls-tree", rev, "--", path)
    if not raw:
        return None
    return os.fsdecode(raw.split(b"\t", 1)[0].split()[0])


def check_index_path(root: Path, path: str) -> None:
    """Reject unsafe, .wiki/.work, coordinator-owned, symlinked, or non-regular paths."""
    if not canonical_file_path(path):
        raise ValueError(f"unsafe staged path: {path!r}")
    if ignored_authority_path(path):
        raise ValueError(f"ignored authority path: {path}")
    if coordinator_owned_path(path):
        raise ValueError(f"coordinator-owned staged path: {path}")
    candidate = root / path
    for parent in candidate.parents:
        if parent == root:
            break
        if parent.is_symlink():
            raise ValueError(f"symlink traversal in staged path: {path}")
    mode = index_mode(root, path)
    if mode is None:
        if tree_mode(root, "HEAD", path) not in ("100644", "100755"):
            raise ValueError(f"staged deletion of a non-regular path: {path}")
        return
    if mode not in ("100644", "100755"):
        raise ValueError(f"staged path is not a regular file: {path}")
    if candidate.is_dir():
        raise ValueError(f"staged path must identify a file: {path}")


def require_staged_only(root: Path) -> None:
    for line in os.fsdecode(git(root, "status", "--porcelain")).splitlines():
        if not line or line[0] == " " or line[:2] in ("??", "!!"):
            raise ValueError(f"untracked or unstaged tracked change: {line}")
        if len(line) > 1 and line[1] != " ":
            raise ValueError(f"unstaged tracked change: {line}")


def integrated_ids(ledger: Path) -> list[str]:
    return sorted(
        package.package_id
        for pr in parse_ledger(ledger).pull_requests
        for package in pr.packages
        if package.status == "Integrated"
    )


def find_package(ledger: Path, package_id: str) -> Any:
    for pr in parse_ledger(ledger).pull_requests:
        for package in pr.packages:
            if package.package_id == package_id:
                return package
    raise ValueError(f"package not found: {package_id}")


def require_snapshots(run_dir: Path, state: dict[str, Any]) -> None:
    for name, path in snapshot_paths(run_dir).items():
        if sha256_file(path) != state["snapshot_sha256"].get(name):
            raise ValueError(f"snapshot copy changed: {path}")


def require_ignored_write_scopes(ledger: Path) -> None:
    """Refuse accepted authority whose packages own .wiki/ or .work/ paths."""
    for pr in parse_ledger(ledger).pull_requests:
        if removed(pr.status):
            continue
        for package in pr.packages:
            if removed(package.status):
                continue
            for path in package.write_scope:
                if ignored_authority_path(path):
                    raise ValueError(
                        f"package Write scope contains ignored authority path: {path}"
                    )


def accept(ledger: Path, root: Path, approved: bool = False) -> dict[str, Any]:
    errors: list[str] = []
    try:
        if not approved:
            raise ValueError("accept requires --approved developer approval")
        root, slug, relative = resolve_ledger(ledger, root)
        require_git_root(root)
        require_clean_tracked(root)
        delivery = inspect_delivery(ledger, root)
        if delivery.get("state") == "invalid":
            raise ValueError(
                "ledger is invalid: " + "; ".join(map(str, delivery["errors"]))
            )
        ledger_data = inspect_ledger(ledger)
        retained = [
            pr for pr in parse_ledger(ledger).pull_requests if not removed(pr.status)
        ]
        if any(
            package.status != "Planned" for pr in retained for package in pr.packages
        ):
            raise ValueError(
                "accept requires every retained package to be initially Planned"
            )
        require_ignored_write_scopes(ledger)
        base = absolute_head(root)
        run_dir = work_dir(root, slug)
        snapshots = snapshot_paths(run_dir)
        if state_path(root, slug).exists() or any(
            path.exists() for path in snapshots.values()
        ):
            raise ValueError(
                f"delivery state already exists at {run_dir}; refusing to overwrite"
            )
        plan_digest = str(delivery["plan_digest"])
        package_digests = ledger_data["package_digests"]
        assert isinstance(package_digests, list)
        state: dict[str, Any] = {
            "state_schema": STATE_SCHEMA,
            "slug": slug,
            "ledger": relative,
            "plan_digest": plan_digest,
            "base_head": base,
            "expected_head": base,
            "package_sha256": {
                item["package"]: item["sha256"] for item in package_digests
            },
            "snapshot_sha256": {},
            "integrated": {},
            "journal": [],
            "pending": None,
        }
        # Snapshot copies first, state.json last, so partial accepts stay refused.
        for name in ("DESIGN", "IMPLEMENTATION"):
            source = (
                ledger.with_name(f"{slug.upper()}-DESIGN.md")
                if name == "DESIGN"
                else ledger
            )
            atomic_write(snapshots[name], source.read_text(encoding="utf-8"))
            state["snapshot_sha256"][name] = sha256_file(snapshots[name])
        backup_path = save_state(root, slug, state)
        return {
            "state": "ok",
            "slug": slug,
            "plan_digest": plan_digest,
            "base_head": base,
            "state_file": str(state_path(root, slug)),
            "backup": backup_path,
        }
    except (OSError, UnicodeError, ValueError, RuntimeError) as error:
        errors.append(str(error))
        return {"state": "error", "errors": errors}


def renew(ledger: Path, root: Path, approved: bool = False) -> dict[str, Any]:
    """Replace accepted static authority after a new independent plan review."""
    try:
        if not approved:
            raise ValueError(
                "renew requires --approved after a fresh independent plan review"
            )
        root, slug, _ = resolve_ledger(ledger, root)
        require_git_root(root)
        require_clean_tracked(root)
        state = read_state(root, slug)
        if state.get("pending") is not None or absolute_head(root) != state[
            "expected_head"
        ]:
            raise ValueError("pending or divergent Git progress requires reconciliation")
        require_snapshots(work_dir(root, slug), state)
        if set(integrated_ids(ledger)) != set(state["integrated"]):
            raise ValueError(
                "ledger integrated progress differs from accepted journal"
            )
        delivery = inspect_delivery(ledger, root)
        if delivery["state"] == "invalid":
            raise ValueError(
                "new plan is invalid: " + "; ".join(delivery["errors"])
            )
        require_ignored_write_scopes(ledger)
        package_digests = inspect_ledger(ledger)["package_digests"]
        assert isinstance(package_digests, list)
        packages = {entry["package"]: entry["sha256"] for entry in package_digests}
        for package_id in state["integrated"]:
            if packages.get(package_id) != state["package_sha256"].get(package_id):
                raise ValueError(f"integrated package authority changed: {package_id}")
        snapshots = snapshot_paths(work_dir(root, slug))
        for name, source in (
            ("DESIGN", ledger.with_name(f"{slug.upper()}-DESIGN.md")),
            ("IMPLEMENTATION", ledger),
        ):
            atomic_write(snapshots[name], source.read_text(encoding="utf-8"))
            state["snapshot_sha256"][name] = sha256_file(snapshots[name])
        state["plan_digest"] = delivery["plan_digest"]
        state["package_sha256"] = packages
        backup_path = save_state(root, slug, state)
        return {
            "state": "ok",
            "plan_digest": state["plan_digest"],
            "expected_head": state["expected_head"],
            "backup": backup_path,
        }
    except (OSError, UnicodeError, ValueError, RuntimeError) as error:
        return {"state": "error", "errors": [str(error)]}


def verify(ledger: Path, root: Path) -> dict[str, Any]:
    errors: list[str] = []
    try:
        root, slug, _ = resolve_ledger(ledger, root)
        require_git_root(root)
        state = read_state(root, slug)
        delivery = inspect_delivery(ledger, root)
        if delivery.get("state") == "invalid":
            raise ValueError(
                "ledger is invalid: " + "; ".join(map(str, delivery["errors"]))
            )
        if delivery["plan_digest"] != state["plan_digest"]:
            raise ValueError("static plan digest drifted from the accepted snapshot")
        package_digests = inspect_ledger(ledger)["package_digests"]
        assert isinstance(package_digests, list)
        static = {
            item["package"]: item["sha256"] for item in package_digests
        }
        if static != state["package_sha256"]:
            raise ValueError("package static hashes drifted from the accepted snapshot")
        require_snapshots(work_dir(root, slug), state)
        if state.get("pending") is not None:
            raise ValueError(
                "pending package marker exists: delivery was interrupted between "
                "precommit and record; verify refuses to infer — advance with "
                "record only after explicit reconciliation approval"
            )
        head = absolute_head(root)
        if head != state["expected_head"]:
            raise ValueError(
                f"HEAD {head} does not match expected_head {state['expected_head']}; "
                "an interrupted delivery must be reconciled, not inferred"
            )
        journal = state.get("journal", [])
        integrated = state.get("integrated", {})
        ledger_ids = set(integrated_ids(ledger))
        integration_entries = [
            entry
            for entry in journal
            if entry.get("kind", "integration") == "integration"
        ]
        journal_ids = {entry["package"] for entry in integration_entries}
        if ledger_ids != journal_ids or set(integrated) != journal_ids or len(integration_entries) != len(journal_ids):
            raise ValueError(
                "ledger Integrated packages do not exactly match the journal"
            )
        boundary_entries = [entry for entry in journal if entry.get("kind") == "boundary"]
        boundary_ids = {entry.get("pr") for entry in boundary_entries}
        merged_ids = {
            pr.pr_id for pr in parse_ledger(ledger).pull_requests
            if not removed(pr.status) and immutable_git_oid(pr.merge_ref)
        }
        if boundary_ids != merged_ids or len(boundary_entries) != len(boundary_ids):
            raise ValueError("ledger merged boundaries do not match the journal")
        boundary_sources = [entry.get("source_head") for entry in boundary_entries]
        for entry in journal:
            commit = entry.get("commit")
            package = entry.get("package")
            kind = entry.get("kind", "integration")
            if not immutable_git_oid(commit):
                raise ValueError("journal contains an invalid commit")
            if kind == "integration" and integrated.get(package) != commit:
                raise ValueError(
                    f"journal entry for {package} does not match the integrated map"
                )
            if kind == "correction" and package not in integrated:
                raise ValueError(
                    f"correction has no integrated owning package: {package}"
                )
            if kind == "boundary":
                boundary = next(
                    (pr for pr in parse_ledger(ledger).pull_requests
                     if pr.pr_id == entry.get("pr")), None
                )
                if boundary is None or boundary.merge_ref != commit:
                    raise ValueError("boundary journal disagrees with the ledger merge ref")
                if not immutable_git_oid(entry.get("source_head")) or not immutable_git_oid(entry.get("target_before")):
                    raise ValueError("boundary journal has invalid source or target")
                git(root, "merge-base", "--is-ancestor", entry["target_before"], commit)
            elif kind not in {"integration", "correction"}:
                raise ValueError(f"unknown journal entry kind: {kind}")
            for reachable in (head, *boundary_sources):
                try:
                    git(root, "merge-base", "--is-ancestor", commit, reachable)
                    break
                except ValueError:
                    continue
            else:
                raise ValueError(f"journal commit is not reachable from an accepted boundary: {commit}")
        return {
            "state": "ok",
            "plan_digest": state["plan_digest"],
            "head": head,
            "expected_head": state["expected_head"],
            "integrated": sorted(ledger_ids),
            "journal": journal,
        }
    except (OSError, UnicodeError, ValueError, RuntimeError) as error:
        errors.append(str(error))
        return {"state": "error", "errors": errors}


def context(ledger: Path, root: Path, package_id: str) -> dict[str, Any]:
    errors: list[str] = []
    try:
        checked = verify(ledger, root)
        if checked["state"] != "ok":
            raise ValueError("verify failed: " + "; ".join(checked["errors"]))
        root, slug, _ = resolve_ledger(ledger, root)
        state = read_state(root, slug)
        classified = inspect_ledger(ledger)
        wave_packages = classified.get("wave_packages", [])
        assert isinstance(wave_packages, list)
        packet = next(
            (item for item in wave_packages if item["id"] == package_id), None
        )
        if classified.get("state") != "runnable" or packet is None:
            raise ValueError("package must be a Planned member of the current wave")
        manifest: dict[str, Any] = {
            "manifest_schema": MANIFEST_SCHEMA,
            "package_id": package_id,
            "base_head": state["expected_head"],
            "plan_digest": state["plan_digest"],
            "package_sha256": state["package_sha256"][package_id],
            "snapshot_paths": {
                name: str(path)
                for name, path in snapshot_paths(work_dir(root, slug)).items()
            },
            "snapshot_sha256": dict(state["snapshot_sha256"]),
            "write_scope": list(find_package(ledger, package_id).write_scope),
        }
        manifest_path = work_dir(root, slug) / f"context-{package_id}.json"
        atomic_write(
            manifest_path, json.dumps(manifest, indent=2, sort_keys=True) + "\n"
        )
        return {"state": "ok", "manifest": str(manifest_path), **manifest}
    except (OSError, UnicodeError, ValueError, RuntimeError) as error:
        errors.append(str(error))
        return {"state": "error", "errors": errors}


def worker_verify(manifest_path: Path, worker_root: Path) -> dict[str, Any]:
    errors: list[str] = []
    try:
        manifest = json.loads(Path(manifest_path).read_text(encoding="utf-8"))
        required = {
            "manifest_schema",
            "package_id",
            "base_head",
            "plan_digest",
            "package_sha256",
            "snapshot_paths",
            "snapshot_sha256",
            "write_scope",
        }
        if (
            not isinstance(manifest, dict)
            or manifest.get("manifest_schema") != MANIFEST_SCHEMA
            or required - set(manifest)
            or not isinstance(manifest["snapshot_paths"], dict)
            or not isinstance(manifest["snapshot_sha256"], dict)
            or not all(isinstance(path, str) for path in manifest["snapshot_paths"].values())
            or not isinstance(manifest["write_scope"], list)
            or not all(isinstance(path, str) for path in manifest["write_scope"])
            or not isinstance(manifest["plan_digest"], str)
            or not isinstance(manifest["package_sha256"], str)
            or not isinstance(manifest["base_head"], str)
        ):
            raise ValueError("invalid worker manifest")
        worker_root = worker_root.resolve(strict=True)
        top = Path(
            os.fsdecode(git(worker_root, "rev-parse", "--show-toplevel")).strip()
        ).resolve()
        if top != worker_root:
            raise ValueError("worker-root is not the Git worktree root")
        head = os.fsdecode(git(worker_root, "rev-parse", "HEAD")).strip()
        if not immutable_git_oid(head):
            raise ValueError(f"invalid worker HEAD: {head!r}")
        if head != manifest["base_head"]:
            raise ValueError(
                f"worker HEAD {head} does not match the manifest base {manifest['base_head']}"
            )
        if set(manifest["snapshot_paths"]) != {"DESIGN", "IMPLEMENTATION"}:
            raise ValueError("worker manifest must include both accepted snapshots")
        for name, raw_path in manifest["snapshot_paths"].items():
            resolved = Path(raw_path).resolve(strict=True)
            if resolved.is_dir():
                raise ValueError(f"snapshot must identify a file: {raw_path}")
            try:
                common = Path(os.path.commonpath([str(resolved), str(worker_root)]))
            except ValueError:
                common = None
            if common == worker_root:
                raise ValueError(
                    f"snapshot must live outside the worker tree: {raw_path}"
                )
            expected = manifest["snapshot_sha256"].get(name)
            if not expected or sha256_file(resolved) != expected:
                raise ValueError(f"snapshot hash mismatch: {raw_path}")
        implementation = Path(manifest["snapshot_paths"]["IMPLEMENTATION"])
        design = Path(manifest["snapshot_paths"]["DESIGN"])
        lines = implementation.read_text(encoding="utf-8").splitlines()
        authority_errors: list[str] = []
        authority = {
            "design_content": design.read_text(encoding="utf-8"),
            "feature_plan": {
                heading: section_text(lines, heading) for heading in PLAN_SECTIONS
            },
            "pull_requests": pull_request_authority(
                implementation, worker_root, authority_errors
            ),
            "packages": inspect_ledger(implementation)["package_digests"],
        }
        if authority_errors:
            raise ValueError(
                "worker snapshot authority is invalid: "
                + "; ".join(authority_errors)
            )
        digest = sha256_bytes(
            json.dumps(authority, sort_keys=True, separators=(",", ":")).encode()
        )
        if digest != manifest["plan_digest"]:
            raise ValueError(
                "worker snapshot plan digest does not match the manifest"
            )
        packages = authority["packages"]
        assert isinstance(packages, list)
        packet = next(
            (
                item
                for item in packages
                if item["package"] == manifest["package_id"]
            ),
            None,
        )
        if packet is None or packet["sha256"] != manifest["package_sha256"]:
            raise ValueError(
                "worker package digest does not match the accepted snapshot"
            )
        if list(
            find_package(implementation, manifest["package_id"]).write_scope
        ) != manifest["write_scope"]:
            raise ValueError(
                "worker Write scope does not match the accepted snapshot"
            )
        return {
            "state": "ok",
            "worker_root": str(worker_root),
            "base_head": head,
        }
    except (OSError, UnicodeError, ValueError, RuntimeError) as error:
        errors.append(str(error))
        return {"state": "error", "errors": errors}


def precommit(
    ledger: Path, root: Path, package_id: str, correction: bool = False
) -> dict[str, Any]:
    errors: list[str] = []
    try:
        root, slug, _ = resolve_ledger(ledger, root)
        require_git_root(root)
        checked = verify(ledger, root)
        if checked["state"] != "ok":
            raise ValueError("verify failed: " + "; ".join(checked["errors"]))
        state = read_state(root, slug)
        if state.get("pending") is not None:
            raise ValueError(
                "pending marker exists: an earlier delivery was interrupted; "
                "complete or discard it only after explicit reconciliation approval"
            )
        delivery = inspect_delivery(ledger, root)
        if delivery["state"] == "invalid":
            raise ValueError(
                "ledger is invalid: " + "; ".join(delivery["errors"])
            )
        if not correction and delivery["state"] != "runnable":
            raise ValueError(f"ledger is not runnable: {delivery['state']}")
        if not correction and delivery["active_package_id"] != package_id:
            raise ValueError(
                f"active package is {delivery['active_package_id']!r}, not {package_id!r}"
            )
        package = find_package(ledger, package_id)
        expected_status = "Integrated" if correction else "Planned"
        if package.status != expected_status:
            raise ValueError(f"package must be {expected_status} before staging")
        if correction and package_id not in state["integrated"]:
            raise ValueError("correction owner has no verified integrated commit")
        paths = staged_paths(root)
        if not paths:
            raise ValueError("nothing is staged; stage the package changes first")
        require_staged_only(root)
        scope = set(package.write_scope)
        for path in paths:
            check_index_path(root, path)
            if path not in scope:
                raise ValueError(f"staged path outside Write scope: {path}")
        pending = {
            "kind": "correction" if correction else "integration",
            "package": package_id,
            "expected_head": state["expected_head"],
            "tree": os.fsdecode(git(root, "write-tree")).strip(),
            "paths": sorted(set(paths)),
            "integrated_before": integrated_ids(ledger),
        }
        state["pending"] = pending
        backup_path = save_state(root, slug, state)
        return {"state": "ok", "pending": pending, "backup": backup_path}
    except (OSError, UnicodeError, ValueError, RuntimeError) as error:
        errors.append(str(error))
        return {"state": "error", "errors": errors}


def record(
    ledger: Path, root: Path, package_id: str, correction: bool = False
) -> dict[str, Any]:
    errors: list[str] = []
    try:
        root, slug, _ = resolve_ledger(ledger, root)
        require_git_root(root)
        state = read_state(root, slug)
        delivery = inspect_delivery(ledger, root)
        if delivery.get("state") == "invalid":
            raise ValueError(
                "ledger is invalid: " + "; ".join(map(str, delivery["errors"]))
            )
        if delivery["plan_digest"] != state["plan_digest"]:
            raise ValueError("static plan digest drifted; reconciliation required")
        package_digests = inspect_ledger(ledger)["package_digests"]
        assert isinstance(package_digests, list)
        static = {
            item["package"]: item["sha256"] for item in package_digests
        }
        if static != state["package_sha256"]:
            raise ValueError("package static hashes drifted; reconciliation required")
        require_snapshots(work_dir(root, slug), state)
        pending = state.get("pending")
        if pending is None:
            raise ValueError("no pending marker; run precommit before record")
        if pending.get("package") != package_id:
            raise ValueError(
                f"pending package is {pending.get('package')!r}, not {package_id!r}"
            )
        if pending.get("kind", "integration") != (
            "correction" if correction else "integration"
        ):
            raise ValueError("pending commit kind does not match record mode")
        expected = state["expected_head"]
        head = absolute_head(root)
        if head == expected:
            raise ValueError(
                "no new commit exists; record after an interruption requires "
                "explicit reconciliation approval — complete the pending commit "
                "or discard the stale pending marker"
            )
        if os.fsdecode(git(root, "rev-parse", "HEAD^")).strip() != expected:
            raise ValueError(
                f"new commit parent is not the expected HEAD {expected}; "
                "interrupted or divergent history requires explicit reconciliation "
                "approval"
            )
        if (
            os.fsdecode(git(root, "rev-list", "--count", f"{expected}..HEAD")).strip()
            != "1"
        ):
            raise ValueError("expected exactly one new commit, found more")
        changed = sorted(
            os.fsdecode(path)
            for path in git(
                root,
                "diff",
                "--no-ext-diff",
                "--no-textconv",
                "--no-renames",
                "--name-only",
                "-z",
                expected,
                head,
                "--",
            ).split(b"\0")
            if path
        )
        if changed != sorted(set(pending.get("paths", []))):
            raise ValueError(
                f"committed paths {changed} do not exactly match the pending staged "
                f"paths {sorted(pending.get('paths', []))}"
            )
        if pending.get("tree") != os.fsdecode(
            git(root, "rev-parse", "HEAD^{tree}")
        ).strip():
            raise ValueError("committed tree differs from the reviewed staged tree")
        package = find_package(ledger, package_id)
        scope = set(package.write_scope)
        for path in changed:
            if ignored_authority_path(path):
                raise ValueError(f"committed path is ignored authority: {path}")
            if (
                not canonical_file_path(path)
                or path not in scope
                or coordinator_owned_path(path)
            ):
                raise ValueError(f"committed path outside exact Write scope: {path}")
            mode = tree_mode(root, head, path) or tree_mode(root, expected, path)
            if mode not in ("100644", "100755"):
                raise ValueError(f"non-regular committed path: {path}")
        before = set(pending.get("integrated_before", []))
        now = set(integrated_ids(ledger))
        if correction:
            if package_id not in before or now != before:
                raise ValueError(
                    "correction must preserve exactly the integrated package set"
                )
        elif package_id in before or now != before | {package_id}:
            raise ValueError(
                "ledger must record exactly the pending package's Planned->Integrated "
                "transition; interrupted ledger edits require explicit reconciliation "
                "approval"
            )
        if package.status != "Integrated":
            raise ValueError("package must be Integrated before advancing the journal")
        classified = inspect_ledger(ledger)
        if classified.get("state") == "invalid":
            ledger_errors = classified.get("errors", [])
            assert isinstance(ledger_errors, list)
            raise ValueError(
                "ledger is invalid: " + "; ".join(map(str, ledger_errors))
            )
        journal = list(state.get("journal", []))
        journal.append(
            {
                "kind": "correction" if correction else "integration",
                "package": package_id,
                "commit": head,
                "paths": changed,
            }
        )
        state["journal"] = journal
        if not correction:
            state["integrated"] = dict(state.get("integrated", {}))
            state["integrated"][package_id] = head
        state["expected_head"] = head
        state["pending"] = None
        backup_path = save_state(root, slug, state)
        return {
            "state": "ok",
            "package": package_id,
            "commit": head,
            "expected_head": head,
            "backup": backup_path,
        }
    except (OSError, UnicodeError, ValueError, RuntimeError) as error:
        errors.append(str(error))
        return {"state": "error", "errors": errors}


def record_boundary(
    ledger: Path, root: Path, pr_id: str, source_head: str,
    target_before: str, approved: bool = False,
) -> dict[str, Any]:
    """Advance accepted code HEAD after independently verified integration.

    Approval attests to separately verified exact-head merge conditions and
    provider/merge evidence. Local Git evidence alone cannot prove a hosted
    squash or rebase review event.
    """
    try:
        if not approved:
            raise ValueError("boundary recording requires verified merge approval")
        root, slug, _ = resolve_ledger(ledger, root)
        require_git_root(root)
        require_clean_tracked(root)
        state = read_state(root, slug)
        if state.get("pending") is not None:
            raise ValueError("pending package marker requires reconciliation")
        if not immutable_git_oid(source_head) or source_head != state["expected_head"]:
            raise ValueError("source head differs from accepted code HEAD")
        if not immutable_git_oid(target_before):
            raise ValueError("invalid approved target SHA")
        delivery = inspect_delivery(ledger, root)
        if delivery["state"] == "invalid" or delivery["plan_digest"] != state["plan_digest"]:
            raise ValueError("invalid or drifted plan authority")
        require_snapshots(work_dir(root, slug), state)
        if set(integrated_ids(ledger)) != set(state["integrated"]):
            raise ValueError("package progress differs from the accepted journal")
        pr = next((item for item in parse_ledger(ledger).pull_requests if item.pr_id == pr_id), None)
        if pr is None or removed(pr.status) or not immutable_git_oid(pr.merge_ref):
            raise ValueError("boundary has no retained immutable merge ref")
        review_ref = unwrap_markdown_autolink(pr.pr_ref)
        if pr.provider == "GitHub" and not (
            review_ref and GITHUB_PR_URL.fullmatch(review_ref)
        ):
            raise ValueError("GitHub boundary requires a valid review URL")
        if pr.provider != "Local" and pr.provider != "GitHub" and not (
            review_ref and manual_ref(review_ref)
        ):
            raise ValueError("manual-provider boundary requires a valid HTTPS review URL")
        if pr.provider == "Local" and (pr.merge_ref != source_head or review_ref != "Not applicable"):
            raise ValueError("Local boundary requires exact source-head fast-forward")
        if any(package.status == "Planned" for package in pr.packages):
            raise ValueError("boundary still has planned packages")
        previous = [
            item for item in parse_ledger(ledger).pull_requests
            if not removed(item.status)
        ]
        if pr not in previous or any(
            item.pr_id not in {entry.get("pr") for entry in state["journal"] if entry.get("kind") == "boundary"}
            for item in previous[:previous.index(pr)]
        ):
            raise ValueError("preceding boundary is not recorded")
        if any(entry.get("kind") == "boundary" and entry.get("pr") == pr_id for entry in state["journal"]):
            raise ValueError("boundary was already recorded")
        if os.fsdecode(git(root, "symbolic-ref", "--short", "HEAD")).strip() != pr.base_branch:
            raise ValueError("checkout is not the recorded base branch")
        if os.fsdecode(git(root, "rev-parse", f"refs/heads/{pr.branch}")).strip() != source_head:
            raise ValueError("source branch differs from accepted code HEAD")
        head = absolute_head(root)
        if head != pr.merge_ref:
            raise ValueError("base HEAD differs from ledger merge ref")
        git(root, "merge-base", "--is-ancestor", target_before, head)
        if pr.provider == "Local":
            git(root, "merge-base", "--is-ancestor", target_before, source_head)
        state["journal"].append({
            "kind": "boundary", "pr": pr_id, "commit": head,
            "source_head": source_head, "target_before": target_before,
        })
        state["expected_head"] = head
        backup_path = save_state(root, slug, state)
        return {
            "state": "ok", "pr": pr_id, "source_head": source_head,
            "expected_head": head, "backup": backup_path,
        }
    except (OSError, UnicodeError, ValueError, RuntimeError) as error:
        return {"state": "error", "errors": [str(error)]}


def backup(ledger: Path, root: Path) -> dict[str, Any]:
    """Back up the canonical wiki and accepted run state on demand."""
    try:
        root, slug, _ = resolve_ledger(ledger, root)
        read_state(root, slug)
        return {"state": "ok", "backup": str(write_backup(root, slug))}
    except (OSError, UnicodeError, ValueError, RuntimeError) as error:
        return {"state": "error", "errors": [str(error)]}


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Local delivery authority for an accepted ignored schema-4 plan. "
            "The accepted DESIGN and ledger stay outside Git under the ignored "
            ".wiki tree; .wiki/ and .work/ paths are never accepted in a package "
            "scope, staged set, or committed result. All commands fail closed; "
            "see the subcommand help for the interruption and reconciliation "
            "policy."
        )
    )
    sub = parser.add_subparsers(dest="command", required=True)
    common = argparse.ArgumentParser(add_help=False)
    common.add_argument("ledger")
    common.add_argument("root")

    accept_parser = sub.add_parser(
        "accept",
        parents=[common],
        help="accept an ignored schema-4 plan into .work/<slug>/ delivery state",
    )
    accept_parser.add_argument(
        "--approved",
        action="store_true",
        help="required developer approval for plan acceptance",
    )

    renew_parser = sub.add_parser(
        "renew",
        parents=[common],
        help="renew reviewed static authority without losing integrated progress",
    )
    renew_parser.add_argument(
        "--approved",
        action="store_true",
        help="required attestation that a fresh plan review cleared the revision",
    )

    sub.add_parser(
        "verify",
        parents=[common],
        help="fail-closed check: digest, snapshot hashes, expected HEAD, journal",
    )

    context_parser = sub.add_parser(
        "context",
        parents=[common],
        help="write the worker packet manifest for a Planned current-wave package",
    )
    context_parser.add_argument("package_id")

    worker_verify_parser = sub.add_parser(
        "worker-verify",
        help=(
            "isolated worker integrity check: assert the Git root and base HEAD, "
            "and verify the coordinator snapshot hashes; run before editing and "
            "again after editing, before any scratch commit"
        ),
    )
    worker_verify_parser.add_argument("manifest")
    worker_verify_parser.add_argument("worker_root")

    precommit_parser = sub.add_parser(
        "precommit",
        parents=[common],
        help="verify the exact staged package paths and save the pending marker",
    )
    precommit_parser.add_argument("package_id")
    precommit_parser.add_argument(
        "--correction",
        action="store_true",
        help="reviewed code correction to an already Integrated package",
    )

    record_parser = sub.add_parser(
        "record",
        parents=[common],
        help=(
            "advance the journal for one verified content commit; after an "
            "interruption this requires explicit reconciliation approval"
        ),
    )
    record_parser.add_argument("package_id")
    record_parser.add_argument(
        "--correction",
        action="store_true",
        help="record a reviewed code correction without changing package status",
    )
    boundary_parser = sub.add_parser(
        "record-boundary", parents=[common],
        help="advance the journal after independently verified boundary integration",
    )
    boundary_parser.add_argument("pr_id")
    boundary_parser.add_argument("source_head")
    boundary_parser.add_argument("target_before")
    boundary_parser.add_argument("--approved", action="store_true")
    sub.add_parser(
        "backup", parents=[common],
        help=(
            "copy .wiki/ and .work/<slug>/ to a new backup outside the repository; "
            "every command that saves state already does this"
        ),
    )
    return parser


def main(argv: list[str]) -> int:
    args = build_parser().parse_args(argv)
    if args.command == "worker-verify":
        result = worker_verify(Path(args.manifest), Path(args.worker_root))
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0 if result["state"] == "ok" else 2
    ledger = Path(args.ledger)
    root = Path(args.root)
    if args.command == "accept":
        result = accept(ledger, root, approved=args.approved)
    elif args.command == "renew":
        result = renew(ledger, root, approved=args.approved)
    elif args.command == "verify":
        result = verify(ledger, root)
    elif args.command == "context":
        result = context(ledger, root, args.package_id)
    elif args.command == "precommit":
        result = precommit(ledger, root, args.package_id, correction=args.correction)
    elif args.command == "record":
        result = record(ledger, root, args.package_id, correction=args.correction)
    elif args.command == "record-boundary":
        result = record_boundary(
            ledger, root, args.pr_id, args.source_head, args.target_before,
            approved=args.approved,
        )
    elif args.command == "backup":
        result = backup(ledger, root)
    else:
        raise SystemExit("unknown command")
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["state"] == "ok" else 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
