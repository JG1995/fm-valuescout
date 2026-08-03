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

pub const SNAPSHOT_SCHEMA_SQL: &str = "
CREATE TABLE saves (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL CHECK (trim(name) <> ''),
    is_active INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0, 1)),
    created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE UNIQUE INDEX idx_saves_one_active
    ON saves(is_active)
    WHERE is_active = 1;

CREATE TABLE snapshots (
    id INTEGER PRIMARY KEY,
    save_id INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    is_current INTEGER NOT NULL DEFAULT 0 CHECK (is_current IN (0, 1)),
    schema_version INTEGER NOT NULL,
    generated_at_utc TEXT NOT NULL,
    game_version TEXT NOT NULL,
    supported_game_version TEXT NOT NULL,
    bridge_version TEXT NOT NULL,
    protocol_version INTEGER NOT NULL,
    game_date TEXT,
    game_date_source TEXT NOT NULL,
    scan_truncated INTEGER NOT NULL CHECK (scan_truncated IN (0, 1)),
    max_accepted INTEGER CHECK (max_accepted IS NULL OR max_accepted >= 0),
    player_count INTEGER NOT NULL CHECK (player_count >= 0),
    loaded_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    CHECK (scan_truncated = 0 OR max_accepted IS NOT NULL)
);

CREATE UNIQUE INDEX idx_snapshots_one_current_per_save
    ON snapshots(save_id)
    WHERE is_current = 1;

CREATE TABLE players (
    snapshot_id INTEGER NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
    uid INTEGER NOT NULL,
    ca INTEGER NOT NULL,
    pa INTEGER NOT NULL,
    name TEXT NOT NULL,
    birth_year INTEGER NOT NULL,
    birth_day_of_year INTEGER NOT NULL,
    age INTEGER,
    nationalities_json TEXT NOT NULL,
    height_cm INTEGER,
    preferred_foot TEXT NOT NULL,
    positions_json TEXT NOT NULL,
    attributes_json TEXT NOT NULL,
    hidden_attributes_json TEXT NOT NULL,
    personality_json TEXT NOT NULL,
    weekly_wage_gbp INTEGER,
    contract_expiry_year INTEGER,
    contract_expiry_day_of_year INTEGER,
    transfer_listed INTEGER CHECK (transfer_listed IS NULL OR transfer_listed IN (0, 1)),
    loan_listed INTEGER CHECK (loan_listed IS NULL OR loan_listed IN (0, 1)),
    not_for_sale INTEGER CHECK (not_for_sale IS NULL OR not_for_sale IN (0, 1)),
    set_for_release INTEGER CHECK (set_for_release IS NULL OR set_for_release IN (0, 1)),
    market_value_gbp INTEGER,
    reputation_current INTEGER,
    reputation_world INTEGER,
    current_club TEXT,
    parent_club TEXT,
    on_loan INTEGER CHECK (on_loan IS NULL OR on_loan IN (0, 1)),
    division TEXT,
    team_level TEXT,
    PRIMARY KEY (snapshot_id, uid)
);

CREATE INDEX idx_players_snapshot_name
    ON players(snapshot_id, name COLLATE NOCASE);

CREATE INDEX idx_players_snapshot_ca
    ON players(snapshot_id, ca DESC);
";

pub const PLAYER_ROLE_SCORES_SQL: &str = "
CREATE TABLE player_role_scores (
    snapshot_id INTEGER NOT NULL,
    uid INTEGER NOT NULL,
    role_id TEXT NOT NULL,
    phase TEXT NOT NULL CHECK (phase IN ('in_possession', 'out_of_possession')),
    score INTEGER CHECK (score IS NULL OR (score >= 0 AND score <= 100)),
    PRIMARY KEY (snapshot_id, uid, role_id),
    FOREIGN KEY (snapshot_id, uid) REFERENCES players(snapshot_id, uid) ON DELETE CASCADE
);

CREATE INDEX idx_player_role_scores_snapshot_role
    ON player_role_scores(snapshot_id, role_id);
