# Database Integration Technical Deep-Dive

## Overview & Purpose

The Database Integration feature provides SQLite-backed persistence for FM ValueScout. It replaces the original `save_import` stub with a complete storage layer: save-game management, seasonal data import with player matching, and multi-season career timeline retrieval. Every downstream feature — scouting, squad management, player profiles — depends on this persistence layer.

**Key capabilities:**
- Save-game CRUD with case-insensitive uniqueness and cascade deletes
- Season import with transactional player matching (`fm_uid + name` identity)
- Date-derived season labels (July–June football year heuristic)
- Hybrid schema: ~12 queryable columns for filter/sort + JSON blob for full `ParsedPlayer` stats
- Career timeline retrieval across seasons
- Graceful degradation when JSON blobs fail to deserialize
- All write operations are transactional (atomic)

**Architecture principle:** Storage is a separate concern from parsing. `parse_csv` remains a pure function; the frontend orchestrates the two-step parse-then-persist flow.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Svelte Frontend                              │
│  (orchestrates: parse_csv → preview → import_season)                │
└────────────────────────────┬────────────────────────────────────────┘
                             │  Tauri IPC
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    commands/storage.rs                               │
│  12 Tauri command wrappers — lock Mutex, delegate, convert error    │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      storage/mod.rs                                  │
│  Re-exports — public API boundary                                    │
└──┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬─────────────────┘
   │      │      │      │      │      │      │      │
   ▼      ▼      ▼      ▼      ▼      ▼      ▼      ▼
┌──────┐┌──────┐┌──────┐┌──────┐┌──────┐┌──────┐┌──────┐
│schema││error ││types ││saves ││season││import││retrie│
│.rs   ││.rs   ││.rs   ││.rs   ││s.rs  ││.rs   ││val.rs│
│DDL   ││Stora ││DbSta ││CRUD  ││CRUD  ││Playr ││Read  │
│init  ││geErr ││te,   ││+vali ││+label││match ││queris│
│      ││or    ││DTOs  ││dation││deriv ││+JSON ││+JSON │
└──────┘└──────┘└──────┘└──────┘└──────┘└──────┘└──────┘
```

**Module responsibilities:**

| Module | Lines | Responsibility |
|--------|-------|----------------|
| `mod.rs` | 19 | Re-exports — public API for command wrappers |
| `schema.rs` | 67 | DDL constants, `init_schema`, `init_db`, `init_db_test` |
| `error.rs` | 35 | `StorageError` enum, `Display`, `From<rusqlite::Error>` |
| `types.rs` | 66 | `DbState`, `Save`, `Season`, `ImportResult`, `PlayerSeasonData` |
| `saves.rs` | 126 | `validate_save_name`, `create_save`, `list_saves`, `rename_save`, `delete_save` |
| `seasons.rs` | 175 | `derive_season_label`, `create_season`, `create_season_tx`, `list_seasons`, `rename_season`, `delete_season` |
| `import.rs` | 126 | `format_positions`, `import_season` (transactional batch insert) |
| `retrieval.rs` | 111 | `row_to_player_season`, `get_players_for_season`, `get_player_career`, `get_latest_season` |

## Data Flow

**Step-by-step from frontend call to SQLite round-trip:**

```
1. App Startup (lib.rs::run)
   └─> Tauri setup callback
       └─> app.path().app_data_dir() → resolve DB location
       └─> Create directory if missing
       └─> storage::init_db("...fm_valuescout.db")
           └─> Connection::open(db_path)
           └─> PRAGMA foreign_keys = ON
           └─> CREATE TABLE IF NOT EXISTS ... (all 4 tables + 4 indexes)
           └─> Return Connection
       └─> app.manage(DbState { conn: Mutex::new(conn) })

