# Squad Planner

## Status

Active

## Intent

Let the user model one FM26 tactic and organize the selected club family into Senior, Reserves, and Youth depth charts. Every team uses the same tactic and can have any number of ordered strings. The planner uses the role scores from the current snapshot and preserves planning state when Load Data replaces that snapshot.

## User-visible behavior

- A new `/planner` route and nav item open the Squad Planner.
- First use asks the user to choose a primary club from the active save's current snapshot.
- The primary club contributes Senior, Reserves, and Youth players according to `teamLevel`.
- The user can attach separate B-team or youth clubs to Reserves or Youth. Attached clubs contribute all their players unless the user selects a narrower team-level filter.
- One shared tactic defines 11 linked player lanes across separate In-Possession (IP) and Out-of-Possession (OOP) shapes and roles.
- The tactic starts from an editable 4-3-3 DM IP / 4-1-4-1 DM OOP shape with compatible general-purpose roles and a 50/50 IP/OOP score weight.
- Senior, Reserves, and Youth appear as team tabs. Each tab shows tactic lanes as rows and ordered strings as columns.
- Every team starts with one string. The user can add strings without a product-level maximum and remove any string while at least one remains.
- The user assigns a player through a slot-fit picker. Candidates come from the configured club family and rank by the selected lane's combined IP/OOP role score.
- One player can occupy only one planner cell across all teams and strings. Selecting an assigned player moves that player after explicit confirmation.
- Right-clicking a string header opens its actions. The same actions are available from a visible keyboard-operable header menu.
- Planner configuration and assignments persist per app save across app restarts and snapshot refreshes.

## Invariants

- Exactly one tactic is active per app save for MVP.
- The tactic always contains 11 stable player lanes. Each lane links one IP placement and role to one OOP placement and role.
- Each phase role must belong to that phase and support the lane's selected position.
- Senior, Reserves, and Youth use the same tactic and score weight.
- A player UID is unique across all assignments in one app save.
- Planner rows reference app saves, not snapshots. Load Data must not delete planner state.
- Assignments store the player UID and last-known name. If the UID is absent from the current snapshot, the cell remains occupied and visibly unresolved until the user clears or replaces it.
- Changing club-family sources or tactic roles never silently deletes assignments.
- Removing a populated string requires confirmation and deletes only that string's assignments.
- Rust validates every persisted planner mutation. React never opens SQLite or recomputes role scores.
- Missing role scores remain unknown and render as `—`; they never become zero.

## Non-goals

- Multiple named tactics or tactic libraries per app save.
- Importing a tactic from FM26 or writing a tactic back to the game.
- Team instructions, set pieces, duties, tactical-style presets, or an FM Visualiser clone.
- Automatic inference of B-team relationships from club names.
- Automatic lineup or string selection. The Squad Optimizer owns automated assignment.
- Custom string names, player comparison, transfer workflows, or squad-history tracking.
- Editing the snapshot's `teamLevel`, club, or loan data.

## Current-state map

- **Relevant components:** `AppNavRail` exposes Dashboard, Search, and Planner. The `/planner` route currently owns the no-snapshot state and club-family setup panel. Shared `Panel`, `Modal`, `Button`, `SelectField`, `EmptyState`, and `ScoreBadge` primitives cover most planned UI.
- **Data model:** `players` already stores `current_club`, `parent_club`, `team_level`, positions, and player UID. `player_role_scores` stores every IP/OOP role score for the current snapshot.
- **Persistence and migrations:** SQLite migration v4 is current. Snapshot replacement cascade-deletes players and role scores, while `planner_club_settings` and `planner_club_sources` stay save-scoped without snapshot foreign keys.
- **Existing behavioral assumptions:** one app save is active; all player reads use its current snapshot; Load Data and save switching invalidate snapshot, planner, search, and profile query trees.
- **Architectural seams:** React feature code belongs in `src/features/planner`; Rust persistence and queries belong in `src-tauri/src/features/planner`; the route composes planner and snapshot context without cross-feature imports.
- **Tests and validation:** Vitest + RTL own route and interaction behavior; Rust tests own migrations, validation, uniqueness, persistence, and score joins; Playwright smoke owns the browser planner path with stubbed IPC.
- **Primary risks:** club relationships are absent from the dump; club identity is a name string; phase-slot linkage must remain clear during tactic edits; horizontally growing strings must remain usable at 1280x800.

