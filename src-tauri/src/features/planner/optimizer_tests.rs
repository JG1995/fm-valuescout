use rusqlite::params;

use crate::features::scoring::catalog::DUMP_ATTRIBUTE_KEYS;

use super::depth::{add_string, assign_player, get_depth, PlannerTeam};
use super::optimizer::{
    allocation_score, foot_matches, match_lanes, optimize_depth, optimize_depth_with_basis,
    OptimizerCandidate, ScoreBasis,
};
use super::service::{self, ClubSourceInput};
use super::tactic;
use super::teams::{save_team_settings, PlannerTeamInput};
use super::test_support::{
    add_picker_candidates, assigned_player_uid, assignment_provenance, current_snapshot_id,
    open_with_snapshot, set_player_age, set_player_positions, set_player_preferred_foot,
    set_right_winger_scores, set_role_score, team_strings,
};

#[test]
fn matcher_finds_the_best_total_instead_of_greedily_filling_the_first_lane() {
    let candidates = [
        OptimizerCandidate {
            player_uid: 10,
            last_known_name: "Flexible player".to_string(),
            lane_scores: vec![Some(100), Some(99)],
        },
        OptimizerCandidate {
            player_uid: 20,
            last_known_name: "First-lane specialist".to_string(),
            lane_scores: vec![Some(98), None],
        },
    ];

    assert_eq!(match_lanes(&[0, 1], &candidates), [Some(1), Some(0)]);
}

#[test]
fn matcher_prefers_a_zero_score_player_to_a_blank_lane() {
    let candidates = [OptimizerCandidate {
        player_uid: 10,
        last_known_name: "Zero-score player".to_string(),
        lane_scores: vec![Some(0)],
    }];

    assert_eq!(match_lanes(&[0], &candidates), [Some(0)]);
}

#[test]
fn matcher_breaks_equal_scores_by_lowest_uid_in_lane_order() {
    let candidates = [
        OptimizerCandidate {
            player_uid: 20,
            last_known_name: "Second UID".to_string(),
            lane_scores: vec![Some(50), Some(50)],
        },
        OptimizerCandidate {
            player_uid: 10,
            last_known_name: "First UID".to_string(),
            lane_scores: vec![Some(50), Some(50)],
        },
    ];

    assert_eq!(match_lanes(&[0, 1], &candidates), [Some(1), Some(0)]);
}

#[test]
fn two_footed_players_match_every_restricted_foot_rule() {
    for (player_foot, preferred_foot, expected) in [
        ("left", "any", true),
        ("right", "any", true),
        ("either", "any", true),
        ("", "any", true),
        ("left", "left", true),
        ("right", "left", false),
        ("either", "left", true),
        ("", "left", false),
        ("left", "right", false),
        ("right", "right", true),
        ("either", "right", true),
        ("", "right", false),
        ("left", "both", false),
        ("right", "both", false),
        ("either", "both", true),
        ("", "both", false),
    ] {
        assert_eq!(
            foot_matches(player_foot, preferred_foot),
            expected,
            "{player_foot:?} against {preferred_foot:?}",
        );
    }
}

#[test]
fn soft_foot_mismatches_are_capped_at_zero() {
    let mut lane = tactic::default_tactic().lanes.remove(0);
    lane.preferred_foot = "right".to_string();

    assert_eq!(allocation_score(3, "left", &lane), Some(0));
}