2. Import Season (user-initiated)
   └─> Frontend calls import_season(save_id, players, in_game_date)
       └─> commands/storage.rs: lock Mutex → get &Connection
       └─> storage::import_season(conn, save_id, date, players)
           ├─> Validate players non-empty
           ├─> Verify save exists (SELECT EXISTS)
           ├─> conn.unchecked_transaction()
           ├─> create_season_tx(&tx, save_id, date)
           │   ├─> derive_season_label(date)
           │   │   └─> NaiveDate::parse_from_str → July→June heuristic
           │   ├─> Check duplicate season (SELECT EXISTS)
           │   └─> INSERT INTO seasons → read back imported_at
           ├─> For each player:
           │   ├─> SELECT id FROM players WHERE (save_id, fm_uid, LOWER(name))
           │   ├─> No match → INSERT INTO players (new record)
           │   ├─> Match found → reuse player_id
           │   ├─> Extract ~12 queryable columns from ParsedPlayer
           │   ├─> Serialize full ParsedPlayer as JSON blob
           │   └─> INSERT INTO player_seasons (player_id, season_id, columns, data)
           ├─> tx.commit() — all-or-nothing
           └─> Return ImportResult { season, total, new, matched }
       └─> MutexGuard dropped → connection released

3. Retrieve Players for Season
   └─> storage::get_players_for_season(conn, season_id)
       └─> SELECT ps.*, p.fm_uid, p.name FROM player_seasons ps
           JOIN players p ON ps.player_id = p.id
           WHERE ps.season_id = ?1 ORDER BY p.name ASC
       └─> For each row: row_to_player_season(row)
           ├─> Named column references: row.get("club"), row.get("data"), ...
           └─> JSON blob: serde_json::from_str(&data_json).ok()
               └─> Failure → data: None (graceful degradation)
```

## Core Types

### StorageError

```rust
#[derive(Debug)]
pub enum StorageError {
    NotFound(String),      // "Save not found.", "Season not found."
    Duplicate(String),     // "A save with the name 'X' already exists."
    Validation(String),    // "Save name cannot be empty."
    Database(String),      // Wrapped rusqlite errors
}
```

Converted to `String` at the Tauri command boundary via `impl From<StorageError> for String`.

### DbState

```rust
/// Tauri-managed state wrapping a single SQLite connection.
pub struct DbState {
    pub conn: Mutex<Connection>,
}
```

Single-user app; `Mutex` prevents concurrent access within the app. Initialized once in `lib.rs::run()`.

### Save

```rust
pub struct Save {
    pub id: i64,
    pub name: String,
    pub managed_club: Option<String>,  // Prepared for squad feature, unused in Phase 1
    pub created_at: String,
    pub season_count: i64,   // Computed via JOIN in list_saves
    pub player_count: i64,   // Computed via JOIN in list_saves
}
```

### Season

```rust
pub struct Season {
    pub id: i64,
    pub save_id: i64,
    pub in_game_date: String,  // "2030-11-15" — the user-provided date
    pub label: String,         // "2030/31" — derived or user-renamed
    pub imported_at: String,   // SQLite datetime('now') at insert time
}
```

### ImportResult

```rust
pub struct ImportResult {
    pub season: Season,
    pub total_players: usize,
    pub new_players: usize,      // Previously unseen (new player record created)
    pub matched_players: usize,  // Existing player record reused
}
```

### PlayerSeasonData

```rust
pub struct PlayerSeasonData {
    pub id: i64,
    pub player_id: i64,
    pub season_id: i64,
    pub fm_uid: i64,
    pub player_name: String,
    // ~12 queryable columns (extracted from ParsedPlayer at import time)
    pub club: Option<String>,
    pub age: Option<i64>,
    pub nationality: Option<String>,
    pub position: String,
    pub minutes: Option<i64>,
    pub appearances_started: Option<i64>,
    pub appearances_sub: Option<i64>,
    pub wage_per_week: Option<f64>,
    pub transfer_value_high: Option<f64>,
    pub contract_expires: Option<String>,
    /// Full player data deserialized from the JSON blob.
    /// None if deserialization fails (graceful degradation).
    pub data: Option<ParsedPlayer>,
}
```

### Type Design Decisions

1. **`StorageError` as internal type, `String` at Tauri boundary:** Rust error types cannot cross the Tauri IPC boundary. The internal enum preserves semantic error categories; command wrappers flatten to strings. This matches the existing pattern in `commands/csv_parser.rs`.

2. **Computed counts on `Save`:** `season_count` and `player_count` are not stored columns — they're computed via `COUNT(DISTINCT ...)` in `list_saves`. This avoids count staleness but means every `list_saves` call runs aggregates.

3. **`Option<ParsedPlayer>` for JSON data:** The `data` field is `Option` rather than requiring successful deserialization. This handles the case where a future app version writes data that an older version can't read — the queryable columns still work.

4. **`i64` for database IDs:** SQLite `INTEGER PRIMARY KEY` maps to `i64` in rusqlite. All ID fields use this consistently.

## Module Deep-Dives

### 1. Schema (`schema.rs`)

**Responsibility:** Define and initialize the database schema.

```rust
const SCHEMA_DDL: &str = "
CREATE TABLE IF NOT EXISTS saves ( ... );
CREATE TABLE IF NOT EXISTS seasons ( ... );
CREATE TABLE IF NOT EXISTS players ( ... );
CREATE TABLE IF NOT EXISTS player_seasons ( ... );
-- 4 indexes for common query patterns
";

