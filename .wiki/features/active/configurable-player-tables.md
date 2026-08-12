# Configurable Player Tables

## Status

Active

## Intent

Turn Search and Squad into consistent, full-height player workspaces. Both tables should keep one continuously scrollable, virtualized result set; retain user choices across profile navigation; support stable, resizable, configurable columns; and expose the same organized player-metric catalog used by Search filters. The feature also adds current and potential role-score metrics and compact nationality flags.

## User-visible behavior

- Search and Squad fill the vertical workspace left below their route controls. Rows remain virtualized and IPC remains paged, but the user sees one continuous table with no Previous or Next controls.
- Search keeps its applied filters, filter-combine mode, sort field, and sort direction in the URL. Squad keeps its workspace and sort state in the URL. Opening a player and navigating back restores that table state.
- Opening the Search filter editor creates a local draft. Changing fields, operators, values, rules, or combine mode does not update the URL or run the query. Done applies one complete, valid draft; Cancel, close, Escape, or backdrop interaction discards it.
- Search and Squad persist independent visible-column order and width preferences across route changes and app launches. Sorting never changes column order or width.
- The whole Squad row opens the player profile. Both tables keep the existing arrow-key row movement and Enter activation behavior.
- Every header supports sorting, pointer and keyboard resizing, and a right-click or keyboard-accessible menu. The menu can remove the current column or open the shared metric picker to append another column. At least one column must remain visible.
- Every scalar or list field currently offered by Search filters is also available as a display and sort column. Position is displayed as a strongest-first list of position keys. Current role scores and potential role scores are separate filter and column families.
- Applying a Search filter with Done adds that metric to the Search table once if it is not already visible. Removing the column later does not remove or disable the filter.
- The metric picker is searchable and grouped by player identity, club and contract, ability and reputation, visible attributes, hidden attributes, personality, position suitability, current role scores, and potential role scores. Role scores are further grouped by playing-area family. Search filter-field selection and table column selection reuse the same picker.
- Nationality cells render one bundled SVG flag for every stored nationality, including second and later nationalities. Hovering a flag shows the full stored country name, and assistive technology receives the same name. An unmapped future value displays its original text rather than a wrong flag.
- Adding a potential role-score column, sorting by it, or applying a potential role-score filter calculates only the required cohort on demand. Results are cached persistently by snapshot, player, role, and projection-model version, so returning to the table or reopening the app does not repeat unchanged work.

## Invariants

- Product reads remain scoped to the active save's effective current snapshot. Search filters and Squad club-family membership remain Rust- and SQLite-owned.
- React never loads the complete result set. It requests bounded pages for the virtual range, while Rust and SQLite own global count, filtering, and sorting.
- All metric, sort, filter, offset, and limit inputs are validated at the Rust command boundary before any SQL fragment is selected. Raw WebView strings are never interpolated as SQL.
- Current role scores continue to come from ingest-owned `player_role_scores`. Potential scores continue to come from the existing CA-to-PA visible-attribute projection and role catalog; their cache is disposable derived data, not a new source of truth.
- A potential score used for global filtering or sorting is complete for the relevant current-snapshot cohort before the count and ordered page query runs. A potential score used only for display may be materialized for the requested page.
- Cache rows with an unknown score are still recorded, so `NULL` is not recalculated on every read. Snapshot or player deletion cascades to the cache, every successful supported player boost invalidates that player's cached potential scores, and a projection-model version change makes older rows ineligible.
- Search and Squad use the same frontend metric metadata and the same Rust metric resolver. Search-specific filter operators stay in Search; Squad-specific club-family membership stays in Planner.
- URL state remains the source of truth for applied filters and sorting. While the filter editor is open, its rules and combine mode exist only as local draft state. Only Done may commit that draft and trigger Search or potential-cache work. Zustand owns only client preferences that do not belong in the URL: visible columns, insertion order, and widths for each table.
- Table rows retain a fixed measured height. Column customization may create horizontal scrolling, but it must not disable row virtualization or create page navigation.
- Context-menu actions and resize handles have keyboard equivalents, visible focus, and accessible names. A right-click-only path is not acceptable.
- Nationality artwork is bundled with the app. The feature does not fetch flag assets at runtime and never substitutes a national flag from a guessed country.
- The memory-reader request, bridge scan scope, dump schema, scoring formula, Planner optimizer, and player-profile calculations remain unchanged.

## Non-goals

- Player gender filtering and the accompanying memory-reader or stored-data investigation. Both are deferred to `.wiki/TODO.md` until representative snapshots contain trustworthy values.
- Adding league or division-country flags, league-country metadata, or a division-country mapping.
- Adding Squad filters, saved filter presets, shared table presets, column drag-and-drop reordering, or table-scroll restoration.
- Precomputing potential role scores during Load Data or replacing persisted current role scores.
- Reusing the table cache in player profiles, Planner depth, or Planner optimization in this feature.
- Changing role definitions, projection weights, role-score bands, or the meaning of Current and Potential.
- Fetching flag assets from a CDN or using platform-dependent flag emoji.

