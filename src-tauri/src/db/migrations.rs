use rusqlite::{Connection, Transaction};

use crate::features::player_metrics::potential_scores;

type MigrationHook = for<'a> fn(&Transaction<'a>) -> Result<(), String>;

pub struct Migration {
    pub version: i32,
    pub description: &'static str,
    pub sql: &'static str,
}

impl Migration {
    fn hook(&self) -> Option<MigrationHook> {
        match self.version {
            34 => Some(potential_scores::backfill_current_snapshots),
            _ => None,
        }
    }
}

pub const INITIAL_DEMO_VALUE_SQL: &str = "
CREATE TABLE IF NOT EXISTS demo_value (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    value TEXT NOT NULL DEFAULT ''
);
";

pub const DROP_DEMO_VALUE_SQL: &str = "
DROP TABLE demo_value;
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

pub const PLAYER_POTENTIAL_ROLE_SCORES_SQL: &str = "
CREATE TABLE player_potential_role_scores (
    snapshot_id INTEGER NOT NULL,
    uid INTEGER NOT NULL,
    role_id TEXT NOT NULL,
    score INTEGER CHECK (score IS NULL OR (score >= 0 AND score <= 100)),
    projection_model_version INTEGER NOT NULL CHECK (projection_model_version > 0),
    PRIMARY KEY (snapshot_id, uid, role_id),
    FOREIGN KEY (snapshot_id, uid) REFERENCES players(snapshot_id, uid) ON DELETE CASCADE
);

CREATE INDEX idx_player_potential_role_scores_snapshot_role_score
    ON player_potential_role_scores(snapshot_id, role_id, projection_model_version, score);
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

pub const PLAYER_BOOST_RECOVERY_SQL: &str = "
ALTER TABLE snapshots
    ADD COLUMN player_boost_recovery_required INTEGER NOT NULL DEFAULT 0
    CHECK (player_boost_recovery_required IN (0, 1));
";

pub const PLAYER_INFORMATION_VISIBILITY_SQL: &str = "
ALTER TABLE saves
    ADD COLUMN reveal_hidden_player_information INTEGER NOT NULL DEFAULT 1
    CHECK (reveal_hidden_player_information IN (0, 1));
";

pub const STAFF_ROLE_SCORES_SQL: &str = "
CREATE TABLE staff_role_scores (
    snapshot_id INTEGER NOT NULL,
    uid INTEGER NOT NULL,
    role_id TEXT NOT NULL,
    score INTEGER CHECK (score IS NULL OR (score >= 0 AND score <= 100)),
    PRIMARY KEY (snapshot_id, uid, role_id),
    FOREIGN KEY (snapshot_id, uid) REFERENCES staff(snapshot_id, uid) ON DELETE CASCADE
);

CREATE INDEX idx_staff_role_scores_snapshot_role
    ON staff_role_scores(snapshot_id, role_id);
";

pub const SHARED_BOOST_RECOVERY_SQL: &str = "
ALTER TABLE snapshots
    RENAME COLUMN player_boost_recovery_required TO boost_recovery_required;
";

pub const SHARED_INFORMATION_VISIBILITY_SQL: &str = "
ALTER TABLE saves
    RENAME COLUMN reveal_hidden_player_information TO reveal_hidden_information;
";

pub const STAFF_SHORTLIST_SCHEMA_SQL: &str = "
CREATE TABLE staff_shortlist_entries (
    save_id INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    staff_uid INTEGER NOT NULL,
    preferred_job TEXT NOT NULL CHECK (trim(preferred_job) <> ''),
    club_job TEXT NOT NULL,
    coaching_qualifications TEXT NOT NULL,
    PRIMARY KEY (save_id, staff_uid)
);

CREATE INDEX idx_staff_shortlist_entries_save_preferred_job
    ON staff_shortlist_entries(save_id, preferred_job COLLATE NOCASE);
";

pub const PLANNER_TEAM_SETTINGS_SQL: &str = "
CREATE TABLE planner_teams (
    save_id INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    team TEXT NOT NULL CHECK (team IN ('senior', 'reserves', 'youth')),
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) BETWEEN 1 AND 40),
    PRIMARY KEY (save_id, team),
    UNIQUE (save_id, display_name COLLATE NOCASE)
);

CREATE INDEX idx_planner_teams_save_team
    ON planner_teams(save_id, team);

INSERT INTO planner_teams (save_id, team, display_name)
SELECT id, 'senior', 'Senior' FROM saves;

INSERT INTO planner_teams (save_id, team, display_name)
SELECT id, 'reserves', 'Reserves' FROM saves;

INSERT INTO planner_teams (save_id, team, display_name)
SELECT id, 'youth', 'Youth' FROM saves;
";

pub const MANAGED_CLUB_SETTINGS_SQL: &str = "
DROP TABLE planner_club_sources;

ALTER TABLE planner_club_settings
    RENAME TO managed_club_settings;

ALTER TABLE managed_club_settings
    RENAME COLUMN primary_club TO club_name;
";

pub const MONEYBALL_PERCENTILE_COHORT_SQL: &str = "
ALTER TABLE player_moneyball_stats
    ADD COLUMN percentiles_json TEXT CHECK (
        percentiles_json IS NULL
        OR (json_valid(percentiles_json) = 1 AND json_type(percentiles_json) = 'object')
    );
";

pub const CLUB_DNA_DEFINITIONS_SQL: &str = "
CREATE TABLE club_dna_definitions (
    save_id INTEGER PRIMARY KEY REFERENCES saves(id) ON DELETE CASCADE,
    attribute_ids_json TEXT NOT NULL CHECK (
        json_valid(attribute_ids_json) = 1
        AND json_type(attribute_ids_json) = 'array'
        AND json_array_length(attribute_ids_json) > 0
    )
);
";

pub const CLUB_DNA_SCORE_CACHE_SQL: &str = "
ALTER TABLE club_dna_definitions
    ADD COLUMN definition_version INTEGER NOT NULL DEFAULT 1 CHECK (definition_version > 0);

CREATE TABLE club_dna_scores (
    snapshot_id INTEGER NOT NULL,
    uid INTEGER NOT NULL,
    definition_version INTEGER NOT NULL CHECK (definition_version > 0),
    score_model_version INTEGER NOT NULL CHECK (score_model_version > 0),
    score INTEGER CHECK (score IS NULL OR score BETWEEN 0 AND 100),
    PRIMARY KEY (snapshot_id, uid, definition_version, score_model_version),
    FOREIGN KEY (snapshot_id, uid) REFERENCES players(snapshot_id, uid) ON DELETE CASCADE
);

CREATE INDEX idx_club_dna_scores_snapshot_definition_model_score
    ON club_dna_scores(snapshot_id, definition_version, score_model_version, score);
";

pub const POTENTIAL_SCORES_V34_SQL: &str = "
ALTER TABLE players
    ADD COLUMN potential_attributes_json TEXT;

ALTER TABLE players
    ADD COLUMN potential_projection_model_version INTEGER
    CHECK (potential_projection_model_version IS NULL OR potential_projection_model_version > 0);
";

pub const PLAYER_TARGETED_SORT_INDEXES_SQL: &str = "
CREATE INDEX idx_players_snapshot_pa_asc_uid
    ON players(snapshot_id, pa ASC, uid ASC);

CREATE INDEX idx_players_snapshot_pa_desc_uid
    ON players(snapshot_id, pa DESC, uid ASC);

CREATE INDEX idx_players_snapshot_age_asc_uid
    ON players(snapshot_id, age ASC, uid ASC);

CREATE INDEX idx_players_snapshot_age_desc_uid
    ON players(snapshot_id, age DESC, uid ASC);

CREATE INDEX idx_players_snapshot_value_asc_uid
    ON players(snapshot_id, market_value_gbp ASC, uid ASC);

CREATE INDEX idx_players_snapshot_value_desc_uid
    ON players(snapshot_id, market_value_gbp DESC, uid ASC);

CREATE INDEX idx_players_snapshot_current_club_uid
    ON players(snapshot_id, current_club, uid);
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
        Migration {
            version: 20,
            description: "add_player_boost_recovery_requirement",
            sql: PLAYER_BOOST_RECOVERY_SQL,
        },
        Migration {
            version: 21,
            description: "create_player_potential_role_scores",
            sql: PLAYER_POTENTIAL_ROLE_SCORES_SQL,
        },
        Migration {
            version: 22,
            description: "drop_demo_value_table",
            sql: DROP_DEMO_VALUE_SQL,
        },
        Migration {
            version: 23,
            description: "add_player_information_visibility",
            sql: PLAYER_INFORMATION_VISIBILITY_SQL,
        },
        Migration {
            version: 24,
            description: "create_staff_role_scores",
            sql: STAFF_ROLE_SCORES_SQL,
        },
        Migration {
            version: 25,
            description: "share_boost_recovery_requirement",
            sql: SHARED_BOOST_RECOVERY_SQL,
        },
        Migration {
            version: 26,
            description: "share_hidden_information_visibility",
            sql: SHARED_INFORMATION_VISIBILITY_SQL,
        },
        Migration {
            version: 27,
            description: "create_staff_shortlist_entries",
            sql: STAFF_SHORTLIST_SCHEMA_SQL,
        },
        Migration {
            version: 28,
            description: "create_planner_team_settings",
            sql: PLANNER_TEAM_SETTINGS_SQL,
        },
        Migration {
            version: 29,
            description: "replace_club_family_with_managed_club",
            sql: MANAGED_CLUB_SETTINGS_SQL,
        },
        Migration {
            version: 30,
            description: "add_moneyball_percentile_cohorts",
            sql: MONEYBALL_PERCENTILE_COHORT_SQL,
        },
        Migration {
            version: 31,
            description: "create_club_dna_definitions",
            sql: CLUB_DNA_DEFINITIONS_SQL,
        },
        Migration {
            version: 32,
            description: "create_club_dna_score_cache",
            sql: CLUB_DNA_SCORE_CACHE_SQL,
        },
        Migration {
            version: 33,
            description: "index_targeted_player_sorts",
            sql: PLAYER_TARGETED_SORT_INDEXES_SQL,
        },
        Migration {
            version: 34,
            description: "persist_current_potential_scores",
            sql: POTENTIAL_SCORES_V34_SQL,
        },
    ]
}

