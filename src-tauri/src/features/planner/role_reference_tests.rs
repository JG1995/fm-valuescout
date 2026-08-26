use std::collections::HashSet;

use rusqlite::params;

use super::role_reference::{get_role_reference, RoleReferenceBasis, RoleReferencePhase};
use super::tactic;
use super::test_support::{
    add_picker_candidates, current_snapshot_id, deny_potential_writes, open_with_snapshot,
    planner_potential_state, set_player_attributes, set_player_positions,
    set_player_preferred_foot, set_right_winger_scores, set_role_score,
};

fn seeded_snapshot() -> (tempfile::TempDir, rusqlite::Connection, i64) {
    let (temp_dir, conn, save_id) = open_with_snapshot();
    tactic::save_tactic(&conn, save_id, &tactic::default_tactic()).expect("seed tactic");
    (temp_dir, conn, save_id)
}

fn clear_current_scores(conn: &rusqlite::Connection, save_id: i64) {
    conn.execute(
        "UPDATE player_role_scores SET score = NULL WHERE snapshot_id = ?1",
        [current_snapshot_id(conn, save_id)],
    )
    .expect("clear current scores");
}

fn full_attributes(value: u8) -> String {
    let attributes = crate::features::scoring::catalog::DUMP_ATTRIBUTE_KEYS
        .iter()
        .map(|key| ((*key).to_string(), serde_json::Value::from(value)))
        .collect::<serde_json::Map<_, _>>();
    serde_json::to_string(&serde_json::Value::Object(attributes)).expect("serialize attributes")
}

fn attributes_with(default: u8, overrides: &[(&str, u8)]) -> String {
    let mut attributes = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(
        &full_attributes(default),
    )
    .expect("parse attributes");
    for (key, value) in overrides {
        attributes.insert((*key).to_string(), serde_json::Value::from(*value));
    }
    serde_json::to_string(&serde_json::Value::Object(attributes)).expect("serialize attributes")
}

fn lane<'a>(
    reference: &'a super::role_reference::RoleReference,
    lane_id: &str,
) -> &'a super::role_reference::RoleReferenceLane {
    reference
        .lanes
        .iter()
        .find(|lane| lane.lane_id == lane_id)
        .expect("tactic lane")
}

#[test]
fn current_ip_assigns_each_managed_club_player_once_or_marks_them_unavailable() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    tactic::save_tactic(&conn, save_id, &tactic::default_tactic()).expect("seed tactic");

    for uid in [77, 78, 79] {
        set_player_positions(&conn, save_id, uid, r#"{"AMR":18}"#);
        set_right_winger_scores(&conn, save_id, uid, Some(80));
    }
    set_player_positions(&conn, save_id, 80, r#"{"AMR":11}"#);

    let reference = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::InPossession,
        RoleReferenceBasis::Current,
    )
    .expect("role reference should load");
    let assigned: Vec<_> = reference
        .lanes
        .iter()
        .flat_map(|lane| lane.players.iter().map(|player| player.player_uid))
        .collect();
    let all_players: Vec<_> = assigned
        .iter()
        .copied()
        .chain(reference.no_eligible.iter().map(|player| player.player_uid))
        .collect();

    assert_eq!(all_players.len(), 4);
    assert_eq!(all_players.iter().copied().collect::<HashSet<_>>().len(), 4);
    assert_eq!(
        reference
            .no_eligible
            .iter()
            .map(|player| player.player_uid)
            .collect::<Vec<_>>(),
        vec![80]
    );
}

#[test]
fn phase_selection_uses_only_the_selected_phase_position_and_role() {
    let (_temp_dir, conn, save_id) = seeded_snapshot();
    set_player_positions(&conn, save_id, 77, r#"{"AML":20,"MR":20}"#);
    clear_current_scores(&conn, save_id);
    set_role_score(&conn, save_id, 77, "winger_ip", Some(20));
    set_role_score(&conn, save_id, 77, "tracking_wide_midfielder_oop", Some(90));

    let ip = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::InPossession,
        RoleReferenceBasis::Current,
    )
    .expect("IP role reference");
    let oop = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::OutOfPossession,
        RoleReferenceBasis::Current,
    )
    .expect("OOP role reference");

    assert_eq!(lane(&ip, "left_winger").players[0].current_score, Some(20));
    assert_eq!(
        lane(&oop, "right_winger").players[0].current_score,
        Some(90)
    );

    set_player_positions(&conn, save_id, 77, r#"{"AML":20,"MR":11}"#);
    let ip_with_unsuitable_oop = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::InPossession,
        RoleReferenceBasis::Current,
    )
    .expect("IP should ignore OOP familiarity");
    assert_eq!(
        lane(&ip_with_unsuitable_oop, "left_winger").players[0].player_uid,
        77
    );

    set_player_positions(&conn, save_id, 77, r#"{"AML":11,"MR":20}"#);
    let oop_with_unsuitable_ip = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::OutOfPossession,
        RoleReferenceBasis::Current,
    )
    .expect("OOP should ignore IP familiarity");
    assert_eq!(
        lane(&oop_with_unsuitable_ip, "right_winger").players[0].player_uid,
        77
    );
}

