use std::cell::RefCell;

use rusqlite::{params, Connection};

use super::commands::read_planner_fm_moves;
use super::depth::{assign_player, get_depth, PlannerTeam};
use super::fm_moves::{
    apply_fm_squad_moves, plan_fm_squad_moves, FmSquadMoveFailure, LOAD_DATA_FIRST,
};
use super::test_support::{
    current_snapshot_id, loan_out, open_with_snapshot, seed_players, set_player_age, team_strings,
};
use crate::db::Db;
use crate::features::memory_read::service::{SquadMoveOutcome, SquadMoveResponse};
use crate::features::player::boost_gate::BOOST_TEST_GATE;

/// A squad of six at the managed club, each with an FM team level, and the Planner placements
/// that put four of them in squads: 81 is in no squad and 82 is placed but loaned out.
fn planned_squad() -> (tempfile::TempDir, Connection, i64) {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    seed_players(&temp_dir, &mut conn, save_id, &[77, 78, 79, 80, 81, 82]);
    for (uid, level) in [
        (77, Some("senior")),
        (78, Some("reserve")),
        (79, Some("youth")),
        (80, None),
        (81, Some("senior")),
        (82, Some("senior")),
    ] {
        set_team_level(&conn, save_id, uid, level);
    }
    set_player_age(&conn, save_id, 77, Some(25));
    set_player_age(&conn, save_id, 79, Some(17));
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let string_id = |team| team_strings(&depth, team)[0].id;
    for (uid, team, lane) in [
        (77, PlannerTeam::Reserves, "right_winger"),
        (78, PlannerTeam::Reserves, "left_winger"),
        (79, PlannerTeam::Senior, "right_winger"),
        (80, PlannerTeam::Youth, "right_winger"),
        (82, PlannerTeam::Youth, "left_winger"),
    ] {
        assign_player(&conn, save_id, string_id(team), lane, uid).expect("place player");
    }
    loan_out(&conn, save_id, 82);
    (temp_dir, conn, save_id)
}

fn set_team_level(conn: &Connection, save_id: i64, uid: i64, level: Option<&str>) {
    let team_type = match level {
        Some("senior") => Some(0),
        Some("reserve") => Some(10),
        Some("youth") => Some(12),
        _ => None,
    };
    conn.execute(
        "UPDATE players SET team_level = ?3, team_type = ?4 WHERE snapshot_id = ?1 AND uid = ?2",
        params![current_snapshot_id(conn, save_id), uid, level, team_type],
    )
    .expect("set team level");
}

