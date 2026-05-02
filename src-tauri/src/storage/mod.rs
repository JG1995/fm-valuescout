mod error;
mod types;
mod schema;
mod saves;
mod seasons;
mod import;
mod retrieval;
mod archetypes;

// Re-exports — public API unchanged for command wrappers
pub use error::StorageError;
pub use types::{DbState, Save, Season, ImportResult, PlayerSeasonData};
pub use schema::{init_db, init_db_test};
pub use saves::{create_save, list_saves, rename_save, delete_save};
pub use seasons::{
    derive_season_label,
    create_season, list_seasons, rename_season, delete_season,
};
pub use import::import_season;
pub use retrieval::{get_players_for_season, get_player_career, get_latest_season};
pub use archetypes::{
    Archetype, MetricWeight,
    validate_metrics, normalize_weights, validate_archetype_name, validate_role,
    create_archetype, list_archetypes, list_all_archetypes,
    get_archetype, update_archetype, delete_archetype,
};

#[cfg(test)]
mod tests {
    use super::*;
    use super::schema::init_schema;
    use crate::parser::types::ParsedPlayer;

    #[test]
    fn storage_error_to_string_not_found() {
        let err = StorageError::NotFound("Save not found.".to_string());
        assert_eq!(err.to_string(), "Save not found.");
    }

    #[test]
    fn storage_error_to_string_duplicate() {
        let err = StorageError::Duplicate("Already exists.".to_string());
        assert_eq!(err.to_string(), "Already exists.");
    }

    #[test]
    fn storage_error_to_string_validation() {
        let err = StorageError::Validation("Name cannot be empty.".to_string());
        assert_eq!(err.to_string(), "Name cannot be empty.");
    }

    #[test]
    fn storage_error_into_string() {
        let err = StorageError::Database("disk full".to_string());
        let s: String = err.into();
        assert_eq!(s, "disk full");
    }

    #[test]
    fn rusqlite_error_converts_to_database() {
        let rusqlite_err = rusqlite::Error::InvalidColumnIndex(999);
        let storage_err: StorageError = rusqlite_err.into();
        match storage_err {
            StorageError::Database(msg) => assert!(msg.contains("999")),
            _ => panic!("Expected Database variant"),
        }
    }


    #[test]
    fn import_result_serializable() {
        let result = ImportResult {
            season: Season {
                id: 1,
                save_id: 1,
                in_game_date: "2030-11-15".to_string(),
                label: "2030/31".to_string(),
                imported_at: "2026-04-30 12:00:00".to_string(),
            },
            total_players: 25,
            new_players: 20,
            matched_players: 5,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: ImportResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.total_players, 25);
        assert_eq!(back.season.label, "2030/31");
    }

    // ── derive_season_label tests ────────────────────────────────────────

    #[test]
    fn season_label_july_starts_new_season() {
        assert_eq!(derive_season_label("2030-07-01").unwrap(), "2030/31");
    }

    #[test]
    fn season_label_december_in_same_season() {
        assert_eq!(derive_season_label("2030-11-15").unwrap(), "2030/31");
    }

    #[test]
    fn season_label_january_in_previous_season() {
        assert_eq!(derive_season_label("2030-01-15").unwrap(), "2029/30");
    }

    #[test]
    fn season_label_june_end_of_season() {
        assert_eq!(derive_season_label("2030-06-30").unwrap(), "2029/30");
    }

    #[test]
    fn season_label_invalid_date_rejected() {
        let result = derive_season_label("not-a-date");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid date format"));
    }

    #[test]
    fn season_label_invalid_format_rejected() {
        let result = derive_season_label("30-06-2026");
        assert!(result.is_err());
    }

    #[test]
    fn season_label_feb_29_leap_year() {
        assert_eq!(derive_season_label("2028-02-29").unwrap(), "2027/28");
    }

    #[test]
    fn season_label_feb_29_non_leap_rejected() {
        let result = derive_season_label("2027-02-29");
        assert!(result.is_err());
    }

    #[test]
    fn season_label_century_boundary() {
        // 2099-12-01 → "2099/00" (year+1 mod 100 = 0)
        assert_eq!(derive_season_label("2099-12-01").unwrap(), "2099/00");
    }

