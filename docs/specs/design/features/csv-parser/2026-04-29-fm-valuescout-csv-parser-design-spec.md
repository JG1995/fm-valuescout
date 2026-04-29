# CSV Parser - Design Spec

## Context

FM ValueScout is a companion app for Football Manager players applying moneyball-style scouting. The app's data comes from CSV files exported from Football Manager. The CSV parser is the entry point for all data — without it, no other feature (scouting, player profiles, squad management) can function.

The project is a Tauri v2 desktop app with a Rust backend and Svelte 5 frontend. The codebase is currently a fresh scaffold with no existing data layer, error handling patterns, or module structure. The CSV parser is the first real feature being built.

## Problem Statement

Football Manager exports player data as semicolon-delimited CSV files with 80+ columns, inconsistent naming, embedded units, mixed types, and locale-dependent formatting. The parser must extract, validate, normalize, and compute derived metrics from these files, producing structured data that the rest of the application can consume without further transformation.

## Assumptions and Constraints

- **FM CSV format is semi-stable.** Column names may change between FM versions, and some columns (CA, PA) may be absent. The parser must not break on these variations.
- **Semicolons are the most likely delimiter.** The sample uses `;`. However, FM locale settings could produce comma or tab delimited files.
- **The in-game date format is dd/mm/yyyy.** FM is a British game; this is the most likely date format in exports.
- **Currency varies by game save.** The currency symbol is embedded in monetary fields. No currency conversion is needed — all comparisons are relative (percentiles within the dataset).
- **CSV file sizes are typically 2,000-3,000 rows.** The theoretical maximum is ~500,000 players, but this is a pathological case.
- **UTF-8 encoding.** Player and club names contain accented and non-Latin characters. Rust and the `csv` crate handle UTF-8 natively.
- **No concurrent access.** This is a single-user desktop app. Only one import runs at a time.

### Hard Constraints

- The parser must not modify the original CSV file.
- The parser must not write to the database — persistence is handled by a separate step.
- `parse_csv` must be a pure function with no side effects.

## Anti-Requirements

- **No "read from memory" feature.** Shelved indefinitely. The parser only consumes CSV files.
- **No column reordering UI.** Not part of the parser's scope.
- **No import progress indicator.** Parsing is instant for realistic file sizes. Not needed.
- **No currency conversion.** Values are stored in their original currency.
- **No partial-save on error.** If the user cancels after reviewing the summary, nothing is persisted.

## Feature Scope

### Must-haves

#### CSV file selection and ingestion

- Purpose: Allow the user to select a CSV file via Tauri's native file dialog.
- Behavior: File dialog filters to `.csv` files. Selected file path is passed to the Rust backend. If the file is unreadable (locked, permissions), return a clear error immediately.
- Failure mode: Unreadable file → error message "Unable to read file. It may be open in another program."

#### In-game date collection

- Purpose: Capture the current in-game date for contract calculations and season derivation.
- Behavior: Frontend presents a date picker (not a text input) so the user selects an unambiguous date. The date is sent as an ISO string to Rust. This is required — no import without it.
- Failure mode: N/A — date picker prevents invalid input.

#### Header parsing and column detection

- Purpose: Map CSV column headers to internal field identifiers, handling FM version differences.
- Behavior:
  1. Strip BOM (`\u{feff}`) from first byte if present.
  2. Detect delimiter: count `;`, `,`, `\t` in header row, use most frequent. Default to `;` on tie.
  3. Build case-insensitive header→index map.
  4. Check for required columns: `Unique ID`, `Player`, `Minutes`, `Position` (case-insensitive matching). If any are missing, reject entire file.
  5. Classify all other columns as found or missing, report in `ParseResult`.
- Failure mode: Missing required column → reject with error listing which required columns are absent. Non-CSV file (< 3 delimiter-separated fields in header) → reject with "File does not appear to be a valid Football Manager export."
- Decisions made: Flexible name-based matching chosen over strict column-order matching to survive FM version changes. Case-insensitive to handle inconsistencies like `PsP` vs `Psp`.

#### Row-level parsing with graceful degradation

