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

pub const PLAYER_SHORTLIST_SCHEMA_SQL: &str = "
CREATE TABLE player_shortlist_entries (
    save_id INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    player_uid INTEGER NOT NULL,
    PRIMARY KEY (save_id, player_uid)
);
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

pub const STAFF_ASSIGNMENT_TARGETS_SQL: &str = "
CREATE TABLE staff_assignment_targets (
    save_id INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    scope TEXT NOT NULL CHECK (scope IN ('senior', 'reserves', 'youth', 'club')),
    job_id TEXT NOT NULL CHECK (trim(job_id) <> ''),
    slot_count INTEGER NOT NULL CHECK (slot_count BETWEEN 1 AND 50),
    PRIMARY KEY (save_id, scope, job_id)
);

CREATE INDEX idx_staff_assignment_targets_save_scope
    ON staff_assignment_targets(save_id, scope);
";

pub const STAFF_ASSIGNMENT_TARGETS_RESET_SQL: &str = "
DELETE FROM staff_assignment_targets;
";

pub const CLUB_SET_PIECE_COACH_TARGET_SQL: &str = "
INSERT OR IGNORE INTO staff_assignment_targets (save_id, scope, job_id, slot_count)
SELECT save_id, 'club', 'set_piece_coach', 1
FROM staff_assignment_targets
WHERE job_id = 'set_piece_coach';

UPDATE staff_assignment_targets
SET slot_count = 1
WHERE scope = 'club' AND job_id = 'set_piece_coach';

DELETE FROM staff_assignment_targets
WHERE scope <> 'club' AND job_id = 'set_piece_coach';
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