pub fn init_schema(conn: &Connection) -> Result<(), StorageError> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(SCHEMA_DDL)?;
    Ok(())
}
```

**Idempotency:** All DDL uses `IF NOT EXISTS`, so calling `init_schema` on an existing database is safe. This simplifies the startup path — no version check needed for the initial release.

**Foreign keys:** `PRAGMA foreign_keys = ON` is set at schema init time and persists for the connection lifetime. SQLite does not enable FK enforcement by default; without this pragma, `ON DELETE CASCADE` is silently ignored.

### 2. Error (`error.rs`)

**Responsibility:** Typed error with `Display` and conversion impls.

```rust
impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::NotFound(msg) => write!(f, "{}", msg),
            // ... each variant just forwards the message
        }
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(err: rusqlite::Error) -> StorageError {
        StorageError::Database(err.to_string())
    }
}
```

The `From<rusqlite::Error>` impl allows using `?` on any rusqlite operation within storage functions. All rusqlite errors are bucketed as `Database` variant — we don't distinguish constraint violations from disk errors internally.

### 3. Types (`types.rs`)

**Responsibility:** Data transfer objects and `DbState`.

All types derive `Debug, Clone, Serialize, Deserialize` for Tauri IPC serialization. `DbState` does not derive these — it wraps a `Mutex<Connection>` and is managed by Tauri's state system.

### 4. Saves (`saves.rs`)

**Responsibility:** Save-game CRUD with validation.

**Name validation:**
```rust
fn validate_save_name(name: &str) -> Result<String, StorageError> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err(StorageError::Validation("Save name cannot be empty.".to_string()));
    }
    if trimmed.len() > 100 {
        return Err(StorageError::Validation("Save name must be 100 characters or fewer.".to_string()));
    }
    Ok(trimmed)
}
```

Names are trimmed before storage. The 100-character limit is arbitrary but reasonable for a save-game name.

**Case-insensitive uniqueness:**
```rust
let exists: bool = conn.query_row(
    "SELECT EXISTS(SELECT 1 FROM saves WHERE LOWER(name) = LOWER(?1))",
    rusqlite::params![name],
    |row| row.get(0),
)?;
```

`LOWER()` comparison handles Unicode correctly for this use case. The UNIQUE constraint on the `name` column provides a database-level backstop, but the application-level check gives a friendlier error message.

**List saves with counts:**
```rust
"SELECT s.id, s.name, s.managed_club, s.created_at,
        COUNT(DISTINCT se.id) AS season_count,
        COUNT(DISTINCT p.id) AS player_count
 FROM saves s
 LEFT JOIN seasons se ON se.save_id = s.id
 LEFT JOIN players p ON p.save_id = s.id
 GROUP BY s.id
 ORDER BY s.created_at DESC"
