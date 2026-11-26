use rusqlite::params;

use std::collections::HashSet;

use super::depth::{
    assign_player, clear_all, get_depth, set_assignment_pinned, PlannerDepth, PlannerTeam,
    StarterComparison,
};
use super::development::{choose_slot, set_optimize_for, set_position_familiarity, OptimizeFor};
use super::fit::PositionRule;
use super::optimizer::{
    self, allocate_rotation, match_lanes, unassigned_players, OptimizerCandidate,
};
use super::tactic;
use super::teams::{save_team_settings, PlannerTeamInput};
use super::test_support::{
    add_picker_candidates, assigned_player_uid, assignment_provenance, clear_current_scores_except,
    current_snapshot_id, deny_potential_writes, forget_managed_club_uid, lane_players,
    move_to_namesake_club, open_with_snapshot, planner_potential_state, seed_players,
    set_development, set_lane_scores, set_player_age, set_player_club, set_player_positions,
    set_player_preferred_foot, set_right_winger, set_right_winger_scores, set_role_score,
    team_strings,
};

/// Optimizes with position familiarity, the Planner's default.
fn optimize_depth(conn: &rusqlite::Connection, save_id: i64) -> Result<PlannerDepth, String> {
    optimizer::optimize_depth(conn, save_id, PositionRule::Familiarity)
}

fn optimize_in_any_position(conn: &rusqlite::Connection, save_id: i64) -> PlannerDepth {
    optimizer::optimize_depth(conn, save_id, PositionRule::AnyPosition)
        .expect("optimize without position familiarity")
}

/// Whether the player placed first in the senior lane cannot play its positions.
fn out_of_position(depth: &PlannerDepth, lane_id: &str) -> bool {
    team_strings(depth, PlannerTeam::Senior)[0]
        .assignments
        .iter()
        .find(|assignment| assignment.lane_id == lane_id)
        .expect("senior assignment")
        .out_of_position
}

