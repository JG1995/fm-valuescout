//! Snapshots with several players at chosen clubs and positions, and setters for the scores,
//! shortlists, and Moneyball percentiles the dashboard reads.

use rusqlite::{params, Connection};

use crate::db::migrations;
use crate::features::managed_club::service as managed_club_service;
use crate::features::moneyball::MONEYBALL_STATISTIC_KEYS;
use crate::features::planner::tactic::default_tactic;
use crate::features::player_metrics::compact::{player_current_column, player_potential_column};
use crate::features::snapshot;

pub(super) const HOME_CLUB: &str = "Home FC";
pub(super) const OTHER_CLUB: &str = "Other FC";

pub(super) struct Fixture {
    pub(super) _temp_dir: tempfile::TempDir,
    pub(super) conn: Connection,
    pub(super) save_id: i64,
    pub(super) snapshot_id: i64,
}

#[derive(Clone, Copy)]
pub(super) struct PlayerSpec {
    pub(super) uid: i64,
    pub(super) name: &'static str,
    pub(super) club: &'static str,
    /// The player's only position, at familiarity 20.
    pub(super) position: &'static str,
}

pub(super) const fn player(
    uid: i64,
    name: &'static str,
    club: &'static str,
    position: &'static str,
) -> PlayerSpec {
    PlayerSpec {
        uid,
        name,
        club,
        position,
    }
}

/// Ingests one snapshot holding `players` and makes `HOME_CLUB` the managed club. Every player
/// starts with no role scores, so a test sets exactly the scores it compares.
pub(super) fn open_with_players(players: &[PlayerSpec]) -> Fixture {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let mut conn = Connection::open(temp_dir.path().join("dashboard.db")).expect("open db");
    conn.pragma_update(None, "foreign_keys", true)
        .expect("enable foreign keys");
    migrations::apply(&conn).expect("apply migrations");
    let save_id = snapshot::service::active_save_id(&conn).expect("active save");

    let mut dump: serde_json::Value =
        serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v9.json"))
            .expect("parse golden dump");
    let template = dump["players"][0].clone();
    let records = players
        .iter()
        .map(|spec| {
            let mut record = template.clone();
            record["uid"] = spec.uid.into();
            record["name"] = spec.name.into();
            record["currentClub"] = spec.club.into();
            record["currentClubUid"] = if spec.club == HOME_CLUB { 1000 } else { 2000 }.into();
            record["parentClub"] = spec.club.into();
            record["parentClubUid"] = record["currentClubUid"].clone();
            record["onLoan"] = false.into();
            for (_, familiarity) in record["positions"]
                .as_object_mut()
                .expect("positions object")
            {
                *familiarity = serde_json::Value::Null;
            }
            record["positions"][spec.position] = 20.into();
            record
        })
        .collect::<Vec<_>>();
    dump["playerCount"] = records.len().into();
    dump["players"] = serde_json::Value::Array(records);
    let dump_path = temp_dir.path().join("dump.json");
    std::fs::write(
        &dump_path,
        serde_json::to_string(&dump).expect("serialize dump"),
    )
    .expect("write dump");
    let snapshot_id = snapshot::ingest::ingest_dump_file_for_save(&mut conn, save_id, &dump_path)
        .expect("ingest dump")
        .id;
    managed_club_service::set_managed_club(&conn, save_id, HOME_CLUB, Some(1000))
        .expect("configure managed club");

    let clear_scores = crate::features::scoring::catalog::all_roles()
        .iter()
        .flat_map(|role| {
            [
                player_current_column(role.role_id)
                    .expect("current column")
                    .to_string(),
                player_potential_column(role.role_id).expect("potential column"),
            ]
        })
        .map(|column| format!("{column} = NULL"))
        .collect::<Vec<_>>()
        .join(", ");
    conn.execute(
        &format!("UPDATE player_role_metrics SET {clear_scores} WHERE snapshot_id = ?1"),
        [snapshot_id],
    )
    .expect("clear role scores");

    Fixture {
        _temp_dir: temp_dir,
        conn,
        save_id,
        snapshot_id,
    }
}

impl Fixture {
    /// Sets both of a default-tactic slot's role scores, so the slot's blended score is `score`.
    pub(super) fn set_slot_score(&self, uid: i64, lane_id: &str, score: u8) {
        self.set_slot_columns(uid, lane_id, score, |role_id| {
            player_current_column(role_id)
                .expect("current column")
                .to_string()
        });
    }

