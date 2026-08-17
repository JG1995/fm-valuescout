# Staff Shortlist CSV Enrichment

## Status

Active

## Intent

Add a save-owned Staff Shortlist workspace that imports a filtered Football Manager staff CSV, joins each row to current extracted staff by exact UID, and keeps CSV-only recruitment context without replacing memory-owned staff facts.

## User-visible behavior

- Staff gains a third **Shortlist** tab beside Search and My Staff.
- A save with no imported shortlist shows an upload prompt instead of an empty table.
- The upload dialog accepts one UTF-8 semicolon-delimited staff CSV. When the active save already has a shortlist, the dialog warns that a successful upload replaces it and labels the action as a replacement before the user selects the file.
- A successful upload replaces the active save's prior shortlist with the valid rows whose UIDs match staff in the effective current snapshot. The outcome reports total, stored, and skipped rows.
- The shortlist persists for its app save across app restarts and later Load Data snapshots. A row appears only while its UID exists in the effective current snapshot; loading a new snapshot never copies CSV name, age, ability, club, or attribute values into extracted staff.
- The table starts with **All jobs** and can filter by one exact **Preferred Job** value from the active upload. All jobs uses the saved shortlist columns and sort.
- Selecting a Preferred Job with one matching ValueScout role shows the basic and CSV shortlist columns plus only that role score, then ranks the filtered staff by that score descending. This contextual projection does not overwrite the saved All jobs layout or sort.
- Selecting **Coach** shows the six Attacking, Defending, and Possession Technical or Tactical coaching scores. It excludes Fitness, Goalkeeping, and Set Piece scores and does not automatically choose one of the six as the score sort.
- Selecting **Manager** shows only the basic and CSV shortlist columns and sorts by CA descending because ValueScout has no Manager role score. An unknown future Preferred Job uses the same safe fallback.
- An independent **Only unemployed** control keeps rows whose trimmed CSV **Club Job** value is blank or exactly `-`. The stored and displayed CSV value remains unchanged.
- The configurable, virtualized table displays current extracted staff fields and role scores together with CSV **Preferred Job**, **Club Job**, and **Coaching Qualifications** columns. Activating a row continues to open the existing staff profile.
- Preferred Job, unemployment state, the independent All jobs layout, and its base sort survive normal route navigation through validated URL or persisted table state. Contextual job columns and effective sort are derived from that state.
- Invalid, stale, unsupported, oversized, duplicate-UID, zero-match, or failed imports leave the prior shortlist unchanged and show actionable feedback.

## Invariants

- The current snapshot remains authoritative for staff identity, age, CA, PA, club, contract, attributes, and job-fit scores. The CSV owns only UID, Preferred Job, Club Job, and Coaching Qualifications for this feature.
- CSV Preferred Job is recruitment metadata. It is separate from extracted contract `job_id` and from calculated `staff_role_scores`.
- Import uses an exact numeric UID join. It never creates staff or mutates rows in `staff` or `staff_role_scores`.
- One app save owns at most one active shortlist. A successful import replaces that save's complete prior entry set inside one transaction; another save's shortlist is never changed.
- Replacement begins only after the whole file passes trusted Rust validation, the active save and snapshot context remains current, and at least one CSV UID matches current staff. Every failure before commit preserves the prior shortlist.
- A later snapshot can hide a saved shortlist entry when its UID is absent, but it does not delete that save-owned entry. Deleting the owning app save cascades its shortlist.
- The parser preserves trimmed Club Job text, including `-` and an empty string. Unemployed means `trim(club_job) = '' OR trim(club_job) = '-'` and has no dependency on current club, extracted `job_id`, or a role score.
- Preferred Job filtering uses equality against a value supplied by the saved shortlist options. It is not substring matching. React maps the selected display value to an existing score ID only for contextual columns and ordering; the mapping never changes CSV data, extracted `job_id`, or score calculation.
- The contextual projection always includes Name, Age/DOB, Nation, current Club, CA, PA, Preferred Job, Club Job, and Coaching Qualifications. A direct job adds one role score; Coach adds exactly six outfield coaching scores; Manager and unknown values add none.
- Contextual columns and their effective sort never mutate the saved All jobs layout or base sort. Returning to All jobs restores both.
- Rust owns file access, format and size validation, context validation, UID reconciliation, replacement, filtering, sorting, joining, and bounded paging. React owns route state, presentation, table preferences, dialog interaction, and bounded TanStack Query caches.
- The staff-specific row limit must accept the supplied 2,180-row export. The existing 1,000-player limit and Youth Tracker or Moneyball behavior remain unchanged.
- The WebView never receives an unbounded shortlist collection and never performs the UID join or unemployment filter.

## Non-goals

- Multiple named shortlists, shortlist history, merge or append imports, manual row editing, row removal, or CSV export.
- Importing CSV values into staff profiles or showing shortlist metadata outside the Shortlist workspace.
- Treating CSV Person, Age, CA, PA, or Club as authoritative or using them as fallback values.
- Adding a Manager score, combining the six Coach scores into one value, changing any staff scoring formula, or inferring a score mapping from extracted contract `job_id`.
- Adding the general Staff Search filter editor to Shortlist. This feature adds only Preferred Job and Only unemployed controls.
- Changing player CSV formats, player enrichment replacement rules, staff extraction, staff scoring formulas, bridge schema, boosts, or current-snapshot selection.
- Supporting non-CSV files, non-UTF-8 text, alternate delimiters, or a header-mapping UI.

## Current-state map

- Relevant components: `src/app/routes/staff.tsx` owns the URL-backed Search and My Staff route; `src/features/staff/components/staff-workspace-tabs.tsx` owns the accessible workspace tabs; `src/features/staff/components/staff-search-results-panel.tsx` adapts both scopes to the shared configurable virtual table.
- Data model: snapshot-owned `staff` rows and `staff_role_scores` provide current facts and calculated fit. Save-owned tables already use `saves(id) ON DELETE CASCADE` when data must survive snapshot replacement.
- Persistence and migrations: `src-tauri/src/db/migrations.rs` has 26 monotonic migrations. Migration v17 established save-owned Youth enrichment, while v18 established snapshot-owned Moneyball enrichment; neither stores staff shortlist metadata.
- Existing behavioral assumptions: `src-tauri/src/features/staff/query.rs` resolves the active save's effective current snapshot, bounds pages to 200 rows, and implements Search and My Staff scopes. `StaffSummary.dynamic_values` contains numeric attributes and role scores only.
- Architectural seams: `src-tauri/src/features/csv_import/` owns trusted file import, regular-file and UTF-8 checks, context-token revalidation, and exact player UID reconciliation. `src/features/csv-import/` owns file-selection UI. The route layer may compose CSV import and Staff feature components without either frontend feature importing the other.
- Frontend state: staff sort and filters are URL-backed. `src/stores/use-player-table-store.ts` persists independent `staff-search` and `my-staff` layouts at storage version 2.
- Existing upload pattern: `src/features/csv-import/components/squad-csv-import-modal.tsx` uses the Tauri dialog, context-aware feedback, and a modal. The dialog permission already exists; this feature needs no broader capability.
- Project validation commands: `./scripts/dev format`, `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev smoke`. Rust tests run through the full `check` surface.
- Primary risks: destructive replacement before validation, importing against a changed save or snapshot, applying the existing 1,000-player limit to a valid 2,180-row staff export, leaking CSV-owned strings into global staff facts, and losing existing table layouts during the store migration.