#[test]
fn selected_basis_changes_the_best_lane_and_keeps_both_persisted_scores_for_that_lane() {
    let (_temp_dir, conn, save_id) = seeded_snapshot();
    let snapshot_id = current_snapshot_id(&conn, save_id);
    clear_current_scores(&conn, save_id);
    set_player_positions(&conn, save_id, 77, r#"{"AML":20,"AMR":20}"#);
    set_player_attributes(
        &conn,
        save_id,
        77,
        &attributes_with(
            10,
            &[
                ("Crossing", 20),
                ("Pace", 20),
                ("FirstTouch", 0),
                ("Composure", 0),
            ],
        ),
    );
    let mut changed_tactic = tactic::default_tactic();
    changed_tactic.lanes[8].ip_role_id = "inside_winger_ip".to_string();
    tactic::save_tactic(&conn, save_id, &changed_tactic).expect("save role comparison tactic");
    set_role_score(&conn, save_id, 77, "inside_winger_ip", Some(10));
    set_role_score(&conn, save_id, 77, "winger_ip", Some(90));
    conn.execute(
        "UPDATE player_potential_role_scores
         SET score = CASE role_id
             WHEN 'inside_winger_ip' THEN 90
             WHEN 'winger_ip' THEN 10
             ELSE score
         END
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("set persisted potential role scores");
    let before = planner_potential_state(&conn, save_id, snapshot_id);
    deny_potential_writes(&conn);

    let current = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::InPossession,
        RoleReferenceBasis::Current,
    )
    .expect("current role reference");
    let potential = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::InPossession,
        RoleReferenceBasis::Potential,
    )
    .expect("potential role reference");

    let current_player = &lane(&current, "right_winger").players[0];
    assert_eq!(current_player.player_uid, 77);
    assert_eq!(current_player.current_score, Some(90));
    assert_eq!(current_player.potential_score, Some(10));

    let potential_player = &lane(&potential, "left_winger").players[0];
    assert_eq!(potential_player.player_uid, 77);
    assert_eq!(potential_player.current_score, Some(10));
    assert_eq!(potential_player.potential_score, Some(90));
    assert_eq!(planner_potential_state(&conn, save_id, snapshot_id), before);
}

#[test]
fn role_reference_rejects_missing_potential_role_for_both_bases_without_writes() {
    let (_temp_dir, conn, save_id) = seeded_snapshot();
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "DELETE FROM player_potential_role_scores
         WHERE snapshot_id = ?1 AND uid = 77 AND role_id = 'winger_ip'",
        params![snapshot_id],
    )
    .expect("remove persisted potential role");
    let before = planner_potential_state(&conn, save_id, snapshot_id);
    deny_potential_writes(&conn);

    for basis in [RoleReferenceBasis::Current, RoleReferenceBasis::Potential] {
        let error = get_role_reference(&conn, save_id, RoleReferencePhase::InPossession, basis)
            .expect_err("reject incomplete potential state");
        assert_eq!(error, "Current potential snapshot is incomplete");
        assert_eq!(planner_potential_state(&conn, save_id, snapshot_id), before);
    }
}

#[test]
fn role_reference_rejects_wrong_version_potential_role_for_both_bases_without_writes() {
    let (_temp_dir, conn, save_id) = seeded_snapshot();
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "UPDATE player_potential_role_scores
         SET projection_model_version = 999
         WHERE snapshot_id = ?1 AND uid = 77 AND role_id = 'winger_ip'",
        params![snapshot_id],
    )
    .expect("set stale potential role version");
    let before = planner_potential_state(&conn, save_id, snapshot_id);
    deny_potential_writes(&conn);

    for basis in [RoleReferenceBasis::Current, RoleReferenceBasis::Potential] {
        let error = get_role_reference(&conn, save_id, RoleReferencePhase::InPossession, basis)
            .expect_err("reject stale potential state");
        assert_eq!(error, "Current potential snapshot is incomplete");
        assert_eq!(planner_potential_state(&conn, save_id, snapshot_id), before);
    }
}

