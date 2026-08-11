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

pub const PLANNER_LANE_FOOT_PREFERENCES_SQL: &str = "
ALTER TABLE planner_tactic_lanes
    ADD COLUMN preferred_foot TEXT NOT NULL DEFAULT 'any'
    CHECK (preferred_foot IN ('any', 'left', 'right', 'both'));

ALTER TABLE planner_tactic_lanes
    ADD COLUMN foot_preference TEXT NOT NULL DEFAULT 'preferred'
    CHECK (foot_preference IN ('preferred', 'strict'));
";

pub const ACADEMY_SCHEMA_SQL: &str = "
CREATE TABLE academy_classes (
    id INTEGER PRIMARY KEY,
    save_id INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    class_year INTEGER NOT NULL CHECK (class_year > 0),
    UNIQUE (save_id, id),
    UNIQUE (save_id, class_year)
);

CREATE INDEX idx_academy_classes_save_year
    ON academy_classes(save_id, class_year DESC);

CREATE TABLE academy_memberships (
    save_id INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    class_id INTEGER NOT NULL,
    player_uid INTEGER NOT NULL,
    last_known_name TEXT NOT NULL CHECK (trim(last_known_name) <> ''),
    PRIMARY KEY (save_id, player_uid),
    FOREIGN KEY (save_id, class_id)
        REFERENCES academy_classes(save_id, id)
        ON DELETE CASCADE
);

CREATE INDEX idx_academy_memberships_class
    ON academy_memberships(save_id, class_id);
";

pub const ACADEMY_AUTOMATIC_CLASSES_SQL: &str = "
ALTER TABLE academy_classes
    ADD COLUMN is_automatic INTEGER NOT NULL DEFAULT 0
    CHECK (is_automatic IN (0, 1));

UPDATE academy_classes
SET is_automatic = 1
WHERE class_year = 2025;

INSERT OR IGNORE INTO academy_classes (save_id, class_year, is_automatic)
SELECT id, 2025, 1
FROM saves;

DROP INDEX idx_academy_classes_save_year;

CREATE INDEX idx_academy_classes_save_year
    ON academy_classes(save_id, class_year ASC);
";

pub const ACADEMY_MEMBER_OUTCOMES_SQL: &str = "
CREATE TABLE academy_member_outcomes (
    save_id INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    player_uid INTEGER NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('sold', 'released')),
    buying_club TEXT,
    sale_fee_eur INTEGER,
    PRIMARY KEY (save_id, player_uid),
    FOREIGN KEY (save_id, player_uid)
        REFERENCES academy_memberships(save_id, player_uid)
        ON DELETE CASCADE,
    CHECK (
        (status = 'sold'
            AND buying_club IS NOT NULL
            AND trim(buying_club) <> ''
            AND sale_fee_eur IS NOT NULL
            AND sale_fee_eur >= 0)
        OR (status = 'released'
            AND buying_club IS NULL
            AND sale_fee_eur IS NULL)
    )
);
";

pub const ACADEMY_CURRENT_SNAPSHOT_CLASSES_SQL: &str = "
INSERT OR IGNORE INTO academy_classes (save_id, class_year, is_automatic)
SELECT id, 2025, 1
FROM saves;

INSERT INTO academy_classes (save_id, class_year, is_automatic)
SELECT save_id, CAST(substr(game_date, 1, 4) AS INTEGER), 1
FROM snapshots
WHERE is_current = 1
  AND game_date_source IN ('memory', 'derived')
  AND game_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'
  AND date(game_date, '+0 days') = game_date
  AND CAST(substr(game_date, 1, 4) AS INTEGER) >= 2025
ON CONFLICT(save_id, class_year) DO UPDATE SET is_automatic = 1;
";

pub const SUPERSCOUT_PARITY_SCHEMA_SQL: &str = "
ALTER TABLE snapshots
    ADD COLUMN game_date_basis TEXT;

ALTER TABLE snapshots
    ADD COLUMN player_database_scope TEXT
    CHECK (player_database_scope IS NULL OR player_database_scope IN ('men', 'women', 'both'));

ALTER TABLE snapshots
    ADD COLUMN staff_count INTEGER NOT NULL DEFAULT 0
    CHECK (staff_count >= 0);

ALTER TABLE snapshots
    ADD COLUMN manager_uid INTEGER;

ALTER TABLE snapshots
    ADD COLUMN manager_name TEXT;

ALTER TABLE snapshots
    ADD COLUMN manager_club TEXT;

ALTER TABLE snapshots
    ADD COLUMN manager_club_reputation INTEGER;

ALTER TABLE players
    ADD COLUMN nation_uid INTEGER;

ALTER TABLE players
    ADD COLUMN gender TEXT NOT NULL DEFAULT 'unknown'
    CHECK (gender IN ('unknown', 'male', 'female'));

ALTER TABLE players
    ADD COLUMN club_reputation INTEGER;

ALTER TABLE players
    ADD COLUMN team_type INTEGER;

CREATE TABLE staff (
    snapshot_id INTEGER NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
    uid INTEGER NOT NULL,
    name TEXT,
    birth_year INTEGER,
    birth_day_of_year INTEGER,
    age INTEGER,
    nationalities_json TEXT NOT NULL,
    nation_uid INTEGER,
    gender TEXT NOT NULL CHECK (gender IN ('unknown', 'male', 'female')),
    ca INTEGER NOT NULL,
    pa INTEGER NOT NULL,
    staff_attributes_json TEXT NOT NULL,
    job_id INTEGER,
    weekly_wage_gbp INTEGER,
    contract_expiry_year INTEGER,
    contract_expiry_day_of_year INTEGER,
    club TEXT,
    division TEXT,
    PRIMARY KEY (snapshot_id, uid)
);
";

pub const SNAPSHOT_BRIDGE_SOURCE_REQUEST_SQL: &str = "
ALTER TABLE snapshots
    ADD COLUMN bridge_source_request_id TEXT;
";