## Feature architecture

Migration v27 adds `staff_shortlist_entries` with `(save_id, staff_uid)` as its primary key and `save_id REFERENCES saves(id) ON DELETE CASCADE`. Each row stores non-empty `preferred_job`, raw trimmed `club_job`, and `coaching_qualifications`. It deliberately has no foreign key to snapshot-owned `staff`, because the shortlist must survive snapshot replacement. An index on `(save_id, preferred_job COLLATE NOCASE)` supports options and exact filtering.

`src-tauri/src/features/csv_import/staff_shortlist.rs` owns the dedicated staff export contract instead of widening player `CsvImportFormat` or changing player summary fields. It requires unique `Unique ID`, `Preferred Job`, `Club Job`, and `Coaching Qualifications` headers, accepts only the existing 1 MiB trusted file bound and at most 10,000 data rows, and ignores non-authoritative extra columns such as Person, Age, CA, PA, and Club. The module captures the active save and current snapshot tokens plus the current staff UID set, parses outside the database lock, revalidates context inside the replacement transaction, rejects a zero-match import, deletes only the captured save's old entries, and inserts the matched rows. A dedicated command returns `totalStaff`, `storedStaff`, and `skippedStaff`.

`src-tauri/src/features/staff/query.rs` gains a Shortlist scope through the existing bounded staff query path rather than a second full query implementation. The scope joins `staff_shortlist_entries` by active save and staff UID, always reads current staff fields from `staff`, selects the three CSV strings into optional shortlist metadata, applies exact Preferred Job and unemployment predicates before count and paging, and returns the distinct saved Preferred Job options in case-insensitive display order. Search and My Staff reject shortlist-only sort fields and continue returning no shortlist metadata. A `no_shortlist` page state is distinct from no current snapshot and from zero results after filters.

React adds a `shortlist` Staff view with independent URL-backed base sort keys, `preferredJob`, and `unemployedOnly`. `StaffSearchResultsPanel` grows a third adapter scope but retains the shared virtual table, row activation, bounded page queries, and one-scroll-owner layout. A shortlist-only metric catalog adds Preferred Job, Club Job, and Coaching Qualifications without exposing empty CSV columns in Search or My Staff. A new `staff-shortlist` table layout and storage migration preserve all existing layouts and give All jobs a default column set that makes the three CSV fields visible.

`src/features/staff/utils/staff-shortlist-presentation.ts` owns one explicit Preferred Job presentation map. All jobs returns the persisted layout and base sort. A direct match returns the fixed basic and CSV columns plus one mapped score and forces that score descending. Coach returns the fixed columns plus the six Attacking, Defending, and Possession Technical or Tactical scores; it keeps the saved sort when that field remains visible and otherwise uses CA descending without selecting a coaching score. Manager and unknown values return the fixed columns with CA descending. These projections are derived and never write to the persisted table store.

| Preferred Job | Contextual score field |
| --- | --- |
| Assistant Manager | `role.assistant_manager` |
| Director of Football | `role.director_of_football` |
| Fitness Coach | `role.coach_fitness` |
| Goalkeeping Coach | `role.coach_goalkeeping` |
| Head of Youth Development | `role.head_of_youth_development` |
| Head Performance Analyst | `role.head_performance_analyst` |
| Loan Manager | `role.loan_manager` |
| Performance Analyst | `role.performance_analyst` |
| Physio | `role.physio` |
| Recruitment Analyst | `role.recruitment_analyst` |
| Scout | `role.scout` |
| Set Piece Coach | `role.set_piece_coach` |
| Sports Scientist | `role.sports_scientist` |
| Technical Director | `role.technical_director` |

Coach maps to `role.coach_attacking_technical`, `role.coach_attacking_tactical`, `role.coach_defending_technical`, `role.coach_defending_tactical`, `role.coach_possession_technical`, and `role.coach_possession_tactical` as a group. Manager deliberately has no score mapping.

`src/features/csv-import/components/staff-shortlist-import-modal.tsx` owns the upload and replacement warning. It uses the current snapshot's save and snapshot IDs as its visible-context key, calls the dedicated import command, suppresses stale outcomes when context changes, and invalidates Staff queries after success. Existing-list uploads show the warning before Browse can open and use **Choose replacement CSV** wording. A successful replacement resets the route filters to All jobs and Only unemployed off so an option from the old upload cannot hide the new results.

## Uncertainty register

### Known

- The supplied FM26 export is UTF-8, semicolon-delimited, and about 171 KiB. It has 2,180 data rows, 2,180 unique numeric UIDs, and the headers `Unique ID`, `Person`, `Age`, `CA`, `PA`, `Preferred Job`, `Club`, `Club Job`, and `Coaching Qualifications`.
- Every supplied row has Preferred Job and Coaching Qualifications. Club Job is `-` for 1,966 rows and contains a named job for the remaining rows; the product contract also treats a blank Club Job as unemployed.
- The supplied export contains 15 distinct Preferred Job values. Thirteen have one direct score match, Coach maps to six outfield coaching scores, and Manager has no current ValueScout score.
- The current player importer is capped at 1,000 rows, so it cannot be reused unchanged for this valid staff export.
- The repository is clean on `main`, `origin/main` is the configured upstream, the latest reachable release tag is `v0.5.2`, and migration v26 is current at planning time.

### Assumptions

- Football Manager can change irrelevant extra columns without changing the four required shortlist columns. The parser therefore validates the required contract and ignores extra columns instead of requiring the supplied nine-column layout exactly.
- Coaching Qualifications can be empty in a future valid export and remains a stored display string. Preferred Job cannot be empty because it defines the requested filter domain.
- A 10,000-row staff-specific cap is sufficient for a user-filtered shortlist and leaves the existing 1 MiB file bound as the earlier practical bound for unusually wide files.

### Decisions

