# My Club Workspace

## Status

Active

## Intent

Implement Linear JAY-25 by giving every managed-club player and staff workflow one **My Club** entry without changing the data, persistence, scoring, or mutation owners behind those workflows. The feature also promotes the managed-club selector out of Settings and applies the single-club, current-snapshot membership contract delivered by JAY-27.

## User-visible behavior

- The primary navigation contains **Player Search**, **Staff Search**, and **My Club** as distinct destinations. The existing `/search` and `/staff` paths remain stable.
- **My Club** opens `/my-club` on **Squad** by default.
- The My Club workspace exposes five flat, URL-backed tabs in this order: **Squad**, **Planner**, **Tactic**, **Staff**, and **Staff Shortlist**.
- **Squad**, **Planner**, and **Tactic** preserve their existing table, CSV import, boost, depth-planning, optimizer, tactic-draft, focus, and URL behavior.
- **Staff** is the current managed-club staff table formerly labelled **My Staff**. **Staff Shortlist** preserves its save-owned import, filters, contextual score columns, and current-snapshot staff join.
- The managed-club selector appears in the My Club header for a loaded snapshot. It retains explicit save behavior, exact current-snapshot options, the saved-missing-club warning, and downstream query invalidation.
- Staff Search remains a standalone recruitment page at `/staff`; Player Search remains at `/search`; staff and player profile routes remain unchanged.
- Legacy `/planner`, `/staff?view=my-staff`, `/staff?view=shortlist`, and `/settings#managed-club` links replace themselves with the equivalent My Club URL and preserve applicable sort and shortlist filter state.
- Missing snapshot and managed-club states keep direct recovery guidance. The selector and five-tab header must not create nested page scrolling or reduce the supported 1280×800 data workspace below its existing usable bounds.

## Invariants

- One save-scoped managed club remains the only club selection. The effective current snapshot remains authoritative, and membership stays an exact current-club match.
- The feature must not restore attached clubs, club-family persistence, fuzzy club matching, or `team_level`-based eligibility.
- Rust remains the owner of managed-club validation, Squad and Staff membership, Planner state and scoring, shortlist persistence and joins, boost policy, and every mutation contract.
- React route composition may move, but feature query keys, IPC command names, DTOs, database tables, migrations, and scoring logic must not change.
- Staff Shortlist moves only in information architecture. It remains a save-owned recruitment list joined to current-snapshot staff and must not become managed-club filtered.
- Internal staff scope and IPC names may retain `my-staff`; only user-facing My Club copy becomes **Staff** or **managed-club staff**.
- Planner and Tactic stay mounted while the user changes My Club tabs so unsaved tactic drafts and current Planner interaction state survive the same transitions they survive today.
- Workspace changes replace URL state instead of adding browser-history entries. Opening a profile pushes a new entry, and browser Back restores the originating My Club URL and table state.
- `/staff/$uid` must remain a profile route. A legacy `/staff` redirect must never capture a profile path.
- Active save and snapshot provenance remain visible in the global top bar on every data view.
- Cross-feature composition remains limited to `src/app/routes/`. Product components stay in their owning feature, and `src/app/components/` remains app-shell-only.

## Non-goals

- Changing managed-club persistence, cohort derivation, Planner eligibility, staff shortlist scope, role scoring, or boost behavior.
- Moving Youth Academy, Dashboard, Player Search, Staff Search, or profile routes under My Club.
- Adding a new nested navigation hierarchy, dashboard summary, club crest, staff or player filters, comparison view, or mobile layout.
- Renaming backend commands, query scopes, database fields, or stored table-layout identifiers only to match visible copy.
- Adding a dependency, migration, IPC command, capability, or ADR.
- Removing compatibility routes after the redirects ship.

## Current-state map

- Relevant components: `src/app/routes/planner.tsx` composes Squad, Planner, and Tactic; `src/app/routes/staff.tsx` composes Search, My Staff, Shortlist, and the `/staff/$uid` outlet; `src/app/routes/settings.tsx` composes the managed-club panel; `src/app/components/app-nav-rail.tsx` links Search, Staff, and Squad as separate top-level items.
- Relevant feature UI: `src/features/managed-club/components/managed-club-panel.tsx`, `src/features/planner/components/planner-workspace-tabs.tsx`, `src/features/staff/components/staff-workspace-tabs.tsx`, `src/features/staff/components/staff-search-results-panel.tsx`, `src/features/squad/components/squad-overview-panel.tsx`, `src/features/planner/components/planner-depth-matrix.tsx`, and `src/features/planner/components/planner-tactic-editor.tsx`.
- Route state: `/planner` owns `view`, Squad `sort`, and Squad `dir`; `/staff` owns three independent table-sort pairs plus Search filters and Shortlist filters; both routes replace search state when workspace controls change.
- Recovery links: Squad, My Staff, and Youth Academy currently target `/settings#managed-club`; Settings owns the selector and its downstream invalidation callback.
- Data model: migration v29 stores one optional exact `managed_club_settings.club_name` per save. Squad, Planner, Academy, and My Staff read exact current-snapshot cohorts. Staff Shortlist remains in save-owned `staff_shortlist_entries` and joins current staff by UID.
- Persistence and migrations: no persistence or migration change is required. Existing save, snapshot, tactic, string, assignment, table-layout, shortlist, and club-selection state remains in place.
- Existing behavioral assumptions: `/planner` defaults to Squad and keeps Planner and Tactic mounted; the Squad table and staff tables keep independent Zustand layouts; staff profile links push `/staff/$uid`; the Staff Shortlist modal suppresses stale context results.
- Architectural seams: TanStack Router owns validated shareable state and compatibility redirects; TanStack Query owns IPC cache and invalidation; route files compose features; Rust and SQLite remain unchanged.
- Project validation commands: `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev smoke`. `./scripts/dev mutate` is unsupported and must not be reported as passed.
- Primary risks: `planner.tsx`, `staff.tsx`, `staff-search-results-panel.tsx`, and `e2e/smoke.spec.ts` are high-churn surfaces; route-state collisions, accidental unmounting, stale query invalidation, profile-route interception, and header height at 1280×800 are the main regression paths.

