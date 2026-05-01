# Task 03 - Archetype CRUD Storage Layer

## Overview

Implement the Rust storage functions for archetype CRUD: create, read (list by role, get by id), update, and delete. These are pure database operations that will be wrapped by Tauri commands in Task 05.

## Files to Create/Modify

- Modify: `src-tauri/src/storage/archetypes.rs` — Add CRUD functions below the types
- Modify: `src-tauri/src/storage/mod.rs` — Add re-exports for CRUD functions

## Context

### Existing CRUD Pattern

Storage functions follow a consistent pattern (see `src-tauri/src/storage/saves.rs`):

1. Function takes `&Connection` as first arg
2. Validates input using a `validate_*` helper
3. Executes SQL via `conn.execute()` or `conn.query_row()`
4. Returns `Result<T, StorageError>`
5. Uses `rusqlite::params![]` macro for parameterized queries
6. Checks affected rows after UPDATE/DELETE — returns `StorageError::NotFound` if 0 rows

### Database Schema (from Task 01)

```sql
CREATE TABLE IF NOT EXISTS archetypes (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    name         TEXT    NOT NULL,
    role         TEXT    NOT NULL,
    metrics_json TEXT    NOT NULL,
    is_default   INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE(name, role)
);
```

### Key Invariant

**Cannot delete the last archetype for a role.** If deleting would leave a role with zero archetypes, return `StorageError::Validation`.

## Steps

- [ ] **Step 1: Write the failing tests**

Append these tests to the `#[cfg(test)] mod tests` block in `src-tauri/src/storage/archetypes.rs`:

