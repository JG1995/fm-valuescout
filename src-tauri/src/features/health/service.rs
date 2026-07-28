use rusqlite::{params, Connection, OptionalExtension};

const DEMO_VALUE_ID: i64 = 1;
const MAX_DEMO_VALUE_LEN: usize = 200;

pub fn validate_demo_value(value: &str) -> Result<(), String> {
    if value.chars().count() > MAX_DEMO_VALUE_LEN {
        return Err(format!(
            "Demo value must be at most {} characters",
            MAX_DEMO_VALUE_LEN
        ));
    }

    Ok(())
}

pub fn get_demo_value(conn: &Connection) -> Result<String, String> {
    let value = conn
        .query_row(
            "SELECT value FROM demo_value WHERE id = ?1",
            params![DEMO_VALUE_ID],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    Ok(value.unwrap_or_default())
}

pub fn set_demo_value(conn: &Connection, value: &str) -> Result<String, String> {
    validate_demo_value(value)?;

    conn.execute(
        "INSERT INTO demo_value (id, value) VALUES (?1, ?2)
         ON CONFLICT(id) DO UPDATE SET value = excluded.value",
        params![DEMO_VALUE_ID, value],
    )
    .map_err(|error| error.to_string())?;

    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use std::path::Path;

    fn open_migrated(db_path: &Path) -> Connection {
        let conn = Connection::open(db_path).expect("open test db");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    #[test]
    fn returns_empty_string_when_demo_value_row_is_missing() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("test.db");
        let conn = open_migrated(&db_path);

        let value = get_demo_value(&conn).expect("read demo value");

        assert_eq!(value, "");
    }

    #[test]
    fn persists_demo_value_across_reads() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("test.db");
        let conn = open_migrated(&db_path);

        set_demo_value(&conn, "hello").expect("write demo value");

        let value = get_demo_value(&conn).expect("read demo value");

        assert_eq!(value, "hello");
    }

    #[test]
    fn persists_demo_value_across_reopen_on_same_file() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("reopen-test.db");

        {
            let conn = open_migrated(&db_path);
            set_demo_value(&conn, "across-reopen").expect("write demo value");
        }

        let reopened = open_migrated(&db_path);
        let value = get_demo_value(&reopened).expect("read after reopen");

        assert_eq!(value, "across-reopen");
    }

    #[test]
    fn rejects_demo_values_longer_than_limit() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("test.db");
        let conn = open_migrated(&db_path);
        let long_value = "x".repeat(MAX_DEMO_VALUE_LEN + 1);

        let error = set_demo_value(&conn, &long_value).expect_err("reject long demo value");

        assert!(error.contains("at most"));
    }
}