## Feature architecture

`src/app/routes/my-club.tsx` becomes the sole My Club route, URL-state owner, and cross-feature composition seam. It validates a compact `MyClubSearch` contract with an optional workspace plus separate optional Squad, Staff, and Staff Shortlist sort pairs. It derives defaults without serializing every inactive workspace default, replaces search state on workspace and sort changes, and directly composes the existing feature components as thin route wiring.

`MyClubWorkspaceTabs` lives in `src/features/my-club/components/`, owns the five accessible tabs and keyboard traversal, and imports no sibling feature. The route composes the existing Squad, Planner, Tactic, managed-club, My Staff, and Shortlist feature components while preserving their query, mutation, hidden-panel, and focus behavior. No route-specific product component moves into `src/app/components/`, and no feature imports another feature.

The managed-club feature keeps its existing Query and mutation adapters. Its Settings-oriented panel becomes a compact `ManagedClubSelector` in the My Club header with the stable `managed-club` anchor. On save, the route invalidates managed-club, Planner (including Squad), Staff, and Academy state through their established query roots. Settings continues to invalidate managed-club state when save or effective-snapshot management changes, but it no longer renders the selector.

Compatibility routes use TanStack Router redirects with `replace: true`. `/planner` maps its validated workspace and Squad sort state to `/my-club`. The exact `/staff` route maps legacy My Staff and Shortlist state to the new My Club fields, while `/staff/$uid` continues through the parent outlet. `/settings#managed-club` maps to the My Club selector. No redirect changes a profile path or discards applicable shortlist filters.

## Uncertainty register

### Known

- Linear JAY-25 requires one My Club entry, separated Squad, staff, Planner, and Tactic views, reliable URL and profile navigation, shared selection, and unchanged data/scoring ownership.
- Linear JAY-27 is complete on current `main`. The old configured club family no longer exists; one save-scoped managed club supplies exact current-snapshot cohorts.
- The current repository already has every required data query, mutation, table, modal, empty state, and profile route. This feature is a frontend information-architecture and composition change.
- The repository remote is GitHub, the trunk is `main`, human-authored PRs use `.github/pull_request_template.md`, the merge method is squash, and strict required status `check` gates merge.

### Assumptions

- Youth Academy remains a separate top-level destination because the developer named only Squad, Planner, Tactic, My Staff, and Staff Shortlist for My Club.
- Dashboard remains a top-level placeholder. Replacing or removing it is outside JAY-25.
- A flat five-tab control is sufficient at the supported desktop widths; no nested Players/Staff grouping is needed.
- Native Tauri visual and focus validation may remain unavailable in the current environment. Chromium smoke must still prove the supported viewport and navigation contracts, and any native gap must be reported rather than treated as passed.

### Decisions

- Use `/my-club` as the canonical route and `feature/my-club-workspace` as the short-lived branch.
- Use top-level labels **Player Search**, **Staff Search**, and **My Club**. Inside My Club, use **Squad**, **Planner**, **Tactic**, **Staff**, and **Staff Shortlist**.
- Keep `squad`, `planner`, and `tactic` as their existing URL values. Add `staff` and `staff-shortlist` as My Club URL values. Keep internal staff query scope `my-staff` unchanged.
- Keep the managed-club selector in the page header, with explicit save and the existing missing-club warning. Do not auto-save a suggestion selection.
- Preserve old paths with replace redirects instead of removing them or duplicating page implementations.
- Use one PR. The route, selector, staff relocation, compatibility tests, and documentation share one review surface and have no risky foundation that benefits from an earlier trunk merge.
- Do not create an ADR. The feature changes navigation composition within accepted Router, Query, app-layer, Rust, and persistence boundaries.

### Unknowns

- The final five-tab header and compact selector need automated 1280×800 and 1600×900 layout evidence. This is a validation question, not a blocking product decision.
- Native Tauri/WebView focus restoration and visual fit require a desktop runtime when available. Chromium route and accessibility tests remain the deterministic gate in this environment.

### Risks

