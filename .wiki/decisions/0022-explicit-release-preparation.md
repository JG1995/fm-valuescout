# 0022 — Explicit Release Preparation

## Status

Accepted

## Context

The former pull-request procedure required every pull request to classify a release intent. A release-bearing PR changed version metadata and triggered Windows package validation in Check. The verified-main Release workflow also started after every successful `main` Check.

That coupled publication timing to feature PRs. It also used GitHub Actions capacity for release-only checks when the maintainer did not intend to release.

## Decision

Ordinary pull requests do not classify or prepare releases. The repository-local `create-pr` skill excludes release metadata from normal PR work.

A developer explicitly invokes `create-release` to inspect the complete range from the latest published tag through `main`, choose the SemVer scope, update the version owners and dated changelog section, increment `release-preparation.json`, and create one release PR.

The Release workflow starts only when that explicit marker reaches `main`. It waits for the exact `main` Check, then packages, stages, verifies, and publishes the matching release. Check no longer runs Windows package validation or release metadata validation.

## Alternatives considered

### Keep release intent on every pull request

This lets a feature PR publish immediately, but it couples release timing to implementation delivery and runs release-only validation too often. Rejected.

### Manually dispatch publication after a release PR merges

This avoids release workflow starts on ordinary pushes, but it needs a second manual operation and can detach the publication source from the reviewed release PR. Rejected.

### Publish on every main push

This keeps the former automation shape but does not meet the opt-in release requirement. Rejected.

## Consequences

### Positive

- Ordinary pull requests do not consume release-only Windows validation or Release workflow capacity.
- A release covers the complete reviewed `main` range since the latest published tag.
- The checked release source remains the only source that can create its tag and GitHub release.

### Negative

- Maintainers must explicitly request a release after merging user-visible work.
- A release PR adds a separate review and merge step.

### Follow-up

- Use `create-release` for the next release and update it only when the release contract changes.
- Keep `release-preparation.json` as the sole release workflow trigger.

## Related work

- Skill: [create-release](../../.pi/skills/create-release/SKILL.md)
- Workflow: [Release](../../.github/workflows/release.yml)
- Supersedes: the release-intent procedure in the removed early-alpha release runbook.
