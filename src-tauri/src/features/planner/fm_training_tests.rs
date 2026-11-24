use std::cell::RefCell;

use rusqlite::{params, Connection};

use super::commands::read_squad_fm_training;
use super::depth::{assign_player, get_depth, PlannerTeam};
use super::fm_training::{
    apply_fm_training, plan_fm_training, FmTrainingFailure, LearningPosition, LOAD_DATA_FIRST,
};
use super::squad::{list_squad_players, SquadSortDir, SquadSortField};
use super::test_support::{
    current_snapshot_id, loan_out, open_with_snapshot, seed_players, set_player_age,
    set_player_attributes, set_player_positions, team_strings,
};
use crate::db::Db;
use crate::features::memory_read::service::{TrainingOutcome, TrainingResponse};
use crate::features::player::boost_gate::BOOST_TEST_GATE;

/// Five copies of the golden right winger at the managed club: 77 in the right-winger slot and 78
/// in the goalkeeper slot, 79 in no slot, 80 too old to develop, and 81 placed but loaned out.
/// The winger is natural at AMR (20) and accomplished at MR (17), the slot's two positions.
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

/// The position, in-possession role, and out-of-possession role a player trains; `None` for a
/// player in no slot.
type TrainedRoles = Option<(String, String, String)>;

fn trained_roles(conn: &Connection, save_id: i64) -> Vec<(i64, TrainedRoles)> {
    plan_fm_training(conn, save_id)
        .expect("plan training")
        .into_iter()
        .map(|planned| {
            (
                planned.player_uid,
                planned.slot.map(|slot| {
                    (
                        slot.roles.position,
                        slot.roles.in_possession,
                        slot.roles.out_of_possession,
                    )
                }),
            )
        })
        .collect()
}

fn roles(position: &str, in_possession: &str, out_of_possession: &str) -> TrainedRoles {
    Some((
        position.to_string(),
        in_possession.to_string(),
        out_of_possession.to_string(),
    ))
}

#[test]
fn plan_trains_each_slots_roles_and_the_squad_overviews_focus() {
    let (_temp_dir, conn, save_id) = squad();

    let plan = plan_fm_training(&conn, save_id).expect("plan training");

    // 81 is on loan, so he trains with the borrower.
    let mut focuses = plan
        .iter()
        .map(|planned| (planned.player_uid, planned.focus.clone()))
        .collect::<Vec<_>>();
    focuses.sort();
    assert_eq!(focuses, squad_column(&conn, save_id));
    assert!(focuses[0].1.is_some(), "a developing winger has a focus");
    assert_eq!(focuses[3].1, None, "a 30-year-old has his focus cleared");
    // Planner order: the goalkeeper slot comes before the right winger's, then the players in
    // no slot by name.
    assert_eq!(
        trained_roles(&conn, save_id),
        vec![
            (78, roles("GK", "Goalkeeper", "Line-Holding Keeper")),
            (77, roles("AMR", "Winger", "Tracking Wide Midfielder")),
            (79, None),
            (80, None),
        ]
    );
    let slot = plan[1].slot.as_ref().expect("77 is in a slot");
    assert_eq!(
        (
            slot.team,
            slot.team_name.as_str(),
            slot.string_name.as_str(),
            slot.ip_position.as_str(),
            slot.oop_position.as_str(),
            slot.in_possession.as_str(),
        ),
        (
            PlannerTeam::Senior,
            "Senior",
            "Starter",
            "AMR",
            "MR",
            "Winger"
        )
    );
}

/// What 77 trains in the right-winger slot with these positions, and the position he is learning.
fn winger_training(
    conn: &Connection,
    save_id: i64,
    positions_json: &str,
) -> (TrainedRoles, Option<LearningPosition>) {
    set_player_positions(conn, save_id, 77, positions_json);
    let planned = plan_fm_training(conn, save_id)
        .expect("plan training")
        .into_iter()
        .find(|planned| planned.player_uid == 77)
        .expect("77 is planned");
    let slot = planned.slot.expect("77 is in a slot");
    (
        Some((
            slot.roles.position,
            slot.roles.in_possession,
            slot.roles.out_of_possession,
        )),
        slot.learning,
    )
}