- A successful upload replaces, rather than merges with, the active save's shortlist. The UI warns before file selection when replacement applies.
- Shortlist entries are save-owned and persist across app restarts and snapshot replacement.
- The current snapshot supplies all staff facts. CSV Person, Age, CA, PA, and Club are ignored after format parsing.
- Blank and `-` Club Job values both mean unemployed. The raw trimmed string is preserved and the query derives unemployment.
- Preferred Job defaults to All jobs and filters exact values. Direct score matches derive one contextual score column and descending score sort; Coach derives six score columns without choosing one; Manager and unknown values derive no score column and use CA descending.
- Contextual job projections are frontend presentation rules over trusted score IDs. They do not change Rust filtering, persistence, score calculation, the saved All jobs layout, or its base sort.
- Imports with no current staff UID matches fail without replacing the prior shortlist. Partially matched imports replace with their matched subset and report skipped rows.
- One PR is sufficient. The additive schema is an atomic commit inside the PR, not a separate trunk publication boundary, because no other feature consumes it and the complete PR remains reviewable and green.
- The compatible user-visible capability has provisional `minor` release intent. At publication, the create-PR procedure must recalculate the complete range from the latest tag before preparing version and changelog files.
- No ADR is required. The save-owned table, transactional replacement, and Rust-owned import/query boundaries apply established repository patterns without introducing a competing architecture.

### Unknowns

- Future FM export header changes are not known. Missing or duplicate required headers fail safely and can trigger a parser update when a real export demonstrates the need.
- Native Tauri dialog and focus behavior still need assembled-app validation; browser tests can prove component behavior but not the operating-system picker.
- No unknown blocks Commit 1.

### Risks

- An import for the wrong FM save can partially overlap UIDs. Exact UID matching and the skipped count reduce the risk but cannot prove the export's origin; the replacement warning and visible outcome remain necessary.
- A save or snapshot can change while the native picker or parser is active. Both backend context-token revalidation and frontend context scoping must prevent a late result from replacing or presenting data for the wrong context.
- Adding shortlist string columns to the common Staff table without scope validation could expose unusable columns or invalid SQL in Search and My Staff.
- Persisted Zustand layouts can lose user choices if storage version 3 does not merge the new layout with version 2 state.
- The query must apply both filters before `COUNT`, sort, `LIMIT`, and `OFFSET`; client-side filtering would produce incorrect totals and page gaps.
- An exact FM Preferred Job label can change in a future export. Unrecognized labels must use the Manager fallback instead of selecting a plausible but unverified score.
- A contextual projection can accidentally mutate or obscure the saved layout and sort. Tests must switch through a direct job, Coach, Manager, and back to All jobs to prove restoration.

## Walking skeleton

Import one synthetic staff row for the active save, persist its UID and three CSV strings, join it to the effective current snapshot, return it through a bounded `list_staff_shortlist` command, and render it in the Shortlist tab with All jobs selected. Selecting a directly mapped job then proves the vertical score path by requesting its role score, showing only that score beside the fixed shortlist columns, and sorting the bounded query descending. The remaining cases harden replacement, stale context, unmatched UIDs, Coach and Manager projections, unemployment filtering, configurable All jobs columns, and user feedback.

## Delivery plan

### PR 1 — CSV-backed Staff Shortlist workspace

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(staff): add CSV-backed shortlist workspace`

**Branch:** `feature/staff-shortlist`

**Base:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** Squash

**Required check:** `check` must pass before merge.

**Provisional release intent:** `minor`, currently targeting `0.6.0` from `v0.5.2`; recalculate from the latest reachable tag and complete unreleased range before the release-preparation commit.

**Feature close-out:** Not run

**CI repair rounds:** 0

**Purpose:** Deliver the complete save-owned import, current-snapshot reconciliation, bounded query, and Staff Shortlist UI as one compatible capability. The schema commit remains separately reviewable without creating an intermediate PR that has no user value or independent consumer.

**Merge to trunk when:** All five commits are complete, the feature-level validation and close-out review pass, documentation is reconciled, release metadata validates for the final range, the repository PR template is complete, and GitHub reports the required `check` status successful.

**Depends on:** The completed Staff Workspace, CSV enrichment persistence, configurable player tables, migration v26, and the current Tauri dialog capability. No external service or new dependency is required.

#### Commit 1 — Save-owned shortlist persistence

**Status:** Completed

**Provisional commit:** `feat(storage): add save-owned staff shortlists`

**Work:** Add the additive SQLite schema that lets one active shortlist persist independently for each app save and survive snapshot replacement.

**Out of scope:**

- CSV parsing, commands, replacement behavior, staff queries, UI, and release metadata.
- Backfilling rows, snapshot foreign keys, import history, timestamps, names, or multiple list identities.

**Implementation packet:**

- Add migration v27 with the minimum save-owned entry table and the indexes needed by the planned exact job query. Keep raw Club Job text representable as either `-` or an empty string.
- Treat the table as user-owned enrichment rather than a derived cache: snapshot replacement must not cascade it, while app-save deletion must.
- Update the current architecture document only with schema facts that become true in this commit; keep planned UI behavior in this ledger until implemented.

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs` — define `STAFF_SHORTLIST_SCHEMA_SQL`, register the next monotonic migration, and add fresh, upgrade, constraint, and cascade coverage.
- `.wiki/ARCHITECTURE.md` — record the implemented save-owned table and its separation from snapshot-owned staff if the migration changes an existing documented persistence inventory.

**Behavior and data flow:**

- Opening a database at v26 applies v27 inside the existing migration transaction and creates `staff_shortlist_entries(save_id, staff_uid, preferred_job, club_job, coaching_qualifications)`.
- `(save_id, staff_uid)` prevents duplicate entries inside one save while allowing the same FM UID in different saves.
- `save_id REFERENCES saves(id) ON DELETE CASCADE` removes entries only when the app save is deleted. Loading or deleting snapshots does not touch them.
- `preferred_job` is non-empty after trimming. `club_job` and `coaching_qualifications` are non-null strings and can be empty. No CSV values are written by the migration.

**Ordered implementation steps:**

1. Add a migration test that expects the new table and cascade contract and confirm `./scripts/dev check` fails because v27 is absent.
2. Add the schema constant and register migration v27 after the current latest migration.
3. Add an upgrade fixture that applies through v26, inserts representative save and snapshot data, then proves v27 preserves that data and creates an empty shortlist table.
4. Add constraint and two-save cascade assertions, then run formatting and the full gate.

**Tests and proof:**

- RED: the migration registry and schema contract tests expect 27 migrations and the shortlist table, so the current v26 code fails for the missing schema.
- GREEN: a fresh database reaches v27; a v26 database upgrades without changing existing save or snapshot rows; duplicate `(save_id, staff_uid)` entries fail; the same UID can exist in two saves; deleting one save removes only its entries; deleting a snapshot preserves entries.

**Patterns to verify:**