## Current-state map

- Relevant components:
  - `src/features/search/components/search-results-panel.tsx` owns Search's fixed and filter-derived columns, virtual row window, page queries, sorting, and whole-row profile navigation.
  - `src/features/squad/components/squad-overview-panel.tsx` duplicates the eight Search-compatible columns, loads one 50-row page, renders Previous / Next controls, and links only the player name.
  - `src/features/search/components/search-filter-editor-modal.tsx` renders the filter-field catalog as one flat native select from `src/features/search/utils/filter-registry.ts`. Its rule, value, and combine callbacks currently update route state immediately, so each keystroke can start another Search query.
  - `src/app/routes/search.tsx` stores Search filters, combine mode, and sort in validated URL search state. `src/app/routes/planner.tsx` stores Squad workspace and its eight-field sort in URL search state.
  - `src/stores/use-layout-store.ts` demonstrates persisted Zustand client preferences. `src/app/components/app-shell-layout.tsx` gives the route outlet a bounded main area, but the Search and Squad route trees do not yet propagate `h-full`, `min-h-0`, and flex growth to their table scrollers.
- Data model:
  - `players.nationalities_json` stores an ordered string array.
  - `player_role_scores` stores all 68 current role scores per snapshot player. Potential attributes and scores are currently projected only inside bounded player and Planner reads.
  - Search can filter all scalar, JSON attribute, position-suitability, and current-role fields. Non-basic filter fields are automatically selected into `dynamicValues`; Squad has no dynamic field projection.
- Persistence and migrations:
  - SQLite schema version 20 is current. No persistent potential-score cache exists.
  - The proposed cache requires additive migration v21 with a composite player foreign key and cascade behavior.
  - Search and Squad column preferences do not exist; only the nav rail and Load Data preferences currently use persisted Zustand state.
- Existing behavioral assumptions:
  - Search is row-virtualized and server-paged in 50-row windows but has a fixed `max-h-[min(70vh,720px)]` scroller.
  - Squad uses the same fixed maximum height but exposes explicit 50-row pagination.
  - Search's active filter fields define its extra visible and sortable columns. Squad sort is limited to the eight fixed fields.
  - Browser history already retains each table route's URL. This feature must preserve that contract while adding broader sort fields.
- Architectural seams:
  - Shared React table presentation, metric metadata, and client preferences belong under `src/components`, `src/types`, `src/utils`, and `src/stores`; Search and Squad retain feature adapters.
  - Shared Rust metric resolution and derived-cache work belongs in a new `features/player_metrics` module; Search keeps filter AST compilation and Planner keeps Squad membership.
  - Rust owns migration, cache population, projection, global query semantics, and bounded DTOs. React sends requested field IDs and renders returned values.
- Project validation commands:
  - `./scripts/dev test [target...]`
  - `./scripts/dev check`
  - `./scripts/dev smoke`
  - `./scripts/dev format [paths...]`
  - `./scripts/dev secrets --staged`
- Primary risks:
  - `db/migrations.rs`, Search query code, and both table panels are high-churn hotspots. The feature must split cache, query, virtualization, configuration, and flag behavior into reviewable commits.
  - Global potential-score sorting or filtering can require projection across roughly 183,000 current players before the first result page.
  - A stale derived cache could silently mis-sort players after a projection change or successful live player boost.
  - Full-height flex changes can create nested scroll owners or make non-Squad Planner workspaces unusable.
  - Custom header menus and resize handles can regress semantic table, focus, pointer, or keyboard behavior.

## Feature architecture

`src/utils/player-metrics.ts` will become the frontend presentation catalog. It assigns every metric a stable ID, label, value kind, alignment, default width, category, optional role family, filter capability, and sort capability. Search derives its operator definitions from that metadata, while a shared searchable `PlayerMetricPicker` presents the same hierarchy to filter rows and column menus. Rust independently validates the same closed ID families at the IPC boundary; the WebView catalog is not trusted for SQL safety.

The Search filter modal will copy the currently applied URL-backed rules and combine mode into local draft state each time it opens. Field, operator, value, add, remove, and combine interactions mutate only that draft. Done remains disabled while any draft rule is incomplete or invalid; one route-level apply callback commits the whole draft, auto-adds its metrics as columns, closes the modal, and lets the resulting URL change begin the next Search refresh cycle. Cancel, the close control, Escape, and backdrop dismissal close without applying. Removing or clearing an already-applied filter from the filter strip remains immediate because it is outside the editing transaction.