```rust
// Add at the top of the tests module, after `use super::*;`:
use super::super::schema::init_schema;
use rusqlite::Connection;

fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    init_schema(&conn).unwrap();
    conn
}

fn sample_metrics() -> Vec<MetricWeight> {
    vec![
        MetricWeight {
            metric_key: "goals_per_90".to_string(),
            weight: 0.6,
            inverted: false,
        },
        MetricWeight {
            metric_key: "assists_per_90".to_string(),
            weight: 0.4,
            inverted: false,
        },
    ]
}

#[test]
fn create_archetype_basic() {
    let conn = setup_test_db();
    let arch = create_archetype(&conn, "Test Arch", "GK", &sample_metrics()).unwrap();
    assert_eq!(arch.name, "Test Arch");
    assert_eq!(arch.role, "GK");
    assert_eq!(arch.metrics.len(), 2);
    assert!(!arch.is_default);
    assert!(arch.id > 0);
}

#[test]
fn create_archetype_normalizes_weights() {
    let conn = setup_test_db();
    let mut metrics = sample_metrics();
    metrics[0].weight = 0.3;
    metrics[1].weight = 0.2;
    let arch = create_archetype(&conn, "Unnormalized", "ST", &metrics).unwrap();
    let sum: f64 = arch.metrics.iter().map(|m| m.weight).sum();
    assert!((sum - 1.0).abs() < 1e-9);
}

#[test]
fn create_archetype_empty_name_rejected() {
    let conn = setup_test_db();
    let result = create_archetype(&conn, "", "GK", &sample_metrics());
    assert!(result.is_err());
}

#[test]
fn create_archetype_empty_metrics_rejected() {
    let conn = setup_test_db();
    let result = create_archetype(&conn, "Empty", "GK", &[]);
    assert!(result.is_err());
}

#[test]
fn create_archetype_duplicate_name_role_rejected() {
    let conn = setup_test_db();
    create_archetype(&conn, "Test Arch", "GK", &sample_metrics()).unwrap();
    let result = create_archetype(&conn, "Test Arch", "GK", &sample_metrics());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[test]
fn create_archetype_same_name_different_role_ok() {
    let conn = setup_test_db();
    create_archetype(&conn, "Playmaker", "DM", &sample_metrics()).unwrap();
    let result = create_archetype(&conn, "Playmaker", "CM", &sample_metrics());
    assert!(result.is_ok());
}

#[test]
fn list_archetypes_empty() {
    let conn = setup_test_db();
    let archs = list_archetypes(&conn, "GK").unwrap();
    assert!(archs.is_empty());
}

#[test]
fn list_archetypes_filters_by_role() {
    let conn = setup_test_db();
    create_archetype(&conn, "Arch A", "GK", &sample_metrics()).unwrap();
    create_archetype(&conn, "Arch B", "ST", &sample_metrics()).unwrap();
    let gk = list_archetypes(&conn, "GK").unwrap();
    let st = list_archetypes(&conn, "ST").unwrap();
    assert_eq!(gk.len(), 1);
    assert_eq!(st.len(), 1);
    assert_eq!(gk[0].name, "Arch A");
}

#[test]
fn list_all_archetypes_returns_all() {
    let conn = setup_test_db();
    create_archetype(&conn, "Arch A", "GK", &sample_metrics()).unwrap();
    create_archetype(&conn, "Arch B", "ST", &sample_metrics()).unwrap();
    let all = list_all_archetypes(&conn).unwrap();
    assert_eq!(all.len(), 2);
}

#[test]
fn get_archetype_by_id_found() {
    let conn = setup_test_db();
    let created = create_archetype(&conn, "Found", "AM", &sample_metrics()).unwrap();
    let found = get_archetype(&conn, created.id).unwrap();
    assert_eq!(found.name, "Found");
    assert_eq!(found.role, "AM");
}

#[test]
fn get_archetype_by_id_not_found() {
    let conn = setup_test_db();
    let result = get_archetype(&conn, 9999);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn update_archetype_name_and_metrics() {
    let conn = setup_test_db();
    let created = create_archetype(&conn, "Original", "GK", &sample_metrics()).unwrap();
    let mut new_metrics = sample_metrics();
    new_metrics[0].metric_key = "xg_per_90".to_string();
    let updated = update_archetype(&conn, created.id, "Updated", &new_metrics).unwrap();
    assert_eq!(updated.name, "Updated");
    assert_eq!(updated.metrics[0].metric_key, "xg_per_90");
    // Verify weights were normalized
    let sum: f64 = updated.metrics.iter().map(|m| m.weight).sum();
    assert!((sum - 1.0).abs() < 1e-9);
}

#[test]
fn update_archetype_not_found() {
    let conn = setup_test_db();
    let result = update_archetype(&conn, 9999, "Ghost", &sample_metrics());
    assert!(result.is_err());
}

#[test]
fn update_archetype_duplicate_name_rejected() {
    let conn = setup_test_db();
    create_archetype(&conn, "Arch A", "GK", &sample_metrics()).unwrap();
    let arch_b = create_archetype(&conn, "Arch B", "GK", &sample_metrics()).unwrap();
    let result = update_archetype(&conn, arch_b.id, "Arch A", &sample_metrics());
    assert!(result.is_err());
}

#[test]
fn update_archetype_preserves_is_default() {
    let conn = setup_test_db();
    // Insert a default archetype directly
    conn.execute(
        "INSERT INTO archetypes (name, role, metrics_json, is_default) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params!["Default GK", "GK", "[]", 1],
    ).unwrap();
    let id = conn.last_insert_rowid();
    // Update via function — is_default should remain true
    let mut metrics = sample_metrics();
    metrics[0].weight = 1.0;
    let updated = update_archetype(&conn, id, "Updated Default", &vec![metrics[0].clone()]).unwrap();
    assert!(updated.is_default);
}

#[test]
fn delete_archetype_basic() {
    let conn = setup_test_db();
    let arch = create_archetype(&conn, "To Delete", "ST", &sample_metrics()).unwrap();
    // Create a second so deletion doesn't violate "last archetype" rule
    create_archetype(&conn, "Keeper", "ST", &sample_metrics()).unwrap();
    delete_archetype(&conn, arch.id).unwrap();
    let remaining = list_archetypes(&conn, "ST").unwrap();
    assert_eq!(remaining.len(), 1);
}

#[test]
fn delete_archetype_not_found() {
    let conn = setup_test_db();
    let result = delete_archetype(&conn, 9999);
    assert!(result.is_err());
}

#[test]
fn delete_last_archetype_for_role_rejected() {
    let conn = setup_test_db();
    let arch = create_archetype(&conn, "Only One", "DM", &sample_metrics()).unwrap();
    let result = delete_archetype(&conn, arch.id);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("at least one"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test --lib storage::archetypes::tests::create_archetype -- --nocapture`
