---
name: create-release
description: Prepare, publish, and verify an explicit FM ValueScout release from the complete unreleased main range. Use only when the developer explicitly asks to create a release.
---

# Create release

## Overview

Releases are opt-in. Do not prepare a release from an ordinary pull request or feature ledger. Invoke this skill only after the developer explicitly asks for a release.

This procedure prepares one release pull request from `main`. Its merged `release-preparation.json` change starts the Release workflow. The workflow waits for the exact `main` Check, creates the tag and GitHub release, packages the Windows installer, verifies the assets, and publishes it. Do not create a tag or GitHub release manually.

## Inspect the unreleased range

1. Confirm the worktree is clean and local `main` equals `origin/main`. Do not discard unrelated local changes.
2. Find the latest published GitHub release and its `v<version>` tag. Confirm the tag resolves to an ancestor of `main`.
3. Inspect the complete range from that tag through `main` with `git log`, `git diff --stat`, the full diff as needed, merged pull requests, and the `Unreleased` section in `CHANGELOG.md`.
4. Confirm no existing release pull request, unpublished draft, or changed `release-preparation.json` already prepares the range. Stop and resolve that state before preparing another release.
5. If the range has no user-visible change, report that no release is needed and stop.

## Determine scope and propose the release

Choose the smallest SemVer scope that covers the complete range:

- `patch` for compatible fixes only.
- `minor` for one or more compatible capabilities, including a mix of capabilities and fixes.
- `major` for a breaking contract. Stop for a maintainer decision.

State the proposed scope, version, changelog categories, and changed files before editing. If compatibility or user visibility is unclear, ask the developer. The initial release uses `minor` and prepares `0.1.0`.

## Prepare the release pull request

1. Create `release/v<version>` from synchronized `main` after receiving the Git authority required by `AGENTS.md`.
2. Update the five version owners: `package.json`, `src-tauri/Cargo.toml`, the root `app` entry in `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, and `bridge/FmDataBridge.csproj`. Do not regenerate the lockfile. Confirm that only the root `app` lock entry changes.
3. Keep `## [Unreleased]`. Add one dated Keep a Changelog section immediately below it. The section must cover the complete range since the latest release tag.
4. Update `release-preparation.json` with the selected version and intent. Increment its positive `sequence`. This is the explicit automation trigger.
5. Run `./scripts/dev release-metadata <latest-tag-or-none> <patch|minor|major>` and `./scripts/dev check`.
6. Run native Windows validation only when the range changes packaging, the bridge, diagnostics, or release automation. For destructive, transformative, reinterpretive, or reassigning SQLite migrations, also run the installed-app upgrade smoke from a closed-app backup.
7. Independently review the release diff. Stage exact files, inspect the staged diff and stat, run `git diff --cached --check`, and obtain commit approval.
8. Create the release-preparation commit and use `.github/pull_request_template.md` to create the release PR. Use `chore(release): prepare v<version>` for the title. In Notes, state the intended tag and that the Release workflow creates it only after the release PR merges.

The release PR still follows normal push, review, and merge authority. Do not create the Git tag before the exact `main` Check passes.

## Publish and verify

After the release PR is squash-merged and local `main` fast-forwards to the exact merge commit:

1. Wait for the exact `main` Check. The Release workflow starts only because that merge changed `release-preparation.json`; it waits for that exact Check before it packages or writes a release.
2. Run release verification first. If the exact version is already published and valid, adopt it and do not rerun publication. Otherwise, wait for the matching Release workflow. Do not use a manual substitute.
3. Verify the tag points to synchronized `HEAD`, the release is published rather than draft or prerelease unless the version itself is a prerelease, the title and body match the dated changelog section, and exactly the installer plus its checksum sidecar exist. Verify the downloaded checksum matches GitHub's installer digest.
4. If the workflow fails before publication, correct the source through a new release PR. If it leaves a matching temporary draft, rerun only the exact failed workflow or repair that draft through the guarded workflow. Never retarget a draft or overwrite a published release.

## Emergency withdrawal

1. Stop further release preparation merges.
2. Withdraw or draft the GitHub release without deleting its tag or history first.
3. Publish a short notice that identifies the affected version and tells users not to install it.
4. Prepare a corrective release through this procedure. Create a private security advisory for a vulnerability and follow `SECURITY.md`.