pub const CSV_ENRICHMENT_SCHEMA_SQL: &str = "
CREATE TABLE player_youth_career_stats (
    save_id INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    player_uid INTEGER NOT NULL,
    career_appearances INTEGER CHECK (career_appearances IS NULL OR career_appearances >= 0),
    international_caps INTEGER CHECK (international_caps IS NULL OR international_caps >= 0),
    career_goals INTEGER CHECK (career_goals IS NULL OR career_goals >= 0),
    career_assists INTEGER CHECK (career_assists IS NULL OR career_assists >= 0),
    imported_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (save_id, player_uid)
);

CREATE TABLE player_moneyball_stats (
    save_id INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    player_uid INTEGER NOT NULL,
    asking_price_kind TEXT CHECK (
        asking_price_kind IS NULL
        OR asking_price_kind IN ('single', 'range', 'not_for_sale')
    ),
    asking_price_lower_eur INTEGER CHECK (
        asking_price_lower_eur IS NULL OR asking_price_lower_eur >= 0
    ),
    asking_price_upper_eur INTEGER CHECK (
        asking_price_upper_eur IS NULL OR asking_price_upper_eur >= 0
    ),
    starts INTEGER CHECK (starts IS NULL OR starts >= 0),
    substitute_appearances INTEGER CHECK (
        substitute_appearances IS NULL OR substitute_appearances >= 0
    ),
    minutes INTEGER CHECK (minutes IS NULL OR minutes >= 0),
    statistics_json TEXT NOT NULL CHECK (
        json_valid(statistics_json) = 1 AND json_type(statistics_json) = 'object'
    ),
    imported_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (save_id, player_uid),
    CHECK (
        CASE
            WHEN asking_price_kind IS NULL THEN
                asking_price_lower_eur IS NULL AND asking_price_upper_eur IS NULL
            WHEN asking_price_kind = 'single' THEN
                asking_price_lower_eur IS NOT NULL AND asking_price_upper_eur IS NULL
            WHEN asking_price_kind = 'range' THEN
                asking_price_lower_eur IS NOT NULL
                AND asking_price_upper_eur IS NOT NULL
                AND asking_price_lower_eur <= asking_price_upper_eur
            WHEN asking_price_kind = 'not_for_sale' THEN
                asking_price_lower_eur IS NULL AND asking_price_upper_eur IS NULL
            ELSE 0
        END
    )
);
";

pub const SNAPSHOT_MONEYBALL_ENRICHMENT_SCHEMA_SQL: &str = "
CREATE TABLE player_moneyball_stats_legacy (
    save_id INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    player_uid INTEGER NOT NULL,
    asking_price_kind TEXT CHECK (
        asking_price_kind IS NULL
        OR asking_price_kind IN ('single', 'range', 'not_for_sale')
    ),
    asking_price_lower_eur INTEGER CHECK (
        asking_price_lower_eur IS NULL OR asking_price_lower_eur >= 0
    ),
    asking_price_upper_eur INTEGER CHECK (
        asking_price_upper_eur IS NULL OR asking_price_upper_eur >= 0
    ),
    starts INTEGER CHECK (starts IS NULL OR starts >= 0),
    substitute_appearances INTEGER CHECK (
        substitute_appearances IS NULL OR substitute_appearances >= 0
    ),
    minutes INTEGER CHECK (minutes IS NULL OR minutes >= 0),
    statistics_json TEXT NOT NULL CHECK (
        json_valid(statistics_json) = 1 AND json_type(statistics_json) = 'object'
    ),
    imported_at_utc TEXT NOT NULL,
    PRIMARY KEY (save_id, player_uid),
    CHECK (
        CASE
            WHEN asking_price_kind IS NULL THEN
                asking_price_lower_eur IS NULL AND asking_price_upper_eur IS NULL
            WHEN asking_price_kind = 'single' THEN
                asking_price_lower_eur IS NOT NULL AND asking_price_upper_eur IS NULL
            WHEN asking_price_kind = 'range' THEN
                asking_price_lower_eur IS NOT NULL
                AND asking_price_upper_eur IS NOT NULL
                AND asking_price_lower_eur <= asking_price_upper_eur
            WHEN asking_price_kind = 'not_for_sale' THEN
                asking_price_lower_eur IS NULL AND asking_price_upper_eur IS NULL
            ELSE 0
        END
    )
);

INSERT INTO player_moneyball_stats_legacy (
    save_id,
    player_uid,
    asking_price_kind,
    asking_price_lower_eur,
    asking_price_upper_eur,
    starts,
    substitute_appearances,
    minutes,
    statistics_json,
    imported_at_utc
)
SELECT
    save_id,
    player_uid,
    asking_price_kind,
    asking_price_lower_eur,
    asking_price_upper_eur,
    starts,
    substitute_appearances,
    minutes,
    statistics_json,
    imported_at_utc
FROM player_moneyball_stats;

DROP TABLE player_moneyball_stats;

CREATE TABLE player_moneyball_stats (
    snapshot_id INTEGER NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
    player_uid INTEGER NOT NULL,
    asking_price_kind TEXT CHECK (
        asking_price_kind IS NULL
        OR asking_price_kind IN ('single', 'range', 'not_for_sale')
    ),
    asking_price_lower_eur INTEGER CHECK (
        asking_price_lower_eur IS NULL OR asking_price_lower_eur >= 0
    ),
    asking_price_upper_eur INTEGER CHECK (
        asking_price_upper_eur IS NULL OR asking_price_upper_eur >= 0
    ),
    starts INTEGER CHECK (starts IS NULL OR starts >= 0),
    substitute_appearances INTEGER CHECK (
        substitute_appearances IS NULL OR substitute_appearances >= 0
    ),
    minutes INTEGER CHECK (minutes IS NULL OR minutes >= 0),
    statistics_json TEXT NOT NULL CHECK (
        json_valid(statistics_json) = 1 AND json_type(statistics_json) = 'object'
    ),
    imported_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (snapshot_id, player_uid),
    FOREIGN KEY (snapshot_id, player_uid)
        REFERENCES players(snapshot_id, uid) ON DELETE CASCADE,
    CHECK (
        CASE
            WHEN asking_price_kind IS NULL THEN
                asking_price_lower_eur IS NULL AND asking_price_upper_eur IS NULL
            WHEN asking_price_kind = 'single' THEN
                asking_price_lower_eur IS NOT NULL AND asking_price_upper_eur IS NULL
            WHEN asking_price_kind = 'range' THEN
                asking_price_lower_eur IS NOT NULL
                AND asking_price_upper_eur IS NOT NULL
                AND asking_price_lower_eur <= asking_price_upper_eur
            WHEN asking_price_kind = 'not_for_sale' THEN
                asking_price_lower_eur IS NULL AND asking_price_upper_eur IS NULL
            ELSE 0
        END
    )
);
";