fn team_level(conn: &Connection, save_id: i64, uid: i64) -> (Option<String>, Option<i64>) {
    conn.query_row(
        "SELECT team_level, team_type FROM players WHERE snapshot_id = ?1 AND uid = ?2",
        params![current_snapshot_id(conn, save_id), uid],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .expect("read team level")
}

fn record_live_scan(conn: &Connection, save_id: i64) {
    conn.execute(
        "UPDATE snapshots SET bridge_source_request_id = 'scan-1' WHERE id = ?1",
        [current_snapshot_id(conn, save_id)],
    )
    .expect("record live scan");
}

fn outcome(outcome: &str, team_type: Option<i32>, level: Option<&str>) -> SquadMoveOutcome {
    SquadMoveOutcome {
        outcome: outcome.to_string(),
        team_type,
        team_level: level.map(str::to_string),
    }
}

#[test]
fn plan_moves_each_placed_player_whose_fm_team_differs_from_his_squad() {
    let (_temp_dir, conn, save_id) = planned_squad();

    let plan = plan_fm_squad_moves(&conn, save_id).expect("plan moves");

    // Senior squad first. 78 is already in the reserves; 81 is in no squad; 82 is on loan.
    assert_eq!(
        plan.moves
            .iter()
            .map(|planned| (
                planned.player_uid,
                planned.from_level.as_str(),
                planned.to_team,
                planned.over_age
            ))
            .collect::<Vec<_>>(),
        vec![
            (79, "youth", PlannerTeam::Senior, false),
            (77, "senior", PlannerTeam::Reserves, true),
        ]
    );
    assert_eq!(plan.moves[1].to_team_name, "Reserves");
    assert_eq!(
        plan.unknown_team
            .iter()
            .map(|(uid, _)| *uid)
            .collect::<Vec<_>>(),
        vec![80]
    );
}

#[test]
fn apply_sends_only_chosen_moves_and_records_the_teams_fm_confirms() {
    let _test_guard = BOOST_TEST_GATE
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let (_temp_dir, conn, save_id) = planned_squad();
    record_live_scan(&conn, save_id);
    let db = Db::new(conn);
    let sent = RefCell::new(Vec::new());

    let report = apply_fm_squad_moves(&db, &[77, 79, 78, 999], |source, club, items| {
        sent.borrow_mut().push((
            source.to_string(),
            club,
            items
                .iter()
                .map(|item| (item.player_uid, item.target_level))
                .collect::<Vec<_>>(),
        ));
        Ok(SquadMoveResponse {
            moves: vec![
                outcome("moved", Some(0), Some("senior")),
                outcome("not-confirmed", Some(0), Some("senior")),
            ],
            error: None,
        })
    })
    .expect("apply moves");

    assert_eq!(
        sent.into_inner(),
        vec![(
            "scan-1".to_string(),
            1000,
            vec![(79, "senior"), (77, "reserve")]
        )]
    );
    assert_eq!(report.moved, vec![79]);
    assert_eq!(
        report.failed,
        vec![FmSquadMoveFailure {
            player_uid: 77,
            name: "Player 77".to_string(),
            reason: "FM did not confirm the move".to_string(),
        }]
    );
    let conn = db.0.lock().expect("lock db");
    assert_eq!(
        team_level(&conn, save_id, 79),
        (Some("senior".to_string()), Some(0))
    );
    assert_eq!(
        team_level(&conn, save_id, 77),
        (Some("senior".to_string()), Some(0))
    );
}

#[test]
fn apply_refuses_without_the_live_scan_or_a_move_to_make() {
    let _test_guard = BOOST_TEST_GATE
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let (_temp_dir, conn, save_id) = planned_squad();
    let db = Db::new(conn);
    let never = |_: &str, _: u32, _: &[_]| -> Result<SquadMoveResponse, String> {
        panic!("nothing may reach the bridge")
    };

    assert_eq!(
        apply_fm_squad_moves(&db, &[77], never),
        Err(LOAD_DATA_FIRST.to_string())
    );

    record_live_scan(&db.0.lock().expect("lock db"), save_id);
    assert_eq!(
        apply_fm_squad_moves(&db, &[78, 81], never),
        Err("None of the chosen players needs a move".to_string())
    );
}

#[test]
fn preview_says_why_moves_cannot_be_applied_before_asking_the_bridge() {
    let (_temp_dir, conn, save_id) = planned_squad();
    let db = Db::new(conn);

    let without_scan = read_planner_fm_moves(&db, || panic!("no bridge read without a live scan"))
        .expect("preview without a live scan");
    assert_eq!(
        without_scan.unavailable_reason.as_deref(),
        Some(LOAD_DATA_FIRST)
    );
    assert_eq!(without_scan.moves.len(), 2);

    record_live_scan(&db.0.lock().expect("lock db"), save_id);
    let bridge_closed = read_planner_fm_moves(&db, || Some("bridge closed".to_string()))
        .expect("preview with the bridge closed");
    assert_eq!(
        bridge_closed.unavailable_reason.as_deref(),
        Some("bridge closed")
    );

    let ready = read_planner_fm_moves(&db, || None).expect("preview ready to apply");
    assert_eq!(ready.unavailable_reason, None);
    assert_eq!(ready.moves[0].to_team, "senior");
    assert_eq!(ready.unknown_team_players[0].player_uid, 80);
}
