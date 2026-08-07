# Youth Academy

## Intent

Track save-scoped youth cohorts from the configured Planner club family across snapshot refreshes. Keep unsupported career statistics unavailable until the memory reader supplies them, and let the user record the sale or release outcomes that only the user knows.

## Delivered behavior

- The navigation rail opens `/academy` for the active app save. URL-backed workspaces provide **Overview**, **Graduates**, and **Class** views.
- Every save has one protected automatic `Class of 2025`. A successful snapshot with a valid trusted (`memory` or `derived`) in-game year at or after 2025 creates that year once. Users can create a unique custom class year. Automatic classes cannot be deleted; custom classes require confirmation before deletion. Classes render from oldest to newest by numeric year.
- **Add players** searches only current-snapshot players whose exact `current_club` appears in the configured Planner club family. A player UID can belong to one Academy class per save. Memberships keep the UID and last-known name across snapshot replacement, club-family departure, and unresolved current-snapshot records.
- Overview and class detail show current-snapshot identity fields and supported aggregates. Reader-owned senior appearances, goals, assists, international caps, and dependent aggregates remain `—` with an unavailable explanation. A graduate is defined as a player with at least one senior league appearance; the Graduates view remains intentionally unavailable until that field exists.
- Users can record a tracked player as **Sold** with a buying club and non-negative euro fee, mark a player **Released**, restore either outcome to **Still at club**, or remove a mistaken membership. Class rosters group members into Still at club, Sold, and Released sections. Manual outcomes persist independently of snapshot data and remain visible for departed or unresolved members.
- Outcome metrics receive primary visual hierarchy while setup counts remain supporting context. Sold and released states use text and icons as well as colour. Loading, error, no-snapshot, no-club-family, no-class, and incomplete-class states retain an actionable recovery path.
- A class view with existing classes but no class identifier, or with an identifier that does not match a listed class, recovers to Overview. Save switching and Load Data refresh the Academy query root with the other save-scoped features.

## Final architecture

```text
SQLite migrations v11–v14
  → save-scoped academy_classes (unique year + automatic marker)
  → academy_memberships (one UID per save, last-known name, same-save FK)
  → academy_member_outcomes (optional sold or released row per membership)

Rust features/academy
  → owns class, candidate, membership, outcome, and summary queries
  → resolves members against the active save's current snapshot
  → validates Planner club-family eligibility and all mutations

Snapshot and save lifecycle
  → creates the protected 2025 baseline for each save
  → creates a protected observed-year class inside a successful ingest transaction
  → invalidates or resets Academy query keys on Load Data and save changes

React features/academy + /academy route
  → typed Tauri IPC and TanStack Query wrappers
  → URL-backed workspace, class creation/deletion, picker, roster actions,
    outcome correction, statistics, and accessible empty/error states
```

React does not access SQLite or recreate persistence, eligibility, graduation, or aggregation rules.

## Important decisions

- Academy reuses the Rust Planner club-family service. The Academy page does not duplicate club configuration, and `team_level` does not decide candidate eligibility.
- Classes, memberships, and manual outcomes are save-scoped rather than snapshot-scoped. Snapshot replacement updates live projections but never deletes class history.
- The memory bridge is unchanged. Reader-owned career fields remain nullable and visibly unavailable; sale and release are explicit user-owned facts.
- Automatic generation creates only the fixed 2025 baseline and years observed in successful snapshots. It does not invent intermediate years. A matching manual class keeps its identifier and memberships and is promoted to automatic when the observed year is trusted.
- The implementation stays within existing Rust IPC, SQLite, save-scoping, and Planner boundaries, so no ADR is required.

## Migration and operational implications

- Migration v11 creates the Academy classes and memberships. Migration v12 adds the automatic marker and backfills the 2025 baseline. Migration v13 adds the one-to-one manual outcome table. Migration v14 backfills any missing 2025 baseline and promotes a matching class for each current snapshot only when `game_date_source` is `memory` or `derived` and the date is a valid `yyyy-MM-dd` value for a year at or after 2025. It never replaces memberships. Unknown, malformed, untrusted, pre-2025, or non-current dates do not create an automatic observed-year class.
- Successful snapshot ingest owns observed-year class creation in the same transaction as the current snapshot replacement. Save creation owns baseline initialization. Failed ingest leaves both the prior snapshot and Academy classes unchanged.
- Manual outcome rows require a same-save tracked membership. Sold rows require a non-empty buying club and non-negative whole-euro fee; released rows contain neither. Removing a membership cascades its outcome.
- The feature range is `def766a^..8f8de51`. Planned slices used Terra xhigh for persistence and cross-boundary Rust work, Luna Max for bounded UI and statistics composition, and Sol High or Sol Medium fresh reviews as recorded below. The final feature-complete review used Sol High.

## Validation

- `./scripts/dev format` — clean.
- `./scripts/dev test` — 205 tests passed.
- `./scripts/dev check` — 228 Rust tests passed; two documented scale tests remained ignored.
- Elevated `./scripts/dev smoke` — 15 of 15 passed. The generic smoke fixture has no populated Academy journey, and no full real-browser populated Academy journey was validated at close-out.
- Feature-complete Sol High review: Blocking **No** after one correction round; no Critical, High, Medium, or NIT findings remained.

## Delivery commits and review profiles

| Commits | Delivered slice | Implementation / review profile |
| --- | --- | --- |
| `def766a`, `df9aa5a` | Academy schema, Rust IPC boundary, route shell, class CRUD, and first-use UI | Terra xhigh / Sol High; Luna Max / Sol Medium |
| `0a620f6`, `c1fc593` | Club-family assignment, retained roster states, statistics, graduate workspace, and lifecycle invalidation | Terra xhigh / Sol High; Luna Max / Sol High |
| `aae9f35`, `a30a5c7` | Automatic baseline and observed-year classes; manual sale and release outcomes | Terra xhigh / Sol High for both slices |
| `1a63f72`, `1f2b096`, `87f97c0` | Outcome-first hierarchy, context metrics, and class picker navigation | Luna Max / Sol Medium for the planned visual slice; follow-up fixes retained the same review scope |
| `5366b66`, `b1678cc`, `8f8de51` | Direct outcome actions, concise roster columns, and incomplete-class recovery with migration v14 | Final feature-complete Sol High review; Blocking **No** |

## Follow-up

- Add memory-reader support for senior league appearances, goals, assists, and international caps. Until then, keep the existing `—` states and explanations.
- Add a populated Academy fixture to browser smoke or a full real-browser journey when that validation path exists.
- CSV/HTML import, historical trends, transfer timelines, charts, notes, class renaming, bulk reassignment, and bridge schema changes remain out of scope.