- `CSV_ENRICHMENT_SCHEMA_SQL` for save-owned enrichment and `ON DELETE CASCADE` ownership.
- The latest v24-v26 upgrade tests and `registers_monotonic_migrations` for migration registration and upgrade proof.
- Current `saves.context_token` and snapshot ownership remain unchanged.

**Constraints and non-goals:**

- Use the existing rusqlite migration registry; add no ORM, repository layer, trigger, batch table, or import-history table.
- Do not add a foreign key to snapshot-owned `staff` and do not store current staff facts.
- Do not edit generated database files or run migrations against user data during implementation.

**Dependencies and sequencing:**

- Starts from migration v26 at planning time. Commit 2 depends on the table name and columns established here.

**Validation:** `./scripts/dev format`; `./scripts/dev check-rust`; `./scripts/dev check`; inspect the migration portion of `git diff --check`.

**Stop conditions:** Stop and replan if `main` gains another migration before this commit, if SQLite cannot enforce the save-owned cascade without snapshot coupling, or if current architecture documents reveal a conflicting owner for save-scoped staff enrichment.

**Review mandate:** Verify monotonic migration order, v26 upgrade safety, composite uniqueness, save-only cascade behavior, ability to preserve blank and `-` Club Job values, absence of snapshot or extracted-staff coupling, and no unrelated schema cleanup.

#### Commit 2 — Transactional Staff CSV replacement

**Status:** Active

**Provisional commit:** `feat(import): replace staff shortlists from CSV`

**Work:** Add a dedicated trusted Rust import command that parses the staff export, reconciles exact UIDs, and atomically replaces the active save's shortlist.

**Out of scope:**

- Staff list queries, job or unemployment filters, React upload UI, table rendering, and release metadata.
- Detection through the existing player `import_csv` command or changes to Youth Tracker and Moneyball limits or summaries.

**Implementation packet:**

- Add a staff-specific parser and import service under `features/csv_import` rather than widening player format detection. Reuse only narrow regular-file, bounded-read, and context-token primitives where doing so does not change player behavior.
- Require one each of `Unique ID`, `Preferred Job`, `Club Job`, and `Coaching Qualifications`; accept extra columns and ignore their values. Require semicolon delimiter, UTF-8, unique positive UIDs, non-empty Preferred Job, at most 1 MiB, and at most 10,000 data rows.
- Capture active save ID and token, current snapshot ID and token, and current staff UIDs under the database lock. Read and parse the file after releasing the lock. Reacquire the lock, start a transaction, and revalidate the exact context before any delete.
- Match by exact integer UID. Reject a zero-match import without mutation. For one or more matches, delete only `WHERE save_id = captured_save_id`, insert the matched entries with raw trimmed CSV strings, and commit. Return total, stored, and skipped counts.

**Files and responsibilities:**

- `src-tauri/src/features/csv_import/staff_shortlist.rs` — staff header contract, parser, bounds, prepared rows, context capture, zero-match guard, transactional replacement, summary, and focused tests.
- `src-tauri/src/features/csv_import/service.rs` — expose or extract only the narrow file/context helpers shared with the dedicated import while keeping both player format limits unchanged.
- `src-tauri/src/features/csv_import/error.rs` — make shared row-limit wording data-neutral if the staff parser reuses `TooManyRows`; preserve existing typed failures.
- `src-tauri/src/features/csv_import/commands.rs` — add `import_staff_shortlist_csv` and a path-based helper that proves parsing occurs outside the database lock.
- `src-tauri/src/features/csv_import/mod.rs` — register the new internal module and exports.
- `src-tauri/src/features/csv_import/fixtures/staff_shortlist.csv` — small synthetic fixture with fake UIDs, one employed row, one `-` row, and one blank Club Job row. Do not copy the developer's personal export.
- `src-tauri/src/features/csv_import/fixtures/README.md` — document the synthetic fixture's contract and provenance.
- `src-tauri/src/lib.rs` — register only the new Tauri command.

**Behavior and data flow:**

- The WebView supplies a path, but Rust validates that it is a regular `.csv` file, bounds bytes before UTF-8 decoding, validates the staff headers and rows, and never trusts CSV identity fields beyond UID.
- The prepared import contains only UID and the three CSV-owned strings. Person, Age, CA, PA, and Club never reach persistence.
- A transaction revalidates save and snapshot tokens, computes the matched subset from the captured staff UIDs, rejects zero matches, replaces the captured save's rows, and commits the summary.
- Syntax, duplicate UID, bounds, file, UTF-8, unsupported format, stale context, zero-match, and database failures return safe messages and preserve every pre-existing shortlist row.

**Ordered implementation steps:**

1. Add parser RED tests for the synthetic export, the supplied 2,180-row shape, blank and `-` preservation, duplicate UID, missing headers, blank Preferred Job, wrong delimiter, and the 10,001-row boundary.
2. Implement the minimum staff parser and prepared-row model without changing `parse_csv_with_row_limit` or `CsvImportFormat`.
3. Add persistence RED tests that seed an old shortlist and expect exact replacement, per-save isolation, skipped unknown UIDs, zero-match preservation, and rollback on malformed or stale imports.
4. Implement context capture, parse-outside-lock, context revalidation, zero-match guard, delete-and-insert transaction, and summary.
5. Register the command, add safe error mapping and serialization proof, then run affected and full validation.

**Tests and proof:**

- RED: parser and service tests fail for the missing staff contract and command.
- GREEN parser assertions cover the four required headers in arbitrary column order, ignored extras, CRLF, BOM handling consistent with existing CSV support, exact UID parsing, preserved trimmed strings, 2,180 accepted rows, 10,001 rejected rows, duplicate headers and UIDs, and malformed records.
- GREEN persistence assertions cover full replacement, omitted old rows removed, matched values stored, unknown UIDs counted as skipped, active save isolation, no staff-row mutation, zero matches preserving the old list, transaction rollback, and stale save or snapshot context preserving the old list.
- Command serialization returns camelCase `{ totalStaff, storedStaff, skippedStaff }` and never exposes a local path or database detail in an error.

**Patterns to verify:**

- `capture_import_context`, `prepare_csv_import_for_expected_format`, `persist_csv_import`, and `revalidate_import_context` for lock and stale-context order.
- `parse_youth_tracker_with_row_limit` for BOM, delimiter, header, duplicate UID, and row-number behavior, while deliberately using a separate 10,000-row staff limit.
- Existing CSV fixtures for synthetic-data documentation and the command helper tests for safe serialization.

**Constraints and non-goals:**

- Keep the existing 1 MiB file cap and use the existing `csv` crate; add no dependency or client-side parser.
- Do not accept commas, aliases not demonstrated by an FM staff export, zero matching rows, or blank Preferred Job values.
- Do not change player import persistence or make staff data part of `CsvImportSummary`.
- The import service must never hold the database mutex during native file I/O or CSV parsing.