- Mapping several existing URL contracts into one route can overwrite an inactive workspace's sort state or lose a legacy deep link.
- Moving route composition can unmount the tactic editor and discard an unsaved local draft.
- A broad parent `/staff` redirect can intercept `/staff/$uid` and break profiles.
- Saving the managed club from a route that is actively displaying its consumers can leave stale Squad, Planner, Academy, or Staff queries unless every established root is invalidated.
- The selector and longer tab labels can reduce table height or create page-level scrolling at the minimum viewport.
- A visible rename can accidentally trigger backend, query-key, or stored-layout renames that provide no user value and increase migration risk.
- Mechanical changes to the large Planner and smoke suites can hide a lost assertion unless focused route behavior is reviewed before broad path replacement.

## Walking skeleton

Commit 1 creates canonical `/my-club`, moves the existing Squad, Planner, and Tactic composition there, makes Squad the default, updates primary navigation, and redirects `/planner` while preserving its workspace and sort state. This proves the new route, app-layer composition, URL contract, full-height Squad table, hidden-mounted Planner/Tactic behavior, profile Back behavior, and legacy compatibility before selection and staff surfaces move.

## Delivery plan

### PR 1 — Unify managed-club workspaces

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/my-club-workspace`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required check rule:** strict `check` must pass and be up to date

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(club): unify managed club workspaces`

**Purpose:** Deliver the complete JAY-25 information architecture in one reviewable frontend PR while preserving all existing domain and persistence contracts.

**Merge to trunk when:** All three commits are complete, full validation and feature close-out clear, the PR template records release intent, and strict `check` passes.

**Depends on:** Current `main` at or after the merged JAY-27 managed-club implementation (`1430f5d` locally when planned).

#### Commit 1 — Move squad planning into My Club

**Status:** Completed

**Provisional commit:** `feat(club): move squad planning into My Club`

**Work:** Establish canonical My Club navigation by moving the existing Squad, Planner, and Tactic route composition to `/my-club` and preserving `/planner` as a compatibility redirect.

**Out of scope:**

- Moving the managed-club selector out of Settings.
- Moving Staff or Staff Shortlist out of `/staff`.
- Renaming Player Search or Staff Search.
- Changing any query, IPC, Rust, SQLite, Planner, Squad, scoring, import, or boost contract.

**Implementation packet:**

- Create the My Club route and feature-owned workspace tabs. Keep URL normalization and all cross-feature composition in the route seam; move existing behavior without forking or duplicating its domain logic.
- Keep the initial tab set limited to Squad, Planner, and Tactic for this commit. Commit 3 extends the same tab component and URL union with Staff surfaces.
- Make `/my-club` with no valid `view` render Squad. Preserve `squad`, `planner`, and `tactic` URL values, CA-descending Squad defaults, replace navigation, hidden mounted Planner/Tactic panels, and profile push/Back behavior.
- Replace the Squad nav item with My Club. Keep Dashboard, Search, Staff, Youth Academy, and Settings unchanged in this commit.
- Convert `/planner` into a replace redirect that maps valid workspace, sort, and direction to the new names and maps retired or invalid workspace values to Squad.
- Update current-state architecture and design text only for behavior made true by this commit.

**Files and responsibilities:**

- `src/app/routes/my-club.tsx` — canonical route, validated My Club search state, pure view and per-workspace sort normalization, legacy Planner mapping, loader dependencies, default selection, and direct composition of the existing Squad boost/import/table, Planner depth, and Tactic editor components with current query and mutation ownership preserved.
- `src/features/my-club/components/my-club-workspace-tabs.tsx` — accessible tablist, visible labels, roving focus, and labelled panel identifiers; it must not import a sibling feature.
- `src/app/routes/planner.tsx` — typed legacy redirect only; no duplicate page implementation.
- `src/app/components/app-nav-rail.tsx` — replace Squad destination and label with My Club while keeping six top-level items.
- `src/app/routes/my-club-squad.test.tsx` — migrated Planner route behavior under canonical My Club URLs; retain the existing focused assertions rather than broadly rewriting them.
- `src/app/routes/legacy-club-routes.test.tsx` — direct legacy Planner redirect, invalid-view fallback, sort mapping, and replacement-history proof.
- `src/app/app-shell-routing.test.tsx` — nav label, href, active state, and route navigation.
- `src/routeTree.gen.ts` — generated route registration; regenerate through the configured TanStack Router plugin and never edit by hand.
- `e2e/smoke.spec.ts` — canonical My Club paths and headings for existing Squad, Planner, and Tactic flows plus one bounded legacy `/planner` redirect assertion.
- `.wiki/ARCHITECTURE.md` and `.wiki/DESIGN.md` — current canonical route, nav entry, and player-workspace composition after the move.

**Behavior and data flow:**