/// Keeps only the named teams, each with its usual age limit unless `max_age` sets one.
fn keep_teams(conn: &rusqlite::Connection, save_id: i64, teams: &[(&str, Option<i64>)]) {
    get_depth(conn, save_id).expect("create planner depth");
    let inputs = teams
        .iter()
        .map(|&(team, max_age)| PlannerTeamInput {
            team: team.to_string(),
            display_name: team.to_string(),
            max_age,
        })
        .collect::<Vec<_>>();
    save_team_settings(conn, save_id, &inputs, true).expect("keep the named teams");
}

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
fn optimizer_penalizes_sub_16_familiarity_without_excluding_eligible_players() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    clear_current_scores_except(
        &conn,
        save_id,
        &[77, 78],
        &["winger_ip", "tracking_wide_midfielder_oop"],
    );
    set_right_winger_scores(&conn, save_id, 77, Some(100));
    set_right_winger_scores(&conn, save_id, 78, Some(98));
    set_player_positions(&conn, save_id, 77, r#"{"AMR": 16, "MR": 15}"#);
    set_player_positions(&conn, save_id, 78, r#"{"AMR": 16, "MR": 16}"#);

    let one_below = optimize_depth(&conn, save_id).expect("optimize one sub-16 position");
    assert_eq!(
        assigned_player_uid(&one_below, PlannerTeam::Senior, "right_winger"),
        Some(78)
    );

    set_right_winger_scores(&conn, save_id, 78, Some(91));
    set_player_positions(&conn, save_id, 77, r#"{"AMR": 15, "MR": 15}"#);

    let both_below = optimize_depth(&conn, save_id).expect("optimize two sub-16 positions");
    assert_eq!(
        assigned_player_uid(&both_below, PlannerTeam::Senior, "right_winger"),
        Some(78)
    );
}

#[test]
fn without_position_familiarity_the_senior_squad_is_picked_on_scores_and_foot_alone() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    clear_current_scores_except(
        &conn,
        save_id,
        &[77, 78, 79, 80],
        &["winger_ip", "tracking_wide_midfielder_oop"],
    );
    for (player_uid, positions, score) in [
        (77, r#"{"AMR": 20, "MR": 20}"#, 70),
        (78, r#"{"ST": 20, "AMR": 11, "MR": 11}"#, 75),
        (79, r#"{"AMR": 12, "MR": 12}"#, 74),
        (80, r#"{"GK": 20}"#, 99),
    ] {
        set_player_positions(&conn, save_id, player_uid, positions);
        set_right_winger_scores(&conn, save_id, player_uid, Some(score));
        set_player_preferred_foot(&conn, save_id, player_uid, "right");
        set_player_age(&conn, save_id, player_uid, Some(25));
    }
    // The left winger takes a role nobody has a score in, so only one lane is in play.
    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    tactic
        .lanes
        .iter_mut()
        .find(|lane| lane.lane_id == "left_winger")
        .expect("left winger")
        .ip_role_id = "inside_forward_ip".to_string();
    tactic::save_tactic(&conn, save_id, &tactic).expect("save left winger role");
    let right_winger = |depth: &PlannerDepth| {
        lane_players(depth, PlannerTeam::Senior, "right_winger")
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
    };

    // The striker cannot play there, and the half-familiar winger loses 10 points.
    let with_positions = optimize_depth(&conn, save_id).expect("optimize with familiarity");
    assert_eq!(right_winger(&with_positions), [77, 79]);
    assert!(!out_of_position(&with_positions, "right_winger"));

    // Neither counts now, but a goalkeeper still plays only in goal.
    let any_position = optimize_in_any_position(&conn, save_id);
    assert_eq!(right_winger(&any_position), [78, 79, 77]);
    assert!(out_of_position(&any_position, "right_winger"));

    // The foot rule still marks the wrong foot down, or keeps him out.
    set_player_preferred_foot(&conn, save_id, 78, "left");
    tactic.lanes[9].preferred_foot = "right".to_string();
    for (foot_preference, expected) in [("preferred", vec![79, 77, 78]), ("strict", vec![79, 77])] {
        tactic.lanes[9].foot_preference = foot_preference.to_string();
        tactic::save_tactic(&conn, save_id, &tactic).expect("save foot preference");
        assert_eq!(
            right_winger(&optimize_in_any_position(&conn, save_id)),
            expected,
            "{foot_preference}"
        );
    }
}

#[test]
fn without_position_familiarity_an_outfield_player_never_plays_in_goal() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    clear_current_scores_except(&conn, save_id, &[77, 78, 79, 80], &[]);
    for (player_uid, positions, score) in [(77, r#"{"GK": 20}"#, 40), (78, r#"{"DC": 20}"#, 90)] {
        set_player_positions(&conn, save_id, player_uid, positions);
        set_lane_scores(&conn, save_id, player_uid, "goalkeeper", score, score);
    }

    let optimized = optimize_in_any_position(&conn, save_id);

    assert_eq!(
        lane_players(&optimized, PlannerTeam::Senior, "goalkeeper"),
        [Some(77), None, None]
    );
}

#[test]
fn without_position_familiarity_a_development_player_stands_out_among_those_who_play_a_slot() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    let rivals = (201..=210).collect::<Vec<_>>();
    let uids = [&[77][..], &rivals].concat();
    seed_players(&temp_dir, &mut conn, save_id, &uids);
    clear_current_scores_except(&conn, save_id, &uids, &[]);
    set_player_age(&conn, save_id, 77, Some(20));
    set_player_positions(&conn, save_id, 77, r#"{"AMR": 18, "MR": 18}"#);
    set_lane_scores(&conn, save_id, 77, "right_winger", 73, 73);
    set_lane_scores(&conn, save_id, 77, "centre_forward", 58, 58);
    // Across the save, right wingers score in the 70s and centre-forwards in the 40s. The
    // wingers would make better centre-forwards than 77, but none of them plays there.
    for (index, rival) in rivals.iter().copied().enumerate() {
        set_player_club(&conn, save_id, rival, "Elsewhere FC");
        let step = (index / 2) as u8;
        if index % 2 == 0 {
            set_player_positions(&conn, save_id, rival, r#"{"AMR": 18, "MR": 18}"#);
            set_lane_scores(&conn, save_id, rival, "right_winger", 70 + step, 70 + step);
            set_lane_scores(&conn, save_id, rival, "centre_forward", 90, 90);
        } else {
            set_player_positions(&conn, save_id, rival, r#"{"ST": 18}"#);
            set_lane_scores(
                &conn,
                save_id,
                rival,
                "centre_forward",
                40 + step,
                40 + step,
            );
        }
    }
    keep_teams(&conn, save_id, &[("reserves", Some(23))]);
    let reserves = |depth: &PlannerDepth| {
        ["right_winger", "centre_forward"]
            .map(|lane_id| lane_players(depth, PlannerTeam::Reserves, lane_id))
    };

    let with_positions = optimize_depth(&conn, save_id).expect("optimize with familiarity");
    assert_eq!(reserves(&with_positions), [[Some(77)], [None]]);

    let any_position = optimize_in_any_position(&conn, save_id);
    assert_eq!(reserves(&any_position), [[None], [Some(77)]]);
}

#[test]
fn optimizer_starts_a_prospect_over_a_declining_veteran_unless_he_is_unprofessional() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let snapshot_id = current_snapshot_id(&conn, save_id);
    set_right_winger(&conn, save_id, 77, 74, 74);
    set_development(&conn, save_id, 77, 33, [10, 10, 10]);
    set_right_winger(&conn, save_id, 78, 70, 84);
    set_development(&conn, save_id, 78, 20, [15, 14, 12]);
    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    tactic.lanes[9].importance_rank = Some(1);
    tactic::save_tactic(&conn, save_id, &tactic).expect("rank right winger");
    let before = planner_potential_state(&conn, save_id, snapshot_id);
    deny_potential_writes(&conn);

    let professional = optimize_depth(&conn, save_id).expect("optimize with a driven prospect");
    set_development(&conn, save_id, 78, 20, [5, 8, 9]);
    let unprofessional = optimize_depth(&conn, save_id).expect("optimize with a lazy prospect");

    assert_eq!(
        assigned_player_uid(&professional, PlannerTeam::Senior, "right_winger"),
        Some(78)
    );
    assert_eq!(
        assigned_player_uid(&unprofessional, PlannerTeam::Senior, "right_winger"),
        Some(77)
    );
    assert_eq!(
        planner_potential_state(&conn, save_id, snapshot_id).compact_rows,
        before.compact_rows
    );
}

#[test]
fn a_development_slot_lists_the_higher_ceiling_before_the_better_player_today() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    set_right_winger(&conn, save_id, 77, 58, 86);
    set_development(&conn, save_id, 77, 16, [14, 12, 13]);
    set_right_winger(&conn, save_id, 78, 66, 72);
    set_development(&conn, save_id, 78, 20, [14, 12, 13]);
    keep_teams(&conn, save_id, &[("reserves", Some(23))]);

    let optimized = optimize_depth(&conn, save_id).expect("optimize the reserves");

    // The reserves weigh the future more than today, and gain a string for the second winger.
    assert_eq!(
        lane_players(&optimized, PlannerTeam::Reserves, "right_winger"),
        [Some(77), Some(78)]
    );

    set_right_winger_scores(&conn, save_id, 78, None);
    let shallower = optimize_depth(&conn, save_id).expect("optimize without the second winger");

    assert_eq!(
        lane_players(&shallower, PlannerTeam::Reserves, "right_winger"),
        [Some(77)]
    );
}

#[test]
fn optimizer_keeps_players_over_a_teams_age_limit_out_of_its_squads() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    set_right_winger(&conn, save_id, 77, 100, 100);
    set_player_age(&conn, save_id, 77, Some(22));
    set_right_winger(&conn, save_id, 78, 50, 50);
    set_player_age(&conn, save_id, 78, Some(21));
    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    tactic.lanes[9].importance_rank = Some(1);
    tactic::save_tactic(&conn, save_id, &tactic).expect("rank right winger");

    keep_teams(&conn, save_id, &[("reserves", Some(21))]);
    let limited = optimize_depth(&conn, save_id).expect("optimize under the limit");
    keep_teams(&conn, save_id, &[("reserves", None)]);
    let unlimited = optimize_depth(&conn, save_id).expect("optimize without a limit");

    assert_eq!(
        assigned_player_uid(&limited, PlannerTeam::Reserves, "right_winger"),
        Some(78)
    );
    assert_eq!(
        assigned_player_uid(&unlimited, PlannerTeam::Reserves, "right_winger"),
        Some(77)
    );
}

#[test]
fn the_senior_squad_fills_before_the_other_teams_take_the_remaining_players() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    for (player_uid, score) in [(77, 100), (78, 90), (79, 80), (80, 70)] {
        set_right_winger(&conn, save_id, player_uid, score, score);
        set_player_age(&conn, save_id, player_uid, Some(18));
    }

    let optimized = optimize_depth(&conn, save_id).expect("optimize planner depth");

    // Starter, Backup, then one rotation player for the slot; the fourth winger is young
    // enough for the youth team, which comes before the reserves.
    assert_eq!(
        lane_players(&optimized, PlannerTeam::Senior, "right_winger"),
        [Some(77), Some(78), Some(79)]
    );
    assert_eq!(
        lane_players(&optimized, PlannerTeam::Reserves, "right_winger"),
        [None]
    );
    assert_eq!(
        lane_players(&optimized, PlannerTeam::Youth, "right_winger"),
        [Some(80)]
    );
}

fn rotation_candidate(player_uid: i64, lane_index: usize, score: u16) -> OptimizerCandidate {
    let mut lane_scores = vec![None; tactic::TACTIC_LANE_COUNT];
    lane_scores[lane_index] = Some(score);
    OptimizerCandidate {
        player_uid,
        last_known_name: format!("Player {player_uid}"),
        lane_scores,
    }
}

#[test]
fn the_rotation_keeps_a_goalkeeper_and_the_best_of_one_more_player_per_slot() {
    let tactic = tactic::default_tactic();
    let every_lane = (0..tactic.lanes.len()).collect::<Vec<_>>();
    // The goalkeeper is the weakest candidate. Each outfield lane has a candidate worth ten
    // times its index, and the right centre-backs' lane has a second, better one.
    let mut candidates = vec![rotation_candidate(100, 0, 1)];
    candidates.extend((1..=10).map(|lane_index| {
        rotation_candidate(100 + lane_index as i64, lane_index, lane_index as u16 * 10)
    }));
    candidates.push(rotation_candidate(200, 3, 500));

    let rotation = allocate_rotation(&tactic, &every_lane, &candidates, &HashSet::new(), 8);
    let mut picked = rotation
        .iter()
        .map(|&(lane_index, candidate_index)| (lane_index, candidates[candidate_index].player_uid))
        .collect::<Vec<_>>();
    picked.sort_unstable();

    // Lanes 1 and 2 hold the weakest outfield candidates, and lane 3 takes only its best.
    assert_eq!(
        picked,
        [
            (0, 100),
            (3, 200),
            (5, 105),
            (6, 106),
            (7, 107),
            (8, 108),
            (9, 109),
            (10, 110),
        ]
    );
    // Pinned players leave fewer places, and the goalkeeper takes the first of them.
    assert_eq!(
        allocate_rotation(&tactic, &every_lane, &candidates, &HashSet::new(), 1),
        [(0, 0)]
    );
}

#[test]
fn a_development_player_goes_to_the_slot_he_stands_out_in_not_his_highest_score() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    let rivals = (201..=210).collect::<Vec<_>>();
    seed_players(
        &temp_dir,
        &mut conn,
        save_id,
        &[&[77][..], &rivals].concat(),
    );
    clear_current_scores_except(&conn, save_id, &[&[77][..], &rivals].concat(), &[]);
    set_player_age(&conn, save_id, 77, Some(20));
    set_player_positions(&conn, save_id, 77, r#"{"AMR": 18, "MR": 18, "ST": 18}"#);
    set_lane_scores(&conn, save_id, 77, "right_winger", 62, 62);
    set_lane_scores(&conn, save_id, 77, "centre_forward", 58, 58);
    // Across the save, right wingers score in the 70s and centre-forwards in the 40s.
    for (index, rival) in rivals.iter().copied().enumerate() {
        set_player_club(&conn, save_id, rival, "Elsewhere FC");
        let step = (index / 2) as u8;
        if index % 2 == 0 {
            set_player_positions(&conn, save_id, rival, r#"{"AMR": 18, "MR": 18}"#);
            set_lane_scores(&conn, save_id, rival, "right_winger", 70 + step, 70 + step);
        } else {
            set_player_positions(&conn, save_id, rival, r#"{"ST": 18}"#);
            set_lane_scores(
                &conn,
                save_id,
                rival,
                "centre_forward",
                40 + step,
                40 + step,
            );
        }
    }
    keep_teams(&conn, save_id, &[("reserves", Some(23))]);

    let optimized = optimize_depth(&conn, save_id).expect("optimize the reserves");

    assert_eq!(
        lane_players(&optimized, PlannerTeam::Reserves, "centre_forward"),
        [Some(77)]
    );
    assert_eq!(
        lane_players(&optimized, PlannerTeam::Reserves, "right_winger"),
        [None]
    );
}

#[test]
fn a_development_player_joins_the_youngest_team_whose_age_limit_takes_him() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    for (player_uid, age) in [(77, Some(17)), (78, Some(20)), (79, Some(25)), (80, None)] {
        set_right_winger(&conn, save_id, player_uid, 60, 60);
        set_player_age(&conn, save_id, player_uid, age);
    }
    keep_teams(
        &conn,
        save_id,
        &[("reserves", Some(23)), ("youth", Some(18))],
    );

    let optimized = optimize_depth(&conn, save_id).expect("optimize the development teams");

    assert_eq!(
        lane_players(&optimized, PlannerTeam::Youth, "right_winger"),
        [Some(77)]
    );
    // A player too old for both, or of unknown age, stays out of every squad.
    assert_eq!(
        lane_players(&optimized, PlannerTeam::Reserves, "right_winger"),
        [Some(78)]
    );
}

#[test]
fn a_development_slot_fills_around_a_pinned_player() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    for (player_uid, score) in [(77, 80), (78, 70), (79, 40)] {
        set_right_winger(&conn, save_id, player_uid, score, score);
        set_player_age(&conn, save_id, player_uid, Some(20));
    }
    keep_teams(&conn, save_id, &[("reserves", Some(23))]);
    let depth = get_depth(&conn, save_id).expect("load planner depth");
    let first_string_id = team_strings(&depth, PlannerTeam::Reserves)[0].id;
    assign_player(&conn, save_id, first_string_id, "right_winger", 79)
        .expect("pin the weakest winger first");

    let optimized = optimize_depth(&conn, save_id).expect("optimize around the pin");

    assert_eq!(
        lane_players(&optimized, PlannerTeam::Reserves, "right_winger"),
        [Some(79), Some(77), Some(78)]
    );

    // Clearing every squad leaves the team its first string only.
    clear_all(&conn, save_id, true).expect("clear every squad");
    let cleared = get_depth(&conn, save_id).expect("load cleared depth");

    assert_eq!(
        lane_players(&cleared, PlannerTeam::Reserves, "right_winger"),
        [None]
    );
}

#[test]
fn optimizer_rejects_a_missing_potential_role_before_assignment_writes() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "DELETE FROM player_role_metrics
         WHERE snapshot_id = ?1 AND uid = 77",
        [snapshot_id],
    )
    .expect("remove compact row");
    let before = planner_potential_state(&conn, save_id, snapshot_id);
    deny_potential_writes(&conn);

    let error = optimize_depth(&conn, save_id).expect_err("reject incomplete potential state");

    assert_eq!(error, "Current potential snapshot is incomplete");
    assert_eq!(planner_potential_state(&conn, save_id, snapshot_id), before);
}

#[test]
fn optimizer_rejects_a_wrong_version_potential_role_before_assignment_writes() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "UPDATE player_role_metrics
         SET projection_model_version = 999
         WHERE snapshot_id = ?1 AND uid = 77",
        [snapshot_id],
    )
    .expect("set stale compact projection version");
    let before = planner_potential_state(&conn, save_id, snapshot_id);
    deny_potential_writes(&conn);

    let error = optimize_depth(&conn, save_id).expect_err("reject stale potential state");

    assert_eq!(error, "Current potential snapshot is incomplete");
    assert_eq!(planner_potential_state(&conn, save_id, snapshot_id), before);
}