**Dependencies and sequencing:**

- Requires Commit 1's table. Commit 3 consumes the persisted rows and command summary contract.

**Validation:** `./scripts/dev format`; `./scripts/dev check-rust`; `./scripts/dev check`; inspect `git diff --check` and confirm the developer-supplied CSV is not tracked.

**Stop conditions:** Stop and replan if the supplied export cannot pass the bounded parser without accepting a broader dialect, if current snapshots cannot provide stable staff UIDs, if a valid import can delete before every fallible validation completes, or if reuse would require changing existing player CSV behavior.

**Review mandate:** Verify trusted file bounds and UTF-8 handling, 2,180-row compatibility, duplicate and zero-match safety, exact UID reconciliation, parse-outside-lock ordering, context-token revalidation, atomic replacement and rollback, save isolation, safe error text, and non-authoritative columns never reaching storage.

#### Commit 3 — Bounded shortlist staff queries

**Status:** Pending

**Provisional commit:** `feat(staff): query CSV-backed staff shortlists`

**Work:** Expose bounded current-snapshot Staff Shortlist pages, saved Preferred Job options, exact job filtering, and the derived unemployment filter through Rust-owned queries and typed IPC.

**Out of scope:**

- React route, table layout, upload dialog, general Staff Search filters, profile enrichment, and release metadata.
- Client-side UID reconciliation, filtering, sorting, or paging.

**Implementation packet:**

- Add a Shortlist query scope through shared Staff page assembly. Keep Search and My Staff behavior and command shapes stable, and do not copy the full SQL query into a second implementation.
- Join the active save's `staff_shortlist_entries` to the effective current snapshot's `staff` by UID. Return current staff fields plus optional `StaffShortlistMetadata { preferred_job, club_job, coaching_qualifications }` for shortlist rows.
- Add scope-validated sortable fields for the three CSV columns. Keep numeric `dynamic_values` limited to attributes and role scores; the CSV strings are always selected as typed metadata for the bounded page.
- Preserve the existing trusted dynamic role-score field path in Shortlist scope so Commit 4 can request one mapped score or the six Coach scores and can order direct matches by the mapped score before paging.
- Apply exact Preferred Job and `trim(club_job) IN ('', '-')` unemployment predicates before total, ordering, limit, and offset. Return distinct Preferred Job options from all saved entries for the active save, not only the current page or current filter.
- Return `no_current_snapshot` when no effective snapshot exists, `no_shortlist` when the active save has no entries, `ready` with zero rows for a valid shortlist whose current snapshot or selected filters yield no matches, and existing states unchanged for other scopes.

**Files and responsibilities:**

- `src-tauri/src/features/staff/query.rs` — Shortlist scope constraints, save-owned join, filters, typed metadata, job options, scoped sorting, count and page queries, and integration tests.
- `src-tauri/src/features/staff/commands.rs` — `list_staff_shortlist` request validation and `StaffShortlistPageDto`; serialize `no_shortlist`, metadata, counts, and options without changing existing command names.
- `src-tauri/src/features/staff/metrics.rs` — define and validate the three shortlist-only sort fields while keeping Search and My Staff metric acceptance unchanged.
- `src-tauri/src/lib.rs` — register `list_staff_shortlist`.

**Behavior and data flow:**

- The command validates offset, limit, direction, requested numeric fields, optional Preferred Job, unemployment boolean, and sort field for Shortlist scope.
- Rust resolves `(snapshot_id, save_id)` from the active save's current snapshot. The shortlist join then selects only rows present in both the saved upload and current snapshot.
- Preferred Job options come from the save-owned entries in case-insensitive display order. The page query applies any selected exact job plus unemployment before `COUNT` and the stable UID tiebreaker.
- Search and My Staff return `shortlist: null`; Shortlist returns non-null metadata on every row. The existing `/staff/$uid` detail query remains current-snapshot-only and unchanged.

**Ordered implementation steps:**

1. Add RED query tests for the no-shortlist state, exact UID join, per-save isolation, persistence across current-snapshot replacement, and missing-current-UID behavior.
2. Add RED tests for All jobs, exact Preferred Job, blank and `-` unemployment, employed exclusion, combined filters, totals before paging, and distinct option ordering.
3. Add the typed metadata and Shortlist scope to shared query assembly with scope-specific join and bind construction.
4. Add scoped CSV string sorts and prove Search and My Staff reject them instead of emitting SQL against a missing alias.
5. Prove Shortlist accepts validated dynamic role-score requests and descending role-score sort while rejecting unknown role IDs before SQL construction.
6. Add the command DTO and serialization tests, register it, and rerun existing Staff query and command tests through the gate.

**Tests and proof:**

- RED: current Staff query tests cannot represent Shortlist and the new behavioral tests fail for the missing state and command.
- GREEN: the same UID resolves to the active save's own metadata; loading a later current snapshot updates current name, CA, club, and role score while preserving CSV metadata; an absent UID remains stored but is not returned.
- Job filtering is equality, case-insensitive only for comparison, and never contains matching. All jobs leaves the predicate absent.
- Only unemployed includes raw blank and `-`, excludes any named Club Job, and composes with Preferred Job before totals and paging.
- CSV string sorts are stable with UID tiebreaking. Search and My Staff regression tests retain their states, filters, scopes, totals, and sort behavior.
- A requested shortlist role score is returned in `dynamic_values`, a role-score sort runs before `LIMIT` and `OFFSET` with UID tiebreaking, and multiple Coach score fields remain a bounded page projection.

**Patterns to verify:**

- `list_staff`, `map_staff`, and `StaffPageState` for current-snapshot resolution, bounded pages, stable order, typed mapping, and empty states.
- `list_my_staff_uids` and My Staff's `EXISTS` constraint for save-scoped filtering without moving domain work into React.
- `parse_requested_fields` and metric SQL expressions for trusted identifiers and parameterized values.

**Constraints and non-goals:**

- Keep `DEFAULT_PAGE_LIMIT = 50` and `MAX_PAGE_LIMIT = 200`; return no unbounded UID or row list.
- Bind every CSV value. Do not interpolate Preferred Job or Club Job text into SQL.
- Do not add CSV strings to numeric `dynamic_values`, expose shortlist-only columns in Search or My Staff, or recalculate role scores.
- Do not delete saved entries merely because the current snapshot lacks their UIDs.

**Dependencies and sequencing:**

- Requires Commits 1 and 2. Commit 4 consumes the exact DTO, state, options, metadata, and filter semantics.

**Validation:** `./scripts/dev format`; `./scripts/dev check-rust`; `./scripts/dev check`; inspect `git diff --check`.

