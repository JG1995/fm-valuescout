# Database Integration - Design Spec

## Context

FM ValueScout is a Tauri v2 desktop app with a Rust backend and Svelte 5 frontend. The CSV parser feature (already implemented) produces `ParsedPlayer` structs from Football Manager exports, but the storage layer is currently an honest stub that returns an error on every call. No data persists between sessions.

The database integration feature replaces that stub with real SQLite-backed persistence, enabling the core app workflow: import CSV data, accumulate seasonal snapshots, and build career timelines. It also introduces save-game isolation so users can maintain separate datasets for different FM playthroughs.

Every downstream feature — scouting, squad management, player profiles, career timelines — depends on this persistence layer existing.

## Problem Statement

The app can parse CSV files into structured player data, but cannot store any of it. Each session starts empty. The `save_import` function in `src-tauri/src/storage/mod.rs` is a stub that returns `Err("Storage is not yet implemented. Your data has not been saved.")`. Without persistence, the app is a stateless parser with no memory.

Additionally, Football Manager players may run multiple saves (different careers, different leagues). Without save-game isolation, data from different playthroughs would mix into one pool with no way to distinguish them.

## Assumptions and Constraints

- **SQLite is the right database for this app.** Single-user desktop app, fully offline, data sizes in the low-MB range. No network, no concurrent users, no server component. SQLite is the standard choice for Tauri apps.
- **`rusqlite` is the right Rust SQLite crate.** Mature, well-maintained, widely used in the Tauri ecosystem. The `bundled` feature simplifies builds by shipping SQLite with the crate.
- **FM UIDs are unique within a single season export but NOT across an entire save's lifetime.** Newgen UIDs can be reused after a player retires. The app must generate its own stable internal player identity.
- **The app is single-user, single-window.** No concurrent access concerns. One import at a time. One active save at a time.
- **CSV file sizes are typically 200–500 rows, max ~500,000 rows.** The theoretical max is a pathological case; realistic data is small enough that full-table scans in Rust are fast.
- **Player identity matching by `fm_uid + name` is sufficient.** FM player names do not change between seasons in practice. UID reuse always produces a different person with a different name.
- **The July→June heuristic for season labeling is acceptable for most leagues.** Non-European leagues (e.g., Norway, Sweden) may get technically incorrect labels, but the user can rename seasons manually.
- **Data is fully offline.** No sync, no cloud backup, no external services. All data lives on the user's machine.

### Hard Constraints

- Must not modify the CSV parser. The parser is a pure function; storage is a separate concern.
- Must not break existing `parse_csv` or `save_import` Tauri command signatures without migration.
- Must not write to the database during `parse_csv` — it remains pure.
- Must be fully offline-capable. No network calls, no cloud dependencies.
- Must use transactions for all write operations. No partial writes on failure.
- Must be backwards-compatible with existing tests (113 passing).

## Anti-Requirements

- **No squad management, archetypes, or user preferences storage.** Those are separate features with their own storage needs. This feature only persists player + season data.
- **No schema migration framework.** The database is created fresh. Future schema changes will be handled by explicit versioned migrations when needed.
- **No import progress indicator.** Imports are instant for realistic file sizes (<1 second for 500 rows).
- **No data export or backup.** The user can re-import from the original CSV. Backup/restore is a separate feature.
- **No multi-window or concurrent access.** Single-user desktop app. One operation at a time.
- **No automatic season detection from CSV data.** The user provides the in-game date; the app derives the season label from it.
- **No fuzzy name matching for player identity.** Exact case-insensitive match on `fm_uid + name`. Close-but-not-exact matches create new player records. This is conservative — a split timeline is recoverable; a merged timeline is not.

## Feature Scope

### Must-haves (Phase 1)

#### Save-game management

- Purpose: Isolate data between different FM playthroughs. Each save has its own set of players, seasons, and career timelines.
- Behavior:
  - Create a new save with a name. Save names must be unique (case-insensitive).
  - List all saves with metadata (name, created date, season count, player count).
  - Rename a save.
  - Delete a save and all associated data (cascade: seasons → player_seasons, then orphaned players).
- Failure mode:
  - Duplicate name → `Err("A save with the name 'My Save' already exists.")`.
  - Delete non-existent save → `Err("Save not found.")`.
  - Empty name → `Err("Save name cannot be empty.")`.