`src/stores/use-player-table-store.ts` will persist versioned `search` and `squad` layouts. Each layout contains a non-empty ordered metric-ID list and clamped widths keyed by metric ID. Unknown IDs from an older store version are dropped. Adding a column appends it; there is no drag reorder. Removing the active sort column selects CA when CA remains visible, otherwise the first visible sortable column, and updates the route URL in the same interaction.

A shared player-table component will own semantic headers, fixed `<colgroup>` widths, resize handles, header menus, row virtualization, page-window selection, roving row focus, horizontal overflow, and its single vertical scroll owner. Search and Squad adapters supply query options, totals, route-backed sort callbacks, cells, and row activation. Both backend commands retain bounded page limits; continuous scrolling is a presentation contract, not an unbounded IPC response.

Rust `features/player_metrics` will own metric parsing, SQL expressions, dynamic cell decoding, position display serialization, and potential-cache orchestration. `search_players` will accept requested display fields in addition to its filter AST. `list_squad_players` will accept requested display fields and the same general sort IDs while retaining exact club-family membership. Duplicate field IDs are removed and every ID is validated before query construction.

Potential table scores will use the additive cache described by [ADR-0019](../../decisions/0019-lazy-potential-role-score-cache.md). The cache stores one nullable result per requested `(snapshot_id, uid, role_id)` plus the projection-model version. When several missing roles are requested together, Rust parses and projects each affected player once and scores every missing requested role from that projection. Display-only fields populate only page players; a potential sort or filter populates its complete Search or Squad cohort in resumable bounded transactions before SQL evaluates it. Repeated reads use SQLite rows across route changes and app restarts. Every successful supported player boost deletes that player's potential cache rows.

Nationality presentation will use the offline `country-flag-icons` SVG package and a tested FM-name-to-ISO or subdivision-code map. A checked-in fixture of every distinct nationality value from the representative snapshot must resolve through that map. It includes FM spellings such as `China PR`, `Ivory Coast`, `South Korea`, `Türkiye`, `The Gambia`, and the four UK home nations. Each value in the stored nationality array renders independently. Only values absent from the observed fixture and explicit mapping retain truthful text fallback.

## Uncertainty register

### Known

- The inspected Windows database stores nationality values as full FM names, not ISO codes. Representative values include `England`, `China PR`, `Türkiye`, `Ivory Coast`, and `The Gambia`.
- Search already has bounded virtual page loading and whole-row profile navigation. Squad already has URL-backed sorting but still paginates visibly and links only the player name.
- The role catalog contains 68 roles. Current scores are persisted on ingest; potential scores are calculated from the existing pure projection and scorer.
- The repository already uses persisted Zustand client state, TanStack Query IPC caching, TanStack Virtual, semantic table markup, and a keyboard-accessible header context-menu pattern in Planner.

### Assumptions

- Returning from a profile means normal browser or app history navigation. Preserving scroll position is not part of the requested state contract.
- Current default columns remain Name, Age / DOB, Nationality, Club, Division, CA, PA, and Value in that order for both tables.
- The stored nationality array order is meaningful and must be preserved when flags render.
- Direct filter-strip removal and clear actions remain immediate; the Done boundary applies to changes made inside the filter editor.

### Decisions

- Applied filter and sort state stays URL-backed, while filter-modal edits remain local until Done. Cancel and every dismissal path discard the draft; incomplete or invalid drafts cannot be applied.
- Column visibility and widths are independent per table and persisted through Zustand local storage.
- Applying a filter draft adds each filter metric column once, but filters and columns can then be changed independently.
- Position becomes a real sortable display metric: Rust returns a strongest-suitability-first comma-separated key list with stable key tie-breaking, and sorting uses that normalized display value.
- Potential cache population is sparse by requested role, versioned, persistent, snapshot-owned, and disposable. It is not populated during ingest.
- Multiple missing potential roles in one request share one player projection pass. A later request for a new role can add only that role without rewriting earlier cached roles.
- Column insertion order is stable and append-only except removal. Sorting does not reorder columns, and drag reordering is deferred.
- The flag dependency is bundled locally; no remote request or emoji rendering is used. Unmapped values show text rather than a guessed flag.

### Unknowns

- Cold full-current-snapshot materialization time for one or several potential role fields on the production Windows machine has not been measured through the assembled Tauri command.
- Future bridge dumps may contain additional FM nationality spellings not present in the inspected snapshot. The explicit fallback remains required even after the initial mapping is exhaustive for the representative dataset.

### Risks

