# Player search

## Status

Active

## Intent

Give the user a Genie Scout–style **Search** surface over the active save’s current snapshot: a virtualized player table, FM-like filter rules (field + operator + value, combined with AND or OR), sortable columns, and a global top-bar name search. Role-score and attribute filters are in scope so this feature delivers the moneyball path, not only identity browse.

## User-visible behavior

- Nav rail gains **Search** → route `/search`.
- Page shows a virtualized results table for the active save’s current snapshot.
- **Basic columns (always):** Name, Age/DOB, Nationality, Club, Division, CA, PA, Market value.
- **Dynamic columns:** any non-basic field that appears in an active filter also appears as a column (including selected role scores and attributes). Removing the filter removes that column.
- **Compact filter strip** above the table shows active rules as removable tags; **Edit filters** opens a full filter editor modal (FM-like).
- Each rule: property + operator + value. Strings: contains / does not contain / is / is not. Integers (and similar): greater than / less than / equals / does not equal.
- Rules combine with a single **AND | OR** mode (flat list — no nested groups in this feature).
- Filters apply immediately (no Apply button). Active filters, combine mode, sort field, and sort direction live in URL search params.
- Table headers sort; **default sort is CA descending**.
- Row click is a **no-op** until player profiles exist.
- Top bar: global name search (pill), focusable with **Ctrl+K**, debounced live results ranked by match quality then CA.
- Empty states: no snapshot → point at Load Data; no matches → clear filters; truncated snapshot → warning banner.

## Invariants

- Query and filter evaluation run in **Rust against SQLite** for the active save’s current snapshot only. The WebView never opens the DB and never holds the full player set.
- List IPC returns **windowed pages** (offset/limit or equivalent) plus total match count — enough for virtualization without loading all rows.
- `null` / unknown dump values never coerce to 0 for filtering or display; missing cells render as `—`.
- Role scores come from `player_role_scores` (ingest-time); the WebView does not recompute scores.
- Filter inputs are validated at the Rust trust boundary.

## Non-goals

- Player profile page or row navigation into profiles.
- Nested filter groups / saved filter presets / export.
- Editing FM or snapshot data.
- Snapshot history or cross-save search.
- Normalizing attributes out of JSON into relational columns (unless a later discovery forces it).
- Combined IP+OOP weight UI (still deferred to squad planner).

## Current-state map

- Relevant components: `AppNavRail` (Dashboard only); home dashboard sanity list (`list_sanity_players`); `AppTopBar` (no global search field yet); shared `TextField` / `SelectField` / `EmptyState` / `Panel`; **no Modal** primitive yet.
- Data model: `players` scalars + JSON maps (`attributes_json`, `hidden_attributes_json`, `personality_json`, `nationalities_json`, `positions_json`); `player_role_scores` (68 roles per player).
- Persistence: migrations through v3; indexes on `(snapshot_id, name)` and `(snapshot_id, ca DESC)` plus role `(snapshot_id, role_id)`.
- Existing assumptions: thin frontend / thick backend; URL-shareable view state in TanStack Router search params; Query for IPC.
- Architectural seams: new `features/search` (React + Rust); reads snapshot tables; may call scoring catalog for role field labels/ids only.
- Tests: Vitest + mockIPC; Rust service tests on temp DBs; Playwright smoke stubs for new IPC.
- Primary risks: JSON attribute filters at ~180k players; filter AST complexity; virtualization + windowed fetch correctness.

## Feature architecture (this feature)

```text
React features/search
  → api/ — searchPlayers / suggestPlayers query options
  → components/ — results table, compact filter strip, filter editor modal content
  → types/ — filter rule DTOs mirrored from IPC

App shell
  → routes/search — thin composition
  → AppNavRail — Search item
  → AppTopBar — global name field + Ctrl+K + live result popover

Rust features/search
  → commands.rs — search_players, suggest_players (names TBD at build)
  → service/filter.rs — validate filter AST; field registry; operators
  → service/query.rs — parameterized SQL; pagination; sort; dynamic select for filtered fields
  → Does not own ingest; reads players + player_role_scores

Shared UI
  → Modal primitive (first use) per DESIGN.md
  → TanStack Virtual for the results table (add dependency when building the table commit)
```