## Feature architecture (this feature)

Planner state is save-scoped and survives replacement of the current snapshot.

- **Club family:** one primary club plus source rows assigned to Senior, Reserves, or Youth. A source stores a club name and an optional `teamLevel` filter. Default primary-club sources use the matching level; an attached B-team or youth club defaults to all levels.
- **Tactic:** one tactic per save with a global IP weight and 11 stable lanes. Each lane stores its IP pitch placement and role plus its OOP pitch placement and role. Pitch placements map to the existing FM position tags used by the scoring catalog.
- **Depth chart:** each team owns ordered string rows. Assignments point to a string and tactic lane, with a save-wide unique player UID and last-known player name. String order produces the displayed 1st, 2nd, and later ordinal labels.
- **Planner read model:** Rust resolves saved assignments against the active save's current snapshot, joins the chosen IP/OOP role scores, calls the existing combined-score helper, and returns bounded tactic, source, string, assignment, and candidate DTOs.
- **Candidate picker:** Rust searches only configured club-family sources, excludes or identifies already assigned players, and sorts for the selected lane. React owns search input, focus, confirmation, and presentation.
- **Cache invalidation:** planner mutations invalidate planner keys. Load Data and active-save changes also invalidate planner reads because current player details and scores may change, while persisted planner configuration remains intact.

## Uncertainty register

### Known

- Dump schema v5 provides `currentClub` as text and `teamLevel` as `senior | reserve | youth | null`.
- Some FM clubs model B teams as separate clubs, so primary-club plus `teamLevel` filtering alone is incomplete.
- The dump has no stable club UID or affiliation relationship.
- FM26 tactics use separate IP and OOP formations and role sets. The same selected player links both phases.
- `combine_role_scores` already supports a caller-supplied IP weight and defaults to 50/50.

### Assumptions

- Club names are stable enough within one app save to persist explicit source mappings.
- One editable tactic per app save meets MVP needs.
- A 4-3-3 DM IP / 4-1-4-1 DM OOP starting shape gives a useful first-run state without restricting later edits.
- The number of user-created strings stays small enough for one save-scoped read without pagination.

### Decisions

- Use one tactic across Senior, Reserves, and Youth.
- Enforce one assignment per player across the whole planner.
- Use explicit club-family sources. Do not infer B-team relationships from names.
- Keep source mappings and assignments when their players or clubs disappear from a later snapshot; show a visible unresolved state.
- Support right-click as a shortcut, with the same string actions exposed through a visible accessible menu.
- Keep tactic creation focused on formations, phase roles, and IP/OOP weight. Defer instructions and automatic selection.

### Unknowns

- No unknown blocks the first commit. Live FM verification may reveal club-name or `teamLevel` edge cases that require source-selection copy or filtering changes, not a new persistence boundary.

### Risks

- A custom database may rename a configured club. The planner must show a missing-source warning and let the user replace the mapping without deleting assignments.
- A player may move outside the configured club family after Load Data. Keep the assignment and mark the player as outside the current pool.
- Free-form phase editing can create invalid role/position pairs. Both the editor and Rust mutation boundary must reject them.
- Many string columns can overflow the viewport. Keep tactic labels sticky and use explicit horizontal scrolling rather than shrinking cells below their readable width.

## Walking skeleton

PR 1, commit 1: open Planner, choose Barcelona as the primary club, attach Barça Athletic to Reserves, switch away and back, and see the saved club-family configuration. This proves the new route, save-scoped persistence, snapshot-derived club choices, and React-Rust IPC path before tactic or assignment complexity lands.

## Delivery plan

### PR 1 — Create the club tactic

**Status:** Active

**Provisional PR title:** `feat(planner): create dual-phase club tactic`

**Purpose:** Establish the save-scoped club family and the shared FM26 dual-phase tactic as an independently useful planner foundation. This PR is a merge boundary because it introduces the persistent model and validates the tactic interaction before squad assignments depend on it.

