use rusqlite::{params, types::Value, Connection};

use crate::db::migrations;
use crate::features::managed_club::service as managed_club_service;
use crate::features::player_metrics::compact::{player_current_column, player_potential_column};
use crate::features::snapshot;

use super::depth::{PlannerDepth, PlannerString, PlannerTeam};

pub(super) fn open_with_snapshot() -> (tempfile::TempDir, Connection, i64) {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let mut conn = Connection::open(temp_dir.path().join("planner-depth.db")).expect("open db");
    conn.pragma_update(None, "foreign_keys", true)
        .expect("enable foreign keys");
    migrations::apply(&conn).expect("apply migrations");
    let save = snapshot::service::list_saves(&conn)
        .expect("seed default save")
        .into_iter()
        .find(|save| save.is_active)
        .expect("active save");
    let dump_path = temp_dir.path().join("dump.json");
    std::fs::write(
        &dump_path,
        include_str!("../memory_read/fixtures/golden_dump_v8.json"),
    )
    .expect("write dump");
    snapshot::ingest::ingest_dump_file(&mut conn, &dump_path).expect("ingest dump");
    managed_club_service::set_managed_club(&conn, save.id, "Loan FC")
        .expect("configure managed club");
    (temp_dir, conn, save.id)
}

pub(super) fn team_strings(depth: &PlannerDepth, team: PlannerTeam) -> &[PlannerString] {
    &depth
        .teams
        .iter()
        .find(|team_depth| team_depth.team == team)
        .expect("team depth")
        .strings
}

pub(super) fn assignment_provenance(conn: &Connection, player_uid: i64) -> String {
    conn.query_row(
        "SELECT provenance FROM planner_assignments WHERE player_uid = ?1",
        [player_uid],
        |row| row.get(0),
    )
    .expect("read assignment provenance")
}

pub(super) fn current_snapshot_id(conn: &Connection, save_id: i64) -> i64 {
    conn.query_row(
        "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
        [save_id],
        |row| row.get(0),
    )
    .expect("current snapshot")
}

pub(super) fn set_right_winger_scores(
    conn: &Connection,
    save_id: i64,
    player_uid: i64,
    score: Option<u8>,
) {
    let snapshot_id = current_snapshot_id(conn, save_id);
    let winger_ip = player_current_column("winger_ip").expect("winger ip column");
    let tracking = player_current_column("tracking_wide_midfielder_oop").expect("tracking column");
    conn.execute(
        &format!(
            "UPDATE player_role_metrics
             SET {winger_ip} = ?1, {tracking} = ?2
             WHERE snapshot_id = ?3 AND uid = ?4"
        ),
        params![score, score, snapshot_id, player_uid],
    )
    .expect("set right-winger scores");
}

pub(super) fn set_role_score(
    conn: &Connection,
    save_id: i64,
    player_uid: i64,
    role_id: &str,
    score: Option<u8>,
) {
    let snapshot_id = current_snapshot_id(conn, save_id);
    let column = player_current_column(role_id).expect("current role column");
    conn.execute(
        &format!(
            "UPDATE player_role_metrics
             SET {column} = ?1
             WHERE snapshot_id = ?2 AND uid = ?3"
        ),
        params![score, snapshot_id, player_uid],
    )
    .expect("set role score");
}

pub(super) fn set_potential_role_score(
    conn: &Connection,
    save_id: i64,
    player_uid: i64,
    role_id: &str,
    score: Option<u8>,
) {
    let snapshot_id = current_snapshot_id(conn, save_id);
    let column = player_potential_column(role_id).expect("potential role column");
    conn.execute(
        &format!(
            "UPDATE player_role_metrics
             SET {column} = ?1
             WHERE snapshot_id = ?2 AND uid = ?3"
        ),
        params![score, snapshot_id, player_uid],
    )
    .expect("set potential role score");
}