**Field-type decisions (operator #8):**

| Kind | Examples | Operators | Match rule |
| --- | --- | --- | --- |
| String | name, club, division, parent club | contains / not contains / is / is not | SQL `LIKE` / equality; case-insensitive for name/club-like text |
| Integer | age, CA, PA, height, wage, value, reputation, birth year, contract year | gt / lt / eq / neq | Direct column compare; `null` never matches |
| Boolean | transfer listed, loan listed, not for sale, set for release, on loan | is / is not | Value Yes/No → 0/1; `null` never matches Yes or No |
| Enum string | preferred foot, team level | is / is not | Closed select of known values |
| String list | nationalities | string ops | Match if **any** list element satisfies the op |
| Position | position key (+ optional suitability) | string ops on key presence; integer ops on suitability for a chosen key | Has-position vs suitability threshold |
| Attribute maps | visible / hidden / personality keys | integer ops | `json_extract` on the map; `null` JSON values never match |
| Role score | catalog `role_id` | integer ops | JOIN/filter `player_role_scores.score`; `null` score never matches |

## Uncertainty register

### Known

- Snapshot + role scores already persist; sanity list proves join path.
- DESIGN.md currently specs an inspector filter panel and range sliders — **superseded for this feature** by compact strip + modal + operator rules (reconcile DESIGN when UI lands).
- ARCHITECTURE notes TanStack Virtual is intentionally not in the default stack — add when the table commit needs it.

### Assumptions

- Flat AND/OR (not nested groups) matches “similar to FM” enough for MVP.
- Windowed offset/limit pagination is enough; cursor pagination only if offset proves too slow.
- `json_extract` for attributes is acceptable for hobbyist scale; if not, spike then index or reshape.

### Decisions

- Nav label **Search** (not Database).
- Role scores and attributes are **in** this feature.
- Dynamic columns follow filters (plus fixed basics).
- Global Ctrl+K live name search is **in** this feature.
- Row activation deferred to profiles.

### Unknowns

- Whether attribute JSON filters need a performance spike before merge of PR 3 — decide with a timed Rust test on a large fixture after the scalar filter path exists.
- Exact match-rank tiers for global suggest (exact / prefix / contains) — implement the simplest clear ordering in PR 4; refine if UX feels wrong.

### Risks

- Large filter ASTs generating heavy SQL.
- Virtualizer requesting rows outside fetched windows (need coherent page cache or overscan fetch).
- Modal + URL param sync edge cases (back/forward).

## Walking skeleton

PR 1: `/search` in the rail → paged IPC → virtualized basic columns → sortable headers (default CA desc) → empty/no-snapshot states. Filters and global search land in PR 2.

## Delivery plan

**2 PRs, 10 commits** — same atomic commit breakpoints as the first draft; only the PR boundaries are condensed (was 4 PRs).

### PR 1 — Search page with paged player list

**Status:** Implementation complete — merge when ready

**Provisional PR title:** `feat(search): add Search page with paged player list`

**Purpose:** Walking skeleton on trunk — browse the active snapshot before filter complexity.

**Merge to trunk when:** Virtualized table shows basic columns; headers sort (default CA desc); gate and smoke green.

**Depends on:** Snapshot ingest + role scoring (done).

#### Commit 1 — Paged player list IPC

**Status:** Completed — `889aed7`

**Work:** Add Rust `features/search` with `search_players`: active current snapshot; page of basic summary DTOs + `uid` + `total`; `offset`/`limit` with server-side cap; default order CA descending; empty when no snapshot. Parameterized SQL. Unit tests.

**Out of scope for this commit:**
- Filters, role-score columns, React UI, sort whitelist beyond default CA, global suggest.

**Validation:** `cargo test` for empty snapshot, ordered page, limit cap; `./scripts/dev check` when Rust is touched.

**Provisional commit:** `feat(search): add paged player list IPC for current snapshot`

#### Commit 2 — Search route and virtualized table

**Status:** Completed — `bbec416`

**Work:** `/search` route, nav **Search**, Query wiring, TanStack Virtual table, basic columns, loading / no-snapshot / empty states. Invalidate on save switch / Load Data. Smoke stub for the new IPC.

**Out of scope for this commit:**
- Filters, sorting UI beyond default CA order from IPC, global search, row navigation.

**Validation:** Vitest route/nav/table with mockIPC; smoke reaches Search; gate.

**Provisional commit:** `feat(search): add Search route with virtualized results table`

#### Commit 3 — Sortable result columns

**Status:** Completed — `1b42133`

**Work:** Extend list IPC with sort field + direction (whitelist basic columns). Table headers set sort via URL search params; default remains CA desc. `aria-sort` on headers.

**Out of scope for this commit:**
- Sorting by dynamic/filter-only columns (lands with those columns in PR 2).

**Validation:** Rust tests for sort whitelist; Vitest header → query params; gate.

**Provisional commit:** `feat(search): add sortable columns defaulting to CA`

### PR 2 — Filters, dynamic columns, and global search

**Status:** Active

**Provisional PR title:** `feat(search): add player filters and global name search`

**Purpose:** Full filter surface (scalars through role scores), filter UI, dynamic columns, and top-bar Ctrl+K suggest — one mergeable PR after the shell is on trunk.

**Merge to trunk when:** Operator filters with AND/OR work end-to-end; dynamic columns follow filters; global name search works from any route; DESIGN Search sections reconciled.

**Depends on:** PR 1 merged.

#### Commit 1 — Filter AST and scalar SQL builder

**Status:** Completed — `183de74`

**Work:** Filter rule DTO + AND/OR; field registry for scalars/bools/enums; validate operators per field; compile to parameterized WHERE; integrate into `search_players`. Reject unknown fields/ops. Rust tests for each operator class and AND vs OR.

**Out of scope for this commit:**
- Attributes, positions, nationalities, role scores, React filter UI.

**Validation:** Focused Rust tests; gate.

**Provisional commit:** `feat(search): add filter AST and scalar query builder`

#### Commit 2 — Compact filter bar and editor modal

**Status:** Completed — `45293c3`

**Work:** Shared Modal primitive per DESIGN. Compact strip: tags, clear all, open editor. Modal: add/remove rules, field/operator/value, AND/OR toggle. Wire to search query. Immediate apply.

**Out of scope for this commit:**
- Deep field types not in commit 1 registry; URL persistence (next commit).

**Validation:** Vitest for strip/modal with mockIPC; gate.

**Provisional commit:** `feat(search): add compact filter bar and editor modal`

#### Commit 3 — Persist filters in URL search params

**Status:** Completed — `9ba8886`

**Work:** Encode/decode filter rules, combine mode, and sort into TanStack Router validated search params so reload and back/forward restore the view. Cap rule count in Rust and UI.

**Out of scope for this commit:**
- Deep field types; shareable deep-links as a product promise beyond surviving reload.

**Validation:** Vitest encode/decode + navigation; gate.

**Provisional commit:** `feat(search): persist filters and sort in URL search params`

#### Commit 4 — Attribute and multi-value filters

**Status:** Completed — `ab97626`

**Work:** Extend registry/SQL for attribute maps (`json_extract`), nationalities list match, position presence/suitability. Indexes only if profiling shows need. Timed note on large fixture; optional `/spike` if query cost is unacceptable.

**Out of scope for this commit:**
- Role scores; dynamic column UI.

**Validation:** Rust operator tests; performance note in discoveries if measured.

**Provisional commit:** `feat(search): filter attributes nationalities and positions`

#### Commit 5 — Role-score filters and dynamic columns

**Status:** Completed — `92a0049`

**Work:** Filter by catalog role score; return values for dynamic columns; UI shows basic columns plus columns for active non-basic filter fields. Sort on visible dynamic columns when practical. Reconcile DESIGN.md Search filter/nav notes to compact strip + modal + operators.

**Out of scope for this commit:**
- One column per role by default; profile role grid; global search.

**Validation:** Rust join/filter tests; Vitest column visibility; gate.

**Provisional commit:** `feat(search): filter role scores and show dynamic columns`

#### Commit 6 — Ranked name suggest IPC

**Status:** Completed — `b47938c`

**Work:** `suggest_players` (or equivalent): query string → limited rows from current snapshot; order by match tier (exact → prefix → contains) then CA desc; parameterized `LIKE`. Empty query → empty list.

**Out of scope for this commit:**
- Full filter AST in suggest; React top-bar UI; fuzzy/typo tolerance.

**Validation:** Rust ranking tests; gate.

**Provisional commit:** `feat(search): add ranked name suggest IPC`

#### Commit 7 — Top-bar global search UI

**Status:** Active

**Work:** Search field in `AppTopBar`; Ctrl+K focus; debounce 200ms; popover results; Escape clears per DESIGN. Activating a hit navigates to `/search` with a name `is` filter (no profile route yet). Smoke stub.

**Out of scope for this commit:**
- Opening player profiles.

**Validation:** Vitest + smoke; gate.

**Provisional commit:** `feat(search): add top-bar global search with Ctrl+K`

## Active work

**PR:** 2 — Filters, dynamic columns, and global search

**Commit:** Top-bar global search UI

### RED test (active commit)

Top-bar search: Ctrl+K focuses field; debounced `suggest_players` fills popover; selecting a hit navigates to `/search` with a name `is` filter; Escape clears.

**Wrong behaviour caught:** No Ctrl+K focus; results ignore ranking IPC; hit does not set name filter; Escape does not clear.

### Expected outcome

Global name search works from the top bar on any route.

### Explicit exclusions

Opening player profiles.

## Discoveries and replanning

- Planning 2026-07-30: product choices locked (Search label; operator filters; compact strip + modal; filter depth D including role scores; dynamic columns; AND/OR; sortable default CA; row no-op; Ctrl+K global search). DESIGN inspector/slider filter spec deferred/superseded for this feature.
- Replanned 2026-07-30: first condensation over-merged commits (2 PRs / 5 commits). Corrected to **2 PRs / 10 commits** — keep original atomic commit breakpoints; only reduce PR count from 4 to 2.
- 2026-07-30 PR2 Commit 4: deep filters use field ids `attr.*` / `hidden.*` / `personality.*` / `nationality` / `position` / `pos.*`. Position presence is exact key match (`is`/`is_not`); never substring LIKE. Attribute filter on a 2k-player fixture stayed under 500ms with `json_extract` (no extra indexes). Spike/index only if full-snapshot p95 exceeds ~200ms.
- 2026-07-30 PR2 Commit 5: role filters use `role.{catalog_role_id}` with EXISTS on `player_role_scores` (null scores never match). Dynamic columns follow active non-basic filters; IPC returns `dynamicValues`. Sort accepts those field ids while the column is visible. `position` presence does not add a dynamic column. DESIGN Search filters reconciled to compact strip + modal + operators (inspector no longer the Search filter surface).
- 2026-07-30 PR2 Commit 6: `suggest_players` ranks exact → prefix → contains (`COLLATE NOCASE`), then CA desc; blank query empty; `escape_like` on LIKE patterns; default/max limit 10/20.

## Completed work

| PR | Commit | Hash | Notes |
| --- | --- | --- | --- |
| 1 | Paged player list IPC | `889aed7` | `search_players` windowed IPC; CA desc; limit cap; active-save isolation |
| 1 | Search route and virtualized table | `bbec416` | `/search` + nav; TanStack Virtual; invalidate on save/Load Data; smoke stub |
| 2 | Filter AST and scalar SQL builder | `183de74` | Filter AST + scalar/bool/enum registry; parameterized WHERE; search_players filters IPC |
| 2 | Compact filter bar and editor modal | `45293c3` | Modal primitive; compact strip + editor; immediate apply via Query/IPC; filter-aware mock |
| 2 | Persist filters in URL search params | `9ba8886` | Validated search params for filters/combine/sort; UI+Rust rule cap 32 |
| 2 | Attribute and multi-value filters | `ab97626` | `json_extract` attrs/hidden/personality; nationality `json_each`; position presence/suitability |
| 2 | Role-score filters and dynamic columns | `92a0049` | `role.*` EXISTS on `player_role_scores`; `dynamicValues`; UI dynamic columns; DESIGN strip/modal |
| 2 | Ranked name suggest IPC | `b47938c` | `suggest_players`: exact→prefix→contains then CA; blank empty; LIKE escape; limit 10/20 |

## Final validation

At feature end: full `./scripts/dev test`, `./scripts/dev check`, smoke with Search nav + global search stubs; manual Windows Load Data → filter by role/attribute on a real snapshot if available.

## Documentation impact

- Update [ARCHITECTURE.md](../../ARCHITECTURE.md) with `features/search` and list/suggest IPC.
- Update [DESIGN.md](../../DESIGN.md): Search nav item; compact filter strip + modal; operator filter model; global search behaviour; drop inspector-as-primary-filter assumption for Search.
- Archive this ledger to `features/completed/` at `/finish-feature`.
- [TODO.md](../../TODO.md): move Player search to Completed when finished; Plan next → Player profiles.
