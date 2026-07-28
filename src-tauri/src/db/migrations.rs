use rusqlite::Connection;

pub struct Migration {
    pub version: i32,
    pub description: &'static str,
    pub sql: &'static str,
}

pub const INITIAL_DEMO_VALUE_SQL: &str = "
CREATE TABLE IF NOT EXISTS demo_value (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    value TEXT NOT NULL DEFAULT ''
);
";

pub fn all() -> &'static [Migration] {
    &[Migration {
        version: 1,
        description: "create_demo_value_table",
        sql: INITIAL_DEMO_VALUE_SQL,
    }]
}

/// Apply pending migrations using `PRAGMA user_version`.
pub fn apply(conn: &Connection) -> Result<(), rusqlite::Error> {
    let current: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    for migration in all() {
        if migration.version <= current {
            continue;
        }

        log::info!(
            "applying migration {}: {}",
            migration.version,
            migration.description
        );

        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(migration.sql)?;
        tx.pragma_update(None, "user_version", migration.version)?;
        tx.commit()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn open_migrated(db_path: &Path) -> Connection {
        let conn = Connection::open(db_path).expect("open test db");
        apply(&conn).expect("apply migrations");
        conn
    }

    #[test]
    fn opening_fresh_db_applies_version_1_and_creates_demo_value() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("migration-test.db");
        let conn = open_migrated(&db_path);

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user_version");
        assert_eq!(version, 1);

        let table_name: String = conn
            .query_row(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'demo_value'",
                [],
                |row| row.get(0),
            )
            .expect("read sqlite_master");
        assert_eq!(table_name, "demo_value");
    }

    #[test]
    fn apply_is_idempotent_on_already_migrated_db() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("migration-idempotent.db");
        let conn = open_migrated(&db_path);
        apply(&conn).expect("re-apply migrations");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user_version");
        assert_eq!(version, 1);
    }

    #[test]
    fn registers_monotonic_demo_value_migration() {
        let migrations = all();

        assert_eq!(migrations.len(), 1);
        assert_eq!(migrations[0].version, 1);
        assert_eq!(migrations[0].description, "create_demo_value_table");
        assert_eq!(migrations[0].sql, INITIAL_DEMO_VALUE_SQL);
    }
}
