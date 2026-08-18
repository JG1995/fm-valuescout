# Settings and Managed Club

## Status

Validation

## Intent

Combine Linear JAY-26 and JAY-27 into one feature PR. Move application-management controls from Dashboard to a dedicated Settings page, reduce Dashboard to a placeholder, and replace the persisted multi-club family with one save-scoped managed-club selection whose current Senior, Reserves, and Youth membership comes from the effective FM snapshot.

## User-visible behavior

- The top bar remains the only active-save selector and keeps snapshot freshness, player-cap controls, and **Load Data**.
- The navigation rail adds **Settings**. `/settings` is one page with separate Save data, Managed club, and Bridge sections.
- Save creation, rename, and deletion; snapshot overview, history, rename, and deletion; bridge status; and bridge plugin install, update, and removal move from Dashboard to Settings without changing their existing safety, focus, error, or invalidation behavior.
- Dashboard remains at `/` and in navigation, but its content is only the `Dashboard` heading and `Placeholder.` until a later feature gives it a product purpose.
- The Dashboard auto-detect CSV importer is removed. The format-specific Moneyball and Youth Academy import actions remain in Squad.
- The development sanity-player table and its `list_sanity_players` IPC path are removed. Settings keeps snapshot metadata and management, not the sample player rows or proof role score.
- Settings lets the user select exactly one managed club for the active app save. It does not expose attached Reserves, Youth, B-team, or affiliate sources.
- Squad overview, Academy candidates, My Staff, and club-wide boost cohorts use exact managed-club membership from the active save's effective current snapshot.
- Planner team candidates and optimization use the same exact managed-club rule plus FM's current-snapshot `team_level`: `senior` -> Senior, `reserve` -> Reserves, and `youth` -> Youth.
- A current managed-club player with no usable FM team level remains visible in Squad but is unavailable to team-specific Planner candidate and optimizer paths. Settings reports that FM did not classify those players instead of assigning them arbitrarily.
- If the selected club is absent from the effective current snapshot, the selection remains saved, Settings shows it as unavailable, current managed-club cohorts are empty, and retained Planner or Academy records are not deleted.
- Changing the managed club, active save, or effective current snapshot refreshes every managed-club consumer. The effective current snapshot is always authoritative.
- Existing Planner assignments remain save-scoped. A player who no longer matches the selected club or assigned FM team remains in the matrix with the existing outside-pool or unresolved warning instead of being deleted.

## Invariants

- Active-save selection and **Load Data** remain in `AppTopBar`.
- The selected managed club is save-scoped. Snapshot replacement never rewrites the selection.
- Only the effective snapshot row with `is_current = 1` supplies membership.
- Player club membership uses an exact current-snapshot `players.current_club` match. `parent_club` remains loan/contract context and does not expand the managed cohort.
- Staff membership uses an exact current-snapshot `staff.club` match.
- Team-specific membership additionally requires the one FM team-level value that maps to the requested canonical Planner category. Null or unknown values never fall back to another team.
- The WebView does not compute membership or query SQLite. Rust owns selection validation and every cohort query.
- One player can enter at most one Planner team pool because one current player row has one `team_level` value.
- Existing primary-club selections survive migration. Obsolete attached-source rows do not.
- Save and snapshot destructive confirmations keep immutable target binding, cascade copy, duplicate-submit protection, dialog-local failures, and focus restoration.
- Frontend feature modules remain isolated. Route files compose snapshot, managed-club, memory-read, Planner, Academy, Staff, and Squad features and own cross-feature cache invalidation.

## Non-goals

- Building the planned **My Club** page or moving the selector there.
- Giving Dashboard its future analytics or product purpose.
- Moving the top-bar active-save selector, freshness indicator, cap controls, or **Load Data**.
- Adding a general CSV importer to Settings or changing the Squad format-specific import contract.
- Inferring affiliates, fuzzy club-name relationships, or replacement clubs.
- Changing the bridge dump schema, FM memory extraction, snapshot-current selector, Planner team availability, team display names, tactics, strings, or assignment persistence.
- Deleting retained Planner assignments, Academy memberships, outcomes, tactics, shortlists, enrichment, snapshots, or saves when membership changes.
- Adding club logos, a multi-club hierarchy, or manual team overrides.