#[test]
fn optimizer_skips_an_unavailable_team() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    set_right_winger_scores(&conn, save_id, 79, Some(100));
    keep_teams(&conn, save_id, &[("senior", None), ("youth", Some(18))]);

    let optimized = optimize_depth(&conn, save_id).expect("optimize planner depth");

    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Senior, "right_winger"),
        Some(79)
    );
    assert!(!optimized
        .teams
        .iter()
        .any(|team| team.team == PlannerTeam::Reserves));
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
    clear_current_scores_except(
        &conn,
        save_id,
        &[77, 78],
        &["winger_ip", "tracking_wide_midfielder_oop"],
    );
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
    clear_current_scores_except(
        &conn,
        save_id,
        &[77, 78],
        &["winger_ip", "tracking_wide_midfielder_oop"],
    );
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
    clear_current_scores_except(
        &conn,
        save_id,
        &[77, 78],
        &[
            "inside_winger_ip",
            "winger_ip",
            "tracking_wide_midfielder_oop",
        ],
    );
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
fn optimizer_ignores_stored_importance_ranks() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    clear_current_scores_except(
        &conn,
        save_id,
        &[77, 78],
        &["winger_ip", "tracking_wide_midfielder_oop"],
    );
    set_right_winger_scores(&conn, save_id, 77, Some(100));
    set_right_winger_scores(&conn, save_id, 78, Some(100));
    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    set_player_positions(&conn, save_id, 77, r#"{"ML": 18, "MR": 18, "AMR": 18}"#);
    // Only player 77 can play the left winger slot, which defends at ML.
    set_player_positions(&conn, save_id, 78, r#"{"MR": 18, "AMR": 18}"#);
    tactic.lanes[8].ip_position = "MR".to_string();
    tactic.lanes[8].oop_position = "ML".to_string();
    // Filling the right winger first would hand it player 77 and leave the left winger empty.
    tactic.lanes[9].importance_rank = Some(1);
    tactic::save_tactic(&conn, save_id, &tactic).expect("save ranked tactic");

    let optimized = optimize_depth(&conn, save_id).expect("optimize ranked tactic");

    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Senior, "right_winger"),
        Some(78)
    );
    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Senior, "left_winger"),
        Some(77)
    );
}