- Decisions made: The `managed_club` column is added as nullable and defaulted to NULL. No UI for setting it in this feature — it's prepared for the squad feature.

#### Season import with persistence

- Purpose: Replace the `save_import` stub. Store parsed player data as a named season snapshot within a save.
- Behavior:
  1. Frontend calls `parse_csv` (existing, unchanged) → gets `ParseResult`.
  2. Frontend shows preview (players, skipped rows, warnings).
  3. Frontend calls `import_season(save_id, players, in_game_date)` → backend persists.
  4. Backend checks if a season with this `in_game_date` already exists in the save → rejects if so.
  5. Backend creates a season record with auto-derived label (e.g., "2025/26" from July→June heuristic).
  6. For each player in the import:
     - Lookup existing player in this save by `(fm_uid, name)`.
     - No match → create new player record, create player_season record.
     - Match found → create player_season record linked to existing player.
     - Same fm_uid, different name → create new player record (UID reuse by FM).
  7. All inserts within a single transaction. All-or-nothing.
  8. Return `ImportResult` with counts: total players, new players, matched players, created season info.
- Failure mode:
  - Season already exists → `Err("Season for 2030-06-30 already exists (256 players). Delete it first to re-import.")`.
  - Save not found → `Err("Save not found.")`.
  - Empty players list → `Err("No players to import.")`.
  - Invalid in_game_date → `Err("Invalid date format. Expected YYYY-MM-DD.")`.
  - DB error mid-import → transaction rolls back, no partial data, `Err` with details.
- Decisions made: The import flow is two-step (parse then persist) to allow preview. The frontend is responsible for the save_id — it tracks which save is active.

#### Season management

- Purpose: List, rename, and delete seasons within a save.
- Behavior:
  - List all seasons for a save, ordered by in_game_date ascending.
  - Rename a season (updates the display label only).
  - Delete a season and all associated player_seasons. Clean up orphaned player records (players with no remaining seasons in this save).
- Failure mode:
  - Season not found → `Err("Season not found.")`.
  - Empty rename → `Err("Season name cannot be empty.")`.

#### Data retrieval

- Purpose: Load stored player data for display, scouting, and career timelines.
- Behavior:
  - `get_players_for_season(season_id)` → all player_season records for a season, including deserialized stats from JSON.
  - `get_player_career(save_id, player_id)` → all player_season records for a player across all seasons in a save, ordered by in_game_date ascending. Used for career timeline.
  - `get_latest_season(save_id)` → the most recent season in a save, or None.
- Failure mode:
  - Season not found → empty Vec (not an error — the season simply has no data).
  - Player not found → empty Vec.
  - Save not found → empty Vec.
  - JSON deserialization failure → skip the record, log warning. Return partial results with a warning. This handles the case where a future app version writes data that an older version can't read.

#### Database initialization

- Purpose: Create the SQLite database and schema on first launch.
- Behavior:
  - DB file stored in Tauri's app data directory (via `app_data_dir`).
  - On startup, check if DB exists. If not, create it with the full schema.
  - Schema creation is idempotent — uses `CREATE TABLE IF NOT EXISTS`.
- Failure mode:
  - Cannot create/open DB file → `Err("Unable to initialize database. Check app permissions.")`. This is a fatal startup error.

### Nice-to-have (Phase 2+, defer)

- **Player search/query:** Full-text search on player name, club, nationality. Currently the frontend loads all players for a season and filters client-side. Sufficient for current scale.
- **Season statistics summary:** Aggregated stats per season (average rating, top scorer, etc.). Can be computed client-side from loaded data.
- **Data export:** Export stored data back to CSV or JSON. The user always has the original CSV.
- **Save-game metadata:** League, difficulty, etc. The `managed_club` column is prepared; other metadata can be added later.
- **Orphan cleanup utility:** Explicit command to clean up orphaned player records. Currently handled implicitly during season deletion.

## Integration Points

### Existing Code Affected

