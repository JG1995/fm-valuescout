# CSV Enrichment Persistence and Derived Statistics

## Status

Active

## Intent

Persist the supported Youth Tracker and Moneyball CSV data that the memory pipeline cannot supply, without allowing a CSV to create or replace a memory-backed player. Transport the implemented Moneyball statistic catalogue and calculations from the retired `fm-valuescout-react` repository into this repository so the data contract remains maintained here.

## User-visible behavior

- The Dashboard imports one supported Youth Tracker or Moneyball CSV into the active app save after a current memory snapshot has been loaded.
- Import matching remains exact numeric FM UID matching against the active save's current snapshot. Rows whose UID is not in that snapshot are reported as skipped and do not create players or enrichment rows.
- A successful Youth Tracker import stores reported all-time career appearances, goals, assists, and international caps for matching players. Academy views use those values for career columns, aggregates, and graduate status.
- A successful Moneyball import stores the latest per-season asking price, starts, substitute appearances, minutes, exported statistics, and calculated statistics for matching players. This feature does not add a Moneyball analytics screen.
- Re-importing either format replaces that format's stored values only for matching UIDs present in the selected file. Existing enrichment for every other player remains unchanged.
- Youth Tracker and Moneyball enrichment coexist for the same player. Importing one format never clears the other format.
- Enrichment survives Load Data snapshot replacement and app restart, remains isolated by app save, and is deleted when its owning save is deleted.
- The import result reports the detected format, total parsed players, stored players, and skipped players. It does not expose or retain the selected path or file contents.
- Invalid, stale, or failed imports write nothing. Native picker cancellation remains a no-op.

## Invariants

- The current memory snapshot is the player identity authority. A CSV row can supplement an existing current-snapshot UID but can never create a player.
- Memory-backed identity, ability, contract, club, position, attribute, value, and foot data always win. CSV copies of those facts are neither persisted nor overlaid.
- Both CSV foot representations are excluded. The bridge already reads and decodes the numeric left- and right-foot attributes before deriving `preferredFoot`; CSV text is redundant even though the individual numeric strengths are not currently emitted in the dump.
- Youth Tracker career values and Moneyball latest-season values have separate storage and replacement lifecycles.
- Import replacement is per format and per matching player. Rows omitted from a later file are not deleted or changed.
- A matched player's newest row replaces the complete stored row for that format, including nullable values. This prevents stale fields from surviving an authoritative later row.
- The active save and current snapshot captured before file I/O must still be active and current in the write transaction. Otherwise the entire import fails as stale.
- Parsing stays outside the SQLite mutex. Revalidation and all writes occur atomically after parsing.
- Missing values remain `null`. Zero remains a real reported value. Derived values are not invented when their required inputs are absent or their denominator is zero.
- Snapshot replacement does not delete enrichment. Save deletion cascades to all enrichment.
- No application read model may use enrichment to manufacture a general player result. Academy may retain career values for an already tracked membership after that player becomes unresolved because the UID was memory-verified when imported.
- No required validation step depends on a running FM26 process. Deterministic Rust tests prove snapshot-replacement behavior; a live scan is optional confidence only.

## Non-goals

- Historical Moneyball seasons, import snapshots, season selection, trends, or comparisons.
- Inferring a season identifier from a file name, import date, or in-game date.
- A Moneyball table, filter, chart, score, or player-profile presentation.
- Persisting source paths, complete CSV files, raw CSV rows, file names, or file hashes.
- Importing CSV-only players, name-based reconciliation, fuzzy matching, or manual UID mapping.
- Persisting Youth Tracker best position, position text, preferred foot, personality label, attributes, hidden attributes, CA, PA, height, or other memory-backed fields.
- Persisting Moneyball identity, nationality, club, division, position, age, height, foot text, CA, PA, transfer value, wage, or contract expiry.
- Changing the bridge dump schema or exposing its already-read numeric foot strengths.
- Requiring a live FM26 session to validate this feature.
- Changing the existing CSV size, row, encoding, regular-file, extension, duplicate-UID, or dialect boundaries.
- Adding a new dependency, ADR, or generalized import framework.

## Current-state map