#[test]
fn optimizer_switches_between_current_and_projected_candidate_scores() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let snapshot_id = current_snapshot_id(&conn, save_id);
    let complete_attributes = serde_json::to_string(
        &DUMP_ATTRIBUTE_KEYS
            .iter()
            .map(|key| ((*key).to_string(), serde_json::Value::from(10)))
            .collect::<serde_json::Map<_, _>>(),
    )
    .expect("serialize complete attributes");

    conn.execute(
        "UPDATE players
         SET ca = ?1, pa = ?2, age = ?3, positions_json = ?4, attributes_json = ?5
         WHERE snapshot_id = ?6 AND uid = ?7",
        params![
            100,
            100,
            20,
            r#"{"AMR": 18, "MR": 18}"#,
            "{}",
            snapshot_id,
            77
        ],
    )
    .expect("make current-fit player missing projected requirements");
    conn.execute(
        "UPDATE players
         SET ca = ?1, pa = ?2, age = ?3, positions_json = ?4, attributes_json = ?5
         WHERE snapshot_id = ?6 AND uid = ?7",
        params![
            80,
            170,
            20,
            r#"{"AMR": 18, "MR": 18}"#,
            complete_attributes,
            snapshot_id,
            78
        ],
    )
    .expect("make future-fit player projectable");
    set_right_winger_scores(&conn, save_id, 77, Some(100));
    set_right_winger_scores(&conn, save_id, 78, Some(90));

    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    tactic.lanes[9].importance_rank = Some(1);
    tactic::save_tactic(&conn, save_id, &tactic).expect("rank right winger");

    let current = optimize_depth(&conn, save_id).expect("optimize current scores");
    let potential = optimize_depth_with_basis(&conn, save_id, ScoreBasis::Potential)
        .expect("optimize potential scores");

    assert_eq!(
        assigned_player_uid(&current, PlannerTeam::Senior, "right_winger"),
        Some(77)
    );
    assert_eq!(
        assigned_player_uid(&potential, PlannerTeam::Senior, "right_winger"),
        Some(78)
    );
    assert_eq!(
        ScoreBasis::parse("unsupported").expect_err("reject unknown basis"),
        "Unknown optimizer score basis `unsupported`"
    );
}

#[test]
fn optimizer_skips_absent_team_sources_for_current_and_potential_scores() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    service::save_club_family(
        &conn,
        save_id,
        "Loan FC",
        &[ClubSourceInput {
            team: "reserves".to_string(),
            club_name: "Loan B FC".to_string(),
            team_level: None,
        }],
    )
    .expect("configure reserve-only source");
    set_right_winger_scores(&conn, save_id, 79, Some(100));
    get_depth(&conn, save_id).expect("initialize planner depth");
    save_team_settings(
        &conn,
        save_id,
        &[
            PlannerTeamInput {
                team: "senior".to_string(),
                display_name: "Senior".to_string(),
            },
            PlannerTeamInput {
                team: "youth".to_string(),
                display_name: "Youth".to_string(),
            },
        ],
        false,
    )
    .expect("remove reserves before optimization");

    let current = optimize_depth(&conn, save_id).expect("optimize current scores");
    let potential = optimize_depth_with_basis(&conn, save_id, ScoreBasis::Potential)
        .expect("optimize potential scores");
    for optimized in [current, potential] {
        assert!(!optimized
            .teams
            .iter()
            .any(|team| team.team == PlannerTeam::Reserves));
        assert!(optimized
            .teams
            .iter()
            .flat_map(|team| team.strings.iter())
            .flat_map(|planner_string| planner_string.assignments.iter())
            .all(|assignment| assignment.player_uid != 79));
    }
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM planner_strings WHERE save_id = ?1 AND team = 'reserves'",
            [save_id],
            |row| row.get::<_, i64>(0),
        )
        .expect("count removed-team strings"),
        0
    );
}

#[test]
fn optimizer_reserves_manual_players_before_automatic_allocation() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    set_right_winger_scores(&conn, save_id, 77, Some(100));
    set_right_winger_scores(&conn, save_id, 78, Some(90));
    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    tactic.lanes[0].importance_rank = Some(1);
    tactic::save_tactic(&conn, save_id, &tactic).expect("rank manually occupied lane");

    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let senior_string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    assign_player(&conn, save_id, senior_string_id, "goalkeeper", 77)
        .expect("preserve the manual player");

    let optimized = optimize_depth(&conn, save_id).expect("optimize planner depth");
    let senior_assignments = &team_strings(&optimized, PlannerTeam::Senior)[0].assignments;
    assert!(senior_assignments
        .iter()
        .any(|assignment| assignment.player_uid == 77 && assignment.lane_id == "goalkeeper"));
    assert!(senior_assignments
        .iter()
        .any(|assignment| assignment.player_uid == 78 && assignment.lane_id == "right_winger"));
}