- Purpose: Parse each data row into a `ParsedPlayer` or skip it, collecting errors along the way.
- Behavior:
  - **Hard reject (skip row):** Missing/invalid UID, empty player name, unparseable position string, duplicate UID within same file.
  - **Soft degradation (store None + warning):** Any other field that fails to parse. The player is still imported; only that field is `None`.
  - Zero minutes → all per-90 computed values are `None` (not zero, to distinguish "hasn't played" from "played and produced zero").
- Failure mode: Skipped rows are collected in `skipped_rows` with row number and reason. The user sees these in the import summary.
- Decisions made: Skip-and-collect chosen over fail-fast because one bad row shouldn't prevent importing 257 good ones.

#### Field parsing — identity fields

- `uid`: parse as `u32`. Missing or non-integer → skip row.
- `name`: trim whitespace, store as-is (UTF-8). Empty → skip row.
- `nationality`: look up 3-letter code in static FIFA country code table (~200 entries). Store `{ code, name }`. Unknown code → store code as name (fallback display).
- `second_nationality`: already a full name in CSV. Store as `Nationality { code: None, name }`.
- `club`: store as-is.
- `age`: parse as `u16`. Negative → `None` + warning.

#### Field parsing — position strings

- Purpose: Convert FM's position notation into structured data for archetype matching.
- Behavior:
  - Split on `, ` to get individual position entries.
  - Parse each entry into `Position { role: Role, sides: Vec<Side> }` where `Role` is an enum (GK, D, WB, DM, M, AM, ST) and `Side` is an enum (L, C, R).
  - Combined roles like `D/WB (L)` produce two positions: `{ role: D, sides: [L] }` and `{ role: WB, sides: [L] }`.
  - No side qualifier (e.g., just `D`) → `sides: []`. Valid.
  - Unrecognized format → skip entire row.
- Failure mode: Unparseable position string → row skipped, added to `skipped_rows`.
- Decisions made: Structured parsing chosen over storing raw strings because archetype matching needs enum comparison, not string matching.

#### Field parsing — physical fields

- `height`: strip ` cm` suffix, parse as `u16`.
- `left_foot` / `right_foot`: match against known labels (`Very Strong`=5, `Strong`=4, `Fairly Strong`=3, `Reasonable`=2, `Weak`=1). Store both label and score. Unrecognized label → score 3 (middle) + warning.

#### Field parsing — ability fields (optional columns)

- `CA` / `PA`: columns may be entirely absent from the CSV. If absent, all values `None`. If present, parse as `u16`. Invalid value → `None`.

#### Field parsing — financial fields

- `transfer_value`: extract currency symbol (leading non-digit characters), extract numeric values handling K/M suffixes with optional decimal (e.g., `€62M`, `€38.5K`). Split range on ` - ` → store `transfer_value_low` and `transfer_value_high`. Single value → low = high. Unparseable → `None` + warning.
- `wage`: extract currency symbol, numeric value, K/M suffix, and denomination (`p/w`, `p/m`, `p/a`). Compute `wage_per_week`: `p/w` as-is, `p/m` ÷ 4.33, `p/a` ÷ 52. Store both normalized value and original raw string. Unparseable → both `None` + warning.

#### Field parsing — date and playing time fields

- `contract_expires`: parse as `dd/mm/yyyy`. Fall back to `dd/mm/yy`. Unparseable → `None` + warning.
- `appearances`: parse `N (M)` → `apps_started = N`, `apps_sub = M`. No parens → `apps_sub = 0`.
- `minutes`: parse as `u16`. Zero is valid.

#### Field parsing — stat fields

- All 80+ stat columns: parse as `f64`. Empty or non-numeric → `None` + warning.
- Negative values: allowed only for xG Overperformance (`xG-OP`) and Expected Goals Prevented (`xGP`). Negative elsewhere → `None` + warning.
- Distance fields: strip `km` suffix, parse as `f64`.
- Decimal precision: store full `f64` precision. Rounding for display is a UI concern.

#### Computed metrics

- Purpose: Derive per-90 values, ratios, and other computed metrics from raw CSV values.
- Behavior: Separate function called after extraction. Computes:
  - **Per-90 values:** `total / minutes * 90`. If minutes = 0 or `None`, result is `None`.
  - **Ratios:** `completed / attempted`. If attempted = 0 or `None`, result is `None`.
  - All computed fields stored as `Option<f64>`.