- `src-tauri/src/storage/mod.rs` — Complete rewrite. Currently 34 lines (stub + 2 tests). Becomes the main database module with connection management, schema creation, and all CRUD operations.
- `src-tauri/src/commands/csv_parser.rs` — Remove the `save_import` command (stub). It moves to the new `commands/storage.rs` as `import_season`. The `parse_csv` command is unchanged.
- `src-tauri/src/lib.rs` — Add database state initialization (`DbState`) to the Tauri builder. Add new command handlers from `commands/storage.rs` to `generate_handler![]`. Remove `save_import` from handler registration.
- `src-tauri/Cargo.toml` — Add `rusqlite` dependency with `bundled` feature.
- `src-tauri/src/parser/types.rs` — No changes needed. `ParsedPlayer` and all stat structs already derive `Serialize`/`Deserialize`, which is sufficient for JSON storage.

### New Code Required

- `src-tauri/src/storage/mod.rs` — Rewrite. Connection management, schema DDL, save CRUD, season CRUD, import logic with player matching, data retrieval with JSON deserialization.
- `src-tauri/src/commands/storage.rs` — New file. Tauri command wrappers for save and season management (separate from csv_parser commands for clarity).
- `src-tauri/tests/integration_storage.rs` — New file. Integration tests against a real SQLite database (in-memory or temp file).
- `src-tauri/tests/edge_case_storage.rs` — New file. Edge case tests for UID reuse, duplicate imports, cascade deletes, JSON deserialization failures.

### Data Changes

- New SQLite database file in Tauri app data directory.
- No changes to existing data structures (`ParsedPlayer`, stat structs, etc.). They are serialized to JSON as-is.

### API Changes

The Tauri command surface changes from:

```
parse_csv(file_path, in_game_date) → Result<ParseResult, String>       // EXISTING, unchanged
save_import(players, in_game_date) → Result<(), String>                 // STUB — REMOVED
```

To:

```
// Existing, unchanged:
parse_csv(file_path, in_game_date) → Result<ParseResult, String>

// Replaces save_import:
import_season(save_id, players, in_game_date) → Result<ImportResult, String>

// New save management:
create_save(name) → Result<Save, String>
list_saves() → Result<Vec<Save>, String>
rename_save(save_id, name) → Result<(), String>
delete_save(save_id) → Result<(), String>

// New season management:
list_seasons(save_id) → Result<Vec<Season>, String>
rename_season(season_id, name) → Result<(), String>
delete_season(season_id) → Result<(), String>

// New data retrieval:
get_players_for_season(season_id) → Result<Vec<PlayerSeasonData>, String>
get_player_career(save_id, player_id) → Result<Vec<PlayerSeasonData>, String>
get_latest_season(save_id) → Result<Option<Season>, String>
```

## Edge Cases and Boundary Conditions

- **FM UID reuse across seasons:** Player with UID 12345 named "John Smith" exists in seasons 1-5. In season 6, UID 12345 appears as "Carlos Garcia" (newgen reuse). Resolution: Name mismatch triggers a new `players` record. Both records have `fm_uid = 12345` but different internal IDs. Career timelines are separate — correct behavior.

- **Same player imported twice in one CSV (duplicate UID in file):** The CSV parser already handles this — duplicate UIDs within a single CSV are rejected at parse time (skipped row). The storage layer receives deduplicated players from `ParseResult`.

- **Season with zero valid players:** `import_season` rejects with `Err("No players to import.")`. No season record is created.

- **Import interrupted mid-transaction (app crash):** SQLite transaction rolls back automatically. No partial data. Next launch finds a clean state.

- **Very large import (500K players):** All inserts happen in a single transaction. SQLite handles this efficiently — transaction commit is atomic. Expected performance: well under 5 seconds for 500K rows.

- **Empty save name:** Rejected at command level with `Err("Save name cannot be empty.")`. Whitespace-only names are also rejected.

- **Save name exceeds reasonable length:** Enforce max 100 characters. `Err("Save name must be 100 characters or fewer.")`.

- **Special characters in save name:** Allowed. SQLite handles quoting. Display in UI is the frontend's concern.

- **In-game date edge cases (Feb 29, etc.):** Validate with `NaiveDate::parse_from_str`. Invalid dates rejected with `Err("Invalid date format. Expected YYYY-MM-DD.")`.

- **In-game date that falls exactly on July 1st:** Treated as start of new season. "2030-07-01" → label "2030/31".