## Current-state map

- Relevant components: `src/app/routes/index.tsx` composes Dashboard; `src/app/components/app-nav-rail.tsx` owns primary navigation; `src/app/components/app-top-bar.tsx` owns active-save and Load Data context changes; `src/features/snapshot/components/snapshot-panels-with-error-boundary.tsx` composes save, history, and overview controls; `src/features/memory-read/components/bridge-status-panel-with-error-boundary.tsx` composes bridge install and status; `src/features/planner/components/planner-club-family-panel.tsx` owns the current multi-source Club Setup UI.
- Route and UI tests: `src/app/app-shell-routing.test.tsx`, `src/app/routes/index.test.tsx`, `src/app/routes/planner.test.tsx`, `src/app/routes/academy.test.tsx`, `src/app/routes/staff.test.tsx`, snapshot and bridge component tests, `src/testing/*-ipc-mock.ts`, `src/testing/setup.ts`, and `e2e/smoke.spec.ts` cover the affected browser contracts.
- Data model: migration v4 created `planner_club_settings(save_id, primary_club)` and `planner_club_sources`; v28 added independent `planner_teams`. `players` contains nullable `current_club`, `parent_club`, and `team_level`; `staff` contains nullable `club`.
- Persistence and migrations: `src-tauri/src/db/migrations.rs` is at v28. Save deletion cascades club settings. Snapshot replacement preserves save-scoped Planner and Academy rows.
- FM source contract: `bridge/Extraction/TeamLevelMap.cs` maps team type `0` to `senior`, `1..9` to `reserve`, and `>=10` to `youth`; `src-tauri/src/features/snapshot/ingest.rs` persists that nullable value unchanged.
- Existing behavioral assumptions: Squad, Planner, Academy, Staff, and the two club-wide boost families read `planner_club_sources`; Planner assignment resolution already distinguishes outside-pool and unresolved records without deleting assignments.
- Architectural seams: Tauri commands are registered in `src-tauri/src/lib.rs`; frontend IPC uses `src/lib/tauri-client.ts`; TanStack Query owns IPC state; app routes may compose feature modules and invalidate sibling query roots.
- Project validation commands: `./scripts/dev test [target...]`, `./scripts/dev check`, and `./scripts/dev smoke`.
- Primary risks: a membership predicate diverges across consumers; v29 loses the existing primary club or unrelated save data; Settings relocation weakens destructive-dialog behavior; stale caches survive a club/save/snapshot change; browser mocks conceal a removed native command.

## Feature architecture

- Add a Rust `features/managed_club` domain that owns the one selected club, current-snapshot availability state, exact player/staff membership inputs, and the `get_managed_club`, `list_managed_club_options`, and `set_managed_club` IPC contract.
- Migration v29 renames `planner_club_settings` to `managed_club_settings`, renames `primary_club` to `club_name`, and drops `planner_club_sources` in the same transaction. It copies no attached-source row forward.
- Add a React `features/managed-club` boundary for types, Query keys/options, the mutation, and the Settings panel. Keep canonical Planner team types under Planner rather than coupling them to the removed club-family DTO.
- Squad overview and player boosts use all current-snapshot players whose `current_club` exactly matches the selected managed club.
- Planner candidate, depth-state, and optimizer paths bind the selected club and requested FM team level. The canonical `reserves` Planner identity maps to dump value `reserve` at the Rust boundary.
- Academy candidate and retained-member state use exact managed-club membership without requiring a team level. Existing memberships retain last-known identity across departures or missing snapshots.
- My Staff queries and staff boosts use current-snapshot `staff.club` exact membership. Staff remains a whole-club view because the dump has no staff team-level field.
- Settings route composition owns managed-club mutation invalidation of managed-club, Planner/Squad, Academy, and Staff query roots. `AppTopBar` invalidates managed-club queries with other snapshot consumers after Load Data and active-save changes.
- Dashboard has no loader or product data dependencies after the relocation. Settings preloads saves, current snapshot, snapshot management, managed-club, bridge, and club-option data through the owning feature Query options.