pub const SNAPSHOT_MANAGEMENT_CONTEXT_SQL: &str = "
ALTER TABLE saves
    ADD COLUMN context_token TEXT;

ALTER TABLE snapshots
    ADD COLUMN context_token TEXT;

ALTER TABLE snapshots
    ADD COLUMN custom_name TEXT;

UPDATE saves
SET context_token = lower(hex(randomblob(16)))
WHERE context_token IS NULL;

UPDATE snapshots
SET context_token = lower(hex(randomblob(16)))
WHERE context_token IS NULL;

CREATE UNIQUE INDEX idx_saves_context_token
    ON saves(context_token);

CREATE UNIQUE INDEX idx_snapshots_context_token
    ON snapshots(context_token);

CREATE TRIGGER populate_save_context_token
AFTER INSERT ON saves
WHEN NEW.context_token IS NULL
BEGIN
    UPDATE saves
    SET context_token = lower(hex(randomblob(16)))
    WHERE id = NEW.id;
END;

CREATE TRIGGER populate_snapshot_context_token
AFTER INSERT ON snapshots
WHEN NEW.context_token IS NULL
BEGIN
    UPDATE snapshots
    SET context_token = lower(hex(randomblob(16)))
    WHERE id = NEW.id;
END;

CREATE TRIGGER prevent_save_context_token_change
BEFORE UPDATE OF context_token ON saves
WHEN OLD.context_token IS NOT NULL
 AND OLD.context_token IS NOT NEW.context_token
BEGIN
    SELECT RAISE(ABORT, 'save context token is immutable');
END;

CREATE TRIGGER prevent_snapshot_context_token_change
BEFORE UPDATE OF context_token ON snapshots
WHEN OLD.context_token IS NOT NULL
 AND OLD.context_token IS NOT NEW.context_token
BEGIN
    SELECT RAISE(ABORT, 'snapshot context token is immutable');
