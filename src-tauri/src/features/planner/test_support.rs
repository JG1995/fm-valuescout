use rusqlite::{params, Connection};

use crate::db::migrations;
use crate::features::planner::service::{self, ClubSourceInput};
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
        include_str!("../memory_read/fixtures/golden_dump_v7.json"),
    )
    .expect("write dump");
    snapshot::ingest::ingest_dump_file(&mut conn, &dump_path).expect("ingest dump");
    service::save_club_family(&conn, save.id, "Loan FC", &[]).expect("configure club family");
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
    conn.execute(
        "UPDATE player_role_scores
         SET score = ?1
         WHERE snapshot_id = ?2
           AND uid = ?3
           AND role_id IN ('winger_ip', 'tracking_wide_midfielder_oop')",
        params![score, snapshot_id, player_uid],
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
    conn.execute(
        "UPDATE player_role_scores
         SET score = ?1
         WHERE snapshot_id = ?2 AND uid = ?3 AND role_id = ?4",
        params![
            score,
            current_snapshot_id(conn, save_id),
            player_uid,
            role_id
        ],
    )
    .expect("set role score");
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
        serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v7.json"))
            .expect("parse golden dump");
    let original = dump["players"][0].clone();
    let mut reserve = original.clone();
    reserve["uid"] = serde_json::Value::Number(78.into());
    reserve["name"] = serde_json::Value::String("Reserve Player".to_string());
    reserve["teamLevel"] = serde_json::Value::String("reserve".to_string());
    let mut b_team = original.clone();
    b_team["uid"] = serde_json::Value::Number(79.into());
    b_team["name"] = serde_json::Value::String("B Team Player".to_string());
    b_team["currentClub"] = serde_json::Value::String("Loan B FC".to_string());
    let mut unknown = b_team.clone();
    unknown["uid"] = serde_json::Value::Number(80.into());
    unknown["name"] = serde_json::Value::String("Unknown Score Player".to_string());
    dump["players"] = serde_json::Value::Array(vec![original, reserve, b_team, unknown]);
    dump["playerCount"] = serde_json::Value::Number(4.into());
    std::fs::write(
        &dump_path,
        serde_json::to_string(&dump).expect("serialize picker candidates"),
    )
    .expect("write picker candidates");
    snapshot::ingest::ingest_dump_file_for_save(conn, save_id, &dump_path)
        .expect("ingest picker candidates");
    service::save_club_family(
        conn,
        save_id,
        "Loan FC",
        &[ClubSourceInput {
            team: "reserves".to_string(),
            club_name: "Loan B FC".to_string(),
            team_level: None,
        }],
    )
    .expect("configure B-team source");
}