- App navigation or a direct `/my-club` URL enters the new route. The route validates `view`, `squadSort`, and `squadDir`, preloads the same snapshot, managed-club, tactic, depth, and first Squad page queries, then renders the moved composition.
- Workspace actions replace only My Club search state. Squad sorting changes only its dedicated keys. Existing Query keys and Zustand table layout stay unchanged.
- Planner and Tactic panels remain mounted and hidden when inactive. Their local drafts, selection, focus, queries, and mutations continue unchanged.
- A legacy `/planner` request validates the old URL, maps it once, and throws a replace redirect to `/my-club`; no legacy component or loader performs duplicate work.
- Row activation still pushes `/players/$uid`. Browser Back restores the exact My Club workspace and Squad sort state.

**Ordered implementation steps:**

1. Add failing canonical-route, default-workspace, nav, legacy redirect, and profile-Back tests.
2. Introduce the pure My Club URL contract and accessible tab component by adapting the current Planner workspace patterns.
3. Move the current Planner route composition directly into the My Club route without changing feature-owned logic or query keys.
4. Add `/my-club`, regenerate the route tree through the configured plugin, and point the nav entry at it.
5. Reduce `/planner` to the typed replace redirect and prove valid, invalid, and retired workspace mappings.
6. Update the affected smoke paths and current-state architecture/design text.
7. Refactor only after the focused suite is green; compare the result with the former Planner route and tab implementation for preserved behavior.

**Tests and proof:**

- RED: direct `/my-club` and the My Club nav link are absent; legacy `/planner` does not canonicalize; the new default/tab/sort assertions fail for the expected missing route.
- GREEN: My Club opens Squad by default, explicit Planner/Tactic views win, tab keys use replace history, the three workspaces retain their existing state and tests, and `/planner` lands on the equivalent canonical URL.
- Boundary coverage: invalid and retired legacy views default to Squad; custom Squad sort survives redirect; profile activation and Back restore `/my-club`; no-snapshot guidance remains visible.
- Existing 4,000-line Planner route suite is migrated with focused path and heading edits. Do not weaken tactic, matrix, optimizer, table, boost, CSV, focus, or viewport assertions to make the move pass.

**Patterns to verify:**

- `src/app/routes/planner.tsx` for existing loaders, Squad sort normalization, hidden mounted panels, boosts, imports, and profile navigation.
- `src/features/planner/components/planner-workspace-tabs.tsx` for keyboard tabs and panel relationships; replace it only after the feature-owned My Club equivalent proves parity.
- `src/app/routes/academy.tsx` for validated URL-backed workspace state that keeps route files and feature ownership separate.
- Official TanStack Router `beforeLoad`/`redirect` behavior for typed search mapping and `replace: true`.

**Constraints and non-goals:**

- Keep cross-feature imports in `src/app/routes/my-club.tsx`; keep `src/app/components/` app-shell-only and prevent `src/features/my-club/` from importing sibling features.
- Do not hand-edit `routeTree.gen.ts`, add a dependency, or move feature code merely to satisfy a new folder aesthetic.
- Do not unmount Planner or Tactic on workspace change.
- Do not change current visible Squad, Planner, or Tactic behavior beyond the My Club heading, route, and tab shell.
- Preserve all recovery, stale-context, partial-failure, accessibility, and data-honesty contracts in the moved code.

**Dependencies and sequencing:**

- Requires current JAY-27 code and its one-managed-club query contract.
- Establishes `/my-club`, route-owned URL and composition wiring, the feature-owned tab component, and test files used by commits 2 and 3.
- Must clear before the selector or staff moves so later commits extend one canonical shell instead of creating parallel pages.

**Validation:** `./scripts/dev test src/app/routes/my-club-squad.test.tsx src/app/routes/legacy-club-routes.test.tsx src/app/app-shell-routing.test.tsx`; `./scripts/dev check`; `./scripts/dev smoke`.

**Stop conditions:** Stop and replan if the move requires a Rust, IPC, schema, query-key, or stored-layout change; if TanStack Router cannot preserve old Planner state with one typed replace redirect; if Planner/Tactic drafts unmount; if composition requires a cross-feature import outside `src/app/routes/`; or if existing Planner behavior can pass only by weakening a regression test.

**Review mandate:**

- Verify canonical and legacy URL parsing, replacement history, and invalid-value normalization.
- Verify the Tactic editor and Planner depth stay mounted and retain local state across every initial tab transition.
- Verify Squad sort, virtual paging, profile Back, CSV, boosts, optimizer, and focus behavior did not change.
- Verify no feature-layer dependency inversion, backend change, generated-file hand edit, or duplicate route implementation was introduced.
- Verify app-nav active state and no-snapshot recovery under direct and redirected loads.
- Treat the large moved test suite as a risk surface: confirm assertions remain semantically equivalent and were not deleted for path convenience.

#### Commit 2 — Move managed-club selection into My Club

**Status:** Active

**Provisional commit:** `feat(club): move managed club selection into My Club`

**Work:** Make the My Club header the single managed-club selection location and remove the selector from Settings without changing its persistence or membership contract.

**Out of scope:**

- Moving Staff or Staff Shortlist into My Club.
- Changing exact-club validation, cohort membership, stored selection, option discovery, or missing-club semantics.
- Auto-saving a club selection or adding club-family, team-level, crest, or summary UI.

**Implementation packet:**