    // ── schema initialization tests ───────────────────────────────────

    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn schema_creates_all_tables() {
        let conn = setup_test_db();
        let tables: Vec<String> = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
        ).unwrap()
            .query_map([], |row| row.get(0)).unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"saves".to_string()));
        assert!(tables.contains(&"seasons".to_string()));
        assert!(tables.contains(&"players".to_string()));
        assert!(tables.contains(&"player_seasons".to_string()));
    }

    #[test]
    fn schema_creates_indexes() {
        let conn = setup_test_db();
        let indexes: Vec<String> = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%' ORDER BY name"
        ).unwrap()
            .query_map([], |row| row.get(0)).unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(indexes.contains(&"idx_seasons_save_id".to_string()));
        assert!(indexes.contains(&"idx_players_save_uid".to_string()));
        assert!(indexes.contains(&"idx_player_seasons_player".to_string()));
        assert!(indexes.contains(&"idx_player_seasons_season".to_string()));
    }

    #[test]
    fn schema_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        init_schema(&conn).unwrap(); // Second call should not fail
    }

    // ── archetypes schema tests ───────────────────────────────────────

    #[test]
    fn schema_creates_archetypes_table() {
        let conn = setup_test_db();
        let tables: Vec<String> = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
        ).unwrap()
            .query_map([], |row| row.get(0)).unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"archetypes".to_string()));
    }

    #[test]
    fn schema_creates_archetypes_indexes() {
        let conn = setup_test_db();
        let indexes: Vec<String> = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%' ORDER BY name"
        ).unwrap()
            .query_map([], |row| row.get(0)).unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(indexes.contains(&"idx_archetypes_role".to_string()));
    }

    #[test]
    fn archetypes_unique_name_role_constraint() {
        let conn = setup_test_db();
        conn.execute(
            "INSERT INTO archetypes (name, role, metrics_json, is_default) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["Test Arch", "GK", "[]", true],
        ).unwrap();
        let result = conn.execute(
            "INSERT INTO archetypes (name, role, metrics_json, is_default) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params!["Test Arch", "GK", "[]", false],
        );
        assert!(result.is_err());
    }

    #[test]
    fn init_db_creates_file_and_schema() {
        let dir = std::env::temp_dir().join("fm_valuescout_test_init_db");
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("test_init.db");
        let path_str = db_path.to_string_lossy().to_string();

        let conn = init_db(&path_str).unwrap();

        // Verify schema - check for our 4 expected tables (sqlite_sequence is auto-created for AUTOINCREMENT)
        let tables: Vec<String> = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
        ).unwrap()
            .query_map([], |row| row.get(0)).unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"saves".to_string()));
        assert!(tables.contains(&"seasons".to_string()));
        assert!(tables.contains(&"players".to_string()));
        assert!(tables.contains(&"player_seasons".to_string()));


        // Clean up
        drop(conn);
        std::fs::remove_file(&db_path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    // ── Save CRUD tests ──────────────────────────────────────────────────

    #[test]
    fn create_save_basic() {
        let conn = setup_test_db();
        let save = create_save(&conn, "My Save").unwrap();
        assert_eq!(save.name, "My Save");
        assert!(save.id > 0);
        assert!(save.managed_club.is_none());
        assert_eq!(save.season_count, 0);
        assert_eq!(save.player_count, 0);
    }

    #[test]
    fn create_save_empty_name_rejected() {
        let conn = setup_test_db();
        let result = create_save(&conn, "");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn create_save_whitespace_name_rejected() {
        let conn = setup_test_db();
        let result = create_save(&conn, "   ");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn create_save_name_too_long_rejected() {
        let conn = setup_test_db();
        let long_name = "x".repeat(101);
        let result = create_save(&conn, &long_name);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("100 characters"));
    }

    #[test]
    fn create_save_name_100_chars_ok() {
        let conn = setup_test_db();
        let name = "x".repeat(100);
        let save = create_save(&conn, &name).unwrap();
        assert_eq!(save.name.len(), 100);
    }

    #[test]
    fn create_save_duplicate_name_rejected() {
        let conn = setup_test_db();
        create_save(&conn, "My Save").unwrap();
        let result = create_save(&conn, "My Save");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn create_save_case_insensitive_duplicate() {
        let conn = setup_test_db();
        create_save(&conn, "My Save").unwrap();
        let result = create_save(&conn, "my save");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn create_save_special_chars_allowed() {
        let conn = setup_test_db();
        let save = create_save(&conn, "Årnes & Ølstad — \"2025/26\"").unwrap();
        assert_eq!(save.name, "Årnes & Ølstad — \"2025/26\"");
    }

    #[test]
    fn list_saves_empty() {
        let conn = setup_test_db();
        let saves = list_saves(&conn).unwrap();
        assert!(saves.is_empty());
    }

    #[test]
    fn list_saves_returns_created_save() {
        let conn = setup_test_db();
        create_save(&conn, "Save A").unwrap();
        create_save(&conn, "Save B").unwrap();
        let saves = list_saves(&conn).unwrap();
        assert_eq!(saves.len(), 2);
        let names: Vec<&str> = saves.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Save A"));
        assert!(names.contains(&"Save B"));
    }

    #[test]
    fn rename_save_basic() {
        let conn = setup_test_db();
        let save = create_save(&conn, "Old Name").unwrap();
        rename_save(&conn, save.id, "New Name").unwrap();
        let saves = list_saves(&conn).unwrap();
        assert_eq!(saves[0].name, "New Name");
    }

    #[test]
    fn rename_save_to_same_name_noop() {
        let conn = setup_test_db();
        let save = create_save(&conn, "Same").unwrap();
        let result = rename_save(&conn, save.id, "Same");
        assert!(result.is_ok());
    }

    #[test]
    fn rename_save_empty_name_rejected() {
        let conn = setup_test_db();
        let save = create_save(&conn, "Valid").unwrap();
        let result = rename_save(&conn, save.id, "");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn rename_save_name_too_long_rejected() {
        let conn = setup_test_db();
        let save = create_save(&conn, "Valid").unwrap();
        let result = rename_save(&conn, save.id, &"x".repeat(101));
        assert!(result.is_err());
    }

    #[test]
    fn rename_save_duplicate_name_rejected() {
        let conn = setup_test_db();
        create_save(&conn, "Save A").unwrap();
        let save_b = create_save(&conn, "Save B").unwrap();
        let result = rename_save(&conn, save_b.id, "Save A");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn rename_save_not_found() {
        let conn = setup_test_db();
        let result = rename_save(&conn, 9999, "New");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn delete_save_basic() {
        let conn = setup_test_db();
        let save = create_save(&conn, "To Delete").unwrap();
        delete_save(&conn, save.id).unwrap();
        let saves = list_saves(&conn).unwrap();
        assert!(saves.is_empty());
    }

    #[test]
    fn delete_save_not_found() {
        let conn = setup_test_db();
        let result = delete_save(&conn, 9999);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn delete_save_cascades_seasons() {
        let conn = setup_test_db();
        let save = create_save(&conn, "With Seasons").unwrap();
        // Insert a season manually to test cascade
        conn.execute(
            "INSERT INTO seasons (save_id, in_game_date, label) VALUES (?1, ?2, ?3)",
            rusqlite::params![save.id, "2030-11-15", "2030/31"],
        ).unwrap();
        delete_save(&conn, save.id).unwrap();
        // Verify seasons are gone
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM seasons", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);
    }

    // ── Season CRUD tests ────────────────────────────────────────────────
    fn create_test_save(conn: &Connection) -> Save {
        create_save(conn, "Test Save").unwrap()
    }

    #[test]
    fn create_season_basic() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let season = create_season(&conn, save.id, "2030-11-15").unwrap();
        assert_eq!(season.save_id, save.id);
        assert_eq!(season.in_game_date, "2030-11-15");
        assert_eq!(season.label, "2030/31");
    }


    #[test]
    fn create_season_invalid_date_rejected() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let result = create_season(&conn, save.id, "not-a-date");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid date format"));
    }

    #[test]
    fn create_season_duplicate_date_rejected() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        create_season(&conn, save.id, "2030-11-15").unwrap();
        let result = create_season(&conn, save.id, "2030-11-15");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }


    #[test]
    fn create_season_save_not_found() {
        let conn = setup_test_db();
        let result = create_season(&conn, 9999, "2030-11-15");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn list_seasons_empty() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let seasons = list_seasons(&conn, save.id).unwrap();
        assert!(seasons.is_empty());
    }

    #[test]
    fn list_seasons_ordered_by_date() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        create_season(&conn, save.id, "2031-06-15").unwrap(); // "2030/31"
        create_season(&conn, save.id, "2030-11-15").unwrap(); // "2030/31"
        create_season(&conn, save.id, "2031-11-15").unwrap(); // "2031/32"
        let seasons = list_seasons(&conn, save.id).unwrap();
        assert_eq!(seasons.len(), 3);
        // Ordered by in_game_date ascending
        assert_eq!(seasons[0].in_game_date, "2030-11-15");
        assert_eq!(seasons[1].in_game_date, "2031-06-15");
        assert_eq!(seasons[2].in_game_date, "2031-11-15");
    }

    #[test]
    fn rename_season_basic() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let season = create_season(&conn, save.id, "2030-11-15").unwrap();
        rename_season(&conn, season.id, "Våren 2026").unwrap();
        let seasons = list_seasons(&conn, save.id).unwrap();
        assert_eq!(seasons[0].label, "Våren 2026");
    }

    #[test]
    fn rename_season_empty_rejected() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let season = create_season(&conn, save.id, "2030-11-15").unwrap();
        let result = rename_season(&conn, season.id, "");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn rename_season_not_found() {
        let conn = setup_test_db();
        let result = rename_season(&conn, 9999, "New Label");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }


    #[test]
    fn delete_season_basic() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let season = create_season(&conn, save.id, "2030-11-15").unwrap();
        delete_season(&conn, season.id).unwrap();
        let seasons = list_seasons(&conn, save.id).unwrap();
        assert!(seasons.is_empty());
    }

    #[test]
    fn delete_season_not_found() {
        let conn = setup_test_db();
        let result = delete_season(&conn, 9999);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn delete_season_cleans_up_orphan_players() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let s1 = create_season(&conn, save.id, "2030-11-15").unwrap();
        let s2 = create_season(&conn, save.id, "2031-11-15").unwrap();

        // Insert a player manually
        conn.execute(
            "INSERT INTO players (save_id, fm_uid, name) VALUES (?1, ?2, ?3)",
            rusqlite::params![save.id, 12345, "John Smith"],
        ).unwrap();
        let player_id = conn.last_insert_rowid();

        // Player has seasons in both s1 and s2
        conn.execute(
            "INSERT INTO player_seasons (player_id, season_id, position, data) VALUES (?1, ?2, 'ST', '{}')",
            rusqlite::params![player_id, s1.id],
        ).unwrap();
        conn.execute(
            "INSERT INTO player_seasons (player_id, season_id, position, data) VALUES (?1, ?2, 'ST', '{}')",
            rusqlite::params![player_id, s2.id],
        ).unwrap();

        // Delete s1 — player still has s2, should NOT be orphaned
        delete_season(&conn, s1.id).unwrap();
        let player_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM players WHERE id = ?1",
            rusqlite::params![player_id],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(player_count, 1);

        // Delete s2 — player is now orphaned, should be removed
        delete_season(&conn, s2.id).unwrap();
        let player_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM players WHERE id = ?1",
            rusqlite::params![player_id],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(player_count, 0);
    }

    // ── import_season tests ─────────────────────────────────────────────

    /// Test helper: create a minimal ParsedPlayer.
    fn make_player(uid: u32, name: &str) -> ParsedPlayer {
        ParsedPlayer {
            uid,
            name: name.to_string(),
            positions: vec![crate::parser::types::Position {
                role: crate::parser::types::Role::ST,
                sides: vec![crate::parser::types::Side::C],
            }],
            ..Default::default()
        }
    }

    /// Test helper: create a ParsedPlayer with club set.
    fn make_player_with_club(uid: u32, name: &str, club: &str) -> ParsedPlayer {
        let mut p = make_player(uid, name);
        p.club = Some(club.to_string());
        p
    }

    #[test]
    fn import_season_basic() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);

        let players = vec![
            make_player(111, "Alice Smith"),
            make_player(222, "Bob Jones"),
        ];

        let result = import_season(&conn, save.id, "2030-11-15", players).unwrap();

        assert_eq!(result.total_players, 2);
        assert_eq!(result.new_players, 2);
        assert_eq!(result.matched_players, 0);
        assert_eq!(result.season.in_game_date, "2030-11-15");
        assert_eq!(result.season.label, "2030/31");
    }

    #[test]
    fn import_season_empty_players_rejected() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);

        let result = import_season(&conn, save.id, "2030-11-15", vec![]);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no players"));
    }

    #[test]
    fn import_season_save_not_found() {
        let conn = setup_test_db();

        let players = vec![make_player(111, "Alice")];
        let result = import_season(&conn, 9999, "2030-11-15", players);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn import_season_invalid_date_rejected() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);

        let players = vec![make_player(111, "Alice")];
        let result = import_season(&conn, save.id, "not-a-date", players);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid date format"));
    }

    #[test]
    fn import_season_duplicate_rejected() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);

        let players = vec![make_player(111, "Alice")];
        import_season(&conn, save.id, "2030-11-15", players).unwrap();

        let players2 = vec![make_player(111, "Alice")];
        let result = import_season(&conn, save.id, "2030-11-15", players2);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("already exists"));
        assert!(err.to_string().contains("1 players"));
    }

    #[test]
    fn import_season_matches_existing_player() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);

        // First season
        let p1 = make_player(111, "Alice Smith");
        import_season(&conn, save.id, "2030-11-15", vec![p1]).unwrap();

        // Second season with same player
        let p2 = make_player(111, "Alice Smith");
        let result = import_season(&conn, save.id, "2031-11-15", vec![p2]).unwrap();

        assert_eq!(result.total_players, 1);
        assert_eq!(result.new_players, 0);
        assert_eq!(result.matched_players, 1);

        // Verify only 1 player record exists
        let player_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM players WHERE save_id = ?1",
            rusqlite::params![save.id],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(player_count, 1);

        // Verify 2 season records
        let season_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM seasons WHERE save_id = ?1",
            rusqlite::params![save.id],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(season_count, 2);
    }

    #[test]
    fn import_season_uid_reuse_different_name_creates_new() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);

        // Player A with UID 111
        let p1 = make_player(111, "Alice Smith");
        import_season(&conn, save.id, "2030-11-15", vec![p1]).unwrap();

        // Player B with same UID but different name
        let p2 = make_player(111, "Alicia Smythe");
        let result = import_season(&conn, save.id, "2031-11-15", vec![p2]).unwrap();

        assert_eq!(result.total_players, 1);
        assert_eq!(result.new_players, 1);
        assert_eq!(result.matched_players, 0);

        // Verify 2 player records exist (different names = different players)
        let player_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM players WHERE save_id = ?1",
            rusqlite::params![save.id],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(player_count, 2);
    }

    #[test]
    fn import_season_json_blob_stores_full_data() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);

        let mut player = make_player(111, "Alice Smith");
        player.age = Some(24);
        player.club = Some("Arsenal".to_string());
        player.appearances_started = Some(15);
        player.appearances_sub = Some(5);
        player.minutes = Some(1380);
        player.wage.wage_per_week = Some(45000.0);
        player.transfer_value.high = Some(12000000.0);
        player.contract_expires = Some("2028-06-30".to_string());

        import_season(&conn, save.id, "2030-11-15", vec![player]).unwrap();

        // Read back JSON blob
        let stored: String = conn.query_row(
            "SELECT data FROM player_seasons WHERE season_id IN \
             (SELECT id FROM seasons WHERE save_id = ?1)",
            rusqlite::params![save.id],
            |row| row.get(0),
        ).unwrap();

        let back: ParsedPlayer = serde_json::from_str(&stored).unwrap();
        assert_eq!(back.name, "Alice Smith");
        assert_eq!(back.uid, 111);
        assert_eq!(back.age, Some(24));
        assert_eq!(back.club, Some("Arsenal".to_string()));
        assert_eq!(back.appearances_started, Some(15));
        assert_eq!(back.appearances_sub, Some(5));
        assert_eq!(back.minutes, Some(1380));
        assert_eq!(back.wage.wage_per_week, Some(45000.0));
        assert_eq!(back.transfer_value.high, Some(12000000.0));
        assert_eq!(back.contract_expires, Some("2028-06-30".to_string()));
    }

    #[test]
    fn import_season_extracts_queryable_columns() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);

        let mut player = make_player_with_club(111, "Alice Smith", "AC Milan");
        player.age = Some(27);
        player.nationality = Some(crate::parser::types::Nationality {
            code: Some("ITA".to_string()),
            name: "Italian".to_string(),
        });
        player.positions = vec![
            crate::parser::types::Position {
                role: crate::parser::types::Role::AM,
                sides: vec![crate::parser::types::Side::L, crate::parser::types::Side::C],
            },
            crate::parser::types::Position {
                role: crate::parser::types::Role::ST,
                sides: vec![crate::parser::types::Side::C],
            },
        ];
        player.appearances_started = Some(20);
        player.appearances_sub = Some(8);
        player.minutes = Some(1800);
        player.wage.wage_per_week = Some(85000.0);
        player.transfer_value.high = Some(25000000.0);
        player.contract_expires = Some("2027-05-31".to_string());

        import_season(&conn, save.id, "2030-11-15", vec![player]).unwrap();

        let row: (String, i64, String, String, i64, i64, i64, Option<f64>, Option<f64>, Option<String>) = conn.query_row(
            "SELECT club, age, nationality, position, minutes,
                    appearances_started, appearances_sub,
                    wage_per_week, transfer_value_high, contract_expires
             FROM player_seasons WHERE season_id IN \
             (SELECT id FROM seasons WHERE save_id = ?1)",
            rusqlite::params![save.id],
            |row| Ok((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?,
                row.get(4)?, row.get(5)?, row.get(6)?,
                row.get(7)?, row.get(8)?, row.get(9)?,
            )),
        ).unwrap();

        assert_eq!(row.0, "AC Milan");
        assert_eq!(row.1, 27);
        assert_eq!(row.2, "Italian");
        assert_eq!(row.3, "AM (L, C), ST (C)");
        assert_eq!(row.4, 1800);
        assert_eq!(row.5, 20);
        assert_eq!(row.6, 8);
        assert!(row.7.is_some());
        assert!(row.8.is_some());
        assert_eq!(row.9, Some("2027-05-31".to_string()));
    }

    #[test]
    fn import_season_rollback_on_failure() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);

        // First import succeeds
        let p1 = make_player(111, "Alice");
        import_season(&conn, save.id, "2030-11-15", vec![p1]).unwrap();

        // Second import with duplicate date fails
        let p2 = make_player(222, "Bob");
        let result = import_season(&conn, save.id, "2030-11-15", vec![p2]);
        assert!(result.is_err());

        // Verify no partial data was written (only the first season exists)
        let season_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM seasons WHERE save_id = ?1",
            rusqlite::params![save.id],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(season_count, 1);

        let player_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM players WHERE save_id = ?1",
            rusqlite::params![save.id],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(player_count, 1); // Only Alice, not Bob
    }

    // ── retrieval tests ──────────────────────────────────────────────────
    #[test]
    fn get_players_for_season_basic() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let season = import_season(&conn, save.id, "2030-11-15", vec![
            make_player_with_club(111, "Charlie Brown", "Man Utd"),
            make_player_with_club(222, "Alice Smith", "Arsenal"),
            make_player_with_club(333, "Bob Jones", "Chelsea"),
        ]).unwrap().season;
        let players = get_players_for_season(&conn, season.id).unwrap();
        assert_eq!(players.len(), 3);
        assert_eq!(players[0].player_name, "Alice Smith");
        assert_eq!(players[1].player_name, "Bob Jones");
        assert_eq!(players[2].player_name, "Charlie Brown");
        assert_eq!(players[0].fm_uid, 222);
        assert_eq!(players[0].club, Some("Arsenal".to_string()));
        assert!(players[0].data.as_ref().is_some());
        let data = players[0].data.as_ref().unwrap();
        assert_eq!(data.name, "Alice Smith");
        assert_eq!(data.uid, 222);
    }
    #[test]
    fn get_players_for_season_empty() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let season = create_season(&conn, save.id, "2030-11-15").unwrap();
        let players = get_players_for_season(&conn, season.id).unwrap();
        assert!(players.is_empty());
    }
    #[test]
    fn get_players_for_season_nonexistent_season() {
        let conn = setup_test_db();
        let players = get_players_for_season(&conn, 9999).unwrap();
        assert!(players.is_empty());
    }
    #[test]
    fn get_player_career_basic() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let mut p1 = make_player_with_club(111, "Dani", "Real Madrid");
        p1.age = Some(22);
        import_season(&conn, save.id, "2029-11-15", vec![p1]).unwrap();
        let mut p2 = make_player_with_club(111, "Dani", "Real Madrid");
        p2.age = Some(23);
        import_season(&conn, save.id, "2030-11-15", vec![p2]).unwrap();
        let mut p3 = make_player_with_club(111, "Dani", "Real Madrid");
        p3.age = Some(21);
        import_season(&conn, save.id, "2028-06-01", vec![p3]).unwrap();
        let player_id: i64 = conn.query_row(
            "SELECT player_id FROM player_seasons LIMIT 1", [], |r| r.get(0),
        ).unwrap();
        let career = get_player_career(&conn, save.id, player_id).unwrap();
        assert_eq!(career.len(), 3);
        // verify chronological order via re-query
        let career_seasons: Vec<String> = {
            let mut stmt = conn.prepare(
                "SELECT s.in_game_date FROM player_seasons ps \
                 JOIN seasons s ON ps.season_id = s.id \
                 WHERE ps.player_id = ?1 ORDER BY s.in_game_date ASC"
            ).unwrap();
            stmt.query_map(rusqlite::params![player_id], |r| r.get(0))
                .unwrap().filter_map(|r| r.ok()).collect()
        };
        assert_eq!(career_seasons[0], "2028-06-01");
        assert_eq!(career_seasons[1], "2029-11-15");
        assert_eq!(career_seasons[2], "2030-11-15");
    }
    #[test]
    fn get_player_career_nonexistent_player() {
        let conn = setup_test_db();
        let career = get_player_career(&conn, 1, 9999).unwrap();
        assert!(career.is_empty());
    }
    #[test]
    fn get_player_career_nonexistent_save() {
        let conn = setup_test_db();
        let career = get_player_career(&conn, 9999, 1).unwrap();
        assert!(career.is_empty());
    }
    #[test]
    fn get_latest_season_basic() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        create_season(&conn, save.id, "2028-06-01").unwrap();
        create_season(&conn, save.id, "2029-11-15").unwrap();
        create_season(&conn, save.id, "2030-11-15").unwrap();
        let latest = get_latest_season(&conn, save.id).unwrap().unwrap();
        assert_eq!(latest.in_game_date, "2030-11-15");
    }
    #[test]
    fn get_latest_season_empty() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let latest = get_latest_season(&conn, save.id).unwrap();
        assert!(latest.is_none());
    }
    #[test]
    fn get_latest_season_nonexistent_save() {
        let conn = setup_test_db();
        let latest = get_latest_season(&conn, 9999).unwrap();
        assert!(latest.is_none());
    }
    #[test]
    fn get_players_for_season_json_failure_graceful() {
        use serde_json;

        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let season = create_season(&conn, save.id, "2030-11-15").unwrap();
        // Bad player: invalid JSON blob
        conn.execute(
            "INSERT INTO players (save_id, fm_uid, name) VALUES (?1, ?2, ?3)",
            rusqlite::params![save.id, 111, "Bad Player"],
        ).unwrap();
        let player_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO player_seasons (player_id, season_id, position, club, age, data)              VALUES (?1, ?2, 'ST', 'Bad Club', 25, 'not valid json {')",
            rusqlite::params![player_id, season.id],
        ).unwrap();
        // Good player: construct valid ParsedPlayer JSON and bind as parameter
        let good_player = ParsedPlayer {
            uid: 222,
            name: "Good Player".to_string(),
            positions: vec![],
            ..Default::default()
        };
        let good_json = serde_json::to_string(&good_player).unwrap();
        conn.execute(
            "INSERT INTO players (save_id, fm_uid, name) VALUES (?1, ?2, ?3)",
            rusqlite::params![save.id, 222, "Good Player"],
        ).unwrap();
        let player_id2 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO player_seasons (player_id, season_id, position, club, age, data)              VALUES (?1, ?2, 'AM', 'Good Club', 28, ?3)",
            rusqlite::params![player_id2, season.id, &good_json],
        ).unwrap();

        let players = get_players_for_season(&conn, season.id).unwrap();
        assert_eq!(players.len(), 2);
        let bad = players.iter().find(|p| p.player_name == "Bad Player").unwrap();
        assert!(bad.data.is_none());
        assert_eq!(bad.club, Some("Bad Club".to_string()));
        assert_eq!(bad.age, Some(25));
        assert_eq!(bad.position, "ST");
        assert_eq!(bad.fm_uid, 111);
        let good = players.iter().find(|p| p.player_name == "Good Player").unwrap();
        assert!(good.data.is_some());
        assert_eq!(good.club, Some("Good Club".to_string()));
        assert_eq!(good.age, Some(28));
    }
}
