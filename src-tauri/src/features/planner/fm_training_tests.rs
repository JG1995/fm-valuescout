use std::cell::RefCell;

use rusqlite::{params, Connection};

use super::commands::read_squad_fm_training;
use super::depth::{assign_player, get_depth, PlannerTeam};
use super::fm_training::{apply_fm_training, plan_fm_training, FmTrainingFailure, LOAD_DATA_FIRST};
use super::squad::{list_squad_players, SquadSortDir, SquadSortField};
use super::test_support::{
    current_snapshot_id, loan_out, open_with_snapshot, seed_players, set_player_age,
    set_player_attributes, team_strings,
};
use crate::db::Db;
use crate::features::memory_read::service::{TrainingOutcome, TrainingResponse};
use crate::features::player::boost_gate::BOOST_TEST_GATE;

/// Five copies of the golden right winger at the managed club: 77 in the right-winger slot and 78
/// in the goalkeeper slot, 79 in no slot, 80 too old to develop, and 81 placed but loaned out.
fn squad() -> (tempfile::TempDir, Connection, i64) {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    seed_players(&temp_dir, &mut conn, save_id, &[77, 78, 79, 80, 81]);
    let attributes = serde_json::Value::Object(
        crate::features::scoring::catalog::DUMP_ATTRIBUTE_KEYS
            .iter()
            .map(|key| ((*key).to_string(), serde_json::Value::from(10)))
            .collect(),
    )
    .to_string();
    for uid in [77, 78, 79, 80, 81] {
        set_player_attributes(&conn, save_id, uid, &attributes);
    }
    for uid in [77, 78, 79, 81] {
        set_player_age(&conn, save_id, uid, Some(22));
    }
    set_player_age(&conn, save_id, 80, Some(30));
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let senior = team_strings(&depth, PlannerTeam::Senior)[0].id;
    for (uid, lane) in [
        (77, "right_winger"),
        (78, "goalkeeper"),
        (81, "left_winger"),
    ] {
        assign_player(&conn, save_id, senior, lane, uid).expect("place player");
    }
    loan_out(&conn, save_id, 81);
    (temp_dir, conn, save_id)
}

fn record_live_scan(conn: &Connection, save_id: i64) {
    conn.execute(
        "UPDATE snapshots SET bridge_source_request_id = 'scan-1' WHERE id = ?1",
        [current_snapshot_id(conn, save_id)],
    )
    .expect("record live scan");
}

/// The Suggested Training the Squad overview shows for each player.
fn squad_column(conn: &Connection, save_id: i64) -> Vec<(i64, Option<String>)> {
    list_squad_players(
        conn,
        save_id,
        0,
        200,
        SquadSortField::Name,
        SquadSortDir::Asc,
        &[],
    )
    .expect("squad page")
    .players
    .into_iter()
    .filter(|player| !player.loaned_out)
    .map(|player| (player.uid, player.suggested_training))
    .collect()
}