pub fn latest_version() -> i32 {
    all()
        .last()
        .expect("migration registry must not be empty")
        .version
}

/// Apply pending migrations using `PRAGMA user_version`.
pub fn apply(conn: &Connection) -> Result<(), String> {
    let current: i32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|error| format!("read database schema version: {error}"))?;

    log::info!(
        "database schema migration state: current={current}, target={}",
        latest_version()
    );

    for migration in all() {
        if migration.version <= current {
            continue;
        }

        log::info!(
            "applying migration {}: {}",
            migration.version,
            migration.description
        );

        let result = (|| -> Result<(), String> {
            let tx = conn
                .unchecked_transaction()
                .map_err(|error| error.to_string())?;
            tx.execute_batch(migration.sql)
                .map_err(|error| error.to_string())?;
            if let Some(hook) = migration.hook() {
                hook(&tx)?;
            }
            tx.pragma_update(None, "user_version", migration.version)
                .map_err(|error| error.to_string())?;
            tx.commit().map_err(|error| error.to_string())
        })();
        if let Err(error) = result {
            let error = format!(
                "migration {} ({}) failed: {error}",
                migration.version, migration.description
            );
            log::error!("database {error}");
            return Err(error);
        }
    }

    log::info!("database schema ready: version={}", latest_version());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, path::Path};

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

    fn complete_attributes_json() -> String {
        let attributes = crate::features::scoring::catalog::DUMP_ATTRIBUTE_KEYS
            .iter()
            .map(|key| ((*key).to_string(), Some(10_u8)))
            .collect::<HashMap<_, _>>();
        serde_json::to_string(&attributes).expect("serialize complete attributes")
    }

    fn insert_player(conn: &Connection, snapshot_id: i64, uid: i64) {
        conn.execute(
            "INSERT INTO players (
                snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                nationalities_json, preferred_foot, positions_json, attributes_json,
                hidden_attributes_json, personality_json
             ) VALUES (?1, ?2, 100, 100, 'CSV player', 2000, 1, '[]', 'Right',
                '{}', ?3, '{}', '{}')",
            params![snapshot_id, uid, complete_attributes_json()],
        )
        .expect("insert player");
    }

    fn query_json_rows(conn: &Connection, query: &str) -> Vec<String> {
        conn.prepare(query)
            .expect("prepare preservation query")
            .query_map([], |row| row.get(0))
            .expect("query preserved rows")
            .collect::<Result<_, _>>()
            .expect("read preserved rows")
    }

    fn player_sort_index_inventory(conn: &Connection) -> Vec<(String, Vec<(String, i64)>)> {
        let index_names = conn
            .prepare(
                "SELECT name
                 FROM sqlite_master
                 WHERE type = 'index' AND name LIKE 'idx_players_snapshot_%'
                 ORDER BY name",
            )
            .expect("prepare player index inventory")
            .query_map([], |row| row.get::<_, String>(0))
            .expect("query player index inventory")
            .collect::<Result<Vec<_>, _>>()
            .expect("read player index inventory");

        index_names
            .into_iter()
            .map(|index_name| {
                let columns = conn
                    .prepare(
                        "SELECT name, \"desc\"
                         FROM pragma_index_xinfo(?1)
                         WHERE \"key\" = 1
                         ORDER BY seqno",
                    )
                    .expect("prepare player index columns")
                    .query_map([&index_name], |row| Ok((row.get(0)?, row.get(1)?)))
                    .expect("query player index columns")
                    .collect::<Result<Vec<_>, _>>()
                    .expect("read player index columns");
                (index_name, columns)
            })
            .collect()
    }

    fn assert_player_sort_index_inventory(conn: &Connection) {
        assert_eq!(
            player_sort_index_inventory(conn),
            [
                (
                    "idx_players_snapshot_age_asc_uid".to_string(),
                    vec![
                        ("snapshot_id".to_string(), 0),
                        ("age".to_string(), 0),
                        ("uid".to_string(), 0),
                    ],
                ),
                (
                    "idx_players_snapshot_age_desc_uid".to_string(),
                    vec![
                        ("snapshot_id".to_string(), 0),
                        ("age".to_string(), 1),
                        ("uid".to_string(), 0),
                    ],
                ),
                (
                    "idx_players_snapshot_ca".to_string(),
                    vec![("snapshot_id".to_string(), 0), ("ca".to_string(), 1)],
                ),
                (
                    "idx_players_snapshot_current_club_uid".to_string(),
                    vec![
                        ("snapshot_id".to_string(), 0),
                        ("current_club".to_string(), 0),
                        ("uid".to_string(), 0),
                    ],
                ),
                (
                    "idx_players_snapshot_name".to_string(),
                    vec![("snapshot_id".to_string(), 0), ("name".to_string(), 0)],
                ),
                (
                    "idx_players_snapshot_pa_asc_uid".to_string(),
                    vec![
                        ("snapshot_id".to_string(), 0),
                        ("pa".to_string(), 0),
                        ("uid".to_string(), 0),
                    ],
                ),
                (
                    "idx_players_snapshot_pa_desc_uid".to_string(),
                    vec![
                        ("snapshot_id".to_string(), 0),
                        ("pa".to_string(), 1),
                        ("uid".to_string(), 0),
                    ],
                ),
                (
                    "idx_players_snapshot_value_asc_uid".to_string(),
                    vec![
                        ("snapshot_id".to_string(), 0),
                        ("market_value_gbp".to_string(), 0),
                        ("uid".to_string(), 0),
                    ],
                ),
                (
                    "idx_players_snapshot_value_desc_uid".to_string(),
                    vec![
                        ("snapshot_id".to_string(), 0),
                        ("market_value_gbp".to_string(), 1),
                        ("uid".to_string(), 0),
                    ],
                ),
            ]
        );
    }

    #[test]
    fn opening_fresh_db_applies_all_migrations_without_demo_value() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("migration-test.db");
        let conn = open_migrated(&db_path);

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user_version");
        assert_eq!(version, 34);
        assert_player_sort_index_inventory(&conn);

        let demo_value_exists: bool = conn
            .query_row(
                "SELECT EXISTS(
                     SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'demo_value'
                 )",
                [],
                |row| row.get(0),
            )
            .expect("check demo table");
        assert!(!demo_value_exists);
    }

    #[test]
    fn opening_fresh_db_creates_one_save_owned_club_dna_definition_with_valid_json_array() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("club-dna-migration-test.db"));
        conn.execute("INSERT INTO saves (name) VALUES ('Club DNA save')", [])
            .expect("insert save");
        let save_id = conn.last_insert_rowid();

        assert_eq!(
            table_columns(&conn, "club_dna_definitions"),
            ["save_id", "attribute_ids_json", "definition_version"]
        );
        for invalid_json in ["[]", "{}", "not-json"] {
            assert!(conn
                .execute(
                    "INSERT INTO club_dna_definitions (save_id, attribute_ids_json) VALUES (?1, ?2)",
                    params![save_id, invalid_json],
                )
                .is_err());
        }
        conn.execute(
            "INSERT INTO club_dna_definitions (save_id, attribute_ids_json) VALUES (?1, '[\"attr.Acceleration\"]')",
            [save_id],
        )
        .expect("insert definition");
        assert!(conn
            .execute(
                "INSERT INTO club_dna_definitions (save_id, attribute_ids_json) VALUES (?1, '[\"attr.Pace\"]')",
                [save_id],
            )
            .is_err());

        conn.execute("DELETE FROM saves WHERE id = ?1", [save_id])
            .expect("delete save");
        let remaining: i64 = conn
            .query_row("SELECT COUNT(*) FROM club_dna_definitions", [], |row| {
                row.get(0)
            })
            .expect("count definitions");
        assert_eq!(remaining, 0);
    }

    #[test]
    fn migrates_v30_without_backfilling_club_dna_definitions() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("club-dna-v30.db")).expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 30) {
            conn.execute_batch(migration.sql)
                .expect("apply migrations through v30");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set v30 version");
        }
        conn.execute("INSERT INTO saves (name) VALUES ('Existing save')", [])
            .expect("insert existing save");

        apply(&conn).expect("apply v31");

        let definition_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM club_dna_definitions", [], |row| {
                row.get(0)
            })
            .expect("count definitions");
        assert_eq!(definition_count, 0);
    }

    #[test]
    fn migrates_v32_players_to_targeted_sort_indexes_without_changing_rows() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn =
            Connection::open(temp_dir.path().join("player-indexes-v32.db")).expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 32) {
            conn.execute_batch(migration.sql)
                .expect("apply migrations through v32");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set v32 version");
        }
        conn.execute("INSERT INTO saves (name) VALUES ('Existing save')", [])
            .expect("insert save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            INSERT_SNAPSHOT_SQL,
            params![save_id, true, false, Option::<i64>::None],
        )
        .expect("insert snapshot");
        let snapshot_id = conn.last_insert_rowid();
        insert_player(&conn, snapshot_id, 42);
        conn.execute(
            "UPDATE players
             SET age = 24, market_value_gbp = 12_000_000, current_club = 'Existing FC'
             WHERE snapshot_id = ?1 AND uid = 42",
            [snapshot_id],
        )
        .expect("seed v32 player values");

        apply(&conn).expect("apply v33");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user_version");
        assert_eq!(version, 34);
        assert_player_sort_index_inventory(&conn);
        assert_eq!(
            conn.query_row(
                "SELECT uid, pa, age, market_value_gbp, current_club
                 FROM players WHERE snapshot_id = ?1",
                [snapshot_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Option<i64>>(2)?,
                        row.get::<_, Option<i64>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                    ))
                },
            )
            .expect("read preserved player"),
            (
                42,
                100,
                Some(24),
                Some(12_000_000),
                Some("Existing FC".to_string())
            )
        );
    }

    #[test]
    fn migrates_v33_current_snapshots_to_complete_potential_scores_only() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("potential-scores-v33.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 33) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v33");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set v33 version");
        }

        let mut current_snapshots = Vec::new();
        let mut retained_snapshots = Vec::new();
        for save_name in ["First save", "Second save"] {
            conn.execute("INSERT INTO saves (name) VALUES (?1)", [save_name])
                .expect("insert save");
            let save_id = conn.last_insert_rowid();
            conn.execute(
                INSERT_SNAPSHOT_SQL,
                params![save_id, true, false, Option::<i64>::None],
            )
            .expect("insert current snapshot");
            let current_snapshot_id = conn.last_insert_rowid();
            insert_player(&conn, current_snapshot_id, save_id * 10);
            current_snapshots.push((current_snapshot_id, save_id * 10));

            conn.execute(
                INSERT_SNAPSHOT_SQL,
                params![save_id, false, false, Option::<i64>::None],
            )
            .expect("insert retained snapshot");
            let retained_snapshot_id = conn.last_insert_rowid();
            insert_player(&conn, retained_snapshot_id, save_id * 10 + 1);
            retained_snapshots.push((retained_snapshot_id, save_id * 10 + 1));
        }
        for (snapshot_id, uid) in current_snapshots.iter().chain(retained_snapshots.iter()) {
            conn.execute(
                "INSERT INTO player_potential_role_scores (
                    snapshot_id, uid, role_id, score, projection_model_version
                 ) VALUES (?1, ?2, 'obsolete_sparse_role', 99, 1)",
                params![snapshot_id, uid],
            )
            .expect("insert disposable sparse row");
        }

        apply(&conn).expect("migrate v33 potential state");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user version");
        assert_eq!(version, 34);
        let player_columns = table_columns(&conn, "players");
        assert!(player_columns.contains(&"potential_attributes_json".to_string()));
        assert!(player_columns.contains(&"potential_projection_model_version".to_string()));
        let tx = conn
            .unchecked_transaction()
            .expect("start post-migration assertion transaction");
        for (snapshot_id, _) in &current_snapshots {
            potential_scores::assert_current_snapshot_complete(&tx, *snapshot_id)
                .expect("current snapshot passes the post-migration assertion");
        }
        tx.commit()
            .expect("commit post-migration assertion transaction");
        for (snapshot_id, uid) in current_snapshots {
            let (attributes_json, model_version): (Option<String>, Option<i64>) = conn
                .query_row(
                    "SELECT potential_attributes_json, potential_projection_model_version
                     FROM players WHERE snapshot_id = ?1 AND uid = ?2",
                    params![snapshot_id, uid],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .expect("read current projected attributes");
            assert!(attributes_json.is_some());
            assert_eq!(
                model_version,
                Some(potential_scores::PROJECTION_MODEL_VERSION)
            );
            let rows: Vec<(String, Option<i64>, i64)> = conn
                .prepare(
                    "SELECT role_id, score, projection_model_version
                     FROM player_potential_role_scores
                     WHERE snapshot_id = ?1 AND uid = ?2 ORDER BY role_id",
                )
                .expect("prepare current scores")
                .query_map(params![snapshot_id, uid], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })
                .expect("query current scores")
                .collect::<Result<_, _>>()
                .expect("read current scores");
            assert_eq!(
                rows.len(),
                crate::features::scoring::catalog::all_roles().len()
            );
            assert!(rows
                .iter()
                .all(|(_, _, version)| *version == potential_scores::PROJECTION_MODEL_VERSION));
        }
        for (snapshot_id, uid) in retained_snapshots {
            let derived: (Option<String>, Option<i64>, i64) = conn
                .query_row(
                    "SELECT potential_attributes_json, potential_projection_model_version,
                            (SELECT COUNT(*) FROM player_potential_role_scores
                             WHERE snapshot_id = players.snapshot_id AND uid = players.uid)
                     FROM players WHERE snapshot_id = ?1 AND uid = ?2",
                    params![snapshot_id, uid],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .expect("read retained potential state");
            assert_eq!(derived, (None, None, 0));
        }
    }

    fn assert_v34_backfill_rolls_back(source_attributes: &str, expected_error: &str) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("potential-scores-v34-rollback.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 33) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v33");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set v33 version");
        }
        conn.execute("INSERT INTO saves (name) VALUES ('Invalid projection')", [])
            .expect("insert save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            INSERT_SNAPSHOT_SQL,
            params![save_id, true, false, Option::<i64>::None],
        )
        .expect("insert current snapshot");
        let snapshot_id = conn.last_insert_rowid();
        insert_player(&conn, snapshot_id, 42);
        conn.execute(
            "UPDATE players
             SET ca = 100, pa = 140, age = 20, attributes_json = ?2
             WHERE snapshot_id = ?1 AND uid = 42",
            params![snapshot_id, source_attributes],
        )
        .expect("set v33 player input");
        conn.execute(
            "INSERT INTO player_potential_role_scores (
                snapshot_id, uid, role_id, score, projection_model_version
             ) VALUES (?1, 42, 'v33_sparse_row', 50, 1)",
            [snapshot_id],
        )
        .expect("insert v33 derived row");

        let error = apply(&conn).expect_err("reject incomplete v34 backfill");
        assert!(error.contains("migration 34 (persist_current_potential_scores) failed"));
        assert!(error.contains(expected_error));
        assert!(!error.contains("Invalid parameter name"));

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read rolled-back version");
        assert_eq!(version, 33);
        assert!(!table_columns(&conn, "players")
            .iter()
            .any(|column| column == "potential_attributes_json"));
        assert_eq!(
            conn.query_row(
                "SELECT attributes_json FROM players WHERE snapshot_id = ?1 AND uid = 42",
                [snapshot_id],
                |row| row.get::<_, String>(0),
            )
            .expect("read unchanged player source"),
            source_attributes
        );
        assert_eq!(
            conn.query_row(
                "SELECT role_id, score, projection_model_version
                 FROM player_potential_role_scores",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<i64>>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .expect("read unchanged v33 score row"),
            ("v33_sparse_row".to_string(), Some(50), 1)
        );
    }

    #[test]
    fn rolls_back_v34_when_current_player_projection_input_is_malformed() {
        assert_v34_backfill_rolls_back("not JSON", "invalid player 42 attributes JSON");
    }

    #[test]
    fn migrates_v33_sparse_current_player_attributes_as_nulls() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("potential-scores-v34-sparse.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 33) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v33");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set v33 version");
        }
        conn.execute("INSERT INTO saves (name) VALUES ('Sparse projection')", [])
            .expect("insert save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            INSERT_SNAPSHOT_SQL,
            params![save_id, true, false, Option::<i64>::None],
        )
        .expect("insert current snapshot");
        let snapshot_id = conn.last_insert_rowid();
        insert_player(&conn, snapshot_id, 42);
        conn.execute(
            "UPDATE players SET attributes_json = '{}' WHERE snapshot_id = ?1 AND uid = 42",
            [snapshot_id],
        )
        .expect("store sparse v33 attributes");
        conn.execute(
            "INSERT INTO player_potential_role_scores (
                snapshot_id, uid, role_id, score, projection_model_version
             ) VALUES (?1, 42, 'v33_sparse_row', 50, 1)",
            [snapshot_id],
        )
        .expect("insert v33 sparse row");

        apply(&conn).expect("migrate sparse v33 potential state");

        let projected_json: String = conn
            .query_row(
                "SELECT potential_attributes_json FROM players
                 WHERE snapshot_id = ?1 AND uid = 42",
                [snapshot_id],
                |row| row.get(0),
            )
            .expect("read projected sparse attributes");
        let projected: HashMap<String, Option<u8>> =
            serde_json::from_str(&projected_json).expect("parse projected sparse attributes");
        assert_eq!(
            projected.len(),
            crate::features::scoring::catalog::DUMP_ATTRIBUTE_KEYS.len()
        );
        assert!(projected.values().all(Option::is_none));
        let potential_row_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM player_potential_role_scores
                 WHERE snapshot_id = ?1 AND uid = 42",
                [snapshot_id],
                |row| row.get(0),
            )
            .expect("count complete sparse potential rows");
        assert_eq!(
            potential_row_count,
            crate::features::scoring::catalog::all_roles().len() as i64
        );
    }

    #[test]
    fn rolls_back_v34_when_current_player_projection_input_is_noninteger_or_out_of_u8_range() {
        assert_v34_backfill_rolls_back("{\"Unknown\":10.5}", "invalid type");
        assert_v34_backfill_rolls_back("{\"Unknown\":300}", "invalid value");
    }

    #[test]
    fn rolls_back_v34_when_current_player_source_attribute_is_outside_the_visible_domain() {
        assert_v34_backfill_rolls_back(
            "{\"Acceleration\":0}",
            "player 42 attribute `Acceleration` must be between 1 and 20",
        );
    }

    #[test]
    fn migrates_v31_definition_to_a_versioned_nullable_club_dna_score_cache() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("club-dna-v31.db")).expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 31) {
            conn.execute_batch(migration.sql)
                .expect("apply migrations through v31");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set v31 version");
        }
        conn.execute("INSERT INTO saves (name) VALUES ('Existing save')", [])
            .expect("insert save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO club_dna_definitions (save_id, attribute_ids_json)
             VALUES (?1, '[\"attr.Acceleration\"]')",
            [save_id],
        )
        .expect("insert v31 definition");
        conn.execute(
            INSERT_SNAPSHOT_SQL,
            params![save_id, true, false, Option::<i64>::None],
        )
        .expect("insert snapshot");
        let snapshot_id = conn.last_insert_rowid();
        insert_player(&conn, snapshot_id, 42);

        apply(&conn).expect("apply v32");

        assert_eq!(
            conn.query_row(
                "SELECT attribute_ids_json, definition_version
                 FROM club_dna_definitions WHERE save_id = ?1",
                [save_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            )
            .expect("read upgraded definition"),
            ("[\"attr.Acceleration\"]".to_string(), 1)
        );
        assert_eq!(
            table_columns(&conn, "club_dna_scores"),
            [
                "snapshot_id",
                "uid",
                "definition_version",
                "score_model_version",
                "score",
            ]
        );
        assert_eq!(
            conn.prepare("PRAGMA index_info(idx_club_dna_scores_snapshot_definition_model_score)")
                .expect("prepare cache index query")
                .query_map([], |row| row.get::<_, String>(2))
                .expect("query cache index")
                .collect::<Result<Vec<_>, _>>()
                .expect("read cache index"),
            [
                "snapshot_id",
                "definition_version",
                "score_model_version",
                "score",
            ]
        );
        for score in [None, Some(0), Some(100)] {
            conn.execute(
                "INSERT INTO club_dna_scores (
                    snapshot_id, uid, definition_version, score_model_version, score
                 ) VALUES (?1, ?2, 1, 1, ?3)",
                params![snapshot_id, 42, score],
            )
            .expect("insert valid cache score");
            conn.execute("DELETE FROM club_dna_scores", [])
                .expect("clear valid cache score");
        }
        for score in [-1, 101] {
            assert!(conn
                .execute(
                    "INSERT INTO club_dna_scores (
                        snapshot_id, uid, definition_version, score_model_version, score
                     ) VALUES (?1, ?2, 1, 1, ?3)",
                    params![snapshot_id, 42, score],
                )
                .is_err());
        }
        conn.execute(
            "INSERT INTO club_dna_scores (
                snapshot_id, uid, definition_version, score_model_version, score
             ) VALUES (?1, ?2, 1, 1, 50)",
            params![snapshot_id, 42],
        )
        .expect("seed cache row");
        conn.execute(
            "DELETE FROM players WHERE snapshot_id = ?1 AND uid = ?2",
            params![snapshot_id, 42],
        )
        .expect("delete player");
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM club_dna_scores", [], |row| row
                .get::<_, i64>(0))
                .expect("count player-cascaded rows"),
            0
        );
        conn.execute(
            INSERT_SNAPSHOT_SQL,
            params![save_id, false, false, Option::<i64>::None],
        )
        .expect("insert second snapshot");
        let second_snapshot_id = conn.last_insert_rowid();
        insert_player(&conn, second_snapshot_id, 43);
        conn.execute(
            "INSERT INTO club_dna_scores (
                snapshot_id, uid, definition_version, score_model_version, score
             ) VALUES (?1, 43, 1, 1, 50)",
            [second_snapshot_id],
        )
        .expect("seed second cache row");
        conn.execute("DELETE FROM snapshots WHERE id = ?1", [second_snapshot_id])
            .expect("delete snapshot");
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM club_dna_scores", [], |row| row
                .get::<_, i64>(0))
                .expect("count snapshot-cascaded rows"),
            0
        );
    }

    #[test]
    fn migrates_populated_v28_to_managed_club_settings() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn =
            Connection::open(temp_dir.path().join("managed-club-v28.db")).expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 27) {
            conn.execute_batch(migration.sql)
                .expect("apply migrations through v27");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set v27 version");
        }

        let first_save_id: i64 = conn
            .query_row(
                "INSERT INTO saves (name, is_active) VALUES ('First save', 1) RETURNING id",
                [],
                |row| row.get(0),
            )
            .expect("create first save");
        let second_save_id: i64 = conn
            .query_row(
                "INSERT INTO saves (name, is_active) VALUES ('Second save', 0) RETURNING id",
                [],
                |row| row.get(0),
            )
            .expect("create second save");

        for (save_id, club_name, staff_uid) in [
            (first_save_id, "First FC", 88_i64),
            (second_save_id, "Second FC", 99_i64),
        ] {
            conn.execute(
                INSERT_SNAPSHOT_SQL,
                params![save_id, true, false, Option::<i64>::None],
            )
            .expect("create existing snapshot");
            conn.execute(
                "INSERT INTO planner_tactic_lanes (
                     save_id, lane_order, lane_id, ip_weight, importance_rank,
                     preferred_foot, foot_preference, ip_position, ip_role_id,
                     oop_position, oop_role_id
                 ) VALUES (?1, 0, 'goalkeeper', 0.5, 1, 'any', 'preferred',
                           'GK', 'goalkeeper', 'GK', 'line_holding_keeper')",
                [save_id],
            )
            .expect("create existing planner tactic lane");
            conn.execute(
                "INSERT INTO planner_club_settings (save_id, primary_club)
                 VALUES (?1, ?2)",
                params![save_id, club_name],
            )
            .expect("create existing planner club settings");
            conn.execute(
                "INSERT INTO planner_club_sources (
                     save_id, team, club_name, team_level, is_primary
                 ) VALUES (?1, 'reserves', ?2, 'reserve', 1)",
                params![save_id, format!("{club_name} Reserves")],
            )
            .expect("create existing planner club source");
            conn.execute(
                "INSERT INTO staff_shortlist_entries (
                     save_id, staff_uid, preferred_job, club_job, coaching_qualifications
                 ) VALUES (?1, ?2, 'Manager', 'Club', 'Good')",
                params![save_id, staff_uid],
            )
            .expect("create existing shortlist row");
        }

        let reserve_string_id: i64 = conn
            .query_row(
                "INSERT INTO planner_strings (save_id, team, string_order)
                 VALUES (?1, 'reserves', 0) RETURNING id",
                [first_save_id],
                |row| row.get(0),
            )
            .expect("create populated reserve string");
        conn.execute(
            "INSERT INTO planner_strings (save_id, team, string_order)
             VALUES (?1, 'senior', 0), (?1, 'youth', 0)",
            [first_save_id],
        )
        .expect("create other planner strings");
        let second_string_id: i64 = conn
            .query_row(
                "INSERT INTO planner_strings (save_id, team, string_order)
                 VALUES (?1, 'senior', 0) RETURNING id",
                [second_save_id],
                |row| row.get(0),
            )
            .expect("create second save planner string");
        conn.execute(
            "INSERT INTO planner_assignments (
                 save_id, string_id, lane_id, player_uid, last_known_name, provenance
             ) VALUES (?1, ?2, 'goalkeeper', 77, 'Fixture player', 'manual')",
            params![first_save_id, reserve_string_id],
        )
        .expect("create planner assignment");
        conn.execute(
            "INSERT INTO planner_assignments (
                 save_id, string_id, lane_id, player_uid, last_known_name, provenance
             ) VALUES (?1, ?2, 'goalkeeper', 99, 'Second fixture player', 'optimizer')",
            params![second_save_id, second_string_id],
        )
        .expect("create second planner assignment");

        let first_snapshot_id: i64 = conn
            .query_row(
                "SELECT id FROM snapshots WHERE save_id = ?1",
                [first_save_id],
                |row| row.get(0),
            )
            .expect("read first snapshot id");
        insert_player(&conn, first_snapshot_id, 77);
        let academy_class_id: i64 = conn
            .query_row(
                "INSERT INTO academy_classes (save_id, class_year, is_automatic)
                 VALUES (?1, 2031, 0) RETURNING id",
                [first_save_id],
                |row| row.get(0),
            )
            .expect("create academy class");
        conn.execute(
            "INSERT INTO academy_memberships (
                 save_id, class_id, player_uid, last_known_name
             ) VALUES (?1, ?2, 77, 'Academy player')",
            params![first_save_id, academy_class_id],
        )
        .expect("create academy membership");
        conn.execute(
            "INSERT INTO academy_member_outcomes (
                 save_id, player_uid, status, buying_club, sale_fee_eur
             ) VALUES (?1, 77, 'sold', 'Buying FC', 1250000)",
            [first_save_id],
        )
        .expect("create academy outcome");
        conn.execute(
            "INSERT INTO player_youth_career_stats (
                 save_id, player_uid, career_appearances, international_caps,
                 career_goals, career_assists
             ) VALUES (?1, 77, 14, 2, 3, 5)",
            [first_save_id],
        )
        .expect("create youth enrichment");
        conn.execute(
            "INSERT INTO player_moneyball_stats (
                 snapshot_id, player_uid, asking_price_kind,
                 asking_price_lower_eur, starts, substitute_appearances,
                 minutes, statistics_json
             ) VALUES (?1, 77, 'single', 2500000, 12, 4, 1080, '{\"xg\":4.5}')",
            [first_snapshot_id],
        )
        .expect("create Moneyball enrichment");

        let planner_team_migration = all()
            .iter()
            .find(|migration| migration.version == 28)
            .expect("find planner team migration");
        conn.execute_batch(planner_team_migration.sql)
            .expect("apply migration 28");
        conn.pragma_update(None, "user_version", 28)
            .expect("set v28 version");

        let preservation_queries = [
            (
                "academy_classes",
                "SELECT json_array(id, save_id, class_year, is_automatic)
                 FROM academy_classes ORDER BY id",
            ),
            (
                "academy_memberships",
                "SELECT json_array(save_id, class_id, player_uid, last_known_name)
                 FROM academy_memberships ORDER BY save_id, player_uid",
            ),
            (
                "academy_member_outcomes",
                "SELECT json_array(save_id, player_uid, status, buying_club, sale_fee_eur)
                 FROM academy_member_outcomes ORDER BY save_id, player_uid",
            ),
            (
                "player_youth_career_stats",
                "SELECT json_array(
                     save_id, player_uid, career_appearances, international_caps,
                     career_goals, career_assists, imported_at_utc
                 ) FROM player_youth_career_stats ORDER BY save_id, player_uid",
            ),
            (
                "player_moneyball_stats",
                "SELECT json_array(
                     snapshot_id, player_uid, asking_price_kind,
                     asking_price_lower_eur, asking_price_upper_eur, starts,
                     substitute_appearances, minutes, statistics_json,
                     imported_at_utc
                 ) FROM player_moneyball_stats ORDER BY snapshot_id, player_uid",
            ),
        ];
        let preserved_rows_before: Vec<_> = preservation_queries
            .iter()
            .map(|(table, query)| (*table, query_json_rows(&conn, query)))
            .collect();

        type SnapshotRow = (
            i64,
            i64,
            i64,
            i64,
            String,
            String,
            String,
            String,
            i64,
            String,
            i64,
            Option<i64>,
        );
        type TacticLaneRow = (
            i64,
            i64,
            i64,
            String,
            f64,
            Option<i64>,
            String,
            String,
            String,
            String,
            String,
            String,
        );
        type ClubSettingRow = (i64, String);
        type ClubSourceRow = (i64, i64, String, String, Option<String>, i64);
        type ShortlistRow = (i64, i64, String, String, String);
        type PlannerStringRow = (i64, i64, String, i64);
        type AssignmentRow = (i64, i64, i64, String, i64, String, String);

        let snapshots_before: Vec<SnapshotRow> = conn
            .prepare(
                "SELECT id, save_id, is_current, schema_version, generated_at_utc,
                        game_version, supported_game_version, bridge_version,
                        protocol_version, game_date_source, scan_truncated,
                        max_accepted
                 FROM snapshots ORDER BY id",
            )
            .expect("prepare snapshot preservation query")
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
                    row.get(10)?,
                    row.get(11)?,
                ))
            })
            .expect("query snapshots before migration")
            .collect::<Result<_, _>>()
            .expect("read snapshots before migration");
        let tactic_lanes_before: Vec<TacticLaneRow> = conn
            .prepare(
                "SELECT id, save_id, lane_order, lane_id, ip_weight, importance_rank,
                        preferred_foot, foot_preference, ip_position, ip_role_id,
                        oop_position, oop_role_id
                 FROM planner_tactic_lanes ORDER BY id",
            )
            .expect("prepare tactic preservation query")
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
                    row.get(10)?,
                    row.get(11)?,
                ))
            })
            .expect("query tactic lanes before migration")
            .collect::<Result<_, _>>()
            .expect("read tactic lanes before migration");
        let club_settings_before: Vec<ClubSettingRow> = conn
            .prepare(
                "SELECT save_id, primary_club
                 FROM planner_club_settings ORDER BY save_id",
            )
            .expect("prepare club settings preservation query")
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("query club settings before migration")
            .collect::<Result<_, _>>()
            .expect("read club settings before migration");
        let club_sources_before: Vec<ClubSourceRow> = conn
            .prepare(
                "SELECT id, save_id, team, club_name, team_level, is_primary
                 FROM planner_club_sources ORDER BY id",
            )
            .expect("prepare club source preservation query")
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })
            .expect("query club sources before migration")
            .collect::<Result<_, _>>()
            .expect("read club sources before migration");
        let shortlist_before: Vec<ShortlistRow> = conn
            .prepare(
                "SELECT save_id, staff_uid, preferred_job, club_job,
                        coaching_qualifications
                 FROM staff_shortlist_entries ORDER BY save_id, staff_uid",
            )
            .expect("prepare shortlist preservation query")
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })
            .expect("query shortlist before migration")
            .collect::<Result<_, _>>()
            .expect("read shortlist before migration");
        let planner_strings_before: Vec<PlannerStringRow> = conn
            .prepare(
                "SELECT id, save_id, team, string_order
                 FROM planner_strings ORDER BY id",
            )
            .expect("prepare planner string preservation query")
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .expect("query planner strings before migration")
            .collect::<Result<_, _>>()
            .expect("read planner strings before migration");
        let assignments_before: Vec<AssignmentRow> = conn
            .prepare(
                "SELECT id, save_id, string_id, lane_id, player_uid,
                        last_known_name, provenance
                 FROM planner_assignments ORDER BY id",
            )
            .expect("prepare assignment preservation query")
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            })
            .expect("query assignments before migration")
            .collect::<Result<_, _>>()
            .expect("read assignments before migration");

        apply(&conn).expect("apply managed club migration");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read migrated version");
        assert_eq!(version, 34);
        let settings: Vec<(i64, String, String)> = conn
            .prepare(
                "SELECT save_id, team, display_name
                 FROM planner_teams ORDER BY save_id, team",
            )
            .expect("prepare settings query")
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .expect("query settings")
            .collect::<Result<_, _>>()
            .expect("read settings");
        assert_eq!(
            settings,
            vec![
                (
                    first_save_id,
                    "reserves".to_string(),
                    "Reserves".to_string()
                ),
                (first_save_id, "senior".to_string(), "Senior".to_string()),
                (first_save_id, "youth".to_string(), "Youth".to_string()),
                (
                    second_save_id,
                    "reserves".to_string(),
                    "Reserves".to_string()
                ),
                (second_save_id, "senior".to_string(), "Senior".to_string()),
                (second_save_id, "youth".to_string(), "Youth".to_string()),
            ]
        );
        let snapshots_after: Vec<SnapshotRow> = conn
            .prepare(
                "SELECT id, save_id, is_current, schema_version, generated_at_utc,
                        game_version, supported_game_version, bridge_version,
                        protocol_version, game_date_source, scan_truncated,
                        max_accepted
                 FROM snapshots ORDER BY id",
            )
            .expect("prepare migrated snapshot query")
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
                    row.get(10)?,
                    row.get(11)?,
                ))
            })
            .expect("query migrated snapshots")
            .collect::<Result<_, _>>()
            .expect("read migrated snapshots");
        assert_eq!(snapshots_after, snapshots_before);

        let tactic_lanes_after: Vec<TacticLaneRow> = conn
            .prepare(
                "SELECT id, save_id, lane_order, lane_id, ip_weight, importance_rank,
                        preferred_foot, foot_preference, ip_position, ip_role_id,
                        oop_position, oop_role_id
                 FROM planner_tactic_lanes ORDER BY id",
            )
            .expect("prepare migrated tactic query")
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
                    row.get(10)?,
                    row.get(11)?,
                ))
            })
            .expect("query migrated tactic lanes")
            .collect::<Result<_, _>>()
            .expect("read migrated tactic lanes");
        assert_eq!(tactic_lanes_after, tactic_lanes_before);

        let club_settings_after: Vec<ClubSettingRow> = conn
            .prepare(
                "SELECT save_id, club_name
                 FROM managed_club_settings ORDER BY save_id",
            )
            .expect("prepare migrated club settings query")
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("query migrated club settings")
            .collect::<Result<_, _>>()
            .expect("read migrated club settings");
        assert_eq!(club_settings_after, club_settings_before);

        assert!(!club_sources_before.is_empty());
        for removed_table in ["planner_club_settings", "planner_club_sources"] {
            let exists: bool = conn
                .query_row(
                    "SELECT EXISTS(
                         SELECT 1 FROM sqlite_master
                         WHERE type = 'table' AND name = ?1
                     )",
                    [removed_table],
                    |row| row.get(0),
                )
                .expect("check removed club-family table");
            assert!(!exists, "{removed_table} should be removed");
        }

        let shortlist_after: Vec<ShortlistRow> = conn
            .prepare(
                "SELECT save_id, staff_uid, preferred_job, club_job,
                        coaching_qualifications
                 FROM staff_shortlist_entries ORDER BY save_id, staff_uid",
            )
            .expect("prepare migrated shortlist query")
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })
            .expect("query migrated shortlist")
            .collect::<Result<_, _>>()
            .expect("read migrated shortlist");
        assert_eq!(shortlist_after, shortlist_before);

        let planner_strings_after: Vec<PlannerStringRow> = conn
            .prepare(
                "SELECT id, save_id, team, string_order
                 FROM planner_strings ORDER BY id",
            )
            .expect("prepare migrated planner string query")
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .expect("query migrated planner strings")
            .collect::<Result<_, _>>()
            .expect("read migrated planner strings");
        assert_eq!(planner_strings_after, planner_strings_before);

        let assignments_after: Vec<AssignmentRow> = conn
            .prepare(
                "SELECT id, save_id, string_id, lane_id, player_uid,
                        last_known_name, provenance
                 FROM planner_assignments ORDER BY id",
            )
            .expect("prepare migrated assignment query")
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            })
            .expect("query migrated assignments")
            .collect::<Result<_, _>>()
            .expect("read migrated assignments");
        assert_eq!(assignments_after, assignments_before);

        for ((table, query), (_, rows_before)) in preservation_queries
            .iter()
            .zip(preserved_rows_before.iter())
        {
            assert_eq!(
                query_json_rows(&conn, query),
                *rows_before,
                "{table} changed during migrations 29 and 30"
            );
        }
    }

    #[test]
    fn migrates_v29_moneyball_rows_without_backfilling_percentiles() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("moneyball-percentiles-v29.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 29) {
            conn.execute_batch(migration.sql)
                .expect("apply migrations through v29");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set v29 version");
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
        .expect("insert snapshot");
        let snapshot_id = conn.last_insert_rowid();
        insert_player(&conn, snapshot_id, 77);
        conn.execute(
            "INSERT INTO player_moneyball_stats (
                snapshot_id, player_uid, asking_price_kind, asking_price_lower_eur,
                starts, substitute_appearances, minutes, statistics_json, imported_at_utc
             ) VALUES (?1, 77, 'single', 2500000, 12, 4, 1080, '{\"goals\":7}',
                       '2026-08-19T00:00:00.000Z')",
            [snapshot_id],
        )
        .expect("insert v29 Moneyball row");

        apply(&conn).expect("apply percentile cohort migration");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read migrated version");
        assert_eq!(version, 34);
        type MoneyballRow = (
            Option<String>,
            Option<i64>,
            Option<i64>,
            Option<i64>,
            Option<i64>,
            String,
            String,
            Option<String>,
        );
        let row: MoneyballRow = conn
            .query_row(
                "SELECT asking_price_kind, asking_price_lower_eur, starts,
                        substitute_appearances, minutes, statistics_json, imported_at_utc,
                        percentiles_json
                 FROM player_moneyball_stats
                 WHERE snapshot_id = ?1 AND player_uid = 77",
                [snapshot_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                    ))
                },
            )
            .expect("read preserved v29 Moneyball row");
        assert_eq!(
            row,
            (
                Some("single".to_string()),
                Some(2_500_000),
                Some(12),
                Some(4),
                Some(1_080),
                "{\"goals\":7}".to_string(),
                "2026-08-19T00:00:00.000Z".to_string(),
                None,
            )
        );
        conn.execute(
            "UPDATE player_moneyball_stats
             SET percentiles_json = '{\"goals\":50}'
             WHERE snapshot_id = ?1 AND player_uid = 77",
            [snapshot_id],
        )
        .expect("accept a percentile object");
        assert!(conn
            .execute(
                "UPDATE player_moneyball_stats
                 SET percentiles_json = '[]'
                 WHERE snapshot_id = ?1 AND player_uid = 77",
                [snapshot_id],
            )
            .is_err());
    }

    #[test]
    fn planner_team_settings_schema_enforces_supported_names_and_constraints() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("planner-team-schema.db"));
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Test save', 1)",
            [],
        )
        .expect("create save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO planner_teams (save_id, team, display_name)
             VALUES (?1, 'senior', 'Senior')",
            [save_id],
        )
        .expect("insert valid settings");

        for (team, display_name) in [
            ("unknown", "Unknown"),
            ("reserves", " "),
            ("youth", &"x".repeat(41)),
        ] {
            assert!(conn
                .execute(
                    "INSERT INTO planner_teams (save_id, team, display_name)
                     VALUES (?1, ?2, ?3)",
                    params![save_id, team, display_name],
                )
                .is_err());
        }
        assert!(conn
            .execute(
                "INSERT INTO planner_teams (save_id, team, display_name)
                 VALUES (?1, 'reserves', 'senior')",
                [save_id],
            )
            .is_err());
    }

    #[test]
    fn player_information_visibility_defaults_to_revealed_and_is_constrained_per_save() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("player-information-visibility.db"));

        assert_eq!(
            table_columns(&conn, "saves").last().map(String::as_str),
            Some("reveal_hidden_information")
        );
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('First save', 1)",
            [],
        )
        .expect("insert first save");
        let first_save_id = conn.last_insert_rowid();
        conn.execute("INSERT INTO saves (name) VALUES ('Second save')", [])
            .expect("insert second save");
        let second_save_id = conn.last_insert_rowid();

        let defaults: Vec<i64> = conn
            .prepare(
                "SELECT reveal_hidden_information
                 FROM saves ORDER BY id",
            )
            .expect("prepare visibility query")
            .query_map([], |row| row.get(0))
            .expect("query visibility defaults")
            .collect::<Result<_, _>>()
            .expect("read visibility defaults");
        assert_eq!(defaults, vec![1, 1]);

        conn.execute(
            "UPDATE saves SET reveal_hidden_information = 0 WHERE id = ?1",
            [first_save_id],
        )
        .expect("conceal first save");
        let states: Vec<(i64, i64)> = conn
            .prepare(
                "SELECT id, reveal_hidden_information
                 FROM saves ORDER BY id",
            )
            .expect("prepare save visibility query")
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("query save visibility")
            .collect::<Result<_, _>>()
            .expect("read save visibility");
        assert_eq!(states, vec![(first_save_id, 0), (second_save_id, 1)]);

        let error = conn
            .execute(
                "UPDATE saves SET reveal_hidden_information = 2 WHERE id = ?1",
                [first_save_id],
            )
            .expect_err("reject invalid visibility state");
        assert_eq!(
            error.sqlite_error_code(),
            Some(rusqlite::ErrorCode::ConstraintViolation)
        );
    }

    #[test]
    fn migrates_v22_saves_to_revealed_player_information() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("player-information-v22.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 22) {
            conn.execute_batch(migration.sql)
                .expect("apply migrations through v22");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set v22 user version");
        }
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Existing save', 1)",
            [],
        )
        .expect("insert existing save");

        apply(&conn).expect("apply player information visibility migration");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read migrated user version");
        assert_eq!(version, 34);
        let existing: i64 = conn
            .query_row("SELECT reveal_hidden_information FROM saves", [], |row| {
                row.get(0)
            })
            .expect("read existing save visibility");
        assert_eq!(existing, 1);
    }

    #[test]
    fn migrates_v25_visibility_to_one_shared_preference_without_changing_values() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("shared-information-v25.db"))
            .expect("open test db");
        for migration in all().iter().filter(|migration| migration.version <= 25) {
            conn.execute_batch(migration.sql)
                .expect("apply through v25");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set v25 version");
        }
        conn.execute(
            "INSERT INTO saves (name, is_active, reveal_hidden_player_information)
             VALUES ('Revealed', 1, 1), ('Concealed', 0, 0)",
            [],
        )
        .expect("insert visibility states");

        apply(&conn).expect("apply shared visibility migration");

        assert_eq!(
            table_columns(&conn, "saves").last(),
            Some(&"reveal_hidden_information".to_string())
        );
        let states: Vec<i64> = conn
            .prepare("SELECT reveal_hidden_information FROM saves ORDER BY id")
            .expect("prepare shared visibility")
            .query_map([], |row| row.get(0))
            .expect("query shared visibility")
            .collect::<Result<_, _>>()
            .expect("read shared visibility");
        assert_eq!(states, vec![1, 0]);
    }

    #[test]
    fn migrates_populated_v21_database_by_dropping_only_demo_value() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("remove-demo-value-v21.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 21) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v21");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set user version");
        }

        conn.execute(
            "INSERT INTO demo_value (id, value) VALUES (1, 'template data')",
            [],
        )
        .expect("insert demo value");
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
        .expect("insert snapshot");
        let snapshot_id = conn.last_insert_rowid();
        insert_player(&conn, snapshot_id, 42);
        conn.execute(
            "INSERT INTO planner_club_settings (save_id, primary_club) VALUES (?1, 'Existing FC')",
            [save_id],
        )
        .expect("insert planner state");
        conn.execute(
            "INSERT INTO academy_classes (save_id, class_year) VALUES (?1, 2032)",
            [save_id],
        )
        .expect("insert academy class");
        conn.execute(
            "INSERT INTO player_youth_career_stats (save_id, player_uid, career_appearances)
             VALUES (?1, 42, 12)",
            [save_id],
        )
        .expect("insert youth enrichment");
        conn.execute(
            "INSERT INTO player_moneyball_stats (snapshot_id, player_uid, statistics_json)
             VALUES (?1, 42, '{}')",
            [snapshot_id],
        )
        .expect("insert Moneyball enrichment");

        apply(&conn).expect("apply latest migration");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user version");
        assert_eq!(version, 34);
        let demo_value_exists: bool = conn
            .query_row(
                "SELECT EXISTS(
                     SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'demo_value'
                 )",
                [],
                |row| row.get(0),
            )
            .expect("check removed demo table");
        assert!(!demo_value_exists);
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM saves", [], |row| row.get::<_, i64>(0))
                .expect("count saves"),
            1
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM snapshots", [], |row| row
                .get::<_, i64>(0))
                .expect("count snapshots"),
            1
        );
        assert_eq!(
            conn.query_row(
                "SELECT club_name FROM managed_club_settings WHERE save_id = ?1",
                [save_id],
                |row| row.get::<_, String>(0),
            )
            .expect("read planner state"),
            "Existing FC"
        );
        assert_eq!(
            conn.query_row(
                "SELECT class_year FROM academy_classes WHERE save_id = ?1 AND class_year = 2032",
                [save_id],
                |row| row.get::<_, i64>(0),
            )
            .expect("read academy class"),
            2032
        );
        assert_eq!(
            conn.query_row(
                "SELECT career_appearances FROM player_youth_career_stats
                 WHERE save_id = ?1 AND player_uid = 42",
                [save_id],
                |row| row.get::<_, i64>(0),
            )
            .expect("read youth enrichment"),
            12
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM player_moneyball_stats
                 WHERE snapshot_id = ?1 AND player_uid = 42",
                [snapshot_id],
                |row| row.get::<_, i64>(0),
            )
            .expect("count Moneyball enrichment"),
            1
        );
    }

    #[test]
    fn migrates_v20_to_a_cascading_potential_role_score_cache() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn =
            Connection::open(temp_dir.path().join("potential-cache-v20.db")).expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 20) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v20");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set migration version");
        }

        apply(&conn).expect("migrate v20 cache schema");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user version");
        assert_eq!(version, 34);
        assert_eq!(
            table_columns(&conn, "player_potential_role_scores"),
            [
                "snapshot_id",
                "uid",
                "role_id",
                "score",
                "projection_model_version",
            ]
        );

        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Cache save', 1)",
            [],
        )
        .expect("insert save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            INSERT_SNAPSHOT_SQL,
            params![save_id, true, false, Option::<i64>::None],
        )
        .expect("insert snapshot");
        let snapshot_id = conn.last_insert_rowid();
        insert_player(&conn, snapshot_id, 42);
        conn.execute(
            "INSERT INTO player_potential_role_scores (
                snapshot_id, uid, role_id, score, projection_model_version
             ) VALUES (?1, ?2, 'goalkeeper_ip', 80, 1)",
            params![snapshot_id, 42],
        )
        .expect("insert derived cache row");

        conn.execute(
            "DELETE FROM players WHERE snapshot_id = ?1 AND uid = ?2",
            params![snapshot_id, 42],
        )
        .expect("delete player");
        let cache_rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM player_potential_role_scores",
                [],
                |row| row.get(0),
            )
            .expect("count cascaded cache rows");
        assert_eq!(cache_rows, 0);
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
    fn migrates_v19_snapshots_to_unlatched_player_boost_recovery() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("player-boost-recovery-v19.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 19) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v19");
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

        apply(&conn).expect("apply player boost recovery migration");

        let recovery_required: i64 = conn
            .query_row(
                "SELECT boost_recovery_required FROM snapshots WHERE id = ?1",
                [snapshot_id],
                |row| row.get(0),
            )
            .expect("read migrated recovery requirement");
        assert_eq!(recovery_required, 0);
        let error = conn
            .execute(
                "UPDATE snapshots SET boost_recovery_required = 2 WHERE id = ?1",
                [snapshot_id],
            )
            .expect_err("reject an invalid recovery requirement");
        assert_eq!(
            error.sqlite_error_code(),
            Some(rusqlite::ErrorCode::ConstraintViolation)
        );
    }

    #[test]
    fn migrates_v24_player_recovery_to_shared_boost_recovery_without_changing_values() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("shared-boost-recovery-v24.db"))
            .expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 24) {
            conn.execute_batch(migration.sql)
                .expect("apply migration through v24");
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
            "UPDATE snapshots SET player_boost_recovery_required = 1 WHERE id = ?1",
            [snapshot_id],
        )
        .expect("set existing recovery requirement");

        apply(&conn).expect("apply shared boost recovery migration");

        assert_eq!(
            table_columns(&conn, "snapshots")
                .into_iter()
                .filter(|column| column.contains("boost_recovery"))
                .collect::<Vec<_>>(),
            ["boost_recovery_required"]
        );
        let recovery_required: i64 = conn
            .query_row(
                "SELECT boost_recovery_required FROM snapshots WHERE id = ?1",
                [snapshot_id],
                |row| row.get(0),
            )
            .expect("read preserved recovery requirement");
        assert_eq!(recovery_required, 1);
        assert!(conn
            .execute(
                "UPDATE snapshots SET boost_recovery_required = 2 WHERE id = ?1",
                [snapshot_id],
            )
            .is_err());
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
                "percentiles_json",
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
        assert_eq!(version, 34);
        let (save_name, is_current, primary_club): (String, i32, String) = conn
            .query_row(
                "SELECT saves.name, snapshots.is_current, managed_club_settings.club_name
                 FROM saves
                 JOIN snapshots ON snapshots.save_id = saves.id
                 JOIN managed_club_settings ON managed_club_settings.save_id = saves.id
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
        insert_player(&conn, snapshot_id, 77);
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
        assert_eq!(version, 34);
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
        assert_eq!(version, 34);
        let primary_club: String = conn
            .query_row(
                "SELECT club_name FROM managed_club_settings WHERE save_id = ?1",
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
        assert_eq!(version, 34);
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
        assert_eq!(version, 34);
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
        assert_eq!(version, 34);

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
    fn staff_role_scores_are_snapshot_scoped_constrained_and_cascade_with_staff() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("staff-role-scores.db"));

        conn.execute_batch(
            "INSERT INTO saves (id, name, is_active) VALUES (1, 'Save', 1);
             INSERT INTO snapshots (
                 id, save_id, is_current, schema_version, generated_at_utc, game_version,
                 supported_game_version, bridge_version, protocol_version, game_date,
                 game_date_source, scan_truncated, max_accepted, player_count
             ) VALUES (
                 1, 1, 1, 8, '2026-08-16T00:00:00Z', '26.3', '26.3', '0.4.0', 1,
                 NULL, 'unavailable', 0, NULL, 0
             );
             INSERT INTO staff (
                 snapshot_id, uid, name, birth_year, birth_day_of_year, age,
                 nationalities_json, nation_uid, gender, ca, pa, staff_attributes_json,
                 job_id, weekly_wage_gbp, contract_expiry_year, contract_expiry_day_of_year,
                 club, division
             ) VALUES (
                 1, 88, 'Staff', 1980, 1, 46, '[\"DEN\"]', 208, 'male', 120, 150,
                 '{}', 1, NULL, NULL, NULL, 'Club', 'Division'
             );",
        )
        .expect("seed staff owner");

        conn.execute(
            "INSERT INTO staff_role_scores (snapshot_id, uid, role_id, score)
             VALUES (1, 88, 'physio', 85)",
            [],
        )
        .expect("insert valid staff score");
        conn.execute(
            "INSERT INTO staff_role_scores (snapshot_id, uid, role_id, score)
             VALUES (1, 88, 'unavailable', NULL)",
            [],
        )
        .expect("insert nullable staff score");
        assert!(conn
            .execute(
                "INSERT INTO staff_role_scores (snapshot_id, uid, role_id, score)
                 VALUES (1, 88, 'invalid', 101)",
                [],
            )
            .is_err());
        assert!(conn
            .execute(
                "INSERT INTO staff_role_scores (snapshot_id, uid, role_id, score)
                 VALUES (1, 999, 'physio', 50)",
                [],
            )
            .is_err());

        conn.execute("DELETE FROM staff WHERE snapshot_id = 1 AND uid = 88", [])
            .expect("delete staff owner");
        let score_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM staff_role_scores", [], |row| {
                row.get(0)
            })
            .expect("count cascaded staff scores");
        assert_eq!(score_count, 0);
    }

    #[test]
    fn staff_shortlist_entries_are_save_owned_and_preserve_csv_strings() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("staff-shortlist.db"));

        assert_eq!(
            table_columns(&conn, "staff_shortlist_entries"),
            [
                "save_id",
                "staff_uid",
                "preferred_job",
                "club_job",
                "coaching_qualifications",
            ]
        );

        conn.execute_batch(
            "INSERT INTO saves (id, name, is_active) VALUES (1, 'First save', 1), (2, 'Second save', 0);
             INSERT INTO snapshots (
                 id, save_id, is_current, schema_version, generated_at_utc, game_version,
                 supported_game_version, bridge_version, protocol_version, game_date,
                 game_date_source, scan_truncated, max_accepted, player_count
             ) VALUES (
                 1, 1, 1, 8, '2026-08-16T00:00:00Z', '26.3', '26.3', '0.4.0', 1,
                 NULL, 'unavailable', 0, NULL, 0
             );",
        )
        .expect("seed shortlist owners");

        conn.execute(
            "INSERT INTO staff_shortlist_entries (
                save_id, staff_uid, preferred_job, club_job, coaching_qualifications
             ) VALUES (1, 88, 'Physio', '', 'Continental Pro')",
            [],
        )
        .expect("insert empty club job");
        conn.execute(
            "INSERT INTO staff_shortlist_entries (
                save_id, staff_uid, preferred_job, club_job, coaching_qualifications
             ) VALUES (2, 88, 'Scout', '-', '')",
            [],
        )
        .expect("allow same UID in another save");

        assert!(conn
            .execute(
                "INSERT INTO staff_shortlist_entries (
                    save_id, staff_uid, preferred_job, club_job, coaching_qualifications
                 ) VALUES (1, 88, 'Scout', '-', '')",
                [],
            )
            .is_err());
        assert!(conn
            .execute(
                "INSERT INTO staff_shortlist_entries (
                    save_id, staff_uid, preferred_job, club_job, coaching_qualifications
                 ) VALUES (1, 99, '   ', '-', '')",
                [],
            )
            .is_err());

        let entries: Vec<(i64, i64, String, String, String)> = conn
            .prepare(
                "SELECT save_id, staff_uid, preferred_job, club_job, coaching_qualifications
                 FROM staff_shortlist_entries ORDER BY save_id",
            )
            .expect("prepare shortlist query")
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })
            .expect("query shortlist entries")
            .collect::<Result<_, _>>()
            .expect("read shortlist entries");
        assert_eq!(
            entries,
            vec![
                (
                    1,
                    88,
                    "Physio".to_string(),
                    "".to_string(),
                    "Continental Pro".to_string()
                ),
                (2, 88, "Scout".to_string(), "-".to_string(), "".to_string()),
            ]
        );

        conn.execute("DELETE FROM snapshots WHERE id = 1", [])
            .expect("delete snapshot");
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM staff_shortlist_entries", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("count shortlist entries after snapshot deletion"),
            2
        );

        conn.execute("DELETE FROM saves WHERE id = 1", [])
            .expect("delete first save");
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM staff_shortlist_entries", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("count shortlist entries after save deletion"),
            1
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM staff_shortlist_entries WHERE save_id = 2",
                [],
                |row| row.get::<_, i64>(0),
            )
            .expect("count second save shortlist entries"),
            1
        );
    }

    #[test]
    fn migrates_populated_v26_database_to_empty_staff_shortlists() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("staff-shortlist-v26.db"))
            .expect("open legacy database");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 26) {
            conn.execute_batch(migration.sql)
                .expect("apply migrations through v26");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set v26 version");
        }
        conn.execute_batch(
            "INSERT INTO saves (id, name, is_active) VALUES (1, 'Existing save', 1);
             INSERT INTO snapshots (
                 id, save_id, is_current, schema_version, generated_at_utc, game_version,
                 supported_game_version, bridge_version, protocol_version, game_date,
                 game_date_source, scan_truncated, max_accepted, player_count
             ) VALUES (
                 1, 1, 1, 8, '2026-08-16T00:00:00Z', '26.3', '26.3', '0.4.0', 1,
                 NULL, 'unavailable', 0, NULL, 0
             );",
        )
        .expect("seed v26 database");

        apply(&conn).expect("apply shortlist migration");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read migrated version");
        assert_eq!(version, 34);
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM saves", [], |row| row.get::<_, i64>(0))
                .expect("count retained saves"),
            1
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM snapshots", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("count retained snapshots"),
            1
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM staff_shortlist_entries", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("count empty shortlist entries"),
            0
        );
    }

    #[test]
    fn migrates_populated_v23_without_backfilling_staff_scores() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("populated-v23.db"))
            .expect("open legacy database");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 23) {
            conn.execute_batch(migration.sql).expect("apply v23 schema");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set legacy version");
        }
        conn.execute_batch(
            "INSERT INTO saves (id, name, is_active) VALUES (1, 'Save', 1);
             INSERT INTO snapshots (
                 id, save_id, is_current, schema_version, generated_at_utc, game_version,
                 supported_game_version, bridge_version, protocol_version, game_date,
                 game_date_source, scan_truncated, max_accepted, player_count
             ) VALUES (
                 1, 1, 1, 8, '2026-08-16T00:00:00Z', '26.3', '26.3', '0.4.0', 1,
                 NULL, 'unavailable', 0, NULL, 0
             );
             INSERT INTO staff (
                 snapshot_id, uid, name, nationalities_json, gender, ca, pa,
                 staff_attributes_json
             ) VALUES (1, 88, 'Existing Staff', '[]', 'unknown', 100, 120, '{}');",
        )
        .expect("seed v23 staff");

        apply(&conn).expect("apply staff score migration");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read migrated version");
        let staff_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM staff", [], |row| row.get(0))
            .expect("count retained staff");
        let score_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM staff_role_scores", [], |row| {
                row.get(0)
            })
            .expect("count absent backfill");
        assert_eq!(version, 34);
        assert_eq!(staff_count, 1);
        assert_eq!(score_count, 0);
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
                "reveal_hidden_information",
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
                "boost_recovery_required",
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
                "potential_attributes_json",
                "potential_projection_model_version",
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
                "idx_club_dna_scores_snapshot_definition_model_score",
                "idx_planner_assignments_string",
                "idx_planner_strings_save_team_order",
                "idx_planner_tactic_lanes_save_importance_rank",
                "idx_planner_tactic_lanes_save_order",
                "idx_planner_teams_save_team",
                "idx_player_potential_role_scores_snapshot_role_score",
                "idx_player_role_scores_snapshot_role",
                "idx_players_snapshot_age_asc_uid",
                "idx_players_snapshot_age_desc_uid",
                "idx_players_snapshot_ca",
                "idx_players_snapshot_current_club_uid",
                "idx_players_snapshot_name",
                "idx_players_snapshot_pa_asc_uid",
                "idx_players_snapshot_pa_desc_uid",
                "idx_players_snapshot_value_asc_uid",
                "idx_players_snapshot_value_desc_uid",
                "idx_saves_context_token",
                "idx_saves_one_active",
                "idx_snapshots_context_token",
                "idx_snapshots_one_current_per_save",
                "idx_staff_role_scores_snapshot_role",
                "idx_staff_shortlist_entries_save_preferred_job",
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
        assert_eq!(version, 34);
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
        assert_eq!(version, 34);
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

        for legacy_version in 1..24 {
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
            assert_eq!(version, 34, "legacy version {legacy_version}");
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

        assert_eq!(migrations.len(), 34);
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
        assert_eq!(migrations[19].version, 20);
        assert_eq!(
            migrations[19].description,
            "add_player_boost_recovery_requirement"
        );
        assert_eq!(migrations[19].sql, PLAYER_BOOST_RECOVERY_SQL);
        assert_eq!(migrations[20].version, 21);
        assert_eq!(
            migrations[20].description,
            "create_player_potential_role_scores"
        );
        assert_eq!(migrations[20].sql, PLAYER_POTENTIAL_ROLE_SCORES_SQL);
        assert_eq!(migrations[21].version, 22);
        assert_eq!(migrations[21].description, "drop_demo_value_table");
        assert_eq!(migrations[21].sql, DROP_DEMO_VALUE_SQL);
        assert_eq!(migrations[22].version, 23);
        assert_eq!(
            migrations[22].description,
            "add_player_information_visibility"
        );
        assert_eq!(migrations[22].sql, PLAYER_INFORMATION_VISIBILITY_SQL);
        assert_eq!(migrations[23].version, 24);
        assert_eq!(migrations[23].description, "create_staff_role_scores");
        assert_eq!(migrations[23].sql, STAFF_ROLE_SCORES_SQL);
        assert_eq!(migrations[24].version, 25);
        assert_eq!(
            migrations[24].description,
            "share_boost_recovery_requirement"
        );
        assert_eq!(migrations[24].sql, SHARED_BOOST_RECOVERY_SQL);
        assert_eq!(migrations[25].version, 26);
        assert_eq!(
            migrations[25].description,
            "share_hidden_information_visibility"
        );
        assert_eq!(migrations[25].sql, SHARED_INFORMATION_VISIBILITY_SQL);
        assert_eq!(migrations[26].version, 27);
        assert_eq!(migrations[26].description, "create_staff_shortlist_entries");
        assert_eq!(migrations[26].sql, STAFF_SHORTLIST_SCHEMA_SQL);
        assert_eq!(migrations[27].version, 28);
        assert_eq!(migrations[27].description, "create_planner_team_settings");
        assert_eq!(migrations[27].sql, PLANNER_TEAM_SETTINGS_SQL);
        assert_eq!(migrations[28].version, 29);
        assert_eq!(
            migrations[28].description,
            "replace_club_family_with_managed_club"
        );
        assert_eq!(migrations[28].sql, MANAGED_CLUB_SETTINGS_SQL);
        assert_eq!(migrations[29].version, 30);
        assert_eq!(
            migrations[29].description,
            "add_moneyball_percentile_cohorts"
        );
        assert_eq!(migrations[29].sql, MONEYBALL_PERCENTILE_COHORT_SQL);
        assert_eq!(migrations[30].version, 31);
        assert_eq!(migrations[30].description, "create_club_dna_definitions");
        assert_eq!(migrations[30].sql, CLUB_DNA_DEFINITIONS_SQL);
        assert_eq!(migrations[31].version, 32);
        assert_eq!(migrations[31].description, "create_club_dna_score_cache");
        assert_eq!(migrations[31].sql, CLUB_DNA_SCORE_CACHE_SQL);
        assert_eq!(migrations[32].version, 33);
        assert_eq!(migrations[32].description, "index_targeted_player_sorts");
        assert_eq!(migrations[32].sql, PLAYER_TARGETED_SORT_INDEXES_SQL);
        assert_eq!(migrations[33].version, 34);
        assert_eq!(
            migrations[33].description,
            "persist_current_potential_scores"
        );
        assert_eq!(migrations[33].sql, POTENTIAL_SCORES_V34_SQL);
    }

    #[test]
    fn opening_fresh_db_applies_managed_club_schema() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("planner-migration-test.db"));

        assert_eq!(
            table_columns(&conn, "managed_club_settings"),
            ["save_id", "club_name"]
        );
        assert!(table_columns(&conn, "planner_club_settings").is_empty());
        assert!(table_columns(&conn, "planner_club_sources").is_empty());
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