- **JSON deserialization failure on read (future schema change):** Skip the record, include in a `warnings` field on the result. Return partial data. This degrades gracefully — the user sees most of their data with a warning about unreadable records.

- **Delete a season that doesn't exist:** `Err("Season not found.")`.

- **Delete the only season in a save:** Season deleted. All player_seasons deleted. Orphaned player records (no remaining seasons) are deleted. Save remains with zero seasons.

- **Rename season to same name:** No-op success. Not an error.

- **Multiple saves with same name:** Rejected at creation. `Err("A save with the name 'X' already exists.")`. Case-insensitive comparison.

- **Player with empty name in CSV:** Already rejected by the CSV parser (hard reject, row skipped). Storage layer never receives empty-named players.

## Failure Modes and Degradation

- **Database file corrupted:** SQLite's integrity check (`PRAGMA integrity_check`) can detect this. On detection, the app should inform the user and offer to recreate the database (losing all data). This is a rare catastrophic failure — the user's original CSVs are their backup.
- **Disk full during import:** SQLite returns an error. Transaction rolls back. `Err("Unable to save data. Disk may be full.")`.
- **Permission denied on app data directory:** Fatal startup error. App cannot function without database. `Err("Unable to initialize database. Check app permissions.")`.
- **Concurrent file access (another instance of the app):** SQLite handles this with WAL mode. Second instance gets a busy error. Single-user desktop app makes this extremely unlikely.
- **JSON blob grows too large (pathological case):** Each player_season's JSON blob is ~2-4KB for 80+ stats. Even 500K players × 4KB = 2GB — well within SQLite's limits. Not a realistic concern.

## Architecture

### Component Design

```
src-tauri/src/
├── commands/
│   ├── csv_parser.rs     (existing, parse_csv unchanged)
│   ├── mod.rs            (add storage module)
│   └── storage.rs        (NEW - save/season/data Tauri commands)
├── storage/
│   └── mod.rs            (REWRITE - DB connection, schema, all CRUD)
├── parser/               (unchanged)
└── lib.rs                (add DB state init, new command handlers)
```

**Storage module responsibilities:**

| Responsibility | Function |
|---|---|
| DB lifecycle | `init_db(app_handle)` — create/open DB, initialize schema |
| Save CRUD | `create_save`, `list_saves`, `rename_save`, `delete_save` |
| Season CRUD | `create_season`, `list_seasons`, `rename_season`, `delete_season` |
| Import | `import_season` — player matching + batch insert in transaction |
| Retrieval | `get_players_for_season`, `get_player_career`, `get_latest_season` |
| Schema | DDL constants for table creation |

**Tauri state management:**

The SQLite connection is wrapped in a `Mutex<Connection>` and managed by Tauri's state system. Initialized once in `lib.rs::run()`, accessible to all command handlers via `tauri::State`.

```rust
pub struct DbState {
    pub conn: Mutex<rusqlite::Connection>,
}
```

### Data Flow

```
Frontend                          Backend (Tauri commands)
────────                          ────────────────────────
User selects save
     │
     ▼
User imports CSV
     │
     ├──> parse_csv(file_path) ──> parser::parse_csv (existing, unchanged)
     │◄── ParseResult ◄─────────
     │
     ▼
Frontend shows preview
     │
     ├──> import_season(save_id, players, date) ──> storage::import_season
     │    │                                            │
     │    │                                            ├─ Begin transaction
     │    │                                            ├─ Check duplicate season → Err if exists
     │    │                                            ├─ Create season record
     │    │                                            ├─ For each player:
     │    │                                            │   ├─ Match by (fm_uid, name) in save
     │    │                                            │   ├─ Create or reuse player record
     │    │                                            │   └─ Create player_season record
     │    │                                            ├─ Commit transaction
     │    │                                            └─ Return ImportResult
     │◄── ImportResult ◄─────────────────────────────
     │
     ▼
Frontend shows import summary
```

### Schema

