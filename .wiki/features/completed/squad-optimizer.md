# Squad Optimizer

## Intent

Fill the configured Planner depth charts with the best eligible players for the shared tactic without changing manual choices.

## Delivered behavior

- **Optimize squads** fills Senior, Reserves, and Youth in that order, then each team's strings in order. It uses an exact per-string matcher that maximizes combined IP/OOP score, then filled lanes, then a stable UID tie-break.
- A candidate must be in the team's configured club-family pool, meet the team's age rule, have IP suitability of at least 16 and OOP suitability of at least 12 for distinct lane positions, and have both role scores. A zero combined score remains eligible. Manual assignments reserve their player UID before optimization.
- Re-running Optimize preserves manual assignments, removes only earlier optimizer assignments, and recalculates from the current snapshot, tactic, and club-family sources in one Rust-owned database transaction.
- **Clear Squad** removes all manual and optimizer assignments from the selected Planner team only after destructive confirmation. It does not change the other teams.
- Both actions provide pending, success, and error feedback. On success, the frontend replaces its Planner depth data and invalidates slot-candidate data. The browser smoke suite exercises the Optimize control path through its IPC stub.

## Final architecture

```text
SQLite migration v7
  → planner_assignments.provenance: manual | optimizer

Rust features/planner
  → optimize_planner_depth transaction
  → exact per-string matching and ordered allocation
  → confirmed selected-team clearing
  → reconciled PlannerDepthDto

React Planner depth matrix
  → Optimize and Clear Squad controls
  → depth-cache replacement and slot-candidate invalidation
```

## Important decisions

- Existing assignment rows migrate to `manual`; manual assign and move mutations also write manual provenance.
- Rust owns eligibility, matching, priority, persistence, and returned Planner depth. React does not recompute optimization decisions.
- Clear Squad has selected-team scope. It removes both provenances in that team.
- No new dependency or ADR was required. The exact matcher is bounded by the eleven existing tactic lanes.

## Migration and operational implications

- Migration v7 adds `planner_assignments.provenance` with a `manual` default, so existing Planner assignments stay protected on the first Optimize run.
- Snapshot replacement does not remove Planner assignments. Optimize uses current snapshot data for new automatic assignments while preserving retained manual rows.
- Playwright smoke proves the stubbed IPC control path only. It does not prove Rust, SQLite, migrations, matching, or a native WebView.

## Validation

- `./scripts/dev format` made no changes.
- `./scripts/dev test` — 146 Vitest tests passed.
- `./scripts/dev check` — passed, including 198 Rust tests and 2 ignored tests.
- `./scripts/dev smoke` — 12 Chromium smoke tests passed.
- Mutation testing remains unsupported (`./scripts/dev mutate` exit 69).
- Native WebView viewport verification at 1280×800 and 1600×900 was unavailable.
- Feature-complete review: Sol High, Blocking **No**, with no retained findings.

**Delivery commits (final hashes):** `4cd14f0` (provenance), `3b15a08` (optimizer), `5947fb1` (selected-team Clear Squad), `0750551` (controls and smoke), `125466b` (source-scoped score loading correction), and `f6d6dc4` (Clear Squad confirmation error feedback correction). Comparison base: `df059a95d847940e004f9fd9e5dec52326fcef68`.

## Follow-up

- Deferred: formation comparison, best-and-worst candidate highlighting, gap recommendations, custom optimizer constraints, custom string names, string reordering, and tactic libraries.