- A cold potential filter or sort can keep the shared SQLite connection busy long enough that other database-backed UI reads wait. If the WebView itself stops responding or the measured delay is outside the user's accepted minute-scale first run, replan the materializer as a cancellable background job rather than hiding the delay.
- Cache version or boost invalidation omissions would produce believable but stale values. Migration, query, and boost tests must prove every invalidation boundary.
- A metric accepted by the frontend but rejected or decoded differently in Rust would break configurable columns. Contract tests must cover every metric family, including Position and both role-score bases.
- Persisted width or field IDs can outlive a catalog change. Store hydration must sanitize rather than crash or render an empty table.
- Many columns require horizontal scrolling. The table must keep fixed widths and one vertical scroll owner without squeezing columns based on sorted cell content.
- Flag aliases are correctness-sensitive. A missing map entry is acceptable only through the truthful text fallback; a wrong national flag is not.
- Draft state can become stale if it survives a close/reopen cycle. The modal must initialize from current route state on every open and discard its draft on every non-Done close path.

## Walking skeleton

The thinnest end-to-end path is: choose a Potential role score in the categorized Search filter draft, click Done, let the resulting applied filter make Rust materialize and reuse its versioned score cache, render the filtered paged result in the virtual table, open a player, and navigate back to the same URL-backed sort/filter state. No query or cache work begins while the user is still editing. Later commits generalize that metric path to requested columns and Squad, then add the shared full-height table, persisted widths, header menus, and nationality renderer.

## Delivery plan