END;
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
        Migration {
            version: 10,
            description: "add_planner_lane_foot_preferences",
            sql: PLANNER_LANE_FOOT_PREFERENCES_SQL,
        },
        Migration {
            version: 11,
            description: "create_academy_schema",
            sql: ACADEMY_SCHEMA_SQL,
        },
        Migration {
            version: 12,
            description: "add_automatic_academy_classes",
            sql: ACADEMY_AUTOMATIC_CLASSES_SQL,
        },
        Migration {
            version: 13,
            description: "create_academy_member_outcomes",
            sql: ACADEMY_MEMBER_OUTCOMES_SQL,
        },
        Migration {
            version: 14,
            description: "backfill_current_snapshot_academy_classes",
            sql: ACADEMY_CURRENT_SNAPSHOT_CLASSES_SQL,
        },
        Migration {
            version: 15,
            description: "add_superscout_parity_snapshot_data",
            sql: SUPERSCOUT_PARITY_SCHEMA_SQL,
        },
        Migration {
            version: 16,
            description: "add_snapshot_bridge_source_request_id",
            sql: SNAPSHOT_BRIDGE_SOURCE_REQUEST_SQL,
        },
        Migration {
            version: 17,
            description: "create_csv_enrichment_schema",
            sql: CSV_ENRICHMENT_SCHEMA_SQL,
        },
        Migration {
            version: 18,
            description: "move_moneyball_enrichment_to_snapshots",
            sql: SNAPSHOT_MONEYBALL_ENRICHMENT_SCHEMA_SQL,
        },
        Migration {
            version: 19,
            description: "add_snapshot_management_context",
            sql: SNAPSHOT_MANAGEMENT_CONTEXT_SQL,
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
    type LegacyMoneyballRow = (
        i64,
        i64,
        Option<String>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        String,
        String,
    );

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

    fn insert_player(conn: &Connection, snapshot_id: i64, uid: i64) {
        conn.execute(
            "INSERT INTO players (
                snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                nationalities_json, preferred_foot, positions_json, attributes_json,
                hidden_attributes_json, personality_json
             ) VALUES (?1, ?2, 100, 100, 'CSV player', 2000, 1, '[]', 'Right',
                '{}', '{}', '{}', '{}')",
            params![snapshot_id, uid],
        )
        .expect("insert player");
    }

    #[test]
    fn opening_fresh_db_applies_all_migrations_and_creates_demo_value() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("migration-test.db");
        let conn = open_migrated(&db_path);

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user_version");
        assert_eq!(version, 19);

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
    fn migrates_v18_rows_to_immutable_snapshot_management_contexts() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("snapshot-management-v18.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 18) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v18");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set migration version");
        }

        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Existing save', 1)",
            [],
        )
        .expect("insert existing save");
        let first_save_id = conn.last_insert_rowid();
        conn.execute("INSERT INTO saves (name) VALUES ('Other save')", [])
            .expect("insert second save");
        let second_save_id = conn.last_insert_rowid();
        conn.execute(
            INSERT_SNAPSHOT_SQL,
            params![first_save_id, true, false, Option::<i64>::None],
        )
        .expect("insert existing snapshot");
        let first_snapshot_id = conn.last_insert_rowid();
        conn.execute(
            INSERT_SNAPSHOT_SQL,
            params![second_save_id, false, false, Option::<i64>::None],
        )
        .expect("insert second snapshot");

        apply(&conn).expect("apply snapshot management migration");

        let save_tokens: Vec<String> = conn
            .prepare("SELECT context_token FROM saves ORDER BY id")
            .expect("prepare save token query")
            .query_map([], |row| row.get(0))
            .expect("query save tokens")
            .collect::<Result<_, _>>()
            .expect("read save tokens");
        let snapshot_tokens: Vec<String> = conn
            .prepare("SELECT context_token FROM snapshots ORDER BY id")
            .expect("prepare snapshot token query")
            .query_map([], |row| row.get(0))
            .expect("query snapshot tokens")
            .collect::<Result<_, _>>()
            .expect("read snapshot tokens");
        assert_eq!(save_tokens.len(), 2);
        assert_eq!(snapshot_tokens.len(), 2);
        assert!(save_tokens.iter().all(|token| token.len() == 32));
        assert!(snapshot_tokens.iter().all(|token| token.len() == 32));
        assert_ne!(save_tokens[0], save_tokens[1]);
        assert_ne!(snapshot_tokens[0], snapshot_tokens[1]);

        conn.execute("INSERT INTO saves (name) VALUES ('New save')", [])
            .expect("insert new save");
        let new_save_id = conn.last_insert_rowid();
        let new_save_token: String = conn
            .query_row(
                "SELECT context_token FROM saves WHERE id = ?1",
                [new_save_id],
                |row| row.get(0),
            )
            .expect("read generated save token");
        assert_eq!(new_save_token.len(), 32);

        conn.execute(
            INSERT_SNAPSHOT_SQL,
            params![new_save_id, false, false, Option::<i64>::None],
        )
        .expect("insert new snapshot");
        let new_snapshot_id = conn.last_insert_rowid();
        let new_snapshot_token: String = conn
            .query_row(
                "SELECT context_token FROM snapshots WHERE id = ?1",
                [new_snapshot_id],
                |row| row.get(0),
            )
            .expect("read generated snapshot token");
        assert_eq!(new_snapshot_token.len(), 32);

        let save_error = conn
            .execute(
                "UPDATE saves SET context_token = 'changed' WHERE id = ?1",
                [first_save_id],
            )
            .expect_err("reject changing save token");
        assert_eq!(
            save_error.sqlite_error_code(),
            Some(rusqlite::ErrorCode::ConstraintViolation)
        );
        let snapshot_error = conn
            .execute(
                "UPDATE snapshots SET context_token = 'changed' WHERE id = ?1",
                [first_snapshot_id],
            )
            .expect_err("reject changing snapshot token");
        assert_eq!(
            snapshot_error.sqlite_error_code(),
            Some(rusqlite::ErrorCode::ConstraintViolation)
        );
    }

    #[test]
    fn opening_fresh_db_creates_snapshot_scoped_moneyball_enrichment_schema() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("csv-enrichment-migration-test.db"));

        assert_eq!(
            table_columns(&conn, "player_youth_career_stats"),
            [
                "save_id",
                "player_uid",
                "career_appearances",
                "international_caps",
                "career_goals",
                "career_assists",
                "imported_at_utc",
            ]
        );
        assert_eq!(
            table_columns(&conn, "player_moneyball_stats_legacy"),
            [
                "save_id",
                "player_uid",
                "asking_price_kind",
                "asking_price_lower_eur",
                "asking_price_upper_eur",
                "starts",
                "substitute_appearances",
                "minutes",
                "statistics_json",
                "imported_at_utc",
            ]
        );
        assert_eq!(
            table_columns(&conn, "player_moneyball_stats"),
            [
                "snapshot_id",
                "player_uid",
                "asking_price_kind",
                "asking_price_lower_eur",
                "asking_price_upper_eur",
                "starts",
                "substitute_appearances",
                "minutes",
                "statistics_json",
                "imported_at_utc",
            ]
        );

        for table in ["player_youth_career_stats", "player_moneyball_stats_legacy"] {
            let (parent_table, on_delete): (String, String) = conn
                .query_row(
                    &format!("SELECT \"table\", on_delete FROM pragma_foreign_key_list('{table}')"),
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .expect("read enrichment foreign key");
            assert_eq!(parent_table, "saves");
            assert_eq!(on_delete, "CASCADE");
        }
        let moneyball_foreign_keys: Vec<(String, String)> = conn
            .prepare("SELECT \"table\", on_delete FROM pragma_foreign_key_list('player_moneyball_stats')")
            .expect("prepare Moneyball foreign-key query")
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("query Moneyball foreign keys")
            .collect::<Result<_, _>>()
            .expect("read Moneyball foreign keys");
        assert!(moneyball_foreign_keys.contains(&("snapshots".to_string(), "CASCADE".to_string())));
        assert!(moneyball_foreign_keys.contains(&("players".to_string(), "CASCADE".to_string())));

        conn.execute("INSERT INTO saves (name) VALUES ('CSV save')", [])
            .expect("insert save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            INSERT_SNAPSHOT_SQL,
            params![save_id, true, false, Option::<i64>::None],
        )
        .expect("insert snapshot");
        let snapshot_id = conn.last_insert_rowid();
        insert_player(&conn, snapshot_id, 77);

        conn.execute(
            "INSERT INTO player_youth_career_stats (save_id, player_uid, career_appearances)
             VALUES (?1, 77, 12)",
            [save_id],
        )
        .expect("insert youth career stats");
        let error = conn
            .execute(
                "INSERT INTO player_youth_career_stats (save_id, player_uid, career_appearances)
                 VALUES (?1, 77, 13)",
                [save_id],
            )
            .expect_err("reject duplicate youth player row");
        assert_eq!(
            error.sqlite_error_code(),
            Some(rusqlite::ErrorCode::ConstraintViolation)
        );
        conn.execute(
            "INSERT INTO player_moneyball_stats_legacy (
                save_id, player_uid, statistics_json, imported_at_utc
             ) VALUES (?1, 77, '{}', '2026-08-11T00:00:00.000Z')",
            [save_id],
        )
        .expect("insert legacy Moneyball stats");
        conn.execute(
            "INSERT INTO player_moneyball_stats (
                snapshot_id, player_uid, asking_price_kind, asking_price_lower_eur, statistics_json
             ) VALUES (?1, 77, 'single', 5000000, '{}')",
            [snapshot_id],
        )
        .expect("insert moneyball stats");
        let error = conn
            .execute(
                "INSERT INTO player_moneyball_stats (snapshot_id, player_uid, statistics_json)
                 VALUES (?1, 77, '{}')",
                [snapshot_id],
            )
            .expect_err("reject duplicate moneyball player row");
        assert_eq!(
            error.sqlite_error_code(),
            Some(rusqlite::ErrorCode::ConstraintViolation)
        );
        let error = conn
            .execute(
                "INSERT INTO player_moneyball_stats (snapshot_id, player_uid, statistics_json)
                 VALUES (?1, 88, '{}')",
                [snapshot_id],
            )
            .expect_err("reject Moneyball player outside the snapshot");
        assert_eq!(
            error.sqlite_error_code(),
            Some(rusqlite::ErrorCode::ConstraintViolation)
        );

        conn.execute("DELETE FROM snapshots WHERE id = ?1", [snapshot_id])
            .expect("delete snapshot");
        for table in ["player_youth_career_stats", "player_moneyball_stats_legacy"] {
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .expect("count enrichment after snapshot deletion");
            assert_eq!(count, 1, "{table} survives snapshot deletion");
        }
        let current_moneyball_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM player_moneyball_stats", [], |row| {
                row.get(0)
            })
            .expect("count current Moneyball rows after snapshot deletion");
        assert_eq!(current_moneyball_count, 0);

        conn.execute("DELETE FROM saves WHERE id = ?1", [save_id])
            .expect("delete save");
        for table in [
            "player_youth_career_stats",
            "player_moneyball_stats_legacy",
            "player_moneyball_stats",
        ] {
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .expect("count enrichment after save deletion");
            assert_eq!(count, 0, "{table} cascades with save deletion");
        }
    }

    #[test]
    fn csv_enrichment_schema_rejects_invalid_values_and_asking_price_shapes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("csv-enrichment-constraint-test.db"));
        conn.execute("INSERT INTO saves (name) VALUES ('CSV save')", [])
            .expect("insert save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            INSERT_SNAPSHOT_SQL,
            params![save_id, true, false, Option::<i64>::None],
        )
        .expect("insert snapshot");
        let snapshot_id = conn.last_insert_rowid();
        for player_uid in 1..=12 {
            insert_player(&conn, snapshot_id, player_uid);
        }

        for (player_uid, kind, lower, upper) in [
            (1, None, None, None),
            (2, Some("single"), Some(5_000_000), None),
            (3, Some("range"), Some(5_000_000), Some(8_000_000)),
            (4, Some("not_for_sale"), None, None),
        ] {
            conn.execute(
                "INSERT INTO player_moneyball_stats (
                    snapshot_id, player_uid, asking_price_kind, asking_price_lower_eur,
                    asking_price_upper_eur, statistics_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, '{}')",
                params![snapshot_id, player_uid, kind, lower, upper],
            )
            .expect("insert valid asking-price shape");
        }

        for (player_uid, kind, lower, upper) in [
            (5, Some("single"), None, None),
            (6, Some("range"), Some(8_000_000), Some(5_000_000)),
            (7, Some("not_for_sale"), Some(1), None),
            (8, None, Some(1), None),
        ] {
            let error = conn
                .execute(
                    "INSERT INTO player_moneyball_stats (
                        snapshot_id, player_uid, asking_price_kind, asking_price_lower_eur,
                        asking_price_upper_eur, statistics_json
                     ) VALUES (?1, ?2, ?3, ?4, ?5, '{}')",
                    params![snapshot_id, player_uid, kind, lower, upper],
                )
                .expect_err("reject invalid asking-price shape");
            assert_eq!(
                error.sqlite_error_code(),
                Some(rusqlite::ErrorCode::ConstraintViolation)
            );
        }

        for (statement, params) in [
            (
                "INSERT INTO player_youth_career_stats (save_id, player_uid, career_goals)
                 VALUES (?1, 9, -1)",
                params![save_id],
            ),
            (
                "INSERT INTO player_moneyball_stats (
                    snapshot_id, player_uid, starts, statistics_json
                 ) VALUES (?1, 10, -1, '{}')",
                params![snapshot_id],
            ),
            (
                "INSERT INTO player_moneyball_stats (snapshot_id, player_uid, statistics_json)
                 VALUES (?1, 11, '[]')",
                params![snapshot_id],
            ),
            (
                "INSERT INTO player_moneyball_stats (
                    snapshot_id, player_uid, asking_price_kind, asking_price_lower_eur, statistics_json
                 ) VALUES (?1, 12, 'single', -1, '{}')",
                params![snapshot_id],
            ),
        ] {
            let error = conn
                .execute(statement, params)
                .expect_err("reject invalid enrichment value");
            assert_eq!(
                error.sqlite_error_code(),
                Some(rusqlite::ErrorCode::ConstraintViolation)
            );
        }
    }

    #[test]
    fn migrates_populated_v16_database_to_csv_enrichment_schema_without_touching_existing_rows() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("csv-enrichment-v16-migration.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 16) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v16");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set migration version");
        }
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Existing save', 1)",
            [],
        )
        .expect("insert existing save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            INSERT_SNAPSHOT_SQL,
            params![save_id, true, false, Option::<i64>::None],
        )
        .expect("insert existing snapshot");
        let snapshot_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO planner_club_settings (save_id, primary_club) VALUES (?1, 'Existing FC')",
            [save_id],
        )
        .expect("insert existing planner state");

        apply(&conn).expect("apply CSV enrichment migration");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user version");
        assert_eq!(version, 19);
        let (save_name, is_current, primary_club): (String, i32, String) = conn
            .query_row(
                "SELECT saves.name, snapshots.is_current, planner_club_settings.primary_club
                 FROM saves
                 JOIN snapshots ON snapshots.save_id = saves.id
                 JOIN planner_club_settings ON planner_club_settings.save_id = saves.id
                 WHERE saves.id = ?1 AND snapshots.id = ?2",
                params![save_id, snapshot_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read preserved v16 rows");
        assert_eq!(save_name, "Existing save");
        assert_eq!(is_current, 1);
        assert_eq!(primary_club, "Existing FC");
        for table in [
            "player_youth_career_stats",
            "player_moneyball_stats_legacy",
            "player_moneyball_stats",
        ] {
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .expect("count new enrichment table");
            assert_eq!(count, 0);
        }
    }

    #[test]
    fn migrates_populated_v17_moneyball_rows_to_legacy_quarantine() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("moneyball-v17-migration.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 17) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v17");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set migration version");
        }
        conn.execute("INSERT INTO saves (name) VALUES ('Existing save')", [])
            .expect("insert save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            INSERT_SNAPSHOT_SQL,
            params![save_id, true, false, Option::<i64>::None],
        )
        .expect("insert current snapshot");
        let snapshot_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO players (
                snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                nationalities_json, preferred_foot, positions_json, attributes_json,
                hidden_attributes_json, personality_json
             ) VALUES (?1, 77, 100, 100, 'Current player', 2000, 1, '[]', 'Right',
                '{}', '{}', '{}', '{}')",
            [snapshot_id],
        )
        .expect("insert current player");
        for (player_uid, imported_at_utc) in [
            (77, "2026-08-10T12:34:56.789Z"),
            (88, "2026-08-10T12:35:56.789Z"),
        ] {
            conn.execute(
                "INSERT INTO player_moneyball_stats (
                    save_id, player_uid, asking_price_kind, asking_price_lower_eur,
                    starts, substitute_appearances, minutes, statistics_json, imported_at_utc
                 ) VALUES (?1, ?2, 'single', 5000000, 10, 2, 900, '{\"goals\":3}', ?3)",
                params![save_id, player_uid, imported_at_utc],
            )
            .expect("insert v17 Moneyball row");
        }

        apply(&conn).expect("migrate populated v17 Moneyball rows");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user version");
        assert_eq!(version, 19);
        let rows: Vec<LegacyMoneyballRow> = conn
            .prepare(
                "SELECT save_id, player_uid, asking_price_kind, asking_price_lower_eur,
                        asking_price_upper_eur, starts, substitute_appearances, minutes,
                        statistics_json, imported_at_utc
                 FROM player_moneyball_stats_legacy
                 ORDER BY player_uid",
            )
            .expect("prepare legacy rows")
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                ))
            })
            .expect("query legacy rows")
            .collect::<Result<_, _>>()
            .expect("read legacy rows");
        assert_eq!(
            rows,
            vec![
                (
                    save_id,
                    77,
                    Some("single".to_string()),
                    Some(5_000_000),
                    None,
                    Some(10),
                    Some(2),
                    Some(900),
                    "{\"goals\":3}".to_string(),
                    "2026-08-10T12:34:56.789Z".to_string(),
                ),
                (
                    save_id,
                    88,
                    Some("single".to_string()),
                    Some(5_000_000),
                    None,
                    Some(10),
                    Some(2),
                    Some(900),
                    "{\"goals\":3}".to_string(),
                    "2026-08-10T12:35:56.789Z".to_string(),
                ),
            ]
        );
        let current_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM player_moneyball_stats", [], |row| {
                row.get(0)
            })
            .expect("count current-format Moneyball rows");
        assert_eq!(current_count, 0);

        apply(&conn).expect("reapply migrations");
        let legacy_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM player_moneyball_stats_legacy",
                [],
                |row| row.get(0),
            )
            .expect("count legacy rows after reapply");
        assert_eq!(legacy_count, 2);
    }

    #[test]
    fn rolls_back_moneyball_migration_when_legacy_backfill_fails() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("moneyball-v17-rollback.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 17) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v17");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set migration version");
        }
        conn.pragma_update(None, "foreign_keys", false)
            .expect("disable foreign keys for corrupt legacy fixture");
        conn.execute(
            "INSERT INTO player_moneyball_stats (save_id, player_uid, statistics_json)
             VALUES (999, 77, '{}')",
            [],
        )
        .expect("insert row that forces the legacy copy to fail");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("reenable foreign keys before migration");

        apply(&conn).expect_err("reject invalid legacy backfill");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user version");
        assert_eq!(version, 17);
        let original_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM player_moneyball_stats", [], |row| {
                row.get(0)
            })
            .expect("count retained source rows");
        assert_eq!(original_count, 1);
        let legacy_table_exists: bool = conn
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM sqlite_master
                    WHERE type = 'table' AND name = 'player_moneyball_stats_legacy'
                 )",
                [],
                |row| row.get(0),
            )
            .expect("check rolled-back legacy table");
        assert!(!legacy_table_exists);
    }

    #[test]
    fn opening_fresh_db_applies_academy_schema() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("academy-migration-test.db"));

        let table_name: String = conn
            .query_row(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'academy_classes'",
                [],
                |row| row.get(0),
            )
            .expect("read academy_classes from sqlite_master");

        assert_eq!(table_name, "academy_classes");
        assert_eq!(
            table_columns(&conn, "academy_classes"),
            ["id", "save_id", "class_year", "is_automatic"]
        );
        assert_eq!(
            table_columns(&conn, "academy_memberships"),
            ["save_id", "class_id", "player_uid", "last_known_name"]
        );
        assert_eq!(
            table_columns(&conn, "academy_member_outcomes"),
            [
                "save_id",
                "player_uid",
                "status",
                "buying_club",
                "sale_fee_eur"
            ]
        );
    }

    #[test]
    fn migrates_populated_v10_without_touching_planner_rows() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("academy-v10-migration-test.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 10) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v10");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set migration version");
        }
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Existing save', 1)",
            [],
        )
        .expect("insert save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO planner_club_settings (save_id, primary_club) VALUES (?1, 'Existing FC')",
            [save_id],
        )
        .expect("insert planner setting");

        apply(&conn).expect("migrate populated v10 database");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user version");
        assert_eq!(version, 19);
        let primary_club: String = conn
            .query_row(
                "SELECT primary_club FROM planner_club_settings WHERE save_id = ?1",
                [save_id],
                |row| row.get(0),
            )
            .expect("read preserved planner setting");
        assert_eq!(primary_club, "Existing FC");
    }

    #[test]
    fn migrates_v11_classes_to_automatic_baselines_without_overwriting_memberships() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("academy-v11-migration-test.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 11) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v11");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set migration version");
        }
        conn.execute("INSERT INTO saves (name) VALUES ('Existing save')", [])
            .expect("insert first save");
        let first_save_id = conn.last_insert_rowid();
        conn.execute("INSERT INTO saves (name) VALUES ('Second save')", [])
            .expect("insert second save");
        let second_save_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO academy_classes (save_id, class_year) VALUES (?1, 2025)",
            [first_save_id],
        )
        .expect("insert existing class");
        let existing_class_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO academy_memberships (save_id, class_id, player_uid, last_known_name)
             VALUES (?1, ?2, 77, 'Existing graduate')",
            params![first_save_id, existing_class_id],
        )
        .expect("insert existing membership");

        apply(&conn).expect("migrate populated v11 database");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user version");
        assert_eq!(version, 19);
        assert_eq!(
            table_columns(&conn, "academy_classes"),
            ["id", "save_id", "class_year", "is_automatic"]
        );
        let (class_id, automatic, member_count): (i64, i32, i64) = conn
            .query_row(
                "SELECT class.id, class.is_automatic, COUNT(member.player_uid)
                 FROM academy_classes class
                 LEFT JOIN academy_memberships member
                   ON member.save_id = class.save_id
                  AND member.class_id = class.id
                 WHERE class.save_id = ?1 AND class.class_year = 2025
                 GROUP BY class.id, class.is_automatic",
                [first_save_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read promoted existing class");
        assert_eq!(class_id, existing_class_id);
        assert_eq!(automatic, 1);
        assert_eq!(member_count, 1);
        let second_baseline_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM academy_classes WHERE save_id = ?1 AND class_year = 2025",
                [second_save_id],
                |row| row.get(0),
            )
            .expect("count second-save baseline");
        assert_eq!(second_baseline_count, 1);
    }

    #[test]
    fn migrates_v13_current_snapshot_to_its_observed_automatic_class() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("academy-v13-migration-test.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 13) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v13");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set migration version");
        }
        conn.execute("INSERT INTO saves (name) VALUES ('Existing save')", [])
            .expect("insert save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO snapshots (
                save_id, is_current, schema_version, generated_at_utc, game_version,
                supported_game_version, bridge_version, protocol_version, game_date,
                game_date_source, scan_truncated, max_accepted, player_count
             ) VALUES (
                ?1, 1, 5, '2030-08-14T12:00:00Z', '26.3.2', '26.3', '0.1.0', 1,
                '2030-08-14', 'memory', 0, NULL, 0
             )",
            [save_id],
        )
        .expect("insert current snapshot");
        conn.execute(
            "INSERT INTO saves (name) VALUES ('Malformed date save')",
            [],
        )
        .expect("insert malformed-date save");
        let malformed_date_save_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO snapshots (
                save_id, is_current, schema_version, generated_at_utc, game_version,
                supported_game_version, bridge_version, protocol_version, game_date,
                game_date_source, scan_truncated, max_accepted, player_count
             ) VALUES (
                ?1, 1, 5, '2030-02-30T12:00:00Z', '26.3.2', '26.3', '0.1.0', 1,
                '2030-02-30', 'memory', 0, NULL, 0
             )",
            [malformed_date_save_id],
        )
        .expect("insert malformed current snapshot");

        apply(&conn).expect("backfill current snapshot class");

        let (game_date_basis, player_database_scope, staff_count, manager_uid): (
            Option<String>,
            Option<String>,
            i64,
            Option<i64>,
        ) = conn
            .query_row(
                "SELECT game_date_basis, player_database_scope, staff_count, manager_uid
                 FROM snapshots WHERE save_id = ?1",
                [save_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("read v15 migration fields");
        assert_eq!(game_date_basis, None);
        assert_eq!(player_database_scope, None);
        assert_eq!(staff_count, 0);
        assert_eq!(manager_uid, None);

        let class_years = conn
            .prepare(
                "SELECT class_year
                 FROM academy_classes
                 WHERE save_id = ?1
                 ORDER BY class_year",
            )
            .expect("prepare class query")
            .query_map([save_id], |row| row.get::<_, i64>(0))
            .expect("query automatic classes")
            .collect::<Result<Vec<_>, _>>()
            .expect("read automatic classes");
        assert_eq!(class_years, vec![2025, 2030]);
        let malformed_date_class_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM academy_classes WHERE save_id = ?1 AND class_year = 2030",
                [malformed_date_save_id],
                |row| row.get(0),
            )
            .expect("count malformed-date classes");
        assert_eq!(malformed_date_class_count, 0);
    }

    #[test]
    fn academy_membership_requires_a_class_from_its_save() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("academy-foreign-key-test.db"));
        conn.execute("INSERT INTO saves (name) VALUES ('First save')", [])
            .expect("insert first save");
        let first_save_id = conn.last_insert_rowid();
        conn.execute("INSERT INTO saves (name) VALUES ('Second save')", [])
            .expect("insert second save");
        let second_save_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO academy_classes (save_id, class_year) VALUES (?1, 2030)",
            [first_save_id],
        )
        .expect("insert first class");
        let first_class_id = conn.last_insert_rowid();

        let error = conn
            .execute(
                "INSERT INTO academy_memberships (save_id, class_id, player_uid, last_known_name)
                 VALUES (?1, ?2, 77, 'Wrong Save')",
                params![second_save_id, first_class_id],
            )
            .expect_err("reject cross-save class reference");

        assert_eq!(
            error.sqlite_error_code(),
            Some(rusqlite::ErrorCode::ConstraintViolation)
        );
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
        assert_eq!(version, 19);
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
    fn migrates_v8_tactic_lanes_with_default_rank_and_foot_preferences() {
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

        let (importance_rank, preferred_foot, foot_preference): (Option<i64>, String, String) =
            conn.query_row(
                "SELECT importance_rank, preferred_foot, foot_preference
                 FROM planner_tactic_lanes
                 WHERE save_id = ?1",
                [save_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read migrated lane preferences");
        assert_eq!(importance_rank, None);
        assert_eq!(preferred_foot, "any");
        assert_eq!(foot_preference, "preferred");
    }

    #[test]
    fn opening_fresh_db_applies_snapshot_schema_tables() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("snapshot-migration-test.db");
        let conn = open_migrated(&db_path);

        for expected_table in ["saves", "snapshots", "players", "staff"] {
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
        assert_eq!(version, 19);

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
                "updated_at_utc",
                "context_token",
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
                "game_date_basis",
                "player_database_scope",
                "staff_count",
                "manager_uid",
                "manager_name",
                "manager_club",
                "manager_club_reputation",
                "bridge_source_request_id",
                "context_token",
                "custom_name",
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
                "nation_uid",
                "gender",
                "club_reputation",
                "team_type",
            ]
        );
        assert_eq!(
            table_columns(&conn, "staff"),
            [
                "snapshot_id",
                "uid",
                "name",
                "birth_year",
                "birth_day_of_year",
                "age",
                "nationalities_json",
                "nation_uid",
                "gender",
                "ca",
                "pa",
                "staff_attributes_json",
                "job_id",
                "weekly_wage_gbp",
                "contract_expiry_year",
                "contract_expiry_day_of_year",
                "club",
                "division",
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
                "idx_academy_classes_save_year",
                "idx_academy_memberships_class",
                "idx_planner_assignments_string",
                "idx_planner_club_sources_save_team",
                "idx_planner_strings_save_team_order",
                "idx_planner_tactic_lanes_save_importance_rank",
                "idx_planner_tactic_lanes_save_order",
                "idx_player_role_scores_snapshot_role",
                "idx_players_snapshot_ca",
                "idx_players_snapshot_name",
                "idx_saves_context_token",
                "idx_saves_one_active",
                "idx_snapshots_context_token",
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
        assert_eq!(version, 19);
    }

    #[test]
    fn migrates_v15_snapshot_with_null_bridge_source_request_id() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("v15-provenance-migration.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 15) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v15");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set migration version");
        }
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Existing save', 1)",
            [],
        )
        .expect("insert save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            INSERT_SNAPSHOT_SQL,
            params![save_id, true, false, Option::<i64>::None],
        )
        .expect("insert existing snapshot");
        let snapshot_id = conn.last_insert_rowid();

        apply(&conn).expect("apply bridge provenance migration");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user version");
        assert_eq!(version, 19);
        let (source_request_id, is_current): (Option<String>, i32) = conn
            .query_row(
                "SELECT bridge_source_request_id, is_current FROM snapshots WHERE id = ?1",
                [snapshot_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read migrated snapshot");
        assert_eq!(source_request_id, None);
        assert_eq!(is_current, 1);
    }

    #[test]
    fn migrates_snapshot_schema_from_every_prior_version() {
        let temp_dir = tempfile::tempdir().expect("temp dir");

        for legacy_version in 1..19 {
            let conn = Connection::open(
                temp_dir
                    .path()
                    .join(format!("snapshot-v{legacy_version}-migration-test.db")),
            )
            .expect("open test db");
            conn.pragma_update(None, "foreign_keys", true)
                .expect("enable foreign keys");
            for migration in all()
                .iter()
                .filter(|migration| migration.version <= legacy_version)
            {
                conn.execute_batch(migration.sql)
                    .expect("apply legacy migration");
                conn.pragma_update(None, "user_version", migration.version)
                    .expect("set legacy user version");
            }

            apply(&conn).expect("apply snapshot migration");

            let version: i32 = conn
                .pragma_query_value(None, "user_version", |row| row.get(0))
                .expect("read user version");
            assert_eq!(version, 19, "legacy version {legacy_version}");
            assert_eq!(
                table_columns(&conn, "staff").first().map(String::as_str),
                Some("snapshot_id"),
                "legacy version {legacy_version}"
            );
        }
    }

    #[test]
    fn registers_monotonic_migrations() {
        let migrations = all();

        assert_eq!(migrations.len(), 19);
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
        assert_eq!(migrations[9].version, 10);
        assert_eq!(
            migrations[9].description,
            "add_planner_lane_foot_preferences"
        );
        assert_eq!(migrations[9].sql, PLANNER_LANE_FOOT_PREFERENCES_SQL);
        assert_eq!(migrations[10].version, 11);
        assert_eq!(migrations[10].description, "create_academy_schema");
        assert_eq!(migrations[10].sql, ACADEMY_SCHEMA_SQL);
        assert_eq!(migrations[11].version, 12);
        assert_eq!(migrations[11].description, "add_automatic_academy_classes");
        assert_eq!(migrations[11].sql, ACADEMY_AUTOMATIC_CLASSES_SQL);
        assert_eq!(migrations[12].version, 13);
        assert_eq!(migrations[13].version, 14);
        assert_eq!(migrations[12].description, "create_academy_member_outcomes");
        assert_eq!(migrations[12].sql, ACADEMY_MEMBER_OUTCOMES_SQL);
        assert_eq!(migrations[14].version, 15);
        assert_eq!(
            migrations[14].description,
            "add_superscout_parity_snapshot_data"
        );
        assert_eq!(migrations[14].sql, SUPERSCOUT_PARITY_SCHEMA_SQL);
        assert_eq!(migrations[15].version, 16);
        assert_eq!(
            migrations[15].description,
            "add_snapshot_bridge_source_request_id"
        );
        assert_eq!(migrations[15].sql, SNAPSHOT_BRIDGE_SOURCE_REQUEST_SQL);
        assert_eq!(migrations[16].version, 17);
        assert_eq!(migrations[16].description, "create_csv_enrichment_schema");
        assert_eq!(migrations[16].sql, CSV_ENRICHMENT_SCHEMA_SQL);
        assert_eq!(migrations[17].version, 18);
        assert_eq!(
            migrations[17].description,
            "move_moneyball_enrichment_to_snapshots"
        );
        assert_eq!(migrations[17].sql, SNAPSHOT_MONEYBALL_ENRICHMENT_SCHEMA_SQL);
        assert_eq!(migrations[18].version, 19);
        assert_eq!(
            migrations[18].description,
            "add_snapshot_management_context"
        );
        assert_eq!(migrations[18].sql, SNAPSHOT_MANAGEMENT_CONTEXT_SQL);
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
                "importance_rank",
                "preferred_foot",
                "foot_preference"
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