**Depends on:** Snapshot ingest, role scoring engine, and the current app shell.

#### Commit 1 — Configure club-family sources

**Status:** Completed — `31b091a`

**Work:** Add save-scoped club-family persistence, distinct-club and source-management IPC, the `/planner` shell, nav entry, no-snapshot Load Data guidance, first-use primary-club setup, and editable Reserves/Youth associated-club sources. Seed the primary club's three team-level sources and keep missing mappings visible after refresh.

**Out of scope for this commit:**

- Tactic persistence or editing.
- Strings, player assignments, candidate ranking, or combined scores.
- Automatic club-affiliation inference.

**Validation:** Start with a failing Rust persistence/isolation test and failing route interaction tests that prove an attached separate B club survives reload for one app save without leaking to another, and that `/planner` points to Load Data when the active save has no snapshot. Run affected Vitest and Rust tests, `./scripts/dev check`, and planner route smoke coverage for both first-run states.

**Provisional commit:** `feat(planner): configure club family sources`

#### Commit 2 — Persist the dual-phase tactic

**Status:** Active

**Work:** Add the save-scoped tactic and 11 stable lane model, seed the default IP/OOP shapes with compatible general-purpose roles, expose phase-compatible role and placement options from the Rust scoring catalog, validate complete tactics and role-position compatibility, and persist the IP weight.

**Out of scope for this commit:**

- Visual pitch editing.
- Squad strings or player assignments.
- Team instructions, multiple tactics, or tactic import.

**Validation:** RED Rust tests for 11-lane persistence, save isolation, invalid phase/role/position rejection, and weight bounds. Run affected Rust tests and `./scripts/dev check`.

**Provisional commit:** `feat(planner): persist dual-phase tactic`

#### Commit 3 — Add the dual-phase tactic editor

**Status:** Pending

**Work:** Add the planned IP, OOP, and side-by-side tactic views with editable pitch placements, phase-filtered role pickers, linked lane identity, IP/OOP weight control, complete loading/error states, and pointer plus keyboard operation. Save through the tactic IPC contract and show validation without losing the draft.

**Out of scope for this commit:**

- Squad strings, assignments, or candidate lists.
- Team instructions or the nine-zone FM Visualiser.
- Multiple tactic presets beyond the seeded starting shape.

**Validation:** RED component tests for linked phase edits, incompatible-role prevention, keyboard editing, weight changes, and failed-save draft retention. Run affected Vitest tests, `./scripts/dev check`, and Playwright smoke for tactic creation.

**Provisional commit:** `feat(planner): add dual-phase tactic editor`

### PR 2 — Plan three-team squad depth

**Status:** Pending

**Provisional PR title:** `feat(planner): plan three-team squad depth`

**Purpose:** Add the depth-chart model and user workflow after the tactic boundary has landed and proved stable. This split keeps the migration and interaction risk of tactic creation independently reviewable before assignments depend on lane identity.

**Depends on:** PR 1 merged to trunk.

#### Commit 1 — Persist squad depth assignments

**Status:** Pending

**Work:** Add save-scoped ordered strings and assignments for the three fixed teams. Seed one string per team, enforce save-wide player uniqueness, retain last-known names across snapshot replacement, resolve current snapshot details and combined lane scores in Rust, and support add, remove, clear, assign, and move mutations.

**Out of scope for this commit:**

- Depth-chart matrix UI or player picker.
- Optimized or automatic assignments.

**Validation:** RED Rust tests for default strings, unlimited ordered additions, final-string protection, populated-string deletion, unique player moves, snapshot survival, combined scores, and save isolation. Prove that replacing club-family sources or changing tactic roles preserves assignments and returns the documented outside-pool or unresolved state instead of deleting them. Run affected Rust tests and `./scripts/dev check`.

**Provisional commit:** `feat(planner): persist squad depth assignments`

#### Commit 2 — Add the three-team depth matrix

**Status:** Pending

