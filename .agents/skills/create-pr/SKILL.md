---
name: create-pr
description: Prepare a human-authored pull request with this repository's template, explicit release intent, version and changelog validation, duplicate check, and draft creation. Use when the user asks to create a pull request, prepare a branch for review, or says to create a PR using the repository template.
---

# Create pull request

## Overview

Use one procedure for every human-authored pull request. Release intent is one template field, not a different pull-request type. Follow `AGENTS.md` for checkpoint and Git authority.

## Inspect the branch

1. Confirm the current branch is not `main`, the worktree/index state is understood, and the upstream relationship is known.
2. Inspect every commit in `main..HEAD`, the complete `main...HEAD` diff, the configured base, and any existing pull request for the branch.
3. Do not pull, merge, rebase, force-push, or update refs without explicit approval.

## Choose one release intent

Inspect the latest reachable `v*` release tag and the complete user-visible change set since that boundary, including relevant changes already on `main` and the complete pull-request diff. Record exactly one template checkbox:

- `none` for documentation, tests, agent configuration, internal tooling, or an eligible Dependabot-only patch. Do not change durable version owners or create a dated changelog section.
- `patch` for an unambiguous compatible user-visible fix. Increment the patch number (for example, `0.2.0` → `0.2.1`). If the latest published tag is the historical `0.1.0-alpha.1`, the patch release normalizes it to `0.1.0`.
- `minor` for an unambiguous compatible capability. Increment the minor number and reset the patch number (for example, `0.1.0` → `0.2.0`). With no previous tag, use `minor` to prepare the initial `0.1.0` release.
- `major` for a breaking behavior or durable contract. Stop and request a maintainer decision.

New releases are published as normal SemVer releases. The repository's historical `0.1.0-alpha.1` remains immutable for its original release, but future compatible fixes and capabilities use plain `0.x.y` versions rather than public `alpha.N` iterations or release candidates.

Do not decide intent from one Conventional Commit title or file path. Stop and request a decision when compatibility is ambiguous.

## Prepare release metadata

For `patch` or `minor`, propose the version, date, complete dated changelog section, and exact files before editing. Obtain any checkpoint approval required by `AGENTS.md`. Then update all five durable version owners:

- `package.json`
- `src-tauri/Cargo.toml`
- root `app` entry in `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`
- `bridge/FmDataBridge.csproj`

Preserve `## [Unreleased]`. Add exactly one dated Keep a Changelog section that covers the complete unreleased user-visible range. Do not regenerate the Cargo lockfile. Confirm only the root `app` lock entry changes.

For a release-bearing intent, update `release-preparation.json` with the matching version and intent, then increment its positive `sequence`. The verified-main workflow treats that changed record as the explicit authorization for this exact SHA and publishes a normal GitHub release. For `none`, leave the record unchanged.

Run:

```bash
./scripts/dev release-metadata <latest-tag-or-none> <none|patch|minor|major>
./scripts/dev check
```

The metadata command only reads local files. It returns machine-readable version, tag, release-required state, and the exact dated changelog section. It does not write files, invoke Git, or call GitHub.

Pass both arguments whenever you validate an intent. A single `none` means no previous tag; it is not the `none` intent.

For `none`, validate against the latest tag when one exists. Leave version owners and dated sections unchanged.

## Validate and create the draft

1. Stage exact files, inspect the complete staged diff, run the required checks, obtain the required review, and create any approved local release-preparation commit.
2. Complete every section of `.github/pull_request_template.md`, including exactly one release-intent checkbox.
3. Run the existing duplicate check for the branch. Do not create a duplicate pull request.
4. Obtain separate explicit approval to push. Re-check the branch, clean worktree, unchanged `HEAD`, and `gh auth status`, then push without force.
5. Obtain separate explicit approval to create a draft pull request. Create it against `main` with the completed repository template.

Return the draft URL, title, verified release intent, exact version/tag when release-bearing, and factual validation results. Do not merge, tag, publish a GitHub release, bypass a stop condition, or enable auto-merge.
