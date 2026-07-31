# Project Wiki

This directory contains durable, version-controlled knowledge about this repository. Temporary investigation notes belong in `.work/` and are not project truth.

## Document map

- [Product concept](CONCEPT.md) owns the product's purpose, intended users, principles, and boundaries. It does not own implementation plans or current architecture.
- [Architecture](ARCHITECTURE.md) owns the currently implemented repository structure, data flows, and operational constraints. It does not describe proposals as current state.
- [Design system](DESIGN.md) owns the visual language, component tokens, and UI design decisions. It does not describe user research or product strategy.
- [Planned work](TODO.md) owns features and initiatives with committed or imminent delivery intent. It does not track aspirational or unscheduled ideas.
- [Backlog](BACKLOG.md) owns aspirational, deferred, or unscheduled work — ideas worth remembering but not planned for the near term. It does not track committed delivery work.
- [Decision records](decisions/README.md) own consequential decisions and their alternatives. They are not a log of routine implementation choices.
- [Active features](features/active/README.md) own the intent, delivery plan (PRs and commits), and material discoveries for multi-commit work.
- [Planned feature specs](features/planned/README.md) own pre-implementation behavioral detail for MVP features not yet in active development.
- [Completed features](features/completed/README.md) own condensed records of finished feature behavior and implications. They are not development diaries.

## Documentation lifecycle

1. **New project bootstrap** — fill `CONCEPT.md`, use `workflow-roadmap` when MVP direction is set (`workflow-stack` only when changing defaults); approve wiki updates before building.
2. **Per-feature planning** — create one active feature ledger with `workflow-plan-feature` and mark exactly one commit `Active`.
3. **Per commit** — update documentation intrinsic to the atomic outcome during `workflow-build` and `workflow-checkpoint`.
4. **Feature completion** — use `workflow-finish-feature` (tests, feature-complete review, then documentation reconciliation).
5. **Archive** — condense and move a completed ledger to `features/completed/`.
6. **Cleanup** — remove disposable notes from `.work/`.

Update durable documentation only when an externally meaningful behavior, command, configuration, contract, or persistent-data assumption changes.

## Source of truth

When documentation and repository evidence disagree, implementation and passing tests describe executable behavior. Documentation must be reconciled. Accepted ADRs explain consequential choices. Active feature plans are proposals and progress records, not guarantees of future implementation.