#[test]
fn optimizer_leaves_a_ranked_lane_blank_without_reserving_candidates() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    clear_current_scores_except(
        &conn,
        save_id,
        &[77, 78],
        &["winger_ip", "tracking_wide_midfielder_oop"],
    );
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
fn a_development_team_requires_a_known_age_both_positions_and_complete_scores() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    set_player_age(&conn, save_id, 78, None);
    set_right_winger_scores(&conn, save_id, 78, Some(100));
    set_player_age(&conn, save_id, 79, Some(23));
    set_right_winger_scores(&conn, save_id, 79, Some(0));
    set_player_age(&conn, save_id, 80, Some(23));
    set_player_positions(&conn, save_id, 80, r#"{"AMR": 18}"#);
    set_right_winger_scores(&conn, save_id, 80, Some(100));
    keep_teams(&conn, save_id, &[("reserves", Some(23))]);

    let optimized = optimize_depth(&conn, save_id).expect("optimize the reserves");

    assert_eq!(
        lane_players(&optimized, PlannerTeam::Reserves, "right_winger"),
        [Some(79)]
    );
    assert!(!optimized
        .teams
        .iter()
        .flat_map(|team| &team.strings)
        .flat_map(|planner_string| &planner_string.assignments)
        .any(|assignment| assignment.player_uid == 78 || assignment.player_uid == 80));
}

