# Youth Academy

## Status

Active

## Intent

Add a save-scoped Youth Academy page for grouping players who came through the configured club family into `Class of YYYY` cohorts and following those players across later snapshots. The first delivery establishes honest class and membership tracking from the current memory data while reserving visible placeholders for career statistics that the bridge does not yet expose.

## User-visible behavior

- The navigation rail contains **Youth Academy**, which opens the dedicated `/academy` route for the active save and current snapshot.
- A user with a configured Planner club family can create a uniquely named `Class of YYYY`, open a class, and delete it with destructive confirmation.
- **Add players** opens a searchable picker containing only current-snapshot players whose current club is one of the configured club-family sources. A player can belong to at most one class in a save.
- A class membership survives snapshot replacement and a player's departure from the club family. The last known name remains visible when the player is absent from the current snapshot, and departed or unresolved members receive an explicit warning rather than disappearing.
- Overview and class detail show counts and current-snapshot identity data that are actually available. Senior league appearances, goals, assists, international caps, sale income, released status, and any aggregate derived from those fields render as `—` with an unavailable explanation.
- A player becomes a graduate when senior league appearances are at least one. Until that field reaches the memory-reader contract, graduate status, the graduate count, and the Graduates workspace remain explicitly unavailable rather than reporting zero.
- Loading a new snapshot or switching saves refreshes Academy data through the same global lifecycle as Search, Player Profile, and Planner.

## Invariants

- Academy classes and memberships belong to one app save and never leak across saves.
- The Academy reads the existing Planner club-family configuration; it does not own or duplicate club setup.
- New memberships can only be created from current-snapshot players in the configured club family, matched by exact `current_club` names. The unreliable `team_level` field does not gate candidate eligibility.
- One player UID belongs to at most one Academy class per save.
- Snapshot replacement never deletes classes or memberships. An unresolved member retains `player_uid` and `last_known_name`.
- A graduate means `senior_league_appearances >= 1`; no manual graduate override or proxy may substitute for that rule.
- Missing memory fields stay nullable and visibly unavailable. Unknown values are never coerced to zero, `No`, released, sold, or graduated.
- SQLite access and aggregation remain Rust-owned behind typed Tauri commands; React does not issue SQL or recreate persistence rules.

## Non-goals

- Extending the FM memory bridge, dump schema, validation, or snapshot ingestion with career statistics.
- CSV or HTML import/export from the reference tracker.
- Manual editing of appearances, goals, assists, caps, transfer fees, career status, personality labels, or graduate status.
- Tracking players who have never been present in the configured club family.
- Inferring academy membership automatically from age, team level, club-name similarity, or career history.
- Replacing or extending Planner club-family setup from the Academy page.
- Charts, historical trends, transfer timelines, notes, class renaming, or bulk reassignment.
- Copying the reference file's visual styling, fonts, emoji, or standalone-app navigation.

## Current-state map

- Relevant components: `src/app/components/app-nav-rail.tsx` owns primary navigation; `src/app/components/app-top-bar.tsx` owns snapshot/save invalidation; routes under `src/app/routes/` remain thin and compose feature modules under `src/features/`.
- Data model: current snapshot players expose UID, name, age/DOB, nationality, height, preferred foot, positions, current/parent club, reported team level, loan state, attributes, CA/PA, and market value. They do not expose senior league appearances, goals, assists, caps, realized transfer fees, released/sold outcomes, or a textual personality label.
- Persistence and migrations: Rust owns SQLite migrations in `src-tauri/src/db/migrations.rs`. Planner classes of data are save-scoped and persist independently of replaceable snapshots by retaining player UID and last-known name.
- Existing behavioral assumptions: Planner's `planner_club_settings` and `planner_club_sources` are the single configured club-family source. Primary-club rows represent Senior, Reserves, and Youth, attached clubs may augment Reserves or Youth, and player-pool membership matches exact current-club names without relying on reported team level.
- Architectural seams: Tauri commands are registered in `src-tauri/src/lib.rs`; Rust feature modules live under `src-tauri/src/features/`; frontend IPC/query/type/component code lives under `src/features/`; TanStack Router owns URL state.
- Project validation commands: `./scripts/dev format`, `./scripts/dev test [target...]`, `./scripts/dev check-rust`, `./scripts/dev check`, and `./scripts/dev smoke`. `bridge-test` is unnecessary while the bridge is unchanged; `mutate` remains unsupported.
- Primary risks: preserving cross-snapshot memberships, enforcing same-save uniqueness in SQLite, reusing rather than cloning Planner configuration, and avoiding false statistics while the dump contract lacks career data.