/// The right-winger slot plays AMR in possession and MR out of possession.
fn learning(position: &str, familiarity: i64) -> Option<LearningPosition> {
    Some(LearningPosition {
        position: position.to_string(),
        familiarity,
        out_of_possession: position == "MR",
    })
}

#[test]
fn a_player_trains_the_slot_position_he_is_not_fully_familiar_with() {
    let (_temp_dir, conn, save_id) = squad();
    let slot_roles = roles("AMR", "Winger", "Tracking Wide Midfielder");
    // FM builds familiarity only where the in-possession role is trained.
    let at_oop_position = roles("MR", "Winger", "Tracking Wide Midfielder");

    assert_eq!(
        winger_training(&conn, save_id, r#"{"AMR": 20, "MR": 16}"#),
        (slot_roles.clone(), None),
        "16 is fully familiar"
    );
    assert_eq!(
        winger_training(&conn, save_id, r#"{"AMR": 20, "MR": 15}"#),
        (at_oop_position.clone(), learning("MR", 15))
    );
    assert_eq!(
        winger_training(&conn, save_id, r#"{"AMR": 16, "MR": null}"#),
        (at_oop_position, learning("MR", 0)),
        "a position he has never played"
    );
    assert_eq!(
        winger_training(&conn, save_id, r#"{"AMR": 15, "MR": 20}"#),
        (slot_roles.clone(), learning("AMR", 15))
    );
    assert_eq!(
        winger_training(&conn, save_id, r#"{"AMR": 10, "MR": 9}"#),
        (slot_roles, learning("AMR", 10)),
        "the in-possession position comes first"
    );
}

#[test]
fn a_role_the_position_does_not_have_gives_way_to_the_closest_one_there() {
    let (_temp_dir, conn, save_id) = squad();
    // Inside Forward plays only at AML and AMR.
    conn.execute(
        "UPDATE planner_tactic_lanes SET ip_role_id = 'inside_forward_ip'
         WHERE save_id = ?1 AND lane_id = 'right_winger'",
        params![save_id],
    )
    .expect("change the slot's role");

    assert_eq!(
        winger_training(&conn, save_id, r#"{"AMR": 20, "MR": 12}"#),
        (
            roles("MR", "Inside Winger", "Tracking Wide Midfielder"),
            learning("MR", 12)
        )
    );
}

#[test]
fn apply_sends_fm_ids_and_reports_what_fm_did_not_take() {
    let _test_guard = BOOST_TEST_GATE
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let (_temp_dir, conn, save_id) = squad();
    record_live_scan(&conn, save_id);
    // 77 learns MR, so FM gets his in-possession role at MR (256) instead of AMR (2048).
    set_player_positions(&conn, save_id, 77, r#"{"AMR": 20, "MR": 9}"#);
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
                    outcome: "not-confirmed".to_string(),
                    ip_position: Some(1),
                    ip_role: Some(1),
                    oop_role: Some(-31),
                    oop_role_name: Some("Goalkeeper".to_string()),
                    focus: focus_ids[0],
                    ..TrainingOutcome::default()
                },
                TrainingOutcome {
                    outcome: "applied".to_string(),
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
                (78, Some(1), Some(1), Some(-29), focus_ids[0]),
                (77, Some(256), Some(128), Some(-2), focus_ids[1]),
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
    assert_eq!(ready.players[0].position.as_deref(), Some("GK"));
    assert_eq!(
        ready.players[0]
            .learning
            .as_ref()
            .map(|learning| (learning.position.as_str(), learning.familiarity)),
        Some(("GK", 0)),
        "a winger in goal learns the position"
    );
    assert_eq!(
        ready.players[1]
            .slot
            .as_ref()
            .map(|slot| (slot.team, slot.string_name.as_str())),
        Some(("senior", "Starter"))
    );
    assert_eq!(ready.players[2].slot.as_ref().map(|slot| slot.team), None);
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

    assert!(plan.iter().all(|planned| planned.slot.is_none()));
    assert!(plan.iter().all(|planned| planned.focus.is_none()));
}