**Work:** Render Senior, Reserves, and Youth tabs over one shared tactic matrix. Keep tactic lanes sticky, strings horizontally scrollable, cells keyboard reachable, and player identity plus combined score honest for missing, outside-pool, and unresolved assignments.

**Out of scope for this commit:**

- Assigning players from the UI.
- Adding or removing strings from the UI.
- Automatic gap analysis or optimization.

**Validation:** RED route/component tests for team switching, shared tactic rows, string order, horizontal overflow structure, score display, and unresolved/outside-pool states. Run affected Vitest tests and `./scripts/dev check`.

**Provisional commit:** `feat(planner): add three-team depth matrix`

#### Commit 3 — Assign players by slot fit

**Status:** Pending

**Work:** Add the searchable slot-fit picker backed by the configured club family. Rank candidates by combined score for the selected tactic lane, show IP/OOP evidence and current assignment location, support assignment and confirmed moves, and restore focus to the originating cell.

**Out of scope for this commit:**

- Transfer search outside the configured club family.
- Automated lineup selection or multi-slot optimization.
- Drag-only interaction.

**Validation:** RED Rust query tests and component tests for the target team's source union, separate B-club candidates, the All club family option, score ordering, null-score display, uniqueness, confirmed moves, cancellation, and focus restoration. Run affected Vitest and Rust tests and `./scripts/dev check`.

**Provisional commit:** `feat(planner): assign players by slot fit`

#### Commit 4 — Manage squad string columns

**Status:** Pending

**Work:** Add strings from the matrix header, expose add/remove actions through right-click and the visible header menu, renumber ordinal labels after removal, confirm destructive removal of populated strings, and complete the browser smoke path for all three teams.

**Out of scope for this commit:**

- Custom string names or reorder controls.
- A fixed maximum string count.
- Optimizer controls or gap recommendations.

**Validation:** RED component tests for pointer and keyboard menus, add-after behavior, ordinal renumbering, last-string protection, populated removal confirmation, cancellation, and focus return. Run affected Vitest tests, `./scripts/dev check`, and the complete Planner Playwright smoke flow.

**Provisional commit:** `feat(planner): manage squad string columns`

## Active work

**PR:** PR 1 — Create the club tactic

**Commit:** Persist the dual-phase tactic

### RED test (active commit)

Persist an 11-lane dual-phase tactic with the default IP/OOP shapes, reload it, and assert that a second app save starts independently. Reject incomplete lanes, phase-incompatible roles, invalid role-position pairs, and IP weights outside 0–1.

### Expected outcome

Each app save has one persisted 11-lane tactic with linked IP and OOP placements and roles, a valid default shape, and a saved IP/OOP weight. The Planner route can load that tactic without introducing strings, assignments, or editor UI yet.

### Explicit exclusions

- Do not add tactic editor UI, string, assignment, or optimizer tables in this commit.
- Do not infer affiliated clubs from names.
- Do not edit bridge schema v5 or memory scanning.

## Discoveries and replanning

- **Planned:** identify the managed club from current snapshot data. **Discovered:** dump schema v5 has no manager or affiliation identity, and B teams can be separate clubs whose players report `teamLevel = senior`. **Why:** an explicit save-scoped club-family mapping handles both same-club levels and separately modeled teams without unreliable name heuristics.

## Completed work

| PR | Commit | Hash | Notes |
| --- | --- | --- | --- |
| PR 1 | Configure club-family sources | `31b091a` | Added migration v4, save-scoped source persistence and validation, Planner route/setup UI, IPC, cache invalidation, and first-use smoke coverage. |

## Final validation

At feature end: `./scripts/dev test`, `./scripts/dev check`, `./scripts/dev smoke`, feature-complete reviewer pass, manual 1280x800 and 1600x900 layout check, keyboard-only tactic and squad workflow, and manual Windows verification against one same-club reserve model plus one separate B-club model when representative saves are available.

## Documentation impact

- Planning adds the Squad Planner interaction contract to `DESIGN.md`.
- Feature completion must update `ARCHITECTURE.md` with migrations, planner persistence, IPC/read paths, invalidation, and route ownership.
- Feature completion must archive this ledger and advance `TODO.md` to Squad Optimizer.
