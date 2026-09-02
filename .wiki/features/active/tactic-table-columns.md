# Tactic Columns for Player Tables (Search, Moneyball, Shortlist)

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** 766c95399660f89696a1d2e9e7ee2c9bcd2c6b49aa7cfe6b3749ffee4c5c9668

## Intent

Add tactic-aware player evaluation to the three player tables (`search`, `moneyball-search`, `shortlist`) by exposing the active save's Planner tactic as optional display-only columns. Each column represents one of the 11 Planner lanes, showing a blended IP+OOP adjusted score derived from the lane's roles, weight, fit, and foot rules. Users can append a "Current" group (11 columns), a "Potential" group (11 columns), or both interleaved, ordered by `TACTIC_POSITION_ORDER`. This closes the loop between tactic design (Planner) and player search: after configuring a tactic, the user can immediately rank any player pool by how well each player fits each lane in the current or potential basis, without leaving Search.

## User-visible behavior

- In every Search view (General, Moneyball, Shortlist) the filter strip area shows two toggle buttons next to `Edit Filters`: `Add Tactic (Current)` and `Add Tactic (Potential)`.
  - Inactive state: secondary/ghost button, not highlighted.
  - Active state: highlighted (primary/filled or `primary-container` per design system) when the table layout contains all 11 lane IDs of that group.
- Clicking an inactive button:
  - Resolves the active save ID from `savesQuery` (route-owned, save-scoped acquisition). If no active save, show inline error `"No active save — configure a save before adding tactic columns"` and add no columns.
  - Fetches the tactic + options via save-scoped queries `plannerTacticQueryOptions(activeSaveId)` and `plannerTacticOptions` (gated on `activeSaveId`, reset on save/snapshot change). If either query is pending/error or `validateTacticDraft` returns a non-null error string, show an inline error in the panel (or toast via existing app feedback) — do not add columns.
  - Otherwise clears any partial/stale columns of that group from `columnIds` for that `tableId`, then appends the full 11 IDs for that group to the far-right of `columnIds` in `TACTIC_POSITION_ORDER` order (IP position primary, OOP tie-break via `compareTacticPositions`) via the store's atomic layout-replace action.
  - Marks the button active. Persistence is via `usePlayerTableStore` per `tableId`: `search`, `moneyball-search`, `shortlist`.
- Clicking a fully active button removes all 11 IDs of that group from `columnIds` via the same atomic store action and marks that button inactive. Removing a group when both were active re-compacts the remaining group to straight tactic order (no interleaving gaps). Interleaving is only applied when both groups are active: iteration over lanes in `orderedTacticLanes` (i.e. `TACTIC_POSITION_ORDER`) emits `tactic_current.<laneId>` then `tactic_potential.<laneId>` per lane at the far-right edge, after any non-tactic columns. When only one group is active, that group's 11 IDs appear straight in tactic order at the far-right.
- Removing any single tactic column via the header `X` (existing `onRemoveColumn` in `PlayerTableHeader`) immediately deactivates its group's button, because the group is no longer full (active iff all 11 IDs of that group are present in `columnIds` for that table), and re-compacts the surviving tactic block to deterministic straight order contiguous at the far-right (no interleaving gaps). The header-X path and the toggle paths share the same atomic store action `replaceLayout(tableId, nextColumnIds)` which replaces `columnIds` and prunes `widths` in one validated write.
- Column identity:
  - ID pattern: `tactic_current.<laneId>` and `tactic_potential.<laneId>` (e.g. `tactic_current.goalkeeper`, `tactic_potential.right_winger`). Lane IDs are the canonical `TACTIC_LANE_IDS` defined in `src/utils/tactic-ids.ts` (neutral shared owner, single source) and re-exported by `src/features/planner/types/tactic.ts` via `import { TACTIC_LANE_IDS } from "@/utils/tactic-ids"`; both are asserted equal to `src-tauri/src/features/planner/tactic.rs` `DEFAULT_LANE_IDS`).
  - Never appears in filter list, metric picker (`PLAYER_METRICS` / `MONEYBALL_SEARCH_METRICS`), or `FilterAst` validation. They are synthetic display-only columns owned by the neutral `src/utils/tactic-ids.ts` (IDs, prefix constants, lane allowlist, `isTacticColumnId`/`isValidTacticColumnId`) and `src/features/search/utils/tactic-columns.ts` (view-model helpers that import only neutral shared modules and `src/utils/position-order.ts`, never `@/features/planner`).
- Column header label format: `"{IP Position} ({IP Role DisplayName}) / {OOP Position} ({OOP Role DisplayName})"` — e.g. `"MC (Deep-Lying Playmaker) / DM (Screening Defensive Midfielder)"`. The label view-model is produced in `src/app/routes/search.tsx` (which owns Planner imports: `validateTacticDraft`, `plannerTacticQueryOptions`, `TacticOptions` role `displayName` lookup) and passed as `laneLabels: Map<laneId, string>` into Search components. Search helpers never import Planner internals; shared/stores never import features (enforced by `biome.json` `noRestrictedImports`).
- Column rendering: right-aligned, default width 112, `ScoreBadge` for numeric score, `"—"` (em dash in `on-surface-variant`) for unavailable. Unavailable when: combined score missing, or eligibility fails (see below), or Moneyball mapping has no counterpart for this lane/position (see Moneyball contract).
- Column sorting: clicking the header sorts by combined numeric score. Nulls (`"—"`) are last regardless of direction, consistent with `Club DNA` and `Moneyball role` null-last ordering in `src-tauri/src/features/search/query.rs`. Direction toggles asc/desc with `players.uid ASC` tie-breaker. Sort field acceptance is route-owned and precedes UI exposure: `isVisibleSortField` in `src/features/search/utils/dynamic-columns.ts` plus `validateSearch` in `src/app/routes/search.tsx` accept a canonical tactic sort ID only when that exact ID is present in the current view's persisted `layouts[tableId].columnIds` (synthetic allowlist, not `getFilterField`). Invalid, removed, or wrong-table tactic sorts are rejected and fallback to `defaultSearchSort(view)`. This is validated against tour: header sort, direct URL restoration with valid tactic ID present, URL with removed/invalid/wrong-table ID falls back, and header-X removal navigates away from a removed tactic sort.
- No-snapshot composition: the route does not early-return before filters/toggles when `!snapshot`. The filter strip (`SearchFilterBar`) plus `TacticColumnToggles` (from commit 7) are rendered above the results panel regardless of snapshot presence; only `SearchResultsPanel` shows the `EmptyState` / `Loading` branch. Tactic toggles remain operable (not `disabled`); click when no snapshot shows `"No snapshot loaded — use Load Data"` via a gated handler and adds no columns, without mutating layout. Save switching resets tactic query keys (activeSaveId-gated) and does not leak columns across saves because layout is per `tableId` not per save (documented).
- Eligibility & scoring (mirrors `src-tauri/src/features/planner/fit.rs` `lane_fit_score` plus `combine_role_scores`, fit adjustments):
  - Require base familiarity `>=12` on both IP and OOP base positions (`base_position(placement)` normalizes `DCR/DC/DCL->DC`, `DMCR/DM/DMCL->DM`, `MCR/MC/MCL->MC`, `AMCR/AMC/AMCL->AMC`, `STCR/ST/STC/STCL->ST`) else `"—"`.
  - Strict foot mismatch -> `"—"` when `foot_matches` is false and `footPreference === "strict"`.
  - Otherwise blended score = `combine_role_scores(ipRoleScore, oopRoleScore, lane.ipWeight)` then penalties: soft foot mismatch `-5` (when `preferredFoot != any` and `footPreference === "preferred"` but foot does not match), familiarity `<16` => `-5` per phase (up to `-10`), total max `-15`, `saturating_sub` at 0.
  - Source scores:
    - For Search (General) and Shortlist (General metrics): Current group reads `role.<ipRoleId>` / `role.<oopRoleId>` from `player_role_metrics` compact current columns; Potential group reads `potential_role.<ipRoleId>` / `potential_role.<oopRoleId>` with fallback `age >= 29 -> current` (same as `features/player` `get_player` potential contract). Shortlist uses General attribute-based scores.
    - For Moneyball view: same lane roles/weights but reads `moneyball_role.<roleId>` derived scores via the Moneyball cohort path (`score_role`). If a tactic role ID has no matching Moneyball catalog entry for this lane's normalized position tag, score is `None` -> `"—"`. Mapping is deterministic (see Moneyball contract below), not a doc gap.
- Persistence: `usePlayerTableStore` layouts per `tableId`. Synthetic IDs are validated via the neutral allowlist (`isValidTacticColumnId` from `src/utils/tactic-ids.ts`) and excluded from `getPlayerMetric`/`getMoneyballSearchMetric` allowlists. Widths persisted, clamped `72..360`. New synthetic columns participate in `sanitizeLayout` retention and reorder/move logic. The store exposes one atomic action `replaceLayout(tableId, nextColumnIds)` (or `setLayout`) that validates the incoming ID list against the neutral allowlist + metric allowlists, deduplicates, replaces `columnIds` wholesale, prunes `widths` to retained IDs in a single `set`, and falls back to `defaultLayout(table).columnIds` when the result would be empty (tactic-only toggle-off) — used by toggles and by tactic-aware header-X removal to avoid interleaved partials.
- Table targets: `search` (General), `moneyball-search`, `shortlist`. Squad (`squad`) is out of scope.
- Error paths: no active save, no snapshot, no tactic, or invalid tactic -> button click shows error via gated handler (button remains operable, not `disabled`) and adds no columns. Switching saves invalidates and refetches tactic queries (activeSaveId-gated). An empty snapshot's zero-row cohort is handled by existing empty-state in `SearchResultsPanel`; toggles remain rendered (operable with gated handler when no snapshot/snapshot mismatch) and tactic columns produce 0 rows when activated after snapshot loads.

## Invariants

- Tactic columns are display-only: they never enter `FilterAst`, never become a `MetricField` for filters, never appear in the categorized metric picker, and never participate in `getPlayerMetric`/`getMoneyballSearchMetric` validation for filters.
- Column group membership is derived from layout: a group (`current` or `potential`) is active iff all 11 canonical lane IDs of that prefix are present in `columnIds` for that `tableId`. No separate persisted boolean.
- Store truth: `PlayerTableId` remains the persisted owner; no new persisted table ID. Synthetic IDs live inside the existing `layouts[<tableId>].columnIds`.
- Module-boundary invariant (enforced by `biome.json` `noRestrictedImports`): `src/utils/**` and `src/stores/**` never import `@/features/**` or `@/app/**`; `src/features/**` never imports `@/features/*/**` or `@/app/**`. The canonical 11 lane IDs and synthetic-ID predicates live in the neutral shared module `src/utils/tactic-ids.ts` (or equivalently `src/utils/tactic-positions.ts`). `src/features/search/utils/tactic-columns.ts` imports only that neutral module and `src/utils/position-order.ts`. All Planner imports (`validateTacticDraft`, `plannerTacticQueryOptions`, `TacticOptions`/`PlannerTactic` types) live in `src/app/routes/search.tsx` which composes Search components by passing a normalized view model (`orderedLanes`, `laneLabels`, `laneRoles`, `onToggle` callbacks). Stores import only the neutral module.
- Lane ID truth is single-source: `src/utils/tactic-ids.ts` defines the 11 canonical lane IDs; `src/features/planner/types/tactic.ts` imports and re-exports them (no mirroring, no duplication); both are asserted equal to `src-tauri/src/features/planner/tactic.rs` `DEFAULT_LANE_IDS` (length+value) at test time.
- Score truth: computation is single-source. For General/Shortlist the Rust resolver owns sort ordering and numeric value; the frontend renders `dynamicValues[fieldId]` as a number or null. For Moneyball the same resolver owns the cohort before scoring. The frontend never recomputes scores for sort.
- Moneyball mapping is deterministic, not a gap: select the Moneyball definition by `(attribute_role_id == lane.ipRoleId / oopRoleId) AND (normalized tactic placement tag ∈ definition.position_tags)` where normalization is `tactic.rs::base_position` on the lane's `ipPosition`/`oopPosition`. The catalog `src-tauri/src/features/moneyball/builtin_role_definitions_v1.json` has 88 Moneyball defs (77 map via `attribute_role_id` to a General role, 11 are Moneyball-only with `attribute_role_id: null`), and all 68 General `role_id`s are represented. Matching by that compound key is unique and covers 103/111 General `(role_id, base_position)` combos. The 8 uncovered combos are unavailable by design (return `None` -> `"—"`): `holding_wing_back_oop`+`DL`, `holding_wing_back_oop`+`DR`, `pressing_wing_back_oop`+`DL`, `pressing_wing_back_oop`+`DR`, `box_to_box_midfielder_ip`+`MC`, `box_to_box_playmaker_ip`+`MC`, `deep_lying_playmaker_ip`+`MC`, `second_striker_ip`+`ST`. Tests must cover one mapped numeric and one uncovered `NULL` case and assert the mapping contract.
- Null handling is invariant: unavailable scores render `"—"` with `ScoreBadge` omitted, are last in sort, and use `UID ASC` tie-breaker.
- Familiarity and foot rules are invariant with `fit.rs` / `combine.rs`: correct base-position normalization, thresholds, and penalty values. Age fallback `>=29` uses current as potential.
- Interleaving invariant: with a single active group, order equals `orderedTacticLanes` in tactic-position order; with both active, order is deterministic interleaved `cur,pot` per lane still respecting tactic-position order. Removing a group or a single header-X when both were active re-compacts the surviving tactic block to contiguous straight order at the far-right via the single atomic store action (no interleaving gaps). Deterministic post-X ordering: `nonTacticIds` in persisted order + `straightTacticIds(survivingGroup, orderedTacticLanes)` in tactic-position order.
- Sort-acceptance invariant: a tactic `sortBy` is visible only when the exact field ID is present in the persisted layout for the current view's `tableId`; otherwise `validateSearch` and `isVisibleSortField` reject it and the route falls back to `defaultSearchSort(view)` (or the panel's `removeColumn` sort fallback when header X removes the active sort).
- No snapshot history for tactic columns: sorting reads the effective current snapshot's compact metric row; historical snapshots are not tactic-scored.

## Non-goals

- No filter, sort-only-outside-SQL, or picker support for tactic columns.
- No new metrics in `player-metrics.ts` / `moneyball-search-metrics.ts` catalogs for tactic IDs; no addition to `FilterAst` or `player_metrics::resolver` allowlists for filters.
- No Squad player table (`squad`) tactic columns, no Planner depth matrix changes, no CSV import changes, no Club DNA interaction.
- No new migration, no schema change, no new IPC for saving layouts, no revision of `TACTIC_POSITION_ORDER`.
- No IndexedDB/SQLite layout migration beyond `sanitizeLayout` tolerance; stale tactic IDs are cleared as incomplete groups on next toggle via the atomic action, but legacy layouts remain loadable.
- No client-side full recompute of scores for sort; Moneyball tactic mapping uses the deterministic compound key above, not a new joint catalog, and uncovered combos intentionally remain `NULL`.
- No BepInEx, FM write, or bridge changes.