";

pub const PLANNER_CLUB_FAMILY_SQL: &str = "
CREATE TABLE planner_club_settings (
    save_id INTEGER PRIMARY KEY REFERENCES saves(id) ON DELETE CASCADE,
    primary_club TEXT NOT NULL CHECK (trim(primary_club) <> '')
);

CREATE TABLE planner_club_sources (
    id INTEGER PRIMARY KEY,
    save_id INTEGER NOT NULL REFERENCES planner_club_settings(save_id) ON DELETE CASCADE,
    team TEXT NOT NULL CHECK (team IN ('senior', 'reserves', 'youth')),
    club_name TEXT NOT NULL CHECK (trim(club_name) <> ''),
    team_level TEXT CHECK (team_level IS NULL OR team_level IN ('senior', 'reserve', 'youth')),
    is_primary INTEGER NOT NULL DEFAULT 0 CHECK (is_primary IN (0, 1)),
    UNIQUE (save_id, team, club_name, team_level)
);

CREATE INDEX idx_planner_club_sources_save_team
    ON planner_club_sources(save_id, team);
";

pub const PLANNER_TACTIC_SQL: &str = "
CREATE TABLE planner_tactics (
    save_id INTEGER PRIMARY KEY REFERENCES saves(id) ON DELETE CASCADE,
    ip_weight REAL NOT NULL DEFAULT 0.5 CHECK (ip_weight >= 0 AND ip_weight <= 1)
);

CREATE TABLE planner_tactic_lanes (
    id INTEGER PRIMARY KEY,
    save_id INTEGER NOT NULL REFERENCES planner_tactics(save_id) ON DELETE CASCADE,
    lane_order INTEGER NOT NULL CHECK (lane_order >= 0 AND lane_order < 11),
    lane_id TEXT NOT NULL CHECK (trim(lane_id) <> ''),
    ip_position TEXT NOT NULL CHECK (trim(ip_position) <> ''),
    ip_role_id TEXT NOT NULL CHECK (trim(ip_role_id) <> ''),
    oop_position TEXT NOT NULL CHECK (trim(oop_position) <> ''),
    oop_role_id TEXT NOT NULL CHECK (trim(oop_role_id) <> ''),
    UNIQUE (save_id, lane_order),
    UNIQUE (save_id, lane_id)
);

CREATE INDEX idx_planner_tactic_lanes_save_order
    ON planner_tactic_lanes(save_id, lane_order);
";

pub const PLANNER_DEPTH_SQL: &str = "
CREATE TABLE planner_strings (
    id INTEGER PRIMARY KEY,
    save_id INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    team TEXT NOT NULL CHECK (team IN ('senior', 'reserves', 'youth')),
    string_order INTEGER NOT NULL CHECK (string_order >= 0),
    UNIQUE (save_id, team, string_order)
);

CREATE INDEX idx_planner_strings_save_team_order
    ON planner_strings(save_id, team, string_order);

CREATE TABLE planner_assignments (
    id INTEGER PRIMARY KEY,
    save_id INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    string_id INTEGER NOT NULL REFERENCES planner_strings(id) ON DELETE CASCADE,
    lane_id TEXT NOT NULL CHECK (trim(lane_id) <> ''),
    player_uid INTEGER NOT NULL,
    last_known_name TEXT NOT NULL CHECK (trim(last_known_name) <> ''),
    UNIQUE (save_id, player_uid),
    UNIQUE (string_id, lane_id)
);

CREATE INDEX idx_planner_assignments_string
    ON planner_assignments(string_id);
";

pub const PLANNER_ASSIGNMENT_PROVENANCE_SQL: &str = "
ALTER TABLE planner_assignments
    ADD COLUMN provenance TEXT NOT NULL DEFAULT 'manual'
    CHECK (provenance IN ('manual', 'optimizer'));
";

pub const PLANNER_LANE_WEIGHTS_SQL: &str = "
DROP TABLE planner_tactic_lanes;
DROP TABLE planner_tactics;