**Stop conditions:** Stop and replan if the shared query cannot isolate shortlist joins without changing Search or My Staff results, if scope validation cannot prevent invalid CSV-only sorts, if filtering occurs after paging, or if snapshot replacement deletes save-owned entries.

**Review mandate:** Verify source-of-truth separation, active-save and current-snapshot joins, per-save isolation, absent-UID persistence, exact parameterized filters, unemployment semantics, count/page parity, bound limits, trusted dynamic role requests, mapped-score ordering before paging, scoped sort validation, stable ordering, and Search/My Staff regression coverage.

#### Commit 4 — Staff Shortlist workspace

**Status:** Pending

**Provisional commit:** `feat(staff): add shortlist workspace`

**Work:** Add the third Staff workspace, replacement-aware upload interaction, adaptive Preferred Job score projections, URL-backed shortlist filters, and an independent configurable virtual table.

**Out of scope:**

- Staff profile enrichment, new score formulas, a Manager score, one combined Coach score, general Search filters, list editing, extra navigation items, and release metadata.
- A custom file browser, header mapper, or client-side CSV inspection.

**Implementation packet:**

- Extend the existing Staff tabs and route with `shortlist`, keeping one top-level Staff navigation item and one full-height panel active at a time. Add independent All jobs base-sort URL keys plus `preferredJob` and `unemployedOnly`; preserve Search and My Staff state when switching tabs.
- Add frontend API, query options, keys, types, and IPC test mocks for `list_staff_shortlist` and `import_staff_shortlist_csv`. Query keys must include the visible save and snapshot IDs as cache identity, selected job, unemployment, page, sort, and requested numeric fields. Rust still resolves the authoritative active context.
- Add a shortlist-only metric catalog and `staff-shortlist` Zustand layout. Bump persisted storage to version 3 while retaining valid version 2 Search, Squad, Staff Search, and My Staff layouts. The saved All jobs layout remains configurable and defaults to Name, Age/DOB, Nation, current Club, CA, PA, Preferred Job, Club Job, Coaching Qualifications, and the existing staff role scores.
- Add a pure presentation resolver for the approved Preferred Job map. Direct matches derive the fixed basic and CSV columns plus one score and descending effective sort. Coach derives the same fixed columns plus exactly six outfield coaching scores without choosing one as the score sort. Manager and unknown values derive no score column and use CA descending. All jobs returns the saved layout and base sort.
- Extend the results panel with a Shortlist adapter and current states. Put the Preferred Job native select, labeled Only unemployed checkbox or switch, and Upload or Replace CSV action above the table without creating a second vertical scroll owner. Keep layout mutation controls scoped to All jobs so a contextual projection cannot rewrite the persisted layout.
- Add a dedicated import modal. Existing-list mode must show a visible warning before Browse is available, explain that successful import replaces this save's complete shortlist, and label the primary action **Choose replacement CSV**. No-list mode uses **Choose CSV** without destructive language.
- Scope visible import state to the current save and snapshot. On success, show total, stored, and skipped counts, reset shortlist filters to All jobs and unemployment off, invalidate all Staff list queries, and keep the modal outcome available until dismissal. On failure, keep existing data and show the safe backend message.

**Files and responsibilities:**

- `src/app/routes/staff.tsx` — validated Shortlist URL state, base and effective sort selection, loader, tab composition, filter reset after import, query invalidation, and profile navigation.
- `src/app/routes/staff.test.tsx` — route, tab, URL, empty, filter, contextual table, replacement, and state-restoration behavior.
- `src/features/staff/components/staff-workspace-tabs.tsx` — third accessible tab and keyboard cycling.
- `src/features/staff/components/staff-search-results-panel.tsx` — Shortlist scope adapter, controls, contextual columns, effective sort status, states, CSV cells, and existing table reuse; rename only if the third scope makes the current name materially misleading.
- `src/features/staff/api/fetch-staff.ts`, `staff-query-options.ts`, and `staff-keys.ts` — bounded Shortlist IPC and complete cache identity.
- `src/features/staff/types/staff-summary.ts` and Staff sort or URL utilities — optional typed shortlist metadata, `no_shortlist`, scoped metrics, filters, and validated view state.
- `src/features/staff/utils/staff-metrics.ts` and `src/utils/staff-table-layout.ts` — shortlist-only string metrics, the fixed contextual basic and CSV column IDs, and default All jobs columns without changing Search or My Staff availability.
- `src/features/staff/utils/staff-shortlist-presentation.ts` and `.test.ts` — exact direct mappings, the six-score Coach group, Manager and unknown fallback, derived contextual columns, effective sort, and All jobs restoration.
- `src/stores/use-player-table-store.ts` and `src/stores/use-player-table-store.test.ts` — independent `staff-shortlist` layout and lossless version 2 to 3 migration.
- `src/features/csv-import/api/import-staff-shortlist-csv.ts` and `src/features/csv-import/types/staff-shortlist-import-summary.ts` — dedicated typed import boundary.
- `src/features/csv-import/components/staff-shortlist-import-modal.tsx` and `.test.tsx` — warning-first file selection, context-aware pending/success/error state, focus and button semantics.
- `src/testing/setup.ts` and `src/testing/staff-ipc-mock.ts` — realistic command mocks that filter before paging and support replacement outcomes.
- `e2e/smoke.spec.ts` — one browser product path for entering Shortlist, applying both filters, verifying direct-job, Coach, Manager, and All jobs column or sort transitions, using the virtual table, and opening a profile; native file-picker behavior remains manual.
- `.wiki/DESIGN.md` and `.wiki/ARCHITECTURE.md` — record only the implemented Staff tab, destructive replacement warning, table state ownership, query boundary, and persistence flow.

**Behavior and data flow:**

- Route validation selects Search by default, preserves independent state for all three tabs, and passes only canonical Preferred Job text or no selection plus a boolean unemployment flag to query options.
- `no_shortlist` renders a clear upload CTA. `no_current_snapshot` retains the Load Data guidance. A ready shortlist with zero current matches explains that imported UIDs are unavailable; a filter-empty result tells the user to choose All jobs or turn off Only unemployed.
- The job select options come from the active save's persisted response and start with All jobs. A selection always triggers the exact backend filter and then supplies the presentation resolver's effective columns, requested role fields, and sort to the bounded query.
- All jobs reads the saved `staff-shortlist` layout and base sort. Direct mappings request one role score and sort it descending. Coach requests only the six approved outfield coaching scores and never adds Goalkeeping, Fitness, or Set Piece. Manager and unknown values request no contextual role score and sort by CA descending.
- The shared table reads current staff cells and role-score `dynamicValues`; three shortlist cells read typed CSV metadata. Contextual modes do not write to the saved layout or base sort, and returning to All jobs restores both exactly.
- Import runs through the native dialog and Rust command. Success refreshes the page and clears stale filters; failure or context change cannot overwrite visible state for the new save or snapshot.

