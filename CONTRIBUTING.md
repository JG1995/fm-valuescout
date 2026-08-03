# Contributing

This template is designed to be forked, modified, and shipped. The workflow keeps the history useful after a break without adding unnecessary ceremony.

## Run the gate before every commit

```bash
./scripts/dev format
./scripts/dev check-fast   # Pre-commit runs this (+ check-rust for staged src-tauri/ files)
./scripts/dev check        # Full local code-quality gate
```

`format` applies Biome fixes and `cargo fmt` before you stage. `check-fast` runs full-tree Biome and TypeScript plus staged secretlint. `check` runs code-quality checks: Biome, TypeScript, secretlint, and Rust.

CI selects frontend, browser, Rust, and bridge checks from the changed paths. Its required `check` status aggregates every applicable result. Desktop installer builds run only from the release workflow. Do not weaken tests to make a gate pass. Run `pnpm exec playwright install chromium` once after `pnpm install` so smoke can run.

## Pre-commit hook

Husky runs `./scripts/dev check-fast`, plus `./scripts/dev check-rust` when staged files include `src-tauri/`. It does not replace the full gate.

Husky installs on `pnpm install`. The repository uses Biome for lint and format and secretlint for secret scanning. To bypass the hook for one commit, use `git commit --no-verify`. Do not disable hooks globally.

## Follow the commit convention

Every commit message uses [Conventional Commits 1.0.0](https://www.conventionalcommits.org/). Load the installed `conventional-commits` skill before writing one.

- Use imperative present tense: “Add,” not “Added.”
- Keep the subject under 72 characters.
- Keep each commit to one coherent outcome.
- Use the same `type(scope): description` form for PR titles.
- Do not amend, rebase, squash, or rewrite history without explicit approval.

## Use the Codex workflow for non-trivial work

The workflow overview is in [.codex/README.md](.codex/README.md), and the canonical repository lifecycle and model-routing policy is in [AGENTS.md](AGENTS.md). Development is trunk-based: short-lived branches merge to `main` frequently, and every commit is atomic.

### New project bootstrap

See [README — Forking this template](README.md#forking-this-template) for setup details. In brief:

1. Run `./scripts/dev check` and `./scripts/dev test` before feature work.
2. Fill `.wiki/CONCEPT.md` with MVP scope.
3. Use `workflow-stack` only when you change the default stack.
4. Use `workflow-roadmap` to approve the development sequence in `TODO.md`.
5. Use `workflow-plan-feature` for the feature named in **Plan next**.

Building the FM26 BepInEx plugin is separate from the Linux gate. See [bridge/README.md](bridge/README.md).

### Per-feature loop

1. `workflow-plan-feature` — create a delivery plan with atomic commits, implementation packets, and separate implementation/review profiles. Trivial changes skip the ledger.
2. `workflow-build` — use the active commit's assigned profile and packet, write a meaningful failing test, make the smallest passing change, and refactor while green.
3. `workflow-checkpoint` — stage exact files, run the gate, dispatch the assigned reviewer in a fresh context, and present evidence-backed findings.
4. `workflow-fix` — address blocking review findings, then checkpoint again.
5. Approve the local commit and reassess the remaining delivery plan.
6. `workflow-finish-feature` — run full validation, feature review, then documentation reconciliation.

`workflow-build-loop` is manual opt-in. It can automate checkpoint and fix rounds, then commit when only NITPICK findings remain. Use `workflow-spike` only when a runtime experiment is necessary. Use `workflow-security-audit` before deployment or after sensitive changes.

For a single-line fix or documentation edit, follow the applicable workflow internally without naming every skill.

## Escalate before assuming

Ask for developer input before choosing persistence, schema, migrations, authentication, concurrency, public APIs, security controls, or a safety-critical boundary. Read `.wiki/ARCHITECTURE.md`, load matching global skills, and search Recallium first. Use `workflow-spike` only when a runtime experiment can answer the question.

## Merge template updates into a fork

This template does not automate cross-fork updates. Add the template as a remote and merge selectively:

```bash
git remote add template <template-repo-url>
git fetch template
git merge template/main --allow-unrelated-histories
```

Resolve conflicts manually. Review changes to `.codex/`, `scripts/`, and `AGENTS.md` before you merge.