#[test]
fn optimizer_uses_the_weight_for_each_tactic_lane() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let snapshot_id: i64 = conn
        .query_row(
            "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
            params![save_id],
            |row| row.get(0),
        )
        .expect("current snapshot");
    conn.execute(
        "UPDATE player_role_scores
         SET score = NULL
         WHERE snapshot_id = ?1
           AND uid IN (77, 78)
           AND role_id NOT IN ('winger_ip', 'tracking_wide_midfielder_oop')",
        [snapshot_id],
    )
    .expect("limit candidates to the right-winger lane");
    set_role_score(&conn, save_id, 77, "winger_ip", Some(100));
    set_role_score(&conn, save_id, 77, "tracking_wide_midfielder_oop", Some(0));
    set_role_score(&conn, save_id, 78, "winger_ip", Some(0));
    set_role_score(
        &conn,
        save_id,
        78,
        "tracking_wide_midfielder_oop",
        Some(100),
    );
    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    tactic.lanes[9].ip_weight = 1.0;
    tactic::save_tactic(&conn, save_id, &tactic).expect("save IP-heavy lane");

    let ip_heavy = optimize_depth(&conn, save_id).expect("optimize IP-heavy tactic");

    assert_eq!(
        assigned_player_uid(&ip_heavy, PlannerTeam::Senior, "right_winger"),
        Some(77)
    );

    tactic.lanes[9].ip_weight = 0.0;
    tactic::save_tactic(&conn, save_id, &tactic).expect("save OOP-heavy lane");
    let oop_heavy = optimize_depth(&conn, save_id).expect("optimize OOP-heavy tactic");

    assert_eq!(
        assigned_player_uid(&oop_heavy, PlannerTeam::Senior, "right_winger"),
        Some(78)
    );
}

#[test]
fn optimizer_applies_strict_and_preferred_foot_rules_without_changing_manual_assignments() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let snapshot_id = super::test_support::current_snapshot_id(&conn, save_id);
    conn.execute(
        "UPDATE player_role_scores
         SET score = NULL
         WHERE snapshot_id = ?1
           AND uid IN (77, 78)
           AND role_id NOT IN ('winger_ip', 'tracking_wide_midfielder_oop')",
        [snapshot_id],
    )
    .expect("limit candidates to the right-winger lane");
    set_right_winger_scores(&conn, save_id, 77, Some(100));
    set_right_winger_scores(&conn, save_id, 78, Some(98));
    set_player_preferred_foot(&conn, save_id, 77, "left");
    set_player_preferred_foot(&conn, save_id, 78, "right");

    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    tactic.lanes[9].preferred_foot = "right".to_string();
    tactic.lanes[9].foot_preference = "preferred".to_string();
    tactic::save_tactic(&conn, save_id, &tactic).expect("save soft preference");

    let soft = optimize_depth(&conn, save_id).expect("optimize soft preference");
    assert_eq!(
        assigned_player_uid(&soft, PlannerTeam::Senior, "right_winger"),
        Some(78)
    );

    tactic.lanes[9].foot_preference = "strict".to_string();
    tactic::save_tactic(&conn, save_id, &tactic).expect("save strict preference");
    let strict = optimize_depth(&conn, save_id).expect("optimize strict preference");
    assert_eq!(
        assigned_player_uid(&strict, PlannerTeam::Senior, "right_winger"),
        Some(78)
    );

    tactic.lanes[9].importance_rank = Some(1);
    tactic.lanes[9].foot_preference = "preferred".to_string();
    tactic::save_tactic(&conn, save_id, &tactic).expect("save ranked soft preference");
    let ranked_soft = optimize_depth(&conn, save_id).expect("optimize ranked soft preference");
    assert_eq!(
        assigned_player_uid(&ranked_soft, PlannerTeam::Senior, "right_winger"),
        Some(78)
    );

    tactic.lanes[9].foot_preference = "strict".to_string();
    tactic::save_tactic(&conn, save_id, &tactic).expect("save ranked strict preference");
    let ranked_strict = optimize_depth(&conn, save_id).expect("optimize ranked strict preference");
    assert_eq!(
        assigned_player_uid(&ranked_strict, PlannerTeam::Senior, "right_winger"),
        Some(78)
    );

    set_player_preferred_foot(&conn, save_id, 78, "left");
    let strict_blank = optimize_depth(&conn, save_id).expect("optimize ranked strict mismatch");
    assert_eq!(
        assigned_player_uid(&strict_blank, PlannerTeam::Senior, "right_winger"),
        None
    );

    let depth = get_depth(&conn, save_id).expect("load depth");
    let senior_string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    tactic.lanes[0].preferred_foot = "right".to_string();
    tactic.lanes[0].foot_preference = "strict".to_string();
    tactic::save_tactic(&conn, save_id, &tactic).expect("save strict goalkeeper preference");
    assign_player(&conn, save_id, senior_string_id, "goalkeeper", 77)
        .expect("assign manual left-footed player");
    let rerun = optimize_depth(&conn, save_id).expect("rerun optimization");
    assert!(team_strings(&rerun, PlannerTeam::Senior)[0]
        .assignments
        .iter()
        .any(|assignment| assignment.lane_id == "goalkeeper" && assignment.player_uid == 77));
}