pub const COMPACT_ROLE_METRICS_V38_SQL: &str = "
CREATE TABLE player_role_metrics (
    snapshot_id INTEGER NOT NULL,
    uid INTEGER NOT NULL,
    score_model_version INTEGER NOT NULL CHECK (score_model_version > 0),
    projection_model_version INTEGER NOT NULL CHECK (projection_model_version > 0),
    goalkeeper_ip INTEGER CHECK (goalkeeper_ip IS NULL OR (goalkeeper_ip >= 0 AND goalkeeper_ip <= 100)),
    ball_playing_goalkeeper_ip INTEGER CHECK (ball_playing_goalkeeper_ip IS NULL OR (ball_playing_goalkeeper_ip >= 0 AND ball_playing_goalkeeper_ip <= 100)),
    no_nonsense_goalkeeper_ip INTEGER CHECK (no_nonsense_goalkeeper_ip IS NULL OR (no_nonsense_goalkeeper_ip >= 0 AND no_nonsense_goalkeeper_ip <= 100)),
    line_holding_keeper_oop INTEGER CHECK (line_holding_keeper_oop IS NULL OR (line_holding_keeper_oop >= 0 AND line_holding_keeper_oop <= 100)),
    sweeper_keeper_oop INTEGER CHECK (sweeper_keeper_oop IS NULL OR (sweeper_keeper_oop >= 0 AND sweeper_keeper_oop <= 100)),
    centre_back_ip INTEGER CHECK (centre_back_ip IS NULL OR (centre_back_ip >= 0 AND centre_back_ip <= 100)),
    ball_playing_centre_back_ip INTEGER CHECK (ball_playing_centre_back_ip IS NULL OR (ball_playing_centre_back_ip >= 0 AND ball_playing_centre_back_ip <= 100)),
    no_nonsense_centre_back_ip INTEGER CHECK (no_nonsense_centre_back_ip IS NULL OR (no_nonsense_centre_back_ip >= 0 AND no_nonsense_centre_back_ip <= 100)),
    wide_centre_back_ip INTEGER CHECK (wide_centre_back_ip IS NULL OR (wide_centre_back_ip >= 0 AND wide_centre_back_ip <= 100)),
    advanced_centre_back_ip INTEGER CHECK (advanced_centre_back_ip IS NULL OR (advanced_centre_back_ip >= 0 AND advanced_centre_back_ip <= 100)),
    overlapping_centre_back_ip INTEGER CHECK (overlapping_centre_back_ip IS NULL OR (overlapping_centre_back_ip >= 0 AND overlapping_centre_back_ip <= 100)),
    covering_centre_back_oop INTEGER CHECK (covering_centre_back_oop IS NULL OR (covering_centre_back_oop >= 0 AND covering_centre_back_oop <= 100)),
    stopping_centre_back_oop INTEGER CHECK (stopping_centre_back_oop IS NULL OR (stopping_centre_back_oop >= 0 AND stopping_centre_back_oop <= 100)),
    covering_wide_centre_back_oop INTEGER CHECK (covering_wide_centre_back_oop IS NULL OR (covering_wide_centre_back_oop >= 0 AND covering_wide_centre_back_oop <= 100)),
    stopping_wide_centre_back_oop INTEGER CHECK (stopping_wide_centre_back_oop IS NULL OR (stopping_wide_centre_back_oop >= 0 AND stopping_wide_centre_back_oop <= 100)),
    full_back_ip INTEGER CHECK (full_back_ip IS NULL OR (full_back_ip >= 0 AND full_back_ip <= 100)),
    inside_full_back_ip INTEGER CHECK (inside_full_back_ip IS NULL OR (inside_full_back_ip >= 0 AND inside_full_back_ip <= 100)),
    holding_full_back_oop INTEGER CHECK (holding_full_back_oop IS NULL OR (holding_full_back_oop >= 0 AND holding_full_back_oop <= 100)),
    pressing_full_back_oop INTEGER CHECK (pressing_full_back_oop IS NULL OR (pressing_full_back_oop >= 0 AND pressing_full_back_oop <= 100)),
    inside_wing_back_ip INTEGER CHECK (inside_wing_back_ip IS NULL OR (inside_wing_back_ip >= 0 AND inside_wing_back_ip <= 100)),
    playmaking_wing_back_ip INTEGER CHECK (playmaking_wing_back_ip IS NULL OR (playmaking_wing_back_ip >= 0 AND playmaking_wing_back_ip <= 100)),
    wing_back_ip INTEGER CHECK (wing_back_ip IS NULL OR (wing_back_ip >= 0 AND wing_back_ip <= 100)),
    advanced_wing_back_ip INTEGER CHECK (advanced_wing_back_ip IS NULL OR (advanced_wing_back_ip >= 0 AND advanced_wing_back_ip <= 100)),
    holding_wing_back_oop INTEGER CHECK (holding_wing_back_oop IS NULL OR (holding_wing_back_oop >= 0 AND holding_wing_back_oop <= 100)),
    pressing_wing_back_oop INTEGER CHECK (pressing_wing_back_oop IS NULL OR (pressing_wing_back_oop >= 0 AND pressing_wing_back_oop <= 100)),
    defensive_midfielder_ip INTEGER CHECK (defensive_midfielder_ip IS NULL OR (defensive_midfielder_ip >= 0 AND defensive_midfielder_ip <= 100)),
    box_to_box_midfielder_ip INTEGER CHECK (box_to_box_midfielder_ip IS NULL OR (box_to_box_midfielder_ip >= 0 AND box_to_box_midfielder_ip <= 100)),
    box_to_box_playmaker_ip INTEGER CHECK (box_to_box_playmaker_ip IS NULL OR (box_to_box_playmaker_ip >= 0 AND box_to_box_playmaker_ip <= 100)),
    deep_lying_playmaker_ip INTEGER CHECK (deep_lying_playmaker_ip IS NULL OR (deep_lying_playmaker_ip >= 0 AND deep_lying_playmaker_ip <= 100)),
    half_back_ip INTEGER CHECK (half_back_ip IS NULL OR (half_back_ip >= 0 AND half_back_ip <= 100)),
    dropping_defensive_midfielder_oop INTEGER CHECK (dropping_defensive_midfielder_oop IS NULL OR (dropping_defensive_midfielder_oop >= 0 AND dropping_defensive_midfielder_oop <= 100)),
    pressing_defensive_midfielder_oop INTEGER CHECK (pressing_defensive_midfielder_oop IS NULL OR (pressing_defensive_midfielder_oop >= 0 AND pressing_defensive_midfielder_oop <= 100)),
    screening_defensive_midfielder_oop INTEGER CHECK (screening_defensive_midfielder_oop IS NULL OR (screening_defensive_midfielder_oop >= 0 AND screening_defensive_midfielder_oop <= 100)),
    wide_covering_defensive_midfielder_oop INTEGER CHECK (wide_covering_defensive_midfielder_oop IS NULL OR (wide_covering_defensive_midfielder_oop >= 0 AND wide_covering_defensive_midfielder_oop <= 100)),
    central_midfielder_ip INTEGER CHECK (central_midfielder_ip IS NULL OR (central_midfielder_ip >= 0 AND central_midfielder_ip <= 100)),
    advanced_playmaker_ip INTEGER CHECK (advanced_playmaker_ip IS NULL OR (advanced_playmaker_ip >= 0 AND advanced_playmaker_ip <= 100)),
    midfield_playmaker_ip INTEGER CHECK (midfield_playmaker_ip IS NULL OR (midfield_playmaker_ip >= 0 AND midfield_playmaker_ip <= 100)),
    wide_central_midfielder_ip INTEGER CHECK (wide_central_midfielder_ip IS NULL OR (wide_central_midfielder_ip >= 0 AND wide_central_midfielder_ip <= 100)),
    pressing_central_midfielder_oop INTEGER CHECK (pressing_central_midfielder_oop IS NULL OR (pressing_central_midfielder_oop >= 0 AND pressing_central_midfielder_oop <= 100)),
    screening_central_midfielder_oop INTEGER CHECK (screening_central_midfielder_oop IS NULL OR (screening_central_midfielder_oop >= 0 AND screening_central_midfielder_oop <= 100)),
    wide_covering_central_midfielder_oop INTEGER CHECK (wide_covering_central_midfielder_oop IS NULL OR (wide_covering_central_midfielder_oop >= 0 AND wide_covering_central_midfielder_oop <= 100)),
    wide_midfielder_ip INTEGER CHECK (wide_midfielder_ip IS NULL OR (wide_midfielder_ip >= 0 AND wide_midfielder_ip <= 100)),
    tracking_wide_midfielder_oop INTEGER CHECK (tracking_wide_midfielder_oop IS NULL OR (tracking_wide_midfielder_oop >= 0 AND tracking_wide_midfielder_oop <= 100)),
    wide_outlet_wide_midfielder_oop INTEGER CHECK (wide_outlet_wide_midfielder_oop IS NULL OR (wide_outlet_wide_midfielder_oop >= 0 AND wide_outlet_wide_midfielder_oop <= 100)),
    inside_winger_ip INTEGER CHECK (inside_winger_ip IS NULL OR (inside_winger_ip >= 0 AND inside_winger_ip <= 100)),
    playmaking_winger_ip INTEGER CHECK (playmaking_winger_ip IS NULL OR (playmaking_winger_ip >= 0 AND playmaking_winger_ip <= 100)),
    winger_ip INTEGER CHECK (winger_ip IS NULL OR (winger_ip >= 0 AND winger_ip <= 100)),
    attacking_midfielder_ip INTEGER CHECK (attacking_midfielder_ip IS NULL OR (attacking_midfielder_ip >= 0 AND attacking_midfielder_ip <= 100)),
    channel_midfielder_ip INTEGER CHECK (channel_midfielder_ip IS NULL OR (channel_midfielder_ip >= 0 AND channel_midfielder_ip <= 100)),
    free_role_ip INTEGER CHECK (free_role_ip IS NULL OR (free_role_ip >= 0 AND free_role_ip <= 100)),
    second_striker_ip INTEGER CHECK (second_striker_ip IS NULL OR (second_striker_ip >= 0 AND second_striker_ip <= 100)),
    central_outlet_attacking_midfielder_oop INTEGER CHECK (central_outlet_attacking_midfielder_oop IS NULL OR (central_outlet_attacking_midfielder_oop >= 0 AND central_outlet_attacking_midfielder_oop <= 100)),
    splitting_outlet_attacking_midfielder_oop INTEGER CHECK (splitting_outlet_attacking_midfielder_oop IS NULL OR (splitting_outlet_attacking_midfielder_oop >= 0 AND splitting_outlet_attacking_midfielder_oop <= 100)),
    tracking_attacking_midfielder_oop INTEGER CHECK (tracking_attacking_midfielder_oop IS NULL OR (tracking_attacking_midfielder_oop >= 0 AND tracking_attacking_midfielder_oop <= 100)),
    wide_forward_ip INTEGER CHECK (wide_forward_ip IS NULL OR (wide_forward_ip >= 0 AND wide_forward_ip <= 100)),
    inside_forward_ip INTEGER CHECK (inside_forward_ip IS NULL OR (inside_forward_ip >= 0 AND inside_forward_ip <= 100)),
    inside_outlet_winger_oop INTEGER CHECK (inside_outlet_winger_oop IS NULL OR (inside_outlet_winger_oop >= 0 AND inside_outlet_winger_oop <= 100)),
    tracking_winger_oop INTEGER CHECK (tracking_winger_oop IS NULL OR (tracking_winger_oop >= 0 AND tracking_winger_oop <= 100)),
    wide_outlet_winger_oop INTEGER CHECK (wide_outlet_winger_oop IS NULL OR (wide_outlet_winger_oop >= 0 AND wide_outlet_winger_oop <= 100)),
    centre_forward_ip INTEGER CHECK (centre_forward_ip IS NULL OR (centre_forward_ip >= 0 AND centre_forward_ip <= 100)),
    channel_forward_ip INTEGER CHECK (channel_forward_ip IS NULL OR (channel_forward_ip >= 0 AND channel_forward_ip <= 100)),
    deep_lying_forward_ip INTEGER CHECK (deep_lying_forward_ip IS NULL OR (deep_lying_forward_ip >= 0 AND deep_lying_forward_ip <= 100)),
    false_nine_ip INTEGER CHECK (false_nine_ip IS NULL OR (false_nine_ip >= 0 AND false_nine_ip <= 100)),
    poacher_ip INTEGER CHECK (poacher_ip IS NULL OR (poacher_ip >= 0 AND poacher_ip <= 100)),
    target_forward_ip INTEGER CHECK (target_forward_ip IS NULL OR (target_forward_ip >= 0 AND target_forward_ip <= 100)),
    central_outlet_centre_forward_oop INTEGER CHECK (central_outlet_centre_forward_oop IS NULL OR (central_outlet_centre_forward_oop >= 0 AND central_outlet_centre_forward_oop <= 100)),
    splitting_outlet_centre_forward_oop INTEGER CHECK (splitting_outlet_centre_forward_oop IS NULL OR (splitting_outlet_centre_forward_oop >= 0 AND splitting_outlet_centre_forward_oop <= 100)),
    tracking_centre_forward_oop INTEGER CHECK (tracking_centre_forward_oop IS NULL OR (tracking_centre_forward_oop >= 0 AND tracking_centre_forward_oop <= 100)),
    potential_goalkeeper_ip INTEGER CHECK (potential_goalkeeper_ip IS NULL OR (potential_goalkeeper_ip >= 0 AND potential_goalkeeper_ip <= 100)),
    potential_ball_playing_goalkeeper_ip INTEGER CHECK (potential_ball_playing_goalkeeper_ip IS NULL OR (potential_ball_playing_goalkeeper_ip >= 0 AND potential_ball_playing_goalkeeper_ip <= 100)),
    potential_no_nonsense_goalkeeper_ip INTEGER CHECK (potential_no_nonsense_goalkeeper_ip IS NULL OR (potential_no_nonsense_goalkeeper_ip >= 0 AND potential_no_nonsense_goalkeeper_ip <= 100)),
    potential_line_holding_keeper_oop INTEGER CHECK (potential_line_holding_keeper_oop IS NULL OR (potential_line_holding_keeper_oop >= 0 AND potential_line_holding_keeper_oop <= 100)),
    potential_sweeper_keeper_oop INTEGER CHECK (potential_sweeper_keeper_oop IS NULL OR (potential_sweeper_keeper_oop >= 0 AND potential_sweeper_keeper_oop <= 100)),
    potential_centre_back_ip INTEGER CHECK (potential_centre_back_ip IS NULL OR (potential_centre_back_ip >= 0 AND potential_centre_back_ip <= 100)),
    potential_ball_playing_centre_back_ip INTEGER CHECK (potential_ball_playing_centre_back_ip IS NULL OR (potential_ball_playing_centre_back_ip >= 0 AND potential_ball_playing_centre_back_ip <= 100)),
    potential_no_nonsense_centre_back_ip INTEGER CHECK (potential_no_nonsense_centre_back_ip IS NULL OR (potential_no_nonsense_centre_back_ip >= 0 AND potential_no_nonsense_centre_back_ip <= 100)),
    potential_wide_centre_back_ip INTEGER CHECK (potential_wide_centre_back_ip IS NULL OR (potential_wide_centre_back_ip >= 0 AND potential_wide_centre_back_ip <= 100)),
    potential_advanced_centre_back_ip INTEGER CHECK (potential_advanced_centre_back_ip IS NULL OR (potential_advanced_centre_back_ip >= 0 AND potential_advanced_centre_back_ip <= 100)),
    potential_overlapping_centre_back_ip INTEGER CHECK (potential_overlapping_centre_back_ip IS NULL OR (potential_overlapping_centre_back_ip >= 0 AND potential_overlapping_centre_back_ip <= 100)),
    potential_covering_centre_back_oop INTEGER CHECK (potential_covering_centre_back_oop IS NULL OR (potential_covering_centre_back_oop >= 0 AND potential_covering_centre_back_oop <= 100)),
    potential_stopping_centre_back_oop INTEGER CHECK (potential_stopping_centre_back_oop IS NULL OR (potential_stopping_centre_back_oop >= 0 AND potential_stopping_centre_back_oop <= 100)),
    potential_covering_wide_centre_back_oop INTEGER CHECK (potential_covering_wide_centre_back_oop IS NULL OR (potential_covering_wide_centre_back_oop >= 0 AND potential_covering_wide_centre_back_oop <= 100)),
    potential_stopping_wide_centre_back_oop INTEGER CHECK (potential_stopping_wide_centre_back_oop IS NULL OR (potential_stopping_wide_centre_back_oop >= 0 AND potential_stopping_wide_centre_back_oop <= 100)),
    potential_full_back_ip INTEGER CHECK (potential_full_back_ip IS NULL OR (potential_full_back_ip >= 0 AND potential_full_back_ip <= 100)),
    potential_inside_full_back_ip INTEGER CHECK (potential_inside_full_back_ip IS NULL OR (potential_inside_full_back_ip >= 0 AND potential_inside_full_back_ip <= 100)),
    potential_holding_full_back_oop INTEGER CHECK (potential_holding_full_back_oop IS NULL OR (potential_holding_full_back_oop >= 0 AND potential_holding_full_back_oop <= 100)),
    potential_pressing_full_back_oop INTEGER CHECK (potential_pressing_full_back_oop IS NULL OR (potential_pressing_full_back_oop >= 0 AND potential_pressing_full_back_oop <= 100)),
    potential_inside_wing_back_ip INTEGER CHECK (potential_inside_wing_back_ip IS NULL OR (potential_inside_wing_back_ip >= 0 AND potential_inside_wing_back_ip <= 100)),
    potential_playmaking_wing_back_ip INTEGER CHECK (potential_playmaking_wing_back_ip IS NULL OR (potential_playmaking_wing_back_ip >= 0 AND potential_playmaking_wing_back_ip <= 100)),
    potential_wing_back_ip INTEGER CHECK (potential_wing_back_ip IS NULL OR (potential_wing_back_ip >= 0 AND potential_wing_back_ip <= 100)),
    potential_advanced_wing_back_ip INTEGER CHECK (potential_advanced_wing_back_ip IS NULL OR (potential_advanced_wing_back_ip >= 0 AND potential_advanced_wing_back_ip <= 100)),
    potential_holding_wing_back_oop INTEGER CHECK (potential_holding_wing_back_oop IS NULL OR (potential_holding_wing_back_oop >= 0 AND potential_holding_wing_back_oop <= 100)),
    potential_pressing_wing_back_oop INTEGER CHECK (potential_pressing_wing_back_oop IS NULL OR (potential_pressing_wing_back_oop >= 0 AND potential_pressing_wing_back_oop <= 100)),
    potential_defensive_midfielder_ip INTEGER CHECK (potential_defensive_midfielder_ip IS NULL OR (potential_defensive_midfielder_ip >= 0 AND potential_defensive_midfielder_ip <= 100)),
    potential_box_to_box_midfielder_ip INTEGER CHECK (potential_box_to_box_midfielder_ip IS NULL OR (potential_box_to_box_midfielder_ip >= 0 AND potential_box_to_box_midfielder_ip <= 100)),
    potential_box_to_box_playmaker_ip INTEGER CHECK (potential_box_to_box_playmaker_ip IS NULL OR (potential_box_to_box_playmaker_ip >= 0 AND potential_box_to_box_playmaker_ip <= 100)),
    potential_deep_lying_playmaker_ip INTEGER CHECK (potential_deep_lying_playmaker_ip IS NULL OR (potential_deep_lying_playmaker_ip >= 0 AND potential_deep_lying_playmaker_ip <= 100)),
    potential_half_back_ip INTEGER CHECK (potential_half_back_ip IS NULL OR (potential_half_back_ip >= 0 AND potential_half_back_ip <= 100)),
    potential_dropping_defensive_midfielder_oop INTEGER CHECK (potential_dropping_defensive_midfielder_oop IS NULL OR (potential_dropping_defensive_midfielder_oop >= 0 AND potential_dropping_defensive_midfielder_oop <= 100)),
    potential_pressing_defensive_midfielder_oop INTEGER CHECK (potential_pressing_defensive_midfielder_oop IS NULL OR (potential_pressing_defensive_midfielder_oop >= 0 AND potential_pressing_defensive_midfielder_oop <= 100)),
    potential_screening_defensive_midfielder_oop INTEGER CHECK (potential_screening_defensive_midfielder_oop IS NULL OR (potential_screening_defensive_midfielder_oop >= 0 AND potential_screening_defensive_midfielder_oop <= 100)),
    potential_wide_covering_defensive_midfielder_oop INTEGER CHECK (potential_wide_covering_defensive_midfielder_oop IS NULL OR (potential_wide_covering_defensive_midfielder_oop >= 0 AND potential_wide_covering_defensive_midfielder_oop <= 100)),
    potential_central_midfielder_ip INTEGER CHECK (potential_central_midfielder_ip IS NULL OR (potential_central_midfielder_ip >= 0 AND potential_central_midfielder_ip <= 100)),
    potential_advanced_playmaker_ip INTEGER CHECK (potential_advanced_playmaker_ip IS NULL OR (potential_advanced_playmaker_ip >= 0 AND potential_advanced_playmaker_ip <= 100)),
    potential_midfield_playmaker_ip INTEGER CHECK (potential_midfield_playmaker_ip IS NULL OR (potential_midfield_playmaker_ip >= 0 AND potential_midfield_playmaker_ip <= 100)),
    potential_wide_central_midfielder_ip INTEGER CHECK (potential_wide_central_midfielder_ip IS NULL OR (potential_wide_central_midfielder_ip >= 0 AND potential_wide_central_midfielder_ip <= 100)),
    potential_pressing_central_midfielder_oop INTEGER CHECK (potential_pressing_central_midfielder_oop IS NULL OR (potential_pressing_central_midfielder_oop >= 0 AND potential_pressing_central_midfielder_oop <= 100)),
    potential_screening_central_midfielder_oop INTEGER CHECK (potential_screening_central_midfielder_oop IS NULL OR (potential_screening_central_midfielder_oop >= 0 AND potential_screening_central_midfielder_oop <= 100)),
    potential_wide_covering_central_midfielder_oop INTEGER CHECK (potential_wide_covering_central_midfielder_oop IS NULL OR (potential_wide_covering_central_midfielder_oop >= 0 AND potential_wide_covering_central_midfielder_oop <= 100)),
    potential_wide_midfielder_ip INTEGER CHECK (potential_wide_midfielder_ip IS NULL OR (potential_wide_midfielder_ip >= 0 AND potential_wide_midfielder_ip <= 100)),
    potential_tracking_wide_midfielder_oop INTEGER CHECK (potential_tracking_wide_midfielder_oop IS NULL OR (potential_tracking_wide_midfielder_oop >= 0 AND potential_tracking_wide_midfielder_oop <= 100)),
    potential_wide_outlet_wide_midfielder_oop INTEGER CHECK (potential_wide_outlet_wide_midfielder_oop IS NULL OR (potential_wide_outlet_wide_midfielder_oop >= 0 AND potential_wide_outlet_wide_midfielder_oop <= 100)),
    potential_inside_winger_ip INTEGER CHECK (potential_inside_winger_ip IS NULL OR (potential_inside_winger_ip >= 0 AND potential_inside_winger_ip <= 100)),
    potential_playmaking_winger_ip INTEGER CHECK (potential_playmaking_winger_ip IS NULL OR (potential_playmaking_winger_ip >= 0 AND potential_playmaking_winger_ip <= 100)),
    potential_winger_ip INTEGER CHECK (potential_winger_ip IS NULL OR (potential_winger_ip >= 0 AND potential_winger_ip <= 100)),
    potential_attacking_midfielder_ip INTEGER CHECK (potential_attacking_midfielder_ip IS NULL OR (potential_attacking_midfielder_ip >= 0 AND potential_attacking_midfielder_ip <= 100)),
    potential_channel_midfielder_ip INTEGER CHECK (potential_channel_midfielder_ip IS NULL OR (potential_channel_midfielder_ip >= 0 AND potential_channel_midfielder_ip <= 100)),
    potential_free_role_ip INTEGER CHECK (potential_free_role_ip IS NULL OR (potential_free_role_ip >= 0 AND potential_free_role_ip <= 100)),
    potential_second_striker_ip INTEGER CHECK (potential_second_striker_ip IS NULL OR (potential_second_striker_ip >= 0 AND potential_second_striker_ip <= 100)),
    potential_central_outlet_attacking_midfielder_oop INTEGER CHECK (potential_central_outlet_attacking_midfielder_oop IS NULL OR (potential_central_outlet_attacking_midfielder_oop >= 0 AND potential_central_outlet_attacking_midfielder_oop <= 100)),
    potential_splitting_outlet_attacking_midfielder_oop INTEGER CHECK (potential_splitting_outlet_attacking_midfielder_oop IS NULL OR (potential_splitting_outlet_attacking_midfielder_oop >= 0 AND potential_splitting_outlet_attacking_midfielder_oop <= 100)),
    potential_tracking_attacking_midfielder_oop INTEGER CHECK (potential_tracking_attacking_midfielder_oop IS NULL OR (potential_tracking_attacking_midfielder_oop >= 0 AND potential_tracking_attacking_midfielder_oop <= 100)),
    potential_wide_forward_ip INTEGER CHECK (potential_wide_forward_ip IS NULL OR (potential_wide_forward_ip >= 0 AND potential_wide_forward_ip <= 100)),
    potential_inside_forward_ip INTEGER CHECK (potential_inside_forward_ip IS NULL OR (potential_inside_forward_ip >= 0 AND potential_inside_forward_ip <= 100)),
    potential_inside_outlet_winger_oop INTEGER CHECK (potential_inside_outlet_winger_oop IS NULL OR (potential_inside_outlet_winger_oop >= 0 AND potential_inside_outlet_winger_oop <= 100)),
    potential_tracking_winger_oop INTEGER CHECK (potential_tracking_winger_oop IS NULL OR (potential_tracking_winger_oop >= 0 AND potential_tracking_winger_oop <= 100)),
    potential_wide_outlet_winger_oop INTEGER CHECK (potential_wide_outlet_winger_oop IS NULL OR (potential_wide_outlet_winger_oop >= 0 AND potential_wide_outlet_winger_oop <= 100)),
    potential_centre_forward_ip INTEGER CHECK (potential_centre_forward_ip IS NULL OR (potential_centre_forward_ip >= 0 AND potential_centre_forward_ip <= 100)),
    potential_channel_forward_ip INTEGER CHECK (potential_channel_forward_ip IS NULL OR (potential_channel_forward_ip >= 0 AND potential_channel_forward_ip <= 100)),
    potential_deep_lying_forward_ip INTEGER CHECK (potential_deep_lying_forward_ip IS NULL OR (potential_deep_lying_forward_ip >= 0 AND potential_deep_lying_forward_ip <= 100)),
    potential_false_nine_ip INTEGER CHECK (potential_false_nine_ip IS NULL OR (potential_false_nine_ip >= 0 AND potential_false_nine_ip <= 100)),
    potential_poacher_ip INTEGER CHECK (potential_poacher_ip IS NULL OR (potential_poacher_ip >= 0 AND potential_poacher_ip <= 100)),
    potential_target_forward_ip INTEGER CHECK (potential_target_forward_ip IS NULL OR (potential_target_forward_ip >= 0 AND potential_target_forward_ip <= 100)),
    potential_central_outlet_centre_forward_oop INTEGER CHECK (potential_central_outlet_centre_forward_oop IS NULL OR (potential_central_outlet_centre_forward_oop >= 0 AND potential_central_outlet_centre_forward_oop <= 100)),
    potential_splitting_outlet_centre_forward_oop INTEGER CHECK (potential_splitting_outlet_centre_forward_oop IS NULL OR (potential_splitting_outlet_centre_forward_oop >= 0 AND potential_splitting_outlet_centre_forward_oop <= 100)),
    potential_tracking_centre_forward_oop INTEGER CHECK (potential_tracking_centre_forward_oop IS NULL OR (potential_tracking_centre_forward_oop >= 0 AND potential_tracking_centre_forward_oop <= 100)),
    PRIMARY KEY (snapshot_id, uid),
    FOREIGN KEY (snapshot_id, uid) REFERENCES players(snapshot_id, uid) ON DELETE CASCADE
);