```sql
CREATE TABLE IF NOT EXISTS saves (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,
    managed_club TEXT,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS seasons (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id      INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    in_game_date TEXT    NOT NULL,
    label        TEXT    NOT NULL,
    imported_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE(save_id, in_game_date)
);

CREATE TABLE IF NOT EXISTS players (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    fm_uid  INTEGER NOT NULL,
    name    TEXT    NOT NULL,
    UNIQUE(save_id, fm_uid, name)
);

CREATE TABLE IF NOT EXISTS player_seasons (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    player_id           INTEGER NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    season_id           INTEGER NOT NULL REFERENCES seasons(id) ON DELETE CASCADE,
    club                TEXT,
    age                 INTEGER,
    nationality         TEXT,
    position            TEXT    NOT NULL,
    minutes             INTEGER,
    appearances_started INTEGER,
    appearances_sub     INTEGER,
    wage_per_week       REAL,
    transfer_value_high REAL,
    contract_expires    TEXT,
    data                TEXT    NOT NULL,  -- JSON blob: full ParsedPlayer stats
    UNIQUE(player_id, season_id)
);

-- Indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_seasons_save_id ON seasons(save_id);
CREATE INDEX IF NOT EXISTS idx_players_save_uid ON players(save_id, fm_uid);
CREATE INDEX IF NOT EXISTS idx_player_seasons_player ON player_seasons(player_id);
CREATE INDEX IF NOT EXISTS idx_player_seasons_season ON player_seasons(season_id);
```

### Season Label Derivation

```rust
fn derive_season_label(in_game_date: &str) -> Result<String, String> {
    let date = NaiveDate::parse_from_str(in_game_date, "%Y-%m-%d")
        .map_err(|_| "Invalid date format. Expected YYYY-MM-DD.".to_string())?;
    let (year, month) = (date.year(), date.month());
    if month >= 7 {
        // July-December: season is year/year+1
        Ok(format!("{}/{}", year, (year + 1) % 100))
    } else {
        // January-June: season is year-1/year
        Ok(format!("{}/{}", year - 1, year % 100))
    }
}
```

Examples:
- "2030-11-15" → "2030/31"
- "2030-06-30" → "2029/30"
- "2030-07-01" → "2030/31"
- "2030-01-15" → "2029/30"

### Error Handling

All storage functions return `Result<T, String>`, consistent with the existing pattern in `commands/csv_parser.rs`. Error strings are user-facing messages that the frontend can display directly.

The `storage` module functions return a specific error type internally:

```rust
#[derive(Debug)]
pub enum StorageError {
    NotFound(String),
    Duplicate(String),
    Validation(String),
    Database(String),
}
```

Command wrappers convert `StorageError` to `String` for the Tauri boundary.

## Invariants

- **(I1)** A season's `in_game_date` is unique within a save. Attempting to create a duplicate is rejected.
- **(I2)** A `player_season` row is unique per `(player_id, season_id)`. A player cannot have two seasonal records for the same season.
- **(I3)** A player's `fm_uid + name` is unique within a save. Different names with the same UID produce different player records.
- **(I4)** Every `player_season` references exactly one `player` and one `season`. No orphan player_seasons.
- **(I5)** Every `season` references exactly one `save`. No orphan seasons.
- **(I6)** Every `player` references exactly one `save`. No orphan players.
- **(I7)** All write operations are atomic within a transaction. No partial writes.
- **(I8)** `parse_csv` remains a pure function with no side effects. It never touches the database.
- **(I9)** The `data` JSON blob in `player_seasons` can be deserialized into a valid `ParsedPlayer`-compatible struct. If deserialization fails, the record is skipped with a warning (degradation, not failure).

## Tech Stack

| Dependency | Version | Purpose |
|---|---|---|
| `rusqlite` | latest | SQLite bindings for Rust |
| `rusqlite` feature `bundled` | — | Ships SQLite with the crate, no system dependency |
| `serde_json` | existing | JSON serialization for stat blobs |
| `chrono` | existing | Date parsing for season label derivation |

No other new dependencies. All existing dependencies (`csv`, `phf`, `serde`, `tauri`) remain unchanged.

## Testing Strategy

- **Unit tests (in `storage/mod.rs`):**
  - Season label derivation (July→June boundary, edge dates)
  - Player matching logic (new player, existing match, UID reuse with name mismatch)
  - Save name validation (empty, too long, duplicate)
  - Date validation (invalid format, Feb 29)
  - JSON serialization/deserialization round-trip