- Replace the Settings-shaped panel with a compact, explicitly saved selector that fits the My Club header. Keep the existing combobox, exact option selection, missing saved club, pending, error, and late-result behavior.
- Render the selector only when a current snapshot exists. Keep the standard My Club no-snapshot state and top-bar Load Data recovery when no options can exist.
- Give the selector wrapper the stable `managed-club` anchor. Update all active recovery links to `/my-club#managed-club` and make `/settings#managed-club` a replace redirect for old bookmarks.
- Move the on-save invalidation callback to My Club. Invalidate managed-club state, Planner including Squad, Staff, and Academy through existing roots. Do not broaden invalidation to unrelated profile or Search state.
- Remove managed-club query prefetch, section UI, and section error boundary from Settings while retaining Settings context invalidation after save or snapshot management changes.
- Update current-state documents for the selector's new owner and stable recovery target.

**Files and responsibilities:**

- `src/features/managed-club/components/managed-club-selector.tsx` — renamed compact selector, existing combobox/mutation behavior, explicit save, missing warning, and safe feedback.
- `src/features/managed-club/components/managed-club-panel.tsx` — remove after callers and tests move; do not retain a second selector implementation.
- `src/app/routes/my-club.tsx` — selector placement, `managed-club` anchor, Suspense/error boundary, and established downstream invalidation callback.
- `src/app/routes/settings.tsx` — remove selector section and prefetch, retain Save data/Bridge and current-context invalidation.
- `src/app/routes/academy.tsx`, `src/features/staff/components/staff-search-results-panel.tsx`, and the Squad empty state in `src/app/routes/my-club.tsx` — update recovery target and copy.
- `src/app/routes/my-club-squad.test.tsx`, `src/app/routes/settings.test.tsx`, `src/app/routes/academy.test.tsx`, and `src/app/routes/staff.test.tsx` — move selector behavior assertions and update recovery/Settings expectations.
- `src/app/routes/legacy-club-routes.test.tsx` — old Settings hash replacement proof.
- `e2e/smoke.spec.ts` — selector save, missing-club, recovery-link, header-fit, and Settings section changes.
- `.wiki/ARCHITECTURE.md` and `.wiki/DESIGN.md` — My Club selector ownership, Settings scope, recovery target, and invalidation path.

**Behavior and data flow:**

- A loaded My Club route prefetches managed-club status and current exact options. The header selector initializes from `managedClub.clubName` and submits the same `set_managed_club` command.
- On success, the managed-club query refreshes and the route invalidates Planner/Squad, Staff, and Academy caches so currently mounted consumers reconcile to the new club.
- A saved club missing from the latest snapshot remains visible with its warning until the user selects and explicitly saves another exact option.
- Squad and Staff missing-club states and Youth Academy link to the header anchor. An old Settings anchor replaces itself with that canonical target.
- Save, snapshot, and active-context changes in Settings still invalidate managed-club and downstream current-context state through the existing callback.

**Ordered implementation steps:**

1. Add failing My Club selector, Settings absence, downstream invalidation, recovery-target, missing-club, late-result, and legacy-hash tests.
2. Adapt the existing picker and mutation into one compact selector component; preserve controlled query reset and accessible combobox behavior.
3. Compose the selector into the My Club header and move the current invalidation callback from Settings.
4. Remove the Settings section and managed-club prefetch without removing context invalidation used by save and snapshot changes.
5. Update Squad, Staff, and Academy recovery links and copy, then add the old hash redirect.
6. Add smoke coverage at 1280×800 and 1600×900 with collapsed and expanded navigation where the existing viewport loops apply.
7. Update current-state documents and compare the selector's final behavior with the former Settings implementation.

**Tests and proof:**

- RED: My Club has no managed-club combobox, Settings still owns it, recovery links still target Settings, and the old anchor cannot reach the new owner.
- GREEN: one selector exists in My Club, Settings exposes only Save data and Bridge, saving invalidates every membership consumer, and old and current recovery links reach the selector.
- Boundary coverage: no snapshot; unconfigured club; saved club missing from the latest snapshot; pending save followed by context replacement; exact suggestion selection; unchanged selection disables Save; save error remains inline.
- Layout coverage: the compact selector and workspace tabs fit without document overflow or a second vertical scroll owner at 1280×800 and 1600×900.

**Patterns to verify:**

- `src/features/managed-club/components/managed-club-panel.tsx` for the exact picker, controlled reset, warning, and mutation behavior to preserve.
- `src/app/routes/settings.tsx` for the current downstream invalidation set and section-level loading/error treatment.
- `src/app/components/app-top-bar.tsx` and Settings snapshot callbacks for active-save and Load Data managed-club invalidation.
- The My Club header and Data Table layout contracts in `.wiki/DESIGN.md`; use existing flex wrapping and tokens instead of a new layout primitive.

**Constraints and non-goals:**

- Keep one selector implementation and one mutation command.
- Preserve explicit Save; do not add auto-save, fuzzy matching, or a second selection state.
- Keep the form keyboard-operable, labelled, focus-visible, and truthful for a missing saved club.
- Do not introduce page-level scrolling around the full-height tables or hide the selector behind hover, a menu, or an unrequested modal.
- Do not invalidate unrelated Search or profile queries on a selection-only change.