#[test]
fn optimizer_uses_the_exact_matcher_when_lanes_are_unranked() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let snapshot_id: i64 = conn
        .query_row(
            "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
            params![save_id],
            |row| row.get(0),
        )
        .expect("current snapshot");
    conn.execute(
        "UPDATE player_role_scores
         SET score = NULL
         WHERE snapshot_id = ?1
           AND uid IN (77, 78)
           AND role_id NOT IN ('inside_winger_ip', 'winger_ip', 'tracking_wide_midfielder_oop')",
        [snapshot_id],
    )
    .expect("limit candidates to conflicting winger lanes");
    set_role_score(&conn, save_id, 77, "inside_winger_ip", Some(100));
    set_role_score(&conn, save_id, 77, "winger_ip", Some(98));
    set_role_score(
        &conn,
        save_id,
        77,
        "tracking_wide_midfielder_oop",
        Some(100),
    );
    set_role_score(&conn, save_id, 78, "inside_winger_ip", Some(98));
    set_role_score(&conn, save_id, 78, "winger_ip", None);
    set_role_score(&conn, save_id, 78, "tracking_wide_midfielder_oop", Some(98));
    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    set_player_positions(&conn, save_id, 77, r#"{"ML": 18, "MR": 18, "AMR": 18}"#);
    set_player_positions(&conn, save_id, 78, r#"{"ML": 18, "MR": 18, "AMR": 18}"#);
    tactic.lanes[8].ip_position = "MR".to_string();
    tactic.lanes[8].oop_position = "ML".to_string();
    tactic.lanes[8].ip_role_id = "inside_winger_ip".to_string();
    tactic::save_tactic(&conn, save_id, &tactic).expect("save conflicting unranked lanes");

    let optimized = optimize_depth(&conn, save_id).expect("optimize unranked tactic");

    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Senior, "left_winger"),
        Some(78)
    );
    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Senior, "right_winger"),
        Some(77)
    );
}

#[test]
fn optimizer_assigns_ranked_lanes_in_ascending_order() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let snapshot_id: i64 = conn
        .query_row(
            "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
            params![save_id],
            |row| row.get(0),
        )
        .expect("current snapshot");
    conn.execute(
        "UPDATE player_role_scores
         SET score = NULL
         WHERE snapshot_id = ?1
           AND uid IN (77, 78)
           AND role_id NOT IN ('winger_ip', 'tracking_wide_midfielder_oop')",
        [snapshot_id],
    )
    .expect("limit candidates to conflicting winger lanes");
    set_right_winger_scores(&conn, save_id, 77, Some(100));
    set_right_winger_scores(&conn, save_id, 78, Some(100));
    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    set_player_positions(&conn, save_id, 77, r#"{"ML": 18, "MR": 18, "AMR": 18}"#);
    set_player_positions(&conn, save_id, 78, r#"{"ML": 18, "MR": 18, "AMR": 18}"#);
    tactic.lanes[8].ip_position = "MR".to_string();
    tactic.lanes[8].oop_position = "ML".to_string();
    tactic.lanes[8].importance_rank = Some(11);
    tactic.lanes[9].importance_rank = Some(1);
    tactic::save_tactic(&conn, save_id, &tactic).expect("save ranked tactic");

    let optimized = optimize_depth(&conn, save_id).expect("optimize ranked tactic");

    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Senior, "right_winger"),
        Some(77)
    );
    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Senior, "left_winger"),
        Some(78)
    );
}