#[test]
fn familiarity_and_strict_foot_rules_are_applied_to_reference_scores() {
    let (_temp_dir, conn, save_id) = seeded_snapshot();
    set_player_positions(&conn, save_id, 77, r#"{"AMR":15}"#);
    set_right_winger_scores(&conn, save_id, 77, Some(80));

    let familiar = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::InPossession,
        RoleReferenceBasis::Current,
    )
    .expect("familiar role reference");
    assert_eq!(
        lane(&familiar, "right_winger").players[0].current_score,
        Some(75)
    );

    set_player_positions(&conn, save_id, 77, r#"{"AMR":11}"#);
    let unsuitable = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::InPossession,
        RoleReferenceBasis::Current,
    )
    .expect("unsuitable role reference");
    assert_eq!(unsuitable.no_eligible.len(), 1);

    let mut strict_tactic = tactic::default_tactic();
    strict_tactic.lanes[9].preferred_foot = "right".to_string();
    strict_tactic.lanes[9].foot_preference = "strict".to_string();
    tactic::save_tactic(&conn, save_id, &strict_tactic).expect("save strict tactic");
    set_player_positions(&conn, save_id, 77, r#"{"AMR":20}"#);
    set_player_preferred_foot(&conn, save_id, 77, "left");
    let wrong_foot = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::InPossession,
        RoleReferenceBasis::Current,
    )
    .expect("strict-foot role reference");
    assert_eq!(wrong_foot.no_eligible.len(), 1);
}

#[test]
fn equal_scores_keep_the_first_lane_in_persisted_tactic_order() {
    let (_temp_dir, conn, save_id) = seeded_snapshot();
    set_player_positions(&conn, save_id, 77, r#"{"AML":20,"AMR":20}"#);
    clear_current_scores(&conn, save_id);
    set_role_score(&conn, save_id, 77, "winger_ip", Some(50));

    let reference = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::InPossession,
        RoleReferenceBasis::Current,
    )
    .expect("role reference");

    assert_eq!(lane(&reference, "left_winger").players[0].player_uid, 77);
    assert!(lane(&reference, "right_winger").players.is_empty());
}

#[test]
fn missing_current_basis_marks_the_player_unavailable_while_persisted_potential_is_eligible() {
    let (_temp_dir, conn, save_id) = seeded_snapshot();
    let snapshot_id = current_snapshot_id(&conn, save_id);
    set_player_positions(&conn, save_id, 77, r#"{"AMR":20}"#);
    clear_current_scores(&conn, save_id);
    set_player_attributes(&conn, save_id, 77, &full_attributes(20));
    conn.execute(
        "UPDATE player_potential_role_scores
         SET score = 100
         WHERE snapshot_id = ?1 AND uid = 77 AND role_id = 'winger_ip'",
        params![snapshot_id],
    )
    .expect("set persisted potential winger score");

    let reference = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::InPossession,
        RoleReferenceBasis::Current,
    )
    .expect("role reference");

    assert_eq!(reference.no_eligible.len(), 1);
    assert_eq!(reference.no_eligible[0].player_uid, 77);
    assert_eq!(reference.no_eligible[0].current_score, None);
    assert_eq!(reference.no_eligible[0].potential_score, None);

    let potential = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::InPossession,
        RoleReferenceBasis::Potential,
    )
    .expect("potential role reference");
    let potential_player = &lane(&potential, "right_winger").players[0];
    assert_eq!(potential_player.current_score, None);
    assert!(potential_player.potential_score.is_some());
}

#[test]
fn baseline_order_is_case_insensitive_name_then_uid() {
    let (temp_dir, mut conn, save_id) = seeded_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    clear_current_scores(&conn, save_id);
    conn.execute(
        "UPDATE players
         SET name = CASE uid WHEN 77 THEN 'beta' WHEN 78 THEN 'Alpha' WHEN 79 THEN 'alpha' ELSE 'zeta' END,
             positions_json = '{\"AMR\":20}'
         WHERE snapshot_id = ?1",
        [current_snapshot_id(&conn, save_id)],
    )
    .expect("set names and positions");
    for uid in [77, 78, 79, 80] {
        set_role_score(&conn, save_id, uid, "winger_ip", Some(50));
    }

    let reference = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::InPossession,
        RoleReferenceBasis::Current,
    )
    .expect("role reference");

    let players = &lane(&reference, "right_winger").players;
    assert_eq!(
        players
            .iter()
            .map(|player| (player.name.as_str(), player.player_uid))
            .collect::<Vec<_>>(),
        vec![("Alpha", 78), ("alpha", 79), ("beta", 77), ("zeta", 80)]
    );
}