Expected: FAIL — `create_archetype` function not defined yet.

- [ ] **Step 3: Implement the CRUD functions**

Add these functions to `src-tauri/src/storage/archetypes.rs`, between the types section and the `#[cfg(test)]` block:

```rust
use rusqlite::Connection;

// ── CRUD Operations ───────────────────────────────────────────────────

/// Create a new archetype. Validates name and metrics, normalizes weights.
/// Returns the created archetype with generated id and timestamps.
pub fn create_archetype(
    conn: &Connection,
    name: &str,
    role: &str,
    metrics: &[MetricWeight],
) -> Result<Archetype, StorageError> {
    let name = validate_archetype_name(name)?;
    validate_metrics(metrics)?;

    // Normalize weights
    let mut normalized = metrics.to_vec();
    normalize_weights(&mut normalized);

    let metrics_json = serde_json::to_string(&normalized)
        .map_err(|e| StorageError::Database(format!("Failed to serialize metrics: {}", e)))?;

    // Check for duplicate name+role
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM archetypes WHERE name = ?1 AND role = ?2)",
        rusqlite::params![name, role],
        |row| row.get(0),
    )?;
    if exists {
        return Err(StorageError::Duplicate(format!(
            "An archetype named '{}' already exists for role '{}'.",
            name, role
        )));
    }

    conn.execute(
        "INSERT INTO archetypes (name, role, metrics_json, is_default) VALUES (?1, ?2, ?3, 0)",
        rusqlite::params![name, role, metrics_json],
    )?;
    let id = conn.last_insert_rowid();

    let created_at: String = conn.query_row(
        "SELECT created_at FROM archetypes WHERE id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    )?;
    let updated_at: String = conn.query_row(
        "SELECT updated_at FROM archetypes WHERE id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    )?;

    Ok(Archetype {
        id,
        name,
        role: role.to_string(),
        metrics: normalized,
        is_default: false,
        created_at,
        updated_at,
    })
}

/// List all archetypes for a specific role, ordered by name.
pub fn list_archetypes(conn: &Connection, role: &str) -> Result<Vec<Archetype>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, role, metrics_json, is_default, created_at, updated_at
         FROM archetypes WHERE role = ?1 ORDER BY name ASC",
    )?;
    let archetypes: Vec<Archetype> = stmt
        .query_map(rusqlite::params![role], |row| row_to_archetype(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(archetypes)
}

/// List all archetypes across all roles, ordered by role then name.
pub fn list_all_archetypes(conn: &Connection) -> Result<Vec<Archetype>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, role, metrics_json, is_default, created_at, updated_at
         FROM archetypes ORDER BY role ASC, name ASC",
    )?;
    let archetypes: Vec<Archetype> = stmt
        .query_map([], |row| row_to_archetype(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(archetypes)
}

/// Get a single archetype by id.
pub fn get_archetype(conn: &Connection, id: i64) -> Result<Archetype, StorageError> {
    let result = conn.query_row(
        "SELECT id, name, role, metrics_json, is_default, created_at, updated_at
         FROM archetypes WHERE id = ?1",
        rusqlite::params![id],
        |row| row_to_archetype(row),
    );
    match result {
        Ok(arch) => Ok(arch),
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(StorageError::NotFound(
            "Archetype not found.".to_string(),
        )),
        Err(e) => Err(StorageError::Database(e.to_string())),
    }
}

/// Update an existing archetype's name and metrics.
/// Role cannot be changed (would break the unique constraint semantics).
/// Normalizes weights before saving. Preserves `is_default` flag.
pub fn update_archetype(
    conn: &Connection,
    id: i64,
    name: &str,
    metrics: &[MetricWeight],
) -> Result<Archetype, StorageError> {
    let name = validate_archetype_name(name)?;
    validate_metrics(metrics)?;

    let mut normalized = metrics.to_vec();
    normalize_weights(&mut normalized);

    let metrics_json = serde_json::to_string(&normalized)
        .map_err(|e| StorageError::Database(format!("Failed to serialize metrics: {}", e)))?;

    // Check for duplicate name+role (excluding self)
    let current = get_archetype(conn, id)?;
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM archetypes WHERE name = ?1 AND role = ?2 AND id != ?3)",
        rusqlite::params![name, current.role, id],
        |row| row.get(0),
    )?;
    if exists {
        return Err(StorageError::Duplicate(format!(
            "An archetype named '{}' already exists for role '{}'.",
            name, current.role
        )));
    }

    let rows = conn.execute(
        "UPDATE archetypes SET name = ?1, metrics_json = ?2, updated_at = datetime('now') WHERE id = ?3",
        rusqlite::params![name, metrics_json, id],
    )?;
    if rows == 0 {
        return Err(StorageError::NotFound("Archetype not found.".to_string()));
    }

    get_archetype(conn, id)
}

/// Delete an archetype.
/// Enforces the invariant: cannot delete the last archetype for a role.
pub fn delete_archetype(conn: &Connection, id: i64) -> Result<(), StorageError> {
    let arch = get_archetype(conn, id)?;

    // Check if this is the last archetype for this role
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM archetypes WHERE role = ?1",
        rusqlite::params![arch.role],
        |row| row.get(0),
    )?;
    if count <= 1 {
        return Err(StorageError::Validation(format!(
            "Cannot delete the last archetype for role '{}'. At least one archetype must remain.",
            arch.role
        )));
    }

    let rows = conn.execute(
        "DELETE FROM archetypes WHERE id = ?1",
        rusqlite::params![id],
    )?;
    if rows == 0 {
        return Err(StorageError::NotFound("Archetype not found.".to_string()));
    }
    Ok(())
}

// ── Row mapping ────────────────────────────────────────────────────────

fn row_to_archetype(row: &rusqlite::Row) -> rusqlite::Result<Archetype> {
    let metrics_json: String = row.get("metrics_json")?;
    let metrics = serde_json::from_str::<Vec<MetricWeight>>(&metrics_json)
        .unwrap_or_default();

    Ok(Archetype {
        id: row.get("id")?,
        name: row.get("name")?,
        role: row.get("role")?,
        metrics,
        is_default: row.get::<_, i64>("is_default")? != 0,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}
```