#[test]
fn optimizer_allocates_the_shared_managed_club_pool_in_team_order() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    set_right_winger_scores(&conn, save_id, 77, None);
    set_right_winger_scores(&conn, save_id, 78, None);
    set_player_age(&conn, save_id, 79, Some(18));
    set_right_winger_scores(&conn, save_id, 79, Some(100));
    set_right_winger_scores(&conn, save_id, 80, None);
    conn.execute(
        "UPDATE players SET team_level = NULL WHERE snapshot_id = ?1 AND uid = 79",
        [current_snapshot_id(&conn, save_id)],
    )
    .expect("remove candidate team level");

    let optimized = optimize_depth(&conn, save_id).expect("optimize planner depth");

    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Senior, "right_winger"),
        Some(79)
    );
    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Reserves, "right_winger"),
        None
    );
    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Youth, "right_winger"),
        None
    );
}

#[test]
fn the_optimizer_leaves_out_a_namesake_clubs_player() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    clear_current_scores_except(
        &conn,
        save_id,
        &[77, 78],
        &["winger_ip", "tracking_wide_midfielder_oop"],
    );
    set_right_winger_scores(&conn, save_id, 77, Some(70));
    set_right_winger_scores(&conn, save_id, 78, Some(90));
    move_to_namesake_club(&conn, save_id, 78);
    keep_teams(&conn, save_id, &[("senior", None)]);

    let optimized = optimize_depth(&conn, save_id).expect("optimize the managed club");
    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Senior, "right_winger"),
        Some(77)
    );

    // A club chosen before UIDs were recorded matches by name.
    forget_managed_club_uid(&conn, save_id);
    let optimized = optimize_depth(&conn, save_id).expect("optimize by club name");
    assert_eq!(
        assigned_player_uid(&optimized, PlannerTeam::Senior, "right_winger"),
        Some(78)
    );
}