    /// Sets a default-tactic slot's in-possession and out-of-possession role scores apart.
    pub(super) fn set_slot_role_scores(&self, uid: i64, lane_id: &str, ip: u8, oop: u8) {
        let tactic = default_tactic();
        let lane = tactic
            .lanes
            .iter()
            .find(|lane| lane.lane_id == lane_id)
            .expect("default tactic lane");
        let column = |role_id: &str| player_current_column(role_id).expect("current column");
        self.conn
            .execute(
                &format!(
                    "UPDATE player_role_metrics SET {} = ?1, {} = ?2
                     WHERE snapshot_id = ?3 AND uid = ?4",
                    column(&lane.ip_role_id),
                    column(&lane.oop_role_id)
                ),
                params![ip, oop, self.snapshot_id, uid],
            )
            .expect("set slot role scores");
    }

    pub(super) fn set_potential_slot_score(&self, uid: i64, lane_id: &str, score: u8) {
        self.set_slot_columns(uid, lane_id, score, |role_id| {
            player_potential_column(role_id).expect("potential column")
        });
    }

    fn set_slot_columns(&self, uid: i64, lane_id: &str, score: u8, column: fn(&str) -> String) {
        let tactic = default_tactic();
        let lane = tactic
            .lanes
            .iter()
            .find(|lane| lane.lane_id == lane_id)
            .expect("default tactic lane");
        let (ip_column, oop_column) = (column(&lane.ip_role_id), column(&lane.oop_role_id));
        self.conn
            .execute(
                &format!(
                    "UPDATE player_role_metrics SET {ip_column} = ?1, {oop_column} = ?1
                     WHERE snapshot_id = ?2 AND uid = ?3"
                ),
                params![score, self.snapshot_id, uid],
            )
            .expect("set slot score");
    }

    /// Sets one `players` column of the player's current-snapshot row.
    pub(super) fn set_player_column(&self, uid: i64, column: &str, value: impl rusqlite::ToSql) {
        self.conn
            .execute(
                &format!("UPDATE players SET {column} = ?1 WHERE snapshot_id = ?2 AND uid = ?3"),
                params![value, self.snapshot_id, uid],
            )
            .expect("set player column");
    }

    /// Records each league's measured strength for the current snapshot.
    pub(super) fn measure_leagues(&self, leagues: &[(&str, f64)]) {
        crate::features::moneyball::league_strength::test_support::measure_leagues(
            &self.conn,
            self.snapshot_id,
            leagues,
        );
    }

    /// Adds a shortlist and returns its ID.
    pub(super) fn add_shortlist(&self, name: &str, uids: &[i64]) -> i64 {
        self.conn
            .execute(
                "INSERT INTO player_shortlists (save_id, name) VALUES (?1, ?2)",
                params![self.save_id, name],
            )
            .expect("insert shortlist");
        let shortlist_id = self.conn.last_insert_rowid();
        for uid in uids {
            self.conn
                .execute(
                    "INSERT INTO player_shortlist_entries (shortlist_id, player_uid)
                     VALUES (?1, ?2)",
                    params![shortlist_id, uid],
                )
                .expect("insert shortlist entry");
        }
        shortlist_id
    }

    /// Gives the player season stats whose every stored percentile is `percentile`, apart
    /// from the listed overrides.
    pub(super) fn set_percentiles(&self, uid: i64, percentile: u8, overrides: &[(&str, u8)]) {
        let mut percentiles = MONEYBALL_STATISTIC_KEYS
            .iter()
            .map(|key| ((*key).to_string(), serde_json::Value::from(percentile)))
            .collect::<serde_json::Map<_, _>>();
        for (key, value) in overrides {
            percentiles.insert((*key).to_string(), (*value).into());
        }
        self.conn
            .execute(
                "INSERT INTO player_moneyball_stats (
                     snapshot_id, player_uid, minutes, statistics_json, percentiles_json
                 ) VALUES (?1, ?2, 900, '{}', ?3)",
                params![
                    self.snapshot_id,
                    uid,
                    serde_json::Value::Object(percentiles).to_string()
                ],
            )
            .expect("insert moneyball stats");
    }

    /// Assigns the player to a slot of the Senior first string, creating the string if needed.
    pub(super) fn assign_first_string(&self, lane_id: &str, uid: i64) {
        self.conn
            .execute(
                "INSERT INTO planner_strings (save_id, team, string_order, display_name)
                 SELECT ?1, 'senior', 0, '1st string'
                 WHERE NOT EXISTS (
                     SELECT 1 FROM planner_strings WHERE save_id = ?1 AND team = 'senior'
                 )",
                [self.save_id],
            )
            .expect("insert first string");
        self.conn
            .execute(
                "INSERT INTO planner_assignments (save_id, string_id, lane_id, player_uid, last_known_name)
                 SELECT ?1, id, ?2, ?3, 'Assigned'
                 FROM planner_strings
                 WHERE save_id = ?1 AND team = 'senior' AND string_order = 0",
                params![self.save_id, lane_id, uid],
            )
            .expect("insert assignment");
    }
}
