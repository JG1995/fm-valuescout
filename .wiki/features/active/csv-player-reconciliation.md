# CSV Parsing and Player Reconciliation

## Status

Active

## Intent

Add a safe, reusable CSV parsing foundation for the established Youth Tracker and Moneyball Football Manager exports. Let the user select one CSV and preview how its numeric FM player UIDs reconcile with the active save's current memory-backed snapshot.

This feature proves format support and player identity without changing application data. Durable CSV enrichment, memory-over-CSV field precedence, import provenance, and calculations for statistics that are not exported belong to the separate follow-up in [TODO.md](../../TODO.md).

## User-visible behavior

- The Dashboard shows a secondary CSV preview panel for the active save.
- With a current snapshot loaded, the user can select one `.csv` file. The app detects the pinned Youth Tracker or Moneyball format from its headers.
- The preview reports the detected format, parsed row count, exact UID matches, unmatched rows, and a bounded diagnostic summary.
- Unsupported headers, malformed required values, duplicate UIDs, invalid UTF-8, unsafe file input, or a changed save/snapshot produce an actionable error instead of a partial result.
- Previewing a file does not change the snapshot, Academy, Search, Player profiles, Planner, or any SQLite row. The result is discarded when its save or snapshot context changes and is not restored after restart.
- With no current snapshot, the panel points to **Load Data** because reconciliation requires memory-backed players.

## Invariants

- `Unique ID` is Football Manager's numeric player UID, not a UUID. Reconciliation uses exact UID equality only; it never falls back to player name.
- Accepted UIDs must fit the bridge-backed positive `u32` domain and each input file must contain each UID at most once.
- The Moneyball signature is checked before the broader Youth Tracker aliases so a Moneyball export cannot be misclassified as a Youth export.
- Youth Tracker CSV accepts the pinned comma/semicolon dialect and case-insensitive aliases. Moneyball CSV accepts the pinned semicolon dialect, exact header aliases, and optional UTF-8 BOM.
- Blank and recognized unavailable values remain `null`. The parser never turns missing or malformed statistics into zero.
- Populated malformed supported values fail with bounded row/field context. The complete file validates before reconciliation begins.
- Rust owns file reading, CSV parsing, normalization, format detection, UID reconciliation, file limits, and safe errors. React owns file selection, mutation state, and presentation only.
- File reading and parsing do not hold the SQLite mutex. The command captures the active save and current snapshot before reading, then revalidates both before returning a match result.
- IPC returns a bounded preview summary, never the complete parsed dataset, raw file contents, or a machine-local path.
- The preview performs no SQL write, schema migration, application-state overlay, or cache mutation. Memory-backed data therefore remains authoritative by construction.
- Only fields physically present in the exports are parsed. Exported per-90, ratio, percentage, and rating columns remain raw source values; calculating any statistic absent from the export is deferred.
- The feature remains offline and local. It does not upload or retain the selected file.

## Non-goals

- SQLite persistence, schema changes, import history, import provenance, retention, replacement, or rollback semantics.
- Query-time memory-over-CSV coalescing. The follow-up must preserve memory values whenever the bridge supplies the same field.
- Calculating Moneyball metrics that are absent from the CSV, including derived rates, ratios, percentiles, composite ratings, or scores.
- Adding CSV fields to Search filters/results, Player profiles, Youth Academy statistics, Planner, or Optimizer.
- Creating players that are absent from the current memory snapshot.
- Name-based matching, manual conflict resolution, fuzzy reconciliation, or duplicate-row repair.
- Moneyball scouting-versus-squad replacement rules or multi-file merging.
- HTML, `.fmf`, save-file, clipboard, drag-and-drop, localized-header, non-UTF-8, or batch import support.
- Changes to the FM26 bridge, dump schema, Load Data flow, or existing snapshot replacement semantics.

## Current-state map