#[test]
fn the_senior_squad_alone_does_not_load_scores_outside_the_managed_club() {
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
    .expect("exclude player from managed club");
    conn.execute_batch("PRAGMA ignore_check_constraints = ON")
        .expect("allow invalid outside-club score");
    conn.execute(
        "UPDATE player_role_metrics SET winger_ip = 'invalid' WHERE snapshot_id = ?1 AND uid = 80",
        params![snapshot_id],
    )
    .expect("set invalid outside-club score");
    conn.execute_batch("PRAGMA ignore_check_constraints = OFF")
        .expect("restore score constraints");
    // Only the other teams rank their players against the whole save.
    keep_teams(&conn, save_id, &[("senior", None)]);

    optimize_depth(&conn, save_id).expect("ignore scores outside managed club");
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
    keep_teams(&conn, save_id, &[("reserves", Some(23))]);

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
fn optimizer_keeps_a_pinned_pick_and_replaces_it_once_unpinned() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    set_right_winger(&conn, save_id, 77, 90, 90);
    set_right_winger(&conn, save_id, 78, 50, 50);
    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    tactic.lanes[9].importance_rank = Some(1);
    tactic::save_tactic(&conn, save_id, &tactic).expect("rank right winger");
    let first = optimize_depth(&conn, save_id).expect("run first optimization");
    let senior_string_id = team_strings(&first, PlannerTeam::Senior)[0].id;
    assert_eq!(
        assigned_player_uid(&first, PlannerTeam::Senior, "right_winger"),
        Some(77)
    );
    set_right_winger(&conn, save_id, 78, 100, 100);

    set_assignment_pinned(&conn, save_id, senior_string_id, "right_winger", true)
        .expect("pin the optimizer's pick");
    let pinned = optimize_depth(&conn, save_id).expect("optimize around the pin");
    set_assignment_pinned(&conn, save_id, senior_string_id, "right_winger", false)
        .expect("unpin the pick");
    let unpinned = optimize_depth(&conn, save_id).expect("optimize without the pin");

    assert_eq!(
        assigned_player_uid(&pinned, PlannerTeam::Senior, "right_winger"),
        Some(77)
    );
    assert_eq!(
        assigned_player_uid(&unpinned, PlannerTeam::Senior, "right_winger"),
        Some(78)
    );
}