#[test]
fn existing_assignments_and_team_age_do_not_filter_reference_players() {
    let (_temp_dir, conn, save_id) = seeded_snapshot();
    set_player_positions(&conn, save_id, 77, r#"{"AMR":20}"#);
    set_right_winger_scores(&conn, save_id, 77, Some(80));
    conn.execute(
        "UPDATE players SET age = NULL, team_level = 'youth' WHERE snapshot_id = ?1 AND uid = 77",
        [current_snapshot_id(&conn, save_id)],
    )
    .expect("set ignored planner fields");
    conn.execute(
        "INSERT INTO planner_strings (save_id, team, string_order) VALUES (?1, 'senior', 0)",
        [save_id],
    )
    .expect("insert planner string");
    let string_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO planner_assignments (save_id, string_id, lane_id, player_uid, last_known_name)
         VALUES (?1, ?2, 'right_winger', 77, 'Golden Fixture Player')",
        [save_id, string_id],
    )
    .expect("insert existing assignment");

    let reference = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::InPossession,
        RoleReferenceBasis::Current,
    )
    .expect("role reference");

    assert_eq!(lane(&reference, "right_winger").players[0].player_uid, 77);
}

#[test]
fn missing_snapshot_or_managed_club_is_reported_without_querying_players() {
    let (_temp_dir, conn, save_id) = seeded_snapshot();
    conn.execute(
        "UPDATE players SET current_club = 'Other FC' WHERE snapshot_id = ?1",
        [current_snapshot_id(&conn, save_id)],
    )
    .expect("move players outside managed club");
    let empty = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::InPossession,
        RoleReferenceBasis::Current,
    )
    .expect("empty role reference");
    assert!(empty.lanes.iter().all(|lane| lane.players.is_empty()));
    assert!(empty.no_eligible.is_empty());
    conn.execute(
        "UPDATE players SET current_club = 'Loan FC' WHERE snapshot_id = ?1",
        [current_snapshot_id(&conn, save_id)],
    )
    .expect("restore managed club players");
    conn.execute(
        "DELETE FROM managed_club_settings WHERE save_id = ?1",
        [save_id],
    )
    .expect("remove managed club");
    let missing_club = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::InPossession,
        RoleReferenceBasis::Current,
    )
    .expect_err("missing club should fail");
    assert_eq!(missing_club, "No managed club configured for this save");

    crate::features::managed_club::service::set_managed_club(&conn, save_id, "Loan FC")
        .expect("restore managed club");
    conn.execute(
        "UPDATE snapshots SET is_current = 0 WHERE save_id = ?1",
        [save_id],
    )
    .expect("remove current snapshot");
    let missing_snapshot = get_role_reference(
        &conn,
        save_id,
        RoleReferencePhase::InPossession,
        RoleReferenceBasis::Current,
    )
    .expect_err("missing snapshot should fail");
    assert_eq!(missing_snapshot, "No current snapshot loaded for this save");
}

#[test]
fn role_reference_inputs_are_closed_enums() {
    assert_eq!(
        super::role_reference::RoleReferencePhase::parse("ip").expect_err("reject alias"),
        "Unknown planner role reference phase `ip`"
    );
    assert_eq!(
        super::role_reference::RoleReferenceBasis::parse("combined")
            .expect_err("reject combined basis"),
        "Unknown planner role reference score basis `combined`"
    );
}

#[test]
fn role_reference_dto_uses_planner_camel_case_fields() {
    let reference = super::role_reference::RoleReference {
        lanes: vec![super::role_reference::RoleReferenceLane {
            lane_id: "right_winger".to_string(),
            players: vec![super::role_reference::RoleReferencePlayer {
                player_uid: 77,
                name: "Player".to_string(),
                current_score: Some(80),
                potential_score: None,
            }],
        }],
        no_eligible: Vec::new(),
    };
    let dto: super::commands::PlannerRoleReferenceDto = reference.into();
    let json = serde_json::to_value(dto).expect("serialize role reference DTO");

    assert_eq!(json["lanes"][0]["laneId"], "right_winger");
    assert_eq!(json["lanes"][0]["players"][0]["playerUid"], 77);
    assert_eq!(json["lanes"][0]["players"][0]["currentScore"], 80);
    assert!(json["lanes"][0]["players"][0]["potentialScore"].is_null());
    assert!(json["noEligible"]
        .as_array()
        .expect("no-eligible array")
        .is_empty());
}