CREATE TABLE staff_role_metrics (
    snapshot_id INTEGER NOT NULL,
    uid INTEGER NOT NULL,
    score_model_version INTEGER NOT NULL CHECK (score_model_version > 0),
    assistant_manager INTEGER CHECK (assistant_manager IS NULL OR (assistant_manager >= 0 AND assistant_manager <= 100)),
    manager INTEGER CHECK (manager IS NULL OR (manager >= 0 AND manager <= 100)),
    coach_attacking_technical INTEGER CHECK (coach_attacking_technical IS NULL OR (coach_attacking_technical >= 0 AND coach_attacking_technical <= 100)),
    coach_attacking_tactical INTEGER CHECK (coach_attacking_tactical IS NULL OR (coach_attacking_tactical >= 0 AND coach_attacking_tactical <= 100)),
    coach_defending_technical INTEGER CHECK (coach_defending_technical IS NULL OR (coach_defending_technical >= 0 AND coach_defending_technical <= 100)),
    coach_defending_tactical INTEGER CHECK (coach_defending_tactical IS NULL OR (coach_defending_tactical >= 0 AND coach_defending_tactical <= 100)),
    coach_possession_technical INTEGER CHECK (coach_possession_technical IS NULL OR (coach_possession_technical >= 0 AND coach_possession_technical <= 100)),
    coach_possession_tactical INTEGER CHECK (coach_possession_tactical IS NULL OR (coach_possession_tactical >= 0 AND coach_possession_tactical <= 100)),
    coach_fitness INTEGER CHECK (coach_fitness IS NULL OR (coach_fitness >= 0 AND coach_fitness <= 100)),
    coach_goalkeeping INTEGER CHECK (coach_goalkeeping IS NULL OR (coach_goalkeeping >= 0 AND coach_goalkeeping <= 100)),
    set_piece_coach INTEGER CHECK (set_piece_coach IS NULL OR (set_piece_coach >= 0 AND set_piece_coach <= 100)),
    loan_manager INTEGER CHECK (loan_manager IS NULL OR (loan_manager >= 0 AND loan_manager <= 100)),
    head_of_youth_development INTEGER CHECK (head_of_youth_development IS NULL OR (head_of_youth_development >= 0 AND head_of_youth_development <= 100)),
    scout INTEGER CHECK (scout IS NULL OR (scout >= 0 AND scout <= 100)),
    director_of_football INTEGER CHECK (director_of_football IS NULL OR (director_of_football >= 0 AND director_of_football <= 100)),
    technical_director INTEGER CHECK (technical_director IS NULL OR (technical_director >= 0 AND technical_director <= 100)),
    recruitment_analyst INTEGER CHECK (recruitment_analyst IS NULL OR (recruitment_analyst >= 0 AND recruitment_analyst <= 100)),
    head_performance_analyst INTEGER CHECK (head_performance_analyst IS NULL OR (head_performance_analyst >= 0 AND head_performance_analyst <= 100)),
    performance_analyst INTEGER CHECK (performance_analyst IS NULL OR (performance_analyst >= 0 AND performance_analyst <= 100)),
    physio INTEGER CHECK (physio IS NULL OR (physio >= 0 AND physio <= 100)),
    sports_scientist INTEGER CHECK (sports_scientist IS NULL OR (sports_scientist >= 0 AND sports_scientist <= 100)),
    PRIMARY KEY (snapshot_id, uid),
    FOREIGN KEY (snapshot_id, uid) REFERENCES staff(snapshot_id, uid) ON DELETE CASCADE
);
";