- Relevant components:
  - Rust `src-tauri/src/features/csv_import/` parses complete Youth Tracker and Moneyball rows and currently returns a non-mutating UID reconciliation summary.
  - React `src/features/csv-import/` and `src/app/routes/index.tsx` own the Dashboard picker, preview state, safe result copy, and context guards.
  - Rust `src-tauri/src/features/academy/` and React `src/features/academy/` already carry nullable career-stat fields and unavailable UI states.
  - `bridge/Extraction/PlayerIdentityReader.cs` reads both foot attributes numerically, decodes them, and emits only a derived preferred-foot category.
- Data model:
  - `players` is keyed by `(snapshot_id, uid)`. Player rows are memory-owned and are replaced when Load Data replaces a save's current snapshot.
  - `saves` supplies the stable ownership boundary used by Planner and Academy state.
  - Academy memberships are save-scoped and retain UID plus last-known name across snapshot replacement.
- Persistence and migrations:
  - SQLite migration v16 is current.
  - Snapshot deletion cascades through `players`, so adding CSV columns or player foreign keys there would erase enrichment on every successful Load Data.
  - Rust-owned migrations and queries follow ADR 0015; file parsing and IPC validation follow ADR 0014.
- Existing behavioral assumptions:
  - CSV parsing is strict, bounded to a regular UTF-8 `.csv` file of at most 1 MiB and 1,000 rows, rejects duplicate UIDs, and preserves missing data as `null`.
  - Preview captures the active save, current snapshot, and current player UIDs under a short lock, parses outside the lock, then revalidates context.
  - Academy currently hard-codes career appearances, goals, assists, international caps, and graduate status to unavailable.
- Architectural seams:
  - The existing parsed row models are the trust-boundary inputs to canonicalization and persistence.
  - The active-save/current-snapshot query is the eligibility gate.
  - Academy member projection is the first read consumer for Youth Tracker enrichment.