## Current-state map

- Relevant components:
  - `biome.json` — enforces `noRestrictedImports`: `src/features/**` forbids `@/app` and `@/features/*/**`; `src/utils/**` and `src/stores/**` forbid `@/app` and `@/features/**`. Prior plan violated this by having Search helpers import Planner internals and `use-player-table-store` import a Search helper. The corrected plan places canonical IDs/predicates in neutral `src/utils/tactic-ids.ts` and keeps Planner query/validation in `src/app/routes/search.tsx`.
  - `src/app/routes/search.tsx` — route owns `view` (`general`/`moneyball`/`shortlist`), `comparisonPool`, filters/combine, sort/dir, and `SearchFilterBar` + `SearchResultsPanel` composition; tab switch clears filters and resets sort. `validateSearch` uses `isVisibleSortField` which currently gates non-basic sorts on `getFilterField` success — therefore excludes synthetic tactic IDs, so tactic sort cannot pass today. The route also early-returns `<Panel>No data loaded…</Panel>` when `!snapshot` before rendering the filter strip, contradicting the promise that tactic toggles remain with zero rows.
  - `src/features/search/components/search-results-panel.tsx` — `SearchResultsPanel` owns `usePlayerTableStore(tableId)` layout (`search`/`moneyball-search`/`shortlist`), derives `columns` via `tableColumnForMetric` + `metrics` prop override, computes `requestedFields` from non-basic columns, runs `searchPlayersQueryOptions` committed/requested sort-replacement cache, renders `VirtualizedPlayerTable` + `PlayerTableHeader` + `ScoreBadge`/`"—"` cells; `tableColumnForMetric` dispatches on `MONEYBALL_SEARCH_METRICS` vs `PLAYER_METRICS`. `removeColumn` guards last column, handles sort fallback to `ca` or `moneyball.average_rating`.
  - `src/features/search/components/search-filter-bar.tsx` + `src/utils/player-metrics.ts` + `src/utils/moneyball-search-metrics.ts` — filter editor and categorized metric picker; every metric has `sortable` flag used by `getPlayerMetric` filter.
  - `src/stores/use-player-table-store.ts` — `PlayerTableId = search | moneyball-search | shortlist | squad | staff-search | my-staff | staff-shortlist`, `PlayerTableLayout { columnIds, widths }`, `PLAYER_TABLE_LAYOUT_VERSION=6`, `sanitizeLayout` filters by `getPlayerMetric(...).sortable` / `getMoneyballSearchMetric(...).sortable` plus identity columns, `migratePersistedState` handles version shifts, `addColumns`/`removeColumn`/`moveColumn`/`setColumnWidth` are the only mutators (no atomic `replaceLayout` yet, so toggle and header-X paths would be two writes risking interleaved partials); `merge` re-sanitizes persisted state.
  - `src/utils/position-order.ts` — `TACTIC_POSITION_ORDER` (GK, DR, DCR, DC, DCL, DL, WBR, DMCR, DM, DMCL, WBL, MR, MCR, MC, MCL, ML, AMR, AMCR, AMC, AMCL, AML, STCR, STC, STCL) plus `compareTacticPositions`, `orderedTacticPositions`.
  - `src/utils/tactic-ids.ts` (new neutral owner after commit 2) — `TACTIC_LANE_IDS` (11) canonical definition; `src/features/planner/types/tactic.ts` — `TACTIC_LANE_IDS` re-exported via `import { TACTIC_LANE_IDS } from "@/utils/tactic-ids"`, plus `TacticLane { laneId, ipWeight, importanceRank, preferredFoot, footPreference, ipPosition, ipRoleId, oopPosition, oopRoleId }`, `PlannerTactic`, `TacticOptions { placements, roles: TacticRoleOption[] }`.
  - `src/features/planner/utils/tactic-editor.ts` — `orderedTacticLanes` (sort by IP then OOP via `compareTacticPositions`), `validateTacticDraft(tactic, options): string | null`, `phasePosition`, `basePosition`, `roleLabel`, `linkedPositionDescription` pattern for header text.
  - `src/features/planner/api/planner-keys.ts` + `src-tauri/src/features/planner/tactic.rs` + `commands.rs` — `plannerKeys.tactic()` (`get_planner_tactic`), `plannerKeys.tacticOptions()` (`get_planner_tactic_options`), save-scoped `planner_tactic_lanes` table seeded with `default_tactic()` 11 linked lanes; `base_position` normalization and validation in Rust mirrors TS `basePosition`. Current `plannerTacticQueryOptions` in `src/features/planner/api/planner-tactic-query-options.ts` is save-independent (`queryKey: plannerKeys.tactic()` with no `activeSaveId`) while `get_planner_tactic` in `src-tauri/src/features/planner/commands.rs:461` resolves `service::active_save_id(&conn)` implicitly — therefore stale across save switches and not gated. Other planner queries (`plannerKeys.slotCandidates(activeSaveId, ...)`, `plannerKeys.roleReference(activeSaveId,...)`) are correctly save-scoped.
  - `src-tauri/src/features/player_metrics/resolver.rs` — `MetricField`, `MetricSource`, `MetricValueKind`, `MAX_REQUESTED_FIELDS=256`, closed-catalog `player_current_column`/`player_potential_column` via `compact.rs`, `sql_expression` + `sql_sort_expression_with_club_dna`, `DynamicValue`, `read_dynamic_value`, `scalar_metric`, `POSITION_KEYS`, `HIDDEN_ATTRIBUTE_KEYS`; `is_moneyball_search_field` gate for Moneyball mode.
  - `src-tauri/src/features/player_metrics/compact.rs` — 68 current + 68 potential nullable columns in `player_role_metrics` (140 total cols), `PROJECTION_MODEL_VERSION=2`, `SCORE_MODEL_VERSION=1`, `player_metrics_join`, `assert_read_models_complete`, `prepare_player_derived`, `persist_rows_borrowed`.
  - `src-tauri/src/features/scoring/combine.rs` — `combine_role_scores(ip, oop, ipWeight) -> Option<u8>`, rejects non-finite/out-of-range weight, `DEFAULT_IP_WEIGHT=0.5`, round to nearest.
  - `src-tauri/src/features/planner/fit.rs` — `lane_fit_score`, `phase_fit_score`, `foot_matches`, `suitable_familiarity` (>=12), `foot_penalty` (strict -> None, preferred mismatch -> 5), `familiarity_penalty` (<16 -> 5), `adjust_score` saturating.
  - `src-tauri/src/features/search/query.rs` + `filter.rs` + `search/mod.rs` — `SearchView`/`ComparisonPool`, `SortField`/`SortDir`, `PlayerSummary { uid, name, age, nationalities, club/division, ca/pa, dynamicValues, moneyballPercentiles }`, `search_players_in_view` with `player_metrics_join`, Club DNA left join nullable-last sort, filtered `INNER JOIN player_moneyball_stats shortlist ON (snapshot_id, player_uid)` for Shortlist, Moneyball `moneyball_role.*` scoring via `role_score.rs`; `SortField::Dynamic(MetricField)` owns sortable dispatch; `ComparisonPool::FullCsv` vs `Filtered` affects percentiles.
  - `src-tauri/src/features/moneyball/role_catalog.rs` + `role_score.rs` + `src/utils/moneyball-search-metrics.ts` + `src/utils/moneyball-role-catalog.ts` — Moneyball role scoring with distinct catalog (`src-tauri/src/features/moneyball/builtin_role_definitions_v1.json`: 88 definitions, 77 with `attribute_role_id` pointing at a General role, 11 Moneyball-only `attribute_role_id: null` families, all 68 General `role_id`s represented; see Moneyball contract in Invariants). `moneyball_role.<id>` dynamic handling in resolver produces `MetricSource::MoneyballRole` -> needs bounded cohort then `score_role`.
  - `src/components/player-table/player-table-header.tsx` + `virtualized-player-table.tsx` — header owns sort caret, `aria-sort`, move/add/remove via context menu, resize handles (keyboard + pointer), virtual 50-row paging.
  - `src/components/ui/score-badge/score-badge.tsx` — `ScoreBadge` renders integer 0-100 with tier ramp.
- Data model:
  - `player_role_metrics(snapshot_id, uid, score_model_version, projection_model_version, 68 current cols, 68 potential cols)` — one row per current player per effective snapshot; accessed via `player_metrics_join`.
  - `player_moneyball_stats(snapshot_id, player_uid, statistics_json, percentiles_json)` — snapshot-owned cumulative Moneyball rows; joined for Moneyball/Shortlist.
  - `players(snapshot_id, uid, attributes_json, positions_json, preferred_foot, age, ca/pa, current_club/division, ...)` — `positions_json` holds `GK..WBL/WBR` familiarity values (0-20), `preferred_foot` is `"left"|"right"|"either"`.
  - `planner_tactic_lanes(save_id, lane_order, lane_id, ip_weight, importance_rank, preferred_foot, foot_preference, ip_position, ip_role_id, oop_position, oop_role_id)`.
  - No new table for tactic columns; tactic column state lives in `localStorage` key `fm-valuescout-player-table-layouts` via Zustand persist.
- Persistence and migrations:
  - `usePlayerTableStore` persisted under `PLAYER_TABLE_LAYOUT_STORAGE_KEY` with version `6`; `defaultPlayerTableLayouts` gives defaults per `tableId`; `sanitizeLayout` deduplicates and validates against metric `sortable`, retains unknown `metricId.length>0` for staff tables, clamps widths to `72..360`; migrations handle dedup of `club`/`division` and identity-only cases.
- Existing behavioral assumptions:
  - `tableColumnForMetric(metricId, view)` returns `undefined` when metric unknown in that view — column is filtered out before render.
  - `SearchResultsPanel` builds `requestedFields` from non-basic columns; `parse_requested_fields_for_moneyball` requires `moneyball` flag for Moneyball mode; General/Shortlist use attribute-based `role.*`/`potential_role.*` columns.
  - Moneyball Search has its own persisted layout, closed catalog; `Shortlist` reuses General read model but restricts rows to `INNER JOIN player_moneyball_stats`.
  - Search sort uses whitelist + `sql_sort_expression_with_club_dna`; null-last via `... IS NULL ASC` for Moneyball roles and Club DNA.
  - Tactic tactic-options `displayName` is authoritative for role label; `TacticLane` weight/foot/positions/roles are validated both in Rust `validate_tactic` and TS `validateTacticDraft`.
- Architectural seams:
  - Frontend store (`usePlayerTableStore`) <-> panel (`SearchResultsPanel`) via `columnIds`/`widths` observables.
  - Frontend IPC (`searchPlayersQueryOptions` -> `search_players_in_view` via `tauri-client.ts`) with `SearchView`/`ComparisonPool` and `requested_fields` string array.
  - `player_metrics::resolver` owns the trusted `MetricField` parse -> `MetricSource` -> SQL expression expansion. Every new sortable column must extend `MetricSource` and its `sql_expression*` methods, and for Moneyball roles the cohort post-score path in `query.rs`.
  - `planner/tactic.rs` owns save-scoped tactic retrieval/validation; `plannerKeys.tactic` is the frontend read (but currently not save-scoped — see risk).
  - Scoring: `scoring/catalog.rs` owns role IDs, `scoring/combine.rs` blends IP+OOP, `planner/fit.rs` applies familiarity/foot adjustments.
- Project validation commands:
  - `./scripts/dev check` (Biome + `tsc -b` + secretlint + `cargo fmt --check` + Clippy + `(cd src-tauri && cargo test)`) — commit gate (Rust package is `app` / `app_lib`, not `fm-valuescout`; there is no cargo workspace at the repo root, so bare `cargo test` at root is invalid).
  - `./scripts/dev check-app` (Biome + `tsc -b` + full-tree secretlint) — frontend CI gate.
  - `./scripts/dev check-rust` — Rust gate (runs `cargo fmt --check`, clippy, and `cargo test` via the `src-tauri` manifest).
  - `./scripts/dev test [target...]` — `vitest run`.
  - `./scripts/dev smoke` — Playwright smoke with IPC stub.
  - Classifiers are positional installed scripts at `/home/jonas/projects/PI_SETUP/scripts/ledger_state.py` and `/home/jonas/projects/PI_SETUP/scripts/delivery_state.py` (with `publication_state.py` alongside) — not `scripts/ledger_state.py` or `.pi/scripts/ledger_state.py`.
  - Branch reality at planning time: work is on `release/v0.15.0`; `main` is behind `origin/main`. The ledger records `feat/tactic-table-columns` as the intended short-lived branch, but branch activation from a synchronized `main` is reserved for the delivery workflow (`/skill:workflow-deliver-feature`), not planning. This plan remains on the current branch until delivery activates the recorded branch.
- Primary risks:
  - Tactic column IDs colliding with filter/metric validation or being persisted unsafely (must be allowlisted only for table layout, not filters) — mitigated by neutral allowlist and `biome.json` boundary.
  - SQL sort correctness for null-last combined adjusted scores (distinct from scalar/Club DNA patterns).
  - Double-counting or wrong prerequisites (positions_json shape / missing foot strings).
  - Moneyball role ID gap is bounded to the 8 uncovered `(role, position)` combos above; remaining 103 are uniquely mapped.
  - Interleaving vs straight-order re-compaction bug when toggling or header-X removes one lane while both groups active — mitigated by the single atomic `replaceLayout` action.
  - Tactic validation error UX inconsistent with existing Search panel errors.

## Feature architecture

