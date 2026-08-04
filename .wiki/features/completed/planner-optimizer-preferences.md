# Planner Optimizer Preferences

## Intent

Let users tune optimizer behavior for each linked tactic lane while preserving manual assignments as absolute overrides.

## Delivered behavior

- Each save-scoped tactic lane owns its IP/OOP score weight, optional unique importance rank, and preferred-foot rule.
- The optimizer processes ranked lanes in ascending rank order within the existing team and string order. It then uses the exact matcher for remaining unranked lanes.
- Each lane accepts Either, Left, Right, or Both. Preferred mismatches subtract five points from the optimizer-only allocation score. Strict mismatches are ineligible for that lane.
- Two-footed snapshot players satisfy Left, Right, and Both. Foot rules do not change persisted role scores, displayed combined scores, or manual assignments.

## Final architecture

- Migration v8 replaces unreleased tactic rows with lane-owned IP/OOP weights while keeping Planner assignments. Migration v9 adds nullable unique importance ranks. Migration v10 adds preferred foot and Preferred or Strict mode.
- Rust validates, persists, and allocates tactic preferences. It reserves manual assignments, applies ranked allocation, and uses foot rules only for optimizer allocation.
- React keeps editable selected-lane drafts and renders the accessible Planner controls. It sends the complete tactic through the existing Planner IPC contract and reconciles Planner queries after saves.

## Important decisions

- The unrestricted tactic value is `any`; snapshot `either` means a genuinely two-footed player.
- The fixed five-point Preferred mismatch penalty has a zero floor. It is not configurable.
- No ADR was needed. The behavior remains inside the existing Planner boundary.

## Migration and operational implications

- The v8 tactic reset was accepted because the product is unreleased. It preserves saves, club-family settings, strings, and assignments through stable lane IDs.
- Native populated-state validation at 1280×800 and 1600×900 was unavailable because UI-agent tooling was removed in `c1e264b`. The developer accepted this evidence gap. Browser smoke covers the Planner IPC stub, not the native Tauri/SQLite boundary.

## Validation

- `./scripts/dev format` made no unintended changes.
- `./scripts/dev test` passed 151 frontend tests.
- `./scripts/dev check` passed, including 211 Rust tests and 2 ignored tests.
- `./scripts/dev smoke` passed 12 of 12 tests.
- `./scripts/dev mutate` remains unsupported.
- Feature-complete review: Sol High reported Blocking **No** with no CRITICAL, HIGH, or MEDIUM findings. The only NITPICK required this documentation reconciliation.

**Delivery commits (final hashes):** `49b8d2854a2f67c65278abedc18ad505f5dfa987`, `7f1114dbb765e9fac66adf57fefefc2a576cd0cc`, `c321f8a14c7e2dde66e3ca967f9cf9d6ec91e9cb` (planning record `f1999ed3e9caf866269f9d810888143f8f3cea3e`).

## Follow-up

- Select future work from [TODO.md](../../TODO.md) and [BACKLOG.md](../../BACKLOG.md) when ready.