CREATE TABLE planner_tactic_lanes (
    id INTEGER PRIMARY KEY,
    save_id INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    lane_order INTEGER NOT NULL CHECK (lane_order >= 0 AND lane_order < 11),
    lane_id TEXT NOT NULL CHECK (trim(lane_id) <> ''),
    ip_weight REAL NOT NULL DEFAULT 0.5 CHECK (ip_weight >= 0 AND ip_weight <= 1),
    ip_position TEXT NOT NULL CHECK (trim(ip_position) <> ''),
    ip_role_id TEXT NOT NULL CHECK (trim(ip_role_id) <> ''),
    oop_position TEXT NOT NULL CHECK (trim(oop_position) <> ''),
    oop_role_id TEXT NOT NULL CHECK (trim(oop_role_id) <> ''),
    UNIQUE (save_id, lane_order),
    UNIQUE (save_id, lane_id)
);

CREATE INDEX idx_planner_tactic_lanes_save_order
    ON planner_tactic_lanes(save_id, lane_order);
";

pub const PLANNER_LANE_IMPORTANCE_RANKS_SQL: &str = "
ALTER TABLE planner_tactic_lanes
    ADD COLUMN importance_rank INTEGER CHECK (importance_rank IS NULL OR (importance_rank >= 1 AND importance_rank <= 11));

CREATE UNIQUE INDEX idx_planner_tactic_lanes_save_importance_rank
    ON planner_tactic_lanes(save_id, importance_rank)
    WHERE importance_rank IS NOT NULL;
";