```

Three-table LEFT JOIN with DISTINCT counts. `LEFT JOIN` ensures saves with zero seasons appear. `ORDER BY created_at DESC` shows most recent saves first.

**Cascade delete:** `delete_save` relies on the schema's `ON DELETE CASCADE`. Deleting from `saves` cascades to `players` and `seasons`; cascading from `seasons` deletes `player_seasons`.

### 5. Seasons (`seasons.rs`)

**Responsibility:** Season CRUD, label derivation, and orphan player cleanup.

**Season label derivation:**
```rust
pub fn derive_season_label(in_game_date: &str) -> Result<String, StorageError> {
    let date = chrono::NaiveDate::parse_from_str(in_game_date, "%Y-%m-%d")
        .map_err(|_| StorageError::Validation("Invalid date format. Expected YYYY-MM-DD.".to_string()))?;
    let (year, month) = (date.year(), date.month());
    if month >= 7 {
        Ok(format!("{}/{:02}", year, (year + 1) % 100))
    } else {
        Ok(format!("{}/{:02}", year - 1, year % 100))
    }
}
```

July–December is "this year / next year (mod 100)". January–June is "last year / this year (mod 100)". The `{:02}` formatting ensures two-digit year suffix (e.g., "2099/00" at century boundary). Date validation rejects invalid dates like February 29 on non-leap years.

**Transactional season creation:**
```rust
pub(crate) fn create_season_tx(tx: &Transaction, save_id: i64, in_game_date: &str) -> Result<Season, StorageError>
```

`create_season_tx` takes a `&Transaction` (not `&Connection`) so it can participate in a larger transaction (e.g., `import_season`). The public `create_season` wraps it with a save-existence check and its own transaction for standalone use.

**Duplicate season detection with player count:**
```rust
let player_count: i64 = tx.query_row(
    "SELECT COUNT(*) FROM player_seasons WHERE season_id = \
     (SELECT id FROM seasons WHERE save_id = ?1 AND in_game_date = ?2)",
    rusqlite::params![save_id, in_game_date],
    |row| row.get(0),
).unwrap_or(0);
return Err(StorageError::Duplicate(format!(
    "Season for {} already exists ({} players). Delete it first to re-import.",
    in_game_date, player_count
)));
```

The error message includes the player count so the user knows what they'd lose by deleting.

**Orphan player cleanup on season delete:**
```rust
pub fn delete_season(conn: &Connection, season_id: i64) -> Result<(), StorageError> {
    let tx = conn.unchecked_transaction()?;
    // ... delete player_seasons, delete season ...
    tx.execute(
        "DELETE FROM players WHERE save_id = :save_id AND id NOT IN \
         (SELECT DISTINCT player_id FROM player_seasons \
          JOIN seasons ON player_seasons.season_id = seasons.id \
          WHERE seasons.save_id = :save_id)",
        rusqlite::named_params!{":save_id": save_id},
    )?;
    tx.commit()?;
    Ok(())
}
```

After deleting a season, any player record that has no remaining `player_seasons` entries in the save is deleted. Players shared with other seasons are preserved. The entire operation is wrapped in a transaction to prevent partial state.

### 6. Import (`import.rs`)

**Responsibility:** Transactional season import with player matching.

**Position formatting:**
```rust
fn format_positions(positions: &[Position]) -> String {
    positions.iter().map(|p| {
        let sides = p.sides.iter().map(|s| match s {
            Side::L => "L", Side::C => "C", Side::R => "R",
        }).collect::<Vec<_>>().join(", ");
        format!("{:?} ({})", p.role, sides)
    }).collect::<Vec<_>>().join(", ")
}
```

Converts typed `Vec<Position>` back to a readable string like `"AM (L, C), ST (C)"` for the queryable `position` column.

**Player matching:**
```rust
let existing_player_id: Option<i64> = tx.query_row(
    "SELECT id FROM players WHERE save_id = ?1 AND fm_uid = ?2 AND LOWER(name) = LOWER(?3)",
    rusqlite::params![save_id, player.uid as i64, player.name],
    |row| row.get(0),
).ok();
```

Identity is `(save_id, fm_uid, LOWER(name))` — case-insensitive name match. FM UIDs can be reused by different players (newgens), so `fm_uid` alone is insufficient. Same UID with different name creates a new player record — conservative by design (a split timeline is recoverable; a merged timeline is not).

**JSON blob storage:**
```rust
let data_json = serde_json::to_string(&player)
    .map_err(|_| StorageError::Validation("Failed to serialize player data.".to_string()))?;

