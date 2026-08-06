# Project Wiki

This directory contains durable, version-controlled knowledge about this repository. Temporary investigation notes belong in `.work/` and are not project truth.

## Document map

- [Product concept](CONCEPT.md) owns the product's purpose, intended users, principles, and boundaries. It does not own implementation plans or current architecture.
- [Architecture](ARCHITECTURE.md) owns the currently implemented repository structure, data flows, and operational constraints. It does not describe proposals as current state.
- [Design system](DESIGN.md) owns the visual language, component tokens, and UI design decisions. It does not describe user research or product strategy.
- [Planned work](TODO.md) owns features and initiatives with committed or imminent delivery intent. It does not track aspirational or unscheduled ideas.
- [Backlog](BACKLOG.md) owns aspirational, deferred, or unscheduled work — ideas worth remembering but not planned for the near term. It does not track committed delivery work.
- [Decision records](decisions/README.md) own consequential decisions and their alternatives. They are not a log of routine implementation choices.
- [Debug reports](debugging/README.md) own confirmed, reusable failure patterns and diagnostic procedures that code and regression tests do not explain well enough. They are not a bug log.
- [Active features](features/active/README.md) own the intent, delivery plan (PRs and commits), and material discoveries for multi-commit work.
- [Planned feature specs](features/planned/README.md) own pre-implementation behavioral detail for MVP features not yet in active development.
- [Completed features](features/completed/README.md) own condensed records of finished feature behavior and implications. They are not development diaries.
- [Wiki notes](notes/README.md) own durable runbooks and cross-project procedures that do not fit another wiki owner.

## Documentation lifecycle

1. **New project bootstrap** — fill `CONCEPT.md`, use `$workflow-roadmap` when MVP direction is set (`$workflow-stack` only when changing defaults), and approve wiki updates before building.
2. **Per-feature planning** — create one active feature ledger with `$workflow-plan-feature` and mark exactly one commit `Active`.
3. **Per commit** — update documentation intrinsic to the atomic outcome during `$workflow-build` and `$workflow-checkpoint`. Add an ADR or debug report only when its documented threshold is met.
4. **Feature completion** — use `$workflow-finish-feature`, or manually opt into `$workflow-finish-feature-loop`, for tests, the ledger-selected feature-complete review (Sol High for a legacy ledger), and documentation reconciliation.
5. **Archive** — condense and move a completed ledger to `features/completed/`.
6. **Cleanup** — remove disposable notes, raw logs, failed hypotheses, and experiment artifacts from `.work/`.

Update durable documentation only when an externally meaningful behavior, command, configuration, contract, or persistent-data assumption changes.

## Source of truth

When documentation and repository evidence disagree, implementation and passing tests describe executable behavior. Documentation must be reconciled. Accepted ADRs explain consequential choices. Debug reports explain confirmed historical failure patterns and may need superseding when the implementation changes. Active feature plans are proposals and progress records, not guarantees of future implementation.