## Uncertainty register

### Known

- Linear JAY-26 and JAY-27 are related and both are In Progress. JAY-26 explicitly leaves Dashboard Club Setup removal to the manual club-family issue. The developer selected Settings as the temporary location while resolving the combined plan's open questions.
- The top bar already owns active-save selection and Load Data and invalidates Search, Player, Planner, Academy, and Staff after context changes.
- The latest/effective snapshot is selected by the existing date/load/ID comparator and exposed through `is_current = 1`.
- The bridge maps FM team types to `senior`, `reserve`, or `youth`. Dump validation accepts any nullable string and ingest persists it unchanged, so managed-club logic must treat every other value as unclassified.
- Planner assignments and Academy memberships are save-scoped and already preserve last-known data across current-snapshot changes.
- GitHub is the publication provider, `main` is the trunk, `.github/pull_request_template.md` is required, the merge method is squash, and the strict required check is named `check`.

### Assumptions

- Exact FM club names are stable within one effective snapshot. The selector uses only values returned from that snapshot and retains an existing missing selection for recovery.
- The combined change is one compatible user-visible capability. Release intent remains provisional until the repository-local PR procedure evaluates the complete range from the latest tag.

### Decisions

- Deliver JAY-26 and JAY-27 in one PR because Settings is the destination for the managed-club selector and both issues share Dashboard, routing, context invalidation, and product copy.
- Use two implementation commits: first establish Settings and remove obsolete Dashboard surfaces; then replace the club-family contract across persistence and every downstream consumer.
- Keep Dashboard copy deliberately minimal: `Placeholder.`
- Drop `planner_club_sources` immediately in v29. Preserve only the existing primary selection.
- Treat `current_club` and `staff.club` as authoritative membership fields. Use `team_level` only for team-specific Planner membership. Do not infer from `parent_club` or fuzzy club relationships.
- Keep existing assignment and Academy retention behavior when membership changes. This is the smallest graceful behavior and avoids data loss.
- Do not create an ADR. The change replaces an obsolete feature contract using fields and current-snapshot ownership that the repository already accepts; the active ledger, migration tests, code, and final current-state reconciliation are sufficient.

### Unknowns

- None blocks the first commit. A runtime spike is not required.

### Risks

- Some representative FM snapshots may contain null `team_level` for a selected-club player. The feature must report the unclassified count and exclude those rows only from team-specific Planner pools.
- A selected club may disappear or be renamed in a later snapshot. The feature must keep the saved value, show `missing`, return empty current cohorts, and preserve historical user data.
- Renaming the settings table can break save deletion or migration fixtures if SQLite foreign-key behavior is assumed instead of tested.
- Removing old IPC commands requires synchronized native registration, browser stubs, frontend APIs, generated routes, and smoke fixtures.

## Walking skeleton

Commit 1 makes `/settings` routable from the nav rail, moves the existing save/snapshot and bridge panels there, moves the current Club Setup panel there as an interim section, and leaves a placeholder Dashboard. It proves the new route and management location before the membership contract changes.

## Delivery plan

### PR 1 — Move app management to Settings and derive managed-club membership

**Status:** Ready for publication

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/settings-managed-club`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required-check rule:** strict required check `check` must pass before merge

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(settings): add managed-club configuration`

**Purpose:** Deliver the shared Settings destination and replace all manual club-family behavior as one coherent product change. A separate PR would leave either a temporary destination without its final control or a managed-club contract without the agreed Settings surface.

**Depends on:** Current `main`, including snapshot history, Squad workspace, Staff workspace, Youth Academy, and optional Planner teams.

#### Commit 1 — Move operational controls to Settings

**Status:** Completed

**Provisional commit:** `feat(settings): move app management from Dashboard`

