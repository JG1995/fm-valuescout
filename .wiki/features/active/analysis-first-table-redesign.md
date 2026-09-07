# Analysis-First Table Redesign

## Status

Validation

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** 8d4e7a439e4c90155fdf007a4fe6cb7a4b5cc3f3e01897a896b865407487d6e8

## Intent

Redesign the existing shared configurable virtual table (Linear [JAY-61](https://linear.app/jaycount/issue/JAY-61/redesign-shared-table-component-for-analysis-first-data-views)) into analysis-first data views. One dense, grouped, sticky-identity table serves Search, Moneyball Search, Squad, Staff Search, My Staff, and Staff Shortlist, so comparing candidates across Profile, Ability, Market, and Tactic Fit reads in the first second of looking. This extends the established native `<table>` + `ConfigurableVirtualizedTable` / `ConfigurableTableHeader` system and the existing Zustand layout store; it adds no new table dependency and no parallel framework.

## User-visible behavior

- All six consumers (Search, Moneyball Search, Squad, Staff Search, My Staff, Staff Shortlist) render one fixed dense 40px/two-line row mode. No Compact/Comfortable switch and no persisted density preference exist; the mockup's density selector is out of scope.
- Identity is a required, visually strong, sticky first region on every table. Player identity shows the name plus available club/division context with stable unobtrusive reserved slots where a face and club logo would sit; no face/logo loading is implemented, text stays truthful, and no broken image ever renders. Staff supplies its own identity rendering through the same required-identity contract. Whole-row click/Enter activation keeps working; identity is the clearest affordance, not the exclusive activation path.
- Headers are grouped and reusable. Player Search defaults approximately to Profile, Ability, Market, and Tactic Fit groups. Other consumers define useful groups and defaults through the same view-owned column/group configuration contract.
- Tactical headers show the compact current tactic placement identifier as the primary label with smaller role context, and keep the current 11 tactic lanes with current/potential interleaving. The complete tactic definition is the accessible name of the focusable leaf header, and a visible disclosure appears on keyboard focus (and may also appear on hover). `title` may only supplement, never satisfy. No 12th lane is invented from the mockup.
- Dataset controls are consolidated into one reusable table-associated toolbar contract with caller-owned slots: count/sort summary, active removable filter chips plus an edit action where filters exist, grouped column configuration, and view-specific controls. Page actions such as Add Tactic and Upload Shortlist stay outside the generic controls.
- Existing behavior is preserved: URL-backed sorting/filtering, Query replacement behavior, ArrowUp/ArrowDown/Enter navigation across page boundaries, per-view saved column layouts, and backend/IPC contracts. Sorting/filtering drafts stay query-silent until applied; applying a filter still adds its metric column once when missing.
- Score semantics and missing-value honesty are unchanged: the current unfilled ScoreBadge/table ramp and neutral `—` are reused; no new scoring rules are added.
- Readable minimum widths and horizontal overflow are preserved. Sticky identity plus the complete grouped + leaf header context stay visible while scrolling. At 3440×1440 the table reveals more columns rather than stretching cells without bound.
- No Previous/Next pagination UI exists anywhere. The mockup's `Showing 1–12` bar and page buttons are explicitly out of scope; the bounded virtual pager remains the only paging mechanism.

## Invariants

- A bounded table container owns vertical scrolling; `@tanstack/react-virtual` keeps fixed-height (40px) virtual rows; TanStack Query keeps fetching bounded 50-row IPC pages; horizontal overflow stays in that same table container; no full dataset enters the WebView.
- Frontend-only unless source proves a necessary change. The WebView never selects raw SQL fields; Rust `player_metrics::resolver` and the Search/Staff/Squad resolvers stay authoritative for metric validation, dynamic values, and sort expressions.
- Existing URL-backed sorting/filtering, committed/requested Query replacement semantics, row activation, and ArrowUp/ArrowDown/Enter navigation keep working across page boundaries.
- Per-view saved analysis layouts persist independently (`search`, `moneyball-search`, `squad`, `staff-search`, `my-staff`, `staff-shortlist`) under store version 8. Analysis `columnIds`/`widths` never contain identity IDs; identity width persists separately as clamped `identityWidth`. Default-like layouts roll to the exact v8 analysis defaults in Decisions; customized layouts keep valid columns/order/widths; identity-only (zero-analysis) is valid and never rolls to defaults; malformed layouts fall back to v8 defaults.
- Identity is required and non-removable, and is stored separately from configurable columns so identity fields are not duplicated.
- Score rendering reuses the current `ScoreBadge` tier ramp (`table` variant in cells) and the accessible `"Role: score, Tier"` naming; missing values render neutral `—`, never `null`, `N/A`, `0`, or an empty cell.
- Semantic `<table>` with `<caption>`, `<thead>`, `<th scope="col">`, per-leaf `aria-sort`, keyboard-operable column configuration and resize, visible focus, no hover-only information, and sticky content that never covers focused rows.
- No new table dependency: no `@tanstack/react-table`, no parallel table framework. The existing Zustand store is extended, not replaced.
- Static feature-specific tables that do not use the shared component (for example Academy roster and profile role tables) are untouched.

## Non-goals

- Compact/Comfortable density modes, a density selector, and any persisted density preference (explicitly rejected by the developer).
- Face/club-logo loading or remote imagery (offline-by-construction; reserved slots only).
- A 12th tactic lane from the mockup; the current 11 lanes are kept.
- Previous/Next pagination UI or unbounded client-side collections.
- New scoring rules, rescaled scores, or new Club DNA / role-score computation.
- Backend, IPC, migration (other than the table-layout store), bridge, CSV, Planner, or optimizer changes unless implementation evidence proves one necessary (which triggers replanning).
- Static feature-specific tables (Academy roster, profile role tables) and any consumer outside the six listed views.
- Copying the mockup image into the repository. The mockup is design reference only; the hierarchy below is the durable record.

## Current-state map

- Relevant components:
  - `src/components/player-table/virtualized-player-table.tsx` — `ConfigurableVirtualizedTable` shell: one `overflow-auto` parent, `ROW_HEIGHT = 40`, `HEADER_HEIGHT = 32`, fixed semantic rows, bounded page-window queries (`pageSize` 50), roving row focus with ArrowUp/ArrowDown/Enter, whole-row activation, proportional `<col>` widths under `width: 100%`, `minimumTableWidth` from column widths. Compatibility wrapper `VirtualizedPlayerTable` for player Search/Squad callers.
  - `src/components/player-table/player-table-header.tsx` — `ConfigurableTableHeader`: one `<thead>` row only, no sticky identity, no groups; header click sorts with `aria-sort`; context-menu/Shift+F10 menu (Move left/right, Add column, Remove column); searchable `MetricPicker`; pointer + keyboard resize handles clamped 72–360.
  - `src/stores/use-player-table-store.ts` — versioned (`PLAYER_TABLE_LAYOUT_VERSION = 7`) persisted layouts for six table IDs; `sanitizeLayout`/`migratePersistedState`; atomic `replaceLayout`; per-table allowlists (`isSuggestedTrainingColumnId`, tactic IDs only for `search`/`moneyball-search`, moneyball/player metric gates). Existing persisted layouts can include/remove/move identity fields (`name`, `club`, `division`); v4–v5 migrations removed duplicate club/division from defaults while keeping explicit re-adds.
  - Consumers: `src/features/search/components/search-results-panel.tsx` (player identity = Name + club/division secondary text in configurable `name` column; tactic block handling; sort-replacement cache), `src/features/squad/components/squad-overview-panel.tsx` (same player identity pattern + Squad-only Suggested Training, non-sortable), `src/features/staff/components/staff-search-results-panel.tsx` (`ConfigurableVirtualizedTable` with generic row shapes; shortlist paths split on `staffShortlistPresentation(preferredJob)` — `undefined` (All jobs) keeps the persisted configurable `staff-shortlist` layout, while a defined presentation uses fixed `fixedColumnIds` with `configurable={false}`), routes `src/app/routes/search.tsx` (tactic toggles, lane labels, filter bar composition), `src/app/routes/staff.tsx` (staff filter bar + assignment/upload header actions).
  - Filter strips: `src/features/search/components/search-filter-bar.tsx` + `search-filter-strip.tsx` (tags, Clear all, Edit filters, `actions`/`afterEditActions` slots) and `src/features/staff/components/staff-filter-bar.tsx` (same pattern with Preferred Job / Only unemployed while shortlisting). Both are separate Panels above the result Panels; result count/sort summaries live inside the Results panels.
  - Catalogs with category metadata: `src/utils/player-metrics.ts` (`PlayerMetricCategory`: identity, club-contract, ability-reputation, visible/hidden attributes, personality, position-suitability, current/potential-role-scores; `roleFamily` per role), `src/utils/moneyball-search-metrics.ts` (Identity, Club and value, Context, Moneyball metric categories, Moneyball roles), `src/features/staff/utils/staff-metrics.ts` (`identity`, `club-contract`, `ability-reputation`, `staff-attributes`, `current-role-scores`, plus `shortlist`), `src/features/squad/utils/squad-columns.ts` (Squad-only Suggested Training kept out of the shared catalog). `src/components/ui/player-metric-picker.tsx` already groups picker items by category and role family.
  - Tactic columns: `src/utils/tactic-ids.ts` (11 canonical lanes, `tactic_current.*`/`tactic_potential.*`, default width 112, group predicates), `src/features/search/utils/tactic-columns.ts` (straight vs interleaved order, far-right contiguous block), route-produced lane labels (`IP Position (Role) / OOP Position (Role)`).
  - Cells: `src/components/ui/score-badge/score-badge.tsx` (`table` unfilled variant, tier ramp, accessible name + `title`), `src/utils/format.ts` (`formatMoney`, `formatCount`, `formatMissable` → `—`, `formatPlayerDob`), `src/components/player-table/nationality-cell.tsx`.
- Data model: no new tables. Table layout state stays in `localStorage` key `fm-valuescout-player-table-layouts` via Zustand persist. New group/identity metadata lives in frontend contracts only.
- Persistence and migrations: store version 8 (from 7). The exact v8 shape, defaults, default-like/custom/malformed/identity-only rules, and migration algorithm are defined in Decisions and the Commit 4 packet — the single source of truth the worker implements without deviation.
- Existing behavioral assumptions: committed/requested sort-replacement with `isReplacementActive`; applying a filter adds its metric column once; filters/columns independent afterwards; tactic group active iff all 11 lane IDs present; tactic block contiguous far-right; null-last tactic/role/Club DNA sorts with UID tie-break; Staff shortlist filtering uses the `staff-shortlist` layout per path: All jobs keeps the persisted configurable layout, while a defined `staffShortlistPresentation(preferredJob)` swaps to fixed `fixedColumnIds` with `configurable={false}`.
- Architectural seams: store layouts ↔ panels via `columnIds`/`widths`; route-owned composition (lane labels, filter state, page context) passed into panels; shared shell/header never imports features (Biome `noRestrictedImports`); panels own cells and activation.
- Project validation commands: `./scripts/dev test [target...]` (Vitest), `./scripts/dev check` (full gate: Biome, `tsc -b`, secretlint, Rust fmt/clippy/test), `./scripts/dev smoke` (Playwright stub suite, `e2e/smoke.spec.ts`); required GitHub status `check`; squash merge per `.github/pull_request_template.md`.
- Primary risks: stretching width model (`width: 100%` + proportional `<col>`) fights the "reveal, don't stretch" ultrawide goal — the worker must change the width model without breaking containment; sticky identity + grouped header must not cover focused rows or break the virtual spacer; store v8 migration must not reset customized layouts; toolbar consolidation must not change filter URL semantics or page-action placement; tactic header shortening must keep the full definition keyboard-reachable.

## Feature architecture

- `src/components/player-table/` owns the shared system: the virtual shell (scroll owner, virtualizer, paging, focus, activation, sticky regions, width model), the grouped header (two-row group + leaf rendering, sorting, menus, resize), the reusable toolbar contract (summary, chips, edit, column configuration, caller slots), and shared cell presentation (text/numeric/currency/score/comparison/unavailable). It exposes caller-owned seams and never imports features.
- Group metadata is the single organizer: the same group definitions drive grouped headers, grouped column configuration, and view-owned defaults. Player Search defaults approximately to Profile, Ability, Market, Tactic Fit; each other consumer defines its groups/defaults through the same contract.
- Identity is a required sticky first region owned by the shell's layout contract but rendered by caller-owned identity cells: player callers render name + club/division context with reserved face/club-logo slots (no loading); staff callers render staff identity; the store keeps identity separate and non-removable.
- Routes and feature panels own composition: filter state and URL behavior, lane labels and tactic data, page actions (Add Tactic, Upload Shortlist, Configure/Optimize), and which toolbar slots they fill. The toolbar contract never owns page actions and never changes filter URL semantics.
- `src/stores/use-player-table-store.ts` owns persisted analysis column/order/width state, `identityWidth`, plus the v8 migration. Defaults change to the exact v8 analysis defaults in Decisions; customized layouts are retained.
- Tests extend existing seams (`configurable-table-contract.test.tsx`, `player-table-header.test.tsx`, `use-player-table-store.test.ts`, route tests under `src/app/routes/`, `e2e/smoke.spec.ts`); no duplicate coverage of already-proven virtualization/paging/sort behavior.

## Uncertainty register

### Known

- Consumers in scope: Search, Moneyball Search, Squad, Staff Search, My Staff, Staff Shortlist. Static tables (Academy roster, profile role tables) are out.
- Fixed architecture: bounded container scroll, `ROW_HEIGHT = 40`, header 32, 50-row IPC pages, horizontal overflow in the same container, no full dataset in the WebView, no Previous/Next UI.
- One fixed dense 40px/two-line mode; no density selector or preference.
- Layout IDs (`search`, `moneyball-search`, `squad`, `staff-search`, `my-staff`, `staff-shortlist`) and store version 7; tactic IDs valid only for `search`/`moneyball-search`; Suggested Training only for `squad`; the shortlist staff layout is configurable for All jobs and fixed for defined presentations while filtering.
- Category metadata already exists in all three catalogs and the picker already groups by it — the display-group contract builds on this, it does not invent a parallel taxonomy.
- Current Search/Squad player identity pattern (Name + `club · division` secondary line; secondary line omitted when both absent) is the baseline the required-identity rendering extends.
- Mockup hierarchy (preserved here so implementation never depends on the untracked image): utility bar + grouped top nav; `Player Search` title with subtitle; right-aligned page actions Add Tactic (Current), Add Tactic (Potential, highlighted), Upload Shortlist; one dataset strip with `Results 194,417 players`, removable chips (`Age 16–30`, `Value > €1M`, `Top 5 leagues`, `Positions: M, AM, ST`), `Clear all`, and right-side Edit filters / Columns / Density (density excluded per decision); grouped columns Profile (Player, Age, Nat), Ability (CA, PA), Market (Value), Tactic Fit/Suitability (12 compact placement lanes with small role context, e.g. GK/Sweeper, DR/FB, MCR/CM); sticky player column with face slot, name, club + division; color-coded scores with `—` for missing. Excluded from scope: density selector, pagination bar, 12th lane, face/logo imagery.

### Assumptions

- Current evidence (single scroll container, fixed rows, bounded pages, validated metric allowlists, URL-backed filter/sort) supports a frontend-only change; any backend/IPC necessity found during implementation triggers replanning rather than silent scope growth.
- The existing `MetricPicker` category/role-family grouping is the base for the grouped column-configuration UI; Commit 5 extends it with the shared group input rather than replacing it.

### Decisions

- One fixed dense 40px/two-line row mode; no Compact/Comfortable modes or persisted density preference. Consequence: the width/containment work targets exactly one row geometry; the mockup's density selector is not implemented.
- No new table dependency; extend the native `<table>` + `ConfigurableVirtualizedTable` / `ConfigurableTableHeader` system and Zustand store. Consequence: no `@tanstack/react-table`, no parallel framework, no store replacement.
- Required sticky identity with caller-owned rendering; whole-row navigation stays. Consequence: player face/club-logo slots are reserved layout only — no loading, truthful text, no broken images.
- Reusable grouped headers with view-owned configuration; Player Search defaults approximately to Profile, Ability, Market, Tactic Fit. Consequence: other consumers define groups/defaults through the same contract; no consumer-specific header forks.
- Compact tactic placement identifiers as primary labels with smaller role context; the complete tactic definition is the accessible name of the focusable leaf header and a visible disclosure appears on keyboard focus (and may also appear on hover) — `title` may only supplement, never satisfy; 11 lanes and interleaving kept. Consequence: header text shortens but no tactic information becomes hover-only; no 12th lane.
- Reveal-more-columns at 3440×1440 instead of unbounded stretching, with explicit 1280×800 and 3440×1440 browser proof. Consequence: the proportional stretch width model must change.
- Reuse current ScoreBadge ramp and `—`; no new scoring rules. Consequence: cell work is presentation consolidation only.
- Reusable table-associated toolbar with caller-owned slots; page actions stay outside; filter URL behavior and query-silent drafts preserved. Consequence: filter strips move into table association without semantic changes.
- Grouped column configuration with required identity separate/non-removable; default-like layouts roll to redesigned defaults; customized layouts retained. Consequence: a store v8 migration with mechanics resolved from store evidence.
- One PR unless repository evidence proves a required independent merge boundary. Consequence: maximal atomic commits inside a single review boundary; companion tests travel with each behavior.
- Store v8 shape: `{ columnIds: string[]; widths: Record<string, number>; identityWidth: number }` per table. `columnIds`/`widths` hold analysis columns only — `name`, `club`, `division` are rejected by `addColumns`/`replaceLayout`/`moveColumn`/`setColumnWidth`, dropped by sanitize, and never offered in column configuration. Identity consumes name/club/division in all six modes. `identityWidth` defaults to 280, clamps to 240–360, and is resizable by pointer and keyboard from the identity header, which exposes these bounds. Identity geometry contract: 40px row; 28px neutral face slot; 8px gap; text budget at least 188px including the secondary 12px club-logo slot + 4px gap; 16px total horizontal cell padding. Therefore `IDENTITY_COLUMN_MIN_WIDTH = 240`, `IDENTITY_COLUMN_DEFAULT_WIDTH = 280`, `IDENTITY_COLUMN_MAX_WIDTH = 360`. Player identity slot order: 28px face slot, 8px gap, flexible text stack; the secondary line starts with the 12px club-logo slot + 4px gap then club/division text. Staff uses the same width/slot geometry but caller-owned content; where no club logo is applicable the 12px sub-slot remains unobtrusive and alignment-stable. Current Name default is 224, so the new 280 default intentionally adds the space required by the reserved slots. Analysis may be empty (identity-only is valid); `removeColumn` no longer floors at one analysis column. Consequence: identity can never duplicate, and clearing all analysis columns is a supported end state rather than a reset trigger.
- Exact v8 analysis defaults (tactic stays opt-in everywhere): Search `age,nationality,ca,pa,value`; Moneyball `age,nationality,moneyball.minutes,moneyball.average_rating,moneyball.goals_per_90,moneyball.assists_per_90,moneyball.xg_per_90,moneyball.xa_per_90`; Squad `age,nationality,ca,pa,value,suggested_training`; Staff Search and My Staff to the current `DEFAULT_STAFF_TABLE_COLUMN_IDS` minus name/club/division; Staff Shortlist (All-jobs configurable path) to the current shortlist default minus name/club/division — fixed presentations use their `fixedColumnIds` minus identity IDs and never touch the persisted layout. Consequence: every default is reviewable in this plan; the worker implements these lists verbatim.
- v7→v8 migration order per table (implemented verbatim): (1) capture the original finite `widths.name` first as the `identityWidth` candidate (clamped 240–360; absent/non-finite uses 280) — the captured value survives even if later normalization strips the name width; (2) preserve the existing pre-v8 upgrades to produce a v7-equivalent layout (the <5 identity normalization and the <7 Suggested Training rollout run unchanged); (3) apply the v8 default-like/custom/malformed/identity-only classification to that v7-equivalent result, comparing default-like against the correct resulting v7 defaults per table (including the post-rollout Squad default with Suggested Training). Default-like means the v7-equivalent `columnIds` exactly equal that table's resulting v7 default IDs in order with exactly empty (`{}`) widths — any reorder, resize, add, or remove is custom. Default-like rolls analysis to the v8 defaults above with empty widths. Custom keeps valid analysis order/widths (clamped) with the captured `identityWidth`, and drops identity IDs/widths. A custom layout whose valid analysis is empty stays identity-only and is never rolled to defaults. Missing/non-array/empty raw `columnIds` is malformed and falls back to v8 defaults with `identityWidth` 280. v8 sanitize additionally rejects tactic IDs outside Search/Moneyball Search and Suggested Training outside Squad for all six tables. Consequence: old customized layouts (v4–v7) are never reset, and no invalid ID survives in any mode.
- Group maps and fallback are defined once in the Commit 2 packet (view-owned input, contiguous runs, hidden-group and fallback rules). Consequence: header and Columns control cannot diverge; the worker implements the recorded maps verbatim.
- Toolbar ownership: the generic toolbar owns only summary, filter chips/Clear/Edit when supplied, grouped Columns, and dataset toggles. Search page header owns Add Tactic (current/potential) and General Upload Shortlist; Staff page header owns Upload Shortlist, Configure, and Optimize; My Staff boost and Squad actions stay outside the toolbar. Consequence: page actions are never generic-toolbar descendants, asserted by test.
- Tactic full definitions are revealed through a visible tooltip/disclosure on keyboard focus (and may also appear on hover) — the complete definition is the accessible name of the focusable leaf header; never hover-only and never `title`-only (`title` may only supplement, never satisfy). Consequence: keyboard-only users reach every lane definition, proven by a visibility assertion.
- DESIGN owns the visual close-out for this feature (two-row grouped headers, 64px sticky geometry, sticky identity region, table-associated toolbar); no ARCHITECTURE change and no ADR are expected.

### Risks

- Width-model change (away from `width: 100%` proportional `<col>`) breaks containment or virtual spacer math. Mitigation: keep `minimumTableWidth` semantics, prove 1280/1600 containment tests still pass, add 3440 proof.
- Sticky identity/grouped header covers focused rows or breaks focus-into-view. Mitigation: `scrollPaddingStart` moves 32→64 with the two-row header; focused tests plus keyboard proof that 64px headers never cover focused rows.
- v8 migration resets customized analyst layouts. Mitigation: migration tests proving customized columns/order/widths survive; default-like detection is conservative.
- Toolbar consolidation accidentally changes filter URL semantics or swallows page actions. Mitigation: URL-behavior tests per consumer; Add Tactic / Upload Shortlist / Configure / Optimize stay outside the generic contract.
- Compact tactic labels lose role context for keyboard/screen-reader users. Mitigation: focusable header with complete accessible name plus a visible focus-revealed definition (never hover-only, never `title`-only — `title` may only supplement, never satisfy), proven by a keyboard visibility test.

## Walking skeleton

The thinnest path proving the approach: Commit 2 extends `ConfigurableTableHeader` to render a two-row grouped header from view-owned group metadata for the three consumer panels covering all six layouts (Search first as the thin slice), keeps every existing sort/menu/resize behavior with legacy `name`/`club`/`division` leaves mapped to Profile, moves header geometry 32→64 with `scrollPaddingStart` 64, and proves it with the header contract tests — before touching identity stickiness, migration, toolbar, or other surface changes. This grouped-header walking skeleton validates the shared `TableGroupInput` that Commits 4–5 consume.

## Delivery plan

### Atomicity principles

- "And" in a title is a warning, not an automatic split. Header geometry is inseparable from grouped headers; tactic accessibility from compact labels; tests from behavior; viewport proof from width behavior; global identity activation from its stripping migration. Each stays inside its owning packet and never appears as a separate outcome.
- Commit 3 is permitted as a foundation because it establishes a complete backward-compatible generic first-region seam with direct contract proof and no consumer behavior change. It adds no speculative extension points: only the optional identity region the very next commit activates.
- Commit 4 is the only intentionally oversized packet; no smaller trunk-safe split exists without disposable compatibility state (see its Size assessment for the exact >200-line reason).
- Commits 7–9 stay separate because each consumer has independent page-action/filter/sort composition and focused proof: Search proves the new toolbar contract once, Staff and Squad each adopt it under different ownership and degradation rules.

### PR 1 — Redesign shared table for analysis-first views

**Status:** Ready for publication

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** feature/analysis-first-tables

**Base branch:** main

**Publication provider:** GitHub

**PR template:** .github/pull_request_template.md

**Merge method:** squash

**Required checks:** check

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(tables): redesign shared table for analysis-first views`

**Purpose:** Single review boundary for the whole redesign: one planning-artifact commit plus exactly ten implementation commits. Every commit is trunk-safe on its own (existing behaviors preserved, no backend changes, store migration atomic with its activation), so no independent merge seam is warranted unless implementation evidence proves one.

**Depends on:** None.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(tables): record approved feature plan`

**Work:** Commit the independently reviewed planning artifacts on the feature branch before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, and unrelated documentation.

**Implementation packet:**

- Preserve the accepted plan-review outcome. Commit only the reviewed planning paths after branch verification.

**Files and responsibilities:**

- `.wiki/features/active/analysis-first-table-redesign.md` — approved feature intent, delivery plan, and packets.
- `.wiki/TODO.md` — active feature state with ledger link.

**Behavior and data flow:**

- Move planning truth into one reviewed active ledger and record the exact delivery sequence before implementation.

**Ordered implementation steps:**

1. Verify the active branch (`feature/analysis-first-tables`) and base (`main`) without changing Git state.
2. Confirm the worktree contains only the two reviewed planning paths.
3. Run the exact planning validators — `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/analysis-first-table-redesign.md` and `python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/active/analysis-first-table-redesign.md /home/jonas/projects/fm-valuescout` — and require the recorded exact fingerprint/classifier-valid state.
4. Stage exactly the ledger and TODO under fingerprint authority; inspect the staged diff.
5. Run `git diff --cached --check -- .wiki/features/active/analysis-first-table-redesign.md .wiki/TODO.md`.

**Tests and proof:**

- Not applicable — this commit changes planning documents only. The `ledger_state` + `delivery_state` classifier pair and the staged diff check are the structure proof.

**Patterns to verify:**

- The active-ledger template, current TODO/BACKLOG ownership rules.

**Constraints and non-goals:**

- Do not alter implementation, tests, executable configuration, plan scope, packet order, or reviewed decisions. Do not create or switch branches in this planning run; ledger state alone grants no branch authority. BACKLOG does not change.

**Dependencies and sequencing:**

- Requires an accepted plan-review verdict, developer acceptance, a valid Delivery fingerprint, and exact branch activation.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/analysis-first-table-redesign.md` and `python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/active/analysis-first-table-redesign.md /home/jonas/projects/fm-valuescout` (schema 2; require the recorded exact fingerprint/classifier-valid state), then `git diff --cached --check -- .wiki/features/active/analysis-first-table-redesign.md .wiki/TODO.md` (no whitespace errors). Record the post-clear `delivery_state` summary and both classifier verdicts with the commit.

**Stop conditions:** Stop on an uncleared review, a classifier error, an unreviewed path, a substantive post-review plan change, or a branch mismatch.

**Review mandate:** Verify that the staged diff contains the complete reviewed planning outcome and no implementation or unrelated files.

#### Commit 2 — Support grouped table headers

**Status:** Completed

**Provisional commit:** `feat(tables): support grouped table headers`

**Work:** Render reusable two-row grouped headers (group row + leaf row) from one view-owned group input and wire them to all six layouts through the three current consumer panels (`search-results-panel`, `squad-overview-panel`, `staff-search-results-panel`); move header geometry 32→64 (two 32px rows) with `scrollPaddingStart` 64; contiguous visible group runs with repeated group cells after moves; fallback group; `rowSpan=2` for fixed-action leaves (where present); keyboard proof that 64px headers never cover focused rows. During this pre-v8 commit, the legacy configurable identity leaves `name`, `club`, and `division` map explicitly to Profile in every mode and retain current sort/menu/resize behavior; Commit 4 then removes these IDs and supplies row-spanning sticky identity. This commit also defines the shared `TableGroupInput` that Commits 4–5 reuse for identity-adjacent grouping and the grouped Columns control — header consumes it here, Columns control consumes the same input in Commit 5. This is the grouped-header walking skeleton.

**Size assessment:** Estimated ~180 changed non-test implementation lines; within the soft target. Header, geometry, and the group contract are one observable outcome — a grouped header with wrong scroll padding or ragged runs is a regression, so they stay together. Header geometry is companion correctness inside this packet, not a separate outcome.

**Out of scope:**

- Identity extraction/rendering, store migration, toolbar, tactic label shortening, cell consolidation, consumer default changes, backend changes.

**Implementation packet:**

- Give the worker a complete handoff for the grouped-header vertical only.

**Files and responsibilities:**

- New neutral group-metadata helper (colocated with table code or `src/utils/`, no feature imports) — owns `TableGroupDef` (`{ id, label }`), the view-owned `TableGroupInput` (`groups` in display priority plus a total `groupForColumn(columnId)` mapper; prefix/category rules allowed), contiguous-run span computation over visible analysis leaves in order (new run whenever the group changes, so interrupted groups repeat; a group with no visible leaves renders nothing), and the automatic `Other` fallback (shared code appends `{ id: "other", label: "Other" }`; views never define it; every mapped-or-unmapped visible leaf lands in exactly one group).
- `src/components/player-table/player-table-header.tsx` — `ConfigurableTableHeader` accepts the group input and renders an optional group row above the leaf row; group cells use `<th scope="colgroup">` with `colSpan` equal to the run length; ungrouped-by-view leaves are impossible (fallback covers them); fixed-action leaves (where present) use `rowSpan=2`; transitional identity leaves (`name`, `club`, `division`) render as ordinary Profile leaves with sort/menu/resize unchanged — Commit 4 converts identity to the row-spanning sticky region. Sticky-corner layers: the `<thead>` stays `sticky top-0 z-10`; the sticky identity corner cell and body sticky-identity offsets land in Commits 3–4.
- `src/components/player-table/virtualized-player-table.tsx` — `HEADER_HEIGHT` 32→64 and virtualizer `scrollPaddingStart` 32→64; nothing else in the shell changes here.
- `src/components/player-table/configurable-table-contract.test.tsx`, `src/components/player-table/player-table-header.test.tsx` — extend: group row spans, `scope="colgroup"`, fallback assignment, repeated groups after moves, hidden groups rendering none, `rowSpan=2` fixed leaves, sort/menu/resize per leaf under groups, header-height/scroll-padding constants, transitional identity leaves in Profile (never in Other) with unchanged sort/menu/resize.
- `src/features/search/components/search-results-panel.tsx`, `src/features/squad/components/squad-overview-panel.tsx`, `src/features/staff/components/staff-search-results-panel.tsx` — supply the recorded view-owned group input for all six layouts (`search`, `moneyball-search`, `squad`, `staff-search`, `my-staff`, `staff-shortlist`) without changing defaults.

**Exact view-owned group maps (implemented verbatim; pre-v8 transitional — legacy configurable identity leaves `name`, `club`, `division` map explicitly to Profile in every mode with current sort/menu/resize retained; Commit 4 removes these IDs and supplies row-spanning sticky identity; `Other` is unmatched fallback only):**

- General Search — Profile: `name`, `club`, `division` (legacy transitional), plus `age`, `nationality`, `position`, `preferred_foot`, plus any other visible leaf whose player-catalog category is `identity`; Ability: category `ability-reputation` (`ca`, `pa`, `reputation`, `world_reputation`) plus `visible-attributes`, `hidden-attributes`, `personality`; Market: all other category `club-contract` leaves (`value`, `wage`, `contract_year`, `transfer_listed`, remainder); Tactic Fit: `tactic_current.*`, `tactic_potential.*`, `role.*`, `potential_role.*`, category `position-suitability`, `club_dna`; anything else → Other.
- Moneyball Search — Profile: `name`, `club`, `division` (legacy transitional), plus category `Identity` (`age`, `nationality`, `preferred_foot`) plus `position`; Market: all other category `Club and value` leaves (`value`, `parent_club`); Playing Time: category `Context` (`moneyball.starts`, `moneyball.substitute_appearances`, `moneyball.minutes`); Performance: all `moneyball.*` performance leaves (Shooting, Creation, Possession, Defending, Aerial, Goalkeeping, Discipline, Results categories); Role Fit: `moneyball_role.*`; anything else → Other.
- Squad — Profile: `name`, `club`, `division` (legacy transitional), plus same rule as Search Profile; Ability: same rule as Search Ability; Market: non-identity `club-contract` leaves (`value`, `wage`, contract/transfer facts); Development: `suggested_training` (future development leaves map here explicitly or fall to Other — never guessed); Role Fit: same rule as Search Tactic Fit minus tactic prefixes (tactic IDs are invalid for Squad); anything else → Other.
- Staff Search / My Staff — Profile: `name`, `club`, `division` (legacy transitional), plus category `identity` minus `name` (`age`, `nationality`, `birth_year`, `birth_day_of_year`, `nation_uid`, `gender`, `job_id`); Ability: category `ability-reputation` (`ca`, `pa`); Attributes: category `staff-attributes` (`attr.*`); Role Fit: category `current-role-scores` (`role.*`); Contract: non-identity `club-contract` leaves (`wage`, `contract_year`, `contract_day`, and future category peers map here explicitly); anything else → Other. Recruitment does not exist in these modes.
- Staff Shortlist — same five groups as Staff Search plus Recruitment: category `shortlist` (`preferred_job`, `club_job`, `coaching_qualifications`), shortlist-only; Recruitment renders no group cell and offers no columns in other modes. The Recruitment group serves the All-jobs configurable shortlist path; fixed presentations suppress the Columns control and offer no toggles.

**Behavior and data flow:**

- Each of the three consumer panels supplies its layouts' analysis `columns` plus the view-owned group input (transitional identity leaves included in Profile); shared code derives contiguous runs in visible-column order; sorting, moving, adding, removing, and resizing address leaf IDs exactly as today; both header rows stick and scroll horizontally with the table; `scrollPaddingStart` 64 keeps keyboard-focused rows below the two-row header.

**Ordered implementation steps:**

1. RED: failing header tests for a two-row grouped header (spans, `scope="colgroup"`, contiguous runs with an interrupted group repeating, hidden group rendering none, unmapped leaf landing in Other, transitional identity leaves in Profile and never in Other, leaf `aria-sort` intact, menu/resize per leaf unchanged for transitional identity leaves, `rowSpan=2` fixed leaves).
2. GREEN: implement the group input, run computation, fallback, and group-row rendering with minimal span logic.
3. Move `HEADER_HEIGHT`/`scrollPaddingStart` 32→64 in the shell; add geometry proof.
4. Wire all six layouts through the three consumer panels to supply their recorded maps without changing defaults.
5. Refactor only while the focused proof stays green; run targeted then affected validation.

**Tests and proof:**

- Extended `player-table-header.test.tsx`: grouped rendering, span math with hidden/moved columns, repeated group after a move, fallback assignment, non-sortable leaf under a group keeps no sort affordance.
- Extended `configurable-table-contract.test.tsx`: caller-owned catalog plus groups composes without changing fixed-column behavior.
- Geometry proof per shared consumer (all six layouts via the three panels): header measures two 32px rows (64 total); virtualizer receives `scrollPaddingStart` 64; keyboard ArrowDown across pages leaves the focused row fully below the header (existing focus-across-pages tests extended per consumer, plus smoke keyboard navigation at 1280).
- GREEN assertions: group headers visible with correct spans; clicking a leaf still sorts; menu and resize still target leaves.

**Patterns to verify:**

- Current `ConfigurableTableHeader` menu/resize/keyboard patterns; `MetricPicker` category/role-family grouping as the metadata analogue; tactic block far-right contiguity must not break.

**Constraints and non-goals:**

- Semantic table rules hold (no `div` table, no ragged row heights, fixed 40px rows). No width-model change here. No new dependency. Commit 2 does not extract identity (legacy identity leaves keep current sort/menu/resize behavior until Commit 4), does not move filter strips, and does not change defaults.

**Dependencies and sequencing:**

- Requires Commit 1 (planning-artifact commit). Commit 4 removes the transitional identity-leaf mappings recorded here and supplies row-spanning sticky identity on the Commit 3 seam; Commit 5 consumes the same `TableGroupInput` for the Columns control. No other prerequisites.

**Validation:** `./scripts/dev test src/components/player-table` then `./scripts/dev test src/app/routes/search.test.tsx src/app/routes/staff.test.tsx src/app/routes/my-club-squad.test.tsx`; commit gate `./scripts/dev check` before checkpoint review.

**Stop conditions:** Stop if grouped headers require changing sort/filter URL semantics, IPC contracts, or the virtual pager — replan instead. Stop if the 64px header breaks focus-across-pages and cannot be fixed with padding/offsets; fix or escalate.

**Review mandate:** Group span correctness with moved/hidden columns; repeated-group and fallback behavior; `scope`/`aria-sort` placement; `rowSpan=2` leaves; keyboard menu and resize under groups; horizontal scroll coupling of both header rows; header-height/scroll-padding change; no fixed-column regression; test value at the contract seam; no speculative group APIs.

#### Commit 3 — Establish the sticky identity shell

**Status:** Completed

**Provisional commit:** `feat(tables): establish the sticky identity shell`

**Work:** Establish a backward-compatible optional generic first-region API in the shared virtual table and header — strict region order identity→analysis→fixed actions, sticky layering/offset/focus behavior, identity-specific 240/280/360 width and resize-callback contract — and mechanically migrate all three existing caller adapters onto the new `renderHeader` callback in the same trunk-safe commit. The adapters forward shell-provided columns and deliberately omit identity, so no identity renderer, store binding, default, or visual behavior activates; existing views render exactly as today.

**Size assessment:** Estimated ~120 changed non-test implementation lines; within the soft target. This is a coherent reusable foundation seam with direct contract proof and no consumer behavior change — not temporary compatibility code and not a line-preparation fragment. No speculative extension points beyond the optional identity object Commit 4 activates.

**Out of scope:**

- Identity activation (no identity object supplied, no identity cell content, no `identityWidth` persistence), grouped Columns control, toolbar, tactic labels, cell consolidation, width-model change beyond what stickiness requires, backend changes.

**Implementation packet:**

- Give the worker a complete handoff for the generic seam plus the mechanical adapter migration; Commit 4 is the first identity consumer.

**Files and responsibilities:**

- `src/components/player-table/virtualized-player-table.tsx` — optional storage-agnostic identity contract on `ConfigurableVirtualizedTable`: the caller supplies one `ConfigurableTableIdentity<TRow>` object (`{ id, label, width, renderCell, onResize }`) plus analysis `columns` and optional fixed columns, and the shell replaces the required opaque `header: ReactNode` prop with an explicit storage-agnostic `renderHeader({ identity, columns, fixedColumns })` callback that the shell invokes to obtain the `<thead>` content. No `header` compatibility prop remains — the old prop is removed, not deprecated alongside. The shell owns the identity `<col>`, body-cell order and rendering (`renderCell` first, in strict order identity → analysis columns → optional fixed action columns), spacer `colSpan`, and minimum-width accounting. This file imports no store and knows no table IDs. `scrollPaddingStart` 64 preserved; virtual spacer math and fixed 40px rows unchanged. When `identity` is absent, the shell renders byte-identical output to today.
- `src/components/player-table/player-table-header.tsx` — `ConfigurableTableHeader` consumes that same identity object and alone renders the sole identity `<th scope="col" rowSpan={2}>` (only when `identity` is present), with a pointer + keyboard resize handle honoring `IDENTITY_COLUMN_MIN_WIDTH = 240`, `IDENTITY_COLUMN_DEFAULT_WIDTH = 280`, `IDENTITY_COLUMN_MAX_WIDTH = 360`; no sort affordance, no context menu, no remove path on the identity header. This file imports no store and knows no table IDs.
- `src/features/search/components/search-results-panel.tsx` (`SearchResultsVirtualTable`), `src/features/squad/components/squad-overview-panel.tsx` (`SquadOverviewTable`), `src/features/staff/components/staff-search-results-panel.tsx` (`StaffSearchTable`) — mechanical adapter migration only: each replaces its `header={...}` JSX with the exact `renderHeader({ identity, columns, fixedColumns })` callback, forwards the shell-provided analysis/fixed columns into the existing header (`PlayerTableHeader` for Search/Squad, `ConfigurableTableHeader` for Staff) with all current sort/add/remove/move/resize/metrics props unchanged, and deliberately omits `identity` (no identity object constructed, no store read, no width passed). No visual, sort, filter, paging, or activation behavior changes.
- `src/components/player-table/configurable-table-contract.test.tsx`, `src/components/player-table/player-table-header.test.tsx` — contract tests exercising the actual shell+header handoff (the shell invokes `renderHeader` with the identity object and the real `ConfigurableTableHeader` renders the sole identity `<th>`; a test-only caller supplies the identity object): region order identity→analysis→fixed; sticky offsets under horizontal overflow; thead layering above scrolled identity cells; `onResize` clamping (below 240 → 240, above 360 → 360, keyboard steps honor bounds); absent-identity output unchanged (all existing behavior assertions and parity remain, but both direct `ConfigurableVirtualizedTable` call sites in `configurable-table-contract.test.tsx` — the two current `header={` uses — mechanically migrate to the required `renderHeader` callback returning the identical `<thead>` output, since the old `header` prop is removed). Adapter-migration proof: all three migrated panels plus both migrated test callers render byte-identical output to today (no `identity` in the DOM, header props / `<thead>` output forwarded verbatim), and a source search over `src/` proves no direct `ConfigurableVirtualizedTable` caller still passes `header` (all three production adapters and both test callers covered).

**Behavior and data flow:**

- Shell accepts an optional storage-agnostic identity object; when present it sticks left under horizontal overflow while grouped + leaf header context stays visible; resize reports clamped widths through `onResize` and the shell applies the supplied `width` without persisting it (persistence lands in Commit 4, where each feature panel reads its `identityWidth` and passes `setIdentityWidth` through `onResize`). All three migrated adapters pass no identity object, so every view renders exactly as today: the shell invokes their `renderHeader` with `identity: undefined` and the existing header output is unchanged. Focus/activation paths are untouched; ArrowUp/ArrowDown focus crosses pages with the 64px header never covering the focused row.

**Ordered implementation steps:**

1. RED: failing contract tests for the optional region (order, sticky offsets, layering, resize-callback clamping via pointer and keyboard, absent-descriptor parity).
2. GREEN: implement the identity object, the `renderHeader` callback, sticky offsets, the sole identity header cell with bounded resize, and callback plumbing with minimal logic.
3. Mechanically migrate the three caller adapters (`SearchResultsVirtualTable`, `SquadOverviewTable`, `StaffSearchTable`) plus both direct test callers in `src/components/player-table/configurable-table-contract.test.tsx` (the two `header={` uses) to `renderHeader`, omitting identity and returning the identical `<thead>` output with all existing behavior assertions retained; confirm all pre-existing shell/header behavior assertions and all affected route tests pass unchanged (parity proof — assertions/parity remain, only call-site syntax migrates because the old prop is removed); run a source search over `src/` proving no direct `ConfigurableVirtualizedTable` caller still passes `header` (all three production adapters and both test callers covered).
4. Refactor only while the focused proof stays green; run targeted then affected validation.

**Tests and proof:**

- New contract tests: strict order with analysis + fixed columns present; sticky identity visible under forced horizontal overflow; open menu renders above identity cells; resize callback receives clamped values at both bounds through pointer and keyboard interaction; default 280 applied when no width supplied.
- Retained behavior assertions: every existing shell/header behavior assertion plus every affected consumer route test — parity is the trunk-safety proof; the adapter and test call-site migration changes no rendered output and no route-test expectation (only the `header={` → `renderHeader` call-site syntax migrates because the old prop is removed).
- GREEN assertions: the shell+header handoff shows the identity region first and sticky; the three migrated adapters plus both migrated test callers render `renderHeader` output identical to their previous `header` output with `identity` omitted; removing the identity object restores current rendering exactly; a source search over `src/` shows no direct `ConfigurableVirtualizedTable` caller still passing `header`.

**Patterns to verify:**

- Current sticky `<thead>` (`sticky top-0 z-10`), `scrollPaddingStart`/`HEADER_HEIGHT` handling from Commit 2, existing pointer + keyboard resize-handle patterns (currently clamped 72–360 for analysis leaves) as the analogue for the 240–360 identity handle.

**Constraints and non-goals:**

- Strictly backward-compatible: no identity renderer, store binding, default, filter/sort, or visual change in any of the six modes — the adapter migration forwards existing header props verbatim and omits identity. No persistence in this commit — the callback contract is storage-agnostic. No hidden `header` compatibility prop remains. No speculative slots or configuration beyond the single optional identity object and the `renderHeader` callback. Neither shared file imports the store or knows table IDs.

**Dependencies and sequencing:**

- Requires Commit 2 (two-row grouped header + `TableGroupInput` + 64px geometry). Commit 4 supplies caller identity content and store binding through this contract only (each panel passes its `identityWidth`/`setIdentityWidth` through `onResize` on top of the migrated `renderHeader` adapters) and invents no wiring. No other prerequisites.

**Validation:** `./scripts/dev test src/components/player-table` then `./scripts/dev test src/app/routes/search.test.tsx src/app/routes/staff.test.tsx src/app/routes/my-club-squad.test.tsx` (unchanged assertions — adapter/test-caller migration parity proof) plus a source search (e.g. `grep -rn "header={" src/components/player-table/configurable-table-contract.test.tsx src/features/search/components/search-results-panel.tsx src/features/squad/components/squad-overview-panel.tsx src/features/staff/components/staff-search-results-panel.tsx`, and a `src/`-wide check that no direct `ConfigurableVirtualizedTable` caller still passes `header`) proving all three production adapters and both test callers migrated; commit gate `./scripts/dev check` before checkpoint review.

**Stop conditions:** Stop if the optional region cannot be added without changing existing rendered output (parity tests fail) — fix or escalate, never ship a visual change here. Stop if sticky offsets disturb virtual spacer math — replan the offset approach.

**Review mandate:** Backward compatibility (absent-identity parity across all three migrated adapters plus both migrated test callers); strict region order; actual shell+header handoff (not a harness-only parallel path); all three production `header={...}` call sites plus both test `header={` call sites in `configurable-table-contract.test.tsx` (five total) migrated to `renderHeader` with verbatim header-prop / `<thead>` forwarding and deliberate identity omission, all existing behavior assertions retained; source-search proof over `src/` that no direct `ConfigurableVirtualizedTable` caller still passes `header`; no remaining `header` prop on the shared shell; one caller-owned identity config with no duplicated ownership; sticky offset correctness under horizontal scroll; header-corner layering; focus visibility under the 64px header; resize bound enforcement (240/280/360) via pointer and keyboard; no store import or table-ID knowledge in either shared file; no store/consumer/default drift; no speculative API surface; test value at the contract seam.

#### Commit 4 — Activate required table identity

**Status:** Completed

**Provisional commit:** `feat(tables): activate required table identity`

**Work:** Atomically activate caller-owned player/staff identity renderers in all six modes on the Commit 3 seam and perform the exact global Zustand v7→v8 migration — persisted `identityWidth` + `setIdentityWidth`, analysis-only defaults/allow lists, no identity duplicates, all-six route/store proof. Migration strips `name`/`club`/`division` globally, so activation and migration ship together.

**Size assessment:** Expected well above 200 changed non-test implementation lines across store + three panels (each panel binds its own `identityWidth`/`setIdentityWidth` through the Commit 3 contract; the shared shell/header take no store binding). Kept together because no split is trunk-safe on its own, and the exact reason is recorded here: (a) a migrated store without caller rendering breaks every table — analysis layouts would lose their Name/Club/Division leaves with nothing rendering identity, so all six modes regress to identity-less rows; (b) caller rendering without the migration duplicates identity fields — the configurable `name`/`club`/`division` leaves would still render alongside the new sticky region in every mode; (c) either half alone requires disposable compatibility machinery (transitional dedup filters, dual-source width resolution, temporary menu hiding) that the whole does not need and that would have to be written, reviewed, and deleted in the next commit; (d) the v8 classification (default-like vs custom vs malformed vs identity-only) is only observable through rendered all-six proof, so migration tests without caller wiring prove nothing user-visible. Only the whole is independently reviewable, revertible, and trunk-safe, which is why this is the single intentionally oversized packet in the plan.

**Out of scope:**

- Grouped Columns control (Commit 5 consumes the same `TableGroupInput`), toolbar, tactic labels, cell consolidation, width-model change beyond what stickiness requires, backend changes.

**Implementation packet:**

- Caller store binding, caller identity cells, migration, and six-caller wiring ship together; no subset is useful alone.

**Files and responsibilities:**

- `src/stores/use-player-table-store.ts` — `PLAYER_TABLE_LAYOUT_VERSION = 8`; layout shape `{ columnIds, widths, identityWidth }` (analysis only + clamped identity width, default 280, clamp 240–360); identity IDs (`name`, `club`, `division`) rejected in `addColumns`/`replaceLayout`/`moveColumn`/`setColumnWidth`; `removeColumn` allows zero-analysis (identity-only valid); new `setIdentityWidth(table, width)` with the 240–360 clamp; `sanitizeLayout` drops identity IDs/widths and additionally rejects tactic IDs outside Search/Moneyball Search and Suggested Training outside Squad for all six tables; per-table v8 analysis defaults exactly as recorded in Decisions; `migratePersistedState` implements this ordered algorithm per table: (1) capture the original finite `widths.name` as the `identityWidth` candidate (clamped 240–360, else 280) — the captured value survives even if step (2) strips the name width; (2) run the existing pre-v8 upgrades unchanged to produce a v7-equivalent layout (the <5 identity normalization, then the <7 Suggested Training rollout); (3) strip identity IDs/widths from analysis; (4) sanitize the remainder through the allowlists with dedupe and width clamping; (5) if the v7-equivalent `columnIds` exactly equal that table's resulting v7 default IDs in order with exactly empty widths → default-like → write the exact v8 analysis defaults with empty widths; (6) else keep the custom remainder verbatim — including an empty remainder, which is valid identity-only and never rolls to defaults; (7) missing/non-array/empty raw `columnIds` is malformed → v8 defaults with `identityWidth` 280.
- Shared shell/header take no store binding in this commit: `src/components/player-table/virtualized-player-table.tsx` and `src/components/player-table/player-table-header.tsx` keep the Commit 3 storage-agnostic contract untouched (no store import, no table-ID knowledge). Each feature panel below reads its layouts' `identityWidth` and `setIdentityWidth` from the migrated store and passes width, `onResize`, and caller-owned render data through that contract; the shell keeps render and sticky order identity → analysis → fixed with `scrollPaddingStart` 64 (from Commit 2), virtual spacer math unchanged, and fixed 40px rows.
- `src/components/player-table/player-table-header.tsx` — identity `<th scope="col" rowSpan={2}>` with resize handle only (no sort, no context menu, no remove path); the identity resize handle exposes the 240–360 bounds (`IDENTITY_COLUMN_MIN_WIDTH = 240`, `IDENTITY_COLUMN_DEFAULT_WIDTH = 280`, `IDENTITY_COLUMN_MAX_WIDTH = 360`). It imports no store and knows no table IDs.
- `src/features/search/components/search-results-panel.tsx` — player identity cells for both `search` and `moneyball-search` table IDs (name primary, club/division secondary with ` · ` join, name-only when both absent; slot order 28px face slot, 8px gap, flexible text stack; secondary line starts with the 12px club-logo slot + 4px gap then club/division text; reserved face and club-logo slots as stable unobtrusive layout boxes with no image loading and no broken-image risk); the panel reads `identityWidth` and `setIdentityWidth` for its table ID and passes them through the Commit 3 identity object (`width`, `onResize`) with caller-owned `renderCell` content; per-view group input (General Search / Moneyball maps from Commit 2); whole-row activation preserved.
- `src/features/squad/components/squad-overview-panel.tsx` — same player identity cells and slot order for `squad`; the panel reads `identityWidth` and `setIdentityWidth` for `squad` and passes them through the Commit 3 identity object with caller-owned `renderCell` content; Squad map (Market, Development, Role Fit); Suggested Training stays Squad-only and sortable-gated as today.
- `src/features/staff/components/staff-search-results-panel.tsx` — staff identity cells through the same required-identity contract for `staff-search`, `my-staff`, and both `staff-shortlist` paths (same width/slot geometry but caller-owned content; where no club logo is applicable the 12px sub-slot remains unobtrusive and alignment-stable). The panel reads `identityWidth` and `setIdentityWidth` for its resolved layout ID and passes them through the Commit 3 identity object with caller-owned `renderCell` content; it invents no wiring. `staffShortlistPresentation(preferredJob) === undefined` (All jobs) keeps the persisted configurable `staff-shortlist` layout with grouped Columns including Recruitment; mapped jobs, Coach, and unrecognized Preferred Job presentations use fixed `fixedColumnIds` with `configurable={false}`, suppress the Columns control, and still render identity first and sticky with the toolbar.
- `src/stores/use-player-table-store.test.ts`, contract/header tests, route tests (`search.test.tsx`, `staff.test.tsx`, `my-club-squad.test.tsx`) — full proof matrix below.

**Exact identity geometry (implemented verbatim):** 40px row; 28px neutral face slot; 8px gap; text budget at least 188px including the secondary 12px club-logo slot + 4px gap; 16px total horizontal cell padding. `IDENTITY_COLUMN_MIN_WIDTH = 240`, `IDENTITY_COLUMN_DEFAULT_WIDTH = 280`, `IDENTITY_COLUMN_MAX_WIDTH = 360`. Current Name default is 224, so the new 280 default intentionally adds the space required by the reserved slots.

**Exact v8 analysis defaults (implemented verbatim; tactic stays opt-in everywhere):** Search `age,nationality,ca,pa,value`; Moneyball `age,nationality,moneyball.minutes,moneyball.average_rating,moneyball.goals_per_90,moneyball.assists_per_90,moneyball.xg_per_90,moneyball.xa_per_90`; Squad `age,nationality,ca,pa,value,suggested_training`; Staff Search and My Staff to the current `DEFAULT_STAFF_TABLE_COLUMN_IDS` minus name/club/division; Staff Shortlist (All-jobs configurable path) to the current shortlist default minus name/club/division — fixed presentations use their `fixedColumnIds` minus identity IDs and never touch the persisted layout.

**Behavior and data flow:**

- On hydrate, persisted state migrates once per the ordered algorithm (original name-width capture, then pre-v8 upgrades to a v7-equivalent layout, then v8 classification); default-like detection is exact-equality plus empty-widths against the correct resulting v7 defaults (any reorder/resize/add/remove is custom, and when in doubt the worker retains, never resets). Staff Shortlist All jobs keeps the persisted configurable layout with grouped Columns; fixed presentations use their `fixedColumnIds` with Columns suppressed. Identity cells render for every row; the region sticks left under horizontal overflow while the grouped + leaf header context stays visible; row click/Enter still activates the whole row; ArrowUp/ArrowDown focus crosses pages with the 64px header never covering the focused row; missing club/division renders name only.

**Ordered implementation steps:**

1. RED: failing store tests for the parameterized migration matrix (six modes × default-like roll, customized retention, resized retention, identity-only validity, malformed safety, shortlist-path handling, tactic/Suggested allowlist enforcement, no name/club/division duplication anywhere).
2. GREEN: implement version bump, identity separation, migration, and exact v8 defaults.
3. Wire each panel's `identityWidth`/`setIdentityWidth` through the Commit 3 contract (panels own the binding; shared files take no store binding), then player identity cells, then staff identity cells, in order Search → Moneyball Search → Squad → Staff Search → My Staff → Staff Shortlist.
4. Refactor only while the focused proof stays green; run targeted then affected validation.

**Tests and proof:**

- Extended `use-player-table-store.test.ts`: update and retain the existing v4–v6 migration tests, plus a parameterized v7 matrix across all six table IDs — v7 default-like (compared against the correct resulting v7 defaults per table, including the post-rollout Squad default) → exact v8 defaults; v7 customized (reordered/resized/added/removed, including old v4–v6 custom layouts) → valid content retained with clamped widths and never reset; `widths.name` captured first → `identityWidth` clamped 240–360 (below 240 clamps to 240, above 360 clamps to 360, absent/non-finite uses 280) and surviving even when normalization strips the name width; identity-only (all-identity raw IDs, or custom emptied to zero) stays valid and never rolls to defaults; malformed → safe v8 defaults; tactic IDs outside Search/Moneyball and Suggested Training outside Squad dropped in every mode; `name`/`club`/`division` absent from all analysis `columnIds`/`widths` and from every configuration menu.
- Header/contract tests: identity `<th>` has no remove affordance and no menu; identity resize handle exposes min 240 / default 280 / max 360 via pointer and keyboard; identity text area stays nonzero and readable at 240; row height stays 40px.
- Route tests per consumer: identity region present first and sticky; reserved slots render with no `img` and no network; name-only fallback; activation via row click and Enter; arrow-key focus across pages; per-view defaults match Decisions exactly.
- GREEN assertions: zero duplication of identity fields in store, menus, and rendered columns for all six consumers.

**Patterns to verify:**

- v6→v7 Suggested Training migration precedent (default-like detection, customized preservation); existing v4–v6 migration tests as the retention base; `replaceLayout` atomicity; existing allowlist structure; current two-line name cell (`h-table-row-height-two-line`, 11px secondary, truncate + `title`) as the visual baseline; DESIGN offline/text-first-identity rules; existing focus/activation handlers.

**Constraints and non-goals:**

- Never silently reset a customized analyst layout — when in doubt, retain. No face/logo loading, no network, no guessed flags/crests. No density change. No filter/sort semantic change. Identity is the clearest affordance, not the only one — do not remove whole-row activation. No backend changes.

**Dependencies and sequencing:**

- Requires Commits 1–3 (plan, group input + geometry, identity shell seam). Commit 5 consumes the same `TableGroupInput` for the Columns control; Commit 7 toolbar assumes the identity region exists.

**Validation:** `./scripts/dev test src/stores/use-player-table-store.test.ts` then `./scripts/dev test src/components/player-table` plus affected route tests; commit gate `./scripts/dev check`.

**Stop conditions:** Stop if migration evidence contradicts the delivery boundary (e.g. identity separation requires IPC/catalog changes) — replan. Stop if default-like detection cannot be made exact-conservative; retain more, not less. Stop if any Staff Shortlist path (configurable All-jobs or fixed presentation) cannot host identity without contract breakage — escalate.

**Review mandate:** Migration conservatism per the exact ordered algorithm (name-width capture first, pre-v8 upgrades to a v7-equivalent layout, then v8 classification against the resulting v7 defaults); identity-width migration clamping (240–360, absent/non-finite → 280) and resize bounds; identity dedup across store/menus/rendering; allowlist enforcement in all six modes; width clamping including `identityWidth`; sticky offset correctness under horizontal scroll; header-corner coherence; focus visibility; no image/network additions; name-only fallback; activation parity (click/Enter/arrows); shortlist-path degradation (All-jobs configurable vs fixed presentations); no unrelated default churn; test coverage of customized-layout survival (including old v4–v6 custom layouts).

#### Commit 5 — Add grouped column management

**Status:** Completed

**Provisional commit:** `feat(tables): add grouped column management`

**Work:** Add the grouped Columns control driven by the same Commit 2 `TableGroupInput` the header consumes: a keyboard-operable grouped list toggling optional analysis columns, identity absent and non-removable throughout, leaf context menus preserved, working in all six modes including the Staff Shortlist path split (All-jobs configurable with Recruitment vs fixed-presentation degradation). Store migration is already complete from Commit 4.

**Size assessment:** Estimated ~120 changed non-test implementation lines; within the soft target. One outcome: the analyst can manage optional analysis columns per group in every mode.

**Out of scope:**

- Store migration (complete), identity rendering/geometry (Commit 4), toolbar association (Commits 7–9 consume this control), tactic labels, cell consolidation, backend changes.

**Implementation packet:**

- Give the worker a complete handoff for the Columns control vertical only.

**Files and responsibilities:**

- `src/components/player-table/player-table-header.tsx` (or the existing Columns/Menu owner colocated with table code) — grouped Columns control rendering the same `TableGroupInput` groups as the header: grouped keyboard-operable checkbox/toggle list over optional analysis columns only; identity absent from the list, the searchable `MetricPicker` results, and every leaf context menu; per-analysis-leaf context menu (Move left/right, Add column, Remove column) kept; `Other` fallback group listed like any other group.
- `src/features/search/components/search-results-panel.tsx`, `src/features/squad/components/squad-overview-panel.tsx`, `src/features/staff/components/staff-search-results-panel.tsx` — wire the control per layout without changing defaults; Staff Shortlist splits paths: All jobs keeps the configurable layout so the Columns slot offers grouped toggles including Recruitment, while defined presentations keep `configurable={false}` semantics so the Columns slot degrades (no toggles offered); identity still renders first and sticky in both paths.
- `src/components/player-table/player-table-header.test.tsx`, `src/components/player-table/configurable-table-contract.test.tsx` — extend: grouped list contents per view map, identity absent, keyboard toggle of an optional column, leaf context-menu operations unchanged, shortlist-path split (All-jobs toggles including Recruitment; fixed presentations offer no toggles).

**Behavior and data flow:**

- Opening Columns shows the view's groups in display priority with their optional analysis columns; toggling adds/removes analysis leaves through the existing `addColumns`/`removeColumn` store paths (identity IDs rejected there per Commit 4); group membership follows the identical mapper the header uses, so header and control cannot diverge; hidden-group and empty-group rules match the header (a group with no available leaves offers nothing).

**Ordered implementation steps:**

1. RED: failing control tests (grouped list per view map, identity absent everywhere, keyboard toggle adds/removes, context-menu move/remove still work, shortlist-path split).
2. GREEN: implement the grouped control on the shared input and wire the three consumer panels.
3. Confirm header-group and control-group outputs agree per layout (same input, same mapper).
4. Refactor only while the focused proof stays green; run targeted then affected validation.

**Tests and proof:**

- Extended header/contract tests: grouped Columns list contains only optional analysis columns grouped per the view map; keyboard toggle flips a column and the header run updates; identity never appears and cannot be removed; leaf context menu still moves/adds/removes analysis leaves; Staff Shortlist All jobs offers grouped toggles including Recruitment while fixed presentations offer no toggles, and identity renders sticky in both paths.
- GREEN assertions: toggling an optional column in the control changes the rendered header runs; identity columns are unofferable.

**Patterns to verify:**

- Current `MetricPicker` category/role-family grouping and searchable patterns as the interaction analogue; existing menu keyboard handling (Shift+F10, arrows, Enter/Escape).

**Constraints and non-goals:**

- Identity stays absent and non-removable; no default changes; no filter/sort semantic change; no new dependency; no speculative cross-view presets.

**Dependencies and sequencing:**

- Requires Commits 2 (shared input) and 4 (identity separation + migrated store). Consumed by Commits 7–9 toolbar wiring.

**Validation:** `./scripts/dev test src/components/player-table` plus affected route tests; commit gate `./scripts/dev check`.

**Stop conditions:** Stop if the control requires diverging from the header's group mapper — reunify or escalate, never ship two group taxonomies. Stop if either Shortlist path breaks its layout contract (All-jobs configurability or fixed-presentation columns) — escalate.

**Review mandate:** Same-input proof (header vs control agreement); identity absence across list/picker/menus; keyboard operability of every toggle; context-menu parity; shortlist-path split; no default churn; test value at the control seam.

#### Commit 6 — Present compact accessible tactic headers

**Status:** Completed

**Provisional commit:** `feat(tables): present compact accessible tactic headers`

**Work:** Show compact current tactic placement identifiers as primary tactic leaf labels with smaller role context; the focusable leaf header carries the complete accessible name (full lane definition); keyboard focus — as well as hover — reveals a visible full-definition tooltip/disclosure next to the leaf. Keep 11 lanes and current/potential interleaving. Accessibility is companion correctness inside this packet, not a separate outcome.

**Size assessment:** Estimated ~80 changed non-test implementation lines; within the soft target.

**Out of scope:**

- Group structure changes, identity, toolbar, cells, defaults, backend scoring changes.

**Implementation packet:**

- Label shortening plus accessibility preservation ship together; one without the other is a regression.

**Files and responsibilities:**

- `src/app/routes/search.tsx` — lane-label view model gains compact primary (placement identifier) plus role context and full definition strings; passes them into `SearchResultsPanel`.
- `src/features/search/components/search-results-panel.tsx` — tactic leaf cells render compact primary with smaller secondary role context; the focusable leaf header exposes the full `IP Position (Role) / OOP Position (Role)` definition as its complete accessible name, and focusing (as well as hovering) the leaf reveals a visible definition tooltip/disclosure. `title` may only supplement, never satisfy — `title`-only is explicitly not proof.
- `src/app/routes/search.test.tsx`, header tests — compact rendering, full-definition reachability without a pointer, interleaving and sort behavior unchanged.

**Behavior and data flow:**

- With tactic groups active, each tactic leaf shows e.g. placement primary with role context secondary; screen-reader and keyboard users get the full `IP Position (Role) / OOP Position (Role)` definition; sorting, null-last ordering, UID tie-break, and interleaved/straight order rules are untouched; no 12th lane appears.

**Ordered implementation steps:**

1. RED: failing tests for compact labels, secondary role context, and non-hover full-definition access.
2. GREEN: implement the label view model change and leaf rendering.
3. Confirm interleaving, sort acceptance, and header-X recompaction still pass unmodified.
4. Run targeted then affected validation.

**Tests and proof:**

- Route tests: compact primary visible, role context visible, accessible name carries the full definition, keyboard Tab onto the leaf reveals a visible definition (assert visibility, not the `title` attribute), 11 lanes only.

**Patterns to verify:**

- Current `tacticLaneLabels` route ownership; `ScoreBadge` accessible naming pattern; DESIGN keyboard-reaches-everything rule.

**Constraints and non-goals:**

- No tactic scoring, mapping, or lane-count change. No hover-only information. No filter/picker exposure of tactic IDs.

**Dependencies and sequencing:**

- Requires Commit 2 (grouped header leaves); sequenced after Commits 4–5 so the tactic leaves render in their final grouped/sticky context. Independent of toolbar and cells.

**Validation:** `./scripts/dev test src/app/routes/search.test.tsx` plus header tests; commit gate `./scripts/dev check`.

**Stop conditions:** Stop if the full definition cannot be made visible on keyboard focus within a leaf header — do not ship shortened labels alone; escalate.

**Review mandate:** Label correctness per lane; secondary-context styling restraint; complete accessible name; visible focus-revealed definition (`title` may only supplement, never satisfy — a `title`-only implementation fails review); no hover-only path; lane count and interleaving intact; sort acceptance untouched.

#### Commit 7 — Integrate the Search table toolbar

**Status:** Completed

**Provisional commit:** `feat(tables): integrate the Search table toolbar`

**Work:** Introduce the minimal reusable generic toolbar contract and prove it through Search/Moneyball: move generic Search summary/chips/Clear/Edit/Columns/dataset toggles into table association, while page actions Add Tactic (current/potential) and General Upload Shortlist move to the Search page header outside the toolbar. One real consumer validates the abstraction; Commits 8–9 adopt it without redesign.

**Size assessment:** Estimated ~150 changed non-test implementation lines (shared contract + one consumer wiring); within the soft target. Contract plus its first proven consumer are one outcome — a contract without a consumer is unproven abstraction.

**Out of scope:**

- Staff/Squad adoption (Commits 8–9), filter/sort semantic changes, page-action behavior changes, backend changes.

**Implementation packet:**

- Shared contract plus Search wiring ship as one proven outcome.

**Files and responsibilities:**

- New/existing shared toolbar module under `src/components/player-table/` — toolbar contract: summary slot, chips slot (remove per chip, Clear-all equivalent, edit action where filters exist), grouped Columns slot (Commit 5 control), dataset-toggles slot; associated with the table region (labels/landmarks tie controls to the dataset). The contract accepts no page-action children: Add Tactic, Upload Shortlist, Configure, Optimize, boosts, and Squad actions are not toolbar slots.
- `src/app/routes/search.tsx` — Search page header owns Add Tactic (current/potential) toggles and the General Upload Shortlist entry (relocated from the filter bar's `actions`/`afterEditActions` slots, same behavior); passes dataset state into the toolbar slots without moving page actions into the toolbar element.
- `src/features/search/components/search-filter-bar.tsx` (+ strip) — Search dataset controls (summary, chips, Clear-all, Edit filters) move into table association via the contract; drafts stay query-silent; Done applies one URL update; filter-adds-column-once preserved. The old separate-panel strip placement is removed only after the toolbar wiring is green.
- `src/features/search/components/search-results-panel.tsx` — hosts the toolbar in table association for both `search` and `moneyball-search` table IDs.

**Behavior and data flow:**

- Each Search table shows its count/sort summary, active chips with per-chip remove and an edit entry point, and grouped column configuration in one table-associated region; removing the last chip restores the unfiltered set; editor Cancel/Escape/backdrop discards drafts without queries; URL remains the source of truth for filters/sort; comparison-pool controls keep current placement rules (inside toolbar view slots, not generic page chrome).

**Ordered implementation steps:**

1. RED: failing contract tests for slots (summary, chips+remove, edit action, column config, view slot) and Search tests for preserved URL/draft semantics.
2. GREEN: implement the shared contract, then wire Search and Moneyball Search in that order.
3. Remove the old separate-panel strip placement only after its toolbar wiring is green.
4. Run targeted then affected validation.

**Tests and proof:**

- Contract tests for the toolbar slots; route tests proving filter URL behavior, silent drafts, chip removal, Clear-all, and comparison-pool wiring unchanged; page-action tests assert Add Tactic / Upload Shortlist render outside the toolbar element (not descendants) with URL/draft/toggle behavior preserved.

**Patterns to verify:**

- Current `SearchFilterStrip` chip/remove/Clear-all/draft patterns; `afterEditActions`/`headerActions` slot precedents; Panel/toolbar styling tokens.

**Constraints and non-goals:**

- The toolbar never owns Add Tactic or Upload Shortlist. No filter-operator or combine-mode semantic change. No new query behavior.

**Dependencies and sequencing:**

- Requires Commits 2, 4, 5 (groups, identity region, Columns control). Commits 8–9 adopt this contract; no redesign there unless source disproves it here.

**Validation:** `./scripts/dev test src/app/routes/search.test.tsx` then `./scripts/dev test src/components/player-table`; commit gate `./scripts/dev check`.

**Stop conditions:** Stop if Search filter URL semantics or draft-silence cannot be preserved — keep the old placement and escalate. Stop if page actions leak into the generic contract — remove them.

**Review mandate:** Slot minimalism (no page-action children — assert non-descendance); URL/draft parity; chip accessibility (labelled remove); comparison-pool wiring; no duplicated filter logic; test parity with old strips; old Panels removed only after wiring.

#### Commit 8 — Integrate the Staff table toolbar

**Status:** Completed

**Provisional commit:** `feat(tables): integrate the Staff table toolbar`

**Work:** Adopt the shared Commit 7 toolbar for Staff Search/My Staff/Staff Shortlist: Staff page header owns Upload Shortlist/Configure/Optimize; My Staff bulk boost remains outside; preserve shortlist fields/toggles/filter URL in both paths (All-jobs configurable layout and fixed-presentation columns).

**Size assessment:** Estimated ~100 changed non-test implementation lines; within the soft target. One outcome: Staff consumers reach toolbar parity with Search under Staff ownership rules.

**Out of scope:**

- Toolbar-contract redesign (adopt as-is), Search/Squad changes, filter/sort semantic changes, backend changes.

**Implementation packet:**

- Staff adoption only; the contract is proven in Commit 7.

**Files and responsibilities:**

- `src/app/routes/staff.tsx` — Staff page header owns Upload Shortlist, Configure, and Optimize placement (assignment/upload actions stay as page-header content, never generic toolbar controls).
- `src/features/staff/components/staff-filter-bar.tsx`, `src/features/staff/components/staff-search-results-panel.tsx` — Staff dataset controls via the contract for `staff-search`, `my-staff`, and `staff-shortlist` (Preferred Job / Only unemployed while shortlisting). My Staff boost actions remain outside the toolbar. Staff Shortlist splits paths: All jobs keeps the configurable layout with the grouped Columns slot including Recruitment; defined presentations keep `configurable={false}` semantics — toolbar Columns slot degrades accordingly.

**Behavior and data flow:**

- Each Staff table shows its summary, chips, and grouped Columns in table association; URL remains the source of truth; shortlist filtering uses the `staff-shortlist` layout per path (All jobs keeps the persisted configurable layout; defined presentations swap to fixed `fixedColumnIds`), with identity first and sticky in both; editor drafts stay silent until applied.

**Ordered implementation steps:**

1. RED: failing Staff tests for toolbar association with preserved URL/draft/shortlist semantics.
2. GREEN: wire the three Staff layouts through the contract in order Staff Search → My Staff → Staff Shortlist.
3. Remove the old separate-panel strip placement only after its toolbar wiring is green.
4. Run targeted then affected validation.

**Tests and proof:**

- Route tests proving filter URL behavior, silent drafts, chip removal, Clear-all, Preferred Job / Only-unemployed toggles, and shortlist-path behavior unchanged (All-jobs configurable add/remove; fixed presentations offer no Columns control); page-action tests assert Upload Shortlist / Configure / Optimize render outside the toolbar element.

**Patterns to verify:**

- Current `StaffFilterStrip` patterns; Commit 7 contract usage as the closest analogue — mirror it, diverge only where Staff ownership requires.

**Constraints and non-goals:**

- No toolbar-contract redesign unless Staff source disproves Commit 7 (which triggers replanning, not silent redesign). No boost-action moves. No filter semantic change.

**Dependencies and sequencing:**

- Requires Commits 4, 5 (identity, Columns control) and 7 (toolbar contract). Independent of Commits 6 and 9–10.

**Validation:** `./scripts/dev test src/app/routes/staff.test.tsx` then `./scripts/dev test src/components/player-table`; commit gate `./scripts/dev check`.

**Stop conditions:** Stop if Staff filter URL semantics or either shortlist-path contract cannot be preserved — keep the old placement for that layout and escalate. Stop if the contract needs redesign — replan instead of forking it.

**Review mandate:** Contract reuse without fork; page-action non-descendance; URL/draft parity; shortlist toggles and path-split degradation; boost actions untouched; no duplicated filter logic.

#### Commit 9 — Integrate the Squad table toolbar

**Status:** Completed

**Provisional commit:** `feat(tables): integrate the Squad table toolbar`

**Work:** Adopt the shared Commit 7 toolbar for Squad summary + grouped Columns; no filters; boosts/uploads/actions remain in the existing feature-owned Panel/page action area outside the toolbar; preserve sort/query/replacement/Suggested Training.

**Size assessment:** Estimated ~60 changed non-test implementation lines; within the soft target. One outcome: Squad reaches toolbar parity with no filter surface.

**Out of scope:**

- Toolbar-contract redesign, filter additions to Squad, boost/upload/action moves, backend changes.

**Implementation packet:**

- Squad adoption only; the smallest toolbar consumer.

**Files and responsibilities:**

- `src/features/squad/components/squad-overview-panel.tsx` — Squad summary/controls via the contract for the `squad` layout (no filters; sort + grouped Columns + dataset toggles). Boosts, uploads, and Squad actions stay in the existing feature-owned Panel/page action area outside the toolbar.

**Behavior and data flow:**

- Squad table shows its summary and grouped Columns in table association; sort/query-replacement semantics, Suggested Training gating, and existing action placement are unchanged.

**Ordered implementation steps:**

1. RED: failing Squad tests for toolbar association with preserved sort/query semantics.
2. GREEN: wire the Squad layout through the contract.
3. Run targeted then affected validation.

**Tests and proof:**

- Route tests proving Squad summary, sort behavior, Query replacement, and Suggested Training gating unchanged; action-area tests assert boosts/actions render outside the toolbar element.

**Patterns to verify:**

- Commit 7–8 contract usage as the analogue; current Squad sort-replacement and `onAddColumn` patterns.

**Constraints and non-goals:**

- No Squad filters introduced. No action moves. No semantic change.

**Dependencies and sequencing:**

- Requires Commits 4, 5 (identity, Columns control) and 7 (toolbar contract). Independent of Commits 6, 8, 10.

**Validation:** `./scripts/dev test src/app/routes/my-club-squad.test.tsx` then `./scripts/dev test src/components/player-table`; commit gate `./scripts/dev check`.

**Stop conditions:** Stop if Squad sort/replacement semantics cannot be preserved — keep the old placement and escalate.

**Review mandate:** Contract reuse without fork; no filter surface added; action-area non-descendance; sort/replacement parity; Suggested Training gating intact.

#### Commit 10 — Standardize shared table cells

**Status:** Completed

**Provisional commit:** `feat(tables): standardize shared table cells`

**Work:** Consolidate common text/numeric/currency/score/comparison/unavailable cell presentation into reusable shared ownership only where current callers need it; reuse ScoreBadge and `—`; no new scoring.

**Size assessment:** Estimated ~120 changed non-test implementation lines; within the soft target.

**Out of scope:**

- Scoring, sorting, filtering, headers, toolbar, defaults, backend changes.

**Implementation packet:**

- Consolidation follows the consumers; shared helpers cover exactly the duplicated cases.

**Files and responsibilities:**

- `src/components/player-table/` — shared cell helpers (text with truncate/`title`, numeric tabular right-aligned, currency via `formatMoney`, score via ScoreBadge `table` variant, comparison cell, unavailable `—`); loading (`…`) vs missing (`—`) distinction preserved.
- `src/features/search/components/search-results-panel.tsx`, `src/features/squad/components/squad-overview-panel.tsx`, `src/features/staff/components/staff-search-results-panel.tsx` — adopt shared cells where duplication exists today; caller-specific cells (nationality flags, suggested training, shortlist context, moneyball percentile+value pairs) stay caller-owned unless true duplication is proven.
- Cell tests at the contract seam; caller route tests confirm unchanged rendering.

**Behavior and data flow:**

- Rendered values are byte-identical for existing cases (same formatters, same ScoreBadge tiers/names, same `—` honesty); moneyball value+percentile pairs and missing-score `—` keep current rules; no cell changes sort/filter inputs.

**Ordered implementation steps:**

1. RED: failing contract tests pinning current rendering (currency, counts, scores, `—`, loading `…`) at the shared seam.
2. GREEN: extract shared helpers and adopt them per caller, one caller at a time.
3. Delete only proven-duplicate caller code; keep caller-specific cells where they diverge.
4. Run targeted then affected validation.

**Tests and proof:**

- Contract tests for each shared cell kind including null/undefined/empty-string honesty; route tests confirm Search/Squad/Staff render unchanged; no new scoring-rule tests (no new rules).

**Patterns to verify:**

- `formatMoney`/`formatCount`/`formatMissable`/`formatPlayerDob` single-ownership; ScoreBadge `table` variant + accessible naming; DESIGN tabular-nums/alignment/missing-value rules.

**Constraints and non-goals:**

- No abstraction for hypothetical cells — shared ownership only where at least two current callers need it. No color-on-chrome, no ramp change, no unit change.

**Dependencies and sequencing:**

- Requires Commits 2 and 4 (grouped headers, identity cells exist). Sequenced after Commits 7–9 so final rendering passes cover the finished toolbar layout.

**Validation:** `./scripts/dev test src/components/player-table` plus affected route tests; commit gate `./scripts/dev check`.

**Stop conditions:** Stop consolidating a cell kind if caller divergence is real — keep caller ownership and record it. Never change a displayed value to fit the abstraction.

**Review mandate:** Duplication proof per helper; value-identical rendering; honesty of missing vs loading vs zero; moneyball pair rules intact; no over-general API; no formatter duplication.

#### Commit 11 — Enforce bounded column widths

**Status:** Completed

**Provisional commit:** `feat(tables): enforce bounded column widths`

**Work:** Fix the width model so minimum widths + horizontal overflow hold, sticky identity and full grouped+leaf header context stay visible, 3440×1440 reveals more columns instead of stretching cells without bound; add explicit 1280×800 and 3440×1440 browser proof. Viewport proof is companion validation inside this packet, not a separate outcome.

**Size assessment:** Estimated ~100 changed non-test implementation lines plus test/viewport proof; within the soft target.

**Out of scope:**

- Any behavior, default, scoring, or semantic change. This commit is layout + proof only.

**Implementation packet:**

- Width-model fix and viewport proof ship together; the fix is unproven without the proof.

**Files and responsibilities:**

- `src/components/player-table/virtualized-player-table.tsx` — width-model change (replacing the `width: 100%` proportional-stretch `<col>` behavior with a containment model that honors readable minimums, overflows horizontally in the same container, and reveals more columns on ultrawide instead of stretching); sticky offsets re-verified; virtual spacer unaffected.
- `e2e/smoke.spec.ts` — explicit 1280×800 checks (existing containment assertions extended to grouped headers + sticky identity + toolbar) and new 3440×1440 checks (more columns visible than at 1280, cells not stretched without bound, sticky identity + full header context visible, no document-level vertical growth).
- Route tests — overflow at 1280 with wide grouped layouts (extending the existing readable-minimums test).

**Behavior and data flow:**

- At 1280×800: table owns vertical scroll, horizontal overflow appears once minimums exceed the viewport, sticky identity and both header rows stay visible, focused rows stay uncovered. At 3440×1440: additional columns are revealed; cells keep bounded widths rather than stretching; no Previous/Next UI; no full-dataset load.

**Ordered implementation steps:**

1. RED: failing viewport/overflow tests (1280 overflow with grouped+identity layout; 3440 reveal-vs-stretch assertions).
2. GREEN: implement the width-model change and sticky-offset adjustments.
3. Extend `e2e/smoke.spec.ts` with the 3440×1440 pass and re-run the 1280×800 containment passes.
4. Run targeted, full UI, and browser suites in the recorded order.

**Tests and proof:**

- Extended overflow/containment route tests; `e2e/smoke.spec.ts` passes at both viewports; Playwright assertions on scroller vs document scroll geometry, sticky visibility, and bounded cell widths.

**Patterns to verify:**

- Existing smoke containment tests (Search/Squad inside desktop viewports, horizontal-minimums test, retry overlay geometry) as the extension base; `minimumTableWidth` semantics; DESIGN 1280×800 minimum rule.

**Constraints and non-goals:**

- No behavior change smuggled with the layout fix. No new breakpoints below 1280. Desktop only.

**Dependencies and sequencing:**

- Requires Commits 2, 4, 5, 7, 8, 9, 10 (final header/identity/Columns/toolbar/cells all present — proof covers the finished surface). Last implementation commit.

**Validation:** `./scripts/dev test src/app/routes/search.test.tsx`, `./scripts/dev test`, `./scripts/dev check`, `./scripts/dev smoke` with viewport evidence recorded.

**Stop conditions:** Stop if the width-model fix breaks virtualization or containment irreparably — revert to the current model for narrow viewports, keep ultrawide proof as an accepted gap only with developer approval; never report unrun browser proof as passed.

**Review mandate:** Width math (minimums, overflow owner, no document scroll); sticky visibility at both viewports; bounded ultrawide cells; virtual spacer integrity; smoke assertions measuring the right geometry; no behavior drift.

## Discoveries and replanning

- 2026-09-06 bounded replanning (developer feedback on atomicity): restructured the delivery sequence from one planning commit plus six implementation commits to one planning commit plus exactly ten implementation commits. Titles now describe one outcome and avoid "and". Changes: header geometry stays inside the grouped-header packet (Commit 2) as companion correctness; the reusable identity seam is established first as a backward-compatible foundation with direct contract proof and no consumer change (Commit 3); global identity activation and the stripping v7→v8 migration stay atomic (Commit 4, the only intentionally oversized packet, with the exact >200-line reason recorded); grouped column management moves out of the migration into its own packet consuming the same `TableGroupInput` (Commit 5); tactic accessibility stays with compact labels (Commit 6); the toolbar contract is introduced once through Search/Moneyball (Commit 7) and adopted per consumer by Staff (Commit 8) and Squad (Commit 9); cells (Commit 10) and bounded widths with viewport proof (Commit 11) close the sequence. Delivery fingerprint reset to `Pending plan review`. All accepted intent, decisions, exact migration rules, group maps, validation, risks, and the no-density virtualization boundary are preserved unchanged.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Redesign shared table for analysis-first views | Commit 1 — Record the approved feature plan | 8de6d8fc26092383c53fbd423df3568471ac0193 | Recorded the accepted schema 2 feature ledger and TODO Active entry on the authorized feature branch. | `ledger_state.py` and `delivery_state.py` runnable with the accepted fingerprint; staged diff check clean. | Not applicable | Clear | 0 | Branch authority was renamed to `feature/analysis-first-tables` and re-fingerprinted before delivery at the developer's request. |
| PR 1 — Redesign shared table for analysis-first views | Commit 2 — Support grouped table headers | 630b89999367e0234167edb51ddb77c330235e12 | Added shared contiguous group resolution, semantic two-row headers, exact group maps for all six layouts, and 64px sticky-header focus geometry. | Component tests 21/21; affected route tests 243/243; `./scripts/dev check` passed; LSP and diff checks clean. | Pass | Clear | 1 | Initial review required direct proofs for all six panel-owned maps and runtime virtualizer focus geometry; correction review accepted them. |
| PR 1 — Redesign shared table for analysis-first views | Commit 3 — Establish the sticky identity shell | 98c0cf80a9d60880e00f1a7a5aca199eb143e176 | Added one optional storage-agnostic identity contract, exact shell-to-header callback handoff, strict identity-analysis-fixed ordering, sticky layering, and bounded identity resizing; migrated all callers with identity omitted. | Component tests 32/32; affected route tests 243/243; `./scripts/dev check` passed; LSP and diff checks clean. | Pass | Clear | 1 | Initial review required a combined menu-over-sticky-identity layering proof; correction review accepted it. |
| PR 1 — Redesign shared table for analysis-first views | Commit 4 — Activate required table identity | 10f9a9fa815a1427b79e1b87e1b425d5952a35bd | Migrated layouts to version 8 with separate identity widths and conservative custom-layout retention, then activated sticky caller-owned identity across all six table modes without imagery or duplication. | Store tests 51/51; affected component and route tests 279/279; `./scripts/dev check` passed; LSP and diff checks clean. | Pass | Clear | 1 | Initial review found nonempty layout replacement dropped identity width; correction preserved it and added persistence regression proof. |
| PR 1 — Redesign shared table for analysis-first views | Commit 5 — Add grouped column management | 484a8bf6bb5ccf853121c1049a9e98ef1a55a78d | Added one keyboard-operable grouped Columns control driven by the header group input, with analysis-only toggles, identity exclusion, zero-analysis support, and fixed-shortlist degradation. | Component and store tests 93/93; Search 78/78; Staff 47/47; Squad 130/130; `./scripts/dev check` passed; LSP and diff checks clean. | Pass | Clear | 1 | Initial review found missing dialog focus and tactic-only removal restoring defaults; correction fixed leaf and whole-group identity-only paths. |
| PR 1 — Redesign shared table for analysis-first views | Commit 6 — Present compact accessible tactic headers | 64fdf8b4cb270797ef91f7514ee803d4de607d58 | Added compact tactic placement labels with restrained role context while preserving full accessible definitions through keyboard-focus and hover disclosure for all 11 lanes. | Header and Search tests 123/123; affected Staff, Squad, and store tests 228/228; `./scripts/dev check` passed; LSP and diff checks clean. | Pass | Clear | 0 | None |
| PR 1 — Redesign shared table for analysis-first views | Commit 7 — Integrate the Search table toolbar | 5d0b0c0fab408d016e85aeba536442f9294074e5 | Added a minimal shared table-associated toolbar and adopted it in Search and Moneyball while keeping tactic and upload actions in the route-owned page header. | Full frontend suite 931/931; `./scripts/dev check` passed; diff check clean. | Pass | Clear | 2 | Corrected three obsolete post-v8 test assumptions from earlier packets; reviews then required same-row page-header actions, current analysis group fixtures, and removal of an invalid type-only test. |
| PR 1 — Redesign shared table for analysis-first views | Commit 8 — Integrate the Staff table toolbar | de54e4b4b730ca9e9b78c03e4fd85c10b5581cd7 | Adopted the shared toolbar across Staff Search, My Staff, and both Staff Shortlist paths while keeping page actions and boosts outside; preserved visible committed rows and truthful dynamic-field replacement recovery. | Staff tests 53/53; full frontend suite 936/936; `./scripts/dev check` passed; LSP and diff checks clean. | Pass | Clear | 1 | The first worker reached its 100-turn limit; a finisher completed replacement-state behavior. Initial review then required visible error and retry handling for requested dynamic fields. |
| PR 1 — Redesign shared table for analysis-first views | Commit 9 — Integrate the Squad table toolbar | e0ac8961b143f1e2edfc4e95e26cb9c826f7542c | Adopted the shared toolbar for Squad summary and grouped Columns without adding filters or moving boosts and feature actions. | Squad tests 132/132; component tests 47/47; full frontend suite 938/938; `./scripts/dev check` passed; LSP and diff checks clean. | Pass | Clear | 0 | None |
| PR 1 — Redesign shared table for analysis-first views | Commit 10 — Standardize shared table cells | 3466681bfea840adbc3e3812b67cf6c64735b5a0 | Consolidated proven shared player text, numeric, currency, dynamic, and score presentation while preserving caller-specific cells and exact loading, missing, zero, and accessible score semantics. | Component tests 54/54; affected route tests 268/268; full frontend suite 945/945; `./scripts/dev check` passed; diff check clean. | Pass | Clear | 2 | Initial review required shared Search/Squad basic cells and zero proof; correction review then found stale Staff score precedence, fixed by loading-first rendering and recovery proof. |
| PR 1 — Redesign shared table for analysis-first views | Commit 11 — Enforce bounded column widths | Pending record | Replaced proportional table stretching with exact bounded pixel widths and added unit, route, and real-browser proof for shared-scroll overflow, sticky context, virtualization, and ultrawide column reveal. | Full frontend suite 947/947; `./scripts/dev check` passed; focused browser cases 4/4 and full smoke suite 60/60 passed; LSP and diff checks clean. | Pass | Clear | 0 | Updated four stale smoke contracts from pre-grouped and pre-identity behavior while retaining their geometry, sorting, and activation proofs. |

## Final validation

1. Focused table/store/route suites: `src/components/player-table`, `src/stores/use-player-table-store.test.ts`, `src/app/routes/search.test.tsx`, `src/app/routes/staff.test.tsx`, `src/app/routes/my-club-squad.test.tsx`.
2. Full UI suite: `./scripts/dev test`.
3. Full gate: `./scripts/dev check`.
4. Browser product suite with viewport evidence: `./scripts/dev smoke`, including explicit 1280×800 and 3440×1440 layout checks from Commit 11.
5. Unsupported or skipped validation is reported as a gap, never as a pass. `./scripts/dev mutate` remains unsupported and is never reported as passed.

## Documentation impact

Complete during reconciliation. Expected: DESIGN.md table-section update (two-row grouped headers, 64px sticky header geometry, sticky identity region, table-associated toolbar). No ARCHITECTURE change (this extends the established shared component and store seams) and no ADR (no durable consequential alternative meets the threshold). If implementation evidence meets the ADR threshold, record the need here and replan — this run is not authorized to write it.
