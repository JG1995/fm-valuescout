# Player search

## Intent

Give the user a Genie Scout–style **Search** surface over the active save's current snapshot: a virtualized player table, FM-like filter rules (field + operator + value, combined with AND or OR), sortable columns, and a global top-bar name search. Role-score and attribute filters validate the moneyball path — not only identity browse.

## Delivered behavior

- Nav rail **Search** → route `/search`; page title "Search".
- Virtualized results table for the active save's current snapshot (TanStack Virtual, 40px two-line rows).
- **Basic columns (always):** Name, Age/DOB, Nationality, Club, Division, CA, PA, Market value.
- **Dynamic columns:** any non-basic field in an active filter also appears as a column (attributes, role scores, and other non-basic fields). Removing the filter removes the column. `position` presence does not add a dynamic column.
- **Compact filter strip** above the table: removable tags, AND|OR when multiple rules, **Clear all**, **Edit filters** opens a modal editor. Filters apply immediately (no Apply button).
- Each rule: property + operator + value. Strings — contains / does not contain / is / is not. Integers (and similar) — greater than / less than / equals / does not equal. Booleans and enums — is / is not.
- Flat AND|OR only — no nested filter groups.
- Filters, combine mode, sort field, and sort direction persist in TanStack Router validated URL search params (reload and back/forward restore the view). Rule count capped at 32 (Rust trust boundary and UI).
- Table headers sort; **default sort is CA descending** (`aria-sort` on headers). Dynamic filter columns are sortable while visible.
- Row click is a **no-op** until player profiles exist.
- Top bar: **GlobalPlayerSearch** pill; **Ctrl+K** / **Meta+K** focus; 200ms debounced `suggest_players`; combobox + listbox; Escape clears before dismissing; hit navigates to `/search` with a `name` `is` filter.
- Empty states: no snapshot → points at Load Data; no matches with filters → "No players match these filters" (strip above for clear); empty snapshot → retry Load Data.
- `null` / unknown values never coerce to 0; missing cells render as `—`.
- Truncated-scan warning appears on the top-bar **SnapshotFreshnessChip** (and dashboard snapshot panels) but **not** yet on the Search results panel or in the results count line — see Follow-up.

## Final architecture

```text
React features/search
  → api/ — searchPlayersQueryOptions, suggestPlayersQueryOptions; SEARCH_PAGE_SIZE 50
  → components/ — SearchResultsPanel (virtual table + windowed page cache), SearchFilterBar,
                  SearchFilterStrip, SearchFilterEditorModal, GlobalPlayerSearch, FilterTag
  → types/ — filter rules, sort fields, PlayerSummary, PlayerSuggestHit (mirror IPC DTOs)
  → utils/ — search-url-search (encode/decode URL params), filter-registry, dynamic-columns,
             role-catalog labels

app/routes/search.tsx — thin composition; loader prefetches current snapshot + first results page
AppNavRail — Dashboard + Search
AppTopBar — GlobalPlayerSearch (all routes)

Rust features/search
  → commands.rs — search_players, suggest_players
  → filter.rs — FilterAst (flat AND|OR), field registry, operator validation, compile to parameterized SQL
  → query.rs — windowed search (offset/limit default 50, max 200), sort whitelist, dynamicValues for
               active non-basic filters; suggest ranking exact → prefix → contains (NOCASE), then CA desc

Shared UI
  → Modal primitive (src/components/ui/modal/) — first use: filter editor
  → @tanstack/react-virtual — results table virtualization

SQLite (read-only from search)
  → players + player_role_scores for current snapshot of active save
  → json_extract for attribute maps; json_each for nationalities; EXISTS on player_role_scores for role.* filters
```

**Field ids (deep filters):** `attr.*`, `hidden.*`, `personality.*`, `nationality`, `position`, `pos.*` (suitability), `role.{catalog_role_id}`.

## Important decisions

- Nav label **Search** (not Database).
- Operator filters with compact strip + modal — DESIGN inspector/slider filter panel is not the Search surface.
- Role scores and attributes in scope for MVP search; dynamic columns follow active non-basic filters.
- Windowed offset/limit pagination (50 default, 200 max) with virtualizer fetching additional pages on scroll — no full player set in the WebView.
- `json_extract` attribute filters acceptable at hobbyist scale; 2k-player fixture stayed under 500ms without extra indexes (spike/index only if full-snapshot p95 exceeds ~200ms).
- `suggest_players`: blank query → empty list; default/max limit 10/20; `escape_like` on LIKE patterns.

## Migration and operational implications

- No new migrations — search reads existing `players` and `player_role_scores` from the current snapshot.
- Query and filter evaluation run in Rust against SQLite; the WebView never opens the DB.
- Role scores are read from `player_role_scores` (ingest-time); the WebView does not recompute scores.
- Invalidating snapshot/save Query keys after Load Data or save switch refreshes search results.

## Validation

- `./scripts/dev test`, `./scripts/dev check`, Playwright smoke (Search nav, `search_players`, `suggest_players` stubs).
- Vitest: route, nav, virtual table, filter strip/modal, URL encode/decode, global search debounce and navigation.
- `cargo test`: filter operators (scalar, bool, enum, JSON attrs, nationalities, positions, role scores), sort whitelist, suggest ranking, page limits, active-save isolation.
- Feature-complete review: Blocking **No** (reviewer `7882613e-8d73-4762-976b-ab4f42a9fac6`).

**Delivery commits (final hashes):** `889aed7`, `bbec416`, `1b42133`, `183de74`, `45293c3`, `9ba8886`, `ab97626`, `92a0049`, `b47938c`, `cc714a1` (comparison base `e47e74f`; PR1 squash `5d00c49` on main + PR2 through `0a99e6f` / content `cc714a1`).

## Follow-up

- **Next feature:** [Player profiles](../../TODO.md) (order 5) — row activation and detail view from search.
- **Review MEDIUM:** Search results omit truncated-scan warning banner and capped-count annotation on the results panel; top-bar `SnapshotFreshnessChip` still warns. Add per DESIGN.md truncated banner on Search when polishing.
- **Review NITPICK:** No-results empty state copy points at the filter strip but has no inline **Clear filters** action.
- **Deferred (unchanged):** nested filter groups, saved presets, export, cross-save search, profile navigation, combined IP+OOP weight UI (squad planner).