## Feature architecture

- Add `src-tauri/src/features/academy/` as the owner of Academy class, membership, candidate, summary, and placeholder-stat behavior. It may call the established Planner club-family service in Rust; no Academy-specific club-family persistence is introduced.
- Add save-scoped `academy_classes` and `academy_memberships` tables. Classes use a positive `class_year` unique within a save. Memberships store `save_id`, `class_id`, `player_uid`, and `last_known_name`, enforce one membership per save and UID, and use a same-save composite foreign key so a membership cannot point to another save's class.
- Resolve members against the active save's current snapshot at read time. Persisted identity is the fallback; current player fields are projections, not copied Academy state.
- Return optional career-stat fields in the Academy member contract. This feature returns them as null placeholders, giving the follow-up memory-reader feature one explicit seam without speculative persistence.
- Add `src/features/academy/` for typed IPC wrappers, query keys, view parsing, display helpers, and focused components. Keep `src/app/routes/academy.tsx` as the route composition boundary.
- Use URL-backed `view=overview|graduates|class` state, with a class identifier when `view=class`. Invalid or deleted class selections fall back to Overview.
- Mutations invalidate the smallest Academy queries. Snapshot load and save switching invalidate the Academy root alongside the existing feature roots.

## Uncertainty register

### Known

- The reference tracker's core useful concept is a manually curated `Class of YYYY` cohort with overview, graduate, and per-class statistics.
- The user explicitly chose the existing Planner club family as the Academy boundary and confirmed that candidate selection remains restricted to that family.
- The user explicitly defined graduation as at least one senior league appearance.
- The current memory schema does not supply the career statistics needed to decide graduation or calculate graduate, released, sales, goal, assist, or cap aggregates.
- The snapshot summary exposes an optional in-game date suitable for prefilling the class year when present.

### Assumptions

- A class year is a positive integer entered by the user; the create form may prefill the current snapshot's in-game year but remains editable and does not impose an arbitrary future-year ceiling.
- Removing a player from a class is sufficient before assigning that player to another class; a dedicated move workflow is unnecessary for the first version.
- A compact table is preferable to reproducing every reference column because current data availability is narrower and desktop space is bounded.

### Decisions

- Deliver one PR because the migration, service, route, assignment flow, and honest statistics states form one additive user capability and do not require an independently deployable foundation.
- Use a dedicated `/academy` route labelled **Youth Academy** and an `academy` code/commit scope.
- Treat the configured club-family source names as the only assignment boundary. Reported team level can be displayed but cannot decide eligibility.
- Retain departed and unresolved members by UID and last-known name rather than restricting every read to the current club family.
- Define nullable career-stat fields now and render unavailable states; do not add manual substitutes or speculative snapshot columns.
- Show class count and tracked-player count immediately. Any statistic whose complete meaning depends on absent data remains `—`; a reported current senior-squad count may include only resolved members whose snapshot explicitly reports `team_level = senior`, with the limitation stated in the UI.
- Use the repository design system and existing Panel, Data Table, Modal, Button, empty-state, loading, and error patterns rather than the reference file's presentation.
- No ADR is required: this extends established Rust IPC, SQLite, save-scoping, and Planner club-family boundaries without changing them.

### Unknowns

- The exact FM memory objects and semantics for senior league appearances, goals, assists, international caps, transfer fees, and release outcomes belong to the follow-up memory-reader feature.
- Whether future sale income should represent the latest fee, cumulative fees, or only fees received by the user's club must be decided when the memory source is understood.
- Whether the reported `team_level` is reliable enough for a durable “in first team” statistic remains unproven; this feature labels any present-tense count as reported snapshot data.

### Risks

