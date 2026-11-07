# Contributing

FM ValueScout uses a small, repository-owned development workflow. Keep changes coherent, validate through `./scripts/dev`, and preserve explicit Git authority.

## Develop from source

Follow [README — Develop from source](README.md#develop-from-source) for dependency and platform setup. Building or installing the FM27 BepInEx plugin is separate from the Linux gate; see [bridge/README.md](bridge/README.md).

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

## Use the jay-claude workflow

[AGENTS.md](AGENTS.md) owns standing repository rules. The globally installed jay-claude Claude Code plugin owns reusable workflow procedures and registers its roles as subagents. This repository does not copy or override them. Repository-owned agent resources are:

- `.claude/skills/create-release/SKILL.md`, which owns explicit release preparation and verification;
- `.claude/skills/ui-inspection/SKILL.md`, which owns populated Chromium captures for UI work; and
- `scripts/*.py`, jay-claude's project-owned ledger and delivery validators.

The maintainer's project wiki, `.wiki/`, is ignored by Git and exists only in the maintainer's checkout. jay-claude run state under `.work/` and worker worktrees under `.claude/worktrees/` are ignored too.

Workflow skills are explicit opt-ins; an ordinary natural-language request does not activate one.

### Feature delivery

1. Invoke `/jay-claude:design-feature <feature>` and accept the feature DESIGN.
2. Invoke `/jay-claude:plan <feature>` to create and independently review its schema 4 IMPLEMENTATION ledger, then accept it with `scripts/local_delivery.py accept`.
3. Invoke `/jay-claude:deliver-feature <ledger>` once to run the planned packages, PRs, merges, and close-out.

Delivery stops only when the feature's core functionality proves impractical or an operational blocker, such as a missing credential or failed check, needs the maintainer. Use `/create-release` only when the maintainer explicitly requests a release. The narrower jay-claude skills remain available for manual recovery; they are not required between normal delivery phases.

For a focused documentation edit or trivial fix, follow the applicable rules directly without creating a feature ledger.

## Commit and pull-request conventions

Every commit and pull-request title follows [Conventional Commits 1.0.0](https://www.conventionalcommits.org/):

- use `type(scope): imperative description`;
- keep the subject under 72 characters with no trailing period;
- keep one coherent, revertible outcome per commit; and
- do not amend, rebase, force-push, or otherwise rewrite history.

Stage exact paths or hunks. Never use `git add .` or `git commit -a`. Inspect the complete staged diff and run `git diff --cached --check` before requesting commit approval.

Every pull request completes [.github/pull_request_template.md](.github/pull_request_template.md) and is squash-merged. Ordinary PRs do not prepare a release. Use [.claude/skills/create-release/SKILL.md](.claude/skills/create-release/SKILL.md) only when the maintainer explicitly requests one.

Do not commit, push, create or update a PR, merge, synchronize a remote, or create a release without the explicit authority described in `AGENTS.md`.

## Escalate before assuming

Ask for developer input before choosing persistence, schema, migrations, authentication, concurrency, public APIs, security controls, or a safety-critical boundary. First inspect `.wiki/ARCHITECTURE.md`, the relevant feature DESIGN and IMPLEMENTATION documents, ADRs, debug reports, implementation, tests, and focused Git history.
