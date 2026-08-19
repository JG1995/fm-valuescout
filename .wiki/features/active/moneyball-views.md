# Optional Moneyball Analysis Views

## Status

Active

## Intent

Turn the already imported Moneyball CSV statistics into an optional analysis workspace without crowding the existing General Search and Player Profile experiences. The feature gives the current snapshot a complete, replaceable Moneyball cohort; calculates comparable 0–100 percentile scores from the matched import; exposes every canonical performance metric as a Moneyball Search column and raw-value filter; and presents the same raw values and full-import percentile scores on player profiles.

The feature extends Linear JAY-19 with the agreed Search workspace and app-wide default-view preference. Composite position or role scoring remains owned by JAY-20.

## User-visible behavior

- Player Search has General and Moneyball views. General preserves its current behavior. Moneyball is an explicit opt-in surface unless the user selects it as the default in Settings.
- Moneyball Search shows only players who both belong to the active save's effective current snapshot and matched the active Moneyball CSV import for that snapshot.
- Moneyball Search uses the shared full-height virtualized table. It has no Previous, Next, page-number, or page-size controls; bounded backend windows remain an internal implementation detail.
- Every one of the 138 canonical Moneyball performance metrics is available as a Moneyball Search column, raw-value sort field, and raw-value filter.
- Moneyball Search also offers the current memory-backed identity, club, position, and value fields needed to narrow a recruitment cohort, plus raw starts, substitute appearances, and minutes from the Moneyball import.
- Moneyball Search defaults to Name, Age / DOB, Nationality, Club, Division, Minutes, Average Rating, Goals / 90, Assists / 90, xG / 90, and xA / 90. Its table layout persists separately from General Search.
- A comparison-pool control offers Filtered results and Full CSV. Filtered results is the default.
- Filtered results recalculates percentiles across every player in the complete filtered cohort before internal table-window fetching. It never calculates against only the rows currently loaded or visible in the WebView.
- Full CSV uses percentiles calculated across every current-snapshot player that matched and was stored from the active upload. Parsed CSV rows whose UIDs are absent from the current snapshot are skipped and cannot affect any percentile.
- Each performance cell keeps the raw value visible and shows an integer 0–100 percentile score whose color uses the existing four-tier score ramp. Sorting and filtering always use the raw metric, not the percentile.
- The Player Profile has General and Moneyball views. General preserves the current attribute and role-fit workspace. Moneyball shows the player's imported playing-time context and all 138 metrics in eight compact, attribute-like categories.
- Player Profile Moneyball scores always use the matched full-CSV cohort. It does not offer the filtered-results comparison pool.
- A player without an active current-snapshot Moneyball row receives a clear, non-error state. A pre-feature row without a trusted percentile cohort asks the user to re-import rather than displaying potentially mixed or stale analysis.
- Moneyball CSV upload and replacement move from My Club / Squad into Moneyball Search. Youth Academy CSV upload remains in My Club / Squad.
- A valid successful Moneyball upload atomically replaces the complete Moneyball cohort for the active snapshot. Players omitted from the new file stop appearing after the import.
- Moneyball data owned by older snapshots remains stored and is ignored by these current-only views. It becomes displayable only through a future timeline feature.
- Settings offers one app-wide Default player analysis view preference: General or Moneyball. It controls the default for both Search and Player Profile.
- An explicit General or Moneyball route value wins over the persisted default. General Search rows open General profiles, Moneyball Search rows open Moneyball profiles, and global/direct profile navigation uses the default when no explicit view is present.

## Invariants

- FM memory data remains authoritative for player identity, current club, position, ability, contract, value, and other memory-owned fields. CSV import never creates a player or replaces those fields.
- Moneyball matching uses exact numeric player UID equality against the captured effective current snapshot.
- Moneyball rows remain owned by snapshot and player. A new snapshot never inherits, copies, or silently exposes the prior snapshot's Moneyball values.
- A successful Moneyball import is a complete replacement for the captured current snapshot's active Moneyball cohort. Youth Tracker import retains its existing per-player upsert behavior.
- Replacement touches only player_moneyball_stats rows for the captured current snapshot. Rows for every other snapshot and the v17 legacy quarantine remain unchanged.
- Parse, format, conversion, database, or stale-context failure leaves the prior committed Moneyball cohort unchanged.
- A syntactically valid Moneyball import with zero matching current-snapshot UIDs is still an authoritative empty replacement: it stores zero players, reports every parsed row as skipped, and clears the current snapshot's active cohort.
- The bounded import contract remains 1 MiB and at most 1,000 player rows.
- The canonical performance contract remains exactly 138 keys. The current canonicalizer maps 66 supplied CSV values and calculates 72 additional values without changing source-value precedence.
- Full-CSV percentile populations contain only rows that matched the captured current snapshot and were stored by the same successful replacement import.
- Null values remain unavailable, are omitted from that metric's percentile population, and never become zero.
- A non-null metric with a one-player population or a population whose values do not vary receives a neutral score of 50.
- Otherwise percentile rank uses the pinned lower-bound rule: number of non-null population values strictly below the player's value, divided by population size minus one, multiplied by 100.
- Tied raw values receive the same percentile. The final displayed and persisted score is rounded to the nearest integer and clamped to 0–100.
- The 19 pinned lower-is-better metrics invert the percentile after ranking: minutes per goal, minutes per assist, minutes per goal or assist, goals conceded and per 90, fouls made and per 90, yellow cards and per 90, red cards and per 90, offsides and per 90, mistakes leading to goal and per 90, possession lost and per 90, and headers lost and per 90.
- Percentile color describes the relative score, while the raw statistic remains the factual value. Context fields such as appearances and minutes are not percentile-colored.
- Moneyball Search filters and sorts compare raw numeric values. Percentile filters and percentile sorting are not introduced.
- Filtered-result percentiles use the cohort after all applied raw filters and AND / OR combination rules, but before LIMIT and OFFSET.
- The WebView receives only the current bounded page plus its requested display values and scores. It never receives all 1,000 rows merely to calculate percentiles.
- General Search remains backward-compatible: existing URLs, default sort, field catalog, query behavior, table layout, and player count are unchanged when the Moneyball view is not active.
- General Player Profile remains backward-compatible: its current tab URL, attribute grouping, role-fit behavior, concealment, and development actions remain unchanged.
- The outer route view and Moneyball comparison pool use validated closed values. Unknown values normalize safely rather than reaching Rust as arbitrary modes.
- The persisted default view is presentation state, applies across app saves, and never changes SQLite data or snapshot selection.
- Old pre-feature Moneyball rows are preserved through migration but are not eligible for current Moneyball UI until a new import records a complete percentile cohort.
- Historical snapshot selection, import-wave history within one snapshot, and automatic carry-forward remain out of scope.

## Non-goals

- Historical Moneyball browsing, snapshot selection, season selection, trend lines, development graphs, or timeline comparisons.
- Preserving multiple Moneyball upload waves inside one snapshot. The latest successful upload is authoritative for that snapshot.
- Composite Moneyball quality, value, position, archetype, or role-fit scores. Those remain JAY-20.
- Position-specific percentile pools, league-strength adjustment, minimum-minutes eligibility, user-defined metric weights, or custom metrics.
- Percentile-based filters, percentile-based sort operands, or league / position comparison toggles.
- Displaying CSV-only players who are absent from the active snapshot.
- Changing the 138-key canonical statistic calculation or accepting new CSV dialects.
- Moving Youth Academy upload away from My Club / Squad.
- Adding visible table pagination or collecting the full Search result set in the WebView.
- A new navigation-rail destination. Moneyball stays inside Player Search, Player Profile, and Settings.
- A database-backed or save-scoped UI preference.
- A redesign of General Search, General Player Profile, My Club, or the shared score ramp.

## Current-state map

- Relevant components:
  - src/app/routes/search.tsx owns the General Search route, validated filter and sort URL state, initial query prefetch, and the shared virtualized Search results composition.
  - src/features/search/ owns Search IPC, query keys, URL parsing, filters, the filter registry, and SearchResultsPanel.
  - src/components/player-table/virtualized-player-table.tsx owns one full-height scroll region and requests bounded 50-row IPC pages as the virtual range moves.
  - src/components/player-table/player-table-header.tsx and src/stores/use-player-table-store.ts own the metric picker, column actions, widths, order, and per-table persisted layouts.
  - src/utils/player-metrics.ts is the current General player metric presentation catalog.
  - src/app/routes/players.$uid.tsx composes the current profile summary, four attribute tabs, role fit, concealment, and development actions.
  - src/features/player-profile/ owns General profile IPC, types, components, and tab helpers.
  - src/features/csv-import/components/squad-csv-import-actions.tsx currently exposes both Moneyball and Youth actions in My Club / Squad.
  - src/features/csv-import/components/squad-csv-import-modal.tsx and src/features/csv-import/utils/use-csv-import.ts own the context-bound picker, native drop, import result, and Youth invalidation.
  - src/app/routes/settings.tsx currently contains Save data and Bridge sections.