#[test]
fn unassigned_players_list_everyone_left_out_with_his_best_slot() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    clear_current_scores_except(
        &conn,
        save_id,
        &[77, 78, 79, 80],
        &["winger_ip", "tracking_wide_midfielder_oop"],
    );
    set_right_winger(&conn, save_id, 77, 90, 90);
    set_right_winger(&conn, save_id, 78, 60, 80);
    set_development(&conn, save_id, 78, 20, [15, 15, 15]);
    set_right_winger(&conn, save_id, 79, 70, 70);
    set_right_winger_scores(&conn, save_id, 80, None);
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let senior_string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    assign_player(&conn, save_id, senior_string_id, "right_winger", 77)
        .expect("place the best winger");

    let unassigned = unassigned_players(&conn, save_id).expect("list unassigned players");

    // The placed player is gone, the rest come best first, and a player who suits no lane
    // comes last without a slot. At 20 with a drive of 15, uid 78 closes 85% of his gap.
    assert_eq!(
        unassigned
            .iter()
            .map(|player| {
                (
                    player.player_uid,
                    player.best_lane.as_ref().map(|lane| {
                        (
                            lane.lane_id.as_str(),
                            lane.current_score,
                            lane.expected_score,
                        )
                    }),
                )
            })
            .collect::<Vec<_>>(),
        [
            (79, Some(("right_winger", 70, 70))),
            (78, Some(("right_winger", 60, 77))),
            (80, None),
        ]
    );
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

