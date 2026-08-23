---
name: create-pr
description: Prepare a human-authored pull request with this repository's template, duplicate check, required validation, and draft creation. Use when the user asks to create a pull request or prepare a branch for review.
---

# Create pull request

## Overview

Use this procedure for every human-authored pull request. Ordinary pull requests do not prepare, classify, or publish a release. Follow `AGENTS.md` for checkpoints and Git authority.

Use [`create-release`](../create-release/SKILL.md) only after a developer explicitly requests a release. It creates the sole release-preparation pull request for the complete unreleased range.

## Inspect the branch

1. Confirm the current branch is not `main`, and understand its worktree, index, upstream relationship, and configured base.
2. Inspect every commit in `main..HEAD`, the complete `main...HEAD` diff, and any existing pull request for the branch.
3. Do not pull, merge, rebase, force-push, or update refs without explicit approval.

## Keep release preparation out of ordinary pull requests

Do not edit release version owners, `CHANGELOG.md` dated sections, or `release-preparation.json` for an ordinary pull request. Do not run `release-metadata` as a pull-request classification step.

Feature delivery has no release fields. An explicit release later covers the complete range from the latest published tag through synchronized `main`.

## Validate and create the draft

1. Stage exact files, inspect the complete staged diff, run the required checks, obtain the required review, and create approved local commits.
2. Complete every section of `.github/pull_request_template.md`.
3. Run the existing duplicate check for the branch. Do not create a duplicate pull request.
4. Obtain separate explicit approval to push unless an explicitly invoked delivery workflow's unchanged Delivery fingerprint covers the exact branch, base, commits, and PR. Re-check the branch, clean worktree, unchanged `HEAD`, and `gh auth status`, then push without force.
5. Obtain separate explicit approval to create a draft pull request unless the same unchanged Delivery fingerprint covers that mutation. Create it against `main` with the completed repository template.

Return the draft URL, title, and factual validation results. Do not merge, tag, publish a GitHub release, bypass a stop condition, or enable auto-merge.
