# Squad Planner

## Intent

Let the user configure one FM26 tactic and manually organize a selected club family into Senior, Reserves, and Youth depth charts. The Planner uses the current snapshot's role scores while preserving the user's planning state across snapshot refreshes.

## Delivered behavior

- `/planner` provides club-family setup, a shared dual-phase tactic editor, and a three-team depth matrix for the active app save.
- The user selects a primary club and can attach explicit separate B-team or youth-club sources to Reserves or Youth. Every player at the primary club is eligible for all three Planner teams, and every player at an associated club is eligible for its target Planner team; dump `teamLevel` does not restrict either pool. Missing configured clubs remain visible after a refresh.
- One save-scoped tactic has 11 linked IP/OOP lanes, a 4-3-3 DM IP / 4-1-4-1 DM OOP default, compatible catalog role choices, and a persisted IP weight.
- Senior, Reserves, and Youth each start with one ordered string. Users can add strings without a product limit and can remove a string when its team retains another one. Populated removal requires confirmation.
- Rust ranks slot candidates from the target team's configured club-family sources by combined IP/OOP score. The picker shows score evidence and an existing assignment location. Assigning an already assigned player requires confirmation to move that player.
- A player UID is unique across all Planner cells in one app save. An occupied cell offers clear-first behavior.
- Assignments persist through Load Data. Cells remain occupied when a player is outside the configured team pool or absent from the current snapshot. Missing combined scores render as `—`.
- Team tabs, matrix cells, tactic controls, picker, confirmation, and header actions have keyboard paths. Right-click is a shortcut for the visible string-header menu.

## Final architecture

```text
SQLite migration v6
  → save-scoped club settings and sources
  → one tactic and eleven stable tactic lanes per save
  → ordered strings for Senior, Reserves, and Youth
  → save-wide unique assignments: string + lane + player UID + last-known name

Rust features/planner
  → validates club sources, tactic lanes and weight, strings, and assignment mutations
  → resolves assignment state and current-snapshot score evidence
  → ranks target-team slot candidates and performs add/remove/clear/assign/move commands

React features/planner + /planner route
  → TanStack Query IPC reads and mutations
  → club-family setup, tactic editor, depth matrix, picker, confirmations, and focus state
  → reconciles depth data and invalidates candidate data after related mutations; invalidates all Planner data after Load Data, active-save changes, and club-family saves
```

## Important decisions

- Club relationships are explicit mappings, not inferred from names. The dump has no stable club identity or affiliation relationship, and separate B teams can report `teamLevel = senior`.
- Planner rows belong to app saves, not snapshots. Snapshot replacement changes current player details and scores but does not delete Planner intent.
- Rust owns persistence, validation, candidate scope, combined-score calculation, ordering, and player uniqueness. React owns only transient presentation and interaction state.
- There is one shared tactic per app save. Custom string names and string reordering remain out of scope. Automatic assignment is delivered by the subsequent [Squad Optimizer](./squad-optimizer.md) feature.

## Migration and operational implications

- Migration v6 adds `planner_strings` and `planner_assignments`; previous Planner club-family and tactic rows remain save-scoped.
- Snapshot replacement cascade-deletes only snapshot-owned player data and role scores. Planner assignments retain a UID and last-known name, then resolve as `resolved`, `outside_pool`, or `unresolved` against the new snapshot.
- Successful tactic saves invalidate depth and slot-candidate caches because the role choices and IP weight affect both. Assignment and string mutations reconcile depth data and invalidate slot-candidate caches.
- Manual verification in a native WebView at 1280x800 and 1600x900, and with representative Windows saves, was unavailable at close-out.

## Validation

- `./scripts/dev format` clean.
- `./scripts/dev test` — 137 tests passed.
- `./scripts/dev check` — 182 Rust tests passed and 2 ignored.
- `./scripts/dev smoke` — 11 of 11 passed.
- Mutation testing remains unavailable (`./scripts/dev mutate` exit 69).
- Feature-complete review: Sol High, Blocking **No**, after correction `b93b41e` refreshed depth and candidate caches after tactic saves and string removal, created club setup before dependent panels, and widened header right-click support.

**Delivery commits (final hashes):** `31b091a`, `88925cc`, `a6a761c`, `1fb57c8`, `6b4e36b`, `b60e2aa`, `edd0133`, `b93b41e` (comparison base `f7d6ac20c15c02c401292b462962dc14725ff467`).

## Follow-up

- **Follow-on feature:** [Squad Optimizer](./squad-optimizer.md) adds automatic lineup selection against the shared tactic and combined scores.
- **Deferred:** gap recommendations, tactic libraries, custom string names, string reordering, club-affiliation inference, and snapshot history.