- **Integration tests (`tests/integration_storage.rs`):**
  - Full import flow against in-memory SQLite: create save → import season → retrieve players → verify data integrity
  - Career timeline: import two seasons → retrieve career → verify both seasons present
  - UID reuse scenario: import season 1 with player A (UID 12345, "John") → import season 5 with player B (UID 12345, "Carlos") → verify two distinct player records
  - Duplicate season rejection: import same date twice → verify error
  - Cascade delete: create save with seasons and players → delete save → verify all data removed
  - Season delete cleanup: import two seasons → delete one → verify remaining season intact, orphaned players cleaned up

- **Edge case tests (`tests/edge_case_storage.rs`):**
  - Empty player list import
  - Save with zero seasons (list seasons returns empty)
  - Player career with one season (single entry timeline)
  - Rename save/season to same name (no-op)
  - Delete non-existent save/season
  - Special characters in save name
  - JSON deserialization failure simulation (write invalid JSON, verify graceful degradation)

- **Existing tests:** All 113 existing tests must continue to pass. The `save_import` stub tests in `storage/mod.rs` are replaced with tests for the new `import_season` function.

## Success Criteria

- The `save_import` stub is replaced with working persistence.
- A user can create a save, import a CSV as a season, and retrieve the data in a subsequent session.
- Career timelines work: importing two seasons for the same FM UID produces a two-entry timeline.
- UID reuse is handled correctly: different players with the same FM UID get separate records.
- Duplicate season imports are rejected with a clear error message.
- All existing tests (113) continue to pass.
- All new storage tests pass.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| SQLite file corruption | SQLite WAL mode for durability. Inform user, offer to recreate. Original CSVs are backup. |
| Schema migration needed in future | Simple versioned approach: check `PRAGMA user_version`, run Deltas. Not built now, but the architecture supports it. |
| JSON blob format changes between app versions | Deserialize with `Default` fallback for missing fields. Skip unreadable records with warning, don't fail the whole query. |
| Large imports (500K rows) are slow | Batch inserts in a single transaction. SQLite handles 500K inserts in <5 seconds. Acceptable for MVP. |
| `rusqlite` bundled feature increases binary size | SQLite adds ~2-3MB. Acceptable for a desktop app. |
| Player matching creates false splits (same player, slightly different name) | Conservative by design. A split timeline is recoverable; a merged timeline is not. User can report and we can add fuzzy matching later. |

## Decision Log

| Decision | Options Considered | Chosen | Rationale |
|---|---|---|---|
| Database engine | SQLite, sled, redb, plain JSON files | SQLite | Standard for Tauri desktop apps. Mature, queryable, transactional, single-file. |
| SQLite crate | rusqlite, diesel, sqlx, prisma | rusqlite | Lightweight, no ORM overhead, `bundled` feature simplifies builds. Diesel/sqlx add complexity for minimal benefit at this scale. |
| Player identity across seasons | Flat (one row per season), Master + snapshot | Master + snapshot | FM UID reuse makes flat model unreliable. Master table provides stable identity. |
| UID reuse detection | fm_uid only, fm_uid + name, fm_uid + name + age, manual confirmation | fm_uid + name (case-insensitive) | Name is stable in FM. Sufficient for disambiguation. Conservative — false splits are recoverable, false merges are not. |
| Stat storage | 80+ flat columns, pure JSON, hybrid | Hybrid (~12 columns + JSON blob) | Queryable identity/financial fields, flexible stat storage. Avoids schema migrations when FM changes stats. |
| Import idempotency | Skip silently, reject with error, upsert, replace | Reject with error | Explicit trumps implicit. User knows exactly what happened. Delete-and-reimport is a deliberate action. |
| Season labeling | Auto-numbered, date only, date-derived + renameable | Date-derived football season format + renameable | Matches FM players' mental model. Rename handles non-European leagues. |
| Save metadata | Minimal (name only), full (club, league, etc.) | Name + prepared managed_club column | YAGNI for full metadata. managed_club costs nothing now and is needed by squad feature. |
| Tauri state for DB | Mutex<Connection>, connection pool, per-request connections | Mutex<Connection> | Single-user app. One connection is sufficient. Mutex prevents concurrent access within the app. |
| Save-game isolation | All data in one pool with save_id filter, separate DB files per save | Single DB with save_id FK | Simpler backup (one file), cross-save queries possible later, SQLite handles FK cascades. |