- Relevant components: `src/app/routes/index.tsx` composes Dashboard panels; `src/app/components/app-top-bar.tsx` owns global Load Data; no CSV surface exists.
- Backend boundaries: `src-tauri/src/features/snapshot/` owns save selection, current-snapshot identity, dump ingest, and Load Data. `src-tauri/src/features/player/` and `src-tauri/src/features/academy/` read current-snapshot players by numeric UID.
- Data model: `players` is snapshot-scoped with primary key `(snapshot_id, uid)`. The current schema is migration v16. Planner and Academy retain save-scoped player UIDs across snapshot replacement.
- Persistence and migrations: SQLite is Rust-owned through `rusqlite`; this feature must leave migration v16 and all existing rows unchanged.
- Existing behavioral assumptions: Load Data is the only operation that replaces the current player snapshot. Unknown memory values stay `null`, and failed ingest preserves the previous current snapshot.
- Architectural seams: `src/lib/tauri-client.ts` is the only frontend invoke wrapper; Tauri commands are registered in `src-tauri/src/lib.rs`; async IPC results use TanStack Query; shared visual rules live in [DESIGN.md](../../DESIGN.md).
- Reference contract — Youth Tracker: [`main@5a9043d`](https://github.com/JG1995/fm-youth-tracker/commit/5a9043d3303984567680c229f6711ba4a022daaf), especially the [Rust parser](https://github.com/JG1995/fm-youth-tracker/blob/5a9043d3303984567680c229f6711ba4a022daaf/src-tauri/src/services/parser/mod.rs#L158-L225), [header mapping](https://github.com/JG1995/fm-youth-tracker/blob/5a9043d3303984567680c229f6711ba4a022daaf/src-tauri/src/services/parser/mod.rs#L374-L502), and [fixtures](https://github.com/JG1995/fm-youth-tracker/tree/5a9043d3303984567680c229f6711ba4a022daaf/fixtures/imports).
- Reference contract — Moneyball: [`main@366aa20`](https://github.com/JG1995/fm-valuescout-react/commit/366aa20b5282d3a63c94854ddb8da6992462b0c5), especially the [required header groups](https://github.com/JG1995/fm-valuescout-react/blob/366aa20b5282d3a63c94854ddb8da6992462b0c5/src-tauri/src/data/ingest.rs#L541-L642), [row mapping](https://github.com/JG1995/fm-valuescout-react/blob/366aa20b5282d3a63c94854ddb8da6992462b0c5/src-tauri/src/data/row.rs#L193-L421), and [fixtures](https://github.com/JG1995/fm-valuescout-react/tree/366aa20b5282d3a63c94854ddb8da6992462b0c5/docs/notes/test-files).
- Reference cautions: both earlier apps parse in Rust, but their handwritten/Arrow persistence, silent zero/null coercions, incoming duplicate handling, name matching, and blind replacement rules are not compatible with this repository's contracts.
- Project validation commands: `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev smoke`; `pnpm tauri dev` is the documented real-WebView integration path.
- Primary risks: dialect detection, an 80-group Moneyball header contract, unit conversion drift, unsafe local-file handling, stale snapshot reconciliation, unbounded IPC, and unproven cross-source UID equality in a real same-save sample.

## Feature architecture

Create a matched `csv_import` feature boundary.

- Rust `src-tauri/src/features/csv_import/` owns format detection, typed parsed rows, explicit field normalization, input limits, safe errors, and snapshot reconciliation. Use a standards-compliant CSV crate instead of copying either reference project's handwritten splitter.
- The parser produces an internal Youth Tracker or Moneyball row collection. The collection exists only for the duration of one preview command and never crosses IPC.
- The preview service briefly captures the active save and current snapshot, releases the database lock, reads and parses the selected file, then reacquires the lock and verifies that the same save and snapshot remain current before matching UIDs.
- The Tauri command returns only a bounded DTO: detected format, row count, matched/unmatched counts, recognized field-group information, and capped diagnostics. It does not return parsed player rows.
- React `src/features/csv-import/` owns the typed command wrapper, mutation, and Dashboard panel. It uses the official Tauri dialog plugin for one-file selection, existing Panel/Button/empty-state patterns, and local result state. The top bar remains unchanged because preview is a setup/diagnostic action, not the app's recurring primary action.
- Save switches and successful Load Data changes clear an obsolete preview. Preview does not invalidate domain query caches because it does not mutate them.
- Rust tests use small synthetic fixtures based on the pinned headers, plus temporary SQLite databases and files. Frontend tests mock the dialog and IPC boundaries; Playwright proves the bounded Dashboard flow with its existing IPC stub where practical. A real Tauri manual check owns native-dialog integration.

No ADR is planned. Rust-owned file/validation work, the IPC boundary, and the lack of persistence follow existing ADR-0014 and ADR-0015. Revisit that decision only if implementation requires a new data-source authority or filesystem permission model.

## Uncertainty register

### Known

- Both references use FM's numeric `Unique ID`; neither uses UUID values.
- Youth Tracker production supports comma or semicolon delimiters, case-insensitive aliases, UTF-8 text, appearance notation such as `1 (4)`, and the player/attribute/career-stat fields in its committed fixtures.
- Moneyball production requires a semicolon CSV with approximately 80 logical header groups, accepts a small pinned alias set, and recognizes optional Division, CA, PA, Asking Price, and actual Save Percentage columns.
- The current app stores bridge player UIDs as numeric values and already uses them for snapshot, Player, Planner, and Academy identity.
- The current repository has no CSV dependency, dialog plugin, parser module, import UI, or CSV persistence schema.
- The developer explicitly deferred persistence so it can be designed with non-exported-stat calculations in a separate feature.

### Assumptions

- The unchanged exports remain valid UTF-8, optionally with a UTF-8 BOM. Non-UTF-8 transcoding is unnecessary until a real file proves otherwise.
- A CSV exported from the same FM26 save uses the same numeric player UID as the bridge dump.
- A single-file preview with no retained data is useful as the implementation and compatibility foundation for the follow-up feature.
- The official Tauri dialog plugin can provide one-file selection with narrower permissions than a general filesystem plugin.

### Decisions

- Deliver one PR with four atomic commits: Youth parser, Moneyball parser, UID reconciliation command, and Dashboard workflow.
- Auto-detect the format; do not ask the user to select Youth Tracker versus Moneyball manually.
- Require numeric UID identity even though the earlier Youth parser permitted name-only rows.
- Reject duplicate UIDs and malformed populated supported values instead of copying earlier silent coercion or duplicate behavior.
- Add only a standards-compliant CSV parser and the official dialog integration required by the active feature. Do not add a general import framework.
- Keep every parsed row ephemeral. Do not add an in-memory overlay that the persistence feature would replace.
- Keep the current memory snapshot untouched and classify persistence/derived statistics as a dependent feature in TODO.

### Unknowns

- Real FM export behavior for Windows-1252, localized headers, currencies other than the pinned Moneyball euro format, or decimal commas.
- Whether the first representative same-save CSV and bridge snapshot will prove exact UID equality.
- The exact production file-size limit. Commit 3 must choose and test a bounded limit from the largest known fixture plus practical desktop headroom.
- Whether future persistence needs Moneyball scouting/squad source labels, season/game-date provenance, or replacement history. The follow-up feature owns those decisions.

### Risks

- Youth aliases are broad enough to accept part of a Moneyball header; detection order and signature tests must prevent misclassification.
- Moneyball's large required field set can drift from fixture reality or accidentally include calculations that belong in the follow-up.
- Parsing on the command thread or holding the database mutex during file I/O could freeze other IPC work.
- A save switch or Load Data completion during parsing could make a match summary describe the wrong snapshot.
- A file-path argument or error could expose more local filesystem information than the bounded preview needs.
- Browser-stub tests cannot prove the native file dialog or real Tauri permission wiring.

## Walking skeleton

The thinnest proving path is commits 1–3: Rust parses one representative fixture from each pinned format, rejects unsafe identity input, and returns a bounded exact-UID match summary for the same still-current snapshot without changing SQLite.

## Delivery plan

### PR 1 — Preview supported FM CSV exports

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/csv-player-reconciliation`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** strict `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Build-feature-loop profile:** Terra xhigh — the PR combines two external file contracts with Tauri file selection and snapshot-lifecycle reconciliation, but it has no migration or persistent partial-failure path.

**Provisional PR title:** `feat(import): preview supported FM CSV exports`

**Purpose:** Deliver one non-mutating user workflow and the reusable, tested parser/reconciliation foundation needed by the separately planned persistence and derived-stat feature.

**Merge to trunk when:** Both formats parse through their pinned contracts, exact UID reconciliation is bounded and stale-safe, the Dashboard preview works accessibly, no persistence path exists, all required validation passes, and fresh commit reviews retain no CRITICAL, HIGH, or MEDIUM findings.

**Depends on:** Completed FM26 memory read, snapshot ingest, and current player UID storage. It does not depend on new bridge work or the deferred persistence feature.

#### Commit 1 — Parse Youth Tracker CSV exports

**Status:** Active

**Provisional commit:** `feat(import): parse Youth Tracker CSV exports`

**Work:** Add the Rust `csv_import` parser foundation, a standards-compliant CSV dependency, the pinned Youth Tracker dialect/header aliases, typed exported values, duplicate/UID validation, and representative parser fixtures/tests.

**Out of scope:**

- Moneyball headers and metrics.
- Tauri commands, file selection, snapshot reconciliation, or React.
- Persistence, derived statistics, UI consumption, or bridge changes.

**Implementation packet:**

- Owners and files: `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/src/features/mod.rs`, new `src-tauri/src/features/csv_import/` parser/model/error files, and small test fixtures under that feature.
- Existing patterns to verify: `features/memory_read/dump_validation.rs` for fail-closed boundary errors; `features/snapshot/ingest.rs` for null preservation; upstream Youth parser/header/fixture links in Current-state map.
- Constraints and invariants: require a positive numeric UID and name header, support comma/semicolon with deterministic dialect selection, accept pinned case-insensitive aliases and appearance notation, preserve missing values, reject duplicates, and keep the module free of SQL/Tauri/UI work.
- Dependencies and ordering: add only the CSV crate needed for standards-compliant records; this parser contract is required by commits 3 and 4.

**Implementation profile:** Terra xhigh — the outcome is fixed, but the alias-rich trust-boundary parser and broad attribute/stat model require material local judgment.

**Review profile:** Sol High — a permissive or lossy parser would become the data foundation for later persistence, and current validation cannot rely on a live FM export alone.

**Validation:** Run `./scripts/dev check`. Evidence must show the new Rust parser tests cover both delimiters, quoted delimiters/escaped quotes, optional UTF-8 BOM, aliases, appearance totals, blank/null values, malformed populated values, invalid/out-of-range UIDs, row-width errors, and duplicate UIDs while the full repository gate remains green.

**Stop conditions:** Stop and replan if the pinned fixtures contradict the analyzed header contract, real required fields need a non-UTF-8/localized policy, valid exported UIDs cannot fit the bridge `u32` domain, or standards-compliant parsing would require a second competing parser.

**Review mandate:**

- Verify the parser requires UID identity and never performs name fallback.
- Verify dialect/header detection cannot silently shift columns.
- Verify blank/unavailable values remain null and malformed populated values never become zero.
- Verify duplicate, quote, multiline, and row-width behavior is explicit and tested.
- Verify errors are bounded and contain no raw row or local-path disclosure.
- Verify no SQL, Tauri command, persistence, or derived-stat logic entered this commit.

#### Commit 2 — Parse Moneyball CSV exports

**Status:** Pending

**Provisional commit:** `feat(import): parse Moneyball CSV exports`

**Work:** Extend the parser with Moneyball format detection, the pinned required/optional header groups and aliases, typed raw exported metrics, exact unit/value normalization, and fixture-backed validation. Detect this strict signature before Youth Tracker.

**Out of scope:**

- Calculating values absent from the export, percentiles, composite ratings, or role scores.
- Scouting/squad merge precedence, persistence, snapshot matching, IPC, or UI.
- Currency/localization behavior not demonstrated by the pinned fixtures.

**Implementation packet:**

- Owners and files: existing `src-tauri/src/features/csv_import/` parser/model/error files plus Moneyball-specific fixtures/tests.
- Existing patterns to verify: pinned Moneyball `ingest.rs`, `row.rs`, `parsers.rs`, and exercised fixture headers; commit 1's shared CSV/error/UID contracts.
- Constraints and invariants: semicolon dialect, exact pinned header/alias policy, BOM on the first header only, required versus optional groups, numeric UID, null preservation, duplicate rejection, and parsing only physically exported values.
- Dependencies and ordering: reuse the commit 1 CSV foundation; add no Arrow/analytics/persistence dependency.

**Implementation profile:** Terra xhigh — the format is known, but its large field surface and unit conversions require careful contract translation without copying obsolete upstream persistence or derivation behavior.

**Review profile:** Sol High — missing or misbound headers can produce plausible but wrong statistics across many rows, while the non-exported-calculation boundary must remain intact.

**Validation:** Run `./scripts/dev check`. Evidence must show Rust tests for required/optional headers, old/new aliases, semicolon and BOM behavior, representative money/height/appearance/percent/stat values, nulls, malformed identity/data, duplicates, format auto-detection, and explicit rejection of a near-match missing a required Moneyball group.

**Stop conditions:** Stop and replan if committed fixtures disagree on a required header without a deterministic alias, correct parsing requires undocumented locale/currency rules, an input field requires a derived calculation rather than export parsing, or Moneyball cannot be distinguished from Youth by headers alone.

**Review mandate:**

- Verify every required and optional logical group maps to the intended field.
- Verify format detection checks Moneyball before the broader Youth aliases.
- Verify conversions match pinned export syntax and do not invent locale behavior.
- Verify no non-exported/per-90/ratio/percentile calculation slipped into scope.
- Verify malformed values, nulls, duplicate UIDs, and extra columns behave explicitly.
- Verify no Arrow cache, source replacement, SQL, or name-based merge logic was copied.

#### Commit 3 — Preview CSV matches by player UID

**Status:** Pending

**Provisional commit:** `feat(import): preview CSV matches by player UID`

**Work:** Add the non-mutating Rust preview service and Tauri command. Validate/read one bounded CSV, parse it outside the DB lock, reconcile exact UIDs against the captured current snapshot, revalidate that context, and return a safe bounded summary.

**Out of scope:**

- Native file-picker or Dashboard UI.
- SQLite writes, schema migration, session overlay, import retention, or cache invalidation.
- Returning full parsed rows or applying CSV values to any read model.

**Implementation packet:**

- Owners and files: new `src-tauri/src/features/csv_import/service.rs` and `commands.rs`, module exports, `src-tauri/src/lib.rs` command registration, and service/command tests with temporary files and migrated temporary databases.
- Existing patterns to verify: `snapshot::commands::load_data` and `snapshot::load_data` for captured save/snapshot lifecycle and short DB locks; `snapshot::service::active_save_id`; current-snapshot queries in `player/query.rs`; bounded DTO/error conventions in existing commands.
- Constraints and invariants: `.csv` regular file only, explicit byte/row limits, UTF-8/BOM policy, no path or raw-data response, exact UID match, current snapshot required, stale-context failure, bounded diagnostics, and provable absence of SQL writes.
- Dependencies and ordering: both parsers from commits 1 and 2; UI and dialog integration wait for commit 4.

**Implementation profile:** Terra xhigh — file I/O, Tauri error shaping, database lock scope, and snapshot lifecycle interact even though the command is read-only.

**Review profile:** Sol High — the command crosses the local-file trust boundary and can report a wrong save's matches if context revalidation or bounds are incomplete.

**Validation:** Run `./scripts/dev check`. Evidence must show Rust tests for Youth and Moneyball summaries, exact matches/unmatched counts, no snapshot, stale save/snapshot, duplicate/invalid data propagation, file type/size/UTF-8 limits, safe bounded errors, no full-row IPC response, and unchanged migration version/table contents after success and failure.

**Stop conditions:** Stop and replan if a representative same-save export proves its Unique IDs differ from bridge UIDs, safe file selection would require broad file-content IPC or unrestricted filesystem output, the command cannot release the DB lock during parsing, or a useful result requires persistence or an unbounded dataset.

**Review mandate:**

- Verify exact UID-only reconciliation and same-current-snapshot revalidation.
- Verify file reading/parsing occurs without the SQLite mutex.
- Verify byte, row, diagnostic, and IPC-result bounds cover success and failure.
- Verify no local path, raw row, or complete parsed dataset crosses IPC.
- Verify the command cannot mutate SQLite or retain an overlay.
- Verify errors distinguish no snapshot, unsupported format, invalid file, and stale context.

#### Commit 4 — Add the CSV reconciliation preview

**Status:** Pending

**Provisional commit:** `feat(import): add CSV reconciliation preview`

**Work:** Add least-privilege native CSV selection and a Dashboard panel that runs the preview command and presents pending, detected-format, match, warning, stale-context, and error states. Reset obsolete results when the active save or current snapshot changes.

**Out of scope:**

- Top-bar changes, a new route/nav item, drag-and-drop, batch selection, or editable reconciliation.
- Persisting or displaying CSV statistics in Search, Player profiles, Academy, Planner, or Optimizer.
- Domain query invalidation after preview.

**Implementation packet:**

- Owners and files: Tauri dialog dependencies/registration and narrow capability changes in `src-tauri/`; new `src/features/csv-import/api/`, `types/`, and `components/`; `src/app/routes/index.tsx`; focused frontend tests; `e2e/tauri-ipc-stub.ts` and smoke only where the native-picker boundary can be represented honestly.
- Existing patterns to verify: Dashboard independent panel composition; Button loading labels; `LoadDataOutcome` phase/error copy; snapshot/save queries; existing modal/focus/error behavior; design rule that Load Data remains the sole top-bar primary action.
- Constraints and invariants: one `.csv` selection, secondary action hierarchy, accessible labelled control and result status, no machine path display, local mutation state rather than a global store, context-reset behavior, no data cache invalidation, and least-privilege dialog permission.
- Dependencies and ordering: commit 3 command/DTO; verify current official Tauri v2 dialog registration and permission details during build.

**Implementation profile:** Terra xhigh — the UI is bounded, but Tauri plugin/permission integration, native dialog behavior, and save/snapshot lifecycle span the WebView boundary.

**Review profile:** Sol High — broad dialog permissions, stale preview state, inaccessible result feedback, or browser-only evidence could make the workflow misleading or unsafe.

**Validation:** Run `./scripts/dev test src/features/csv-import src/app/routes/index.test.tsx`, `./scripts/dev check`, and `./scripts/dev smoke`. Evidence must cover no-snapshot guidance, selection cancel, pending label, both format summaries, unmatched warning, safe error copy, context reset, keyboard/focus behavior, no raw path, and unchanged existing Dashboard/Load Data behavior. Record the native-dialog gap separately from stubbed browser evidence.

**Stop conditions:** Stop and replan if the dialog integration needs general filesystem permissions, the plugin cannot provide a single local file without exposing contents to the WebView, the preview competes with Load Data as a primary/global action, context changes can leave a credible stale result, or the Dashboard cannot hold the panel at the 1280×800 minimum.

**Review mandate:**

- Verify Tauri dialog capabilities are limited to the selected one-file workflow.
- Verify React receives only the path token needed by the command and never reads/parses the file.
- Verify pending, cancel, success, warning, failure, and stale-context states are accessible.
- Verify save/snapshot changes clear obsolete results without mutating domain caches.
- Verify the top bar and existing Dashboard hierarchy remain intact.
- Verify tests distinguish mocked dialog/IPC behavior from real native integration.

## Active work

**PR:** PR 1 — Preview supported FM CSV exports

**Commit:** Parse Youth Tracker CSV exports

### RED proof

Add the smallest Rust parser test with a semicolon-delimited Youth Tracker fixture containing a numeric `Unique ID`, quoted name, appearance notation, one nullable statistic, and one attribute. The test must fail because no `csv_import` parser exists, then later prove normalized identity/value output. Add focused negative tests before implementation for duplicate UID and malformed populated numeric data so a parser that silently coerces either case cannot pass.

### Expected outcome

The repository has a Rust-only Youth Tracker CSV parser with representative fixtures and explicit identity, dialect, null, quoting, alias, and failure contracts. `./scripts/dev check` passes. No command, UI, SQL, or persistence behavior exists yet.

### Explicit exclusions

- Moneyball format work.
- Snapshot reconciliation or Tauri commands.
- React, dialog integration, or Dashboard changes.
- Persistence, derived statistics, and any downstream consumer.

## Discoveries and replanning

- 2026-08-10: The developer clarified that persistence is deferred because it must be designed together with Moneyball-style calculations for statistics absent from the exports. This feature therefore ends at parsing and non-mutating UID reconciliation; [TODO.md](../../TODO.md) owns the dependent persistence/derivation feature.
- 2026-08-10: Reference analysis confirmed that both earlier projects call the field a Unique ID/UID but use a numeric FM player identifier, not a UUID. The current app's bridge/player domain is numeric too.
- 2026-08-10: The earlier repos provide useful parser/header/fixture contracts but not suitable current-app merge semantics. This plan reuses their evidence and converters selectively while retaining current snapshot, null, UID, and trust-boundary rules.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| — | — | — | None | None | None |

## Final validation

**Feature review profile:** Sol High — final review must assess two external data contracts, local-file permissions, snapshot lifecycle, bounded IPC, and the non-persistence boundary across Rust and React.

- Run `./scripts/dev format` and confirm no unrelated formatting changes.
- Run `./scripts/dev test` and retain the complete frontend result.
- Run `./scripts/dev check` and retain the complete Biome, TypeScript, secretlint, Rust format/lint/test result.
- Run `./scripts/dev smoke` and retain the complete Playwright result. State explicitly that the stub cannot prove the native dialog.
- In a real `pnpm tauri dev` session, preview one valid Youth Tracker CSV and one valid Moneyball CSV, cancel selection once, and confirm detected format plus bounded row/match results without a path or raw player values.
- Use one representative same-save export and bridge snapshot to prove at least one numeric CSV UID equals the current `players.uid`. If no such evidence is available, retain it as a feature-blocking compatibility gap rather than claiming real mapping.
- Prove by Rust integration tests that successful and failed previews leave `PRAGMA user_version`, snapshot/player counts, and existing current-snapshot identity unchanged.
- Confirm invalid UTF-8, unsupported headers, missing required groups, malformed values, duplicate UIDs, oversized files, no snapshot, and stale snapshot context all fail safely without partial output or retained state.
- Dispatch a fresh Sol High feature reviewer over the exact planned implementation refs after all commit reviews clear.

## Documentation impact

Planning creates this active ledger and activates the feature in [TODO.md](../../TODO.md). No current-state document changes during planning.

At feature completion, reconcile:

- [ARCHITECTURE.md](../../ARCHITECTURE.md) with the implemented Rust parser/preview command, Dashboard flow, dialog capability, and test boundary.
- [CONCEPT.md](../../CONCEPT.md) so the optional supplemental CSV preview is not contradicted by the existing no-manual-import differentiation while live memory remains the canonical data source.
- [DESIGN.md](../../DESIGN.md) with the implemented Dashboard preview panel and its states.
- The completed feature record and TODO status. Keep the persistence/derived-stat follow-up active in TODO until it is separately planned and delivered.

No ADR or debug report is planned unless implementation changes the data-source authority, filesystem permission model, or reveals a reusable failure pattern that code/tests cannot explain.
