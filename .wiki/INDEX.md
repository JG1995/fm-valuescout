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
- [Completed features](features/completed/README.md) preserve historical completion records and complete schema 2 ledgers for newly delivered or explicitly abandoned outcomes. Do not rewrite legacy records only to match the current schema.
- [Wiki notes](notes/README.md) own durable runbooks and cross-project procedures that do not fit another wiki owner.

## Documentation lifecycle

1. **New project bootstrap** — fill `CONCEPT.md`, then explicitly invoke `/skill:workflow-stack` and `/skill:workflow-roadmap` when MVP direction is set.
2. **Per-feature planning** — explicitly invoke `/skill:workflow-plan-feature`, move accepted planned-spec detail into one schema 2 active ledger, obtain independent plan review, and accept its Delivery fingerprint.
3. **Delivery** — explicitly invoke `/skill:workflow-deliver-feature <ledger>` for the normal end-to-end path. It implements, validates, reviews, commits, publishes, merges, synchronizes, and closes out the fingerprinted work. Use the repository-local `create-release` skill only when the maintainer explicitly requests a release. Use narrower workflow skills only for manual recovery.
4. **Per commit** — update only documentation intrinsic to each atomic outcome. Add an ADR or debug report only when its documented threshold is met.
5. **Feature completion** — before the final merge, run full validation, bounded feature review and correction, documentation reconciliation, and the reviewed close-out commit.
6. **Archive** — move the complete schema 2 ledger to `features/completed/` without removing Delivery fingerprint inputs. Preserve records from earlier workflows in their historical format.
7. **Abandonment** — after explicit developer approval, validate the abandoned ledger state, preserve completed evidence, archive without publication, and require a fresh plan before later work.
8. **Cleanup** — remove disposable notes, raw logs, failed hypotheses, and experiment artifacts from `.work/`.

Update durable documentation only when an externally meaningful behavior, command, configuration, contract, or persistent-data assumption changes.

## Source of truth

When documentation and repository evidence disagree, implementation and passing tests describe executable behavior. Documentation must be reconciled. Accepted ADRs explain consequential choices. Debug reports explain confirmed historical failure patterns and may need superseding when the implementation changes. Active feature plans are proposals and progress records, not guarantees of future implementation.