**Dependencies and sequencing:**

- Depends on Commit 1's canonical route, header, route composition, feature-owned tabs, and compatibility test file.
- Must complete before Staff moves so both managed-club player and staff surfaces use the selector in their final parent route.

**Validation:** `./scripts/dev test src/app/routes/my-club-squad.test.tsx src/app/routes/settings.test.tsx src/app/routes/academy.test.tsx src/app/routes/staff.test.tsx src/app/routes/legacy-club-routes.test.tsx`; `./scripts/dev check`; `./scripts/dev smoke`.

**Stop conditions:** Stop and replan if the selector move requires persistence, IPC, membership, or fuzzy-matching changes; if saving cannot invalidate currently mounted consumers through existing roots; if the old Settings hash cannot redirect without affecting normal Settings loads; if the compact header cannot meet the minimum viewport without nested scrolling; or if late mutation results can overwrite a newer save/snapshot context.

**Review mandate:**

- Verify exactly one selector and unchanged exact-club persistence semantics.
- Verify current-context and selection-change invalidation for Squad, Planner, Staff, and Academy, including active mounted views.
- Verify missing, pending, error, stale-result, and no-snapshot paths.
- Verify all recovery links and the legacy Settings hash reach the canonical selector.
- Verify Settings keeps save/snapshot/bridge behavior and no longer preloads or renders selection UI.
- Verify 1280×800 and 1600×900 table containment, keyboard access, labels, focus, and warning contrast.

#### Commit 3 — Move club staff views into My Club

**Status:** Pending

**Provisional commit:** `feat(club): move club staff views into My Club`

**Work:** Complete the five-workspace shell by moving managed-club Staff and Staff Shortlist into My Club, leaving `/staff` as Staff Search, and applying the accepted navigation names.

**Out of scope:**

- Moving Staff Search, Player Search, Youth Academy, Dashboard, or profiles to new paths.
- Changing Staff Search filters, My Staff membership, Staff Shortlist persistence or scope, staff scoring, table layouts, profiles, imports, or boosts.
- Renaming backend `my-staff` commands, scopes, query keys, or persisted table-layout identifiers.

**Implementation packet:**

- Extend the My Club URL union and tabs with `staff` and `staff-shortlist`. Preserve the order Squad, Planner, Tactic, Staff, Staff Shortlist and the Squad default.
- Add independent optional `staffSort`/`staffDir` and Shortlist sort/context/filter keys. Switching views preserves inactive table state and uses replace history.
- Move the current My Staff and Shortlist route composition directly into the My Club route. Reuse `StaffSearchResultsPanel`, import modal, presentation helper, current query scopes, table IDs, and context-key suppression.
- Change visible My Staff copy to **Staff** or **managed-club staff** while keeping internal `my-staff` identifiers. Keep **Staff Shortlist** explicit and do not add a managed-club predicate to its query.
- Reduce `/staff` to Search-only UI while retaining the parent outlet for `/staff/$uid`. Redirect legacy My Staff and Shortlist URLs only when the exact path is `/staff`, map all applicable sort and shortlist filter state, and never redirect a profile path.
- Rename the top-level `/search` and `/staff` labels and headings to **Player Search** and **Staff Search**. Keep both paths and their table state unchanged.
- Remove obsolete Planner-only and Staff-only workspace tab components after the My Club tab owner has every caller.
- Update route, profile-Back, table-layout, shortlist, boost, deep-link, app-shell, smoke, concept, architecture, and design assertions without weakening existing behavior.

**Files and responsibilities:**

- `src/features/my-club/components/my-club-workspace-tabs.tsx` — final five labels, order, keyboard traversal, and panel IDs without sibling-feature imports.
- `src/app/routes/my-club.tsx` — five-view URL union, separate Staff/Shortlist state normalization, active-view loader dependencies, direct My Staff and Shortlist composition, URL callbacks, import context, contextual columns, bulk boost refresh, profile navigation, and workspace-state updates.
- `src/app/routes/staff.tsx` — Search-only canonical page, exact-path legacy redirects, and unchanged profile outlet behavior.
- `src/app/routes/search.tsx` — visible **Player Search** heading only; no path or query change.
- `src/app/components/app-nav-rail.tsx` — visible **Player Search** and **Staff Search** labels while keeping their existing destinations and icons.
- `src/features/staff/components/staff-search-results-panel.tsx` — user-facing managed-club Staff titles and recovery copy only; keep API scope and table identifiers.
- `src/features/staff/components/staff-workspace-tabs.tsx` and `src/features/planner/components/planner-workspace-tabs.tsx` — remove when no caller remains.
- `src/app/routes/my-club-staff.test.tsx` — moved My Staff and Shortlist behavior, independent URL state, imports, boosts, empty states, profile Back, and table-layout persistence.
- `src/app/routes/staff.test.tsx` — Search-only route behavior, filters, columns, profiles, and absence of club tabs.
- `src/app/routes/legacy-club-routes.test.tsx` — My Staff/Shortlist mapping, filters, replacement history, and profile-route exclusion.
- `src/app/routes/search.test.tsx` and `src/app/app-shell-routing.test.tsx` — accepted headings, nav labels, paths, and active states.
- `e2e/smoke.spec.ts` — canonical My Club Staff/Shortlist flows, Staff Search isolation, legacy links, profile Back, keyboard tabs, imports, boosts, long-table containment, and final viewport fit.
- `.wiki/CONCEPT.md`, `.wiki/ARCHITECTURE.md`, and `.wiki/DESIGN.md` — final product labels, route ownership, five-tab layout, selector location, compatibility paths, and preserved data boundaries.