**Ordered implementation steps:**

1. Add route and component RED tests for the Shortlist tab, no-list upload prompt, All jobs default, exact job request, unemployment request, and independent URL base-sort state.
2. Add frontend DTOs, API, query keys/options, test mocks, and the Shortlist route loader until the read-only table skeleton is GREEN.
3. Add shortlist metrics, cells, default All jobs layout, and the version 3 persisted-store migration with regression proof for every existing layout.
4. Add parameterized RED tests for every direct Preferred Job mapping, the exact six-score Coach group, Manager and unknown fallback, descending mapped-score sort, neutral Coach fallback, and restoration after returning to All jobs. Implement the pure presentation resolver and wire its effective fields and sort into the query adapter.
5. Add the import modal RED tests for pre-selection replacement warning, primary-action wording, pending lockout, safe error, success counts, context change, invalidation, and filter reset.
6. Implement the modal and route composition using the existing dialog and Modal/Button primitives, then add all empty and error states.
7. Add the browser smoke path, update current-state architecture and design facts, format, and run focused, app, full, and browser validation.

**Tests and proof:**

- RED: `/staff?view=shortlist` currently normalizes to Search and no shortlist command, controls, layout, or modal exists.
- GREEN route tests prove tab keyboard order, valid URL round trips, independent base-sort state, All jobs, exact Preferred Job, Only unemployed, current profile navigation, no-list and no-current-snapshot states, filtered-empty copy, and one-scroll-owner table composition.
- Presentation tests prove every direct label resolves to exactly one current role ID and descending score sort; Coach resolves to exactly the six Attacking, Defending, and Possession Technical or Tactical scores with no chosen role-score sort; Manager and unknown labels resolve to fixed columns and CA descending; and returning to All jobs restores the untouched saved layout and base sort.
- Query-adapter tests prove contextual requests fetch only the score fields that can render, apply the effective sort on the Rust side before paging, compose with Only unemployed, and never add Goalkeeping, Fitness, or Set Piece scores for Coach.
- Modal tests prove an existing shortlist shows replacement consequences before selection, the button says **Choose replacement CSV**, pending state prevents duplicate work and closure, successful counts render, failures preserve the table, and a context change suppresses stale outcomes.
- Store tests start from a version 2 payload with customized layouts and prove version 3 preserves all four while adding the default `staff-shortlist` layout.
- Query mock and smoke tests apply filters before count and paging and prove the displayed CSV values and current staff values come from separate fields.

**Patterns to verify:**

- `StaffWorkspaceTabs` for ARIA tab, roving focus, Home/End, and panel linkage.
- `StaffSearchResultsPanel` plus `ConfigurableVirtualizedTable` for bounded paging, fixed rows, full-height containment, column controls, and row activation.
- `STAFF_ROLE_METRICS` and the Rust `STAFF_ROLES` catalog for exact score IDs; do not infer IDs from labels at runtime.
- `SquadCsvImportModal` and `useCsvImport` for dialog feedback and context lifecycle, while using a dedicated staff command and summary instead of widening player types.
- `usePlayerTableStore` version 2 migration and the Search/My Staff URL state tests for lossless state evolution.
- `.wiki/DESIGN.md` Staff workspace and table rules for tokens, native labeled controls, focus visibility, empty states, and no nested vertical scrolling.

**Constraints and non-goals:**

- Reuse current UI primitives, tokens, table components, Tauri dialog permission, TanStack Router, TanStack Query, and Zustand. Add no dependency or top-level navigation item.
- Use a native labeled select and semantic checkbox or switch. Preserve keyboard operation, visible focus, modal Escape behavior when not pending, and clear live feedback.
- Never parse the CSV, filter an unbounded row list, or join UIDs in React.
- Do not expose shortlist-only columns in Search or My Staff or overwrite their persisted layouts.
- Do not mutate the saved All jobs layout or base sort when deriving a direct-job, Coach, Manager, or unknown presentation. Do not map Manager to Assistant Manager.

**Dependencies and sequencing:**

- Requires Commits 1-3 and their stable command contracts. Commit 5 depends on the complete user-visible range and validation evidence from this commit.

**Validation:** `./scripts/dev format src e2e`; `./scripts/dev test src/app/routes/staff.test.tsx src/features/staff/utils/staff-shortlist-presentation.test.ts src/features/csv-import/components/staff-shortlist-import-modal.test.tsx src/stores/use-player-table-store.test.ts`; `./scripts/dev check-app`; `./scripts/dev check`; `CI=1 ./scripts/dev smoke`; inspect `git diff --check`.

**Stop conditions:** Stop and replan if route state cannot restore the exact All jobs layout and base sort after contextual filtering, if any observed Preferred Job lacks the approved direct, Coach, or Manager behavior, if contextual fields require client-side score sorting, if the modal can start replacement without first presenting the warning, if store migration drops an existing layout, if the shared table requires client-side filtering or an unbounded response, or if native dialog permissions must broaden.

**Review mandate:** Verify the complete exact job map, six-score Coach exclusion set, Manager and unknown fallback, descending mapped-score ordering before paging, contextual requested-field bounds, All jobs layout and sort restoration, warning timing and wording, stale-context handling, URL and cache identity, filters before paging, Shortlist-only metric scope, lossless layout migration, accessible tabs/select/toggle/modal, accurate empty and error states, one vertical scroll owner, profile activation, and no frontend trust-boundary regression.

#### Commit 5 — Minor release preparation

**Status:** Pending

**Provisional commit:** `chore(release): prepare version 0.6.0`

**Work:** Prepare and validate the complete compatible release range for the Staff Shortlist capability before PR publication.

**Out of scope:**

- Feature implementation, new fixes, release publication, tagging, pushing, PR creation, or merging.
- Assuming `0.6.0` or `v0.5.2` remains correct if trunk or the reachable tag set changes.

**Implementation packet:**

- Reinspect the latest reachable `v*` tag and every user-visible change from that tag through the branch. Keep `minor` only if the complete range remains a compatible capability; stop for a developer decision if compatibility changes.
- If `v0.5.2` remains the latest release boundary, set all five durable version owners to `0.6.0`, preserve `## [Unreleased]`, and add one dated Keep a Changelog section covering the complete range, including any already-merged unreleased changes.
- Set `release-preparation.json` to the same version and `minor` intent and increment its positive sequence once. Do not regenerate Cargo.lock; edit only the root `app` package entry.

**Files and responsibilities:**