- Frontend ownership:
  - `src/utils/tactic-ids.ts` (new, neutral shared owner) — single owner of the synthetic column contract's identity: constants `TACTIC_CURRENT_PREFIX = "tactic_current."`, `TACTIC_POTENTIAL_PREFIX = "tactic_potential."`, canonical `TACTIC_LANE_IDS` (defined here as the single source; `src/features/planner/types/tactic.ts` imports and re-exports it — no mirroring), helpers `isTacticColumnId`, `tacticGroupForColumnId`, `tacticLaneIdForColumnId`, `isValidTacticColumnId`, `allTacticColumnIdsForGroup(group, orderedLanes)`, `isFullTacticGroup(columnIds, group)`, `sanitizeTacticIds(columnIds)` to validate lane suffix, and `TACTIC_COLUMN_DEFAULT_WIDTH = 112`. No React, no Zustand, no `@/features` or `@/app` imports.
  - `src/features/search/utils/tactic-columns.ts` — view-model helpers that import only `src/utils/tactic-ids.ts` and `src/utils/position-order.ts` (and pure `planner/types` lane type as a type-only import is forbidden by `biome.json` — therefore the module operates on plain `{laneId, ipPosition, oopPosition}` shapes or the route passes ordered lane IDs directly). Provides `buildTacticColumnOrder(orderedLaneIds, currentActive, potentialActive)` returning interleaved vs straight array, `tacticColumnHeaderLabelParts` helpers are NOT here — header labels are produced in the route from Planner `TacticOptions.displayName`. This module has no `@/features/planner` runtime import.
  - `src/stores/use-player-table-store.ts` — extend `sanitizeLayout` to retain synthetic IDs via `isValidTacticColumnId` from the neutral module for `tableId` in `search | moneyball-search | shortlist` (still rejected on squad/staff). Add one atomic action `replaceLayout(tableId, nextColumnIds: string[])` (or `setLayout`) that validates every ID against the combined allowlists (`getPlayerMetric`/`getMoneyballSearchMetric` or `isValidTacticColumnId`), deduplicates preserving first-seen order, replaces `columnIds` wholesale, and prunes `widths` to retained IDs with `clampWidth`. All toggle and header-X tactic paths use this single action to avoid interleaved partials. Keep `defaultPlayerTableLayouts` unchanged (no default tactic columns). Store does NOT implement group toggling logic beyond accepting/rejecting synthetic IDs and exposing the atomic replace.
  - `src/app/routes/search.tsx` — owns all Planner imports (`validateTacticDraft` from `src/features/planner/utils/tactic-editor.ts`, `plannerTacticQueryOptions`/`plannerKeys.tacticOptions`, `TacticLane`/`TacticOptions` types). Resolves `activeSaveId` from `savesQuery`, creates save-scoped `plannerTacticQueryOptions(activeSaveId)` (enabled only when `activeSaveId != null`, query key includes `activeSaveId`, reset on save/snapshot change). composes the filter strip to always render `SearchFilterBar` + `TacticColumnToggles` even when `!snapshot` (only `SearchResultsPanel` branches to empty-state). Passes a normalized view model to `TacticColumnToggles` and `SearchResultsPanel`: `orderedLaneIds: string[]` (derived via `orderedTacticLanes` + `compareTacticPositions` using IP primary / OOP tie-break), `laneLabels: Map<laneId, string>` (formatted `"{IP Position} ({IP Role DisplayName}) / {OOP Position} ({OOP Role DisplayName})"` per lane from `TacticOptions.roles[].displayName`), `currentActive`/`potentialActive` derived from layout, and `onToggle(group)` / `onRemoveTacticColumn` callbacks that compute `nonTacticIds + tacticSuffix` via `buildTacticColumnOrder` and call `replaceLayout`. Search components never import Planner internals directly.
  - `src/features/search/components/search-results-panel.tsx` + `src/features/search/components/tactic-column-toggles.tsx` (new) — panel renders the tactic column space: `tableColumnForMetric` gets a fallback branch that returns a synthetic `PlayerTableColumn { id, label: laneLabels.get(laneId) ?? laneId, align: "right", width: widths[id] ?? TACTIC_COLUMN_DEFAULT_WIDTH }` when `isTacticColumnId(id)` and `isValidTacticColumnId` is known, otherwise filters out unknown tactic IDs. Cell rendering uses `ScoreBadge` when `typeof dynamicValue === "number"` else `"—"` with variant. Sorting dispatches through `requestedFields` inclusion; the panel passes synthetic IDs through `requestedFields` sorted alongside other non-basic columns. `TacticColumnToggles` is a presentational component receiving the normalized view model and callbacks (no direct Planner or store import beyond the neutral predicates).
  - `src/utils/role-catalog.ts` + `src/utils/position-order.ts` — reused without change for ordering; no duplication.
  - `src/features/search/utils/dynamic-columns.ts` — extend `isVisibleSortField` plus route `validateSearch` to accept canonical tactic sort IDs only when present in the current view's persisted tactic layout (`layouts[tableId].columnIds` includes the exact `tactic_current.*` / `tactic_potential.*` ID). General tactic sorts are not gated on `getFilterField`. Invalid/removed/wrong-table tactic sort IDs are rejected and fallback to `defaultSearchSort(view)`. This change lands before the toggle UI commit so any exposed UI sort is already trunk-safe.
- Backend ownership:
  - `src-tauri/src/features/player_metrics/resolver.rs` — extend `MetricSource` with `TacticCurrent { laneId, roleIds }` and `TacticPotential { laneId }` or a unified `Tactic { basis: Current|Potential, laneId }` that does not resolve to a single compact column but to a computed expression over two compact columns + positions/ foot; add `parse_tactic_field_id(field, moneyball)` helper that returns a `MetricField` for `tactic_current.*` / `tactic_potential.*` when `moneyball` is false-or-both depending on view (see view gating). Provide `is_tactic_field` guard, `tactic_column_width` defaults, and ensure `MetricField::kind` is `Integer` (0-100 nullable), `sql_expression_with_club_dna` routes to a placeholder that `query.rs` replaces with a joined computed expression rather than a single column.
  - `src-tauri/src/features/search/query.rs` (and `src-tauri/src/features/squad/query.rs` is NOT touched — Squad stays without tactic columns):
  - Extend prerequisites discovery: detect `uses_tactic_current` / `uses_tactic_potential` from `requested_fields` and `sort_by`. Call `assert_read_models_complete` with `require_score_model = uses_tactic_current || uses_tactic_potential` and `require_projection_model = uses_tactic_potential`.
  - Fetch tactic once per query: `let tactic = load_tactic(conn, save_id)?` (save-scoped). If fetch fails (no save or tactic corruption), surface `Err` that the frontend will render as error banner (distinct from layout mutation error; both use same string when invalid tactic was allowed past validation — defensive second check).
  - For Moneyball view: route tactic columns through the same bounded cohort path that `moneyball_role.*` uses when `view == Moneyball` and tactic column is present. Select the Moneyball role def deterministically by `(attribute_role_id, base_position(phasePosition))` where `base_position` is `tactic.rs::base_position` and `phasePosition` is the lane's `ipPosition` / `oopPosition` normalized. For uncovered combos return `None` -> `NULL`. Reuse `filtered_moneyball_percentiles` + `score_role` per lane then `combine + adjust`; do not duplicate Moneyball cohort logic. Ensure `ComparisonPool` selection (`FullCsv` vs `Filtered`) flows through. Document the 8 uncovered combos in an inline comment and in ledger artifacts — no standalone docs-only commit.
  - For General/Shortlist: for each tactic field in `dynamicFields` + `sort_by`, build one `CASE` expression per lane via a helper `tactic_sql_expression(player_alias, lane, group, age_col, positions_json_col, foot_col, current_col, potential_col, ip_weight)` that mirrors `combine_role_scores` + `fit` penalties. Keep null-last: `ORDER BY (<expr> IS NULL) ASC, <expr> <dir>, players.uid ASC` via the existing `order_sql` branch for tactic sort.
  - Map each requested tactic column into `dynamic_values[fieldId]` as `Some(Integer(score_i64))` or `None`. For `Potential` with age fallback: expression uses `CASE WHEN players.age >= 29 THEN player_metrics.<current_col> ELSE player_metrics.<potential_col> END` for each of the two role reads before combine; alternatively evaluate fallback in Rust per-row after fetching both columns. Choose single-expression path to keep SQL sort correct.
  - `src-tauri/src/features/search/filter.rs` — ensure no tactic ID passes `compile_filters`; this is enforced because `MetricField::parse_for_moneyball` accepts tactic fields but `parse_filter_ast` rejects any rule whose `field` starts with `tactic_current.`/`tactic_potential.` with an `unknown field` error. Document this rejection as intentional.
  - `src-tauri/src/features/scoring/combine.rs` + `src-tauri/src/features/planner/fit.rs` — reuse as pure fns, guarded by `#[cfg(test)]` coverage of penalties and saturation; do not duplicate logic.
- Query & sort seam:
  - `SearchPlayersRequest { view, requested_fields, sort_by, sort_dir }` — `requested_fields` includes tactic IDs when columns are visible; server returns them in `dynamic_values[fieldId]` as `Option<i64>` (value) where `Some(INT)` maps to `DynamicValue::Integer`.
  - `ORDER BY <sort_expression> <dir>, players.uid ASC` with null-last for tactic columns (`... IS NULL ASC` branch similar to Club DNA / Moneyball role).
- Cross-cutting constraints:
  - No WebView-supplied SQL: `MetricField::parse_*` remains the only path that produces a tactic column SQL expression.
  - Width 112 is owned by `src/utils/tactic-ids.ts` default, still clamped via store's `clampWidth`.
  - Error string from `validateTacticDraft` is user-visible; preserve its exact wording for the inline message.

## Uncertainty register

### Known

- Exactly one layout bucket per tableId: `search`, `moneyball-search`, `shortlist`. Squad not included.
- 11 lane IDs are fixed in `TACTIC_LANE_IDS` / `DEFAULT_LANE_IDS`; `TACTIC_POSITION_ORDER` ordering uses `compareTacticPositions` with OOP tie-break. Both are repo-truth.
- Scoring penalties: soft foot `-5`, familiarity `<16` => `-5` per phase, max `-15`, `saturating_sub(0)`.
- Familiarity threshold `>=12` required on both base positions, strict foot mismatch produces unavailability.
- Age fallback `>=29` potential reads current visible attributes — implemented in `features/player::get_player` and `prepare_player_derived`.
- Moneyball mapping is deterministic (evidence: `src-tauri/src/features/moneyball/builtin_role_definitions_v1.json` 88 defs, 77 with `attribute_role_id`, 11 Moneyball-only, all 68 General `role_id`s represented; compound key `(attribute_role_id, TACTIC_POSITION_ORDER base via tactic.rs::base_position)` is unique and covers 103/111 General `(role_id, base_position)` combos; 8 uncovered combos enumerated in Invariants). Frontend and Rust share `base_position` normalization.

### Assumptions

- The existing `SearchResultsPanel` error pattern (retry button + message) is acceptable for tactic validation errors. The worker must inspect current Search error rendering before choosing inline vs toast and keep the chosen pattern in the commit.
- Synthetic IDs `tactic_current.<laneId>` / `tactic_potential.<laneId>` are globally unique and cannot collide with future catalog metrics because catalog metrics use prefixes `role.`, `potential_role.`, `attr.`, `pos.`, `moneyball.`, `moneyball_role.`, `club_dna`, etc. The worker must assert this in `src/utils/tactic-ids.ts` with a compile-time or unit test.
- TypeScript `TacticOptions.roles` `displayName` is the desired header displayName without further formatting (not a lowercase-normalized variant).

### Decisions

- One PR is the correct delivery boundary. Justification: no migration, no new IPC service that must land before UI can merge safely, no separate library that benefits from independent review. Every commit remains trunk-safe via synthetic-ID allowlisting and feature-flag-free panels; a two-PR split would only add branch overhead.
- Neutral shared ownership for tactic identity: `src/utils/tactic-ids.ts` owns the canonical 11 lane IDs, prefix constants, synthetic-ID predicates, lane allowlist check, and default width. `src/features/search/utils/tactic-columns.ts` (if retained) imports only that neutral module plus `src/utils/position-order.ts` and never `@/features/planner`. Route `src/app/routes/search.tsx` owns all Planner imports and passes a normalized view model (ordered lane IDs, laneLabels, lane role/position data, toggle callbacks) into Search components. Stores import only the neutral module. Rejected: Search helper importing Planner internals and stores importing Search helper (violates `biome.json` `noRestrictedImports`; fixed by this decision).
- Display-only column definition is owned by `src/utils/tactic-ids.ts` (identity) plus `src/features/search/utils/tactic-columns.ts` (view-model helpers, no Planner import) on the frontend and by `player_metrics::resolver` (backend) as synthetic metric IDs — never in `PLAYER_METRICS` / `MONEYBALL_SEARCH_METRICS` / filter allowlists. Rejected alternative: adding them to `player-metrics.ts` catalog — would incorrectly make them filterable and conflate the global picker with a tactic-local header label that depends on the active tactic.
- Single atomic store action `replaceLayout(tableId, nextColumnIds)` for all tactic layout mutations (toggle append/remove, interleaving/strip, and tactic-aware header-X removal). Replaces `columnIds` wholesale and prunes `widths` in one validated write, guaranteeing the surviving tactic block is contiguous straight order at the far-right after any X or toggle-off while both were active. Rejected: successive `removeColumn`/`addColumns` calls that leave interleaved partials.
- Rust-owned SQL sort for tactic columns (single `CASE` expression per field) is preferred over client-side post-sort. Rejected alternative: compute in frontend and sort via JS — would require requesting additional fields (positions/foot/role scores per lane) and would break SQL pagination sort stability.
- Tactic header label format is exactly `"{IP Position} ({IP Role DisplayName}) / {OOP Position} ({OOP Role DisplayName})"` per lane; no abbreviation or dual-line stacking, consistent with the dispatch. Labels are produced in the route, not in shared/search helpers.
- Interleaving rule: when both groups active, iterate `orderedTacticLanes(tactic.lanes)` (i.e. tactic-position order) and emit `current` then `potential` per lane. Only when both active; otherwise straight order at far-right via the atomic action.
- Save-scoped tactic acquisition: `plannerTacticQueryOptions(activeSaveId)` with `queryKey: plannerKeys.tactic(activeSaveId)` (or equivalent save-scoped key), `enabled: activeSaveId != null`, and `queryFn` that calls `fetchPlannerTactic` gated on the active save. Invalidates/resets on `activeSaveId` / `snapshot.id` change. Route forbids tactic mutation when `activeSaveId` is null (no active save) and composes filter strip + toggles even when `!snapshot` (only panel branches). Rejected: original save-independent `plannerTacticQueryOptions` that resolves `active_save_id` implicitly in `get_planner_tactic`.
- Route sort acceptance before UI exposure: `isVisibleSortField` + `validateSearch` accept a canonical tactic sort only when the exact ID is present in the current view's persisted layout (`layouts[tableId].columnIds`). Invalid/removed/wrong-table IDs fallback to `defaultSearchSort(view)`. This commit precedes toggle UI so every exposed sort is already trunk-safe.
- Fold the speculative docs-only gap commit into implementation commit 5 plus durable ledger/docs: comment the Moneyball-`NULL` branch and the 8-case invariant directly in `resolver.rs`/`query.rs`; do not create a standalone docs-only commit and do not copy catalog counts into `src/utils/tactic-ids.ts`.
- Tooling facts: ledger/delivery classifiers are positional scripts at `/home/jonas/projects/PI_SETUP/scripts/ledger_state.py` and `/home/jonas/projects/PI_SETUP/scripts/delivery_state.py`; Rust crate is `app` (`app_lib`) with no repo-root Cargo manifest — run Rust tests via `(cd src-tauri && cargo test <filter>)` or `--manifest-path src-tauri/Cargo.toml`, never `cargo test -p fm-valuescout` or bare `cargo test` at root. Review branch is `release/v0.15.0` not `main`; `main` is behind `origin/main`; the intended feature branch `feat/tactic-table-columns` is recorded as intent but not activated during planning — activation from a synchronized `main` is reserved for `/skill:workflow-deliver-feature`.