pub const DROP_NORMALIZED_SCORE_TABLES_V39_SQL: &str = "
DROP TABLE IF EXISTS player_role_scores;
DROP TABLE IF EXISTS player_potential_role_scores;
DROP TABLE IF EXISTS staff_role_scores;
";

pub const MIGRATION_V40_SQL: &str = "
ALTER TABLE player_role_metrics ADD COLUMN goalkeeper_oop INTEGER CHECK (goalkeeper_oop IS NULL OR goalkeeper_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN centre_back_oop INTEGER CHECK (centre_back_oop IS NULL OR centre_back_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN wide_centre_back_oop INTEGER CHECK (wide_centre_back_oop IS NULL OR wide_centre_back_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN full_back_oop INTEGER CHECK (full_back_oop IS NULL OR full_back_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN wing_back_oop INTEGER CHECK (wing_back_oop IS NULL OR wing_back_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN defensive_midfielder_oop INTEGER CHECK (defensive_midfielder_oop IS NULL OR defensive_midfielder_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN central_midfielder_oop INTEGER CHECK (central_midfielder_oop IS NULL OR central_midfielder_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN wide_midfielder_oop INTEGER CHECK (wide_midfielder_oop IS NULL OR wide_midfielder_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN attacking_midfielder_oop INTEGER CHECK (attacking_midfielder_oop IS NULL OR attacking_midfielder_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN winger_oop INTEGER CHECK (winger_oop IS NULL OR winger_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN centre_forward_oop INTEGER CHECK (centre_forward_oop IS NULL OR centre_forward_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN potential_goalkeeper_oop INTEGER CHECK (potential_goalkeeper_oop IS NULL OR potential_goalkeeper_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN potential_centre_back_oop INTEGER CHECK (potential_centre_back_oop IS NULL OR potential_centre_back_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN potential_wide_centre_back_oop INTEGER CHECK (potential_wide_centre_back_oop IS NULL OR potential_wide_centre_back_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN potential_full_back_oop INTEGER CHECK (potential_full_back_oop IS NULL OR potential_full_back_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN potential_wing_back_oop INTEGER CHECK (potential_wing_back_oop IS NULL OR potential_wing_back_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN potential_defensive_midfielder_oop INTEGER CHECK (potential_defensive_midfielder_oop IS NULL OR potential_defensive_midfielder_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN potential_central_midfielder_oop INTEGER CHECK (potential_central_midfielder_oop IS NULL OR potential_central_midfielder_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN potential_wide_midfielder_oop INTEGER CHECK (potential_wide_midfielder_oop IS NULL OR potential_wide_midfielder_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN potential_attacking_midfielder_oop INTEGER CHECK (potential_attacking_midfielder_oop IS NULL OR potential_attacking_midfielder_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN potential_winger_oop INTEGER CHECK (potential_winger_oop IS NULL OR potential_winger_oop BETWEEN 0 AND 100);
ALTER TABLE player_role_metrics ADD COLUMN potential_centre_forward_oop INTEGER CHECK (potential_centre_forward_oop IS NULL OR potential_centre_forward_oop BETWEEN 0 AND 100);
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
        Migration {
            version: 35,
            description: "create_staff_assignment_targets",
            sql: STAFF_ASSIGNMENT_TARGETS_SQL,
        },
        Migration {
            version: 36,
            description: "reset_staff_assignment_targets",
            sql: STAFF_ASSIGNMENT_TARGETS_RESET_SQL,
        },
        Migration {
            version: 37,
            description: "make_set_piece_coach_club_wide",
            sql: CLUB_SET_PIECE_COACH_TARGET_SQL,
        },
        Migration {
            version: 38,
            description: "create_compact_role_metrics",
            sql: COMPACT_ROLE_METRICS_V38_SQL,
        },
        Migration {
            version: 39,
            description: "drop_normalized_score_tables",
            sql: DROP_NORMALIZED_SCORE_TABLES_V39_SQL,
        },
        Migration {
            version: 40,
            description: "expand_compact_role_metrics_for_generic_oop",
            sql: MIGRATION_V40_SQL,
        },
        Migration {
            version: 41,
            description: "create_player_shortlist_entries",
            sql: PLAYER_SHORTLIST_SCHEMA_SQL,
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
    fn opening_fresh_db_applies_staff_assignment_target_schema() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("staff-assignment-targets.db"));

        let columns = table_columns(&conn, "staff_assignment_targets");
        assert_eq!(columns, ["save_id", "scope", "job_id", "slot_count"]);
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Target save', 1)",
            [],
        )
        .expect("create active save");
        let save_id: i64 = conn
            .query_row("SELECT id FROM saves WHERE is_active = 1", [], |row| {
                row.get(0)
            })
            .expect("active save");
        for slot_count in [1, 50] {
            conn.execute(
                "INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count)
                 VALUES (?1, 'club', ?2, ?3)",
                params![save_id, format!("job_{slot_count}"), slot_count],
            )
            .expect("accept boundary target");
        }
        for slot_count in [0, 51] {
            assert!(conn
                .execute(
                    "INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count)
                     VALUES (?1, 'club', ?2, ?3)",
                    params![save_id, format!("invalid_{slot_count}"), slot_count],
                )
                .is_err());
        }
        conn.execute("DELETE FROM saves WHERE id = ?1", [save_id])
            .expect("delete save");
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM staff_assignment_targets", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("target cascade"),
            0
        );
    }

    #[test]
    fn migrates_populated_v35_by_clearing_only_staff_assignment_targets() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("v35-target-reset.db")).expect("open db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 35) {
            conn.execute_batch(migration.sql)
                .expect("apply v35 migration");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set v35 version");
        }
        conn.execute(
            "INSERT INTO saves (id, name, is_active) VALUES (1, 'Retained save', 1)",
            [],
        )
        .expect("insert save");
        conn.execute_batch(
            "INSERT INTO planner_teams (save_id, team, display_name) VALUES
                 (1, 'senior', 'Senior'),
                 (1, 'reserves', 'Reserves');
             INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count) VALUES
                 (1, 'senior', 'head_performance_analyst', 1),
                 (1, 'club', 'scout', 4);",
        )
        .expect("seed v35 state");

        apply(&conn).expect("migrate v35 database");

        assert_eq!(
            conn.pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
                .expect("read v41 version"),
            41
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM saves", [], |row| row.get::<_, i64>(0))
                .expect("retain saves"),
            1
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM planner_teams", [], |row| row
                .get::<_, i64>(0))
                .expect("retain planner teams"),
            2
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM staff_assignment_targets", [], |row| {
                row.get::<_, i64>(0)
            },)
                .expect("clear target rows"),
            0
        );
        let save_token: String = conn
            .query_row("SELECT context_token FROM saves WHERE id = 1", [], |row| {
                row.get(0)
            })
            .expect("read save token");
        let targets = crate::features::staff::assignment_targets::get_targets(&conn, &save_token)
            .expect("read reset target catalog");
        assert_eq!(targets.targets.len(), 22);
        assert!(targets.targets.iter().all(|target| target.slot_count == 0));
    }

    #[test]
    fn opening_fresh_db_applies_all_migrations_without_demo_value() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("migration-test.db");
        let conn = open_migrated(&db_path);

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user_version");
        assert_eq!(version, 41);
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
        assert_eq!(version, 41);
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
    fn migrates_v33_without_preserving_normalized_potential_state() {
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
        assert_eq!(version, 41);
        let player_columns = table_columns(&conn, "players");
        assert!(player_columns.contains(&"potential_attributes_json".to_string()));
        assert!(player_columns.contains(&"potential_projection_model_version".to_string()));
        for (snapshot_id, uid) in current_snapshots.iter().chain(retained_snapshots.iter()) {
            let derived: (Option<String>, Option<i64>) = conn
                .query_row(
                    "SELECT potential_attributes_json, potential_projection_model_version
                     FROM players WHERE snapshot_id = ?1 AND uid = ?2",
                    params![snapshot_id, uid],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .expect("read unmaterialized compact state");
            assert_eq!(derived, (None, None));
        }
        // Normalized tables are dropped in v39; end-state must have no normalized rows.
        let normalized_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'player_potential_role_scores')",
                [],
                |row| row.get(0),
            )
            .expect("check normalized absence");
        assert!(!normalized_exists);
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
        assert_eq!(version, 41);
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
        assert_eq!(version, 41);
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
        assert_eq!(version, 41);
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
        assert_eq!(version, 41);
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
        assert_eq!(version, 41);
        // v39 drops normalized tables
        for table in ["player_role_scores", "player_potential_role_scores"] {
            let exists: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
                    [table],
                    |row| row.get(0),
                )
                .expect("check post-upgrade absence");
            assert!(!exists, "{table} must be absent after v39");
        }

        // Verify upgrade preserved cascade semantics via compact
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
        let tx = conn
            .unchecked_transaction()
            .expect("start compact materialization");
        crate::features::player_metrics::potential_scores::rebuild_snapshot(&tx, snapshot_id)
            .expect("materialize compact after v20 upgrade");
        tx.commit().expect("commit compact");
        let before: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM player_role_metrics WHERE snapshot_id = ?1",
                [snapshot_id],
                |row| row.get(0),
            )
            .expect("count compact before delete");
        assert_eq!(before, 1);
        conn.execute(
            "DELETE FROM players WHERE snapshot_id = ?1 AND uid = ?2",
            params![snapshot_id, 42],
        )
        .expect("delete player");
        let after: i64 = conn
            .query_row("SELECT COUNT(*) FROM player_role_metrics", [], |row| {
                row.get(0)
            })
            .expect("count cascaded compact rows");
        assert_eq!(after, 0);
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
        assert_eq!(version, 41);
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
        assert_eq!(version, 41);
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
        assert_eq!(version, 41);
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
        assert_eq!(version, 41);
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
        assert_eq!(version, 41);
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
    fn fresh_db_has_no_normalized_score_tables_or_indexes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("no-normalized.db"));

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read user_version");
        assert_eq!(version, 41);

        for table in [
            "player_role_scores",
            "player_potential_role_scores",
            "staff_role_scores",
        ] {
            let exists: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
                    [table],
                    |row| row.get(0),
                )
                .expect("check table absence");
            assert!(!exists, "{table} must be absent after v39");
        }
        for index in [
            "idx_player_role_scores_snapshot_role",
            "idx_player_potential_role_scores_snapshot_role_score",
            "idx_staff_role_scores_snapshot_role",
        ] {
            let exists: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'index' AND name = ?1)",
                    [index],
                    |row| row.get(0),
                )
                .expect("check index absence");
            assert!(!exists, "{index} must be absent after v39");
        }
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
    fn player_shortlist_entries_are_save_owned_without_a_redundant_save_index() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("player-shortlist.db"));

        assert_eq!(
            table_columns(&conn, "player_shortlist_entries"),
            ["save_id", "player_uid"]
        );
        let shortlist_indexes: Vec<String> = conn
            .prepare(
                "SELECT name FROM sqlite_master
                 WHERE type = 'index' AND tbl_name = 'player_shortlist_entries'
                 ORDER BY name",
            )
            .expect("prepare shortlist index inventory")
            .query_map([], |row| row.get(0))
            .expect("query shortlist index inventory")
            .collect::<Result<_, _>>()
            .expect("read shortlist index inventory");
        assert_eq!(
            shortlist_indexes,
            ["sqlite_autoindex_player_shortlist_entries_1"],
            "the composite primary key prefix serves per-save lookups"
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
            "INSERT INTO player_shortlist_entries (save_id, player_uid) VALUES (1, 77)",
            [],
        )
        .expect("insert first save entry");
        conn.execute(
            "INSERT INTO player_shortlist_entries (save_id, player_uid) VALUES (2, 77)",
            [],
        )
        .expect("allow same UID in another save");
        assert!(
            conn.execute(
                "INSERT INTO player_shortlist_entries (save_id, player_uid) VALUES (1, 77)",
                [],
            )
            .is_err(),
            "duplicate (save_id, player_uid) must be rejected"
        );

        conn.execute("DELETE FROM snapshots WHERE id = 1", [])
            .expect("delete snapshot");
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM player_shortlist_entries", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("count shortlist entries after snapshot deletion"),
            2,
            "snapshot replacement must not clear the save-owned list"
        );

        conn.execute("DELETE FROM saves WHERE id = 1", [])
            .expect("delete first save");
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM player_shortlist_entries", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("count shortlist entries after save deletion"),
            1
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM player_shortlist_entries WHERE save_id = 2",
                [],
                |row| row.get::<_, i64>(0),
            )
            .expect("count second save shortlist entries"),
            1
        );
    }

    #[test]
    fn migrates_populated_v40_database_to_empty_player_shortlists() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("player-shortlist-v40.db"))
            .expect("open legacy database");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 40) {
            conn.execute_batch(migration.sql)
                .expect("apply migrations through v40");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set v40 version");
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
        .expect("seed v40 database");

        apply(&conn).expect("apply player shortlist migration");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read migrated version");
        assert_eq!(version, 41);
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
            conn.query_row("SELECT COUNT(*) FROM player_shortlist_entries", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("count empty shortlist entries"),
            0
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
        assert_eq!(version, 41);
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
        let normalized_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'staff_role_scores')",
                [],
                |row| row.get(0),
            )
            .expect("check normalized absence");
        assert_eq!(version, 41);
        assert_eq!(staff_count, 1);
        assert!(!normalized_exists);
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
                "idx_staff_assignment_targets_save_scope",
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
        assert_eq!(version, 41);
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
        assert_eq!(version, 41);
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
            assert_eq!(version, 41, "legacy version {legacy_version}");
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

        assert_eq!(migrations.len(), 41);
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
        assert_eq!(migrations[34].version, 35);
        assert_eq!(
            migrations[34].description,
            "create_staff_assignment_targets"
        );
        assert_eq!(migrations[34].sql, STAFF_ASSIGNMENT_TARGETS_SQL);
        assert_eq!(migrations[35].version, 36);
        assert_eq!(migrations[35].description, "reset_staff_assignment_targets");
        assert_eq!(migrations[35].sql, STAFF_ASSIGNMENT_TARGETS_RESET_SQL);
        assert_eq!(migrations[36].version, 37);
        assert_eq!(migrations[36].description, "make_set_piece_coach_club_wide");
        assert_eq!(migrations[36].sql, CLUB_SET_PIECE_COACH_TARGET_SQL);
        assert_eq!(migrations[37].version, 38);
        assert_eq!(migrations[37].description, "create_compact_role_metrics");
        assert_eq!(migrations[37].sql, COMPACT_ROLE_METRICS_V38_SQL);
        assert_eq!(migrations[38].version, 39);
        assert_eq!(migrations[38].description, "drop_normalized_score_tables");
        assert_eq!(migrations[38].sql, DROP_NORMALIZED_SCORE_TABLES_V39_SQL);
        assert_eq!(migrations[39].version, 40);
        assert_eq!(
            migrations[39].description,
            "expand_compact_role_metrics_for_generic_oop"
        );
        assert_eq!(migrations[39].sql, MIGRATION_V40_SQL);
        assert_eq!(migrations[40].version, 41);
        assert_eq!(
            migrations[40].description,
            "create_player_shortlist_entries"
        );
        assert_eq!(migrations[40].sql, PLAYER_SHORTLIST_SCHEMA_SQL);
    }

    #[test]
    fn migrates_v36_set_piece_targets_to_one_club_wide_slot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn =
            Connection::open(temp_dir.path().join("v36-set-piece-target.db")).expect("open db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 36) {
            conn.execute_batch(migration.sql)
                .expect("apply v36 migration");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set v36 version");
        }
        conn.execute(
            "INSERT INTO saves (id, name, is_active) VALUES (1, 'Save', 1)",
            [],
        )
        .expect("insert save");
        conn.execute_batch(
            "INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count) VALUES
                 (1, 'senior', 'set_piece_coach', 4),
                 (1, 'reserves', 'set_piece_coach', 2),
                 (1, 'senior', 'coaches', 6);",
        )
        .expect("insert v36 targets");

        apply(&conn).expect("migrate v36 database");

        assert_eq!(
            conn.pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
                .expect("read v41 version"),
            41
        );
        let targets = conn
            .prepare(
                "SELECT scope, job_id, slot_count FROM staff_assignment_targets
                 ORDER BY job_id, scope",
            )
            .expect("prepare targets")
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .expect("read targets")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect targets");
        assert_eq!(
            targets,
            [
                ("senior".to_string(), "coaches".to_string(), 6),
                ("club".to_string(), "set_piece_coach".to_string(), 1),
            ]
        );
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

    #[test]
    fn opening_fresh_db_creates_the_exact_compact_player_role_metrics_inventory() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("compact-player-metrics.db"));

        assert_eq!(
            table_columns(&conn, "player_role_metrics"),
            [
                "snapshot_id",
                "uid",
                "score_model_version",
                "projection_model_version",
                "goalkeeper_ip",
                "ball_playing_goalkeeper_ip",
                "no_nonsense_goalkeeper_ip",
                "line_holding_keeper_oop",
                "sweeper_keeper_oop",
                "centre_back_ip",
                "ball_playing_centre_back_ip",
                "no_nonsense_centre_back_ip",
                "wide_centre_back_ip",
                "advanced_centre_back_ip",
                "overlapping_centre_back_ip",
                "covering_centre_back_oop",
                "stopping_centre_back_oop",
                "covering_wide_centre_back_oop",
                "stopping_wide_centre_back_oop",
                "full_back_ip",
                "inside_full_back_ip",
                "holding_full_back_oop",
                "pressing_full_back_oop",
                "inside_wing_back_ip",
                "playmaking_wing_back_ip",
                "wing_back_ip",
                "advanced_wing_back_ip",
                "holding_wing_back_oop",
                "pressing_wing_back_oop",
                "defensive_midfielder_ip",
                "box_to_box_midfielder_ip",
                "box_to_box_playmaker_ip",
                "deep_lying_playmaker_ip",
                "half_back_ip",
                "dropping_defensive_midfielder_oop",
                "pressing_defensive_midfielder_oop",
                "screening_defensive_midfielder_oop",
                "wide_covering_defensive_midfielder_oop",
                "central_midfielder_ip",
                "advanced_playmaker_ip",
                "midfield_playmaker_ip",
                "wide_central_midfielder_ip",
                "pressing_central_midfielder_oop",
                "screening_central_midfielder_oop",
                "wide_covering_central_midfielder_oop",
                "wide_midfielder_ip",
                "tracking_wide_midfielder_oop",
                "wide_outlet_wide_midfielder_oop",
                "inside_winger_ip",
                "playmaking_winger_ip",
                "winger_ip",
                "attacking_midfielder_ip",
                "channel_midfielder_ip",
                "free_role_ip",
                "second_striker_ip",
                "central_outlet_attacking_midfielder_oop",
                "splitting_outlet_attacking_midfielder_oop",
                "tracking_attacking_midfielder_oop",
                "wide_forward_ip",
                "inside_forward_ip",
                "inside_outlet_winger_oop",
                "tracking_winger_oop",
                "wide_outlet_winger_oop",
                "centre_forward_ip",
                "channel_forward_ip",
                "deep_lying_forward_ip",
                "false_nine_ip",
                "poacher_ip",
                "target_forward_ip",
                "central_outlet_centre_forward_oop",
                "splitting_outlet_centre_forward_oop",
                "tracking_centre_forward_oop",
                "potential_goalkeeper_ip",
                "potential_ball_playing_goalkeeper_ip",
                "potential_no_nonsense_goalkeeper_ip",
                "potential_line_holding_keeper_oop",
                "potential_sweeper_keeper_oop",
                "potential_centre_back_ip",
                "potential_ball_playing_centre_back_ip",
                "potential_no_nonsense_centre_back_ip",
                "potential_wide_centre_back_ip",
                "potential_advanced_centre_back_ip",
                "potential_overlapping_centre_back_ip",
                "potential_covering_centre_back_oop",
                "potential_stopping_centre_back_oop",
                "potential_covering_wide_centre_back_oop",
                "potential_stopping_wide_centre_back_oop",
                "potential_full_back_ip",
                "potential_inside_full_back_ip",
                "potential_holding_full_back_oop",
                "potential_pressing_full_back_oop",
                "potential_inside_wing_back_ip",
                "potential_playmaking_wing_back_ip",
                "potential_wing_back_ip",
                "potential_advanced_wing_back_ip",
                "potential_holding_wing_back_oop",
                "potential_pressing_wing_back_oop",
                "potential_defensive_midfielder_ip",
                "potential_box_to_box_midfielder_ip",
                "potential_box_to_box_playmaker_ip",
                "potential_deep_lying_playmaker_ip",
                "potential_half_back_ip",
                "potential_dropping_defensive_midfielder_oop",
                "potential_pressing_defensive_midfielder_oop",
                "potential_screening_defensive_midfielder_oop",
                "potential_wide_covering_defensive_midfielder_oop",
                "potential_central_midfielder_ip",
                "potential_advanced_playmaker_ip",
                "potential_midfield_playmaker_ip",
                "potential_wide_central_midfielder_ip",
                "potential_pressing_central_midfielder_oop",
                "potential_screening_central_midfielder_oop",
                "potential_wide_covering_central_midfielder_oop",
                "potential_wide_midfielder_ip",
                "potential_tracking_wide_midfielder_oop",
                "potential_wide_outlet_wide_midfielder_oop",
                "potential_inside_winger_ip",
                "potential_playmaking_winger_ip",
                "potential_winger_ip",
                "potential_attacking_midfielder_ip",
                "potential_channel_midfielder_ip",
                "potential_free_role_ip",
                "potential_second_striker_ip",
                "potential_central_outlet_attacking_midfielder_oop",
                "potential_splitting_outlet_attacking_midfielder_oop",
                "potential_tracking_attacking_midfielder_oop",
                "potential_wide_forward_ip",
                "potential_inside_forward_ip",
                "potential_inside_outlet_winger_oop",
                "potential_tracking_winger_oop",
                "potential_wide_outlet_winger_oop",
                "potential_centre_forward_ip",
                "potential_channel_forward_ip",
                "potential_deep_lying_forward_ip",
                "potential_false_nine_ip",
                "potential_poacher_ip",
                "potential_target_forward_ip",
                "potential_central_outlet_centre_forward_oop",
                "potential_splitting_outlet_centre_forward_oop",
                "potential_tracking_centre_forward_oop",
                "goalkeeper_oop",
                "centre_back_oop",
                "wide_centre_back_oop",
                "full_back_oop",
                "wing_back_oop",
                "defensive_midfielder_oop",
                "central_midfielder_oop",
                "wide_midfielder_oop",
                "attacking_midfielder_oop",
                "winger_oop",
                "centre_forward_oop",
                "potential_goalkeeper_oop",
                "potential_centre_back_oop",
                "potential_wide_centre_back_oop",
                "potential_full_back_oop",
                "potential_wing_back_oop",
                "potential_defensive_midfielder_oop",
                "potential_central_midfielder_oop",
                "potential_wide_midfielder_oop",
                "potential_attacking_midfielder_oop",
                "potential_winger_oop",
                "potential_centre_forward_oop",
            ]
        );
        assert_eq!(table_columns(&conn, "player_role_metrics").len(), 162);
    }

    #[test]
    fn migrates_v39_compact_rows_to_v40_without_backfilling_scores() {
        const NEW_ROLE_IDS: [&str; 11] = [
            "goalkeeper_oop",
            "centre_back_oop",
            "wide_centre_back_oop",
            "full_back_oop",
            "wing_back_oop",
            "defensive_midfielder_oop",
            "central_midfielder_oop",
            "wide_midfielder_oop",
            "attacking_midfielder_oop",
            "winger_oop",
            "centre_forward_oop",
        ];
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("v40-generic-oop.db")).expect("open db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        for migration in all().iter().filter(|migration| migration.version <= 39) {
            conn.execute_batch(migration.sql)
                .expect("apply legacy migration");
            conn.pragma_update(None, "user_version", migration.version)
                .expect("set legacy version");
        }
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Legacy save', 1)",
            [],
        )
        .expect("insert save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            INSERT_SNAPSHOT_SQL,
            params![save_id, true, false, Option::<i64>::None],
        )
        .expect("insert legacy snapshot");
        let snapshot_id = conn.last_insert_rowid();
        insert_player(&conn, snapshot_id, 77);
        conn.execute(
            "INSERT INTO player_role_metrics (
                snapshot_id, uid, score_model_version, projection_model_version,
                goalkeeper_ip, potential_goalkeeper_ip
             ) VALUES (?1, 77, 1, 2, 73, 81)",
            [snapshot_id],
        )
        .expect("insert legacy compact row");

        apply(&conn).expect("migrate v39 database");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read v41 version");
        assert_eq!(version, 41);
        assert_eq!(table_columns(&conn, "player_role_metrics").len(), 162);
        let row_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM player_role_metrics", [], |row| {
                row.get(0)
            })
            .expect("count compact rows");
        assert_eq!(row_count, 1);
        let (score_version, kept_current, kept_potential): (i64, Option<i64>, Option<i64>) = conn
            .query_row(
                "SELECT score_model_version, goalkeeper_ip, potential_goalkeeper_ip
                 FROM player_role_metrics WHERE snapshot_id = ?1 AND uid = 77",
                [snapshot_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read migrated legacy row");
        assert_eq!(score_version, 1);
        assert_eq!(kept_current, Some(73));
        assert_eq!(kept_potential, Some(81));
        for role_id in NEW_ROLE_IDS {
            for column in [role_id.to_string(), format!("potential_{role_id}")] {
                let value: Option<i64> = conn
                    .query_row(
                        &format!(
                            "SELECT {column} FROM player_role_metrics
                             WHERE snapshot_id = ?1 AND uid = 77"
                        ),
                        [snapshot_id],
                        |row| row.get(0),
                    )
                    .expect("read new nullable column");
                assert_eq!(value, None, "{column} must stay NULL without backfill");
            }
        }
        let ddl: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master
                 WHERE type = 'table' AND name = 'player_role_metrics'",
                [],
                |row| row.get(0),
            )
            .expect("read compact DDL");
        assert!(
            ddl.contains("goalkeeper_oop IS NULL OR goalkeeper_oop BETWEEN 0 AND 100"),
            "new columns must carry the nullable 0-100 check"
        );
    }

    #[test]
    fn opening_fresh_db_creates_the_exact_compact_staff_role_metrics_inventory() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("compact-staff-metrics.db"));

        assert_eq!(
            table_columns(&conn, "staff_role_metrics"),
            [
                "snapshot_id",
                "uid",
                "score_model_version",
                "assistant_manager",
                "manager",
                "coach_attacking_technical",
                "coach_attacking_tactical",
                "coach_defending_technical",
                "coach_defending_tactical",
                "coach_possession_technical",
                "coach_possession_tactical",
                "coach_fitness",
                "coach_goalkeeping",
                "set_piece_coach",
                "loan_manager",
                "head_of_youth_development",
                "scout",
                "director_of_football",
                "technical_director",
                "recruitment_analyst",
                "head_performance_analyst",
                "performance_analyst",
                "physio",
                "sports_scientist",
            ]
        );
        assert_eq!(table_columns(&conn, "staff_role_metrics").len(), 24);
    }

    #[test]
    fn compact_role_metrics_enforce_identity_model_score_constraints_and_cascades() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("compact-constraints.db"));

        conn.execute_batch(
            "INSERT INTO saves (id, name, is_active) VALUES (1, 'Save', 1);
             INSERT INTO snapshots (
                 id, save_id, is_current, schema_version, generated_at_utc, game_version,
                 supported_game_version, bridge_version, protocol_version, game_date,
                 game_date_source, scan_truncated, max_accepted, player_count
             ) VALUES (
                 1, 1, 1, 8, '2026-08-16T00:00:00Z', '26.3', '26.3', '0.4.0', 1,
                 NULL, 'unavailable', 0, NULL, 0
             );",
        )
        .expect("seed compact owners");
        insert_player(&conn, 1, 1);
        insert_player(&conn, 1, 2);
        let insert_staff = "INSERT INTO staff (
             snapshot_id, uid, name, birth_year, birth_day_of_year, age,
             nationalities_json, nation_uid, gender, ca, pa, staff_attributes_json,
             job_id, weekly_wage_gbp, contract_expiry_year, contract_expiry_day_of_year,
             club, division
         ) VALUES (1, ?1, 'Staff', 1980, 1, 46, '[]', 208, 'male', 120, 150,
             '{}', 1, NULL, NULL, NULL, 'Club', 'Division')";
        conn.execute(insert_staff, [88]).expect("seed staff owner");
        conn.execute(insert_staff, [77])
            .expect("seed second staff owner");

        conn.execute(
            "INSERT INTO player_role_metrics (
                snapshot_id, uid, score_model_version, projection_model_version,
                goalkeeper_ip, potential_goalkeeper_ip
             ) VALUES (1, 1, 1, 2, NULL, 90)",
            [],
        )
        .expect("insert nullable compact player row");
        assert!(conn
            .execute(
                "INSERT INTO player_role_metrics (
                    snapshot_id, uid, score_model_version, projection_model_version
                 ) VALUES (1, 1, 1, 2)",
                [],
            )
            .is_err());
        assert!(conn
            .execute(
                "INSERT INTO player_role_metrics (
                    snapshot_id, uid, score_model_version, projection_model_version
                 ) VALUES (1, 2, 0, 2)",
                [],
            )
            .is_err());
        assert!(conn
            .execute(
                "INSERT INTO player_role_metrics (
                    snapshot_id, uid, score_model_version, projection_model_version
                 ) VALUES (1, 2, 1, 0)",
                [],
            )
            .is_err());
        assert!(conn
            .execute(
                "INSERT INTO player_role_metrics (
                    snapshot_id, uid, score_model_version, projection_model_version, goalkeeper_ip
                 ) VALUES (1, 2, 1, 2, 101)",
                [],
            )
            .is_err());
        assert!(conn
            .execute(
                "INSERT INTO player_role_metrics (
                    snapshot_id, uid, score_model_version, projection_model_version, goalkeeper_ip
                 ) VALUES (1, 2, 1, 2, -1)",
                [],
            )
            .is_err());

        conn.execute(
            "INSERT INTO staff_role_metrics (
                snapshot_id, uid, score_model_version, physio
             ) VALUES (1, 88, 1, NULL)",
            [],
        )
        .expect("insert nullable compact staff row");
        assert!(conn
            .execute(
                "INSERT INTO staff_role_metrics (
                    snapshot_id, uid, score_model_version
                 ) VALUES (1, 88, 1)",
                [],
            )
            .is_err());
        assert!(conn
            .execute(
                "INSERT INTO staff_role_metrics (
                    snapshot_id, uid, score_model_version, manager
                 ) VALUES (1, 77, 1, 101)",
                [],
            )
            .is_err());
        assert!(conn
            .execute(
                "INSERT INTO staff_role_metrics (
                    snapshot_id, uid, score_model_version
                 ) VALUES (1, 77, 0)",
                [],
            )
            .is_err());

        conn.execute("DELETE FROM players WHERE snapshot_id = 1 AND uid = 1", [])
            .expect("delete player owner");
        let player_rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM player_role_metrics", [], |row| {
                row.get(0)
            })
            .expect("count cascaded player metrics");
        assert_eq!(player_rows, 0);

        conn.execute("DELETE FROM staff WHERE snapshot_id = 1 AND uid = 88", [])
            .expect("delete staff owner");
        let staff_rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM staff_role_metrics", [], |row| {
                row.get(0)
            })
            .expect("count cascaded staff metrics");
        assert_eq!(staff_rows, 0);
    }

    #[test]
    fn v40_generic_oop_columns_reject_out_of_range_scores() {
        const NEW_COLUMNS: [&str; 22] = [
            "goalkeeper_oop",
            "centre_back_oop",
            "wide_centre_back_oop",
            "full_back_oop",
            "wing_back_oop",
            "defensive_midfielder_oop",
            "central_midfielder_oop",
            "wide_midfielder_oop",
            "attacking_midfielder_oop",
            "winger_oop",
            "centre_forward_oop",
            "potential_goalkeeper_oop",
            "potential_centre_back_oop",
            "potential_wide_centre_back_oop",
            "potential_full_back_oop",
            "potential_wing_back_oop",
            "potential_defensive_midfielder_oop",
            "potential_central_midfielder_oop",
            "potential_wide_midfielder_oop",
            "potential_attacking_midfielder_oop",
            "potential_winger_oop",
            "potential_centre_forward_oop",
        ];
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("v40-oop-checks.db"));

        conn.execute_batch(
            "INSERT INTO saves (id, name, is_active) VALUES (1, 'Save', 1);
             INSERT INTO snapshots (
                 id, save_id, is_current, schema_version, generated_at_utc, game_version,
                 supported_game_version, bridge_version, protocol_version, game_date,
                 game_date_source, scan_truncated, max_accepted, player_count
             ) VALUES (
                 1, 1, 1, 8, '2026-08-16T00:00:00Z', '26.3', '26.3', '0.4.0', 1,
                 NULL, 'unavailable', 0, NULL, 0
             );",
        )
        .expect("seed compact owners");
        for (index, column) in NEW_COLUMNS.iter().enumerate() {
            let uid = 100 + index as i64;
            insert_player(&conn, 1, uid);
            for invalid in [101, -1] {
                let error = conn
                    .execute(
                        &format!(
                            "INSERT INTO player_role_metrics (
                                snapshot_id, uid, score_model_version,
                                projection_model_version, {column}
                             ) VALUES (1, {uid}, 2, 2, {invalid})"
                        ),
                        [],
                    )
                    .expect_err(&format!("{column} must reject {invalid}"));
                assert!(
                    error.to_string().contains("CHECK"),
                    "{column} must reject {invalid} via its CHECK ({error})"
                );
            }
            conn.execute(
                &format!(
                    "INSERT INTO player_role_metrics (
                        snapshot_id, uid, score_model_version,
                        projection_model_version, {column}
                     ) VALUES (1, {uid}, 2, 2, NULL)"
                ),
                [],
            )
            .expect("NULL stays accepted");
        }
    }

    #[test]
    fn compact_role_metrics_add_no_per_role_indexes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("compact-no-index.db"));

        let explicit_indexes = conn
            .prepare(
                "SELECT name FROM sqlite_master
                 WHERE type = 'index'
                   AND tbl_name IN ('player_role_metrics', 'staff_role_metrics')
                   AND name NOT LIKE 'sqlite_autoindex%'
                 ORDER BY name",
            )
            .expect("prepare compact index inventory")
            .query_map([], |row| row.get::<_, String>(0))
            .expect("query compact index inventory")
            .collect::<Result<Vec<_>, _>>()
            .expect("read compact index inventory");

        assert!(
            explicit_indexes.is_empty(),
            "unexpected compact indexes: {explicit_indexes:?}"
        );
    }
}