pub fn all() -> &'static [Migration] {
    &[
        Migration {
            version: 1,
            description: "create_demo_value_table",
            sql: INITIAL_DEMO_VALUE_SQL,
        },
        Migration {
            version: 2,
            description: "create_snapshot_schema",
            sql: SNAPSHOT_SCHEMA_SQL,
        },
        Migration {
            version: 3,
            description: "create_player_role_scores",
            sql: PLAYER_ROLE_SCORES_SQL,
        },
        Migration {
            version: 4,
            description: "create_planner_club_family",
            sql: PLANNER_CLUB_FAMILY_SQL,
        },
        Migration {
            version: 5,
            description: "create_planner_tactic",
            sql: PLANNER_TACTIC_SQL,
        },
        Migration {
            version: 6,
            description: "create_planner_depth",
            sql: PLANNER_DEPTH_SQL,
        },
        Migration {
            version: 7,
            description: "add_planner_assignment_provenance",
            sql: PLANNER_ASSIGNMENT_PROVENANCE_SQL,
        },
        Migration {
            version: 8,
            description: "move_tactic_weight_to_lanes",
            sql: PLANNER_LANE_WEIGHTS_SQL,
        },
        Migration {
            version: 9,
            description: "add_planner_lane_importance_ranks",
            sql: PLANNER_LANE_IMPORTANCE_RANKS_SQL,
        },
    ]
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

    use rusqlite::params;

    const INSERT_SNAPSHOT_SQL: &str = "
        INSERT INTO snapshots (
            save_id,
            is_current,
            schema_version,
            generated_at_utc,
            game_version,
            supported_game_version,
            bridge_version,
            protocol_version,
            game_date_source,
            scan_truncated,
            max_accepted,
            player_count
        )
        VALUES (?1, ?2, 5, '2026-07-29T10:00:00Z', '26.3.2', '26.3', '0.1.0', 1, 'memory', ?3, ?4, 0)
    ";

    fn open_migrated(db_path: &Path) -> Connection {
        let conn = Connection::open(db_path).expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        apply(&conn).expect("apply migrations");
        conn
    }

    fn table_columns(conn: &Connection, table: &str) -> Vec<String> {
        let mut statement = conn
            .prepare("SELECT name FROM pragma_table_info(?1) ORDER BY cid")
            .expect("prepare table info query");
        statement
            .query_map([table], |row| row.get(0))
            .expect("query table info")
            .collect::<Result<Vec<_>, _>>()
            .expect("read table columns")
    }

    #[test]
    fn opening_fresh_db_applies_all_migrations_and_creates_demo_value() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("migration-test.db");
        let conn = open_migrated(&db_path);

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user_version");
        assert_eq!(version, 9);

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
    fn migrates_v6_planner_assignments_to_manual_provenance() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("planner-v6-migration-test.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 6) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v6");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set migration version");
        }
        conn.execute("INSERT INTO saves (name) VALUES (?1)", ["Legacy save"])
            .expect("insert legacy save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO planner_strings (save_id, team, string_order) VALUES (?1, 'senior', 0)",
            [save_id],
        )
        .expect("insert legacy string");
        let string_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO planner_assignments (
                 save_id, string_id, lane_id, player_uid, last_known_name
             ) VALUES (?1, ?2, 'goalkeeper', 77, 'Legacy Player')",
            params![save_id, string_id],
        )
        .expect("insert legacy assignment");

        apply(&conn).expect("migrate legacy assignment");

        let provenance: String = conn
            .query_row(
                "SELECT provenance FROM planner_assignments WHERE player_uid = 77",
                [],
                |row| row.get(0),
            )
            .expect("read migrated assignment provenance");
        assert_eq!(provenance, "manual");
    }

    #[test]
    fn migrates_v7_tactics_to_lane_weights_without_deleting_assignments() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("planner-v7-migration-test.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 7) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v7");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set migration version");
        }
        conn.execute("INSERT INTO saves (name) VALUES (?1)", ["Legacy save"])
            .expect("insert legacy save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO planner_tactics (save_id, ip_weight) VALUES (?1, 0.7)",
            [save_id],
        )
        .expect("insert legacy tactic");
        conn.execute(
            "INSERT INTO planner_tactic_lanes (
                 save_id, lane_order, lane_id, ip_position, ip_role_id, oop_position, oop_role_id
             ) VALUES (?1, 0, 'goalkeeper', 'GK', 'goalkeeper_ip', 'GK', 'line_holding_keeper_oop')",
            [save_id],
        )
        .expect("insert legacy lane");
        conn.execute(
            "INSERT INTO planner_strings (save_id, team, string_order) VALUES (?1, 'senior', 0)",
            [save_id],
        )
        .expect("insert legacy string");
        let string_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO planner_assignments (
                 save_id, string_id, lane_id, player_uid, last_known_name
             ) VALUES (?1, ?2, 'goalkeeper', 77, 'Legacy Player')",
            params![save_id, string_id],
        )
        .expect("insert legacy assignment");

        apply(&conn).expect("migrate legacy tactic");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user version");
        assert_eq!(version, 9);
        let tactic_table_exists: bool = conn
            .query_row(
                "SELECT EXISTS(
                     SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'planner_tactics'
                 )",
                [],
                |row| row.get(0),
            )
            .expect("check removed tactic table");
        assert!(!tactic_table_exists);
        let lane_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM planner_tactic_lanes", [], |row| {
                row.get(0)
            })
            .expect("count reset tactic lanes");
        assert_eq!(lane_count, 0);
        let assignment_name: String = conn
            .query_row(
                "SELECT last_known_name FROM planner_assignments WHERE player_uid = 77",
                [],
                |row| row.get(0),
            )
            .expect("read preserved assignment");
        assert_eq!(assignment_name, "Legacy Player");
        let foreign_key_parent: String = conn
            .query_row(
                "SELECT \"table\" FROM pragma_foreign_key_list('planner_tactic_lanes')",
                [],
                |row| row.get(0),
            )
            .expect("read lane foreign key");
        assert_eq!(foreign_key_parent, "saves");
        let foreign_key_errors: i64 = conn
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .expect("check foreign keys");
        assert_eq!(foreign_key_errors, 0);
    }

    #[test]
    fn migrates_v8_tactic_lanes_with_no_importance_rank() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("planner-v8-migration-test.db"))
            .expect("open test db");
        for migration in all().iter().filter(|migration| migration.version <= 8) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v8");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set migration version");
        }
        conn.execute("INSERT INTO saves (name) VALUES (?1)", ["Legacy save"])
            .expect("insert legacy save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO planner_tactic_lanes (
                 save_id, lane_order, lane_id, ip_weight, ip_position, ip_role_id, oop_position, oop_role_id
             ) VALUES (?1, 0, 'goalkeeper', 0.5, 'GK', 'goalkeeper_ip', 'GK', 'line_holding_keeper_oop')",
            [save_id],
        )
        .expect("insert v8 lane");

        apply(&conn).expect("migrate v8 tactic lane");

        let importance_rank: Option<i64> = conn
            .query_row(
                "SELECT importance_rank FROM planner_tactic_lanes WHERE save_id = ?1",
                [save_id],
                |row| row.get(0),
            )
            .expect("read migrated importance rank");
        assert_eq!(importance_rank, None);
    }

    #[test]
    fn opening_fresh_db_applies_snapshot_schema_tables() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("snapshot-migration-test.db");
        let conn = open_migrated(&db_path);

        for expected_table in ["saves", "snapshots", "players"] {
            let table_name: String = conn
                .query_row(
                    "SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [expected_table],
                    |row| row.get(0),
                )
                .expect("read snapshot table from sqlite_master");
            assert_eq!(table_name, expected_table);
        }
    }

    #[test]
    fn opening_fresh_db_applies_version_3_and_creates_player_role_scores() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("role-scores-migration-test.db");
        let conn = open_migrated(&db_path);

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user_version");
        assert_eq!(version, 9);

        let table_name: String = conn
            .query_row(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'player_role_scores'",
                [],
                |row| row.get(0),
            )
            .expect("read player_role_scores from sqlite_master");
        assert_eq!(table_name, "player_role_scores");

        assert_eq!(
            table_columns(&conn, "player_role_scores"),
            ["snapshot_id", "uid", "role_id", "phase", "score"]
        );
    }

    #[test]
    fn snapshot_tables_contain_dump_and_query_fields() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("snapshot-columns-test.db");
        let conn = open_migrated(&db_path);

        assert_eq!(
            table_columns(&conn, "saves"),
            [
                "id",
                "name",
                "is_active",
                "created_at_utc",
                "updated_at_utc"
            ]
        );
        assert_eq!(
            table_columns(&conn, "snapshots"),
            [
                "id",
                "save_id",
                "is_current",
                "schema_version",
                "generated_at_utc",
                "game_version",
                "supported_game_version",
                "bridge_version",
                "protocol_version",
                "game_date",
                "game_date_source",
                "scan_truncated",
                "max_accepted",
                "player_count",
                "loaded_at_utc",
            ]
        );
        assert_eq!(
            table_columns(&conn, "players"),
            [
                "snapshot_id",
                "uid",
                "ca",
                "pa",
                "name",
                "birth_year",
                "birth_day_of_year",
                "age",
                "nationalities_json",
                "height_cm",
                "preferred_foot",
                "positions_json",
                "attributes_json",
                "hidden_attributes_json",
                "personality_json",
                "weekly_wage_gbp",
                "contract_expiry_year",
                "contract_expiry_day_of_year",
                "transfer_listed",
                "loan_listed",
                "not_for_sale",
                "set_for_release",
                "market_value_gbp",
                "reputation_current",
                "reputation_world",
                "current_club",
                "parent_club",
                "on_loan",
                "division",
                "team_level",
            ]
        );
    }

    #[test]
    fn snapshot_schema_creates_current_and_player_query_indexes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("snapshot-indexes-test.db");
        let conn = open_migrated(&db_path);

        let mut statement = conn
            .prepare(
                "SELECT name
                 FROM sqlite_master
                 WHERE type = 'index' AND name LIKE 'idx_%'
                 ORDER BY name",
            )
            .expect("prepare index query");
        let indexes = statement
            .query_map([], |row| row.get::<_, String>(0))
            .expect("query indexes")
            .collect::<Result<Vec<_>, _>>()
            .expect("read indexes");

        assert_eq!(
            indexes,
            [
                "idx_planner_assignments_string",
                "idx_planner_club_sources_save_team",
                "idx_planner_strings_save_team_order",
                "idx_planner_tactic_lanes_save_importance_rank",
                "idx_planner_tactic_lanes_save_order",
                "idx_player_role_scores_snapshot_role",
                "idx_players_snapshot_ca",
                "idx_players_snapshot_name",
                "idx_saves_one_active",
                "idx_snapshots_one_current_per_save",
            ]
        );
    }

    #[test]
    fn snapshot_schema_rejects_a_second_active_save() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("active-save-constraint-test.db");
        let conn = open_migrated(&db_path);
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES (?1, 1)",
            ["First save"],
        )
        .expect("insert first active save");

        let error = conn
            .execute(
                "INSERT INTO saves (name, is_active) VALUES (?1, 1)",
                ["Second save"],
            )
            .expect_err("reject second active save");

        assert_eq!(
            error.sqlite_error_code(),
            Some(rusqlite::ErrorCode::ConstraintViolation)
        );
    }

    #[test]
    fn snapshot_schema_rejects_a_second_current_snapshot_for_one_save() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("current-snapshot-constraint-test.db");
        let conn = open_migrated(&db_path);
        conn.execute("INSERT INTO saves (name) VALUES (?1)", ["Test save"])
            .expect("insert save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            INSERT_SNAPSHOT_SQL,
            params![save_id, true, false, Option::<i64>::None],
        )
        .expect("insert first current snapshot");

        let error = conn
            .execute(
                INSERT_SNAPSHOT_SQL,
                params![save_id, true, false, Option::<i64>::None],
            )
            .expect_err("reject second current snapshot");

        assert_eq!(
            error.sqlite_error_code(),
            Some(rusqlite::ErrorCode::ConstraintViolation)
        );
    }

    #[test]
    fn snapshot_schema_rejects_truncated_snapshot_without_max_accepted() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("truncation-constraint-test.db");
        let conn = open_migrated(&db_path);
        conn.execute("INSERT INTO saves (name) VALUES (?1)", ["Test save"])
            .expect("insert save");
        let save_id = conn.last_insert_rowid();

        let error = conn
            .execute(
                INSERT_SNAPSHOT_SQL,
                params![save_id, false, true, Option::<i64>::None],
            )
            .expect_err("reject truncated snapshot without max accepted");

        assert_eq!(
            error.sqlite_error_code(),
            Some(rusqlite::ErrorCode::ConstraintViolation)
        );
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
        assert_eq!(version, 9);
    }

    #[test]
    fn registers_monotonic_migrations() {
        let migrations = all();

        assert_eq!(migrations.len(), 9);
        assert_eq!(migrations[0].version, 1);
        assert_eq!(migrations[0].description, "create_demo_value_table");
        assert_eq!(migrations[0].sql, INITIAL_DEMO_VALUE_SQL);
        assert_eq!(migrations[1].version, 2);
        assert_eq!(migrations[1].description, "create_snapshot_schema");
        assert_eq!(migrations[1].sql, SNAPSHOT_SCHEMA_SQL);
        assert_eq!(migrations[2].version, 3);
        assert_eq!(migrations[2].description, "create_player_role_scores");
        assert_eq!(migrations[2].sql, PLAYER_ROLE_SCORES_SQL);
        assert_eq!(migrations[3].version, 4);
        assert_eq!(migrations[3].description, "create_planner_club_family");
        assert_eq!(migrations[3].sql, PLANNER_CLUB_FAMILY_SQL);
        assert_eq!(migrations[4].version, 5);
        assert_eq!(migrations[4].description, "create_planner_tactic");
        assert_eq!(migrations[4].sql, PLANNER_TACTIC_SQL);
        assert_eq!(migrations[5].version, 6);
        assert_eq!(migrations[5].description, "create_planner_depth");
        assert_eq!(migrations[5].sql, PLANNER_DEPTH_SQL);
        assert_eq!(migrations[6].version, 7);
        assert_eq!(
            migrations[6].description,
            "add_planner_assignment_provenance"
        );
        assert_eq!(migrations[6].sql, PLANNER_ASSIGNMENT_PROVENANCE_SQL);
        assert_eq!(migrations[7].version, 8);
        assert_eq!(migrations[7].description, "move_tactic_weight_to_lanes");
        assert_eq!(migrations[7].sql, PLANNER_LANE_WEIGHTS_SQL);
        assert_eq!(migrations[8].version, 9);
        assert_eq!(
            migrations[8].description,
            "add_planner_lane_importance_ranks"
        );
        assert_eq!(migrations[8].sql, PLANNER_LANE_IMPORTANCE_RANKS_SQL);
    }

    #[test]
    fn opening_fresh_db_applies_planner_club_family_schema() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("planner-migration-test.db"));

        for expected_table in ["planner_club_settings", "planner_club_sources"] {
            let table_name: String = conn
                .query_row(
                    "SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [expected_table],
                    |row| row.get(0),
                )
                .expect("read planner table from sqlite_master");
            assert_eq!(table_name, expected_table);
        }

        assert_eq!(
            table_columns(&conn, "planner_club_sources"),
            [
                "id",
                "save_id",
                "team",
                "club_name",
                "team_level",
                "is_primary"
            ]
        );
    }

    #[test]
    fn opening_fresh_db_applies_planner_tactic_schema() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("planner-tactic-migration-test.db"));

        let tactic_table_exists: bool = conn
            .query_row(
                "SELECT EXISTS(
                     SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'planner_tactics'
                 )",
                [],
                |row| row.get(0),
            )
            .expect("check removed tactic table");
        assert!(!tactic_table_exists);

        assert_eq!(
            table_columns(&conn, "planner_tactic_lanes"),
            [
                "id",
                "save_id",
                "lane_order",
                "lane_id",
                "ip_weight",
                "ip_position",
                "ip_role_id",
                "oop_position",
                "oop_role_id",
                "importance_rank"
            ]
        );
    }

    #[test]
    fn planner_lane_importance_ranks_are_unique_when_set() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("planner-rank-migration-test.db"));
        conn.execute("INSERT INTO saves (name) VALUES (?1)", ["Test save"])
            .expect("insert save");
        let save_id = conn.last_insert_rowid();
        let insert_lane = "INSERT INTO planner_tactic_lanes (
             save_id, lane_order, lane_id, ip_weight, ip_position, ip_role_id, oop_position, oop_role_id, importance_rank
         ) VALUES (?1, ?2, ?3, 0.5, 'GK', 'goalkeeper_ip', 'GK', 'line_holding_keeper_oop', ?4)";
        conn.execute(insert_lane, params![save_id, 0, "goalkeeper", 1])
            .expect("insert ranked lane");

        let error = conn
            .execute(insert_lane, params![save_id, 1, "left_back", 1])
            .expect_err("reject duplicate non-null importance rank");

        assert_eq!(
            error.sqlite_error_code(),
            Some(rusqlite::ErrorCode::ConstraintViolation)
        );
    }

    #[test]
    fn opening_fresh_db_applies_planner_depth_schema() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("planner-depth-migration-test.db"));

        for expected_table in ["planner_strings", "planner_assignments"] {
            let table_name: String = conn
                .query_row(
                    "SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [expected_table],
                    |row| row.get(0),
                )
                .expect("read depth table from sqlite_master");
            assert_eq!(table_name, expected_table);
        }

        assert_eq!(
            table_columns(&conn, "planner_strings"),
            ["id", "save_id", "team", "string_order"]
        );
        assert_eq!(
            table_columns(&conn, "planner_assignments"),
            [
                "id",
                "save_id",
                "string_id",
                "lane_id",
                "player_uid",
                "last_known_name",
                "provenance"
            ]
        );
    }
}