- Failure mode: Division by zero → `None`. No error or warning (this is expected for players with zero minutes).
- Decisions made: Computation separated from extraction for testability and clean boundaries. Both are called within the same `parse_csv` command — the user sees the full enriched result.

#### User confirmation before persisting

- Purpose: Prevent unwanted data from entering the database.
- Behavior: Frontend renders a summary of `ParseResult` (player count, skipped rows, warnings). User can "Save" or "Cancel". Save triggers `save_import` command. Cancel discards the parsed data.
- Failure mode: N/A — this is a UX gate, not a system failure.

#### Two-command Tauri interface

- `parse_csv(file_path: String, in_game_date: String) -> ParseResult` — pure function, no side effects.
- `save_import(players: Vec<ParsedPlayer>, in_game_date: String)` — persists to database.

#### Idempotent imports

- Purpose: Prevent duplicate data from accidental re-imports.
- Behavior: If a snapshot with the same UID + in-game date already exists in the database, skip it silently during `save_import`. No warning needed — the data is identical.

#### Per-field validation warnings

- Purpose: Give the user visibility into data quality issues.
- Behavior: Every field that degrades to `None` (except missing optional columns) produces a `ParseWarning { row_number, field, message }`. These are shown in the import summary.
- Failure mode: N/A — warnings are informational, not blocking.

### Nice-to-have (defer)

None remaining. All discussed features are either in must-have or removed entirely.

## Integration Points

### Existing Code Affected

- `src-tauri/src/lib.rs` — register new Tauri commands (`parse_csv`, `save_import`).
- `src/routes/+page.svelte` — will need import UI (file dialog trigger, date picker, summary screen). Not part of the parser spec but the frontend integration point.

### New Code Required

- `src-tauri/src/commands/mod.rs` — Tauri command handlers.
- `src-tauri/src/commands/csv_parser.rs` — `parse_csv` and `save_import` command implementations.
- `src-tauri/src/parser/mod.rs` — parser module root.
- `src-tauri/src/parser/headers.rs` — header parsing, delimiter detection, column mapping.
- `src-tauri/src/parser/fields.rs` — individual field parsing functions (identity, physical, financial, etc.).
- `src-tauri/src/parser/positions.rs` — position string parsing.
- `src-tauri/src/parser/metrics.rs` — computed metrics (per-90, ratios).
- `src-tauri/src/parser/types.rs` — core types (`ParsedPlayer`, `ParseResult`, `Position`, `Footedness`, `Nationality`, stat category structs).
- `src-tauri/src/parser/countries.rs` — static FIFA country code lookup table.
- `src-tauri/src/storage/mod.rs` — persistence layer (called by `save_import`, separate concern).

### Data Changes

- New data structures: `ParsedPlayer`, `ParseResult`, `Position`, `Role`, `Side`, `Footedness`, `Nationality`, `SkippedRow`, `ParseWarning`, and 8 stat category structs.
- No existing data to migrate — this is the first feature.

### API Changes

- New Tauri commands: `parse_csv`, `save_import`.

## Edge Cases and Boundary Conditions

- **Non-CSV file:** < 3 delimiter-separated fields in header → reject immediately with clear error message.
- **Empty CSV (headers only):** Return 0 players, no error, informative message.
- **Different delimiter:** Auto-detect from header row. Most frequent of `;`, `,`, `\t` wins.
- **Duplicate UID within same CSV:** First occurrence wins, second skipped with warning.
- **Same CSV imported twice (same in-game date):** Idempotent — existing snapshots are not duplicated.
- **Multiple imports within same season:** All snapshots preserved. Active snapshot per player = latest in-game date.
- **All rows fail validation:** Return 0 players with all rows in `skipped_rows`. Inform user data may be corrupted.
- **Special characters in names:** UTF-8 handled natively. No special processing.
- **Monetary values without K/M suffix:** Parse raw number (e.g., `€500`). Store as-is.
- **Single transfer value (no range):** `low` = `high`.
- **Position with no side qualifier:** Valid. `sides: []`.
- **Unrecognized footedness label:** Score defaults to 3 (middle), warning logged.
- **Unrecognized nationality code:** Store code as name (fallback display).
- **BOM prefix:** Stripped before header parsing.

## Failure Modes and Degradation