#[test]
fn optimizer_leaves_a_ranked_lane_blank_without_reserving_candidates() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let snapshot_id: i64 = conn
        .query_row(
            "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
            params![save_id],
            |row| row.get(0),
        )
        .expect("current snapshot");
    conn.execute(
        "UPDATE player_role_scores
         SET score = NULL
         WHERE snapshot_id = ?1
           AND uid IN (77, 78)
           AND role_id NOT IN ('winger_ip', 'tracking_wide_midfielder_oop')",
        [snapshot_id],
    )
    .expect("limit candidates to the right-winger lane");
    set_player_positions(&conn, save_id, 77, r#"{"AMR": 18, "MR": 18}"#);
    set_player_positions(&conn, save_id, 78, r#"{"AMR": 18, "MR": 18}"#);
    set_right_winger_scores(&conn, save_id, 77, Some(100));
    set_right_winger_scores(&conn, save_id, 78, Some(90));
    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    tactic.lanes[0].importance_rank = Some(1);
    tactic::save_tactic(&conn, save_id, &tactic).expect("save rank with no eligible player");

    let optimized = optimize_depth(&conn, save_id).expect("optimize ranked tactic");

    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Senior, "goalkeeper"),
        None
    );
    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Senior, "right_winger"),
        Some(77)
    );
}

#[test]
fn optimizer_keeps_senior_priority_over_a_higher_global_total() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    set_player_age(&conn, save_id, 77, Some(18));
    set_player_age(&conn, save_id, 78, Some(24));
    set_right_winger_scores(&conn, save_id, 77, Some(100));
    set_right_winger_scores(&conn, save_id, 78, Some(90));
    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    tactic.lanes[9].importance_rank = Some(1);
    tactic::save_tactic(&conn, save_id, &tactic).expect("rank right winger");

    let optimized = optimize_depth(&conn, save_id).expect("optimize planner depth");

    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Senior, "right_winger"),
        Some(77)
    );
    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Reserves, "right_winger"),
        None
    );
}

#[test]
fn optimizer_allocates_strings_in_ascending_order_within_a_team() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let first_string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    let second_string_id = add_string(&conn, save_id, PlannerTeam::Senior)
        .expect("add second string")
        .id;
    set_right_winger_scores(&conn, save_id, 77, Some(100));
    set_right_winger_scores(&conn, save_id, 78, Some(90));
    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    tactic.lanes[9].importance_rank = Some(1);
    tactic::save_tactic(&conn, save_id, &tactic).expect("rank right winger");

    let optimized = optimize_depth(&conn, save_id).expect("optimize planner depth");
    let senior_strings = team_strings(&optimized, PlannerTeam::Senior);

    assert_eq!(senior_strings[0].id, first_string_id);
    assert_eq!(senior_strings[1].id, second_string_id);
    assert_eq!(
        senior_strings[0]
            .assignments
            .iter()
            .find(|assignment| assignment.lane_id == "right_winger")
            .map(|assignment| assignment.player_uid),
        Some(77)
    );
    assert_eq!(
        senior_strings[1]
            .assignments
            .iter()
            .find(|assignment| assignment.lane_id == "right_winger")
            .map(|assignment| assignment.player_uid),
        Some(78)
    );
}

#[test]
fn optimizer_requires_age_both_positions_and_complete_scores() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let senior_string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    assign_player(&conn, save_id, senior_string_id, "right_winger", 77)
        .expect("occupy the senior lane manually");

    set_player_age(&conn, save_id, 78, None);
    set_right_winger_scores(&conn, save_id, 78, Some(100));
    set_player_age(&conn, save_id, 79, Some(23));
    set_right_winger_scores(&conn, save_id, 79, Some(0));
    set_player_age(&conn, save_id, 80, Some(23));
    set_player_positions(&conn, save_id, 80, r#"{"AMR": 18}"#);
    set_right_winger_scores(&conn, save_id, 80, Some(100));

    let optimized = optimize_depth(&conn, save_id).expect("optimize planner depth");

    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Reserves, "right_winger"),
        Some(79)
    );
    assert!(!optimized
        .teams
        .iter()
        .flat_map(|team| &team.strings)
        .flat_map(|planner_string| &planner_string.assignments)
        .any(|assignment| assignment.player_uid == 78 || assignment.player_uid == 80));
}