### PR 1 — Configurable player tables

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(tables): add configurable player views`

**Purpose:** Deliver the complete Search and Squad table refinement as one user-facing feature while keeping staged metric selection, cache persistence, query contracts, virtualization, column interaction, and flag rendering in separate revertible commits.

**Depends on:** Current `main` at `d15b854178d8`; no external feature branch or unpublished PR.

#### Commit 1 — Stage organized filter changes

**Status:** Active

**Provisional commit:** `feat(search): stage organized filter changes`

**Work:** Create the shared frontend metric catalog and searchable grouped picker, derive existing Search filter definitions from it, replace the flat filter-field select, and make the modal edit a local rules-and-combine draft that updates Search only when the user clicks Done.

**Out of scope:**

- Potential role filters, table column menus, width state, backend query changes, or flags.

**Implementation packet:**

- Owners and files: `src/utils/player-metrics.ts`, shared picker files under `src/components/ui/`, Search filter registry/editor/bar, Search route wiring, and focused Search component/route tests.
- Existing patterns to verify: `Modal` dismissal behavior, `TextField`, native-labelled field controls, current immediate `onRulesChange` and `onCombineChange` callbacks, filter URL parsing, `ROLE_CATALOG`, and Planner's keyboard menu behavior.
- Constraints and invariants: preserve every current filter ID, operator, and applied URL representation; initialize a fresh draft from current route state on every open; keep every edit local; disable Done for an incomplete or invalid rule; invoke one apply callback with the complete rules-and-combine value; discard on Cancel, close, Escape, and backdrop; group current role scores by stable playing-area families; keep search, group headings, error state, and focus behavior accessible.
- Dependencies and ordering: this establishes the presentation registry, picker, and apply boundary reused by commits 2 and 5. Rust validation remains unchanged.

**Implementation profile:** Terra xhigh — the work is UI-heavy and cross-cuts a high-fan-out registry and route boundary but remains frontend-only.

**Review profile:** Sol Medium — focus on catalog parity, transactional filter behavior, query suppression, and accessible picker interaction.

**Validation:** Start with failing modal tests proving field, operator, value, add, remove, and combine edits make zero apply calls; Done applies the full valid draft exactly once; incomplete drafts disable Done; Cancel, close, Escape, and backdrop make zero apply calls; and reopening starts from the latest applied route state. Add the failing grouped-picker behavior test, then run `./scripts/dev test src/features/search/components/search-filter-editor-modal.test.tsx src/app/routes/search.test.tsx`, `./scripts/dev check`, and `./scripts/dev secrets --staged`. Expected evidence: no URL or Search query update occurs while typing, one update occurs on Done, existing filters round-trip unchanged, grouped search selects a field, and the full gate passes.

**Stop conditions:** Stop and replan if the shared catalog would require Search to trust frontend-only metadata for SQL safety, if any editor interaction still mutates route state before Done, or if the picker cannot preserve a labelled keyboard path without adding a new general interaction dependency.

**Review mandate:**

- Confirm every pre-existing filter ID, kind, operator, and enum option remains available.
- Confirm role grouping does not change role IDs or labels.
- Confirm all draft mutations are query-silent and Done is the only modal path that applies them.
- Exercise search, empty search, keyboard selection, invalid drafts, Done, Cancel, close, Escape, backdrop, reopen, and focus restoration.
- Confirm direct removal or clearing from the applied-filter strip remains immediate.
- Reject cross-feature imports from Search into Squad-facing shared code.

#### Commit 2 — Cache potential table role scores

**Status:** Pending

**Provisional commit:** `feat(scoring): cache potential table role scores`

**Work:** Add migration v21 and shared Rust potential-score cache orchestration, invalidate affected rows after successful boosts, expose Potential role score filters beside Current scores, and materialize the full Search cohort before a potential filter evaluates.

**Out of scope:**

- Potential attributes as table columns, ingest-time potential scoring, profile/Planner cache adoption, selected table columns, or a background job system.

**Implementation packet:**

- Owners and files: `src-tauri/src/db/migrations.rs`, new `src-tauri/src/features/player_metrics/` cache and metric resolver modules, Search filter/query/commands, player boost reconciliation, frontend metric/filter catalogs, IPC stubs, and focused Rust/Search tests. Include ADR-0019 with this behavior when the content commit is checkpointed.
- Existing patterns to verify: composite `(snapshot_id, uid)` foreign keys, `player_role_scores`, `project_attributes`, `score_role`, snapshot cascades, v20 upgrade fixtures, boost transactions, and Search suspense/error states.
- Constraints and invariants: one nullable row per snapshot/player/requested role; projection-model version checked on every read; several missing roles share one projection; full Search snapshot materialized in bounded resumable transactions before potential filter count; draft filters never start materialization; current role behavior unchanged; cache writes are never authoritative.
- Dependencies and ordering: commit 1 supplies the catalog, picker, and Done boundary. Potential-filter cache work begins only after Done applies the draft. Commit 3 reuses the cache for display and Squad sorting.

**Implementation profile:** Terra Max — additive persistence, high-volume projection, query compilation, and boost invalidation cross several Rust ownership boundaries.

**Review profile:** Sol xhigh — stale-cache or incomplete-cohort defects would silently return wrong scouting order and filter results.

**Validation:** Start RED with v20-to-v21 migration, cold-cache potential filter, repeated-read reuse, multi-role shared population, cached `NULL`, model-version mismatch, snapshot cascade, boost invalidation, and a frontend route test proving a drafted potential filter makes no IPC request until Done. Run `./scripts/dev test src/app/routes/search.test.tsx`, `./scripts/dev check`, and `./scripts/dev secrets --staged`. On a representative Windows snapshot, record cold and warm timings for at least two potential roles and prove the warm read adds no rows. Expected evidence: Done starts the applied Search request cycle, global results are correct before its first page returns, and repeated reads reuse cache rows.

**Stop conditions:** Stop and replan if a cold full-snapshot materialization freezes the WebView, cannot complete within the accepted minute-scale first-use delay, or requires holding a transaction across unrelated user work. Stop if boost reconciliation cannot invalidate cache rows atomically with its SQLite update.

**Review mandate:**

- Inspect migration upgrade, constraints, index, foreign key, and cascade behavior.
- Prove filter count and ordering never observe a partially populated required role.
- Prove draft potential filters cannot populate or read the cache before Done.
- Prove `NULL` and non-`NULL` results are both reusable and version-gated.
- Prove every successful supported boost invalidates only the affected player's derived rows.
- Check SQL field and role IDs remain catalog-validated.
- Check cold-path errors cannot leave authoritative tables modified.

#### Commit 3 — Query selected player metrics

**Status:** Pending

**Provisional commit:** `feat(tables): query selected player metrics`

**Work:** Extend Search and Squad page commands with validated requested-field lists and dynamic values, allow both commands to sort all sortable metric families, make Position a real display/sort field, and use page-only or cohort-wide potential cache population according to query semantics.

**Out of scope:**

- Column menus, persisted layout, resizing, full-height layout, row interaction, or flags.

**Implementation packet:**

- Owners and files: shared Rust player-metric resolver/row decoder, Search and Planner Squad commands/queries/tests, frontend Search/Squad fetchers, query keys, DTOs, and IPC mocks.
- Existing patterns to verify: Search dynamic field selection, `MAX_PAGE_LIMIT`, Squad club-family membership, whitelisted sort expressions, stable UID tie-breaks, and typed `dynamicValues` serialization.
- Constraints and invariants: requested fields are deduplicated and validated; page size stays bounded; filters need not remain visible to remain active; Position emits a strongest-first stable string; display-only potential fields populate page UIDs, while potential sort/filter fields populate their complete relevant cohort first.
- Dependencies and ordering: requires commit 2's cache/resolver and prepares the data contract consumed by commits 4 and 5.

**Implementation profile:** Terra Max — this unifies two backend read paths without erasing their distinct Search-filter and Squad-membership rules.

**Review profile:** Sol xhigh — field projection and global sort correctness affect every configurable column across high-volume queries.

**Validation:** Start with failing Rust query tests for every field family, Position display/sort, invalid IDs, duplicate IDs, selected current/potential role values, Search global potential ordering, Squad cohort potential ordering, and bounded pages. Add failing frontend serialization/query-key tests. Run `./scripts/dev test src/features/search src/features/squad src/app/routes/search.test.tsx src/app/routes/planner.test.tsx`, `./scripts/dev check`, and `./scripts/dev secrets --staged`. Expected evidence: both commands return the same metric semantics without unbounded IPC.

**Stop conditions:** Stop if sharing the resolver would make Planner depend on Search internals, if any filter field cannot produce a truthful sortable display value, or if requested fields can bypass Rust catalog validation.

**Review mandate:**

- Compare Search and Squad values for the same fixture and metric IDs.
- Prove current and potential role IDs cannot be injected or confused.
- Verify filters remain authoritative even when their column is hidden.
- Verify Position display order, tie-breaking, and sort semantics.
- Verify no complete player collection crosses IPC and all page limits remain enforced.
- Check query keys include requested field order so cached pages cannot carry the wrong shape.

#### Commit 4 — Virtualize full-height player lists

**Status:** Pending

**Provisional commit:** `feat(tables): virtualize full-height player lists`

**Work:** Extract the stable shared virtual-table behavior, make Search and Squad propagate full-height flex constraints, remove Squad's page controls in favor of virtual page loading, and make every Squad row pointer- and keyboard-activatable.

**Out of scope:**

- User-selected columns, resizing, header context menus, nationality flags, or scroll-position restoration.

**Implementation packet:**

- Owners and files: shared player-table component/hook under `src/components/`, Search and Squad panels, Search/Planner routes, narrowly necessary Panel/app-shell flex hooks, route tests, IPC stub, and smoke coverage.
- Existing patterns to verify: Search virtual page math and roving focus, `ResizeObserver` fallback, sticky header constants, Squad URL sort, hidden Planner tab panels, and the existing route-level suspense states.
- Constraints and invariants: one vertical scroll owner; no visible pagination; no all-row client array; fixed row height; bounded overscan/page queries; row click and Enter open the profile; browser back restores route sort/filter state; Planner and Tactic tabs retain usable scrolling.
- Dependencies and ordering: consumes commit 3's common paged metric shape and creates the header/table base extended in commit 5.

**Implementation profile:** Terra Max — two high-churn table implementations, route sizing, virtualization, and keyboard navigation must converge without regressions.

**Review profile:** Sol High — focus on user-visible navigation, paging boundaries, layout containment, and accessibility.

**Validation:** Start with failing tests for Squad virtual range page requests, absence of page navigation, whole-row click/Enter, Search and Squad sort persistence after profile/back, and bounded full-height scrollers. Run `./scripts/dev test src/app/routes/search.test.tsx src/app/routes/planner.test.tsx`, `./scripts/dev check`, `./scripts/dev smoke`, and `./scripts/dev secrets --staged`. Manually inspect Search and all three Squad workspaces at 1280×800 and 1600×900. Expected evidence: one continuous table fills remaining height and only virtual pages load.

**Stop conditions:** Stop if a shared component needs feature-specific imports, if route sizing introduces nested vertical scrollbars, or if hidden Planner panels influence the active Squad table's measured height.

**Review mandate:**

- Verify page calculations at first, middle, last, empty, and changing-total ranges.
- Verify Search did not lose its loading placeholders or row focus behavior.
- Verify Squad has no Previous / Next UI and never requests an unbounded limit.
- Verify click, Enter, ArrowUp, ArrowDown, and visible focus on both tables.
- Inspect 1280×800 overflow and Planner/Tactic workspace behavior.
- Verify back navigation preserves URL state without promising scroll restoration.

#### Commit 5 — Persist resizable column layouts

**Status:** Pending

**Provisional commit:** `feat(tables): persist resizable column layouts`

**Work:** Add per-table Zustand layouts, fixed column widths, pointer and keyboard resize handles, header context menus, add/remove behavior, and the shared metric picker for every filterable metric including Current and Potential role scores. Attach automatic filter-column insertion to the modal's single Done transaction.

**Out of scope:**

- Drag reorder, shared Search/Squad presets, column-width URLs, flags, or additional filters on Squad.

**Implementation packet:**

- Owners and files: `src/stores/use-player-table-store.ts`, shared table header/menu/resize components, shared metric catalog/picker, Search and Squad adapters/routes, filter-add integration, and component/route/smoke tests.
- Existing patterns to verify: `useLayoutStore` persistence, Planner header right-click plus explicit button, semantic `aria-sort`, fixed table tokens, route sort callbacks, and query requested-field lists.
- Constraints and invariants: independent Search/Squad layouts; sanitized store version; at least one visible column; active-sort removal resets predictably; new fields append; applying a draft adds each newly filtered field at most once; cancelled drafts add no columns; filtering can remain hidden; explicit `<colgroup>` widths and `table-layout: fixed`; resizing cannot trigger sort or row navigation; pointer capture is cleaned up; separator keys adjust width within registry bounds.
- Dependencies and ordering: uses commit 1's picker and apply boundary, commit 3's requested fields, and commit 4's shared header/table base.

**Implementation profile:** Terra Max — persisted state, context menus, resizing, routing, and dynamic query shape interact across both primary data tables.

**Review profile:** Sol xhigh — subtle state or event defects can corrupt layouts, hide active sorting, or make headers unusable by keyboard.

**Validation:** Begin with failing store hydration/sanitization, context-menu add/remove, Done-only filter-auto-add, cancelled-draft no-op, last-column guard, active-sort removal, pointer/keyboard resize, and sort-without-layout-change tests. Run `./scripts/dev test src/features/search src/features/squad src/app/routes/search.test.tsx src/app/routes/planner.test.tsx`, `./scripts/dev check`, `./scripts/dev smoke`, and `./scripts/dev secrets --staged`. Manually reload the app and revisit both routes after profile navigation. Expected evidence: widths and insertion order persist, sort/filter URLs persist, a draft changes neither, and multiple potential columns reuse cached values.

**Stop conditions:** Stop if hydration depends on route timing, if a context-menu-only action has no keyboard trigger, or if width persistence requires duplicating server data in Zustand.

**Review mandate:**

- Verify persisted unknown IDs and invalid widths are safely normalized.
- Verify Search and Squad layouts cannot overwrite one another.
- Verify sort changes preserve DOM column order and exact widths.
- Verify add, remove, final-column guard, and active-sort reset semantics.
- Verify filter-derived columns are added only by Done and only once per metric.
- Verify resize works by pointer and keyboard without triggering sort.
- Verify every Search filter metric, Position, and both role-score bases can be selected and sorted.
- Verify a hidden active filter still affects results.

#### Commit 6 — Render all nationality flags

**Status:** Pending

**Provisional commit:** `feat(tables): render nationality flags`

**Work:** Add the bundled SVG flag dependency, explicit FM-name mapping, shared nationality cell, full-name hover/accessibility labels, multi-nationality rendering, and truthful fallback in Search and Squad.

**Out of scope:**

- League or division flags, remote assets, flag emoji, nationality normalization in SQLite, or changing nationality sort semantics.

**Implementation packet:**

- Owners and files: `package.json`, `pnpm-lock.yaml`, a checked-in fixture containing every distinct nationality from the representative snapshot, shared nationality mapping/component/tests, and Search/Squad cell renderers. Record the dependency license through existing package metadata rather than vendoring untracked artwork.
- Existing patterns to verify: ordered `nationalities` arrays, existing title-based compact tooltips, fixed row height, Vite asset bundling, and the complete distinct set of stored FM nationality names.
- Constraints and invariants: every value in the observed-nationality fixture resolves to a flag; render every array value in order; use ISO 3166-1 or supported subdivision codes only after explicit mapping; include England, Scotland, Wales, and Northern Ireland flags; only unseen future strings show original text; no network request; no league icon.
- Dependencies and ordering: lands last so one shared configurable nationality column receives the behavior in both tables.

**Implementation profile:** Terra xhigh — a contained presentation change with a correctness-sensitive mapping and new bundled asset dependency.

**Review profile:** Sol High — the main risk is silently showing the wrong nation or dropping additional nationalities.

**Validation:** Start with a failing coverage test that enumerates every distinct nationality from the representative snapshot and requires a flag mapping for each one. Add renderer tests for all four UK home nations, two and three nationalities, empty arrays, and an invented future value that exercises truthful text fallback. Run `./scripts/dev test src/components src/features/search src/features/squad`, `./scripts/dev check`, `./scripts/dev smoke`, and `./scripts/dev secrets --staged`. Manually hover every flag in a multi-nationality cell and inspect the packaged/offline asset path. Expected evidence: all currently stored values resolve and render in order with full-name tooltips, while only the invented unknown falls back to text and no league flags appear.

**Stop conditions:** Stop if the selected package cannot bundle the required home-nation subdivisions offline under a compatible license, or if representative FM names cannot be mapped without guessing.

**Review mandate:**

- Verify every committed alias maps to the intended nation or subdivision.
- Verify the observed-nationality coverage fixture matches the representative snapshot's complete distinct value set and every entry resolves.
- Verify second and later nationalities are not collapsed or discarded.
- Verify tooltip and screen-reader names use the full stored value.
- Verify unknown values remain truthful text and empty arrays render `—`.
- Verify no CDN, runtime network, emoji, league flag, or reader change entered the diff.

## Active work

**PR:** PR 1 — Configurable player tables

**Commit:** Commit 1 — Stage organized filter changes

### RED proof

Add focused Search filter-editor tests that expect a searchable grouped metric picker and a local draft transaction. They must prove that typing, changing controls, adding or removing rules, and changing combine mode make zero apply calls; Done applies one valid draft; invalid drafts cannot be applied; every dismissal path discards the draft; and reopening uses the latest applied state. They must fail because the current editor renders one flat native select and invokes route callbacks immediately.

### Expected outcome

The existing Search filter catalog is presented through an accessible organized picker and shared presentation registry. Modal edits remain local until Done performs one URL update and Search refresh; cancelled edits perform none. Later potential-filter and column work can reuse the same picker and apply boundary without importing Search internals.

### Explicit exclusions

No Rust, migration, table layout, column state, potential-score, flag, Git, or publication change belongs in the active implementation commit.

## Discoveries and replanning

- 2026-08-12: Direct inspection found 182,836 current rows, all stored as `gender = 'unknown'`. The developer removed gender filtering from this feature and deferred both the memory-reader/data investigation and an eventual Men / Women / Both control to `.wiki/TODO.md`.
- 2026-08-12: Filter-modal changes must be transactional. Typing and other draft edits remain query-silent; Done applies once, while cancellation and dismissal discard the draft.
- 2026-08-12: Both Current and Potential role scores must be available for display and Search filtering. Potential values remain calculated only when a column, sort, or filter requires them.
- 2026-08-12: League-country flags were removed from scope. Nationality flags must cover every stored nationality in the array, including second and later values.
- 2026-08-12: Repeated and multi-column potential-score latency is addressed with the sparse versioned SQLite cache in ADR-0019. One request shares a player projection across all missing requested roles; unchanged later reads reuse the rows.
- 2026-08-12: Stored nationality values are full FM names, so flag rendering requires an explicit alias map rather than treating the value as an ISO code.

## Completed work

No implementation commits completed.

## Final validation

**Feature review profile:** Sol xhigh — the feature combines a high-volume derived cache, migration, shared query compiler, two virtualized tables, persisted client preferences, and custom accessible header interactions. A cross-commit review must prioritize silent data errors and incomplete cohort behavior over styling nits.

Before feature-complete review:

- Run `./scripts/dev format` and confirm it introduces only intended formatting.
- Run `./scripts/dev test` and report the exact frontend file/test counts.
- Run `./scripts/dev check` and report frontend, Rust, migration, lint, format, and secret-scan evidence.
- Run `./scripts/dev smoke` and report the exact Playwright count.
- Run `./scripts/dev secrets --staged` at every checkpoint.
- Run `git diff --check <planning-ref>...HEAD` for the exact implementation range.
- On Windows with a representative current snapshot, record cold and warm potential-role filter/sort timings, request at least two potential fields together, verify warm reads do not add or rewrite cache rows, and verify a successful supported player boost invalidates only the changed player's rows.
- At 1280×800 and 1600×900, inspect Search and Squad vertical fill, horizontal overflow, sticky headers, resize behavior, no pagination controls, full-row activation, and the Planner/Tactic sibling workspaces.
- Verify profile/back navigation preserves Search filters/combine/sort and Squad workspace/sort, while both table column layouts survive route changes and app restart.
- Verify right-click and keyboard header menus, keyboard resize, focus restoration, last-column protection, active-sort removal, and multi-page arrow navigation.
- Verify every filter-editor control is query-silent before Done, Done issues one applied update, dismissal issues none, potential-score materialization cannot start from draft state, and filter-derived columns are inserted only when the draft is applied.
- Verify mapped FM nationality aliases, all four UK home nations, second and later nationality flags, full-name hover labels, unknown fallback, offline bundling, and the absence of league flags.
- `./scripts/dev bridge-test` is not part of this feature's affected validation because no bridge or C# path may change. `./scripts/dev mutate` remains unsupported and must not be reported as passed.

## Documentation impact

- During feature reconciliation, update `.wiki/ARCHITECTURE.md` for the shared player-metric boundary, v21 derived cache, invalidation map, dynamic Search/Squad DTOs, virtual paging, and per-table Zustand state.
- During feature reconciliation, update `.wiki/DESIGN.md` for full-height table containment, fixed resizable columns, header context menus, categorized metric picker, staged Done/Cancel filter editing, and the approved nationality-flag exception to the current no-flag guidance.
- Keep [ADR-0019](../../decisions/0019-lazy-potential-role-score-cache.md) with the cache commit and update its implementation references at close-out.
- Move this ledger to completed features and restore `.wiki/TODO.md` to no active feature only after final validation, feature review, and documentation reconciliation.
- No CONCEPT, memory-reader, dump-schema, bridge, league, or deployment documentation change is planned in this feature.

## Publication plan

```yaml
status: active
pr_status: not_published
merge_status: not_merged
pr_ref: "Not published"
merge_ref: "Not merged"
branch: feature/configurable-player-tables
base_branch: main
publication_provider: GitHub
pr_template: .github/pull_request_template.md
merge_method: squash
required_checks: strict_check
required_check_name: check
pr_count: 1
build_feature_loop_profile: terra_max
feature_close_out: not_run
feature_review_profile: sol_xhigh
ci_repair_attempts: 0
```