/// Player 77, aged 19, plays right winger and centre forward. He is the club's best winger
/// today, but his potential ranks below the other winger's and above every striker's, so he is
/// developing into a centre forward. The others are 30.
fn winger_developing_into_a_striker(
    strikers: &[i64],
) -> (tempfile::TempDir, rusqlite::Connection, i64) {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    let mut uids = vec![77, 201];
    uids.extend(strikers);
    seed_players(&temp_dir, &mut conn, save_id, &uids);
    set_player_positions(&conn, save_id, 77, r#"{"AMR": 18, "MR": 18, "ST": 18}"#);
    set_lane_scores(&conn, save_id, 77, "right_winger", 70, 75);
    set_lane_scores(&conn, save_id, 77, "centre_forward", 60, 85);
    set_player_age(&conn, save_id, 77, Some(19));
    set_player_positions(&conn, save_id, 201, r#"{"AMR": 18, "MR": 18}"#);
    set_lane_scores(&conn, save_id, 201, "right_winger", 50, 90);
    set_player_age(&conn, save_id, 201, Some(30));
    for &striker in strikers {
        set_player_positions(&conn, save_id, striker, r#"{"ST": 18}"#);
        set_lane_scores(&conn, save_id, striker, "centre_forward", 75, 75);
        set_player_age(&conn, save_id, striker, Some(30));
    }
    (temp_dir, conn, save_id)
}

#[test]
fn optimizing_for_development_keeps_a_young_player_in_his_development_slot() {
    let (_temp_dir, conn, save_id) = winger_developing_into_a_striker(&[204]);

    // The strongest squad starts him on the wing, where the club needs him today.
    let strongest = optimize_depth(&conn, save_id).expect("optimize for the strongest squads");
    assert_eq!(
        lane_players(&strongest, PlannerTeam::Senior, "right_winger")[0],
        Some(77)
    );
    assert_eq!(
        lane_players(&strongest, PlannerTeam::Senior, "centre_forward")[0],
        Some(204)
    );
    assert_eq!(strongest.starter_comparison, None);

    // For development he takes only the striker's slot. The older striker is the better one
    // today, so he sits behind him, and the weaker winger starts.
    set_optimize_for(&conn, save_id, OptimizeFor::Development).expect("optimize for development");
    let developed = optimize_depth(&conn, save_id).expect("optimize for development");
    assert_eq!(
        lane_players(&developed, PlannerTeam::Senior, "right_winger")[..2],
        [Some(201), None]
    );
    assert_eq!(
        lane_players(&developed, PlannerTeam::Senior, "centre_forward")[..2],
        [Some(204), Some(77)]
    );
    assert_eq!(
        developed.starter_comparison,
        Some(StarterComparison {
            starters: 2,
            score: 50 + 75,
            strongest_starters: 2,
            strongest_score: 70 + 75,
        })
    );
    // The comparison belongs to the Optimize result, not to the board.
    assert_eq!(
        get_depth(&conn, save_id)
            .expect("read the board")
            .starter_comparison,
        None
    );
}

#[test]
fn a_development_slot_suggested_in_any_position_places_a_player_he_is_to_learn() {
    let (_temp_dir, conn, save_id) = winger_developing_into_a_striker(&[204]);
    // He has a striker's scores but can no longer play up front.
    set_player_positions(&conn, save_id, 77, r#"{"AMR": 18, "MR": 18}"#);
    set_optimize_for(&conn, save_id, OptimizeFor::Development).expect("optimize for development");

    // By position he develops on the wing, where he starts.
    let by_position = optimize_depth(&conn, save_id).expect("optimize for development");
    assert_eq!(
        lane_players(&by_position, PlannerTeam::Senior, "right_winger")[0],
        Some(77)
    );

    // Suggested on role scores alone, he is growing into a striker, and Optimize puts him
    // behind the striker although it places everyone else by position.
    set_position_familiarity(&conn, save_id, false).expect("suggest in any position");
    let anywhere = optimize_depth(&conn, save_id).expect("optimize for development");
    assert_eq!(
        lane_players(&anywhere, PlannerTeam::Senior, "centre_forward")[..2],
        [Some(204), Some(77)]
    );
    assert_eq!(
        lane_players(&anywhere, PlannerTeam::Senior, "right_winger")[0],
        Some(201)
    );
    let backup = &team_strings(&anywhere, PlannerTeam::Senior)[1];
    assert!(
        backup
            .assignments
            .iter()
            .find(|assignment| assignment.player_uid == 77)
            .expect("77 is placed")
            .out_of_position
    );
}

#[test]
fn outside_the_senior_squad_a_developing_player_takes_his_development_slot() {
    // Three better strikers fill the senior squad's centre-forward slots.
    let (_temp_dir, conn, save_id) = winger_developing_into_a_striker(&[204, 205, 206]);
    set_optimize_for(&conn, save_id, OptimizeFor::Development).expect("optimize for development");

    let depth = optimize_depth(&conn, save_id).expect("optimize for development");

    assert_eq!(
        lane_players(&depth, PlannerTeam::Senior, "centre_forward"),
        vec![Some(204), Some(205), Some(206)]
    );
    assert_eq!(
        lane_players(&depth, PlannerTeam::Senior, "right_winger"),
        vec![Some(201), None, None],
        "the wing stays closed to him"
    );
    assert_eq!(
        lane_players(&depth, PlannerTeam::Reserves, "centre_forward"),
        vec![Some(77)]
    );
}

#[test]
fn a_developing_player_fills_any_slot_that_trains_as_his_development_slot() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    seed_players(&temp_dir, &mut conn, save_id, &[77, 201]);
    for (uid, age, current) in [(77, 19, 60), (201, 30, 70)] {
        set_player_positions(&conn, save_id, uid, r#"{"DC": 18}"#);
        set_lane_scores(&conn, save_id, uid, "left_centre_back", current, 80);
        set_player_age(&conn, save_id, uid, Some(age));
    }
    set_optimize_for(&conn, save_id, OptimizeFor::Development).expect("optimize for development");

    let depth = optimize_depth(&conn, save_id).expect("optimize for development");

    // The two centre-back slots ask for the same roles, so both players start.
    let starters = ["left_centre_back", "right_centre_back"]
        .map(|lane_id| lane_players(&depth, PlannerTeam::Senior, lane_id)[0]);
    assert_eq!(
        starters.into_iter().flatten().collect::<HashSet<_>>(),
        HashSet::from([77, 201])
    );
}

#[test]
fn a_chosen_development_slot_places_a_player_he_cannot_yet_play() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    seed_players(&temp_dir, &mut conn, save_id, &[77]);
    set_right_winger(&conn, save_id, 77, 70, 80);
    set_player_age(&conn, save_id, 77, Some(19));
    // The left wing asks for the same roles at positions he has never played.
    choose_slot(
        &conn,
        save_id,
        current_snapshot_id(&conn, save_id),
        77,
        Some("left_winger"),
    )
    .expect("choose the left wing");
    set_optimize_for(&conn, save_id, OptimizeFor::Development).expect("optimize for development");

    let depth = optimize_depth(&conn, save_id).expect("optimize for development");

    assert_eq!(
        assigned_player_uid(&depth, PlannerTeam::Senior, "left_winger"),
        Some(77)
    );
    assert!(out_of_position(&depth, "left_winger"));
    assert_eq!(
        assigned_player_uid(&depth, PlannerTeam::Senior, "right_winger"),
        None
    );
}