### Unknowns

- Whether `position_order.ts` `compareTacticPositions` with IP-primary + OOP tie-break already matches the design's intent vs pure IP order — the plan specifies `compareTacticPositions(ipPosition)` primary, OOP only for ties, which aligns with `orderedTacticLanes`.

### Risks

- Resolver/SQL expression for tactic columns expands to 22 possible dynamic fields (11 lanes x 2 bases) all referencing `json_extract(positions_json, ...)` and two compact columns each. A naive per-column expression duplicates identical subexpressions and risks query size/CPU growth for clients that request all 22 columns. Mitigation: the helper generates a minimal `CASE` with shared `json_extract` per base position for the two shapes; cap total `requested_fields` at `MAX_REQUESTED_FIELDS=256` keeps it bounded; add a unit test that asserts generated SQL length stays under a budget for 22 columns.
- Store persistence leakage: a user who manually edits `localStorage` could inject arbitrary `tactic_current.*` IDs with unknown lane suffixes. Mitigation: sanitization filters unknown lane suffixes via neutral allowlist (`TACTIC_LANE_IDS` set) in `sanitizeLayout` and `replaceLayout`.
- Moneyball filtered vs full-CSV scoring interaction: tactic Moneyball columns reuse the same `ComparisonPool`; ensure tactic `moneyball_role.*` underlying scores respect `ComparisonPool::Filtered` computation path when `comparisonPool === "filtered"`.
- Race between tactic save and Search toggle: a concurrent tactic edit invalidates `plannerKeys.tactic(activeSaveId)` mid-render. Mitigation: Search toggle reads tactic from Query cache snapshot at click handler start and validates before layout mutation; a post-mutation tactic invalidation is not reverted.
- Existing `getPlayerMetric` callers (filter validation in `SanitizeLayout`) must not treat synthetic IDs as unknown metrics to drop — the store's sanitize must retain them when their lane suffix is valid; regression would silently strip tactic columns after refresh.
- Sort validation must not regress basic metrics: `isVisibleSortField` must still delegate to `getFilterField` for non-tactic IDs; tactic acceptance is an allowlist branch checked first against `layouts[tableId].columnIds`.

## Walking skeleton

The thinnest end-to-end path that proves the approach:

1. Add `src/utils/tactic-ids.ts` with lane constants, `isTacticColumnId`/`isValidTacticColumnId` guard, and `TACTIC_COLUMN_DEFAULT_WIDTH`, plus a test that asserts prefix non-collision and allows only the 11 canonical lane suffixes.
2. Extend `usePlayerTableStore` with `sanitizeLayout` retaining valid synthetic IDs via the neutral predicate and one atomic `replaceLayout` action that replaces `columnIds` and prunes `widths` in one write, with one unit test proving `sanitizeLayout` keeps `tactic_current.goalkeeper` and the atomic action prunes correctly.
3. Extend `player_metrics::resolver` to parse `tactic_current.*` / `tactic_potential.*` into a guarded `MetricSource::Tactic` (not filter-eligible) and extend `SearchResultsPanel.tableColumnForMetric` to render synthetic columns via the route-passed `laneLabels` map with width `TACTIC_COLUMN_DEFAULT_WIDTH` and `ScoreBadge`/`"—"`.
4. In `src-tauri/src/features/search/query.rs` wire one tactic column to a Rust-generated `CASE` SQL expression (deterministic Moneyball mapping via `attribute_role_id + base_position` for Moneyball view, compact columns for General) that reads the two role columns + positions/foot and emits `NULL` vs `0..100`, then verify virtual paging and null-last sort for that single column across General and Moneyball (mapped numeric vs uncovered `NULL`). The two Search toggle buttons and interleaving polish are not required for the skeleton; one hardcoded lane proves the Rust/UI contract.
5. Proof is a `vitest run` targeting `src/stores/use-player-table-store.test.ts` plus `(cd src-tauri && cargo test --lib -- resolver)` and `(cd src-tauri && cargo test --lib -- search::query)` and `(cd src-tauri && cargo test --lib -- planner::fit)` (or `./scripts/dev check-rust`) with one new test each showing synthetic column retention, header label formation, and valid SQL expression for a tactic sort. Separate filtered Cargo commands are required — `cargo test` accepts only one `TESTNAME` filter.

## Delivery plan

### PR 1 — Tactic columns for player tables (Search, Moneyball, Shortlist)

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** feat/tactic-table-columns

**Base branch:** main

**Publication provider:** GitHub

**PR template:** .github/pull_request_template.md

**Merge method:** squash

**Required checks:** GitHub required checks

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(search): add tactic lane columns for player tables`

**Purpose:** Ship tactic-aware evaluation as optional display columns on the three player tables (Search General, Moneyball, Shortlist) with persistence, interleaving, sorting, and blended adjusted scoring. Single PR because there is no cross-cutting migration or dependency that demands an independent review seam; every commit stays trunk-safe without a feature flag. The single docs-only gap commit from the prior draft is folded into implementation commits 5 and 7 plus durable ledger/docs.

**Depends on:** None beyond current `main`. The branch `feat/tactic-table-columns` is intended but not activated during planning — activation from a synchronized `main` is reserved for `/skill:workflow-deliver-feature` so the plan remains read-only. Current work is on `release/v0.15.0`; activation from a synchronized `main` is handled by `/skill:workflow-deliver-feature` before publication (no manual `rebase` in planning).

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(tactic-table-columns): record approved feature plan`

**Work:** Commit the independently reviewed planning artifacts on the feature branch before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, and unrelated documentation.

**Implementation packet:**

- Preserve the accepted plan-review outcome. Commit only the reviewed planning paths after branch verification.

**Files and responsibilities:**

- `.wiki/features/active/tactic-table-columns.md` — approved feature intent, delivery plan, and packets.
- `.wiki/TODO.md` — move entry to Active with link to the ledger (transfer the resolved developer intent table into ledger Intent verbatim).
- `.wiki/BACKLOG.md` — no change (deferred scope not introduced).
- `.wiki/features/planned/tactic-table-columns.md` — remove after its accepted content is preserved, when present (if absent, record no-op).
- `.wiki/decisions/<id>-tactic-table-columns-display.md` — rejected as unwarranted ADR: decision is tactic-column display plumbing, not a durable structural choice with meaningful alternatives across releases; rationale lives in ledger Uncertainty register instead. No ADR is created.

**Behavior and data flow:**

- Move planning truth from the planned source into one reviewed active ledger and record the exact delivery sequence before implementation.

**Ordered implementation steps:**

1. Verify the active branch and base without changing Git state (`git rev-parse --abbrev-ref HEAD`, `git merge-base --is-ancestor`).
2. Confirm the worktree contains only the reviewed planning paths (`git status --porcelain`).
3. Run the ledger classifier at its installed positional path `/home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/tactic-table-columns.md` (if the classifier is absent, record the gap and do not block the planning-only commit, but log the missing validator).
4. Stage and inspect the exact planning diff for independent checkpoint review (`git diff --cached --stat`, `--check`).

**Tests and proof:**

- Not applicable — this commit changes planning documents only. The ledger classifier and documentation checks prove structural consistency when available.

**Patterns to verify:**

- The active-ledger template, current TODO/BACKLOG ownership rules, and that no implementation path is staged.

**Constraints and non-goals:**

- Do not alter implementation, tests, executable configuration, plan scope, packet order, or reviewed decisions.

**Dependencies and sequencing:**