#[test]
fn plan_trains_each_slots_roles_and_the_squad_overviews_focus() {
    let (_temp_dir, conn, save_id) = squad();

    let plan = plan_fm_training(&conn, save_id).expect("plan training");

    // 81 is on loan, so he trains with the borrower.
    assert_eq!(
        plan.iter()
            .map(|planned| (planned.player_uid, planned.focus.clone()))
            .collect::<Vec<_>>(),
        squad_column(&conn, save_id)
    );
    assert!(plan[0].focus.is_some(), "a developing winger has a focus");
    assert_eq!(plan[3].focus, None, "a 30-year-old has his focus cleared");
    let roles = plan
        .iter()
        .map(|planned| {
            planned.roles.as_ref().map(|roles| {
                (
                    roles.position.as_str(),
                    roles.in_possession.as_str(),
                    roles.out_of_possession.as_str(),
                )
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(
        roles,
        vec![
            Some(("AMR", "Winger", "Tracking Wide Midfielder")),
            Some(("GK", "Goalkeeper", "Line-Holding Keeper")),
            None,
            None,
        ]
    );
}

#[test]
fn apply_sends_fm_ids_and_reports_what_fm_did_not_take() {
    let _test_guard = BOOST_TEST_GATE
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let (_temp_dir, conn, save_id) = squad();
    record_live_scan(&conn, save_id);
    let focus_ids = plan_fm_training(&conn, save_id)
        .expect("plan")
        .iter()
        .map(|planned| {
            planned
                .focus
                .as_deref()
                .map_or(Some(-1), super::suggested_training::fm_focus_id)
        })
        .collect::<Vec<_>>();
    let db = Db::new(conn);
    let sent = RefCell::new(Vec::new());

    let report = apply_fm_training(&db, &[80, 77, 78, 81, 999], |source, club, items| {
        sent.borrow_mut().push((
            source.to_string(),
            club,
            items
                .iter()
                .map(|item| {
                    (
                        item.player_uid,
                        item.ip_position,
                        item.ip_role,
                        item.oop_role,
                        item.focus,
                    )
                })
                .collect::<Vec<_>>(),
        ));
        Ok(TrainingResponse {
            players: vec![
                TrainingOutcome {
                    outcome: "applied".to_string(),
                    ..TrainingOutcome::default()
                },
                TrainingOutcome {
                    outcome: "not-confirmed".to_string(),
                    ip_position: Some(1),
                    ip_role: Some(1),
                    oop_role: Some(-31),
                    oop_role_name: Some("Goalkeeper".to_string()),
                    focus: focus_ids[1],
                    ..TrainingOutcome::default()
                },
                TrainingOutcome {
                    outcome: "not-confirmed".to_string(),
                    focus: Some(23),
                    focus_name: Some("Passing".to_string()),
                    ..TrainingOutcome::default()
                },
            ],
            error: None,
        })
    })
    .expect("apply training");

    assert_eq!(
        sent.into_inner(),
        vec![(
            "scan-1".to_string(),
            1000,
            vec![
                (77, Some(2048), Some(128), Some(-2), focus_ids[0]),
                (78, Some(1), Some(1), Some(-29), focus_ids[1]),
                (80, None, None, None, Some(-1)),
            ]
        )]
    );
    assert_eq!(report.applied, vec![77]);
    assert_eq!(
        report.failed,
        vec![
            FmTrainingFailure {
                player_uid: 78,
                name: "Player 78".to_string(),
                reason: "FM did not take the out-of-possession role (FM shows Goalkeeper)"
                    .to_string(),
            },
            FmTrainingFailure {
                player_uid: 80,
                name: "Player 80".to_string(),
                reason: "FM did not take the focus (FM shows Passing)".to_string(),
            },
        ]
    );
}

#[test]
fn apply_refuses_without_the_live_scan_or_a_squad_player() {
    let _test_guard = BOOST_TEST_GATE
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let (_temp_dir, conn, save_id) = squad();
    let db = Db::new(conn);
    let never = |_: &str, _: u32, _: &[_]| -> Result<TrainingResponse, String> {
        panic!("nothing may reach the bridge")
    };

    assert_eq!(
        apply_fm_training(&db, &[77], never),
        Err(LOAD_DATA_FIRST.to_string())
    );

    record_live_scan(&db.0.lock().expect("lock db"), save_id);
    assert_eq!(
        apply_fm_training(&db, &[81, 999], never),
        Err("None of the chosen players is in the squad".to_string())
    );
}

#[test]
fn preview_says_why_training_cannot_be_set_before_asking_the_bridge() {
    let (_temp_dir, conn, save_id) = squad();
    let db = Db::new(conn);

    let without_scan = read_squad_fm_training(&db, || panic!("no bridge read without a live scan"))
        .expect("preview without a live scan");
    assert_eq!(
        without_scan.unavailable_reason.as_deref(),
        Some(LOAD_DATA_FIRST)
    );
    assert_eq!(without_scan.players.len(), 4);

    record_live_scan(&db.0.lock().expect("lock db"), save_id);
    let bridge_closed = read_squad_fm_training(&db, || Some("bridge closed".to_string()))
        .expect("preview with the bridge closed");
    assert_eq!(
        bridge_closed.unavailable_reason.as_deref(),
        Some("bridge closed")
    );

    let ready = read_squad_fm_training(&db, || None).expect("preview ready to apply");
    assert_eq!(ready.unavailable_reason, None);
    assert_eq!(ready.players[1].position.as_deref(), Some("GK"));
    assert_eq!(ready.players[2].in_possession_role, None);
}

#[test]
fn a_save_without_a_tactic_sets_only_focuses() {
    let (_temp_dir, conn, save_id) = squad();
    conn.execute(
        "DELETE FROM planner_tactic_lanes WHERE save_id = ?1",
        params![save_id],
    )
    .expect("drop tactic");

    let plan = plan_fm_training(&conn, save_id).expect("plan without a tactic");

    assert!(plan.iter().all(|planned| planned.roles.is_none()));
    assert!(plan.iter().all(|planned| planned.focus.is_none()));
}