- Data model:
  - Schema v29 stores player_moneyball_stats by snapshot_id and player_uid with asking-price context, starts, substitute appearances, minutes, one exact 138-key statistics_json object, and import time.
  - The v17 player_moneyball_stats_legacy quarantine remains save-owned, unread, and intentionally detached from current snapshots.
  - Current rows contain no percentile object or import-cohort marker.
- Persistence and migrations:
  - src-tauri/src/db/migrations.rs owns ordered PRAGMA user_version migrations and upgrade tests.
  - src-tauri/src/features/csv_import/service.rs captures save and snapshot tokens plus the current UID set, parses outside the database lock, revalidates inside a transaction, and currently upserts only included matching Moneyball players.
  - Current re-import behavior leaves omitted current-snapshot players in player_moneyball_stats, so it cannot define an exact active comparison cohort.
- Existing behavioral assumptions:
  - Moneyball parsing is bounded, null-preserving, and exact-UID-only.
  - src-tauri/src/features/csv_import/statistics.rs already produces the exact 138 canonical exported-or-derived statistics and pins their spelling through moneyball_canonical_statistics.txt.
  - Normal product reads select only the active save's effective current snapshot.
  - General Search filters, sorts, counts, and pages in Rust / SQLite; the WebView does not open the database.
  - Potential-role Search already proves the pattern of cohort-wide backend work before bounded page reads.
  - The legacy project at commit 366aa20b5282d3a63c94854ddb8da6992462b0c5 precomputes all-import percentiles in [import.rs](https://github.com/JG1995/fm-valuescout-react/blob/366aa20b5282d3a63c94854ddb8da6992462b0c5/src-tauri/src/commands/import.rs), recomputes selected cohorts in [scores.rs](https://github.com/JG1995/fm-valuescout-react/blob/366aa20b5282d3a63c94854ddb8da6992462b0c5/src-tauri/src/commands/scores.rs), and defines the inversion and neutral-pool behavior in [percentile.rs](https://github.com/JG1995/fm-valuescout-react/blob/366aa20b5282d3a63c94854ddb8da6992462b0c5/src-tauri/src/analysis/percentile.rs). Its lower-bound rank lives in [arrow.rs](https://github.com/JG1995/fm-valuescout-react/blob/366aa20b5282d3a63c94854ddb8da6992462b0c5/src-tauri/src/data/arrow.rs). Its null-to-zero behavior is explicitly not transported.
- Architectural seams:
  - A new Rust features/moneyball module can own the closed metric contract, percentile engine, and current-snapshot Moneyball read models. CSV import calls it; Search calls it; React never recalculates scores.
  - The existing Search command can accept a closed view and comparison-pool mode while defaulting absent values to General behavior.
  - The existing DynamicValue contract needs a real-number variant so SQLite numeric JSON values remain numeric through IPC.
  - A Moneyball-specific frontend catalog can own labels, category, raw precision, units, and default columns without adding 138 fields to General Search.
  - TanStack Router search params remain the source of truth for explicit view, filters, sort, and comparison pool. Zustand remains the source of truth for table layout and the app-local default view.
- Project validation commands:
  - ./scripts/dev test [target...]
  - ./scripts/dev format [paths...]
  - ./scripts/dev check
  - ./scripts/dev smoke
- Primary risks:
  - A replacement import could delete the prior current-snapshot cohort before a later conversion or insert failure.
  - A skipped CSV row could incorrectly influence full-import percentiles.
  - Filtered-result scores could accidentally use only one 50-row page.
  - Raw integer and decimal values could lose type or precision across SQLite JSON, Rust, IPC, and formatting.
  - The independently maintained Rust validation and frontend presentation catalogs could drift.
  - Route defaults could override explicit links or leak Moneyball-only state into General Search.
  - Adding 138 metrics could regress table scroll containment, keyboard access, or column-store sanitation.

## Feature architecture

### Metric and percentile contract

Rust owns the canonical statistic key set, numeric extraction, lower-is-better direction, percentile calculation, and score serialization. The percentile engine accepts nullable raw values keyed by player UID and returns nullable integer scores keyed by the same canonical metric IDs. It is a pure bounded calculation used both during import and during filtered Search reads.

The frontend owns presentation metadata only: Moneyball-prefixed field ID, user-facing label, one of eight categories, raw kind, precision, unit, alignment, default width, and filter operators. The eight categories are Attacking & Finishing, Creativity & Chance Creation, Transition & Ball Progression, Defensive Actions, Aerial Presence, Goalkeeper & Shot Stopping, Discipline & Error Margins, and Match Impact.

Category ownership follows the canonical key order already pinned in moneyball_canonical_statistics.txt: Attacking & Finishing runs from goals through minutes_per_goal_or_assist; Creativity & Chance Creation runs from assists through minutes_per_assist; Transition & Ball Progression runs from passes_attempted through possession_lost_per_90; Defensive Actions runs from tackles_attempted through clearances_per_90; Aerial Presence runs from headers_attempted through key_headers_per_90; Goalkeeper & Shot Stopping runs from clean_sheets through penalties_saved_ratio; Discipline & Error Margins runs from fouls_made through mistakes_leading_to_goal_per_90; and Match Impact runs from average_rating through team_goals_per_90. These boundaries cover each canonical key exactly once.

Formatting is explicit per metric in the presentation catalog rather than guessed from arbitrary field text. Canonical count values render as whole numbers; per-90 and expected-value decimals use two decimal places; ratios and expected save percentage carry a percentage unit; average rating uses two decimal places; distance fields carry their distance unit; and minutes-per-event fields keep one decimal place. Zero remains a visible value and null remains an em dash.

The Rust catalog remains the trust-boundary authority. Frontend and Rust tests independently pin all 138 canonical IDs and reject duplicates; Rust tests also pin the complete lower-is-better list. This follows the existing player-metrics pattern rather than making Rust runtime behavior depend on a frontend source file.

### Import and persistence

Migration v30 adds nullable percentiles_json to player_moneyball_stats with an object-or-null JSON check. Existing rows remain unchanged with a null percentile object. Null is an explicit migration marker: those rows stay stored for future timeline work but are not presented as a trusted active analysis cohort.

The import path keeps parsing and percentile preparation outside the database lock. After capture, it filters parsed Moneyball players against the captured current-snapshot UID set, calculates percentiles from only those matched rows, and prepares one complete statistics object plus one complete percentile object per matched player.

Inside one write transaction, import revalidates the immutable save and snapshot context, deletes every player_moneyball_stats row for the captured current snapshot, inserts the prepared matched cohort, and commits. Any delete, insert, or commit failure rolls back to the prior cohort. Rows owned by older snapshots and the v17 quarantine are untouched.

### Player Profile

Rust exposes a separate current-snapshot Moneyball detail command rather than inflating get_player for users who remain in General view. The command accepts only the player UID, resolves the active save's effective current snapshot, joins the current player, and returns one of: no active row, legacy row requiring re-import, or a complete context plus raw-statistics and percentile map.

The route adds a validated outer view value while retaining the current inner attribute tab value. A shared compact identity shell remains visible, while General-only role summaries, concealment controls, development actions, attributes, and role fit stay in General. Moneyball uses a full-width internally contained panel with playing-time context and eight keyboard-accessible metric category tabs.

### Search

The existing search_players command gains optional search_view and comparison_pool inputs. Omitted inputs preserve General behavior. Moneyball mode inner-joins the active current snapshot to scored player_moneyball_stats rows, accepts only a bounded Moneyball Search field catalog, and returns only players in the current active import cohort.

Moneyball field IDs use moneyball.<canonical-key> plus the raw context IDs moneyball.starts, moneyball.substitute_appearances, and moneyball.minutes. Rust resolves every ID through a closed catalog and constructs only trusted SQLite expressions. Raw performance values are cast to real numbers for a consistent IPC value type; frontend presentation metadata decides whether a value renders as an integer, decimal, percentage, distance, or rating.

The non-Moneyball fields available in this view are exactly name, age, nationality, club, division, parent_club, preferred_foot, position, and value. General attributes, hidden values, personality fields, CA / PA, position-suitability scores, and role scores remain in General Search. This keeps the Moneyball picker below the existing requested-field bound and preserves the opt-in separation.

Full CSV reads the persisted percentile object for requested performance fields. Filtered results first builds the complete filtered UID and requested-value cohort in Rust, runs the shared percentile engine, and then attaches only the current page's scores to the bounded page DTO. Count, filter, sort, percentile population, and page selection all use the same compiled predicate.

The route retains virtual scrolling and 50-row page queries. Moneyball has Average Rating descending as its default raw sort, separate URL defaults, separate table layout storage, a comparison-pool URL value, and its own metric picker. Switching outer views resets incompatible filter and sort state; browser history preserves the prior URL. The upload action and replacement result live above the Moneyball table, and an empty cohort renders the upload prompt.

### Preferences and navigation

A small persisted Zustand store owns defaultAnalysisView as general or moneyball. It is app-local and independent of saves. Search and Profile calculate an effective view from explicit validated URL value first and the persisted default second.

General Search rows navigate with an explicit General profile view. Moneyball Search rows navigate with an explicit Moneyball profile view. Global search and direct profile URLs omit the outer view and therefore follow the persisted default.

## Uncertainty register

### Known

- The current branch starts from main commit 8a6b93c3c08177f268919df7dd743862fa64542c with no competing active ledger.
- Schema v29 already gives Moneyball rows the required snapshot and player ownership.
- The canonical Moneyball statistics contract is exactly 138 keys: 66 values are supplied by the CSV mapping and 72 are derived by the existing canonicalizer.
- Moneyball imports are bounded to 1,000 rows, so complete filtered-cohort calculation is bounded.
- General Search already has full-height virtualization, fixed rows, bounded 50-row backend windows, and no visible pagination controls.
- Existing score tokens and ScoreBadge cover 0–100 values and the required four color tiers.
- The pinned legacy repository provides the lower-bound percentile rule, neutral-pool behavior, all-import precomputation, filtered-cohort recomputation, and the 19 inverted metrics.
- JAY-20 owns composite Moneyball role scoring and is not a dependency for JAY-19.

### Assumptions

- The eight named categories provide sufficient first-version organization for all 138 metrics; label wording can be adjusted without changing the persisted or IPC contract.
- Starts, substitute appearances, and minutes are useful Search context and raw filters but are not percentile-scored performance metrics.
- Asking price remains profile context in this feature. Its single, range, not-for-sale, and missing forms do not define one honest raw filter operand, while General Search already exposes memory-backed market value.
- A valid zero-match file is an intentional empty replacement rather than a failed import because the new file is authoritative and stale players must disappear.
- The bounded 1,000-row cohort can be scored in Rust without a new cache, worker, dependency, or database index.
- One app-wide default is preferable to separate Search and Profile defaults because the developer explicitly wants both surfaces to move together.

### Decisions

- Deliver the feature in one PR. Persistence, profile, Search, upload relocation, and preference share one user contract, and a separate foundation PR would expose no independently useful behavior.
- Use five atomic commits inside the sole PR.
- Add nullable percentiles_json in migration v30. Do not backfill pre-feature rows from a potentially mixed cohort and do not delete them.
- Hide pre-feature rows from current analysis until re-import records a complete percentile object.
- Treat each successful Moneyball import as a full current-snapshot cohort replacement; retain other snapshots unchanged.
- Calculate full-import percentiles after exact UID matching, not across every parsed CSV row.
- Calculate filtered-result percentiles in Rust across the complete filtered cohort before LIMIT and OFFSET.
- Use the legacy lower-bound rank and exact inversion list, but preserve nulls instead of coercing them to zero.
- Round persisted and returned percentile scores to integer 0–100 values.
- Use a Moneyball-specific frontend metric catalog and table layout so General Search does not gain 138 fields.
- Reuse the existing search_players and VirtualizedPlayerTable boundaries rather than introducing a second table engine or a client-side collection.
- Use explicit route values as overrides and a persisted Zustand preference only as the fallback.
- Move only Moneyball upload to Search; Youth upload stays in My Club / Squad.
- Do not create an ADR. The plan extends established snapshot ownership, SQLite migration, Rust query, TanStack Router, and Zustand presentation-state decisions without introducing a new durable architectural style.

### Unknowns

- The exact visual density of eight profile metric tabs and raw-plus-score cells needs native desktop inspection at 1280×800 and 1600×900.
- Representative 1,000-row filtered-percentile latency is not yet measured. The implementation must retain the bounded server-side design and record manual timing evidence without inventing a brittle CI timing assertion.
- Native Tauri picker, WebView drop, SQLite upgrade, and restart behavior cannot be proven by browser IPC stubs alone.

None of these unknowns blocks Commit 1. If the bounded Rust path cannot meet interactive Search behavior without new persistence or a background computation architecture, stop and request an explicit workflow spike rather than adding an unplanned cache.

### Risks

- Atomic replacement can become destructive if delete and insert do not share one revalidated transaction.
- Using parsed rather than matched rows as the percentile population would violate snapshot authority and change every score.
- Treating null as zero would materially mis-rank sparse metrics, especially goalkeeper fields.
- A page-local implementation could show different percentiles while scrolling.
- A catalog typo involving hyphenated keys such as np-xg or xg-op could break route validation or SQL JSON extraction.
- Persisted table layouts may contain fields that become invalid when view-specific sanitation changes.
- Default-view hydration could cause the route loader to prefetch or briefly render the wrong surface.
- Moving the upload action could leave duplicate owners or stale invalidation behavior.

## Walking skeleton

Commits 1 and 2 form the walking skeleton. Commit 1 turns one matched current-snapshot Moneyball upload into an atomic, scored cohort. Commit 2 reads one stored player through a current-snapshot Rust command and renders raw values plus full-import scores in an explicit Moneyball Player Profile view. Together they prove CSV → exact UID match → snapshot persistence → percentile calculation → IPC → optional UI before the broader Search workspace is added.

## Delivery plan

### PR 1 — Optional Moneyball analysis views

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** feat(moneyball): add optional analysis views

**Branch:** feature/moneyball-views

**Base branch:** main

**Publication provider:** GitHub

**PR template:** .github/pull_request_template.md

**Merge method:** Squash

**Required-check rule:** Strict required check named check

**Feature close-out:** Not run

**CI repair rounds:** 0

**Purpose:** Deliver one coherent current-snapshot Moneyball workflow from replacement import and percentile persistence through Player Profile, virtualized Search, upload ownership, and default-view preference. The shared score and route contracts make one PR the clearest review and merge surface.

**Merge to trunk when:** All five commits are completed and individually reviewed, the final feature validation and feature-complete review clear, documentation is reconciled, the GitHub PR template is complete, and the required check passes.

**Depends on:** Main commit 8a6b93c3c08177f268919df7dd743862fa64542c; the completed CSV enrichment, snapshot history, player profile, configurable player tables, and My Club workspace features; Linear JAY-19 product intent. JAY-20 is explicitly not a dependency.

#### Commit 1 — Persist Moneyball percentile cohorts

**Status:** Active

**Provisional commit:** feat(import): persist Moneyball percentile cohorts

**Work:** Make one successful Moneyball upload define an exact, scored, current-snapshot cohort while preserving every other snapshot and all prior data on failure.

**Out of scope:**

- Player Profile or Search reads.
- Frontend metric labels, formatting, or color.
- Moving the upload action.
- Filtered-result percentile calculation.
- Changes to Youth Tracker persistence.

**Implementation packet:**

- Add a Rust Moneyball domain module that owns the closed 138-key metric contract, the 19 lower-is-better directions, nullable numeric extraction, and the pure integer percentile engine.
- Add migration v30 with nullable percentiles_json on player_moneyball_stats. Existing rows must survive with null scores and no inferred cohort.
- Refactor Moneyball preparation so exact current-snapshot UID filtering and percentile calculation happen outside the SQLite mutex after context capture.
- Change only Moneyball persistence from per-included-player upsert to a complete current-snapshot delete-and-insert replacement inside the existing revalidated transaction.
- Store an exact 138-key percentile JSON object beside the exact 138-key raw statistics object. Each value is an integer 0–100 or null.
- Preserve the existing safe import summary and all Youth behavior.

**Files and responsibilities:**

- src-tauri/src/features/moneyball/mod.rs — expose the new domain module.
- src-tauri/src/features/moneyball/catalog.rs — own canonical keys and lower-is-better metadata used at Rust trust boundaries.
- src-tauri/src/features/moneyball/percentile.rs — own the pure null-preserving percentile calculation and integer score result.
- src-tauri/src/features/mod.rs — register the Moneyball feature module.
- src-tauri/src/features/csv_import/statistics.rs — consume or cross-check the Moneyball catalog without changing calculation semantics.
- src-tauri/src/features/csv_import/service.rs — prepare the matched cohort, serialize percentile JSON, and replace current-snapshot rows atomically.
- src-tauri/src/features/csv_import/commands.rs — preserve capture / parse / prepare / re-lock sequencing and ensure percentile work occurs outside the database lock.
- src-tauri/src/db/migrations.rs — add migration v30 plus fresh-install and v29-upgrade coverage.
- src-tauri/src/features/csv_import/fixtures/moneyball_canonical_statistics.txt — remain the independent exact-key fixture; change only if repository evidence disproves the existing contract.
- .wiki/ARCHITECTURE.md — document migration v30, exact-cohort replacement, nullable legacy rows, and import-time percentiles only after implementation makes them true.

**Behavior and data flow:**

- import_csv captures active save ID/token, effective current snapshot ID/token, and the snapshot UID set under the existing brief lock.
- CSV parsing and canonical 138-key calculation continue outside the lock.
- Moneyball preparation drops unknown UIDs, calculates total/stored/skipped counts, builds each metric's non-null matched-player population, ranks matched values, applies inversion, rounds to integer, and prepares complete raw and score JSON objects.
- Persistence reopens the database lock, starts one transaction, revalidates the same save and snapshot tokens, deletes only rows whose snapshot_id is the captured current snapshot, inserts every prepared matched row, and commits.
- A valid zero-match import deletes the captured current snapshot's rows and commits an empty active cohort.
- Any preparation error occurs before deletion. Any revalidation, delete, insert, or commit error rolls the transaction back and preserves the prior current-snapshot cohort.
- Older snapshot rows and player_moneyball_stats_legacy are never selected by the delete.
- Existing v29 rows receive percentiles_json null during migration and remain byte-for-byte equivalent in their existing columns.

**Ordered implementation steps:**

1. Add RED Rust tests for null exclusion, tie behavior, one-player and non-varying pools, inversion, matched-only populations, atomic same-snapshot replacement, zero-match replacement, rollback, and v29 migration preservation.
2. Add the smallest pure Moneyball catalog and percentile engine that turns the calculation tests GREEN.
3. Add migration v30 and make fresh and upgraded database tests GREEN without backfilling or deleting old rows.
4. Reshape Moneyball preparation to filter against captured UIDs and calculate scores before the write lock; keep Youth preparation unchanged.
5. Replace current-snapshot Moneyball persistence inside the existing transaction and make replacement, isolation, stale-context, and rollback proofs GREEN.
6. Refactor only while the focused Rust proofs stay green, then update current-state architecture.
7. Run format and the full commit gate.

**Tests and proof:**

- Percentile unit tests: empty/null-only population returns null; one non-null value returns 50; identical non-null values return 50; lower-bound ranks produce 0, 50, and 100 for a three-value population; ties share rank; an inverted metric reverses score; null players remain null.
- Catalog tests: exactly 138 unique keys; exact fixture equality; exact 19-key inversion list; hyphenated keys remain valid.
- Import tests: unknown UIDs increase skipped count but do not affect stored-player percentiles; omitted previously stored players disappear after a successful same-snapshot re-import; rows on another snapshot remain; a zero-match valid import clears only the captured snapshot; invalid/stale imports preserve the prior cohort.
- Transaction proof: force an insert or conversion failure after a seeded prior cohort and assert the prior rows remain.
- Migration tests: v29 fixture upgrades to v30; existing raw/context/import fields remain; percentiles_json is null; new rows enforce object-or-null JSON and score object values are produced by the service.
- Existing canonical statistic and Youth import tests remain green.
- Expected RED failures are missing v30, absent percentile engine, and current upsert behavior retaining an omitted player.

**Patterns to verify:**

- src-tauri/src/features/csv_import/service.rs capture, prepare, revalidation, and transaction boundaries.
- src-tauri/src/features/csv_import/staff_shortlist.rs complete replacement and rollback behavior, while deliberately retaining Moneyball's valid zero-match decision.
- src-tauri/src/features/csv_import/statistics.rs exact-key fixture test and null-preserving derived calculations.
- src-tauri/src/db/migrations.rs additive column migrations and version-upgrade tests.
- Legacy percentile.rs and data/arrow.rs at pinned commit 366aa20b5282d3a63c94854ddb8da6992462b0c5 for rank and inversion behavior, with an explicit divergence for nulls.

**Constraints and non-goals:**

- No new dependency, background job, cache table, season field, import batch table, or historical read.
- Do not alter the CSV dialect, file size, row limit, duplicate handling, UID type, source precedence, or 138 raw calculations.
- Do not hold the database mutex while parsing or calculating percentiles.
- Do not infer percentiles for pre-feature rows.
- Do not weaken context-token checks or atomic failure behavior.
- Do not change Youth Tracker replacement semantics.

**Dependencies and sequencing:**

- Requires only current main schema v29 and existing CSV import behavior.
- Must complete before Profile or Search can treat a Moneyball row as an active scored cohort.
- Later commits may extend the Moneyball module but must not change this import contract silently.

**Validation:** Run ./scripts/dev format, then ./scripts/dev check. The gate must include the new Rust unit, migration, import, and regression tests. Run git diff --check before checkpoint.

**Stop conditions:** Stop and replan if SQLite cannot add the nullable checked JSON column without rebuilding or losing existing rows; exact matching cannot happen before the write lock; transaction rollback cannot preserve the prior cohort; the existing 138-key fixture is disproved; percentile calculation would require parsed unknown UIDs; or implementation requires season/import-wave identity, a new dependency, or a new persistence table beyond percentiles_json.

**Review mandate:** Verify atomic delete-and-insert rollback; snapshot isolation; matched-only percentile populations; null and tie semantics; exact inversion list; zero-match behavior; v29 upgrade preservation; bounded work outside the database lock; and unchanged Youth behavior.

#### Commit 2 — Add the Moneyball Player Profile view

**Status:** Pending

**Provisional commit:** feat(profile): add Moneyball analysis view

**Work:** Prove the scored cohort end to end by rendering one current player's raw metrics and full-import percentiles in an optional profile view.

**Out of scope:**

- Moneyball Search, filtered-result percentiles, or table columns.
- Moving the Moneyball upload action from My Club / Squad.
- Persisted default-view behavior; absent explicit view still defaults to General in this commit.
- Composite Moneyball role scoring.

**Implementation packet:**

- Add a UID-only current-snapshot Moneyball profile command under the Moneyball Rust feature.
- Add frontend Moneyball types, IPC query keys, presentation catalog, raw formatting, and reusable raw-plus-score cell.
- Add a validated outer General / Moneyball view to the player route while preserving the current inner attribute tab.
- Keep common player identity visible, keep all existing analysis and development controls in General, and render playing-time context plus eight metric-category tabs in Moneyball.
- Treat absent rows and pre-feature unscored rows as distinct, non-error states with the current upload location as the recovery path until Commit 4 moves it.

**Files and responsibilities:**

- src-tauri/src/features/moneyball/commands.rs — expose get_player_moneyball with a UID-only input.
- src-tauri/src/features/moneyball/query.rs — resolve active current snapshot, current player membership, scored row, legacy-row marker, raw context, raw statistics, and persisted percentiles.
- src-tauri/src/lib.rs — register the new command.
- src/features/moneyball/api/ — own invoke, query options, and Moneyball query keys.
- src/features/moneyball/types/ — mirror the closed profile DTO and metric score maps.
- src/features/moneyball/utils/moneyball-metrics.ts — own 138 presentation definitions, eight categories, labels, raw kinds, precision, units, and widths.
- src/features/moneyball/utils/format-moneyball-metric.ts — format raw counts, decimals, ratios, distance, rating, and missing values without changing numeric operands.
- src/features/moneyball/components/moneyball-metric-value.tsx — render raw value and accessible ScoreBadge treatment.
- src/features/moneyball/components/moneyball-profile-panel.tsx — render context, category tabs, loading, empty, re-import, and complete states.
- src/features/player-profile/components/player-overview-panel.tsx and the smallest supporting component split — separate common identity from General-only analysis without changing General output.
- src/app/routes/players.$uid.tsx — validate outer view, preserve inner tab, select the correct workspace, and prefetch only required current queries.
- src/app/routes/players.$uid.test.tsx plus focused Moneyball component and utility tests — prove route compatibility and metric presentation.
- src/testing/player-ipc-mock.ts or a focused Moneyball IPC mock — provide no-row, re-import, scored, and sparse-value fixtures.
- .wiki/CONCEPT.md, .wiki/ARCHITECTURE.md, and .wiki/DESIGN.md — record the implemented optional profile behavior, command boundary, metric categories, score treatment, and states.

**Behavior and data flow:**

- The route always resolves the current snapshot and existing get_player identity detail.
- Effective explicit view is General unless view=moneyball is present in this commit.
- General renders the same summary, attributes, role fit, concealment, and boost behavior as before.
- Moneyball invokes get_player_moneyball only for the active UID and displays no row when the player was not in the active import.
- A row with percentiles_json null returns needsReimport and does not expose old raw analysis in the UI.
- A scored row returns asking-price context, starts, substitutes, minutes, exact raw statistics, and exact integer/null percentile maps.
- The Moneyball panel renders context without percentile color and one selected category of raw-plus-score metrics at a time. Arrow keys, Home, and End operate across visible category tabs.
- Each metric's raw value uses presentation precision; the ScoreBadge accessible name states metric label, percentile number, and tier.
- Switching outer views preserves the existing General inner tab value so returning to General restores the prior attribute category.

**Ordered implementation steps:**

1. Add RED Rust tests for current-snapshot selection, absent row, legacy unscored row, scored row decoding, unknown UID, and older-snapshot isolation.
2. Add the smallest query and command that turn those tests GREEN and register the command.
3. Add RED frontend catalog and formatting tests that pin 138 unique IDs, all eight non-empty categories, exact context treatment, null display, precision, and raw-plus-score accessibility.
4. Implement the presentation catalog and reusable metric value component.
5. Add RED route tests for explicit Moneyball view, unchanged General view, inner-tab preservation, empty state, re-import state, and complete category rendering.
6. Refactor the profile shell only as required to keep common identity shared and General-only analysis isolated, then render the Moneyball panel.
7. Compare the completed General view against the existing route tests and component analogues, update current-state docs, format, and run validation.

**Tests and proof:**

- Rust query tests assert only active-save current-snapshot rows can return and that raw and percentile JSON must satisfy the exact catalog.
- Frontend catalog test asserts 138 unique performance IDs, exact category coverage, explicit formatting metadata, and no General metric ID collision.
- Formatter tests cover integer, two-decimal per-90, percentage, distance, rating, zero, and null.
- Metric component tests assert raw value remains visible, percentile has existing tier color semantics, accessible label includes the tier, and missing values do not become 0.
- Route tests assert view=general keeps the current workspace, view=moneyball renders the new panel, invalid view normalizes safely, the General tab search value remains intact, and no profile view reads an older snapshot.
- The expected RED proof is that view=moneyball currently normalizes away and no Moneyball query or panel exists.

**Patterns to verify:**

- src-tauri/src/features/player/query.rs and commands.rs for active-current UID-only profile reads and nullable DTO decoding.
- src/app/routes/players.$uid.tsx for current snapshot states, query invalidation, route validation, and bounded workspace layout.
- src/features/player-profile/components/player-attributes-panel.tsx and player-profile-tabs.tsx for attribute-like values and keyboard tabs.
- src/components/ui/score-badge/ for tier, accessible label, and missing-value boundaries.
- src/features/staff profile read model for separate feature-specific profile data without direct WebView database access.

**Constraints and non-goals:**

- No raw JSON parsing or percentile calculation in React.
- No profile history, full/filtered toggle, composite score, chart, radar, or comparison player.
- Do not display a pre-feature unscored row as active analysis.
- Do not alter General concealment, boosts, role summaries, attribute tabs, or route compatibility.
- Do not make Moneyball data part of get_player when General does not need it.

**Dependencies and sequencing:**

- Depends on Commit 1's migration, active-cohort marker, and persisted full-import percentiles.
- The empty-state recovery link may target the existing My Club / Squad upload until Commit 4 moves that owner; Commit 4 must update it in the same change that relocates the action.
- Supplies the frontend Moneyball catalog and metric value component reused by Search.

**Validation:** Run ./scripts/dev test 'src/app/routes/players.$uid.test.tsx' and the new src/features/moneyball test targets, then ./scripts/dev format, ./scripts/dev check, and git diff --check.

**Stop conditions:** Stop and replan if the profile cannot distinguish no row from unscored legacy row; common identity cannot be separated without changing General behavior; the 138 metrics cannot fit a bounded internally scrolling workspace at the supported desktop size; route search validation would require replacing the existing inner tab contract; or the backend would have to expose historical rows.

**Review mandate:** Verify active-snapshot isolation; General profile parity; legacy-row hiding; exact raw and score decoding; category completeness; accessible color treatment; outer-versus-inner tab routing; query loading only in Moneyball view; and empty/re-import recovery.

#### Commit 3 — Query Moneyball Search cohorts

**Status:** Pending

**Provisional commit:** feat(search): query Moneyball player cohorts

**Work:** Extend the existing Rust Search boundary with a closed Moneyball mode, raw numeric filters and sorts, full-import scores, and complete filtered-cohort score recalculation.

**Out of scope:**

- Search tabs, metric picker, table layout, upload UI, or visible Moneyball cells.
- Settings default behavior.
- Client-side percentile calculation.
- Composite role scoring.

**Implementation packet:**

- Add optional search_view and comparison_pool command inputs with backward-compatible General defaults.
- Add real-number DynamicValue and FilterValue variants.
- Extend trusted metric and filter resolution with a Moneyball-only catalog: bounded memory-backed identity/club/position/value fields, three raw context fields, and all 138 raw performance fields.
- Inner-join only scored current-snapshot Moneyball rows in Moneyball mode.
- Reuse one compiled predicate for count, full filtered cohort, and page selection.
- Return requested raw values plus a separate nullable moneyballPercentiles map for current page rows.
- Use persisted scores for Full CSV and the shared Rust engine across the complete filtered cohort for Filtered results.

**Files and responsibilities:**

- src-tauri/src/features/search/commands.rs — parse closed view and comparison-pool inputs and serialize real values plus percentile maps.
- src-tauri/src/features/search/query.rs — select General or Moneyball FROM / WHERE shape, count, order, cohort scoring, bounded page reads, and DTO assembly.
- src-tauri/src/features/search/filter.rs — accept real numeric filter values and resolve Moneyball fields only in Moneyball mode.
- src-tauri/src/features/player_metrics/resolver.rs — add real dynamic values and the minimum mode-aware seam needed to resolve trusted Moneyball expressions without exposing them to General mode.
- src-tauri/src/features/moneyball/catalog.rs and percentile.rs — expose closed key lookup and reusable cohort calculation to Search.
- src-tauri/src/features/search/mod.rs or focused new test helpers — keep tests in the owning module without introducing a parallel Search service.
- .wiki/ARCHITECTURE.md — record the implemented Moneyball Search query mode, raw operands, and server-side comparison pools.

**Behavior and data flow:**

- Missing search_view preserves the current General query byte-for-byte at the behavioral contract level.
- Moneyball mode resolves the active save's effective current snapshot and joins player_moneyball_stats on snapshot and UID with percentiles_json IS NOT NULL.
- Invalid or General-only field IDs in Moneyball mode, and Moneyball-only field IDs in General mode, fail at the Rust trust boundary.
- Filters compile to bound SQLite parameters. Decimal JSON values remain numeric; no user field text becomes SQL.
- Count applies the active view's full predicate.
- Full CSV attaches persisted percentile values for only the requested Moneyball performance fields.
- Filtered results queries the complete filtered set of UID plus requested Moneyball raw values before LIMIT/OFFSET, calculates percentiles, and retains scores by UID.
- Page selection then applies trusted raw sort and stable UID tie-break, returns at most the clamped page limit, and attaches only that page's raw values and scores.
- Context fields are raw only and have no percentile entry.
- Null raw metrics remain null, sort with the established null behavior, and do not enter percentile populations.

**Ordered implementation steps:**

1. Add RED command and query tests proving absent mode preserves General behavior and Moneyball mode excludes unscored, absent, and older-snapshot rows.
2. Add RED resolver and filter tests for real values, all representative metric key shapes including hyphens, mode rejection, bound parameters, and raw decimal comparisons.
3. Implement the smallest closed mode, value type, join, and raw field resolution that turns those tests GREEN.
4. Add RED comparison-pool tests with more than 50 matched rows so the second page proves scores use the whole filtered cohort rather than one page.
5. Implement persisted Full CSV score projection and server-side Filtered results scoring for only requested performance fields.
6. Add boundaries for no filters, AND / OR filters, empty cohort, one-player cohort, all-null metric, raw sort, stable tie-break, requested field deduplication, and requested-field limit.
7. Refactor only while General and Moneyball query tests stay green, update current-state architecture, format, and run the gate.

**Tests and proof:**

- General regression tests compare default mode results, totals, sorts, filters, requested fields, and page limits with the current contract.
- Moneyball eligibility tests seed current, older, unscored, and absent rows and assert only current scored matches return.
- Numeric tests cover integer and decimal raw values, negative rejection where the catalog disallows it, zero, null, hyphenated keys, and parameterized filters.
- Full CSV tests assert persisted scores survive identity filters and pages unchanged.
- Filtered tests seed at least 101 players, apply a predicate, request pages 0 and 50, and assert both pages use the full post-filter cohort.
- A skipped/non-player row cannot enter because only persisted matched rows are joined.
- Requested context fields return raw values without percentile entries.
- Invalid view, pool, field, operator, non-finite value, and over-limit requests fail safely.
- The expected RED proof is that the current command has no view/pool input, no real DynamicValue, no Moneyball join, and no percentile map.

**Patterns to verify:**

- src-tauri/src/features/search/query.rs current count / order / page flow and stable UID tie-break.
- src-tauri/src/features/search/filter.rs bound AST compilation and field-kind validation.
- src-tauri/src/features/player_metrics/resolver.rs closed metric ID parsing and requested-field limit.
- src-tauri/src/features/player_metrics/potential_cache.rs cohort-before-page behavior as the closest existing server-side full-cohort analogue.
- src-tauri/src/features/staff/query.rs for a scoped mode that changes joins and available fields while retaining bounded paging.

**Constraints and non-goals:**

- No SQL field, path, operator, direction, view, or pool comes directly from unchecked WebView text.
- No full result set or percentile population crosses IPC.
- No visible pagination contract changes.
- No General metric or URL behavior changes.
- No persisted filtered-percentile cache or new migration.
- No percentile operand is accepted for filter or sort.
- Keep the existing requested-field bound; the Moneyball-specific catalog remains below it.

**Dependencies and sequencing:**

- Depends on Commit 1's scored cohort and Commit 2's shared metric catalog semantics and formatter contract.
- Must finish before the Search UI can request Moneyball pages.
- Commit 4 owns all frontend adoption and upload relocation.

**Validation:** Run ./scripts/dev format, then ./scripts/dev check. Confirm the Rust test output includes the 101-player cross-page comparison-pool proof and all existing General Search tests. Run git diff --check.

**Stop conditions:** Stop and replan if one predicate cannot be reused consistently for count, cohort, and page; filtered scores require sending the cohort to React; a 1,000-row bounded server calculation requires a new cache or background architecture; raw SQLite JSON numbers cannot retain required precision through the existing DTO; requested field count exceeds the established bound; or General behavior changes when mode is absent.

**Review mandate:** Verify trust-boundary validation; General backward compatibility; current-scored-row eligibility; raw filter/sort semantics; complete filtered-cohort scope; persisted Full CSV scores; null handling; page stability; requested-field bounds; and absence of unbounded IPC.

#### Commit 4 — Add the Moneyball Search view

**Status:** Pending

**Provisional commit:** feat(search): add Moneyball search view

**Work:** Deliver the optional virtualized Moneyball Search workspace, all metric columns and filters, comparison-pool control, separate layout, and its sole upload owner.

**Out of scope:**

- Persisted default view; absent explicit route value still defaults to General until Commit 5.
- Profile score calculation changes.
- Composite role scoring or new navigation.
- Youth upload relocation.

**Implementation packet:**

- Add validated General / Moneyball Search view and Moneyball comparison-pool URL state.
- Add a Moneyball-specific filter registry, default sort, default columns, table layout ID, and cell renderer using the catalog from Commit 2.
- Reuse the shared VirtualizedPlayerTable and bounded query adapter; do not fork the virtualizer.
- Refactor the table header and player-table store only enough to accept a view-specific metric catalog and sanitizer.
- Move the genericized Moneyball import modal and action into Moneyball Search, retain Youth in My Club / Squad, and remove every duplicate Moneyball action.
- Invalidate current Moneyball Search and Profile queries after a successful import while preserving Youth-only Academy invalidation.
- Add empty, loading, error, no-result, replacement, and success states plus product smoke coverage.

**Files and responsibilities:**

- src/app/routes/search.tsx — validate view/pool, choose view-specific filters and defaults, compose outer tabs, prefetch the effective explicit view, and keep General behavior intact.
- src/features/search/api/fetch-search-players.ts, search-players-query-options.ts, and search-keys.ts — include view/pool in IPC and query identity.
- src/features/search/types/player-summary.ts, filter-rule.ts, and search-sort.ts — accept real raw values and percentile maps without weakening General types.
- src/features/search/utils/search-url-search.ts — parse and serialize closed view, pool, real filter values, and view-specific compatible state.
- src/features/search/utils/filter-registry.ts and dynamic-columns.ts — select the General or Moneyball presentation catalog and operators.
- src/features/search/components/search-results-panel.tsx — render a Moneyball adapter with raw-plus-score cells and explicit Moneyball profile navigation while preserving the General adapter.
- src/features/search/components/search-filter-bar.tsx and editor/strip components — display Moneyball labels and decimal inputs through the selected catalog.
- src/features/moneyball/components/moneyball-search-controls.tsx — own Full CSV / Filtered results selection, upload / replace action, and import outcome.
- src/components/player-table/player-table-header.tsx — receive the active catalog or field resolver instead of assuming the General catalog.
- src/stores/use-player-table-store.ts — add moneyball-search with independent defaults, widths, ordering, sanitation, and storage migration.
- src/features/csv-import/components/squad-csv-import-modal.tsx and test — rename/generalize the modal without changing picker/drop/context safety.
- src/features/csv-import/components/squad-csv-import-actions.tsx — retain only Youth Academy upload and remove Moneyball state.
- src/features/csv-import/utils/use-csv-import.ts — invoke targeted Moneyball query invalidation after success while preserving Youth behavior.
- src/app/routes/my-club.tsx and src/app/routes/my-club-squad.test.tsx — prove My Club retains Youth and no Moneyball action.
- src/app/routes/search.test.tsx plus Search component, URL, store, and catalog tests — prove the complete view.
- src/testing/search-ipc-mock.ts and src/testing/csv-import-ipc-mock.ts — support Moneyball modes, percentiles, replacement, and import states.
- e2e/tauri-ipc-stub.ts and e2e/smoke.spec.ts — cover upload ownership, virtual scrolling, filtering, comparison-pool switching, and profile entry.
- .wiki/CONCEPT.md, .wiki/ARCHITECTURE.md, and .wiki/DESIGN.md — record the implemented Search, table, upload, and visual contracts.

**Behavior and data flow:**

- Search renders outer General / Moneyball tabs beneath the page heading.
- An explicit view change writes the route view, resets incompatible filters and sort to that view's defaults, and leaves browser history able to restore the prior URL.
- General uses the existing catalog, layout ID, default CA sort, query mode, cells, and profile navigation with view=general.
- Moneyball uses the Moneyball catalog, moneyball-search layout, default Average Rating descending sort, Filtered results pool, and current scored cohort query.
- The shared virtualizer requests 50-row windows using view/pool/filter/sort/requested-field query keys. It renders no pager.
- Performance cells show formatted raw value plus the returned percentile score. Context and memory-backed cells show raw values only.
- Applying a Moneyball filter adds its column once, exactly as General does. Column removal does not remove the filter.
- No active cohort shows an Upload Moneyball CSV action. An existing cohort shows Replace Moneyball CSV plus the most recent safe stored/skipped result in the current context.
- A successful upload invalidates Moneyball Search pages and Moneyball Profile queries; the new complete cohort replaces visible rows immediately.
- Save or snapshot changes close an open modal, suppress late outcomes, and refresh the current-only empty or cohort state.
- My Club / Squad retains only Upload Youth Academy CSV and its existing Academy invalidation.

**Ordered implementation steps:**

1. Add RED URL, query-key, and IPC mapping tests for view, pool, decimal filters, and backward-compatible General defaults.
2. Add the Moneyball query adapter and types that consume Commit 3 without rendering a second table implementation.
3. Add RED store and catalog tests for moneyball-search defaults, independent persistence, migration, sanitation, all 138 selectable fields, and General isolation.
4. Generalize the header's metric source and add the separate Moneyball table layout.
5. Add RED route and results tests for tabs, virtual page fetching, raw-plus-score cells, filter/sort operands, pool switch, no cohort, no results, and explicit profile navigation.
6. Implement the Moneyball Search composition with the shared table, filter components, and score cell.
7. Add RED CSV ownership tests, generalize the modal, move Moneyball controls to Search, leave Youth in My Club, and add targeted invalidation.
8. Extend IPC stubs and smoke with at least 101 Moneyball players so internal second-window scrolling is exercised without pager controls.
9. Compare the General Search and My Club results against existing tests, update current-state docs, format, and run all validation.

**Tests and proof:**

- URL tests assert unknown view/pool normalization, view-specific defaults, decimal round-trip, invalid Moneyball fields rejected in General, and incompatible state reset on view switch.
- Query-key tests assert view, pool, filters, requested fields, offset, and limit all distinguish cached pages.
- Store tests assert General and Moneyball layouts do not overwrite each other, v3 state migrates with a valid new default, invalid IDs are dropped by the correct catalog, and at least one column remains.
- Route tests assert only the active view queries, all 138 metrics are searchable in the picker, raw filter application adds a column, Full CSV and Filtered results return distinct stubbed scores, and General remains unchanged.
- Virtual table tests use more than 100 rows and assert one internal scroll owner, bounded document height, a second 50-row request, stable keyboard navigation, and no Previous / Next controls.
- Cell tests assert raw number plus integer percentile, correct formatting, accessible score tier, neutral context fields, and null as an em dash.
- Import tests assert Search owns Moneyball upload and replacement, My Club no longer exposes it, Youth remains, stale context closes the modal, and successful Moneyball import invalidates Search/Profile rather than Academy.
- Smoke covers explicit Moneyball tab, upload prompt/action, scored rows, one filter, comparison pool, virtual scroll beyond the first page, profile opening in Moneyball view, and return-state restoration.
- Expected RED failures are the absent view/pool URL fields, absent Moneyball table ID/catalog, current General-only table renderer, and current My Club Moneyball action.

**Patterns to verify:**

- SearchResultsPanel and StaffSearchResultsPanel for adapters around the shared virtual table.
- configurable-table-contract.test.tsx for scroll, header, row, and keyboard invariants.
- use-player-table-store.ts v3 migration and per-workspace layouts.
- Search filter editor tests for query-silent draft state and Done behavior.
- Staff Shortlist upload for a replacement action owned by the list it populates.
- Current Squad CSV modal for picker, native drop, context generation, safe path handling, and result copy.

**Constraints and non-goals:**

- Do not add a second virtualizer, pager, unbounded WebView collection, or route.
- Do not expose the Moneyball catalog in General Search.
- Do not change the existing General table layout stored under search.
- Do not color raw context values or use score as the filter/sort operand.
- Do not leave a duplicate Moneyball upload action in My Club.
- Do not move or change Youth Tracker behavior.
- Do not add a new dependency for tabs, tables, formatting, or state.

**Dependencies and sequencing:**

- Depends on Commit 2's frontend catalog/value component and Commit 3's query mode.
- Must update Commit 2's profile empty/re-import recovery target from My Club to Moneyball Search.
- Supplies explicit view behavior that Commit 5 will use as the override above the persisted default.

**Validation:** Run ./scripts/dev test src/app/routes/search.test.tsx src/app/routes/my-club-squad.test.tsx src/features/search src/features/csv-import src/features/moneyball src/stores/use-player-table-store.test.ts, then ./scripts/dev format, ./scripts/dev check, ./scripts/dev smoke, and git diff --check.

**Stop conditions:** Stop and replan if the shared table cannot accept a view-specific catalog without regressing General; more than 256 requested fields are needed; filtered percentiles depend on loaded pages; layout migration would discard the existing General layout; moving upload breaks the context-generation safety contract; or 101-row smoke shows document growth, nested scrolling, or visible pagination.

**Review mandate:** Verify General isolation; complete 138-field availability; raw filter/sort operands; pool URL and cache identity; full-height virtualization beyond 50 rows; separate layout persistence; accessible raw-plus-score cells; upload single ownership; stale-context safety; invalidation targets; and profile view navigation.

#### Commit 5 — Choose the default player analysis view

**Status:** Pending

**Provisional commit:** feat(settings): choose default player analysis view

**Work:** Add one persisted app preference that selects the default General or Moneyball view for Search and Player Profile while explicit route state remains authoritative.

**Out of scope:**

- Save-scoped or database-backed preferences.
- Separate Search and Profile defaults.
- Automatic CSV upload, background refresh, or redirect away from empty Moneyball states.
- Any change to percentile calculations or query eligibility.

**Implementation packet:**

- Add a small versioned persisted Zustand preference store with general as the migration-safe default.
- Add an accessible Settings Preferences section with one Default player analysis view control.
- Make Search and Profile derive effective view from explicit URL first and the persisted default second.
- Keep context-specific Search row navigation explicit and let global/direct profile navigation follow the default.
- Ensure loader/query selection and first render use the same effective view with no General/Moneyball flash or wrong prefetch.

**Files and responsibilities:**

- src/stores/use-moneyball-preferences.ts and test — own versioned persisted defaultAnalysisView, validation, migration, and setter.
- src/app/routes/settings.tsx and settings.test.tsx — render and verify the Preferences section and control.
- src/app/routes/search.tsx and search.test.tsx — apply the fallback when view is absent while retaining explicit override.
- src/app/routes/players.$uid.tsx and players.$uid.test.tsx — apply the same fallback and preserve inner tab.
- src/features/search/components/global-player-search.tsx and test only if an explicit view is currently injected — ensure global navigation omits view so the preference can decide.
- src/app/router.test.ts or focused route tests — prove direct navigation, reload, and invalid persisted values.
- .wiki/ARCHITECTURE.md and .wiki/DESIGN.md — record the implemented app-local preference and precedence rule.

**Behavior and data flow:**

- The persisted store hydrates one closed general or moneyball value from localStorage; absent or invalid state becomes general.
- Settings writes the value immediately through the store and exposes a visible label and explanatory copy that it affects both surfaces.
- Search and Profile keep optional explicit route view values. A present valid value wins.
- When the URL omits view, the route loader and component both use the store's current default for query selection and rendering.
- Changing the setting while a route has no explicit view updates that route to the new effective surface; a route with explicit view remains unchanged.
- General Search rows link to view=general and Moneyball Search rows link to view=moneyball, so the user's active context wins over the fallback.
- Global player search and a direct /players/$uid URL omit view and therefore open the configured default.
- An empty default Moneyball Search or Profile shows the existing upload/empty state; the preference does not silently fall back to General.

**Ordered implementation steps:**

1. Add RED store tests for default, valid persistence, invalid migration, version handling, and setter.
2. Implement the minimum Zustand store using the current persisted-store pattern.
3. Add RED Settings tests for the labelled control, shared-surface explanation, immediate update, and keyboard operation.
4. Add the Preferences section without changing Save data or Bridge behavior.
5. Add RED Search and Profile tests for absent-view fallback, explicit override, changing preference with and without an explicit view, correct loader query, and no wrong-surface flash.
6. Apply one shared effective-view helper or the smallest equivalent that keeps loader and component decisions identical.
7. Verify General and Moneyball row navigation precedence plus global/direct default behavior.
8. Update current-state docs, format, and run full feature validation.

**Tests and proof:**

- Store tests seed no value, valid General, valid Moneyball, malformed JSON, unknown value, and prior version.
- Settings tests use the accessible label and assert both Search and Profile react to the same single value.
- Search route tests assert absent view uses each default, explicit General wins over Moneyball default, explicit Moneyball wins over General default, and changing the setting does not rewrite an explicit URL.
- Profile route tests assert the same precedence while retaining the current inner attribute tab.
- Loader/query tests assert Moneyball default does not prefetch General pages and General default does not invoke Moneyball profile data.
- Global search tests assert navigation leaves view absent; General and Moneyball Search row tests assert explicit view.
- Smoke extends the workflow to set Moneyball default, navigate to Search and a profile without view, confirm Moneyball, switch General explicitly, and confirm the override survives reload/back.
- Expected RED failure is that no preference store or Settings control exists and absent route values always resolve to General.

**Patterns to verify:**

- src/stores/use-layout-store.ts and use-player-table-store.ts for versioned Zustand persistence and safe migration.
- src/features/memory-read/stores/use-load-data-preferences.ts for app-local presentation state.
- src/app/routes/search.tsx and players.$uid.tsx validated search and loader boundaries.
- Settings existing section heading hierarchy and control primitives.

**Constraints and non-goals:**

- No SQLite migration, IPC command, save ID, or snapshot ID participates in the preference.
- Do not duplicate the preference into separate Search/Profile values.
- Do not force a General fallback when Moneyball has no data.
- Do not make validateSearch depend on unvalidated localStorage content.
- Do not change explicit URL values when the setting changes.
- Do not add a new state library or browser storage abstraction.

**Dependencies and sequencing:**

- Depends on Commits 2 and 4 providing both explicit views and their empty states.
- This is the final implementation commit. After it clears checkpoint, set the feature to Validation and run the feature completion workflow rather than starting unplanned polish.

**Validation:** Run ./scripts/dev test src/app/routes/settings.test.tsx src/app/routes/search.test.tsx 'src/app/routes/players.$uid.test.tsx' src/stores/use-moneyball-preferences.test.ts plus affected global-search tests, then ./scripts/dev format, ./scripts/dev check, ./scripts/dev smoke, and git diff --check.

**Stop conditions:** Stop and replan if loader and component cannot share one effective-view decision; persistence hydration causes an unavoidable wrong-view render; explicit URL values cannot remain authoritative; one setting cannot satisfy both routes without duplicated state; or implementation requires database persistence.

**Review mandate:** Verify one shared preference; safe persistence migration; explicit-route precedence; consistent loader/render behavior; no query flash; Search/Profile synchronization; context-specific versus global navigation; empty Moneyball defaults; Settings accessibility; and unchanged Save data / Bridge sections.

## Active work

**PR:** PR 1 — Optional Moneyball analysis views

**Commit:** Commit 1 — Persist Moneyball percentile cohorts

### RED proof

Add a Rust service test that seeds two current-snapshot Moneyball rows, imports a valid replacement containing only one matched player, and asserts the omitted player no longer exists while a row on an older snapshot remains. Add pure percentile assertions showing null values are excluded, one/non-varying populations receive 50, and a skipped unknown UID cannot change a matched player's score. The current implementation fails because it upserts included players, has no percentile engine, and has no percentiles_json column.

### Expected outcome

Schema v30 preserves existing rows with null percentile provenance. Every new successful Moneyball import prepares a matched-only 138-metric percentile cohort outside the database lock and atomically replaces only the captured current snapshot's Moneyball rows. Youth import and every failure path retain their existing behavior.

### Explicit exclusions

- No React or IPC read model.
- No Search or Profile view.
- No upload relocation.
- No filtered-result calculation.
- No JAY-20 composite scoring.

## Discoveries and replanning

- Planning started only after My Club documentation close-out reached main at 8a6b93c3c08177f268919df7dd743862fa64542c and .wiki/features/active/ contained no competing ledger.
- The developer clarified that Moneyball presentation is current-snapshot-only, older snapshot data must remain stored for future timelines, and CSV UIDs absent from the current snapshot must be skipped before percentile calculation.
- The developer clarified that Search remains virtualized with no visible pagination and Player Profile always uses the matched full-CSV population.
- The developer accepted raw values plus integer percentile color, Filtered results as the default Search pool, and one shared default-view setting.
- Repository inspection found current same-snapshot Moneyball re-import is an upsert that retains omitted players. The plan changes only Moneyball to complete current-snapshot replacement.
- Repository inspection found pre-feature rows have no trustworthy exact import-cohort marker. Migration preserves them with null percentiles and current UI requires re-import instead of backfilling potentially mixed rows.
- The pinned legacy implementation coerces missing values to zero. This plan deliberately diverges: null remains unavailable and is excluded.
- Repowise CLI was unavailable. Codebase Memory was available as advisory architecture/search evidence and direct repository files, tests, configuration, and Git remain authoritative.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | None | Pending record | Planning only | Not run | None |

## Final validation

- Run ./scripts/dev format and confirm no unintended files change.
- Run ./scripts/dev test and record the complete frontend result.
- Run ./scripts/dev check and record Biome, TypeScript, secretlint, Rust format, Clippy, and Rust test results.
- Run ./scripts/dev smoke with the Moneyball browser fixture containing at least 101 players and record the complete Playwright result.
- Run git diff --check against the exact feature range.
- Verify migration from a seeded schema-v29 database preserves all existing Moneyball raw/context/import fields with null percentile provenance and that a new import creates exact score objects.
- Verify import-time percentile proofs for matched-only population, unknown UID exclusion, null exclusion, ties, neutral pools, inversion, successful omitted-player replacement, zero-match replacement, older-snapshot preservation, stale context, and transactional rollback.
- Verify all 138 performance fields exist once in the Rust trust catalog and once in the Moneyball frontend presentation catalog, with eight complete categories; verify the exact 19 inverse directions in the Rust scoring contract.
- Verify General Search URLs, query defaults, filters, sort, table layout, virtualization, and profile navigation remain compatible.
- Verify Moneyball Search has no pager, owns one scroll region, requests bounded pages beyond row 50, exposes every performance metric as column/filter/sort, and uses raw operands.
- Verify Filtered results scores use the complete post-filter cohort rather than the loaded page; verify Full CSV scores remain the persisted matched-import values.
- Verify Player Profile General behavior remains unchanged and Moneyball renders context plus every category using full-import scores only.
- Verify no-row, re-import-required, no-results, loading, error, successful import, and successful replacement states.
- Verify Settings preference default and explicit URL precedence for Search, Player Profile, Search-row navigation, global search, reload, and browser back.
- Run native Tauri/WebView manual checks at 1280×800 and 1600×900 for category density, table containment, horizontal overflow, keyboard tabs, filter editor, column menu, resize, import picker/drop, replacement result, and focus restoration.
- Run a representative near-1,000-row native import and filtered Search interaction, record observed import and filter response timing, and confirm the UI remains interactive. This is evidence, not a brittle pass/fail benchmark.
- Restart the native app after import and confirm the current scored cohort, table layout, and default view persist.
- Switch save and current snapshot contexts during an open import and after a completed import; confirm late outcomes are suppressed, old snapshot rows stay stored, and current UI never displays them.
- Do not claim browser smoke proves native picker, real WebView IPC, SQLite persistence, upgrade, restart, or file access.
- ./scripts/dev bridge-test is outside the affected path unless implementation unexpectedly changes bridge code. ./scripts/dev mutate remains unsupported and must not be reported as passed.
- After all commits complete, run the fresh feature-complete review and documentation reconciliation required by the feature completion workflow.

## Documentation impact

Complete during implementation and final reconciliation:

- .wiki/CONCEPT.md — move optional Moneyball analysis from import-only wording to the delivered current-snapshot Search/Profile capability while retaining advanced JAY-20 and history exclusions.
- .wiki/ARCHITECTURE.md — record schema v30, full-cohort replacement, percentile ownership, current-only profile query, Moneyball Search mode, server-side comparison pools, upload ownership, and app-local preference.
- .wiki/DESIGN.md — record General/Moneyball outer views, eight metric categories, raw-plus-percentile treatment, comparison-pool control, Moneyball table defaults, empty/re-import states, upload placement, and default preference.
- .wiki/TODO.md — keep this ledger Active until feature completion, then move JAY-19 to Completed during reconciliation.
- .wiki/features/completed/ — archive a condensed feature record only after implementation, final validation, feature review, and documentation reconciliation.
- No ADR is planned unless implementation disproves the established snapshot, Rust/SQLite, route, or preference boundaries and forces a consequential alternative.