| Failure | Detection | Response | Recovery |
|---|---|---|---|
| File unreadable | IO error on open | Return error to frontend | User closes file / fixes permissions |
| Not a valid CSV | < 3 fields in header | Return error with message | User provides correct file |
| Missing required column | Header map check | Reject entire import | User exports with correct FM view |
| Row missing UID/name/position | Per-row validation | Skip row, add to summary | User can re-import after fixing |
| Field parse failure | Type conversion error | Store `None`, add warning | Data available, degraded field shown as N/A |
| Division by zero (per-90) | minutes = 0 | Store `None` for computed field | Normal — player hasn't played |
| Duplicate import (same UID + date) | Database lookup in `save_import` | Skip silently | N/A — data is identical |
| Parser panic / crash | Rust panic | Tauri returns error to frontend | User retries |

## Architecture

### Component Design

```
src-tauri/src/
├── commands/
│   ├── mod.rs              // command registration
│   └── csv_parser.rs       // parse_csv, save_import handlers
├── parser/
│   ├── mod.rs              // module root, orchestrates parse pipeline
│   ├── types.rs            // ParsedPlayer, ParseResult, enums, stat structs
│   ├── headers.rs          // BOM strip, delimiter detect, header map
│   ├── fields.rs           // per-field parsing functions
│   ├── positions.rs        // position string parser
│   ├── metrics.rs          // computed metrics (per-90, ratios)
│   └── countries.rs        // static country code lookup
├── storage/
│   └── mod.rs              // persistence (separate concern)
├── lib.rs                  // Tauri builder, register commands
└── main.rs                 // entry point
```

### Data Flow

```
CSV File
  │
  ▼
parse_csv(path, date)
  │
  ├─► read file, detect delimiter, strip BOM
  │
  ├─► parse headers → column map
  │     └─► check required columns
  │
  ├─► iterate rows
  │     ├─► extract fields → typed values
  │     ├─► validate (UID, name, position)
  │     └─► collect into ParsedPlayer or SkippedRow
  │
  ├─► compute metrics (per-90, ratios) for each ParsedPlayer
  │
  └─► return ParseResult
        │
        ▼
Frontend summary screen
        │
        ▼ user confirms
save_import(players, date)
  │
  ├─► for each player: check UID+date uniqueness
  │     └─► persist snapshot to database
  │
  └─► return success
```

### Error Handling

- **File-level errors** (unreadable, not CSV, missing required columns) → returned as `Result::Err` from `parse_csv`. Frontend shows error message.
- **Row-level errors** → collected in `ParseResult.skipped_rows`. Frontend shows in summary.
- **Field-level warnings** → collected in `ParseResult.warnings`. Frontend shows in summary.
- **Persistence errors** → returned as `Result::Err` from `save_import`. Frontend shows error message.

## Invariants

1. Every `ParsedPlayer` in `ParseResult` has a valid UID (positive `u32`), non-empty name, and at least one parseable `Position`.
2. No two `ParsedPlayer` entries in a single `ParseResult` share the same UID.
3. All per-90 computed fields are `None` if and only if minutes is `0` or `None`.
4. All ratio computed fields are `None` if and only if the denominator field is `0` or `None`.
5. `wage_per_week` is `None` if and only if the raw wage string is `None` or unparseable.
6. `transfer_value_low ≤ transfer_value_high` when both are present.
7. `parse_csv` has no side effects — it never writes to the database or modifies global state.
8. `save_import` is idempotent — calling it twice with the same data produces the same database state.
9. Footedness scores are in the range 1-5. Unrecognized labels default to 3.

## Tech Stack

- **`csv` crate** (BurntSushi) — RFC 4180 compliant CSV parsing with `StringRecord` API.
- **`chrono` crate** — `NaiveDate` for date handling.
- **`serde` / `serde_json`** — serialization of `ParsedPlayer` and `ParseResult` across Tauri IPC bridge.
- **`phf` or static `HashMap`** — compile-time country code lookup table.

No other new dependencies anticipated.

## Testing Strategy