- A uniqueness constraint that omits `save_id` could incorrectly prevent the same FM UID from being classified in different saves.
- A membership foreign key that does not include the save could allow cross-save class references.
- Filtering all reads by current club family would silently erase the exact graduates and departures the feature exists to track.
- Treating null career stats as zero would falsely claim that nobody has graduated or generated income.
- Depending on Planner React modules for club configuration would create a cross-feature frontend ownership leak; reuse belongs in the Rust service seam.
- `src-tauri/src/db/migrations.rs`, `src-tauri/src/lib.rs`, and `src/app/components/app-top-bar.tsx` are high-coupling or high-churn files. Repowise's index was stale by two commits, so these are review-focus hints only and direct repository evidence remains authoritative.

## Walking skeleton

Commits 1 and 2 establish the thinnest end-to-end path: navigate to `/academy`, read the active save and its existing club family through Tauri, create `Class of YYYY`, reload the query, and see the class persist in Overview. Player assignment and career-stat presentation extend that proven route without changing its ownership boundary.

## Delivery plan

### PR 1 — Add youth graduate tracking

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(academy): add youth graduate tracking`

**Purpose:** Deliver the first complete Youth Academy page: persistent classes, club-family member assignment, current player context, and honest placeholders for graduation and career statistics.

**Depends on:** Existing snapshot/save persistence, Planner club-family configuration, and the Rust-owned Tauri IPC architecture.

#### Commit 1 — Persist youth class membership

**Status:** Completed

**Provisional commit:** `feat(academy): persist youth class membership`

**Work:** Add the Academy migration and Rust feature boundary for listing, creating, and deleting classes; resolving members; listing eligible club-family candidates; assigning and removing one player; and returning nullable career-stat fields. Prove save isolation, same-save class references, unique class years, unique player membership, club-family filtering, and cross-snapshot retention in Rust tests.

**Out of scope:**

- Navigation, route components, frontend queries, or visible UI.
- Memory bridge or snapshot schema changes.
- Automatic membership or derived career-stat values.

**Implementation packet:**

- Owners and files: `src-tauri/src/db/migrations.rs`; new `src-tauri/src/features/academy/{mod.rs,service.rs,commands.rs}` and focused test module/support as needed; `src-tauri/src/features/mod.rs`; `src-tauri/src/lib.rs`.
- Existing patterns to verify: Planner save-scoped migrations and assignment retention; `snapshot::service::active_save_id`; Planner's `get_club_family`; typed DTO conversion and command registration.
- Constraints and invariants: use migration v11; positive class year unique per save; one membership per save and player UID; composite same-save class foreign key; cascade on save/class deletion; exact club-name candidate membership; retain UID/name when snapshots change; null statistics remain null.
- Dependencies and ordering: migration precedes service writes; service owns validation/transactions; commands translate DTOs only; registration follows compiled module exposure.

**Implementation profile:** Terra xhigh — persistence constraints, cross-feature Rust service reuse, and several mutation invariants need deliberate execution but fit established local patterns.

**Review profile:** Sol High — a fresh review must scrutinize migration safety, save isolation, data retention, and the candidate trust boundary before UI depends on it.

**Validation:** Add one focused Rust test at a time and run `./scripts/dev check-rust` to capture RED for the expected missing Academy table or service behavior, then GREEN for migration application, class CRUD, uniqueness, family filtering, membership retention, and null placeholders. Finish with `./scripts/dev format` and `./scripts/dev check`.

**Stop conditions:** Replan before writing if same-save membership cannot be enforced without altering an established persistence contract, if Planner club-family access would require duplicating configuration, or if adding placeholder fields forces a memory dump schema change. Ask the developer if current files reveal a conflicting definition of class year or membership.

**Review mandate:**

- Verify v11 applies cleanly to both empty and populated v10 databases without altering existing Planner data.
- Verify class and membership constraints cannot cross save boundaries.
- Verify duplicate class years and duplicate player membership fail predictably without partial writes.
- Verify only current-snapshot members of configured exact club names can be newly assigned.
- Verify departed and unresolved assigned players remain queryable with last-known identity.
- Verify absent career statistics serialize as null and never become zero-derived aggregates.
- Verify commands remain thin and do not expose raw SQL or accept a caller-controlled save ID in place of the active save.

#### Commit 2 — Add youth class workspace

**Status:** Completed

**Provisional commit:** `feat(academy): add youth class workspace`

**Work:** Add the Youth Academy nav item, `/academy` route, typed frontend Academy API/query layer, IPC mock, URL-backed Overview/Class/Graduates workspace shell, class creation, class deletion, and the configured/no-snapshot/no-club-family states. This completes the walking skeleton with persisted class cards on Overview.

**Out of scope:**

- Player picker, membership mutation UI, or populated class table.
- Claims that unavailable career statistics are known.
- Reworking global navigation or Planner club setup.

**Implementation packet:**

- Owners and files: `src/app/components/app-nav-rail.tsx`; new `src/app/routes/academy.tsx` and `src/app/routes/academy.test.tsx`; new `src/features/academy/{api,components,types,utils}/`; `src/testing/academy-ipc-mock.ts`; `src/testing/setup.ts`; generated `src/routeTree.gen.ts`; `src/app/app-shell-routing.test.tsx`.
- Existing patterns to verify: thin Search/Planner routes; `planner-keys.ts`; Planner workspace URL parsing; shared Button, Panel, Modal, EmptyState, loading, error, and nav patterns; Tauri IPC test mocks.
- Constraints and invariants: `/academy` is accessible from the rail; class year uses an accessible labelled integer input and optional in-game-year prefill; invalid/deleted URL selection returns to Overview; destructive deletion requires confirmation; no-club-family state links to `/planner?view=clubs`; no duplicate feature-owned club setup.
- Dependencies and ordering: consumes commit 1 commands; route generation follows file creation; UI mutations invalidate Academy keys.

**Implementation profile:** Luna Max — the path is a bounded UI composition task with strong existing route, query, modal, and workspace patterns.

**Review profile:** Sol Medium — fresh review should focus on route behavior, mutation recovery, accessibility, and adherence to the planned design contract.

**Validation:** First add a route test that fails because `/academy` and persisted class creation do not exist. Run `./scripts/dev test src/app/routes/academy.test.tsx src/app/app-shell-routing.test.tsx` for RED and GREEN, then `./scripts/dev format` and `./scripts/dev check`.

**Stop conditions:** Replan if the class workspace requires a second club-family owner, if route search cannot represent the three views without incompatible global changes, or if a generated-route change touches unrelated routes.

**Review mandate:**

- Verify rail navigation and direct `/academy` entry resolve without disturbing existing routes.
- Verify URL state is validated and deleted/unknown classes recover to Overview.
- Verify create/delete errors stay visible and do not discard recoverable input.
- Verify delete confirmation clearly names the class and protects populated membership data.
- Verify no-snapshot and no-club-family states give the correct next action.
- Verify keyboard/focus behavior and semantic workspace controls meet the design system.
- Verify the reference HTML's styling and import flows did not leak into scope.

#### Commit 3 — Assign club-family players to classes

**Status:** Completed

**Provisional commit:** `feat(academy): assign club-family players to classes`

**Work:** Add the searchable Add Players modal, exclude already classified players, assign/remove membership, and render the current class roster with current-snapshot identity, reported team context, PA, determination, height, preferred foot, and explicit departed/unresolved warnings.

**Out of scope:**

- Automatic class assignment, bulk import, or a dedicated cross-class move operation.
- Editing player facts or using reported team level as an eligibility gate.
- Memory-reader career statistics.

**Implementation packet:**

- Owners and files: Academy API, type, component, and route-test files under `src/features/academy/` and `src/app/routes/academy.test.tsx`; `src/testing/academy-ipc-mock.ts`.
- Existing patterns to verify: Planner slot candidate modal and assignment invalidation; Search table identity formatting; unresolved/outside-pool Planner warnings; shared Data Table and Modal behavior.
- Constraints and invariants: candidate rows come only from exact configured club-family names in the current snapshot; one UID can be assigned once per save; removing then reassigning is supported; persisted members remain visible after departure/snapshot replacement; unknown values render `—`; no implicit manual stat store.
- Dependencies and ordering: consumes commit 1 candidate/member commands and commit 2 workspace; refresh class, overview, and candidate queries after mutation.

**Implementation profile:** Terra xhigh — the visible flow spans modal state, server-owned eligibility, uniqueness errors, and resilient current-versus-persisted identity handling.

**Review profile:** Sol High — fresh review must challenge the eligibility and uniqueness contracts and ensure the UI cannot silently drop departed Academy members.

**Validation:** Start with a failing `src/app/routes/academy.test.tsx` case for club-family-only candidates and persisted membership. Prove RED, implement GREEN, then cover duplicate prevention, removal/reassignment, departed/unresolved display, mutation failure retention, and modal focus restoration. Run `./scripts/dev test src/app/routes/academy.test.tsx`, `./scripts/dev format`, and `./scripts/dev check`.

**Stop conditions:** Replan if eligibility cannot be enforced in Rust from existing club-family data, if resolving membership requires snapshot-history persistence, or if a move workflow becomes necessary to prevent destructive user behavior.

**Review mandate:**

- Verify server-side candidate eligibility matches the full configured club family and ignores team level.
- Verify already classified UIDs cannot be duplicated through stale or direct IPC requests.
- Verify removal affects only the chosen membership and failed mutations preserve prior UI state.
- Verify departed and missing players remain listed with accessible warnings and last-known names.
- Verify displayed current values are distinguished from unavailable career values.
- Verify candidate search, selection, dismissal, and focus restoration work by keyboard.
- Verify the class table stays usable at 1280×800 without document-level horizontal overflow.

#### Commit 4 — Surface graduate tracking statistics

**Status:** Active

**Provisional commit:** `feat(academy): surface graduate tracking statistics`

**Work:** Complete Overview, per-class summaries, and the Graduates workspace. Show class and tracked counts, a carefully labelled reported-senior count when known, nullable career-stat cells and aggregate cards, the exact graduation rule, and unavailable explanations. Refresh Academy queries after snapshot loads and save switches, then verify the complete page visually and through the product smoke path.

**Out of scope:**

- Populating absent career statistics or changing the bridge/dump contract.
- Transfer-history interpretation, sale/release inference, charts, or manual overrides.
- Claiming graduate or income totals when source fields are unavailable.

**Implementation packet:**

- Owners and files: Academy summary/workspace components and tests under `src/features/academy/`; `src/app/routes/academy.test.tsx`; `src/app/components/app-top-bar.tsx`; `src/app/app-top-bar.test.tsx`; `e2e/smoke.spec.ts` only when the existing smoke journey needs an Academy assertion.
- Existing patterns to verify: snapshot/search/player/planner invalidation in AppTopBar; numeric formatting and `—` conventions; compact KPI panels; smoke navigation and empty-state fixtures.
- Constraints and invariants: graduation is true only when a non-null senior league appearance value is at least one; null means unavailable, never false; unavailable aggregate cards explain the missing memory source; current-snapshot refresh cannot delete persisted membership; save switch cannot show stale Academy data.
- Dependencies and ordering: consumes the complete class roster and placeholder contract; this commit finishes all planned user-facing statistics states without bridge work.

**Implementation profile:** Luna Max — the aggregation and presentation contract is explicit, and the remaining work follows established query invalidation and state patterns.

**Review profile:** Sol High — fresh review should prioritize data honesty across every summary and ensure lifecycle invalidation cannot mix saves or snapshots.

**Validation:** Begin with failing Academy route tests that require `—` rather than zero for null career data and require the graduation rule when a fixture supplies appearances. Add an AppTopBar test that fails until Academy keys invalidate after load/switch. Run `./scripts/dev test src/app/routes/academy.test.tsx src/app/app-top-bar.test.tsx src/app/app-shell-routing.test.tsx`, `./scripts/dev format`, `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev smoke`.

**Stop conditions:** Replan if UI completion would require inventing source values, if graduation cannot remain nullable in the typed contract, or if global invalidation would require broad cache clearing instead of the Academy key root.

**Review mandate:**

- Verify `>= 1` senior league appearance is the sole graduation rule and null remains unknown.
- Verify every unavailable count, total, and cell renders `—` plus enough context to avoid a false zero.
- Verify reported senior-squad data is explicitly limited to resolved snapshot records and is not used as a graduation proxy.
- Verify Academy refreshes on both snapshot load and active-save switch without cross-save cache bleed.
- Verify Overview, Graduates, and Class views remain useful in empty, partial, loading, and error states.
- Verify number formatting, table semantics, focus order, and responsive overflow follow `DESIGN.md`.
- Verify full tests, commit gate, smoke path, and manual populated-state inspection support the completion claim.

## Active work

**PR:** Add youth graduate tracking

**Commit:** Surface graduate tracking statistics

### RED proof

Add Academy route tests that require `—`, rather than zero, for null career data and require the exact graduation rule when a fixture supplies senior league appearances. Add an AppTopBar test that fails until Academy queries invalidate after snapshot load and save switch. The expected RED is missing graduate presentation or Academy cache invalidation, not a test-harness failure.

### Expected outcome

Overview, Class, and Graduates views show only source-supported statistics, label unavailable values honestly, and treat a player as a graduate only when reported senior league appearances are at least one. Academy queries refresh after snapshot load and active-save switch without changing persisted memberships.

### Explicit exclusions

No bridge/schema change, source-value invention, transfer-history interpretation, sale or release inference, charts, manual overrides, CSV/HTML flow, or Git publication belongs in the active commit.

## Discoveries and replanning

- Initial planning confirmed that the Planner club family is reusable at the Rust service seam; no separate Academy club configuration is needed.
- The reference file mixes manual facts with imported statistics. The app will keep live memory data authoritative and show unsupported fields as unavailable instead of reproducing manual edit controls.
- Repowise architecture/risk output was two commits stale and its answer synthesis was unavailable. Direct source, tests, manifests, and wiki documents were used for the plan; its migration/lib/top-bar risk signals are advisory review focus only.
- Commit 1 adds `UNIQUE (save_id, id)` to the v11 class table so SQLite can enforce the planned composite same-save membership foreign key. Class deletion also requires backend confirmation; candidate search is bounded to 100 results and 120 characters, following established local IPC limits.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Commit 1 — Persist youth class membership | Pending record | v11 Academy schema, Rust commands, member retention, and nullable career-stat placeholders | Sol High accepted; no Critical, High, or Medium findings | Backend delete confirmation and bounded candidate input recorded under Discoveries. |
| PR 1 | Commit 2 — Add youth class workspace | Pending record | Youth Academy rail entry, URL-backed workspace shell, typed queries, class create/delete, first-use states, IPC mock, and route coverage | Sol Medium accepted after correction; no Critical, High, Medium, or NIT findings | Two modal lifecycle findings were fixed before commit; the planned live `Class of YYYY` preview was added. |
| PR 1 | Commit 3 — Assign club-family players to classes | Pending record | Searchable club-family picker, typed membership mutations, current/departed/unresolved roster, and assignment/removal route coverage | Sol High accepted after two correction rounds; no Critical, High, or Medium findings remain | Internal table overflow is structural coverage only; populated viewport inspection remains in Commit 4 final validation. |

## Final validation

**Feature review profile:** Sol High — the feature crosses migration, Rust IPC, query lifecycle, and data-honesty boundaries, so final review must assess the complete path with emphasis on save isolation and unavailable statistics.

- Run `./scripts/dev format` before the final staged review.
- Run `./scripts/dev test` and retain the full-suite result.
- Run `./scripts/dev check` as the required commit gate.
- Run `./scripts/dev smoke` with Chromium installed and include an Academy route assertion when the fixture supports the required state.
- Inspect empty, configured-empty, populated, departed/unresolved, loading, error, and unavailable-stat states at 1280×800 and 1600×900. Confirm no document-level horizontal overflow, keyboard-operable dialogs/tabs, visible focus, and honest `—` values.
- Do not report `./scripts/dev bridge-test` or `./scripts/dev mutate` as passed; the bridge is unchanged and mutation tooling is unsupported.
- Dispatch the ledger-selected fresh Sol High feature review after all commits and validation pass.

## Documentation impact

- `DESIGN.md` owns the planned `/academy` layout and unavailable-stat presentation.
- `ARCHITECTURE.md` records the Academy Rust module, Tauri commands, v11 tables, Planner club-family dependency, searchable picker, and delivered roster boundary.
- `CONCEPT.md` needs reconciliation only if implementation establishes Youth Academy as a durable product capability beyond its existing product purpose.
- Archive this ledger to `.wiki/features/completed/` only after full validation, feature review, and documentation reconciliation.
