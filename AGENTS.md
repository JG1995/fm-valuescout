# Development Contract

This file contains durable repository rules. Use [.codex/README.md](.codex/README.md) for the human workflow and `.agents/skills/` for task procedures. Project facts belong in `.wiki/`. Repository commands, tests, and CI enforce the rules that need deterministic checks.

## Project scope

This is a hobbyist solo-developer project. Keep structural quality high, but do not add ceremony or defensive layers that cost more than the failure they prevent. Cover critical paths and data-loss risks. Prefer clear, direct code and documentation over premature abstraction.

## Read order

For broad, architectural, risky, or multi-commit work:

1. Read `.wiki/INDEX.md` and the relevant current-state documents.
2. Read the active feature ledger in `.wiki/features/active/`.
3. Inspect the relevant implementation and tests.

For ordinary work, inspect the relevant code and tests before changing them.

## Guidance layers

- `.wiki/CONCEPT.md` owns product purpose and boundaries.
- `.wiki/ARCHITECTURE.md` owns implemented architecture and operating constraints.
- `.wiki/features/active/` owns active multi-commit plans and discoveries.
- `.agents/skills/` owns reusable domain and workflow procedures.
- `.codex/agents/` owns specialist roles. The reviewer is read-only; the documentation steward may change only documentation.
- `.codex/config.toml` owns trusted project MCP configuration.

Temporary notes belong in `.work/`. They are not project truth. Do not document proposed behavior as implemented.

## Workflow and validation

Use the matching `workflow-*` skill for planning, building, checkpointing, fixing, reviewing, documentation reconciliation, feature completion, spikes, and security audits. The normal cycle is `workflow-plan-feature` → `workflow-build` → `workflow-checkpoint` → `workflow-fix` when review blocks → `workflow-finish-feature`.

For non-trivial behavior, work test-first: confirm a meaningful RED failure, make the smallest change GREEN, then refactor only while green. Do not weaken or remove tests to make a change pass. Every non-trivial staged change needs a separate read-only reviewer pass when the reviewer role is available.

Use only the stable command surface:

```bash
./scripts/dev test [target...]
./scripts/dev check
./scripts/dev format [paths...]
./scripts/dev secrets [--staged]
./scripts/dev smoke
./scripts/dev mutate <target...>
./scripts/dev bridge-install
```

`./scripts/dev check` is the commit gate. Run `./scripts/dev format` before staging. `mutate` is unsupported until tooling is wired; never report it as passed.

## Change decisions

Before adding code, use this order: remove unnecessary work; use the standard library; use the native platform; use an existing dependency; use one clear line; then write the minimum code that works. Do not add dependencies or abstractions for hypothetical use cases.

Ask the developer before unresolved decisions about persistence, schemas, migrations, authentication, concurrency, security, public APIs, or layer boundaries. Read `.wiki/ARCHITECTURE.md`, matching skills, and Recallium first. Use `workflow-spike` only when a runtime experiment can answer the question.

Always invest full rigor in input validation at trust boundaries, data-loss prevention, security controls, accessibility, hardware calibration, and explicit user requirements.

## Git

- Keep commits focused, atomic, and revertible.
- Stage exact files or hunks. Never use `git add .` or `git commit -a`.
- Before a commit, inspect status and the complete staged diff, run `git diff --cached --check`, the relevant validation, and an independent review.
- Wait for explicit developer approval before committing locally, except for the manual `workflow-build-loop` opt-in.
- Never push, amend, rebase, squash, or rewrite history without explicit approval.

## Recallium

Use the exact project name `fm-valuescout` for every Recallium operation. Search before non-obvious decisions or unfamiliar conventions. Save only durable context that is not already clear from the repository, wiki, or Git history. If Recallium is unavailable, continue with repository evidence and report the skipped lookup when it matters.

## Documentation

Follow `.wiki/INDEX.md` for ownership. Update durable documentation when externally meaningful behavior, commands, configuration, contracts, or persistent-data assumptions change. Multi-commit work has one active feature ledger. Reconcile and archive feature documentation at feature completion.

Do not let the documentation steward modify implementation, tests, schemas, executable scripts, CI workflows, Codex configuration, agent definitions, command templates, or Git state.