- **Unit tests — field parsing:** Test each field parser in isolation with known inputs. Cover: valid values, empty strings, unexpected formats, boundary values (0, negative, very large).
- **Unit tests — position parsing:** Test all observed FM position formats: `GK`, `D (LC)`, `AM (C), ST (C)`, `D/WB (L)`, `M/AM (C)`, `AM (RLC)`, empty string. Verify correct `Role`/`Side` enum output.
- **Unit tests — metric computation:** Test per-90 and ratio calculations. Cover: normal values, minutes = 0, attempted = 0, `None` inputs.
- **Unit tests — financial parsing:** Test `€62M - €94M`, `€57M`, `€38.5K p/w`, `€1.2M p/w`, `€500`, values with no currency symbol.
- **Integration test — full parse:** Feed the sample CSV (`docs/notes/test-files/Test_CSV_2026_04_29.csv`) through `parse_csv`. Assert: 257 players, 0 skipped rows, all fields correctly populated, all computed metrics match expected values.
- **Integration test — malformed CSV:** Construct CSVs with missing columns, bad rows, duplicate UIDs, empty data. Assert correct error/warning collection.
- **E2E test — import flow:** (Future, requires UI) Select file → provide date → review summary → confirm save → verify database state.

## Success Criteria

1. The parser correctly handles the sample CSV — all 257 data rows parsed, zero skips, all computed metrics accurate.
2. A CSV with missing optional columns (no CA, no PA) imports cleanly with warnings, no errors.
3. A non-CSV file is rejected immediately with a clear error message.
4. An empty CSV (headers only) returns 0 players with no error.
5. A CSV with mixed valid and invalid rows imports the valid rows and reports the invalid ones.
6. All 8 stat categories are populated with correct values for known columns.
7. Per-90 values match manual calculation (e.g., goals_per_90 = goals / minutes * 90).
8. Duplicate import (same UID + date) is idempotent — no duplicate records.
9. The parser completes a 3,000-row CSV in under 1 second.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| FM changes column names in a new version | Flexible header matching with warnings for unrecognized columns. No hard-coded column indices. |
| FM adds new columns | Ignored gracefully — listed in `columns_found` but not mapped. |
| FM changes delimiter | Auto-detection from header row. |
| FM changes date format | Parse `dd/mm/yyyy` with `dd/mm/yy` fallback. Frontend date picker for in-game date eliminates user-input format issues. |
| Extremely large CSV (500k rows) | Rust handles this easily. IPC payload may be large but manageable on desktop. Not a realistic use case. |
| Parser performance regression | All parsing is O(n) with no allocations per field beyond string storage. Hard to regress. |
| Country code lookup missing entries | Unknown codes fall back to displaying the raw code. Not an error. |

## Decision Log

| Decision | Options Considered | Chosen | Rationale |
|---|---|---|---|
| Parser runtime | Rust backend, TypeScript frontend | Rust | CPU-bound work, co-located with persistence, trivial performance |
| CSV parsing approach | `csv` crate + StringRecord, `csv` crate + Serde derive, manual split | `csv` crate + StringRecord | RFC 4180 compliance, robust against column renames, explicit field mapping |
| Header matching | Strict (all columns required), flexible (name-based with floor) | Flexible with required floor | Survives FM version changes; rejects only when critical columns missing |
| Season vs snapshot storage | One snapshot per season (overwrite), preserve all snapshots | Preserve all | Richer timeline data; active snapshot = latest date; can be changed trivially later |
| Currency handling | Normalize to single currency, store as-is | Store as-is | Comparisons are relative (percentiles); no exchange rate data needed |
| Wage normalization | Store raw only, store raw + normalized per-week | Raw + per-week | Enables value-for-money comparisons without re-computation |
| Transfer value range | Store single value, store low + high separately | Store low + high | More data for display; use high for calculations |
| Error handling | Fail-fast, skip-and-collect, skip silently | Skip and collect | One bad row doesn't waste 257 good ones; user gets full visibility |
| Persistence timing | Auto-save after parse, user confirmation gate | User confirmation | Prevents unwanted data from entering database |
| Computed metrics scope | Parse only (defer computation), parse + compute | Parse + compute (separate function) | All inputs available at import time; pre-computed values simplify downstream queries; separate function maintains testability |
| Position storage | Raw strings, structured enums | Structured enums | Archetype matching needs enum comparison |
| Footedness storage | String only, numeric score only, both | Both (label + score) | Score for comparisons, label for display |
| Duplicate import handling | Reject, overwrite, skip silently | Skip silently | Idempotent; identical data, no harm |
| Required per-row fields | UID only, UID + name, UID + name + position | UID + name + position | Without position, player can't be matched to archetypes — unusable in the app |