- [ ] **Step 4: Update mod.rs re-exports**

Add to `src-tauri/src/storage/mod.rs` re-exports:

```rust
pub use archetypes::{
    Archetype, MetricWeight,
    validate_metrics, normalize_weights, validate_archetype_name,
    create_archetype, list_archetypes, list_all_archetypes,
    get_archetype, update_archetype, delete_archetype,
};
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --lib storage::archetypes`
Expected: ALL PASS.

- [ ] **Step 6: Run full test suite**

Run: `cd src-tauri && cargo test --lib`
Expected: ALL PASS.

## Dependencies

- Task 01 (schema migration) — archetypes table must exist
- Task 02 (archetype types) — `Archetype`, `MetricWeight`, validation functions must be defined

## Success Criteria

- `create_archetype` validates, normalizes, persists, returns the archetype
- `list_archetypes` filters by role
- `list_all_archetypes` returns everything
- `get_archetype` returns by id or NotFound
- `update_archetype` validates, normalizes, preserves `is_default`
- `delete_archetype` enforces "at least one per role" invariant
- All existing tests still pass

## Tests

### Test 1: Create archetype

**What to test:** Basic creation, weight normalization, empty name rejection, duplicate rejection.
**Feasibility:** ✅ Can be tested — in-memory SQLite.

### Test 2: List archetypes

**What to test:** Filter by role, list all, empty list.
**Feasibility:** ✅ Can be tested — in-memory SQLite.

### Test 3: Update archetype

**What to test:** Name/metrics update, weight normalization, duplicate name rejection, preserves `is_default`.
**Feasibility:** ✅ Can be tested — in-memory SQLite.

### Test 4: Delete archetype

**What to test:** Basic deletion, not found, last-archetype-for-role rejection.
**Feasibility:** ✅ Can be tested — in-memory SQLite.