- `package.json`, `src-tauri/Cargo.toml`, root `app` entry in `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, and `bridge/FmDataBridge.csproj` — exact durable version owners.
- `CHANGELOG.md` — one complete dated section for the full latest-tag-to-branch user-visible range while preserving Unreleased.
- `release-preparation.json` — exact version, `minor` intent, and one sequence increment.
- `.wiki/features/active/staff-shortlist.md` — record a changed release boundary or intent only if execution evidence disproves this provisional packet.

**Behavior and data flow:**

- Repository metadata presents one consistent version. The release validator reads the latest tag, version owners, changelog section, and release-preparation authorization and returns matching machine-readable release evidence.
- This commit authorizes no remote action. Verified-main packaging and publication remain downstream of PR merge and required checks.

**Ordered implementation steps:**

1. Read the latest reachable tag, `git log`, and complete range diff; confirm the compatible minor intent and calculate the target version.
2. Add a RED proof by running `./scripts/dev release-metadata <latest-tag> minor` before metadata changes and confirm it reports the expected missing or mismatched preparation.
3. Update the five version owners, complete changelog section, and release-preparation record without regenerating the lockfile.
4. Run release metadata validation, inspect the root Cargo lock entry and exact diff, then run the full gate.

**Tests and proof:**

- RED: the release validator rejects the old version or absent dated section for minor intent.
- GREEN: `./scripts/dev release-metadata <latest-tag> minor` reports one consistent target version, matching changelog section, and release-required state; only the root `app` Cargo lock entry changes.
- `./scripts/dev check` passes with the prepared version and release record.

**Patterns to verify:**

- `.agents/skills/create-pr/SKILL.md` for range-based release classification and the five version owners.
- Current `CHANGELOG.md` Keep a Changelog format and `release-preparation.json` sequence rules.
- `./scripts/dev release-metadata` output rather than inferred version consistency.

**Constraints and non-goals:**

- Do not write a changelog section that covers only the current PR when other user-visible commits exist after the latest tag.
- Do not change dependency lock entries, publish artifacts, call GitHub, or treat the provisional version as authoritative after base drift.
- Keep the PR template's Release intent to exactly one checked `minor` box when publication is later authorized.

**Dependencies and sequencing:**

- Requires Commit 4 and the final intended user-visible range. Run before feature close-out and PR publication; recalculate after any later corrective commit that changes release classification or notes.

**Validation:** `./scripts/dev release-metadata v0.5.2 minor` only while `v0.5.2` remains the verified latest tag; `./scripts/dev check`; inspect `git diff --check`, all five version owners, the dated changelog section, and the release-preparation sequence.

**Stop conditions:** Stop and replan if the latest tag is no longer `v0.5.2`, `main` contains additional unreleased user-visible changes not covered by the planned section, the range is not an unambiguous compatible capability, or the release validator reports a target other than the prepared metadata.

**Review mandate:** Verify full-range release classification, exact version agreement, one dated changelog section with complete user-visible coverage, preserved Unreleased heading, root-only Cargo lock edit, one sequence increment, validator evidence, and absence of remote or generated artifact changes.

## Active work

**PR:** PR 1 — CSV-backed Staff Shortlist workspace

**Commit:** Commit 2 — Transactional Staff CSV replacement

### RED proof

Add parser and service tests for the dedicated staff CSV contract, including valid semicolon exports, exact UID reconciliation, preserved blank and `-` Club Job values, rejection without replacing an old shortlist, and transactional replacement. The current player-only import path must fail those assertions because no staff importer exists.

### Expected outcome

A trusted Rust command validates and parses a bounded staff CSV outside the database lock, then replaces only the captured active save's matching shortlist entries in one context-checked transaction and returns total, stored, and skipped counts.

### Explicit exclusions

No Staff Shortlist query scope, Preferred Job or unemployment filtering, React upload UI, table rendering, or release metadata belongs in Commit 2.

## Discoveries and replanning

- Planning inspected the supplied 2,180-row export and found that the existing 1,000-player row cap is incompatible. The feature therefore uses a separate 10,000-row staff cap and leaves player imports unchanged.
- The developer confirmed full replacement per upload, per-save persistence across restarts and snapshots, All jobs plus exact filtering, and blank or `-` Club Job as the unemployment rule.
- The developer then approved contextual role-score presentation inside Commit 4: direct jobs show and sort one score descending; Coach shows six outfield coaching scores without choosing one; Manager shows no role score and uses CA descending; All jobs restores its saved layout and sort.
- The current staff `job_id` is contract metadata and the 20 role scores are calculated suitability. Neither can substitute for CSV Preferred Job or Club Job.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Commit 1 — Save-owned shortlist persistence | Pending record | Migration v27 adds save-owned shortlist rows, uniqueness, exact-job index, and upgrade/cascade tests | Cleared after 1 fix round | None |

## Final validation

- Run `./scripts/dev format` and confirm the exact feature diff remains focused.
- Run `./scripts/dev test` and record the full frontend count.
- Run `./scripts/dev check` and record Biome, TypeScript, secretlint, Rust formatting, clippy, and Rust test evidence.
- Run `CI=1 ./scripts/dev smoke` and record the complete Staff Shortlist browser path result.
- Run `./scripts/dev release-metadata <verified-latest-tag> minor` and record the exact version, target tag, changelog section, and release-required result.
- Run `git diff --check` for the exact feature range and confirm the developer-supplied CSV and local database artifacts are not tracked.
- In an assembled Tauri app, verify the native file picker, pre-selection replacement warning, successful 2,180-row import, skipped-row feedback, replacement, app restart persistence, Load Data persistence, save switching, All jobs restoration, one direct Preferred Job with one descending score, Coach with exactly six unforced scores, Manager with no role score and CA descending, Only unemployed for blank and `-`, combined filters, table scrolling, keyboard controls, modal focus and Escape behavior, and row activation at 1280x800 and 1600x900.
- Confirm invalid, duplicate-UID, zero-match, stale-context, oversized, and wrong-dialect imports preserve the prior shortlist.
- Run the required fresh feature-complete review across the exact recorded implementation commits. No CRITICAL, HIGH, or MEDIUM finding may remain before documentation reconciliation unless the developer explicitly accepts it.
- Reconcile `.wiki/ARCHITECTURE.md`, `.wiki/DESIGN.md`, `.wiki/TODO.md`, `CHANGELOG.md`, and this ledger against implemented behavior, then archive the condensed record under `.wiki/features/completed/`.

## Documentation impact

Planning creates this active ledger and adds one feature-level Active entry to `.wiki/TODO.md`. Implementation is expected to update `.wiki/ARCHITECTURE.md` for the save-owned schema and Rust-owned import/query flow, `.wiki/DESIGN.md` for the third Staff workspace, adaptive contextual columns, sort restoration, and replacement-warning interaction, and release metadata for the final verified range. No ADR or debug report is planned because the feature uses established ownership and failure patterns; create one only if implementation exposes a genuinely new durable decision or reusable failure mode.