/// Nulls every compact current column except `keep_roles` for the players,
/// mirroring the normalized per-role cleanup the optimizer tests used.
pub(super) fn clear_current_scores_except(
    conn: &Connection,
    save_id: i64,
    player_uids: &[i64],
    keep_roles: &[&str],
) {
    let snapshot_id = current_snapshot_id(conn, save_id);
    let columns = crate::features::scoring::catalog::all_roles()
        .iter()
        .filter(|role| !keep_roles.contains(&role.role_id))
        .map(|role| {
            player_current_column(role.role_id)
                .expect("current role column")
                .to_string()
        })
        .collect::<Vec<_>>();
    let uid_placeholders = (0..player_uids.len())
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "UPDATE player_role_metrics
         SET {}
         WHERE snapshot_id = ?1 AND uid IN ({uid_placeholders})",
        columns
            .iter()
            .map(|column| format!("{column} = NULL"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let mut values = vec![Value::Integer(snapshot_id)];
    values.extend(player_uids.iter().map(|uid| Value::Integer(*uid)));
    conn.execute(&sql, rusqlite::params_from_iter(values.iter()))
        .expect("clear non-lane current scores");
}

pub(super) fn set_player_age(conn: &Connection, save_id: i64, player_uid: i64, age: Option<i64>) {
    conn.execute(
        "UPDATE players SET age = ?1 WHERE snapshot_id = ?2 AND uid = ?3",
        params![age, current_snapshot_id(conn, save_id), player_uid],
    )
    .expect("set player age");
}

pub(super) fn set_player_positions(
    conn: &Connection,
    save_id: i64,
    player_uid: i64,
    positions_json: &str,
) {
    conn.execute(
        "UPDATE players SET positions_json = ?1 WHERE snapshot_id = ?2 AND uid = ?3",
        params![
            positions_json,
            current_snapshot_id(conn, save_id),
            player_uid
        ],
    )
    .expect("set player positions");
}

pub(super) fn set_player_attributes(
    conn: &Connection,
    save_id: i64,
    player_uid: i64,
    attributes_json: &str,
) {
    conn.execute(
        "UPDATE players SET attributes_json = ?1 WHERE snapshot_id = ?2 AND uid = ?3",
        params![
            attributes_json,
            current_snapshot_id(conn, save_id),
            player_uid
        ],
    )
    .expect("set player attributes");
}

pub(super) fn set_player_preferred_foot(
    conn: &Connection,
    save_id: i64,
    player_uid: i64,
    preferred_foot: &str,
) {
    conn.execute(
        "UPDATE players SET preferred_foot = ?1 WHERE snapshot_id = ?2 AND uid = ?3",
        params![
            preferred_foot,
            current_snapshot_id(conn, save_id),
            player_uid
        ],
    )
    .expect("set player preferred foot");
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct PlannerPotentialState {
    pub(super) teams: Vec<(String, String)>,
    pub(super) strings: Vec<(i64, String, i64)>,
    pub(super) assignments: Vec<(i64, i64, String, i64, String, String)>,
    pub(super) projections: Vec<(i64, Option<String>, Option<i64>)>,
    pub(super) compact_rows: Vec<(
        i64,
        Option<crate::features::player_metrics::compact::test_support::CompactRowShape>,
    )>,
}

pub(super) fn planner_potential_state(
    conn: &Connection,
    save_id: i64,
    snapshot_id: i64,
) -> PlannerPotentialState {
    let teams = conn
        .prepare("SELECT team, display_name FROM planner_teams WHERE save_id = ?1 ORDER BY team")
        .expect("prepare teams")
        .query_map([save_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("query teams")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect teams");
    let strings = conn
        .prepare(
            "SELECT id, team, string_order
             FROM planner_strings WHERE save_id = ?1 ORDER BY id",
        )
        .expect("prepare strings")
        .query_map([save_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .expect("query strings")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect strings");
    let assignments = conn
        .prepare(
            "SELECT id, string_id, lane_id, player_uid, last_known_name, provenance
             FROM planner_assignments WHERE save_id = ?1 ORDER BY id",
        )
        .expect("prepare assignments")
        .query_map([save_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })
        .expect("query assignments")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect assignments");
    let projections = conn
        .prepare(
            "SELECT uid, potential_attributes_json, potential_projection_model_version
             FROM players WHERE snapshot_id = ?1 ORDER BY uid",
        )
        .expect("prepare projections")
        .query_map([snapshot_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .expect("query projections")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect projections");
    let compact_rows = conn
        .prepare("SELECT uid FROM players WHERE snapshot_id = ?1 ORDER BY uid")
        .expect("prepare current player uids")
        .query_map([snapshot_id], |row| row.get::<_, i64>(0))
        .expect("query current player uids")
        .collect::<Result<Vec<_>, _>>()
        .expect("read current player uids")
        .into_iter()
        .map(|uid| {
            (
                uid,
                crate::features::player_metrics::compact::test_support::read_row(
                    conn,
                    snapshot_id,
                    uid,
                ),
            )
        })
        .collect();
    PlannerPotentialState {
        teams,
        strings,
        assignments,
        projections,
        compact_rows,
    }
}

pub(super) fn deny_potential_writes(conn: &Connection) {
    conn.execute_batch(
        "CREATE TRIGGER deny_potential_player_update
         BEFORE UPDATE OF potential_attributes_json, potential_projection_model_version ON players
         BEGIN SELECT RAISE(ABORT, 'potential writes are forbidden'); END;
         CREATE TRIGGER deny_compact_role_inserts
         BEFORE INSERT ON player_role_metrics
         BEGIN SELECT RAISE(ABORT, 'compact role writes are forbidden'); END;
         CREATE TRIGGER deny_compact_role_updates
         BEFORE UPDATE ON player_role_metrics
         BEGIN SELECT RAISE(ABORT, 'compact role writes are forbidden'); END;
         CREATE TRIGGER deny_compact_role_deletes
         BEFORE DELETE ON player_role_metrics
         BEGIN SELECT RAISE(ABORT, 'compact role writes are forbidden'); END;",
    )
    .expect("deny potential writes");
}

pub(super) fn assigned_player_uid(
    depth: &PlannerDepth,
    team: PlannerTeam,
    lane_id: &str,
) -> Option<i64> {
    team_strings(depth, team)
        .iter()
        .flat_map(|planner_string| &planner_string.assignments)
        .find(|assignment| assignment.lane_id == lane_id)
        .map(|assignment| assignment.player_uid)
}

pub(super) fn add_picker_candidates(
    temp_dir: &tempfile::TempDir,
    conn: &mut Connection,
    save_id: i64,
) {
    let dump_path = temp_dir.path().join("picker-candidates.json");
    let mut dump: serde_json::Value =
        serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v8.json"))
            .expect("parse golden dump");
    let original = dump["players"][0].clone();
    let mut reserve = original.clone();
    reserve["uid"] = serde_json::Value::Number(78.into());
    reserve["name"] = serde_json::Value::String("Reserve Player".to_string());
    reserve["teamLevel"] = serde_json::Value::String("senior".to_string());
    let mut b_team = original.clone();
    b_team["uid"] = serde_json::Value::Number(79.into());
    b_team["name"] = serde_json::Value::String("Second Reserve Player".to_string());
    b_team["teamLevel"] = serde_json::Value::String("reserve".to_string());
    let mut unknown = b_team.clone();
    unknown["uid"] = serde_json::Value::Number(80.into());
    unknown["name"] = serde_json::Value::String("Youth Player".to_string());
    unknown["teamLevel"] = serde_json::Value::String("youth".to_string());
    dump["players"] = serde_json::Value::Array(vec![original, reserve, b_team, unknown]);
    dump["playerCount"] = serde_json::Value::Number(4.into());
    std::fs::write(
        &dump_path,
        serde_json::to_string(&dump).expect("serialize picker candidates"),
    )
    .expect("write picker candidates");
    snapshot::ingest::ingest_dump_file_for_save(conn, save_id, &dump_path)
        .expect("ingest picker candidates");
    managed_club_service::set_managed_club(conn, save_id, "Loan FC")
        .expect("configure managed club");
}
