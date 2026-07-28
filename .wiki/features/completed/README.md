# Completed Feature Records

This directory contains condensed records of completed feature behavior, architecture, important decisions, and validation. It is not an unedited development diary or a duplicate of current architecture.

At feature completion, run `/finish-feature` (or `/docs-review` for standalone milestones), remove temporary implementation detail, and move the active ledger here. Update [`ARCHITECTURE.md`](../../ARCHITECTURE.md) only for implemented current-state changes. Update [`TODO.md`](../../TODO.md) only at feature level.

## Completion record template

```markdown
# <Feature Name>

## Intent

Why the feature was introduced.

## Delivered behavior

- ...

## Final architecture

- ...

## Important decisions

- [ADR <number> — <title>](../../decisions/<file>.md), if applicable

## Migration and operational implications

- ...

## Validation

- ...

## Follow-up

- ...
```