**Work:** Add the Settings route and navigation item, relocate existing management panels, reduce Dashboard to a placeholder, remove the Dashboard CSV importer, and retire the sanity-player query path.

**Out of scope:**

- Changing the existing Club Setup data model, commands, sources, or membership rules.
- Changing save/snapshot/bridge persistence or mutation semantics.
- Changing Squad's format-specific CSV import actions or the Rust CSV parser/import service.
- Implementing future Dashboard content.

**Implementation packet:**

- Create a production Settings route with one vertical page split into Save data, Club setup, and Bridge sections. Reuse the current feature components and established panel geometry rather than redesigning them.
- Keep route-level cross-feature invalidation exactly where Settings composes the moved controls. A save/current-snapshot mutation must still invalidate Search, Player, Planner/Squad, Academy, and Staff.
- Remove Dashboard's snapshot, Club Setup, CSV, and bridge composition. Render only its heading and `Placeholder.`.
- Remove the auto-detect `CsvImportPanel` component once no caller remains. Keep shared `CsvImportOutcome`, `useCsvImport`, format-specific Squad modals, and Rust `import_csv` behavior.
- Remove the sanity list from `SnapshotOverviewPanel`, then remove its frontend Query option/fetcher/type, test IPC handler, browser stub, Rust query/command/DTO, and Tauri registration. Keep current snapshot metadata, incomplete-scan warning, and history.
- Add Settings to the nav rail without exceeding the six-item design limit. Update generated TanStack route output through the normal toolchain; do not hand-edit generated routing semantics.

**Files and responsibilities:**

- `src/app/routes/settings.tsx` — new Settings loader, section composition, Suspense fallbacks, and route-owned invalidation callbacks.
- `src/app/routes/index.tsx` — placeholder-only Dashboard.
- `src/app/components/app-nav-rail.tsx` — Settings navigation item and icon.
- `src/app/app-shell-routing.test.tsx`, `src/app/routes/index.test.tsx`, new `src/app/routes/settings.test.tsx`, and `e2e/smoke.spec.ts` — route, navigation, relocation, placeholder, and browser-history proof.
- `src/features/snapshot/components/snapshot-overview-panel.tsx` and snapshot panel tests — metadata-only overview and preserved empty/truncated states.
- `src/features/snapshot/api/sanity-players-query-options.ts`, `fetch-sanity-players.ts`, `types/player-sanity.ts`, `src-tauri/src/features/snapshot/query.rs`, `commands.rs`, `src-tauri/src/lib.rs`, `src/testing/setup.ts`, snapshot mocks, and `e2e/tauri-ipc-stub.ts` — delete only the now-unused sanity-list contract and test paths.
- `src-tauri/src/features/snapshot/ingest.rs` — replace the direct sanity-query assertion with an enduring direct proof that player visibility follows the effective current snapshot.
- `src/features/csv-import/components/csv-import-panel.tsx` and Dashboard CSV tests/mocks — remove the auto-detect Dashboard surface while retaining format-specific Squad import support.
- `src/features/memory-read/components/bridge-status-panel.tsx` — update location-specific copy only if it still says the install section is `above` after the Settings section layout is composed.
- `src/routeTree.gen.ts` — generated Settings route registration, updated by the repository toolchain.

**Behavior and data flow:**

- Nav **Settings** -> `/settings` loader -> owning Query options -> Settings sections render independently so one failed panel does not blank the page.
- Save/snapshot mutation -> existing snapshot component callback -> Settings route invalidates current-only sibling feature roots.
- Bridge install/status mutations remain inside memory-read feature components and retain platform-specific unsupported/error states.
- Dashboard navigation -> `/` -> heading plus static placeholder, with no IPC prefetch.
- Squad CSV actions continue to call `import_csv` with an explicit expected format. No Settings or Dashboard action calls the auto-detect flow.

**Ordered implementation steps:**