#[test]
fn optimizer_limits_attached_club_sources_to_their_configured_team() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    set_right_winger_scores(&conn, save_id, 77, None);
    set_right_winger_scores(&conn, save_id, 78, None);
    set_player_age(&conn, save_id, 79, Some(18));
    set_right_winger_scores(&conn, save_id, 79, Some(100));
    set_right_winger_scores(&conn, save_id, 80, None);

    let optimized = optimize_depth(&conn, save_id).expect("optimize planner depth");

    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Senior, "right_winger"),
        None
    );
    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Reserves, "right_winger"),
        Some(79)
    );
    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Youth, "right_winger"),
        None
    );
}

#[test]
fn optimizer_does_not_load_scores_outside_configured_club_family() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let snapshot_id: i64 = conn
        .query_row(
            "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
            params![save_id],
            |row| row.get(0),
        )
        .expect("current snapshot");
    conn.execute(
        "UPDATE players SET current_club = 'Other FC' WHERE snapshot_id = ?1 AND uid = 80",
        params![snapshot_id],
    )
    .expect("exclude player from configured sources");
    conn.execute_batch("PRAGMA ignore_check_constraints = ON")
        .expect("allow invalid non-source score");
    conn.execute(
        "UPDATE player_role_scores SET score = 'invalid' WHERE snapshot_id = ?1 AND uid = 80",
        params![snapshot_id],
    )
    .expect("set invalid non-source score");
    conn.execute_batch("PRAGMA ignore_check_constraints = OFF")
        .expect("restore score constraints");

    optimize_depth(&conn, save_id).expect("ignore scores outside configured sources");
}

#[test]
fn optimizer_excludes_players_with_a_missing_role_score() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    set_player_age(&conn, save_id, 78, Some(23));
    set_right_winger_scores(&conn, save_id, 78, Some(100));
    set_role_score(&conn, save_id, 78, "winger_ip", None);
    set_player_age(&conn, save_id, 79, Some(23));
    set_right_winger_scores(&conn, save_id, 79, Some(50));

    let optimized = optimize_depth(&conn, save_id).expect("optimize planner depth");

    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Reserves, "right_winger"),
        Some(79)
    );
    assert!(!optimized
        .teams
        .iter()
        .flat_map(|team| &team.strings)
        .flat_map(|planner_string| &planner_string.assignments)
        .any(|assignment| assignment.player_uid == 78));
}

#[test]
fn optimizer_replaces_only_prior_optimizer_assignments() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    set_right_winger_scores(&conn, save_id, 77, Some(90));
    let first = optimize_depth(&conn, save_id).expect("run first optimization");
    assert_eq!(
        assigned_player_uid(&first, PlannerTeam::Senior, "right_winger"),
        Some(77)
    );
    assert_eq!(assignment_provenance(&conn, 77), "optimizer");

    let senior_string_id = team_strings(&first, PlannerTeam::Senior)[0].id;
    assign_player(&conn, save_id, senior_string_id, "goalkeeper", 78)
        .expect("add a manual assignment");
    set_right_winger_scores(&conn, save_id, 77, None);

    let rerun = optimize_depth(&conn, save_id).expect("rerun optimization");
    assert_eq!(
        assigned_player_uid(&rerun, PlannerTeam::Senior, "right_winger"),
        None
    );
    assert_eq!(assignment_provenance(&conn, 78), "manual");
}

#[test]
fn optimizer_rolls_back_replacement_when_an_insert_fails() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    set_right_winger_scores(&conn, save_id, 77, Some(90));
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let senior_string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    conn.execute(
        "INSERT INTO planner_assignments (
             save_id, string_id, lane_id, player_uid, last_known_name, provenance
         ) VALUES (?1, ?2, 'right_winger', 77, 'Existing optimizer row', 'optimizer')",
        params![save_id, senior_string_id],
    )
    .expect("seed optimizer assignment");
    conn.execute_batch(
        "CREATE TRIGGER fail_optimizer_assignment
         BEFORE INSERT ON planner_assignments
         WHEN NEW.provenance = 'optimizer'
         BEGIN
             SELECT RAISE(ABORT, 'forced optimizer failure');
         END;",
    )
    .expect("create failing trigger");

    let error = optimize_depth(&conn, save_id).expect_err("roll back failed optimization");

    assert!(error.contains("forced optimizer failure"));
    assert_eq!(assignment_provenance(&conn, 77), "optimizer");
}