- Requires an accepted plan-review verdict, developer acceptance, a valid Delivery fingerprint when the classifier is available, and exact branch activation via the delivery workflow (not planning).

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/tactic-table-columns.md` when available plus `ls .wiki/features/active/tactic-table-columns.md`; otherwise record validator gap.

**Stop conditions:** Stop on an uncleared review, a classifier error, an unreviewed path, a substantive post-review plan change, or a branch mismatch.

**Review mandate:** Verify that the staged diff contains the complete reviewed planning outcome and no implementation or unrelated files; that TODO linking is correct; and that no ADR was created without justification.

#### Commit 2 — Add neutral tactic-identity helpers and ordering

**Status:** Completed

**Provisional commit:** `feat(search): add neutral tactic column helpers`

**Work:** Add the single neutral owner of the synthetic column identity and lane ordering without yet wiring store persistence or IPC. No Planner or app imports.

**Size assessment:** ~120 non-test lines. Within the soft target.

**Out of scope:**

- Store sanitization for synthetic IDs, Rust resolver, Search query integration, toggle buttons, Moneyball mapping, route composition.

**Implementation packet:**

**Files and responsibilities:**

- `src/utils/tactic-ids.ts` (new, neutral shared owner) — defines canonical `TACTIC_LANE_IDS` (single source) with `TACTIC_CURRENT_PREFIX = "tactic_current."`, `TACTIC_POTENTIAL_PREFIX = "tactic_potential."`, `TACTIC_COLUMN_DEFAULT_WIDTH = 112`, lane allowlist `Set<string>`, helpers `isTacticColumnId(id)`, `tacticGroupForId(id)`, `tacticLaneIdForId(id)`, `tacticColumnId(group, laneId)`, `isValidTacticColumnId(id)` (allowlist check), `allTacticColumnIdsForGroup(group, orderedLaneIds)` (11 IDs in tactic-position order), `isFullTacticGroup(columnIds, group)`, `sanitizeTacticIds(columnIds)` filter. Assert equality with `src-tauri/src/features/planner/tactic.rs` `DEFAULT_LANE_IDS` at test time.
- `src/features/planner/types/tactic.ts` (edit existing) — remove local `TACTIC_LANE_IDS` array definition; add `import { TACTIC_LANE_IDS } from "@/utils/tactic-ids"` and re-export it (`export { TACTIC_LANE_IDS } from "@/utils/tactic-ids"` or `export const TACTIC_LANE_IDS = ...imported...`). No other type changes. Verifies `biome.json` boundary (neutral imports only upward) and single source is preserved. Added to this commit per HIGH1.
- `src/utils/tactic-ids.test.ts` (new) — unit tests: valid vs invalid lane suffix (`not_a_lane` rejected), `isFullTacticGroup` for 10 vs 11, prefix non-collision with `role.`/`moneyball_role.`/`attr.` etc, and lane count 11 matches Rust source (`DEFAULT_LANE_IDS`) and re-exported `src/features/planner/types/tactic.ts`.
- `src/features/search/utils/tactic-columns.ts` (new, feature-local but neutral-import-only) — pure helpers `buildTacticColumnOrder(orderedLaneIds, currentActive, potentialActive)` returning deterministic interleaved vs straight array, and `orderedLaneIdsForColumns(orderedTacticLanes)` that delegates to `compareTacticPositions` with OOP tie-break (no `features/planner` import). Consumes only `src/utils/tactic-ids.ts` and `src/utils/position-order.ts`. Unit-tested for interleaved order `GK Cur, GK Pot, DR Cur, DR Pot,…` and straight order.
- `src/utils/position-order.ts` — no change; read-only ordering contract.
- `src/features/planner/utils/tactic-editor.ts` — no change; verified read-only for `orderedTacticLanes` shape but not imported from shared/search.

**Behavior and data flow:**

- Modules are pure: no React, no Zustand, no IPC, no `@/features/planner` or `@/app` imports. `tactic-ids` is importable by stores and feature helpers without violating `biome.json` boundaries. Header label formation is NOT in these helpers — it is composed in the route passing a normalized view model.

**Ordered implementation steps:**

1. Create `src/utils/tactic-ids.ts` with every exported helper and lane allowlist derived from the canonical IDs. Add `TACTIC_COLUMN_DEFAULT_WIDTH = 112` export. This is the single source for `TACTIC_LANE_IDS`.
2. Edit `src/features/planner/types/tactic.ts` to remove its local `TACTIC_LANE_IDS` array and replace with `import { TACTIC_LANE_IDS } from "@/utils/tactic-ids"` + re-export. Verify `rg -n "@/features" src/utils/tactic-ids.ts` is empty and `src/features/planner/types/tactic.ts` now imports from `@/utils/tactic-ids` only (allowed: features may import from `@/utils`).
3. Create `src/features/search/utils/tactic-columns.ts` with `buildTacticColumnOrder` and ordered-lane helper delegating to `compareTacticPositions` so drift with Planner ordering is impossible.
4. Add unit tests for both modules covering: lane allowlist, `isFullTacticGroup`, interleaved vs straight, assertion that synthetic ID prefixes cannot collide with real catalog prefixes (visual grep in test), and that `src/utils/tactic-ids.ts` `TACTIC_LANE_IDS` equals `src-tauri/src/features/planner/tactic.rs` `DEFAULT_LANE_IDS` and equals the re-export from `src/features/planner/types/tactic.ts`.
5. Run `vitest run` and `tsc -b --noEmit`. Verify `rg -n "from \"@/features" src/utils/tactic-ids.ts` is empty and `rg -n "from \"@/features/planner" src/features/search/utils/tactic-columns.ts` is empty.

**Tests and proof:**

- New tests listed above; expected RED before implementation is missing module/exports. GREEN asserts exact ordering against `TACTIC_POSITION_ORDER` slice and lane allowlist of 11. No obsolete test asset to remove; deliberately retain existing planner utils tests.

**Patterns to verify:**

- `src/utils/position-order.ts` `compareTacticPositions` stable sort contract. Mirror its import, do not re-implement ordering.
- `biome.json` `noRestrictedImports` patterns — verify no shared→features or features→features cross-import remains.

**Constraints and non-goals:**

- Do not import across features (no `@/features/planner` from shared or from search helper beyond allowed shared modules). Tactic-columns must not import `validateTacticDraft` or Planner query options — those stay in `app/routes`.
- Do not add synthetic metrics to `PLAYER_METRICS` / `MONEYBALL_SEARCH_METRICS`.
- Do not expose `TacticLane` type through the neutral module (use `laneId: string` shapes only).

**Dependencies and sequencing:**

- Depends on commit 1 (ledger). No prior implementation commit.

**Validation:** `./scripts/dev check-app` (Biome + `tsc -b`), `./scripts/dev test src/utils/tactic-ids.test.ts src/features/search/utils/tactic-columns.test.ts`, `rg -n "@/features/planner|@/app" src/utils/tactic-ids.ts src/features/search/utils/tactic-columns.ts` must show no import (Biome check would fail if present).

**Stop conditions:** Stop if `TACTIC_LANE_IDS` vs `tactic.rs` `DEFAULT_LANE_IDS` drift is observed (counts or values mismatch) — escalate to ledger discovery before continuing.

**Review mandate:**

1. Synthetic IDs cannot be constructed from arbitrary WebView strings (allowlist is exactly 11 lane IDs).
2. No `@/features/planner` or `@/app` import in `src/utils/tactic-ids.ts` or `src/features/search/utils/tactic-columns.ts` (Biome would error).
3. Ordering delegates to `compareTacticPositions` rather than duplicating sort.
4. Module has no React/Zustand/IPC import.
5. Prefix `tactic_current.` / `tactic_potential.` does not collide with `role.` / `moneyball_role.` checked by grep/test.
6. Tests assert null vs truncated ordering difference.
7. Exports include `isValidTacticColumnId` and `TACTIC_COLUMN_DEFAULT_WIDTH` for store/panel reuse.
8. No behavior is wired into panels yet (isolated helper commit).

#### Commit 3 — Persist synthetic tactic layouts with an atomic replace action

**Status:** Completed

**Provisional commit:** `feat(search): persist tactic column layouts atomically`

**Work:** Make store accept, retain, and re-hydrate synthetic tactic column IDs with valid widths; add one atomic layout-replace action used by all tactic mutations. No other toggle or query logic yet.

**Size assessment:** ~120 non-test lines + ~80 test lines. Within the soft target.

**Out of scope:**

- Rust resolver extension, panel rendering of synthetic columns, toggle button UI, Moneyball scoring, IPC changes, validation UX, route sort gating.

**Implementation packet:**

**Files and responsibilities:**

- `src/stores/use-player-table-store.ts` — extend `sanitizeLayout` for `tableId` in `search | moneyball-search | shortlist`:
  - Accept `tactic_current.<laneId>` / `tactic_potential.<laneId>` when `isValidTacticColumnId` from `src/utils/tactic-ids.ts` holds (lane in `TACTIC_LANE_IDS`), otherwise drop with same dedup logic as existing metrics. Ensure `getPlayerMetric`/`getMoneyballSearchMetric` are NOT consulted for synthetic IDs (they would reject them); synthetic validation is the neutral allowlist only.
  - Preserve `widths` for synthetic IDs with `clampWidth`; drop widths for IDs not in retained `columnIds` (as existing logic does). Keep `defaultLayout` defaults unchanged (no default tactic columns).
  - Add one atomic action `replaceLayout(table: PlayerTableId, nextColumnIds: readonly string[])` (name `setLayout` or `replaceLayout`) that:
    - Validates every `nextColumnIds` entry: either `getPlayerMetric(...).sortable` / `getMoneyballSearchMetric(...).sortable` or `isValidTacticColumnId` for the three tactic-capable tables (otherwise reuses existing `metricId.length>0` for staff tables).
    - Deduplicates preserving first-seen order (`indexOf` guard as in `sanitizeLayout`).
    - Replaces `layouts[table].columnIds` wholesale to `nextColumnIds` (no append), rebuilds `widths` to exactly retained IDs via `clampWidth`, and handles the empty-result fallback: if deduplicated `nextColumnIds` is empty (e.g. tactic-only layout where toggling off the sole full group would leave zero columns), replace with `defaultLayout(table).columnIds` instead of rejecting or persisting empty; otherwise enforces the single-column guard (`length===1` cannot be removed via caller — action rejects empty before fallback unless fallback applies).
    - Is the only tactic mutation path; existing `addColumns`/`removeColumn` remain for non-tactic callers but tactic toggle/header-X code must use this atomic action.
  - Keep `de` and `moveColumn`/`setColumnWidth` behavior for tactic IDs (width 112 default is owned by neutral module, clamped via store's `clampWidth`).
- `src/stores/use-player-table-store.test.ts` — add cases: hydrates valid synthetic group from persisted state, drops invalid lane suffix, clamps width 10_000 -> 360, retains after `sanitizePersistedState` re-run, asserts that a full group of 11 is kept verbatim, that `replaceLayout(table, [...nonTactic, ...straightTactic])` atomically replaces and prunes widths in one write (no intermediate partial visible) including the recompaction case, and that a tactic-only layout (`columnIds` equals exactly one full `tactic_current` group) toggling off via `replaceLayout(table, [])` falls back to `defaultLayout(table).columnIds` (not empty, not rejected) — per MEDIUM4.

**Behavior and data flow:**

- Persisted JSON -> `sanitizePersistedState` -> `sanitizeLayout` retains synthetic IDs when lane valid. `layouts[tableId].columnIds` now may contain tactic IDs; every other `tableId` (squad, staff) rejects them.
- `replaceLayout(table, nextIds)` is a single Zustand `set` that swaps `columnIds` and rebuilds `widths` atomically.

**Ordered implementation steps:**

1. Import `isValidTacticColumnId` from `src/utils/tactic-ids.ts` into `use-player-table-store.ts` (verify `rg -n "@/features" src/stores/use-player-table-store.ts` is empty — neutral import only).
2. Modify `sanitizeLayout` filter branch for the three tableIds to `metrics allowlist OR isValidTacticColumnId`.
3. Add `replaceLayout` atomic action as described.
4. Add unit test fixtures: persist a JSON with `search: { columnIds: ["name","tactic_current.goalkeeper", ...11 current, "tactic_potential.goalkeeper"...], widths: { "tactic_current.goalkeeper": 999, "tactic_current.not_a_lane": 120 } }` and assert retained vs stripped vs clamped, plus atomic replace pruning.
5. Run targeted vitest + `tsc -b`. Verify Biome `noRestrictedImports` passes (stores must not import features).

**Tests and proof:**

- Updated `use-player-table-store.test.ts` with four+ tactic cases listed above plus atomic replace action test; expected RED is `"unknown metric"` style drop before change. GREEN asserts retained 11+width-clamp, invalid suffix stripped, and atomic replace prunes widths without leaving interleaved partial.

**Patterns to verify:**

- Existing `sanitizeLayout` dedup and width-strip pattern — copy exactly for synthetic branch.
- `clampWidth(72..360)` reuse; default width 112 is owned by `src/utils/tactic-ids.ts` not store.
- Keep `PLAYER_TABLE_LAYOUT_VERSION` unchanged; retention is a compatible read of existing persisted JSON (no migration needed). If version bump is needed, justify in commit notes and add `migratePersistedState` entry — avoid speculating.

**Constraints and non-goals:**

- Do not expose tactic columns in the metric picker filter list — store path must not call `getPlayerMetric` for them.
- Do not add default tactic columns; do not change `defaultLayout` for any table.
- Do not write interleaving logic here beyond the atomic replace (panel toggle owns the order computation; store merely persists whatever order the route computes via `replaceLayout`).

**Dependencies and sequencing:**

- Depends on commit 2 (neutral helpers).

**Validation:** `./scripts/dev check-app`, `./scripts/dev test src/stores/use-player-table-store.test.ts`, `./scripts/dev test src/utils/tactic-ids.test.ts src/features/search/utils/tactic-columns.test.ts`.

**Stop conditions:** Import cycle detected between store and tactic helpers -> re-route `isValidTacticColumnId` into a tiny shared `src/utils/tactic-ids.ts` only (already the case). If store were to import planner types, revert.

**Review mandate:**

1. Synthetic IDs retained only on the three player tables, rejected on squad/staff.
2. Invalid lane suffix stripped and its width removed.
3. Dedup preserved (`all.indexOf` guard still applies).
4. Width clamping reused, not duplicated limit.
5. No bump of `PLAYER_TABLE_LAYOUT_VERSION` without a documented reason and migration.
6. `getPlayerMetric` not called for synthetic branch (neutral allowlist only).
7. Persisted-state round-trip via `sanitizePersistedState` retained.
8. No `@/features` import in `src/stores/use-player-table-store.ts` (Biome rule).
9. Atomic action replaces `columnIds` and prunes `widths` in one write, usable by toggles and header-X.

#### Commit 4 — Extend Rust resolver for synthetic tactic metric fields

**Status:** Completed

**Provisional commit:** `feat(player-metrics): support tactic metric fields in resolver`

**Work:** Teach `MetricField::parse_for_moneyball` to recognize `tactic_current.<laneId>` / `tactic_potential.<laneId>` synthetic fields as valid sortable display columns, without making them filter-eligible.

**Size assessment:** ~130 Rust lines + ~40 test lines. Within the soft target.

**Out of scope:**

- Score computation, SQL expression generation, sort integration, filter compilation path, store/panel wiring, route sort gating.

**Implementation packet:**

**Files and responsibilities:**

- `src-tauri/src/features/player_metrics/resolver.rs` — extend `MetricSource` with `Tactic { group: TacticGroup, laneId: String }` (or two variants `TacticCurrent`/`TacticPotential`), add `TACTIC_PREFIXES` constants, `parse_tactic_field_id(field, allowMoneyball: bool)` that:
  - rejects unless `field` starts with `tactic_current.` / `tactic_potential.`,
  - extracts `laneId` and validates it is in `DEFAULT_LANE_IDS` (reusing `planner::tactic::DEFAULT_LANE_IDS` constant or a local allowlist mirroring it; prefer reusing the constant to avoid drift, add cross-module `assert_eq!` test comparing lengths/values),
  - validates lane id snake_case is already safe (it is), returns a new `MetricField { id, kind: Integer, source: MetricSource::TacticRegistered }`.
  - In `MetricField::parse_for_moneyball`, check tactic prefixes before delegation to Moneyball/scalar branches; for Moneyball mode, tactic fields are accepted as display columns in both views (General/Shortlist accepted via General path, Moneyball accepted via Moneyball path) but never as filter targets — tactic sort is routed via `MetricField` -> SQL, not via filter allowlist. Ensure the `unknown player metric` string for non-allowlist tactic suffix matches existing error shape (starts with `"unknown player metric:"` or a distinct `"unknown tactic lane"` — keep one stable string and document it).
  - Add `is_tactic_field()` / `is_tactic_current()` / `tactic_lane_id()` accessors.
  - Ensure `MetricField::kind() == Integer` for tactic fields, and `sql_expression` for this commit returns `"NULL"` (placeholder) — query commit will override with `sql_expression_with_*` specialization; keep placeholder so `MAX_REQUESTED_FIELDS` + parsing does not panic.
- `src-tauri/src/features/player_metrics/resolver.rs` tests — add `#[test]` cases: valid current+potential for every `DEFAULT_LANE_IDS` member, invalid suffix rejected, dedup via `parse_requested_fields*` preserves order, and `is_moneyball_search_field` still excludes tactic fields from filter eligibility (tactic fields are not Moneyball search fields).
- `src-tauri/src/features/search/filter.rs` — verify filter rejection (no code change expected beyond confirming tactic prefix is not in Moneyball search field allowlist). If filter path currently accepts unknown fields as `MetricField` and then filters them, add an explicit guard that `field.starts_with("tactic_")` is rejected at `parse_filter_ast`. Keep change minimal.

**Behavior and data flow:**

- `parse_requested_fields_for_moneyball(&["tactic_current.goalkeeper"], false)` now `Ok(vec![MetricField { id: "tactic_current.goalkeeper", kind: Integer, source: Tactic { ... }}])`.
- `parseFilterAst` with `field: "tactic_current.goalkeeper"` still `Err("unknown filter field")` or similar — tactic columns never become filters.

**Ordered implementation steps:**

1. Create `TACTIC_LANE_IDS` allowlist tied to `planner::tactic::DEFAULT_LANE_IDS` (import or replicate with `assert_eq!` compile test).
2. Extend `MetricSource` enum and `MetricField::{is_tactic_field, tactic_group, tactic_lane_id}` accessors.
3. Implement `parse_tactic_field_id` and wire it in `parse_for_moneyball`.
4. Add the Rust unit tests listed above; ensure `MAX_REQUESTED_FIELDS` bound still enforced for tactic IDs.
5. Run `(cd src-tauri && cargo test resolver)` (or `cargo test --manifest-path src-tauri/Cargo.toml resolver`).

**Tests and proof:**

- New Rust tests assert every laneId valid for both prefixes, invalid `not_a_lane` rejected, `parse_requested_fields` dedup preserves order, and `compile_filters` rejects tactic rule. Expected RED before commit is `unknown player metric: tactic_current.goalkeeper`. GREEN provides same-ID roundtrip `sql_sort_expression` placeholder `"NULL"` (updated in next commit).

**Patterns to verify:**

- `MoneyballStatistic` / `MoneyballRole` branches as nearest analogues for conditional Moneyball gating and `sql_expression` dispatch.
- Shelved field case: `MetricSource::MoneyballRole` returns `"NULL"` for `sql_expression` intentionally — tactic fields copy that, but commit 5 replaces with real CASE.

**Constraints and non-goals:**

- Do not enable tactic fields as filter targets — `FilterAst` compile must continue to reject them.
- Do not derive `MetricField` via WebView SQL: pure allowlist+suffix check only.

**Dependencies and sequencing:**

- Depends on commit 3. Frontend helpers are not required for this Rust commit, but lane ID truth must align between TS and Rust.

**Validation:** `./scripts/dev check-rust` (includes `cargo fmt --check`, clippy, and `(cd src-tauri && cargo test)`), focused `(cd src-tauri && cargo test resolver)`.

**Stop conditions:** If `planner::tactic::DEFAULT_LANE_IDS` is not `pub` or cannot be reused without import cycle, replicate with a local `const` and add a cross-module `assert_eq!` test comparing lengths/values.

**Review mandate:**

1. Allowlist is the exact 11 lane IDs, not open-ended string suffix.
2. `MetricField::parse_for_moneyball` gate does not accidentally permit tactic fields as Moneyball metrics in `is_moneyball_search_field`.
3. Filter path rejects tactic rule (error string stable and user-visible not leaked as SQL).
4. Placeholder `NULL` expression does not escape into user-visible sort.
5. No new migration or column.
6. Tests cover all 11 lanes for both prefixes.
7. Error strings match existing `"unknown player metric: {field}"` shape.
8. `MAX_REQUESTED_FIELDS` cap still enforced.

#### Commit 5 — Implement tactic lane scoring and query sort (Rust, Moneyball mapping)

**Status:** Active

**Provisional commit:** `feat(search): compute tactic lane scores and sort`

**Work:** Realize the blended, familiarity/foot-adjusted score for each tactic column and wire it into `search_players_in_view` so sortable tactic columns produce numeric values for display and correct null-last `ORDER BY`. Record the deterministic Moneyball mapping contract and make uncovered combos return `NULL`.

