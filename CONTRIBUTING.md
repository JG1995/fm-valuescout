# Contributing

FM ValueScout uses a small, repository-owned development workflow. Keep changes coherent, validate through `./scripts/dev`, and preserve explicit Git authority.

## Develop from source

Follow [README — Develop from source](README.md#develop-from-source) for dependency and platform setup. Building or installing the FM26 BepInEx plugin is separate from the Linux gate; see [bridge/README.md](bridge/README.md).

Run formatting before staging and the full gate before every commit:

```bash
./scripts/dev format
./scripts/dev check-fast
./scripts/dev check
```

`format` applies Biome fixes and `cargo fmt`. `check-fast` is the Husky pre-commit path: full-tree Biome and TypeScript, staged secretlint, and conditional Rust validation when `src-tauri/` changes are staged. `check` is the full local gate. It selects Biome, TypeScript, secretlint, Rust format, Clippy, and tests through one stable command.

CI selects frontend, browser, Rust, bridge, and CI jobs from changed paths. The required `check` does not validate release metadata or package a Windows installer. Only an explicit release-preparation change on `main` starts the Release workflow, which waits for that exact `check` before it publishes.

Install Chromium once after `pnpm install`:

```bash
pnpm exec playwright install chromium
```

Then use `./scripts/dev smoke` for the Playwright product suite.

## Use the Pi workflow

[AGENTS.md](AGENTS.md) owns standing repository rules. Installed PI_SETUP skills own reusable workflow procedures and launch globally installed roles as direct subagents. Repository-owned Pi resources are tracked under `.pi/`:

- `.pi/settings.json` intentionally leaves machine package and preference settings global;
- `.pi/skills/create-pr/SKILL.md` owns this repository's ordinary pull-request preparation procedure; and
- `.pi/skills/create-release/SKILL.md` owns explicit release preparation and verification.

PI_SETUP installs its role definitions globally. This repository does not override those roles or configure a project orchestration runtime. Generated Pi package data under `.pi/npm/` and `.pi/git/` is ignored. `.pi/sessions/` is also ignored if a maintainer later opts into project-local session storage; by default, session history remains in Pi's global user directory.

Pi loads trusted project resources at startup. Run `/reload` after pulling or editing project context, settings, extensions, skills, prompts, themes, or PI_SETUP itself.

Workflow skills are explicit opt-ins; an ordinary natural-language request does not activate one.

### Feature delivery

1. Invoke `/skill:workflow-plan-feature` to create and independently review one schema 2 active ledger.
2. Accept its Delivery fingerprint.
3. Invoke `/skill:workflow-deliver-feature <ledger>` once to execute the recorded commits, PRs, and close-out.

The delivery workflow stops when authority changes, a decision or replan is required, validation fails, correction limits are exhausted, or a PR head is stale. Use `/skill:create-release` only when the maintainer explicitly requests a release. Narrower workflow skills remain available for manual recovery; they are not required between normal delivery phases.

For a focused documentation edit or trivial fix, follow the applicable rules directly without creating a feature ledger.

## Commit and pull-request conventions

Every commit and pull-request title follows [Conventional Commits 1.0.0](https://www.conventionalcommits.org/):

- use `type(scope): imperative description`;
- keep the subject under 72 characters with no trailing period;
- keep one coherent, revertible outcome per commit; and
- do not amend, rebase, force-push, or otherwise rewrite history.

Stage exact paths or hunks. Never use `git add .` or `git commit -a`. Inspect the complete staged diff and run `git diff --cached --check` before requesting commit approval.

Every human-authored pull request uses [.pi/skills/create-pr/SKILL.md](.pi/skills/create-pr/SKILL.md) and [.github/pull_request_template.md](.github/pull_request_template.md). Ordinary PRs do not prepare a release. Use [.pi/skills/create-release/SKILL.md](.pi/skills/create-release/SKILL.md) only when the maintainer explicitly requests one.

Do not commit, push, create or update a PR, merge, synchronize a remote, or create a release without the explicit authority described in `AGENTS.md`.

## Escalate before assuming

Ask for developer input before choosing persistence, schema, migrations, authentication, concurrency, public APIs, security controls, or a safety-critical boundary. First inspect `.wiki/ARCHITECTURE.md`, the active feature ledger, relevant completed records, ADRs, debug reports, implementation, tests, and focused Git history.