**Behavior and data flow:**

- `/my-club?view=staff` renders `StaffSearchResultsPanel` with internal scope `my-staff`, current snapshot context, its established table layout, and the existing managed-club bulk boost.
- `/my-club?view=staff-shortlist` renders the same save-owned Shortlist query, filters, contextual score columns, upload modal, context-bound feedback, and profile links now used at `/staff?view=shortlist`.
- The My Club route stores independent sort state for Squad, Staff, and Staff Shortlist. Tab changes preserve all keys and replace the current history entry.
- `/staff` always renders Staff Search. Its filters, combine mode, sort state, table layout, profile links, loader, and child outlet remain in place.
- Exact legacy Staff workspace URLs map once to My Club. `/staff/$uid` bypasses this mapping, so direct profile loads and browser Back remain reliable.
- Visible navigation becomes Player Search, Staff Search, My Club, and Youth Academy without changing the existing Search, Staff Search, or profile paths.

**Ordered implementation steps:**

1. Add failing My Club Staff/Shortlist, Search-only Staff, naming, legacy redirect, independent state, profile Back, and shortlist-scope tests.
2. Extend the pure My Club URL contract and tab component, including accessible focus order and panel relationships.
3. Move only My Staff and Shortlist composition directly into the My Club route; retain query scopes, table IDs, mutations, and modal state.
4. Remove those views from `/staff` while retaining Search and the profile outlet. Add exact-path replace redirects for legacy view URLs.
5. Apply visible copy and top-level nav/heading names without renaming backend or stored identifiers.
6. Split the existing Staff route tests by final owner, update canonical smoke paths, and retain every meaningful import, boost, table, focus, and profile assertion.
7. Remove obsolete workspace tab components, run the full validation set, and reconcile current product, architecture, and design documents.

**Tests and proof:**

- RED: My Club lacks Staff tabs; `/staff` still exposes My Staff and Shortlist; legacy Staff links do not canonicalize; Player/Staff recruitment naming is ambiguous.
- GREEN: five tabs render in order, Staff and Staff Shortlist retain their current behavior under My Club, `/staff` exposes only Staff Search, and legacy URLs preserve state through replace redirects.
- Boundary coverage: Staff Search filters/sort; My Club Staff sort independence; Shortlist Preferred Job, unemployment, contextual score sort, replacement import and stale context; missing snapshot; missing managed club; missing shortlist; bulk boost partial/recovery behavior; profile direct load and Back from both My Club staff views.
- Negative coverage: Staff Shortlist results remain the imported save-owned set rather than the managed-club cohort; `/staff/$uid` never redirects; switching My Club tabs does not reset table layouts or an unsaved tactic draft.
- Viewport coverage: final five-tab header, selector, Staff table, Shortlist controls, Squad table, Planner, and Tactic remain usable at 1280×800 and 1600×900 with both rail states where the existing smoke harness supports them.

**Patterns to verify:**

- `src/app/routes/staff.tsx` for independent per-view URL state, Shortlist presentation, context-key suppression, bulk refresh, profile history, and child outlet handling.
- `src/features/staff/components/staff-workspace-tabs.tsx` and Planner tabs for keyboard behavior to consolidate in the feature-owned five-tab control.
- `src/features/staff/components/staff-search-results-panel.tsx` for internal scopes, stable table IDs, managed-club boost behavior, and empty-state ownership.
- `src/app/routes/search.tsx` for the standalone Player Search route, whose behavior and path must remain unchanged.
- Staff and Planner route suites plus `e2e/smoke.spec.ts` for table containment, focus, imports, boosts, state restoration, and existing viewport loops.

**Constraints and non-goals:**

- Preserve the My Club tab order and Squad default decided in this ledger.
- Keep Staff Shortlist unfiltered by managed club and retain its save/snapshot semantics.
- Keep backend and persisted identifiers stable even when visible copy changes.
- Keep profile routes and URL paths stable. Compatibility redirects replace history and never wrap a profile route.
- Do not duplicate Search filters into My Club Staff or add filters to Squad.
- Do not weaken existing large-route or smoke assertions to reduce migration work.

**Dependencies and sequencing:**

- Depends on Commit 1's canonical route and route-owned composition plus Commit 2's selector ownership and recovery target.
- Completes the final PR. After this commit clears checkpoint, run feature-level validation and `$workflow-finish-feature` before publication.