1. Add RED route and shell tests for a Settings nav link, direct `/settings` refresh, moved management regions, and placeholder-only Dashboard.
2. Add the Settings route and move existing snapshot, interim Club Setup, and bridge composition without changing their behavior.
3. Remove Dashboard CSV composition and prove both Squad upload actions remain available.
4. Add RED proof that snapshot metadata renders without a sanity table or `list_sanity_players` call, remove that end-to-end IPC path, and replace the ingest regression's direct call with a current-snapshot visibility proof.
5. Update browser stubs, route generation, smoke coverage, and location-specific copy.
6. Run formatting, focused tests, the full gate, and smoke while the focused proofs remain green.

**Tests and proof:**

- RED: `/settings` is not routable or linked; Dashboard still owns operational panels and the CSV importer; snapshot overview calls `list_sanity_players`.
- GREEN: direct and in-app Settings navigation renders Save data, Club setup, and Bridge sections; Back/Forward retain the route; Dashboard renders no operational regions or import action; current snapshot metadata and incomplete-scan warning render without a player table.
- Preserve tests for save/snapshot destructive confirmation, focus restoration, pending protection, unsupported bridge hosts, panel-local retries, current-context invalidation, and Squad CSV imports.
- Preserve the ingest regression that proves an earlier retained snapshot cannot replace the later effective snapshot's visible players.
- Smoke must cover Settings navigation and the moved controls with browser IPC stubs. It must stop expecting Dashboard Club Setup or Dashboard CSV import.

**Patterns to verify:**

- `src/app/routes/academy.tsx` and `src/app/routes/staff.tsx` for file-route loaders, Suspense, and independent product sections.
- Current `src/app/routes/index.tsx` for route-owned sibling query invalidation.
- `SnapshotPanelsWithErrorBoundary` and `BridgeStatusPanelWithErrorBoundary` for panel-local errors and retries.
- `AppNavRail` active-link and collapsed-label behavior.

**Constraints and non-goals:**

- Keep one primary action per section and existing design tokens; do not introduce new primitives or visual language.
- Preserve accessible headings, landmark structure, dialog focus, keyboard navigation, and all unsupported/loading/empty/error states.
- Do not move active-save selection or Load Data from the top bar.
- Do not alter the club-family contract in this commit; reviewers must treat the interim Club setup section as intentional pending Commit 2.

**Dependencies and sequencing:**

- No prior feature PR. This is the walking skeleton and must clear before Commit 2 changes the cross-layer club contract.
- The generated route tree must match the new file route before validation.

**Validation:**

- `./scripts/dev format src/app src/features/snapshot src/features/memory-read src/features/csv-import src/testing e2e`
- `./scripts/dev test src/app/app-shell-routing.test.tsx src/app/routes/index.test.tsx src/app/routes/settings.test.tsx src/features/snapshot/components/snapshot-panels.test.tsx src/features/memory-read/components/bridge-status-panel.test.tsx src/features/csv-import/components/squad-csv-import-modal.test.tsx`
- `./scripts/dev check`
- `./scripts/dev smoke`

**Stop conditions:** Stop and replan if the moved snapshot controls require frontend feature-to-feature imports; if route refresh cannot preserve `/settings`; if removing sanity IPC also removes metadata required by another consumer; if Squad imports depend on `CsvImportPanel`; or if destructive target/focus behavior changes after relocation.

**Review mandate:** Verify direct route and Back/Forward behavior; Dashboard contains no removed surfaces; top-bar controls stay unchanged; destructive dialogs and cache invalidation survive relocation; sanity IPC is fully removed without deleting snapshot metadata; format-specific Squad CSV import remains intact; Settings sections keep accessible headings and independent failures.

#### Commit 2 — Replace club-family configuration with managed-club membership

**Status:** Completed

**Provisional commit:** `feat(club): derive managed membership from FM data`

**Work:** Migrate to one managed club, simplify the Settings control, derive all current cohorts from exact current-snapshot club and team-level data, and remove attached-source types, IPC, persistence, recovery links, and copy.

**Out of scope:**

- Bridge or dump-schema changes.
- Fuzzy club matching, affiliate discovery, or manual source overrides.
- Planner team availability/display-name changes.
- My Club page work, Dashboard content, or deletion of retained assignments and Academy history.