tx.execute(
    "INSERT INTO player_seasons (...) VALUES (?1, ?2, ?3, ... ?13)",
    rusqlite::params![player_id, season.id, club, age, ..., data_json],
)?;
```

The full `ParsedPlayer` is serialized as a JSON string. Serialization failure returns a `Validation` error — this should never happen with well-formed `ParsedPlayer` data (all fields are `Option` or serializable primitives).

**Transaction guarantee:** The entire import — season creation, player matching/insertion, and all `player_seasons` inserts — happens within a single `unchecked_transaction`. If any step fails (duplicate season, database error), the transaction is never committed. Rust's drop semantics ensure the transaction is rolled back when the `Transaction` value is dropped without calling `commit()`.

### 7. Retrieval (`retrieval.rs`)

**Responsibility:** Read data from SQLite with JSON deserialization.

**Row mapping with named column references:**
```rust
fn row_to_player_season(row: &rusqlite::Row) -> rusqlite::Result<PlayerSeasonData> {
    let data_json: String = row.get("data")?;
    let data = serde_json::from_str::<ParsedPlayer>(&data_json).ok();

    Ok(PlayerSeasonData {
        id: row.get("id")?,
        player_id: row.get("player_id")?,
        // ... named references for each field
        data,
    })
}
```

Named column references (`row.get("club")`) are robust against SELECT column reordering. The SELECT statements use table aliases and column aliases (e.g., `p.name AS player_name`) to disambiguate joins.

**Graceful JSON degradation:**
```rust
let data = serde_json::from_str::<ParsedPlayer>(&data_json).ok();
```

`.ok()` converts the `Result` to `Option` — deserialization failure becomes `None` rather than an error. The queryable columns (`club`, `age`, `position`, etc.) are always available regardless of JSON health.

**Empty-result semantics:**
- `get_players_for_season(nonexistent_season_id)` → empty `Vec` (not an error)
- `get_player_career(nonexistent_player)` → empty `Vec`
- `get_latest_season(save_with_zero_seasons)` → `Ok(None)`

These are not error conditions — the data simply doesn't exist.

## Tauri Integration

**Command wrapper pattern:**
```rust
#[tauri::command]
pub fn create_save(state: State<DbState>, name: String) -> Result<Save, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::create_save(&conn, &name).map_err(|e| e.into())
}
```

All 12 commands follow this identical pattern:
1. Lock the `Mutex<Connection>` via `State<DbState>`
2. Delegate to the storage function
3. Convert `StorageError` to `String` for the Tauri IPC boundary

The `Mutex` lock is held for the duration of the storage operation. This is acceptable for a single-user desktop app — there is no contention.

**App lifecycle (`lib.rs`):**
```rust
.setup(|app| {
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Unable to access app data directory: {}", e))?;
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Unable to create app data directory: {}", e))?;
    let db_path = app_data_dir.join("fm_valuescout.db");
    let conn = storage::init_db(&db_path.to_string_lossy())
        .map_err(|e| format!("Unable to initialize database: {}", e))?;
    app.manage(DbState { conn: Mutex::new(conn) });
    Ok(())
})
```

Database initialization happens during Tauri's `setup` callback, before any command can execute. Failure is fatal — the app cannot function without a database.

**Command registration:**
```rust
.invoke_handler(tauri::generate_handler![
    commands::csv_parser::parse_csv,
    commands::storage::create_save,
    commands::storage::list_saves,
    commands::storage::rename_save,
    commands::storage::delete_save,
    commands::storage::list_seasons,
    commands::storage::rename_season,
    commands::storage::delete_season,
    commands::storage::import_season,
    commands::storage::get_players_for_season,
    commands::storage::get_player_career,
    commands::storage::get_latest_season,
])
```

12 storage commands plus the existing `parse_csv`. The original `save_import` stub command was removed.

## Error Handling Philosophy

### Error Categories

| Variant | When | Example |
|---------|------|---------|
| `NotFound` | Target record doesn't exist | "Save not found.", "Season not found." |
| `Duplicate` | Uniqueness violation | "A save with the name 'X' already exists." |
| `Validation` | Input constraint violation | "Save name cannot be empty.", "Invalid date format." |
| `Database` | SQLite/rusqlite error | Any `rusqlite::Error` forwarded via `From` impl |

### Graceful Degradation for JSON Blobs

The `data` field in `player_seasons` stores a complete `ParsedPlayer` as JSON. If deserialization fails (schema drift, corruption, future app version), the retrieval layer does not fail the query. Instead:

1. `serde_json::from_str` returns `Err`
2. `.ok()` converts to `None`
3. The `PlayerSeasonData.data` field is `None`
4. All queryable columns (`club`, `age`, `position`, `minutes`, etc.) remain available

This is a pragmatic hedge: the app degrades gracefully rather than failing completely when encountering data it can't fully parse.

### Error Messages are User-Facing

All error messages are human-readable strings designed to be shown in the UI. Examples:
- "A save with the name 'My Save' already exists."
- "Season for 2030-06-30 already exists (256 players). Delete it first to re-import."
- "Invalid date format. Expected YYYY-MM-DD."

The `From<StorageError> for String` conversion at the Tauri boundary ensures these messages reach the frontend.

## Schema Design

### Hybrid Approach: Queryable Columns + JSON Blob

The `player_seasons` table stores ~12 frequently-filtered columns as real SQL columns:

| Column | Type | Purpose |
|--------|------|---------|
| `club` | TEXT | Filter by team |
| `age` | INTEGER | Filter by age range |
| `nationality` | TEXT | Filter by country |
| `position` | TEXT | Filter by role (e.g., "ST (C)") |
| `minutes` | INTEGER | Sort by playing time |
| `appearances_started` | INTEGER | Playing time detail |
| `appearances_sub` | INTEGER | Playing time detail |
| `wage_per_week` | REAL | Financial filter/sort |
| `transfer_value_high` | REAL | Financial filter/sort |
| `contract_expires` | TEXT | Expiring contracts filter |
| `data` | TEXT | Full ParsedPlayer JSON blob |

The remaining 80+ stat fields live inside the JSON blob. This avoids:
- Schema migrations when FM changes stat columns
- 80+ nullable columns that are queried rarely
- Complex JOINs for stat data that the frontend deserializes anyway

**Tradeoff:** You can't write SQL queries like `WHERE goals > 10` directly. For this app, that's acceptable — stat filtering happens client-side after loading a season's players.

### Indexes

```sql
CREATE INDEX idx_seasons_save_id ON seasons(save_id);
CREATE INDEX idx_players_save_uid ON players(save_id, fm_uid);
CREATE INDEX idx_player_seasons_player ON player_seasons(player_id);
CREATE INDEX idx_player_seasons_season ON player_seasons(season_id);
```

These cover the primary query patterns:
- Listing seasons for a save
- Looking up players by `(save_id, fm_uid)` during import
- Retrieving player_seasons by player (career timeline)
- Retrieving player_seasons by season (season view)

### Foreign Keys with CASCADE

All FK relationships use `ON DELETE CASCADE`:
- `seasons.save_id → saves.id` — deleting a save removes its seasons
- `players.save_id → saves.id` — deleting a save removes its players
- `player_seasons.player_id → players.id` — deleting a player removes their seasonal records
- `player_seasons.season_id → seasons.id` — deleting a season removes its player_seasons

The cascading chain `saves → seasons → player_seasons` and `saves → players → player_seasons` means deleting a save removes all associated data in one statement.

### Unique Constraints

- `saves.name` — UNIQUE (enforced at DB level, checked at app level for friendlier errors)
- `seasons.(save_id, in_game_date)` — one season per date per save
- `players.(save_id, fm_uid, name)` — stable player identity within a save
- `player_seasons.(player_id, season_id)` — one record per player per season

## Test Coverage

### 1. Unit Tests (in `storage/mod.rs`)

**Purpose:** Test individual functions in isolation against in-memory SQLite.

**Categories:**

**StorageError (5 tests):**
- Display for each variant (`NotFound`, `Duplicate`, `Validation`, `Database`)
- `From<StorageError> for String` conversion
- `From<rusqlite::Error>` conversion

**ImportResult serialization (1 test):**
- JSON round-trip preserves all fields

**Season label derivation (9 tests):**
- July starts new season (2030-07-01 → "2030/31")
- December in same season (2030-11-15 → "2030/31")
- January in previous season (2030-01-15 → "2029/30")
- June end of season (2030-06-30 → "2029/30")
- Invalid date format rejected
- Invalid format rejected (wrong order)
- Feb 29 leap year accepted (2028-02-29 → "2027/28")
- Feb 29 non-leap rejected
- Century boundary (2099-12-01 → "2099/00")

**Schema initialization (4 tests):**
- Creates all 4 tables
- Creates 4 indexes
- Idempotent (double init succeeds)
- `init_db` creates file and schema on disk

**Save CRUD (10 tests):**
- Basic create, list, rename, delete
- Empty/whitespace name rejected
- Name too long (101 chars) rejected, 100 chars accepted
- Case-insensitive duplicate detection
- Special characters in names (Unicode, quotes, newlines)
- Delete not-found error

**Season CRUD (8 tests):**
- Basic create with label derivation
- Invalid date, duplicate date, save-not-found rejected
- List ordered by `in_game_date` ascending
- Rename (including empty label rejection)
- Delete with cascade to player_seasons
- Delete cleans up orphaned players but preserves shared players

**Import (9 tests):**
- Basic import with counts
- Empty players rejected
- Save not found, invalid date rejected
- Duplicate season rejected
- Player matching across seasons (same UID + name → reuse)
- UID reuse with different name → separate record
- JSON blob stores full data (round-trip verification)
- Queryable columns extracted correctly
- Transaction rollback on failure (no partial data)

**Retrieval (9 tests):**
- Get players for season (ordered by name)
- Empty season, nonexistent season → empty Vec
- Player career across 3 seasons (ordered by date, age progression)
- Nonexistent player/save → empty Vec
- Latest season by date descending
- Save with zero seasons → None
- JSON deserialization failure → data: None, queryable fields intact

**Total unit tests: 70**

### 2. Integration Tests (`tests/integration_storage.rs`)

**Purpose:** Test full flows against real SQLite database (in-memory).

**7 tests:**
- `full_import_flow` — Create save → import → retrieve → verify data + JSON blob
- `career_timeline_across_seasons` — 3 seasons, age progression, date ordering
- `uid_reuse_creates_separate_records` — Same UID, different names → 2 player records
- `duplicate_season_rejected_with_count` — Error includes player count
- `delete_save_cascades_all` — Verify all 4 tables cleaned
- `delete_season_cleans_orphans_preserves_shared` — Orphan cleanup, shared player preservation
- `save_game_isolation` — Same player in two saves → independent records

### 3. Edge Case Tests (`tests/edge_case_storage.rs`)

**Purpose:** Boundary conditions, error paths, unusual scenarios.

**24 tests:**
- Import empty player list → rejected
- Save with zero seasons → None/empty
- Player career with one season → Vec len 1
- Rename save/season to same name → no-op success
- Delete/rename nonexistent save/season → NotFound
- Special characters in save name (Unicode, emojis, newlines, backslashes)
- JSON deserialization failure → graceful degradation
- Season label boundary dates (July 1, June 30, Dec 31, Jan 1)
- Feb 29 leap/non-leap validation
- Save name length boundaries (100 accepted, 101 rejected)
- Whitespace-only name → rejected
- Name trimming
- Same date in different saves → allowed
- Minimal player (all optionals None) → imports successfully
- Large import (500 players) → all imported and retrievable

**Total storage tests: 101** (70 unit + 7 integration + 24 edge case)

Full test suite: 219 tests across all features (storage + CSV parser), all passing.

## Key Design Decisions

### 1. Hybrid Schema: Queryable Columns + JSON Blob

**Decision:** Store ~12 frequently-filtered columns as real SQL columns; serialize the full `ParsedPlayer` as a JSON text blob.

**Rationale:**
- Avoids schema migrations when FM adds/changes stat columns
- Queryable columns cover the primary filter/sort use cases (club, position, age, minutes)
- JSON blob preserves all data without schema bloat
- Frontend deserializes the blob once per player — no SQL needed for stat access

**Tradeoff:** Can't write SQL queries against stat columns. Acceptable — stat filtering is client-side for this app's scale.

### 2. Player Identity: `fm_uid + name` (Case-Insensitive)

**Decision:** Match players across seasons by `(save_id, fm_uid, LOWER(name))`.

**Rationale:**
- FM UIDs are reused for newgens after retirement — `fm_uid` alone is unreliable
- Player names are stable within a save in practice
- Case-insensitive matching handles minor formatting differences
- Conservative: false splits (same player, two records) are recoverable; false merges (different players, one record) are catastrophic

**Tradeoff:** A player renamed via editor would create a split timeline. Acceptable edge case.

### 3. Single-Step Import (No Preview at Storage Level)

**Decision:** `import_season` creates the season and all player records in one transaction. No separate "preview" step in the storage layer.

**Rationale:**
- Preview already happens at the parse level (`parse_csv` returns before storage)
- The frontend shows parse results, user confirms, then calls `import_season`
- Adding a preview step in storage would complicate the API without adding value

### 4. Import Idempotency: Reject, Not Upsert

**Decision:** Importing a season with an existing `in_game_date` is rejected with an error.

**Rationale:**
- Explicit trumps implicit — the user knows exactly what happened
- Upsert would silently overwrite data, which is dangerous
- Delete-and-reimport is a deliberate action
- The error includes the player count so the user can make an informed decision

### 5. Mutex<Connection> (Single Connection)

**Decision:** Wrap a single `rusqlite::Connection` in a `Mutex` as Tauri managed state.

**Rationale:**
- Single-user desktop app — no concurrent access
- One connection avoids connection pool overhead
- Mutex prevents concurrent access within the app (e.g., overlapping imports)
- SQLite's own locking handles file-level concerns

**Tradeoff:** Long-running operations (large imports) block all other DB access. Acceptable — imports are fast (<1s for typical data).

### 6. Date-Derived Season Labels with Manual Override

**Decision:** Auto-derive season labels from the in-game date using July→June heuristic; allow manual rename.

**Rationale:**
- Matches FM players' mental model ("the 2030/31 season")
- Non-European leagues (Norway, Sweden) may get technically incorrect labels
- Rename provides an escape hatch for any league

### 7. Orphan Player Cleanup on Season Delete

**Decision:** When a season is deleted, player records with no remaining `player_seasons` in the save are removed.

**Rationale:**
- Prevents unbounded player table growth
- A player with no data is useless — all information comes from `player_seasons`
- Players shared with other seasons are preserved (checked via subquery)

## Evolution

### First-Pass Implementation

The storage layer started as a single `mod.rs` file (~1700 lines) containing all implementation and tests. This was a reasonable starting point for a greenfield feature — all code was in one place for easy reference during development.

### Refactor (Post-Retrospective)

A critical retrospective identified structural issues in the monolith. The module was split into a directory structure:

```
src-tauri/src/storage/
├── mod.rs          (re-exports only)
├── schema.rs       (DDL, init)
├── error.rs        (StorageError)
├── types.rs        (DTOs, DbState)
├── saves.rs        (Save CRUD)
├── seasons.rs      (Season CRUD + label derivation)
├── import.rs       (Transactional import)
└── retrieval.rs    (Read queries + JSON deserialization)
```

**What changed and why:**

| Finding | Before | After | Rationale |
|---------|--------|-------|-----------|
| Monolith | 1700 lines in `mod.rs` | 8 files, largest ~175 lines | Navigation, review friction |
| Dead code | `save_import` stub present | Deleted | Replaced by `import_season` |
| Silent error swallowing | `filter_map(\|r\| r.ok())` | `collect::<Result<Vec<_>, _>>()?` | Prevents silent data loss |
| Hardcoded column indices | `row.get(14)` positional | `row.get("data")` named | Robust against column reordering |
| Duplicated logic | `create_season` copied `create_season_tx` | `create_season` delegates to `create_season_tx` | Single source of truth |
| Manual cascade | `delete_save` had 4 DELETE statements | Trusts `ON DELETE CASCADE` | Simpler, schema-verified |
| Missing transaction | `delete_save`/`delete_season` unwrapped | Wrapped in `unchecked_transaction()` | Atomicity guarantee |
| Empty `created_at` | `create_save` returned `""` | Re-reads from DB after insert | Consistent API |

All 219 existing tests passed without modification after the refactor.

## Future Enhancements

**Planned (from design spec):**
- **Player search/query:** Full-text search on player name, club, nationality. Currently the frontend loads all players for a season and filters client-side — sufficient for current scale.
- **Season statistics summary:** Aggregated stats per season (average rating, top scorer). Can be computed client-side from loaded data.
- **Data export:** Export stored data back to CSV or JSON. The user always has the original CSV.
- **Save-game metadata:** League, difficulty, etc. The `managed_club` column is prepared; other metadata can be added later.
- **Orphan cleanup utility:** Explicit command to clean up orphaned player records. Currently handled implicitly during season deletion.

**Considered but deferred:**
- **Schema migration framework:** The database is created fresh in the initial release. Versioned migrations will be added when the schema needs to change.
- **Fuzzy name matching for player identity:** Close-but-not-exact name matches create separate records. This is intentional — a split timeline is recoverable; a merged timeline is not.
- **Import progress indicator:** Imports are instant for realistic file sizes (<1 second for 500 rows). Progress indication is unnecessary until file sizes are much larger.
- **WAL mode for concurrent reads:** Single-user app with single `Mutex<Connection>`. No concurrent read/write scenarios to optimize.