- External source contract:
  - The pinned legacy source is commit `366aa20b5282d3a63c94854ddb8da6992462b0c5` of `JG1995/fm-valuescout-react`.
  - Its [`schema.rs`](https://github.com/JG1995/fm-valuescout-react/blob/366aa20b5282d3a63c94854ddb8da6992462b0c5/src-tauri/src/data/schema.rs) defines 198 canonical columns: 22 base columns and 176 performance-stat columns.
  - Its [`row.rs`](https://github.com/JG1995/fm-valuescout-react/blob/366aa20b5282d3a63c94854ddb8da6992462b0c5/src-tauri/src/data/row.rs) and [`derived.rs`](https://github.com/JG1995/fm-valuescout-react/blob/366aa20b5282d3a63c94854ddb8da6992462b0c5/src-tauri/src/data/derived.rs) define the implemented calculations. These files, not older speculative documentation, are the transport source of truth.
- Project validation commands:
  - `./scripts/dev test [target...]`
  - `./scripts/dev check`
  - `./scripts/dev smoke`
- Primary risks:
  - An incomplete metric catalogue or incorrect formula would silently persist misleading Moneyball values.
  - Per-player replacement can intentionally leave players at different import timestamps; UI and later history work must not pretend the table is one atomic season snapshot.
  - A stale-context or partial-write defect could enrich the wrong save or leave half an import committed.
  - Conflating snapshot-owned players with save-owned enrichment would lose imported data on Load Data.

## Feature architecture

### Ownership and storage

- Migration v17 adds `player_youth_career_stats`, keyed by `(save_id, player_uid)`, with nullable career appearances, international caps, career goals, career assists, and a row-level `imported_at_utc` timestamp.
- Migration v17 adds `player_moneyball_stats`, keyed by `(save_id, player_uid)`, with a normalized nullable asking-price shape, nullable starts, substitute appearances, and minutes, a non-null canonical statistics JSON object, and a row-level `imported_at_utc` timestamp.
- Both tables reference `saves(id) ON DELETE CASCADE`. They intentionally do not reference `players`, because a player row belongs to a replaceable snapshot rather than the stable save.
- The Moneyball statistics object contains all 176 canonical performance keys from the pinned legacy schema, each with a numeric value or `null`. Persisted keys keep the exact pinned schema spelling, including identifiers such as `np-xg` and `xg-op`; CSV export-header spelling remains a parser concern.
- The asking-price shape preserves missing, single price, price range, and not-for-sale without reusing the memory-owned transfer-value field.
- No season key or import-parent table is added. A later history feature must introduce an explicit season identity and copy or migrate the stable canonical metric contract into historical rows.

### Field ownership

| Source | Persisted by this feature | Deliberately discarded |
| --- | --- | --- |
| Youth Tracker | All-time career appearances, international caps, all-time goals, assists | UID except as key; identity; age; nationality; positions and best position; CA/PA; height; determination; personality label; preferred foot; visible and hidden attributes |
| Moneyball base data | Asking price; starts; substitute appearances; minutes | UID except as key; name; nations; club; division; position; age; height; left/right foot; CA/PA; transfer value; wage; expiry; internal source marker |
| Moneyball performance data | All 176 canonical exported-or-calculated statistic values under the exact pinned schema keys | Moneyball CSV header spelling and duplicate source formatting |

### Moneyball calculation contract

- Keep a populated exported value authoritative unless the pinned implementation explicitly defines a fallback. Calculate only the absent partner or aggregate required by the pinned canonical schema.
- Calculate per-90 values as `value * 90 / minutes` when minutes are positive.
- Calculate missing integer totals from per-90 values as the rounded result of `per90 * minutes / 90` when minutes are positive.
- Calculate completion and success ratios from their numerator and denominator when the denominator is positive: penalties scored, crosses, open-play crosses, passes, tackles, pressures, headers, and penalties saved.
- Calculate clean-sheet ratio from clean sheets divided by starts plus substitute appearances when total appearances are positive.
- Use an exported save percentage only when it is greater than zero, normalized from percentage points to a ratio. For zero, negative, or missing percentages, calculate saves divided by saves plus goals conceded when possible, matching the pinned implementation.
- Calculate game-win ratio from wins divided by wins plus draws plus losses.
- Calculate minutes per goal, assist, and goal-or-assist only when the relevant event count is positive.
- Preserve the legacy attacking, creativity, transition, defensive, aerial, goalkeeping, discipline, and match-impact catalogue exactly. Do not add metrics mentioned only in legacy prose when they are absent from the implemented schema.

### Import transaction

1. Capture active `save_id`, current `snapshot_id`, and the current snapshot's UID set under a brief database lock.
2. Send the native picker's selected path to Rust as the inbound command argument, then validate, read, parse, and canonicalize the file outside the SQLite mutex using the existing trust-boundary limits. The path is not returned, logged, or persisted, and raw rows never cross IPC.
3. Reacquire the database, start one transaction, and revalidate the same active save and current snapshot.
4. Filter rows by the captured current UID set. Upsert the complete format-owned row for each matching UID; count every non-matching UID as skipped.
5. Leave same-format rows for UIDs omitted from the file untouched. Leave the other format's table untouched.
6. Commit all matching rows together and return only the bounded import summary. Any error rolls the transaction back.

### Read behavior

- General player identity and memory-owned fields continue to come from the current `players` row. Enrichment is optional data joined by active `save_id` and UID only after a current player has established identity.
- Academy membership already establishes a durable save-scoped player reference. Its member projection may read Youth career values even when the current player row is absent, while continuing to show current identity fields as unresolved.
- Academy renames its nullable appearance contract and copy from `seniorLeagueAppearances` to reported career appearances so the UI matches the Youth Tracker all-time export semantics. A graduate has at least one reported career appearance.
- Existing honest completeness rules remain: an individual imported value can render, while an aggregate or Graduates workspace that requires complete coverage remains unavailable if any required member value is `null`.

## Uncertainty register

### Known

- The current CSV parsers already retain every field needed for the Youth and Moneyball persistence contracts.
- The current `players` lifecycle is snapshot-scoped and therefore unsuitable for durable enrichment columns.
- The memory bridge reads numeric left- and right-foot strength before deriving preferred foot.
- The legacy implemented schema contains 176 performance fields and the implemented formulas cover total/per-90 completion plus the listed ratios and minutes-per-event metrics.
- Neither supported CSV format supplies a trustworthy, normalized Moneyball season identifier.
- Academy already has nullable presentation seams for the four Youth career values.

### Assumptions

- Youth Tracker `AT Apps` is the reported all-time senior career appearance count intended to drive Academy graduate status.
- A selected Moneyball file represents the latest known season values for every matching player row it contains.
- Blank or absent optional data in a newer matched row should clear the older value to `null`; omission of the player row itself should preserve that player's prior enrichment.
- The pinned legacy implementation, rather than prose in the retired repository, is the accepted metric and formula boundary.

### Decisions

- Use separate save-scoped Youth career and Moneyball latest-season tables rather than expanding snapshot-owned `players`.
- Keep one latest row per save, format, and player. Prepare for history with stable metric keys and row-level import timestamps, but do not add an unused season abstraction now.
- Store the large canonical Moneyball metric set as one validated JSON object per player rather than adding 176 nullable SQLite columns or one row per metric.
- Preserve a normalized asking-price value because memory does not supply it; discard Moneyball transfer value because memory already owns market value.
- Exclude both Youth and Moneyball foot text because memory is the source for the underlying foot attributes.
- Make per-player upsert the replacement unit. Do not clear all prior rows when a file contains only a subset of the current snapshot.
- Use Academy as the immediate Youth Tracker consumer; persist Moneyball data without adding a presentation consumer in this feature.
- No ADR is required. The feature applies the existing Rust IPC and Rust-owned SQLite decisions without introducing a new platform boundary.

### Unknowns

- A future history feature must decide how the user identifies a season and whether one import can amend an existing season. That does not block latest-only persistence.
- A future Moneyball experience must decide which statistics, comparisons, and filters to present. That does not block storing the complete canonical contract.

### Risks

- JSON provides a compact evolvable persistence boundary but is less directly queryable than dedicated columns; a future analytics path may need Rust-side projection or a deliberate schema change.
- Full-row replacement means a newer sparse row can clear previously populated values. This is intentional latest-row behavior and must be explicit in tests and result copy.
- Rows not included in a later file keep their earlier timestamp and values, so current storage can contain a mixture of import waves.
- The Academy meaning of `AT Apps` must be replanned if a verified export proves it includes appearances that should not count toward the product's graduate definition.
- Numeric conversion from parsed unsigned CSV values to SQLite integers must reject out-of-range values without partial writes.

## Walking skeleton

Commits 1 through 3 form the thinnest end-to-end backend path: migration v17 provides save-owned storage, the Moneyball parser produces the complete canonical raw-and-derived statistic contract, and the import command atomically stores only rows whose UID belongs to the active save's current snapshot. The path is trunk-safe and fully integration-tested before Academy or Dashboard presentation changes.

## Delivery plan

### PR 1 — Persist CSV player enrichment

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(import): persist CSV player enrichment`

**Purpose:** Deliver the complete save-scoped persistence boundary, Moneyball calculation contract, matched-player import workflow, Academy career-stat consumer, and Dashboard import experience in one cohesive merge boundary.

**Depends on:** Completed CSV parsing and player reconciliation, snapshot ingest, and Youth Academy features.

| Field | Value |
| --- | --- |
| Branch | `feature/csv-enrichment-persistence` |
| Base branch | `main` |
| Publication provider | GitHub |
| PR template | `.github/pull_request_template.md` |
| Merge method | squash |
| Required checks | strict `check` |
| Feature close-out | Not run |
| CI repair rounds | 0 |

**Build-feature-loop profile:** Terra Max

#### Commit 1 — Add save-scoped enrichment schema

**Status:** Completed

**Provisional commit:** `feat(import): add save-scoped enrichment schema`

**Work:** Add migration v17 with the Youth career and Moneyball latest-season tables, save cascades, row-level import timestamps, normalized asking-price constraints, and schema-upgrade coverage.

**Out of scope:**

- Import writes or commands.
- Moneyball canonicalization or calculations.
- Academy and Dashboard changes.
- Historical season rows.

**Implementation packet:**

- Owners and files: `src-tauri/src/db/migrations.rs` and its migration tests.
- Existing patterns to verify: migration v11 Academy save-scoped tables, migration v15 JSON-bearing snapshot data, `saves(id) ON DELETE CASCADE`, and `PRAGMA user_version` tests.
- Constraints and invariants: upgrade v16 without changing existing rows; no foreign key to snapshot-owned `players`; exact `(save_id, player_uid)` uniqueness; valid nullable asking-price shapes; non-negative count and money fields; save deletion cascades; snapshot deletion does not.
- Dependencies and ordering: establishes the storage contract used by commits 3 and 4; no runtime consumer is added yet.

**Implementation profile:** Terra xhigh — the outcome is fixed, but an additive SQLite migration and future-compatible lifecycle constraints require material persistence judgment.

**Review profile:** Sol xhigh — migration correctness, existing-database upgrade behavior, constraints, and snapshot-versus-save ownership can create durable data-loss or compatibility defects.

**Validation:** Add migration tests that first fail because v17 and its tables are absent, then run `./scripts/dev check`; evidence must show fresh creation, v16 upgrade preservation, exact constraints, save cascade, and snapshot independence.

**Stop conditions:** Replan if the working schema is no longer v16, a required field cannot be represented without changing the parser contract, or preserving enrichment across snapshot replacement requires a foreign-key or ownership boundary different from the accepted design.

**Review mandate:**

- Verify the migration is additive and upgrades a populated v16 database without modifying existing data.
- Verify save deletion cascades and snapshot replacement cannot cascade into enrichment.
- Verify primary keys implement one latest row per save and player for each format.
- Verify asking-price and non-negative numeric constraints accept every parser variant and reject impossible shapes.
- Verify table naming and JSON ownership match current Rust-owned SQLite conventions without premature history tables.

#### Commit 2 — Derive canonical Moneyball statistics

**Status:** Active

**Provisional commit:** `feat(import): derive canonical Moneyball statistics`

**Work:** Transport the pinned 176-field Moneyball performance catalogue and its implemented calculation rules into the current Rust CSV domain, producing stable canonical keys and nullable exported-or-derived numeric values.

**Out of scope:**

- SQLite writes or reads.
- Changing CSV dialects, aliases, limits, or duplicate handling.
- Adding metrics that are absent from the pinned implemented schema.
- Moneyball presentation.

**Implementation packet:**

- Owners and files: `src-tauri/src/features/csv_import/model.rs`, `parser.rs`, a focused statistics module if separation improves clarity, and CSV parser/statistic tests and fixtures.
- Existing patterns to verify: current `MoneyballMetricValue`, `BTreeMap` ordering, fixture-backed parser tests, blank-as-null behavior, and safe parse diagnostics.
- Constraints and invariants: exactly 176 canonical performance keys with the exact pinned schema spelling; exported values remain authoritative except where the pinned implementation explicitly defines a fallback; calculations follow the pinned `schema.rs`, `row.rs`, and `derived.rs`; zero and null remain distinct; no non-finite JSON numbers.
- Dependencies and ordering: consumes the existing Moneyball parsed fields and supplies the complete object written in commit 3.

**Implementation profile:** Terra xhigh — the formula outcomes are pinned, but transporting a large typed catalogue without omissions or semantic drift requires substantial local mapping judgment.

**Review profile:** Sol High — a reviewer must independently compare the complete public data contract and edge semantics with the pinned legacy implementation.

**Validation:** Add failing catalogue and representative formula tests, then run `./scripts/dev check`; evidence must cover the exact key count and spelling, every export-source-to-pinned-key mapping, total/per-90 calculation in both directions, rounding, ratio families, the zero-percentage save-ratio fallback, positive-denominator rules, and null/zero boundaries.

**Stop conditions:** Replan if the pinned schema count or spelling cannot be reproduced, current parsed inputs cannot support an implemented legacy field, two legacy sources disagree on a formula, or the pinned code has an input-precedence rule that cannot be expressed without a new product decision.

**Review mandate:**

- Compare all 176 canonical keys against the pinned schema, including spelling and numeric kind.
- Verify every exported legacy input maps once and no speculative prose-only metric is added.
- Verify per-90 and reverse-total formulas, rounding, and positive-minute boundaries.
- Verify each ratio uses the correct numerator and denominator and preserves zero numerators.
- Verify the special save and clean-sheet ratios match the pinned implementation.
- Verify missing inputs remain null and serialization cannot emit NaN or infinity.

#### Commit 3 — Persist matched CSV player enrichment

**Status:** Pending

**Provisional commit:** `feat(import): persist matched CSV player enrichment`

**Work:** Add the import command and service transaction that parses outside the database lock, revalidates the active context, filters by current memory-backed UIDs, and upserts complete Youth or Moneyball rows for matching players only.

**Out of scope:**

- Dashboard activation or copy.
- Academy read behavior.
- Deleting rows omitted from the file.
- Historical imports, season identity, or Moneyball read UI.

**Implementation packet:**

- Owners and files: `src-tauri/src/features/csv_import/commands.rs`, `service.rs`, `model.rs`, feature module exports, `src-tauri/src/lib.rs`, and Rust command/service integration tests.
- Existing patterns to verify: `preview_csv_matches_for_path`, captured preview context, bounded opened-handle reads, snapshot ingest transactions, save lookup, and safe string errors.
- Constraints and invariants: memory UID eligibility; save isolation; parse outside mutex; stale revalidation inside the write transaction; all-or-nothing writes; per-player full-row replacement; no delete for omitted or unmatched UIDs; formats remain independent; the selected path crosses IPC only as the inbound file argument and is never returned, logged, or persisted; raw rows never cross IPC or enter SQLite.
- Dependencies and ordering: requires migration v17 and canonical Moneyball statistics; exposes a backend command for the Dashboard commit.

**Implementation profile:** Terra Max — save lifecycle, existing enrichment replacement, bounded file I/O, stale-context concurrency, atomic partial-failure behavior, and two format-specific mappings combine in one critical write path.

**Review profile:** Sol xhigh — the import can overwrite user data, and a missed eligibility, transaction, or stale-context defect could persist the wrong data to the wrong save.

**Validation:** Add failing Rust integration tests, then run `./scripts/dev check`; evidence must cover both formats, matched-only writes, skipped unknown UIDs, two-save isolation, same-player re-import replacement, preservation of omitted players, format coexistence, snapshot replacement survival, save deletion, stale context, malformed input, numeric conversion failure, transaction rollback, and the bounded summary DTO.

**Stop conditions:** Replan if eligibility cannot be proven from the same captured snapshot used for stale revalidation, any write must occur before full parsing succeeds, SQLite cannot commit all matched rows atomically, or the import needs to clear rows not present in the file.

**Review mandate:**

- Trace active save and snapshot identity from capture through transactional revalidation and every inserted key.
- Verify unmatched rows never insert or update, even when an older enrichment row for that UID exists.
- Verify a partial re-import replaces only included matching players and preserves all others.
- Verify parse, conversion, database, and stale-context failures leave both tables unchanged.
- Verify Youth and Moneyball mappings exclude every memory-owned field, including both foot representations.
- Verify snapshot replacement preserves enrichment while save deletion removes it.
- Verify IPC errors and results remain bounded and do not expose machine-local paths or rows.

#### Commit 4 — Use imported Youth career statistics in Academy

**Status:** Pending

**Provisional commit:** `feat(academy): use imported Youth career statistics`

**Work:** Join save-scoped Youth career enrichment into Academy member projections, populate career appearances, goals, assists, international caps, and graduate status, and align Academy field names and copy with all-time career semantics.

**Out of scope:**

- Moneyball presentation.
- Changing Academy membership, outcome, or club-family rules.
- Treating missing values as zero or showing incomplete aggregates as complete.
- Adding career timelines or charts.

**Implementation packet:**

- Owners and files: `src-tauri/src/features/academy/service.rs` and `commands.rs`; `src/features/academy/types/`, statistics utilities, Academy components, and existing Academy tests.
- Existing patterns to verify: save-scoped Academy membership projection, unresolved/departed state, `completeSum` and `completeCount`, graduate filtering, nullable IPC DTOs, and Academy query keys.
- Constraints and invariants: current identity still comes only from `players`; a tracked membership may retain previously verified career enrichment while unresolved; `AT Apps` is reported career appearances; graduate means at least one; partial coverage remains visibly incomplete.
- Dependencies and ordering: requires Youth persistence from commit 3; Dashboard invalidation after import is owned by commit 5.

**Implementation profile:** Terra xhigh — the data source is settled, but save-scoped joins, unresolved-member behavior, nullable aggregate semantics, and Rust-to-React contract alignment require cross-layer judgment.

**Review profile:** Sol High — the review must verify historical membership behavior and avoid presenting partial imported data as complete career truth.

**Validation:** Add the failing Academy service and utility/route expectations, run `./scripts/dev test src/features/academy/utils/academy-statistics.test.ts src/app/routes/academy.test.tsx`, then run `./scripts/dev check`; evidence must cover resolved and unresolved memberships, imported and missing values, graduate threshold, class aggregates, two-save isolation, and existing outcome behavior.

**Stop conditions:** Replan if a verified Youth Tracker definition shows `AT Apps` is not suitable for the graduate contract, or if populating unresolved memberships would bypass the durable Academy membership identity boundary.

**Review mandate:**

- Verify Academy queries scope enrichment by the same save and player UID as the membership.
- Verify current identity fields remain memory-owned and unresolved players are not recreated from CSV.
- Verify reported career data can survive snapshot replacement and remain useful for tracked departed players.
- Verify null values keep aggregates and Graduates availability honest.
- Verify the graduate threshold and all labels describe career rather than current-season appearances.
- Verify sale, release, class, and club-family behavior remains unchanged.

#### Commit 5 — Import CSV enrichment from the Dashboard

**Status:** Pending

**Provisional commit:** `feat(import): import CSV enrichment from Dashboard`

**Work:** Replace the reconciliation-only Dashboard experience and IPC surface with the persistent import action, bounded result summary, safe error states, context guards, and Academy cache invalidation after a successful Youth Tracker import.

**Out of scope:**

- Preview-before-confirmation workflow.
- Displaying stored Moneyball statistics.
- Import history, delete/reset controls, or source-file metadata.
- Changes to the primary Load Data action.

**Implementation packet:**

- Owners and files: `src/features/csv-import/`, `src/app/routes/index.tsx` and `index.test.tsx`, test IPC mocks, `e2e/tauri-ipc-stub.ts`, `e2e/smoke.spec.ts`, obsolete preview-only Rust/React API pieces, and command registration as needed.
- Existing patterns to verify: current native picker options, context-key late-result guard, safe error mapping, Load Data cache invalidation, Academy query root, status chips, and smoke IPC stub honesty.
- Constraints and invariants: choosing a file is the explicit import action; cancellation writes nothing; no selected path is displayed; pending and completed state cannot cross save or snapshot changes; summary distinguishes stored and skipped; Youth success refreshes Academy; Moneyball success does not invalidate unrelated domains.
- Dependencies and ordering: activates the tested command from commit 3 and the Academy consumer from commit 4; removes the obsolete preview-only surface without changing parser limits.

**Implementation profile:** Terra xhigh — the UI pattern exists, but coordinating a mutating IPC action, lifecycle invalidation, late-result suppression, and retirement of the preview contract spans several layers.

**Review profile:** Sol High — user-visible overwrite semantics and cross-context mutation require stronger review than an ordinary bounded presentation change.

**Validation:** Add failing route and smoke assertions, run `./scripts/dev test src/app/routes/index.test.tsx`, `./scripts/dev check`, and `./scripts/dev smoke`; evidence must cover no-snapshot guidance, cancellation, pending state, both success formats, skipped-player warning, safe failures, stale context, late completion, result clearing after Load Data/save switch, Academy invalidation, and no path disclosure.

**Stop conditions:** Replan if a successful import cannot invalidate only the affected Academy query root, the native dialog cannot make the mutating action explicit without a confirmation step, or removing the preview command would break another current consumer.

**Review mandate:**

- Verify the UI clearly says data is imported and matching rows are replaced, not merely previewed.
- Verify picker cancellation, parser failure, stale context, and late results cannot imply or trigger a successful write.
- Verify save and snapshot changes clear or suppress state from the old context.
- Verify stored and skipped counts use accurate, non-alarming copy and never expose a path.
- Verify Youth success refreshes Academy while Moneyball avoids unrelated invalidation.
- Verify obsolete preview-only API, mocks, and copy are removed without weakening browser-stub honesty.

## Active work

**PR:** PR 1 — Persist CSV player enrichment

**Commit:** Commit 1 — Add save-scoped enrichment schema

### RED proof

Add migration tests that expect schema version 17, both enrichment tables, exact key and cascade behavior, and preservation of a populated v16 database. Before the migration exists, `./scripts/dev check` must fail for the missing version and tables rather than for test setup.

### Expected outcome

Fresh and existing databases migrate to v17 with empty, save-scoped Youth and Moneyball enrichment tables. Existing saves, snapshots, players, Planner state, and Academy state remain unchanged. Deleting a save removes enrichment; replacing a snapshot cannot.

### Explicit exclusions

No parser, command, service write, Academy, React, bridge, or history change belongs in the active commit.

## Discoveries and replanning

- Planning inspection found that the user's initial player-column assumption does not fit the current lifecycle: `players` is snapshot-owned and is deleted on snapshot replacement. The accepted plan uses separate save-owned tables.
- The user selected per-player replacement: a new import changes matching players in the file and preserves enrichment for every omitted player.
- The bridge inspection confirmed that numeric left- and right-foot strengths are already read and decoded internally. Both CSV foot representations are therefore excluded, and the bridge remains unchanged.
- The user delegated the immediate consumer decision. Academy is selected because it already exposes nullable placeholders for the four Youth career values; Moneyball presentation is deferred.
- The retired repository's implemented schema and formulas are pinned at commit `366aa20b5282d3a63c94854ddb8da6992462b0c5`. Older prose is not accepted when it differs from code.
- Pre-commit review corrected four planning contradictions: the native selected path is an inbound IPC argument but is never returned or retained; live FM26 is not a completion gate; persisted metric keys keep exact legacy schema spelling; and a zero exported save percentage uses the pinned counts-based fallback.
- No planned feature spec exists to promote. No new ADR is justified because the plan follows the existing Rust IPC, SQLite ownership, and save-scoping decisions.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 — Persist CSV player enrichment | Commit 1 — Add save-scoped enrichment schema | Pending record | Migration v17 and coverage | Sol xhigh — Accept | None |

## Final validation

**Feature review profile:** Sol xhigh — final review must trace the persisted contract across migration, derivation, atomic import, snapshot/save lifecycle, Academy projection, and Dashboard invalidation; an error can silently overwrite or misattribute user data.

- Run `./scripts/dev test` and record the complete frontend test result.
- Run `./scripts/dev check` and record Biome, TypeScript, secretlint, Rust format/lint, migration, parser, import, Academy, and command test evidence.
- Run `./scripts/dev smoke` and record the complete Playwright result, explicitly identifying the stubbed IPC boundary.
- In a real desktop run, confirm native picker cancellation performs no import and exposes no path.
- In a real desktop run against an already-loaded save, import a valid Youth Tracker file; confirm stored/skipped counts, Academy career fields and graduate behavior, and app restart persistence. A running FM26 process is not required.
- Use deterministic Rust integration tests to confirm that snapshot replacement, which is the database lifecycle effect of Load Data, preserves both enrichment tables. A live FM26 scan is optional and is not a completion gate.
- Import a valid Moneyball file; confirm stored/skipped counts and restart persistence through deterministic database/service evidence because no Moneyball screen is included.
- Re-import a file containing a subset of previously imported matching players; confirm those rows are replaced, omitted player rows are unchanged, and blank values clear only the included player's prior fields.
- Confirm a CSV-only UID is skipped and cannot appear in player search, profile, Planner, or Academy candidate results.
- Confirm the same UID in two app saves has isolated enrichment and that importing against one active save cannot change the other.
- Confirm malformed, duplicate-UID, oversized, excessive-row, invalid-UTF-8, stale-context, conversion, and database failures leave prior enrichment unchanged.
- Confirm the canonical Moneyball object contains exactly the pinned 176 keys and representative exported/derived values after persistence round-trip.
- Run a fresh-context Sol xhigh feature review over the exact implementation set and resolve all retained Critical, High, and Medium findings before documentation reconciliation.

## Documentation impact

Complete during feature reconciliation:

- Update `.wiki/CONCEPT.md` so CSV enrichment persistence, memory precedence, matched-only imports, and Academy career data are current product behavior rather than a deferred boundary.
- Update `.wiki/ARCHITECTURE.md` for migration v17, the two save-scoped enrichment tables, the transactional import flow, canonical Moneyball calculations, and Academy's CSV-supplemented career projection.
- Update `.wiki/TODO.md` and the relevant backlog history item without presenting season history as implemented.
- Condense and move this ledger to `.wiki/features/completed/` with exact implementation and publication refs.
- Leave bridge schema documentation unchanged because the bridge contract is not modified.