**Implementation packet:**

- Add migration v29. Rename `planner_club_settings` to `managed_club_settings`, rename `primary_club` to `club_name`, and drop `planner_club_sources` transactionally. Preserve every existing primary selection and prove unrelated save-owned data remains unchanged.
- Add a `features/managed_club` Rust module. It owns selected-club reads/writes, current-snapshot availability, club options, unclassified-player counts, validation, and commands. Remove club-family DTOs and commands from Planner.
- Add `features/managed-club` React types, Query keys/options, mutation, and Settings panel. The panel keeps the existing searchable exact-club picker but removes every source editor and uses **Managed club** copy.
- Move `PlannerTeam` and its fixed constants from removed club-family frontend types to a Planner-owned team type file. Update direct imports without adding a compatibility barrel.
- Replace every `planner_club_sources` query in Squad, Planner depth/candidates, optimizer, Academy, Staff, and player/staff club-wide boost paths.
- Use exact `current_club = managed club` for player-wide cohorts, exact `staff.club = managed club` for staff, and exact player club plus mapped `team_level` for team-specific Planner paths.
- Return an explicit managed-club status. `unconfigured` has no selection; `available` has a current exact club match; `missing` retains a selection not represented in the effective snapshot. Include a bounded `unclassifiedPlayerCount` for exact-club players whose team level cannot map to Planner.
- Keep a missing selection in the picker and show warning copy. Saving a new club requires an exact current option; re-saving the unchanged missing selection remains allowed only as a no-op. Consumer queries return empty current cohorts when missing.
- Preserve existing Planner assignment and Academy membership rows. Existing state readers must show outside-pool or unresolved warnings after a club, save, or snapshot change.
- Change every recovery link and message from Dashboard Club Setup/club family to Settings Managed club at `/settings#managed-club`.
- Add `managedClubKeys.all` invalidation to AppTopBar Load Data and active-save transitions. A successful Settings mutation invalidates managed-club, Planner/Squad, Academy, and Staff roots.

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs` — v29 schema migration and preservation/drop tests.
- `src-tauri/src/features/managed_club/{mod.rs,service.rs,commands.rs}` and `src-tauri/src/features/mod.rs` — shared managed-club persistence, status, validation, current-snapshot context, and IPC.
- `src-tauri/src/lib.rs` — register new commands and remove old Planner club-family commands.
- `src-tauri/src/features/planner/{commands.rs,service.rs,squad.rs,depth.rs,optimizer.rs,*tests.rs,test_support.rs}` — remove club-family ownership and bind exact club/team-level eligibility.
- `src-tauri/src/features/academy/service.rs` — exact managed-club candidate and retained-member state.
- `src-tauri/src/features/staff/{query.rs,commands.rs}` — exact managed staff state, pages, and bulk cohort.
- `src-tauri/src/features/player/commands.rs` — exact managed player cohort for both squad-wide actions and Settings recovery copy.
- `src/features/managed-club/**` — typed IPC, Query state, simplified searchable Settings panel, missing/unclassified states, and focused component tests.
- `src/features/planner/types/team.ts` and Planner API/component imports — canonical team identity after deleting `types/club-family.ts`.
- `src/app/routes/settings.tsx`, `planner.tsx`, `academy.tsx`, Staff/Squad/Academy components, and route tests — compose the new state, update headings/context/recovery links, and preserve current workspaces.
- `src/app/components/app-top-bar.tsx` — managed-club invalidation on current-context changes.
- `src/testing/planner-ipc-mock.ts`, a managed-club IPC mock, `src/testing/setup.ts`, squad/staff/academy mocks, and `e2e/tauri-ipc-stub.ts` — remove source fields and mirror native status/membership behavior.
- `.wiki/ARCHITECTURE.md`, `.wiki/DESIGN.md`, `.wiki/CONCEPT.md`, affected completed records/ADRs, and this ledger — update only when implementation makes the new current state true; final broad reconciliation remains feature-close-out work.

**Behavior and data flow:**

- Settings picker -> `set_managed_club` -> active save's `managed_club_settings` row -> returned status -> invalidate managed-club and all membership consumers.
- Load Data or active-save switch -> existing snapshot transition -> invalidate managed-club status and consumer roots -> every read resolves the new effective snapshot.
- Squad/Academy/player boost -> resolve active save, effective current snapshot, and managed club -> exact player `current_club` membership -> current rows only.
- Planner picker/optimizer/depth state -> same context -> exact club plus mapped FM `team_level` -> canonical team pool without duplicates.
- My Staff/staff boost -> same context -> exact `staff.club` membership -> whole-club current staff cohort.
- Missing selection -> Settings status warning; no guessed replacement; current cohorts empty; persisted Planner and Academy records remain.

**Ordered implementation steps:**

1. Add RED migration tests for primary selection preservation, attached-source removal, renamed schema, cascade behavior, and unrelated save-owned data.
2. Add RED managed-club service tests for exact options, save isolation, current-snapshot authority, missing state, unclassified count, and bounded validation.
3. Implement v29 and the Rust managed-club service/commands; keep the gate green before switching consumers.
4. Add RED consumer tests for exact player/staff membership, one-to-one team-level mapping, null-level exclusion from Planner only, snapshot/save refresh, missing club, and retained outside-pool assignments/memberships.
5. Replace each backend consumer and delete `planner_club_sources` reads, old Planner club-family services, DTOs, command registration, and obsolete test helpers.
6. Add RED Settings panel and route tests for one selector, missing/unclassified warnings, no attached-source controls, and complete consumer invalidation.
7. Replace frontend APIs/types/mocks/copy/recovery links, move the canonical Planner team type, and remove obsolete source fields and commands.
8. Update implementation-intrinsic current-state docs, run formatting, focused route and Rust proofs, the full gate, and smoke.

**Tests and proof:**

- Migration: a v28 database with two saves, a primary club, attached sources, snapshots, tactics, assignments, Academy history, shortlist, and enrichment upgrades to v29 with the primary club preserved, sources absent, and every unrelated row unchanged.
- Managed service: club options come only from the effective current snapshot; an earlier retained snapshot cannot add options or members; save switching isolates selections; invalid names and stale new options fail safely.
- Membership: exact club rows enter Squad/Academy/player boosts; exact staff club rows enter My Staff/staff boosts; another club and an earlier snapshot are excluded.
- Planner: `senior`, `reserve`, and `youth` map to exactly one canonical category; null/unknown team level is not guessed; optimizer, picker, and assignment state use the same rule.
- Lifecycle: changing the managed club, active save, or effective snapshot refreshes all consumers; late results do not restore stale status.
- Retention: a transferred, missing, or differently classified assigned player stays in its Planner cell with the existing warning; Academy membership and manual outcomes remain.
- UI: Settings exposes one managed-club combobox and no source buttons/selects; missing and unclassified states are textual and accessible; all recovery links target `/settings#managed-club`.

**Patterns to verify:**

- `src-tauri/src/features/snapshot/service.rs` migration/lifecycle tests for preserving save-scoped children through snapshot changes and cascading them on save deletion.
- `src-tauri/src/features/planner/teams.rs` for a save-scoped setting with canonical values and complete validation.
- Existing `planner::depth` outside-pool and unresolved assignment states.
- Existing Academy retained-member states and Staff `NoClubFamily` empty-state contract, renamed without weakening behavior.
- Existing primary-club searchable combobox keyboard behavior in `planner-club-family-panel.tsx`.

**Constraints and non-goals:**

- No frontend SQL or membership recomputation; no new dependency.
- Use parameterized exact-name queries. Do not build SQL from club names or accept client-supplied cohorts.
- Preserve null honesty. A missing team level is unavailable, not Senior and not zero.
- Keep all process-memory write cohorts Rust-derived and frozen under their existing gates.
- Do not delete user-owned assignments, Academy history, tactics, strings, shortlists, enrichment, saves, or snapshots.
- Historical completed records and ADRs explain old behavior until final reconciliation; do not rewrite implementation history as if it never existed.

**Dependencies and sequencing:**

- Depends on Commit 1 because Settings is the agreed selector location and recovery target.
- Migration, native commands, frontend API, mocks, and recovery links ship in this one commit so no checked-out commit exposes a dead command or misleading source editor.

**Validation:**

- `./scripts/dev format src/app src/features/managed-club src/features/planner src/features/squad src/features/academy src/features/staff src/testing e2e`
- `./scripts/dev test src/app/routes/settings.test.tsx src/app/routes/planner.test.tsx src/app/routes/academy.test.tsx src/app/routes/staff.test.tsx src/app/components/app-top-bar.test.tsx`
- `./scripts/dev check`
- `./scripts/dev smoke`

**Stop conditions:** Stop and replan if current FM data cannot distinguish the three canonical `team_level` values already accepted by the dump contract; if an existing database cannot preserve the primary selection and unrelated rows while dropping sources; if any consumer still needs attached sources for a confirmed product path; if a cohort would need `parent_club` inference or fuzzy relationships; if assignment retention requires a schema change; or if consumer predicates cannot share the exact club/team-level contract without duplicating business rules inconsistently.

**Review mandate:** Verify v29 data preservation and source removal; exact current-snapshot authority; identical membership across Squad, Planner, optimizer, Academy, Staff, and both boost families; no WebView-selected cohorts; null/missing state honesty; assignment/Academy retention; complete old IPC/type/copy removal; cache invalidation across club/save/snapshot changes; and no regression in Planner team availability or boost recovery gates.

## Active work

No implementation commit is active. PR 1 is ready for publication and the feature awaits feature-complete validation and documentation reconciliation.

## Discoveries and replanning

- Linear readback on 2026-08-18 confirmed JAY-26 and JAY-27 are related, In Progress, and have no additional comments or blockers.
- Repository inspection confirmed Repowise is synchronized to `ad5c12ff386274057dd2f06b2f03e4adcbe9dbfb`. Its broad synthesized answers had weak retrieval quality, so this plan relies on direct source, tests, current-state docs, and Git evidence for file-level contracts.
- The dump already persists the required nullable `team_level`; no bridge change or runtime spike is needed.
- Commit 2 review found and corrected a nullable retained-assignment membership decode, a late managed-club mutation cache race, loss of the searchable picker contract, and stale current-state documentation. One correction round cleared all findings.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Commit 1 — Move operational controls to Settings | Pending record | Settings route and navigation; placeholder Dashboard; relocated management panels; Dashboard importer and sanity IPC removed | Sol Medium accepted after one documentation correction round | None |
| PR 1 | Commit 2 — Replace club-family configuration with managed-club membership | Pending record | Migration v29; one managed-club selector; exact effective-snapshot membership across Squad, Planner, Academy, Staff, and boost cohorts; obsolete club-family persistence, IPC, types, and copy removed | Sol Medium accepted after one correction round; full gate and 44-test smoke suite passed | None |

## Final validation

- `./scripts/dev format`
- `./scripts/dev test`
- `./scripts/dev check`
- `./scripts/dev smoke`
- Migration v28 -> v29 preservation proof with populated save, snapshot, Planner, Academy, shortlist, and enrichment state.
- Fresh feature-complete review of the exact two-commit implementation set.
- Native Tauri route, focus, unsupported-platform bridge, and minimum-window checks when the desktop environment is available; report any unavailable native proof explicitly.
- Documentation reconciliation for current Settings, managed-club architecture, design/recovery copy, affected historical-decision implications, TODO state, and completed feature record.

## Documentation impact

Complete during implementation and feature reconciliation. Expected current-state owners are `.wiki/CONCEPT.md`, `.wiki/ARCHITECTURE.md`, and `.wiki/DESIGN.md`. Update affected ADR consequences only where the accepted club-family premise is superseded; preserve their historical rationale. No new ADR or debug report is planned.
