# Contributing

This template is designed to be forked, modified, and shipped from. Most of the
ceremony exists so that one person (you) can return to the project after a
break and still trust the commit history. Follow the rules below when you work
on a project derived from this template.

## Run the gate before every commit

```bash
./scripts/dev format
./scripts/dev check-fast   # pre-commit runs this (+ check-rust when src-tauri/ is staged)
./scripts/dev check        # full gate — same surface as CI
```

`format` applies Biome lint and format fixes, then `cargo fmt` in `src-tauri/`, before you stage. `check-fast` runs full-tree Biome and TypeScript, plus staged secretlint only. `check` is the full gate: contract tests, Playwright smoke contract, full-tree secretlint, and Rust.

CI runs the full `./scripts/dev check`, then `./scripts/dev test`, then `pnpm build`.
A failing gate must return non-zero; do not weaken tests to make a change pass.

Run `pnpm exec playwright install chromium` once after `pnpm install` so the smoke contract inside `./scripts/dev check` can pass.

## Pre-commit hook

The Husky pre-commit hook runs a **fast** local gate:

```bash
./scripts/dev check-fast
# plus ./scripts/dev check-rust when staged files include src-tauri/
```

`check-fast` runs full-tree Biome and TypeScript, and secretlint on **staged** files only. It does not run contract tests, Playwright smoke, full-tree secretlint, or Rust unless `src-tauri/` is in the staged diff.

The **full** gate — `./scripts/dev check` — runs in CI and before you merge. It includes dispatcher contract tests (Vitest file pattern, Playwright smoke, mutate unsupported status) via `test-dev.sh`, but not the full Vitest suite (run `./scripts/dev test` locally or rely on CI for the full suite).

**Husky** installs the hook on `pnpm install` — no manual Git config step. We use **Biome only** for lint and format (no ESLint, Prettier, or lint-staged). Secret scanning at commit time uses **secretlint --staged** via `check-fast`; the full gate scans the whole tree. See [ADR-0009](.wiki/decisions/0009-biome.md), [ADR-0011](.wiki/decisions/0011-husky-git-hooks.md), and [ADR-0012](.wiki/decisions/0012-secretlint.md).

To bypass the hook for a single commit, use `git commit --no-verify`. Do not disable hooks globally.

## Follow the commit convention

Every commit message uses [Conventional Commits
1.0.0](https://www.conventionalcommits.org/). Read
[`.cursor/skills/conventional-commits/SKILL.md`](.cursor/skills/conventional-commits/SKILL.md)
before writing one.

Quick rules:

- Imperative mood, present tense. "Add", not "Added" or "Adds".
- Subject under 72 characters. Body and footers are optional.
- One coherent outcome per commit. If the subject needs "and", split the commit.
- PR titles use the same `type(scope): description` shape as commit subjects.
- Mark breaking changes with `!` after the type/scope or a `BREAKING CHANGE:`
  footer.
- Do not amend, rebase, squash, or rewrite history without explicit
  approval. The history is the audit trail.

## Use the workflow loop for non-trivial work

The workflow loop lives in [`.cursor/README.md`](.cursor/README.md). Development is **trunk-based** — short-lived PR branches merge to `main` frequently; each commit is atomic and messages follow Conventional Commits.

### New project bootstrap

See [README — Forking this template](README.md#forking-this-template) for prerequisites, the rename table, Playwright setup, and editor extensions. Summary:

1. Green gate (`./scripts/dev check`, `./scripts/dev test`) before feature work.
2. In `AGENTS.md` § Recallium project, replace `[REPLACE_WITH_RECALLIUM_PROJECT_NAME]` when you use Recallium.
3. Fill `.wiki/CONCEPT.md` (especially MVP scope).
4. `/stack` — only when you change the default stack; skip when you keep template defaults.
5. `/roadmap` — approve development sequence in `TODO.md` (CONCEPT bullets alone suffice for a provisional sequence).
6. `/plan-feature` on the feature named in **Plan next**.

Building the FM26 BepInEx plugin (Windows host, .NET 6) is separate from the Linux gate — see [bridge/README.md](bridge/README.md).

### Per-feature loop

1. `/plan-feature` — plan one feature (PRs and commits). Trivial changes skip the ledger.
2. `/build` — write a failing test, implement the smallest passing change, refactor while green (default: one active commit, then stop).
3. `/checkpoint` — stage exact files, run the gate, dispatch the reviewer, present evidence.
4. `/fix` — when review blocks, address delegated findings (default: CRITICAL, HIGH, and MEDIUM), then checkpoint again.
5. Approve the staged commit.
6. Reassess remaining commits in the delivery plan.
7. `/finish-feature` — when the delivery plan is complete: full tests, feature-complete review, then documentation reconciliation when review clears.

**Optional:** `/build-loop` — manual opt-in only; automates build, checkpoint, and fix (up to five fix rounds) and commits when only NITPICK findings remain. Mixed verdicts fix NITPICK alongside CRITICAL/HIGH/MEDIUM. See `.cursor/commands/build-loop.md`.

Optional (not every feature): `/spike` when a runtime experiment is the only way to unblock planning or build; `/security-audit` before first deploy or after auth, payments, or sensitive data features.

For a single-line fix or a doc edit, follow the loop internally without
invoking each command.

## Escalate before assuming

These decisions need explicit developer input before implementation:

- Persistence, schema, or migration shape
- Authentication or authorisation
- Concurrency model
- Public API surface
- Security controls
- Anything that touches a safety carve-out (input validation at trust
  boundaries, data-loss prevention, accessibility)

The agent will not guess on these. Read `.wiki/ARCHITECTURE.md`, scan matching
skills in `.cursor/skills/`, and search Recallium. If still blocked, ask the
developer — or use optional `/spike` when only a runtime experiment can answer
the question.

## Merge template updates into a fork

This template does not solve the cross-fork update problem. To pull changes
back in, add the template as a remote and merge selectively:

```bash
git remote add template <template-repo-url>
git fetch template
git merge template/main --allow-unrelated-histories
```

Resolve conflicts manually. Watch for changes to `.cursor/`, `scripts/`,
and `AGENTS.md` — those are the most likely to need attention. Do not auto-merge
without reviewing the diff.