**Size assessment:** ~180 Rust lines + ~90 tests. Near soft limit; atomicity (single scorer + SQL expression + sort wiring + mapping contract) justifies keeping it as one commit rather than splitting scorer and query. The former standalone gap doc commit is folded here: Moneyball mapping notes live as inline comments plus the uncovered-`NULL` test and ledger invariant, not as a separate commit.

**Out of scope:**

- Frontend toggle/interleaving UI, route sort validation, store atomic action beyond what exists, no-snapshot composition (owned by next commit).

**Implementation packet:**

**Files and responsibilities:**

- `src-tauri/src/features/planner/fit.rs` — add `pub(crate) fn tactic_adjusted_score(ip: Option<u8>, oop: Option<u8>, ip_weight: f64, player_foot: &str, positions: &BTreeMap<String, Option<i64>>, lane: &TacticLane) -> Option<u8>` composing `combine_role_scores` then `suitable_familiarity` check on both bases (`base_position` per `tactic.rs`), `foot_matches` strict-null path, then `familiarity_penalty` per phase (+ foot soft -5) with `adjust_score` saturation. Keep `lane_fit_score` private but verify same behavior; if `lane_fit_score` currently expects `score: Option<u8>` pre-combined, this new function is the lane helper that owns combine first. Preserve `age >= 29` fallback outside `fit.rs` (in compact column read selection), not inside `adjust_score`.
- `src-tauri/src/features/player_metrics/resolver.rs` — extend `sql_sort_expression_with_club_dna` for `MetricSource::Tactic` to return a Rust-generated `CASE` expression rather than `"NULL"`. For General/Shortlist builds the expression references `player_metrics.<ip_role_col>`, `player_metrics.<oop_role_col>`; include `players.age` read for potential fallback. For Moneyball view, the expression references `moneyball.statistics_json` vs percentile-derived role scores via the post-score path — determine in `query.rs` which branch applies and make resolver return `"NULL"` for Moneyball tactic fields while query post-scores them (alternatively return a sentinel that query substitutes).
- `src-tauri/src/features/search/query.rs` (and `src-tauri/src/features/squad/query.rs` is NOT touched — Squad stays without tactic columns):
  - Extend prerequisites discovery: detect `uses_tactic_current` / `uses_tactic_potential` from `requested_fields` and `sort_by`. Call `assert_read_models_complete` with `require_score_model = uses_tactic_current || uses_tactic_potential` and `require_projection_model = uses_tactic_potential`.
  - Fetch tactic once per query: `let tactic = load_tactic(conn, save_id)?` (save-scoped). If fetch fails (no save or tactic corruption), surface `Err` that the frontend will render as error banner (distinct from layout mutation error).
  - For Moneyball view: route tactic columns through the same bounded cohort path that `moneyball_role.*` uses when `view == Moneyball` and tactic column is present. Deterministically select the Moneyball role def by `(attribute_role_id, base_position(placement))` using `tactic.rs::base_position`; for the 8 uncovered combos emit `None` -> `NULL`. Reuse `filtered_moneyball_percentiles` + `score_role` per lane then `combine + adjust`; do not duplicate Moneyball cohort logic. Ensure `ComparisonPool` selection (`FullCsv` vs `Filtered`) flows through. Inline comment: `"// If tactic role id has no Moneyball catalog entry for this position tag, score is None -> \"—\" (by design — 8 uncovered combos enumerated in ledger Invariants)."`
  - For General/Shortlist: for each tactic field in `dynamicFields` + `sort_by`, build one `CASE` expression per lane via helper `tactic_sql_expression(...)` that mirrors `combine_role_scores` + `fit` penalties. Keep null-last: `ORDER BY (<expr> IS NULL) ASC, <expr> <dir>, players.uid ASC`.
  - Map each requested tactic column into `dynamic_values[fieldId]` as `Some(Integer(score_i64))` or `None`. For `Potential` with age fallback: expression uses `CASE WHEN players.age >= 29 THEN player_metrics.<current_col> ELSE player_metrics.<potential_col> END` for each of the two role reads before combine.
- `src-tauri/src/features/scoring/combine.rs` — no change, verified read-only.
- `src-tauri/src/features/planner/tactic.rs` — expose `base_position` as `pub(crate)` if not already (`pub(super)` today); make it `pub(crate)` so `fit.rs` and `query.rs` and Moneyball mapping share it. If visibility change is needed, keep it minimal and add a unit test that `base_position("DCR")=="DC"` holds in both modules.
- `src/utils/tactic-ids.ts` — add doc comment block referencing the Moneyball contract: `"88 Moneyball defs (77 mapped, 11 Moneyball-only), 103/111 General combos mapped, 8 unavailable by design"` and list the 8 combos for durable surface without a standalone commit. Single ownership for this doc comment per NITPICK (commit 7 must not duplicate it).
- `src-tauri/src/features/moneyball/role_catalog.rs` (extend existing tests, or `src-tauri/src/features/moneyball/tactic_mapping.rs` helper if extracted) — add regression proof for the deterministic Moneyball mapping contract (HIGH2): compound-key `(attribute_role_id, base_position)` uniqueness (no duplicate Moneyball def shares the same `(attribute_role_id, position_tag)` pair), 103/111 coverage (count of distinct General `(role_id, base_position)` combos that have a matching Moneyball def via that key), and exact 8 uncovered pairs (`holding_wing_back_oop`+`DL`, `holding_wing_back_oop`+`DR`, `pressing_wing_back_oop`+`DL`, `pressing_wing_back_oop`+`DR`, `box_to_box_midfielder_ip`+`MC`, `box_to_box_playmaker_ip`+`MC`, `deep_lying_playmaker_ip`+`MC`, `second_striker_ip`+`ST`) asserted as the precise set difference. Existing `role_catalog.rs` tests cover 88/77/11 but not compound uniqueness/coverage — these new tests close the gap. Keep the existing mapped-numeric and uncovered-NULL query tests as behavioural proof (see below); the catalog-level tests ensure the 103/111 + 8 invariant cannot silently drift.

**Behavior and data flow:**

- Example `tactic_current.goalkeeper` where lane = `goalkeeper` (GK/GK, weight 0.5): read `player_metrics.goalkeeper_ip` (current) + `player_metrics.line_holding_keeper_oop` (current OOP) as IP/OOP, combine 50/50, check `positions_json` has `GK >=12` both placements, check foot `any` -> no penalty, emit blended score 0..100 or NULL.
- For `tactic_potential.left_back`: read `player_metrics.potential_full_back_ip` + `player_metrics.potential_holding_full_back_oop` when `age <29`, else current cols; same foot/familiarity rules.
- Moneyball view: same lane weight/positions/foot, but IP/OOP role scores come from `moneyball_role_scores` cohort selected by the compound key; one uncovered combo yields `NULL`/`"—"`.

**Ordered implementation steps:**

1. Expose or reuse `base_position` as `pub(crate)` in `tactic.rs`; add compile guard asserting `TACTIC_LANE_IDS` alignment across TS/Rust.
2. Create `tactic_adjusted_score` pure function in `fit.rs` composing combine + penalties; add `fit_tests.rs` cases covering familiarity `<16` per phase, strict foot -> None, soft foot -5, saturation at 0.
3. Extend `resolver.rs` tactic `MetricSource` branch to produce a placeholder that `query.rs` replaces; keep `sql_expression_with_club_dna` for tactic returning a sentinel that `query.rs` detects and substitutes with the real CASE before building `select_sql` and `order_sql`.
4. In `query.rs` implement helper that, given `MetricField` tactic + `tactic: PlannerTactic`, returns the CASE expression and updates prerequisites. Ensure `player_metrics_join` predicates are already present for tactic columns. Include the 8-case `None` path for Moneyball.
5. Wire `dynamicFields` expansion to project each tactic column as one SELECT expression; wire sort expression for `SortField::Dynamic(tacticField)` with null-last and `players.uid ASC` tie-breaker.
6. Add Rust tests:
   - `fit.rs` cases for every penalty combination (both familiarities <16 => -10, plus foot soft => -15 -> saturates).
   - `query.rs` integration: ingest a fixture snapshot with 3 players via `ingest_dump_file` (use `memory_read/fixtures/golden_dump_v8.json` plus `prepare_player_derived` shape), call `search_players_in_view` with `requested_fields=[tactic_current.goalkeeper]` and assert one player's dynamicValue is `Some` blended and another with foot strict mismatch is `None`.
   - Moneyball path: two tests — one mapped combo yields numeric, one uncovered combo (`holding_wing_back_oop` at `DL` or `second_striker_ip` at `ST`) yields `NULL`/`"—"`, proving the mapping contract.
   - Mapping-contract regression (HIGH2, in `role_catalog.rs` or mapping helper): assert compound-key uniqueness (no duplicate `(attribute_role_id, position_tag)` among the 88 defs), 103/111 coverage, and exact 8 uncovered pairs enumerated above — deterministic set equality, not just count.

**Tests and proof:**

- New `fit_tests.rs` cases (positive + saturated). Integration query cases listed above; each expected RED before this commit is `unknown player metric: tactic_current.*` or `NULL` placeholder not sorting. GREEN asserts exact numeric score for at least one General `tactic_current` column and correct null-last sort, plus one uncovered Moneyball combo is `NULL`. The uncovered-`NULL` test is non-skippable per dispatch HIGH 2. Additionally, `role_catalog::tests` now prove compound-key uniqueness, 103/111 coverage, and exact 8 uncovered pairs — preventing silent catalog drift.

**Patterns to verify:**

- Existing `Club DNA` `LEFT JOIN` + `ORDER BY ... IS NULL` path as the null-last pattern to copy.
- `player_metrics_join` `require_score_model`/`require_projection_model` predicate builder — reuse without duplicating.
- Moneyball `search_players_with_roles` filtered vs full-CSV branching.
- `assert_read_models_complete` call before reading role metrics.

**Constraints and non-goals:**

- Do not add per-role indexes; rely on existing virtual paging and `SortField` SQL path.
- Do not change catalog or `VISIBLE_ATTRIBUTE_KEYS`; do not add tactic fields to moneyball metric catalog beyond the deterministic compound-key mapping.
- Do not make tactic columns groupable in OR filter AST.
- Keep one SQLite writer, one transaction semantics (applies only to read helper, not writes).

**Dependencies and sequencing:**

- Depends on commit 4.

**Validation:** `./scripts/dev check-rust` + `(cd src-tauri && cargo test query)` + `(cd src-tauri && cargo test fit)` + `./scripts/dev test` (ensures no frontend breakage from this backend-only commit).

**Stop conditions:** Strict foot check requires `preferred_foot` values outside `any/left/right/both` — abort if observed.

**Review mandate:**

1. SQL `CASE` does not interpolate WebView input (laneId allowlisted, role columns closed-catalog).
2. Familiarity base normalization matches `tactic.rs::base_position`, not frontend `basePosition` typo.
3. Penalties and saturation mirror `fit.rs` exactly (no off-by-5).
4. `age >= 29` fallback reads `players.age` column, not compact potential column for that row.
5. Moneyball tactic mapping uses `(attribute_role_id, base_position(placement))` compound key; missing-role -> `NULL` for the 8 uncovered combos, documented with inline comment and ledger invariant.
6. Sort `NULLS LAST` regardless of `ASC`/`DESC`, with `players.uid ASC` tie.
7. No new column in `player_role_metrics`; computation uses existing columns + `positions_json`/`preferred_foot` already in `players`.
8. Completeness guard fails before values are read (wrong-version compact row never leaks as NULL).

#### Commit 6 — Route sort validation, save-scoped tactic acquisition, and no-snapshot composition

**Status:** Pending

**Provisional commit:** `feat(search): gate tactic sort and save-scoped tactic load`

**Work:** Make tactic sorts route-valid before any UI toggles, fix tactic loading to be save-scoped and gated, fix the no-snapshot composition so filter strip + toggles remain, and prepare the tactic-aware header-X recompaction wiring to share the atomic store action.