**Validation:** `./scripts/dev test src/app/routes/my-club-squad.test.tsx src/app/routes/my-club-staff.test.tsx src/app/routes/staff.test.tsx src/app/routes/search.test.tsx src/app/routes/legacy-club-routes.test.tsx src/app/app-shell-routing.test.tsx`; `./scripts/dev test`; `./scripts/dev check`; `./scripts/dev smoke`.

**Stop conditions:** Stop and replan if moving staff views requires a backend/query/persistence change; if Shortlist becomes managed-club scoped; if the exact `/staff` redirect cannot exclude `/staff/$uid`; if independent table state cannot survive tab changes without duplicating domain data; if moving the route breaks stale import or bulk-boost recovery guarantees; or if the final header cannot meet supported viewports without nested scrolling or hidden controls.

**Review mandate:**

- Verify final information architecture, labels, default, tab order, keyboard operation, and route history.
- Trace every legacy Planner, Staff, Settings-anchor, player-profile, and staff-profile path through direct load, navigation, refresh, and Back.
- Verify Staff Search stays standalone and unchanged; verify Staff Shortlist remains save-owned and not managed-club filtered.
- Verify independent Squad, Staff, and Shortlist URL sorts plus persisted table layouts survive workspace changes.
- Verify tactic drafts, shortlist modal context suppression, bulk Staff boost recovery, Squad actions, and current-context invalidation survive the move.
- Verify no backend, migration, IPC, query-key, table-layout ID, capability, or cross-feature dependency change slipped into the frontend move.
- Verify final docs describe implemented behavior and do not revive club-family terminology.
- Treat Repowise hotspot signals as review focus only; retain findings only with a violated contract, execution path, and observable consequence.

## Active work

**PR:** PR 1 — Unify managed-club workspaces

**Commit:** Commit 2 — Move managed-club selection into My Club

### RED proof

Add failing My Club selector, Settings absence, downstream invalidation, recovery-target, missing-club, late-result, and legacy-hash tests. They must fail because the selector is still owned by Settings and My Club has no saved managed-club control.

### Expected outcome

My Club owns one explicit managed-club selector in its header, Settings no longer renders the selector, and existing managed-club persistence, exact current-snapshot membership, and downstream invalidation behavior remain unchanged.

### Explicit exclusions

- Staff and Staff Shortlist relocation.
- Recruitment-page naming changes.
- Backend, persistence, scoring, query-key, stored-layout, or feature behavior changes.

## Discoveries and replanning

- Planning supersedes JAY-25's stale configured club-family wording with the merged JAY-27 contract: one save-scoped managed club and exact current-snapshot membership.
- The existing route files are high-churn, and the Planner route suite and browser smoke suite are large. The plan therefore consolidates cross-feature wiring in the canonical route and separates canonical-route tests from compatibility redirects instead of duplicating routes.
- No product or structural question blocks Commit 1. Visual fit remains an automated and native validation concern.
- Commit 1 review found that legacy direction-only sort state could be dropped by the compatibility mapper. The mapper now preserves a valid `dir` as canonical `squadDir`, with regression coverage for absent and invalid legacy sort fields.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Commit 1 — Move squad planning into My Club | Pending record | Canonical My Club route, mounted Squad/Planner/Tactic workspaces, typed `/planner` compatibility redirect, nav and current-state docs | Sol Medium clean after one bounded compatibility correction | None |
| PR 1 | Commit 2 — Move managed-club selection into My Club | Pending record | Pending | Pending | None |
| PR 1 | Commit 3 — Move club staff views into My Club | Pending record | Pending | Pending | None |

## Final validation

- `./scripts/dev test` — full frontend behavior suite, including canonical routes, compatibility redirects, table state, profiles, imports, boosts, Planner/Tactic persistence, and managed-club lifecycle.
- `./scripts/dev check` — Biome, TypeScript, secretlint, Rust format, Clippy, and Rust tests through the stable repository gate.
- `./scripts/dev smoke` — full Chromium product suite with canonical My Club paths and 1280×800 and 1600×900 coverage for the final header, selector, tables, Planner, Tactic, Staff, and Staff Shortlist.
- `git diff --check` — no whitespace errors in the complete working change; feature completion must also review the exact recorded implementation range.
- Manual native Tauri/WebView pass when a desktop runtime is available: direct and legacy routes, five-tab keyboard traversal, selector focus/save, unsaved tactic draft retention, staff and player profile Back, expanded/collapsed rail, and no document-level scrolling at 1280×800 and 1600×900. If unavailable, record the gap and do not claim it passed.
- Confirm no migration, IPC, capability, dependency, release metadata, or backend file changed. Confirm `./scripts/dev mutate` remains unsupported and is not reported as passed.

## Documentation impact

Planning creates this ledger and marks the feature active in `.wiki/TODO.md`. Implementation must update `.wiki/ARCHITECTURE.md` and `.wiki/DESIGN.md` in the commits that change current route and selector ownership. Commit 3 must reconcile user-facing names in `.wiki/CONCEPT.md`. No ADR or debug report is warranted unless implementation disproves the established boundary or exposes a reusable failure pattern.