**Size assessment:** ~150 non-test lines + ~60 tests. Within the soft target (near limit; validation + save-scope + composition kept together atomically because they are the route's single tactic contract).

**Out of scope:**

- Toggle button rendering and interleaving arithmetic (next commit), Rust scorer beyond what commit 5 provides, store atomic action definition beyond what commit 3 provides, metric picker promotion.

**Implementation packet:**

**Files and responsibilities:**

- `src/features/search/utils/dynamic-columns.ts` — extend `isVisibleSortField(value, _filters, view, layouts?)` or keep signature `(value, filters, view)` but add an overload / companion `isVisibleTacticSortField(value, tableId)` that consults `usePlayerTableStore.getState().layouts[tableId].columnIds` inclusion (or pass `columnIds` as param). Canonical tactic IDs `tactic_current.*` / `tactic_potential.*` are accepted only when the exact ID is present in the current view's persisted layout for that `tableId`; all other non-basic sorts still delegate to `getFilterField`. Keep the original `(value, filters, view)` fallback for non-tactic callers but ensure `validateSearch` uses the layout-aware path.
- `src/app/routes/search.tsx` — extend `validateSearch` to compute `tableIdForView(view)` and accept tactic `sort` only when `isValidTacticColumnId(sort) && layouts[tableId].columnIds.includes(sort)` (read via `usePlayerTableStore.getState()` or via a helper that avoids render-loop). Invalid/removed/wrong-table tactic sorts fallback to `defaultSearchSort(view)`. Wire `activeSave` from `savesQuery` into `plannerTacticQueryOptions(activeSaveId)` (and matching `plannerTacticOptionsQueryOptions(activeSaveId)`) whose `queryKey` is `plannerKeys.tactic(activeSaveId)` / `plannerKeys.tacticOptions(activeSaveId)` (enabled only when `activeSaveId != null`), with `fetchPlannerTactic` now taking `activeSaveId` as param or via closure; gate the query on `activeSaveId` and `snapshotContext` and invalidate/reset on `activeSaveId` / `snapshot.id` change. Refactor the `if (!snapshot)` branch to NOT early-return before the filter strip: render `<div class="flex flex-col gap-gutter"><SearchFilterBar .../><Panel empty-state or SearchResultsPanel /></div>` so filter bar remains mounted (tactic toggles themselves are created/rendered in commit 7; this commit only ensures the composition slot remains).
- `src/features/planner/api/planner-keys.ts` — change `tactic: () => [...]` to `tactic: (activeSaveId: number) => [...plannerKeys.all, "tactic", activeSaveId]` and `tacticOptions: (activeSaveId: number) => [...plannerKeys.all, "tactic-options", activeSaveId]` (no shim; migrate every consumer). Remove the zero-arg overload. Add `tactic`/`tacticOptions` now require `activeSaveId`.
- `src/features/planner/api/planner-tactic-query-options.ts` and `src/features/planner/api/planner-tactic-options-query-options.ts` — change to `export const plannerTacticQueryOptions = (activeSaveId: number | null | undefined) => queryOptions({ queryKey: plannerKeys.tactic(activeSaveId ?? -1), queryFn: () => fetchPlannerTactic(), enabled: activeSaveId != null })` (same for `plannerTacticOptionsQueryOptions`). No backward-compatible shim per MEDIUM5.
- `src/app/routes/my-club.tsx` — migrate loader to first `ensureQueryData(savesQueryOptions)` to obtain `activeSaveId`, then `ensureQueryData(plannerTacticQueryOptions(activeSaveId))` / `ensureQueryData(plannerTacticOptionsQueryOptions(activeSaveId))` when `activeSaveId != null` (no implicit active-save read). Replace any `useSuspenseQuery(plannerTacticQueryOptions)` with gated `useQuery(plannerTacticQueryOptions(activeSaveId))` (`enabled: activeSaveId != null`) because suspense options type excludes `enabled`. Update `my-club` tests accordingly.
- `src/features/planner/components/planner-tactic-editor.tsx` — migrate `queryClient.setQueryData(plannerKeys.tactic(), nextTactic)` to `queryClient.setQueryData(plannerKeys.tactic(activeSaveId), nextTactic)` (thread `activeSaveId` prop from `my-club.tsx` or read from `savesQuery`); update invalidation callers that use `plannerKeys.tactic()` to include `activeSaveId`.
- `src/app/routes/my-club-squad.test.tsx` (and any other `plannerKeys.tactic()` test consumers) — update `setQueryData` / `getQueryData` / `getQueryState` calls to `plannerKeys.tactic(activeSaveId)` with the fixture save id; add save-switch test coverage there as well.
- `src/features/search/components/search-results-panel.tsx` — adapt `removeColumn` (header X) to be tactic-aware: when the removed column is a tactic ID, compute `nonTacticIds = columnIds.filter(id => !isTacticColumnId(id))` plus surviving tactic group straight ordering, then call `replaceLayout(tableId, [...nonTacticIds, ...survivingStraight])` instead of `removeStoredColumn` alone, so the surviving group re-compacts without gaps. When the removed sort column was a tactic sort, apply the same sort fallback as existing `removeColumn` does (fallback to `ca` / `moneyball.average_rating` or first remaining column).
- `src/stores/use-player-table-store.ts` — no new width logic here (atomic action already in commit 3); this commit only wires the caller (`replaceLayout`) for the header-X path.

**Behavior and data flow:**

- URL entry with `?sort=tactic_current.goalkeeper` while that ID is not in `layouts[search].columnIds` (or is for `moneyball-search` while view is `general`) -> `validateSearch` rejects -> route navigates with `defaultSearchSort(view)` fallback, never issuing a tactic SQL sort that would error.
- Save switch `A -> B` -> `activeSaveId` changes -> `plannerTacticQueryOptions(activeSaveId)` key changes -> old tactic cache not reused -> toggles recompute `currentActive` from layout (which is per-table, not per-save) but header labels now reflect `B`'s tactic. Layout not mutated on save switch (documented).
- No snapshot: route renders filter strip (filter bar slot) with an inline message slot; panel shows `EmptyState No data loaded`. Commit 6 only validates the route composition keeps the filter strip mounted (toggles themselves are rendered and their gated-error handling is proven in commit 7).
- Header X on `tactic_current.left_back` while both groups active -> `replaceLayout(tableId, [...nonTactic, ...remainingStraight])` where `remainingStraight` is the straight survivor group's 11 IDs in tactic-position order, contiguous at far-right (no interleaving holes).

**Ordered implementation steps:**

1. Extend `dynamic-columns.ts` with the layout-aware tactic branch and add a companion export `isTacticSortVisible(value, tableId)` used by route.
2. Add save-scoped key/options in `planner-keys.ts` + `planner-tactic-query-options.ts` with `enabled` gate and tests that `queryKey` includes `activeSaveId`.
3. Refactor `search.tsx` `validateSearch` to accept only when tactic sort present in `layouts[tableId].columnIds`; wire `activeSaveId`-gated tactic query; refactor `!snapshot` early-return to keep filter strip + toggles mounted.
4. Wire `search-results-panel.tsx` header-X to `replaceLayout` tactic-aware recompaction for the X case (single-column removal) as a pre-step for the full toggle logic in the next commit.
5. Migrate `my-club.tsx` and `planner-tactic-editor.tsx` (and their tests) to the save-scoped keys; remove the zero-arg shim; verify `grep -R "plannerKeys.tactic()" --include="*.ts" --include="*.tsx"` returns zero after this commit.
6. Run focused tests: `src/features/search/utils/dynamic-columns.test.ts` (tactic sort accepted only when in layout, rejected when wrong-table), `src/app/routes/search.test.tsx` (URL with valid tactic sort hydrates, invalid falls back, save switch changes tactic query key, no-snapshot still renders filter strip), `src/features/search/components/search-results-panel.test.tsx` (header-X on one tactic while both active leaves straight survivor at far-right), `src/app/routes/my-club-squad.test.tsx` (save-scoped key migration).

**Tests and proof:**

- Unit for `isVisibleSortField` / companion: valid tactic ID in layout -> `true`; same ID not in layout -> `false`; tactic ID for wrong `tableId` -> `false`; non-tactic non-basic still requires `getFilterField` gating.
- Route tests: seed `localStorage` with `search: { columnIds: [..., "tactic_current.goalkeeper"] }`, navigate to `/search?sort=tactic_current.goalkeeper` -> stays `tactic_current.goalkeeper`; navigate with `?sort=tactic_current.not_a_lane` -> fallback; header X removing active tactic sort -> fallback to `ca`/`moneyball.average_rating`; mount with `snapshot: null` and assert `SearchFilterBar` still in DOM and the no-snapshot slot remains while panel shows empty state (toggle click proof moves to commit 7).
- Save-switch test: render route with `activeSaveId=1` tactic, switch mock `savesQuery` to `id=2` and assert `queryClient.getQueryData(plannerKeys.tactic(1))` not reused for the new view (or via spy on `enabled`).
- Expected RED before this commit: `isVisibleSortField("tactic_current.goalkeeper", [], "general")` returns `false` always or `true` unconditionally (either way wrong); route early-return hides toggles when `!snapshot`; header-X leaves interleaved hole.

**Patterns to verify:**

- Existing `validateSearch` pattern for `isVisibleSortField` + `defaultSearchSort` fallback — extend, not replace.
- Existing `useQuery(currentSnapshotQueryOptions)` + `savesQueryOptions` pattern for gating.
- Store atomic action `replaceLayout` from commit 3 is the only writer for tactic layout.

**Constraints and non-goals:**

- Do not make tactic columns filter-eligible.
- Do not persist tactic sort across tableIds (wrong-table fallback).
- Do not leak previous save's tactic after switch (enabled gate).

**Dependencies and sequencing:**

- Depends on commits 2-4 (neutral IDs, store atomic action, resolver). Must land before the toggle UI commit so that commit 7's exposed header sort never reaches an unvalidated SQL path.

**Validation:** `./scripts/dev check-app`, `./scripts/dev test src/features/search/utils/dynamic-columns.test.ts src/app/routes/search.test.tsx src/features/search/components/search-results-panel.test.tsx src/stores/use-player-table-store.test.ts`, `(cd src-tauri && cargo test resolver)` still green (no Rust change expected beyond read).

**Stop conditions:** If `usePlayerTableStore.getState().layouts[*]` is unavailable inside `validateSearch` (static function), create a module-level helper `isTacticSortVisibleFromPersistedState` that reads the persisted storage key directly or add the check inside the component's `useEffect` navigation fallback rather than inside `validateSearch`.

**Review mandate:**

1. `isVisibleSortField` + `validateSearch` accept canonical tactic sort only when present in the current view's persisted layout, and reject invalid/removed/wrong-table IDs (fallback verified).
2. Save-scoped `plannerTacticQueryOptions(activeSaveId)` has `activeSaveId` in the query key and `enabled` gate; switching saves does not reuse stale tactic.
3. No-snapshot composition keeps `SearchFilterBar` mounted (toggles rendered in commit 7); panel alone branches to empty state.
4. Header-X on one tactic while both active uses `replaceLayout` to re-compact survivor straight at far-right with no gaps.
5. Invalid tactic sorts never reach the Rust query (route fallback before IPC).
6. `biome.json` boundaries still hold (route owns Planner imports, shared/store remain clean).
7. Tests cover header sort, URL restoration, fallback after removal, and save-switching.
8. No new IPC or migration.

#### Commit 7 — Toggle buttons, interleaving, header wiring, widths and null-last polish

**Status:** Pending

**Provisional commit:** `feat(search): add tactic lane toggles and interleaving`

**Work:** Surface the two buttons next to `Edit Filters` on the three player tables, implement append/remove/interleaving/re-compaction via the atomic store action, complete header-X deactivation, and ensure column widths and null-last sorting polish with full test coverage.

**Size assessment:** ~170 non-test lines + ~80 tests. Within the soft target (interleaving + derived state + header wiring kept together atomically because they share the same atomic action and view model).

**Out of scope:**

- Additional backend logic beyond what commit 5 provides; Store atomic action definition beyond what commit 3 provides; route sort validation beyond what commit 6 provides; metric picker promotion.

**Implementation packet:**

**Files and responsibilities:**

- `src/features/search/components/tactic-column-toggles.tsx` (new) — presentational component `TacticColumnToggles { tableId, orderedLaneIds, laneLabels, currentActive, potentialActive, validationError, onToggleGroup, onError }` that:
  - Derives `currentActive`/`potentialActive` as `isFullTacticGroup(columnIds, group)` from `usePlayerTableStore(state => state.layouts[tableId].columnIds)` (or receives them as props from the route's normalized view model — prefer props to keep Planner labels out of the component).
  - Renders two `Button` (variant `secondary` inactive, `primary`/`primary-container` active) with accessible labels `"Add Tactic (Current)"` / `"Add Tactic (Potential)"`, `aria-pressed={active}`.
  - On click: if inactive -> delegate to `onToggleGroup(group, "add")` which the route implements as: validate (`validateTacticDraft(tactic, options)`), if error call `onError(error)` else atomically compute `nonTacticIds = columnIds.filter(id => !isTacticColumnId(id))` + `tacticSuffix = buildTacticColumnOrder(orderedLaneIds, pendingCurrentActive, pendingPotentialActive)` and call `replaceLayout(tableId, [...nonTacticIds, ...tacticSuffix])`. If active -> delegate to `onToggleGroup(group, "remove")` which computes `nextColumnIds = columnIds.filter(id => !id.startsWith(prefixForGroup))` plus recompaction when both were active (`nonTacticIds + remainingGroupStraight` via `buildTacticColumnOrder`). No store logic inside the component beyond the delegated callback.
- `src/app/routes/search.tsx` — render `TacticColumnToggles` in the filter strip line next to `Edit Filters` button (same flex row), for each view `general | moneyball | shortlist` share the same toggles instance per `tableId`. Pass the normalized view model (`orderedLaneIds` derived via `orderedTacticLanes` + `compareTacticPositions`, `laneLabels`, save-scoped `tactic`/`options` from gated queries). Wire `onError` to local `useState<string | null>` that renders a `Panel` inline error matching existing `SearchResultsPanel` error style; clear on next successful toggle or save/snapshot change. Keep Shortlist and Moneyball views reusing the same toggles + panel paths.
- `src/features/search/components/search-results-panel.tsx` — extend `tableColumnForMetric` fallback to produce synthetic columns: when `isTacticColumnId(metricId)` and `isValidTacticColumnId(metricId)` found, return `PlayerTableColumn { id: metricId, label: laneLabels.get(laneId) ?? laneId, align: "right", width: widths[metricId] ?? TACTIC_COLUMN_DEFAULT_WIDTH }`; otherwise filter out unknown tactic id. Extend `renderCells` branch for `isTacticColumnId(column.id)` to render right-aligned `ScoreBadge` when `typeof dynamicValue === "number"` else `"—"` span (`text-on-surface-variant`). Ensure `onRemoveColumn` delegates to the tactic-aware `replaceLayout` path when the removed column is a tactic ID (reuses commit 6's wiring). Ensure `setColumnWidth(tableId, columnId, width)` is wired for `isTacticColumnId(columnId)` exactly as other role columns; default width fallback is `TACTIC_COLUMN_DEFAULT_WIDTH` (not `96` or `176`).
- `src/components/player-table/player-table-header.tsx` — read-only inspection: confirm resize and sort caret work for synthetic columns (no column-type-specific logic required).
- `src/features/search/utils/dynamic-columns.ts` / `src-tauri/src/features/search/query.rs` — no further change in this commit beyond width/sort polish verification (Moneyball mapping doc comment lives in commit 5 per NITPICK; do not duplicate in commit 7).

**Behavior and data flow:**

- Entry: save-scoped `plannerKeys.tactic(activeSaveId)` query (+ options) cached; `usePlayerTableStore.layouts[tableId]` observed; `orderedLaneIds`/`laneLabels` passed as normalized view model from route.
- Toggle click -> validation in route -> `replaceLayout` atomic write -> `SearchResultsPanel` derives new `columns` -> new `requestedFields` includes synthetic IDs -> `searchPlayersQueryOptions` refetches with tactic sorts preserved.
- Header X click -> `replaceLayout` with tactic-aware recompaction -> `columnIds` loses one lane id -> `isFullTacticGroup` flips to false -> button `aria-pressed=false`.
- Resize lifecycle: drag -> `setColumnWidth(tableId, tactic_current.goalkeeper, newWidth)` -> `layouts[tableId].widths[...] = clampWidth(newWidth)` -> panel applies inline width and persists.

**Ordered implementation steps:**

1. Create `tactic-column-toggles.tsx` as presentational component with derived state and delegated callbacks; unit-test its rendering and callback delegation without mounting the store (prop-driven).
2. Add `SearchResultsPanel` synthetic column dispatch and cell rendering for `tactic_*` with `ScoreBadge`; verify story still renders `…` placeholder when `player === undefined`.
3. Compose toggles into `src/app/routes/search.tsx` in the filter bar line (reused for `shortlist`/`moneyball` views' identical bar). Pass correct `tableId` per view and the normalized view model. Wire validation via `validateTacticDraft` in the route (Biome boundary respected).
4. Verify initial render plus interactions with mocked `plannerKeys.tactic(activeSaveId)` via `src/testing/planner-ipc-mock.ts` + `mockIPC`.
5. Run `./scripts/dev test` focused then full `check-app`.

**Tests and proof:**

- Unit for `tactic-column-toggles` logic (prop-driven): append current (clears partial then 11 appended via `replaceLayout` spy), second toggle interleaving emit order assertion (`GK Cur, GK Pot, …`), toggle-off removes all 11 and re-compacts, header X deactivates that group's button, and header X while both active leaves surviving group straight at far-right via atomic action. Non-skippable: Moneyball mapped numeric vs uncovered `NULL` is already proven in commit 5 integration tests — this commit adds frontend proof that `"—"` renders for `NULL` and `ScoreBadge` renders for numeric.
- Integration `src/app/routes/search.test.tsx` addition: render Search page with mocked save-scoped tactic, interact with toggles, assert `columnIds` in `localStorage` contain `tactic_current.goalkeeper` etc., assert active style and `aria-pressed`, assert synthetic header label text contains `" ("` pattern, assert remove via header/context menu deactivates button, assert error toast when tactic mocked invalid. Include Moneyball view tactic toggle path and save-switching: switching `activeSaveId` does not leak previous tactic's labels/widths; no-snapshot mount still renders filter strip + toggles operable (gated error) and shows empty state.
- Width/sort polish tests: `widths["tactic_current.goalkeeper"]=200` survives re-hydrate and travels with id; clicking tactic header toggles `asc`/`desc` with `NULLS LAST` (fixtures: one player 70, one `—`), asserted via sorted `page.players` order; removed tactic sort falls back to `defaultSearchSort`.

**Patterns to verify:**

- Existing `SearchFilterBar` compact strip pattern — buttons sit alongside `Edit Filters`, not inside the modal.
- `SearchResultsPanel` `removeColumn` + sort fallback pattern — now tactic-aware via `replaceLayout`.
- App existing error/feedback: inline `Panel` with `error` tone matching Shortlist load errors.

**Constraints and non-goals:**

- Do not make `isTacticColumnId` columns available in `PLAYER_METRICS` picker; do not change `FilterAst` or `search-url-search.ts`.
- Do not style tactic columns differently than other role `ScoreBadge` columns (same ramp).
- Do not persist active booleans; derive strictly from `columnIds`.
- Do not modify Squad's layout.
- Do not create a standalone docs-only commit — mapping docs live inline in commit 5 and durable ledger/architecture notes.

**Dependencies and sequencing:**

- Depends on commits 2-6 (helpers + store atomic action + resolver + query scorer + route validation/save-scope/no-snapshot).

**Validation:** `./scripts/dev check-app`, `./scripts/dev test src/utils/tactic-ids.test.ts src/features/search/utils/tactic-columns.test.ts src/stores/use-player-table-store.test.ts src/app/routes/search.test.tsx src/features/search/components/search-results-panel.test.tsx`, and `./scripts/dev check-rust` (or separate `(cd src-tauri && cargo test --lib -- resolver)` + `(cd src-tauri && cargo test --lib -- query)` + `(cd src-tauri && cargo test --lib -- fit)`) still green. Do not use a single `cargo test query fit resolver` invocation (invalid — one `TESTNAME`).

**Stop conditions:** `plannerKeys.tactic(activeSaveId)` unavailable because `fetchPlannerTactic` still expects implicit active save -> gate in route and add param threading before wiring toggles; route must not assume planner loader exists without `activeSaveId`.

**Review mandate:**

1. Button active state derived solely from `columnIds` (no stray Zustand bool).
2. Append is always far-right (after non-tactic) and cleared partial first via atomic `replaceLayout`.
3. Interleaving order is `GK Cur, GK Pot, DR Cur, DR Pot,...` (not alphabetical) when both active, straight otherwise.
4. Removing one X immediately deactivates that group's button and re-compacts survivor straight at far-right without gaps.
5. Removing a group when both active re-compacts via same atomic action (no intermediate interleaved partial observable).
6. Invalid tactic shows error and mutates no layout (verified by localStorage snapshot).
7. Shortlist's identical persistence bucket `shortlist` owns its own toggles (cross-table independence); save switching does not leak tactic between saves.
8. Width default `TACTIC_COLUMN_DEFAULT_WIDTH = 112` and right-align verified; `NULL` `"—"` rows are last in both ASC and DESC.
9. No `biome.json` boundary violation: Planner imports only in `app/routes`, shared/store remain clean, search helpers import only neutral shared modules.
10. No new behavior introduced without the required Moneyball mapped vs unmatched, save-switching, no-snapshot, and post-X recompaction proofs.

## Active work

**PR:** PR 1 — Tactic columns for player tables (Search, Moneyball, Shortlist)

**Commit:** Implement tactic lane scoring and query sort (Rust, Moneyball mapping)

### RED or removal proof

Add focused fit, resolver/query, and Moneyball mapping tests against the current `NULL` placeholder. Confirm canonical tactic fields return no score or fail the required sort and mapping assertions.

### Expected outcome

Rust computes each requested tactic lane from the active save's validated Planner tactic. General and Shortlist use compact current or potential role scores with the age-29 fallback; Moneyball uses the deterministic `(attribute_role_id, base_position)` mapping and cohort scoring. Familiarity and foot eligibility match Planner fit rules. Requested values and sorts share one score truth, with unavailable values last in both directions and UID ties. Catalog tests prove unique 103/111 coverage and the exact eight unavailable combinations.

### Explicit exclusions

Frontend route/panel/toggle work, filter eligibility, Squad, migrations, new tables or IPC commands, client-side scoring, catalog changes, and unrelated refactors.

## Discoveries and replanning

- Scope expansion vs original TODO Next: the provisional TODO `Next` entry was a gender data investigation; this feature is tactic columns, a dispatch-driven override. The ledger deliberately does not reference gender scope; TODO placement is under `Active` as the dispatched feature.
- Backend scorer choice required reuse of `base_position` from `tactic.rs` which is currently `pub(super)`. The plan promotes it to `pub(crate)` in commit 5 to share normalization exactly — recorded as the only architectural visibility change.
- Moneyball deterministic mapping discovery: `src-tauri/src/features/moneyball/builtin_role_definitions_v1.json` has 88 defs, 77 map via `attribute_role_id` to the 68 General roles, 11 are Moneyball-only (enumerated in tests), all 68 General role IDs represented; matching by `(attribute_role_id, base_position(placement))` covers 103/111 General `(role, position_tag)` combos, 8 unavailable by design (`holding_wing_back_oop`+DL/DR, `pressing_wing_back_oop`+DL/DR, `box_to_box_midfielder_ip`+MC, `box_to_box_playmaker_ip`+MC, `deep_lying_playmaker_ip`+MC, `second_striker_ip`+ST). Prior plan treated mapping as an unknown gap; correction folds the gap doc into implementation commits 5/7 and durable invariants with non-skippable mapped vs unmatched tests.
- `biome.json` boundary violation discovered: shared/stores may not import features and features may not import across features. Prior plan had Search helpers importing Planner internals and `use-player-table-store` importing a Search helper. Correction introduces neutral `src/utils/tactic-ids.ts` (canonical lane IDs, prefix constants, predicates, default width) and keeps Planner query/validation in `src/app/routes/search.tsx` which passes a normalized view model into Search components; stores import only the neutral module.
- Route/sort boundary discovered: `isVisibleSortField` gated non-basic sorts on `getFilterField`, so synthetic tactic IDs cannot pass. Correction assigns the layout-aware sort acceptance change to commit 6 landing before the toggle UI (commit 7), ensuring every exposed sort is already trunk-safe and tested for header, URL restoration, and fallback after removal.
- Store atomicity discovered: `removeColumn`/`addColumns` only drop one ID/width each, leaving interleaved partials after header-X. Correction defines one atomic `replaceLayout` action in commit 3 validated and used by both toggle and tactic-aware header-X paths (commit 6/7) with deterministic post-X straight recompaction.
- Save-scoping and no-snapshot discovered: `plannerTacticQueryOptions` is save-independent while `get_planner_tactic` resolves `active_save_id` implicitly, and `search.tsx` early-returns before the filter strip when `!snapshot`, contradicting the toggle-remaining promise. Correction makes the query save-scoped (`plannerKeys.tactic(activeSaveId)`, enabled gate, reset on save/snapshot change) in commit 6 and refactors `search.tsx` to always render the filter strip + toggles (only the results panel branches).
- Tooling/branch factual corrections: classifiers are positional at `/home/jonas/projects/PI_SETUP/scripts/ledger_state.py` and `/home/jonas/projects/PI_SETUP/scripts/delivery_state.py` (with `publication_state.py`); Rust crate is `app`/`app_lib` with no repo-root `Cargo.toml` — run Rust tests via `(cd src-tauri && cargo test <filter>)` or `--manifest-path`; review branch is `release/v0.15.0` not `main`; `main` is behind `origin/main`; intended feature branch `feat/tactic-table-columns` is not activated during planning — activation from synchronized `main` is reserved for delivery workflow. Prior ledger used `cargo test -p fm-valuescout` / bare `cargo test` at root and `main` as the active branch; corrected throughout validation fields and PR base notes.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Tactic columns for player tables (Search, Moneyball, Shortlist) | Commit 1 — Record the approved feature plan | 944551c774cb30c0d7f2bcb42f232572680e870d | Recorded the reviewed schema 2 ledger and activated the feature in TODO without changing executable behavior. | `ledger_state.py`; `delivery_state.py`; `git diff --check 406dca7..944551c` — passed. | Not applicable | Clear | 0 | None |
| PR 1 — Tactic columns for player tables (Search, Moneyball, Shortlist) | Commit 2 — Add neutral tactic-identity helpers and ordering | d8be592b873076aa8611d569ecd8badb38a4e021 | Added the closed 11-lane synthetic ID contract, Planner re-export, and deterministic straight/interleaved tactic ordering. | Focused Vitest: 11 passed; `./scripts/dev check-app`; boundary import search; `git diff --cached --check` — passed. | Pass | Clear | 1 | Corrected constructors and order builders to reject unknown runtime lane IDs. |
| PR 1 — Tactic columns for player tables (Search, Moneyball, Shortlist) | Commit 3 — Persist synthetic tactic layouts with an atomic replace action | 83bdccd0db2b4707a416a1e78fc36b0088cb7493 | Retained canonical tactic IDs on the three Search layouts and added one atomic, validated layout replacement with width pruning and default fallback. | Store Vitest: 27 passed; tactic helpers: 11 passed; `./scripts/dev check-app`; boundary import search; `git diff --cached --check` — passed. | Pass | Clear | 1 | Unified all store mutators on one table-specific allowlist and added single-notification proof. |
| PR 1 — Tactic columns for player tables (Search, Moneyball, Shortlist) | Commit 4 — Extend Rust resolver for synthetic tactic metric fields | d83a06250244829c08cb2c66c9b344084e2dd167 | Added closed Search-only tactic metric parsing with integer/NULL placeholder semantics while preserving generic Squad and filter rejection. | Resolver: 18 passed; Squad: 33 passed; filter: 59 passed; `./scripts/dev check-rust`: 737 passed, 2 ignored; `git diff --cached --check` — passed. | Pass | Clear | 1 | Split Search-specific parsing from generic parsing and consolidated exact boundary tests. |

## Final validation

- `./scripts/dev check-app` (Biome format/lint + `tsc -b` + full-tree secretlint) — frontend gate; must pass for every frontend commit and at close-out. Verifies `biome.json` `noRestrictedImports` boundaries.
- `./scripts/dev check-rust` (`cargo fmt --check` + `cargo clippy -- -D warnings` + `(cd src-tauri && cargo test)`) — Rust gate; required for commits 4–7 and at close-out. The `cargo test` invocation is `cd src-tauri && cargo test` or `cargo test --manifest-path src-tauri/Cargo.toml`, never `cargo test -p fm-valuescout` or bare `cargo test` at repo root.
- `./scripts/dev check` — full commit gate before each PR publication (runs `check-app` + `check-rust` + secretlint).
- `./scripts/dev test` — targeted `src/utils/tactic-ids.test.ts`, `src/features/search/utils/tactic-columns.test.ts`, `src/stores/use-player-table-store.test.ts`, `src/app/routes/search.test.tsx`, `src/features/search/components/search-results-panel.test.tsx`, plus `src/features/search/utils/dynamic-columns.test.ts` for sort validation; full suite at close-out.
- `(cd src-tauri && cargo test resolver)` + `(cd src-tauri && cargo test query)` + `(cd src-tauri && cargo test fit)` for commits 4–5; all `(cd src-tauri && cargo test)` at close-out.
- Classifiers: `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/tactic-table-columns.md` then `python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/active/tactic-table-columns.md /home/jonas/projects/fm-valuescout` — record the returned `delivery_fingerprint` under Delivery authorization and rerun both after any change.
- Moneyball mapping proof: at least one Moneyball-mapped tactic lane returns numeric in General vs Moneyball view and one of the 8 uncovered combos returns `NULL`/`"—"` (non-skippable, commit 5).
- Route/sort proof: tactic header sort toggles with `NULLS LAST` both directions, valid tactic sort restores from URL only when present in the persisted layout, invalid/removed/wrong-table tactic sort falls back to `defaultSearchSort(view)`, and header-X on one tactic while both active leaves the survivor straight at far-right without gaps.
- No-snapshot/save-switch proof: with `snapshot: null` the filter strip + toggles (commit 7) remain rendered operable and toggle click shows gated error without mutating layout; switching `activeSaveId` changes `plannerKeys.tactic(activeSaveId)` and does not leak prior save's tactic.
- Manual smoke (keyboard): with a fixture snapshot + a configured Planner tactic, toggle `Add Tactic (Current)` in Search General, Moneyball, and Shortlist — verify 11 columns appear at far-right in tactic order with header `Position (Role) / Position (Role)`, that `Potential` appends correctly, both interleaves as `GK Cur, GK Pot...`, that header X deactivates its group's button and re-compacts the survivor straight at far-right, that invalid tactic shows an inline error and mutates no layout, that widths persist after reload, and that sorting a tactic column keeps Nulls-last (`"—"` rows last) in both asc and desc.

## Documentation impact

Complete during reconciliation. Expected: no `ARCHITECTURE.md` or `DESIGN.md` structural change until implementation lands. At close-out, `ARCHITECTURE.md § Search` notes tactic synthetic columns, the neutral `src/utils/tactic-ids.ts` ownership, save-scoped tactic loading, deterministic `(attribute_role_id, base_position)` Moneyball mapping, and Rust scorer; `DESIGN.md` notes the two toggle buttons and header format if one is added. The Moneyball mapping contract's durable surface is `src-tauri/src/features/player_metrics/resolver.rs` / `query.rs` inline comment plus this ledger's Invariants; no standalone docs-only commit is created and no catalog counts are copied into `src/utils/tactic-ids.ts`. No ADR is required for this display plumbing; rationale stays in the ledger.
