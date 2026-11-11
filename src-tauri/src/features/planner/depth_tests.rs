use rusqlite::{params, Connection};

use crate::features::managed_club::service as managed_club_service;
use crate::features::planner::tactic;
use crate::features::player_metrics::potential_scores;
use crate::features::scoring::combine::combine_role_scores;
use crate::features::snapshot;

use super::depth::{
    assign_player, clear_all, clear_assignment, get_depth, get_slot_candidates, move_player,
    set_assignment_pinned, AssignmentState, PlannerTeam,
};
use super::fit::{FootFit, PositionFit};
use super::teams::{save_team_settings, PlannerStringInput, PlannerTeamInput};
use super::test_support::{
    add_picker_candidates, assignment_provenance, current_snapshot_id, deny_potential_writes,
    open_with_snapshot, planner_potential_state, set_player_age, team_strings,
};

#[test]
fn returns_ranked_candidates_from_the_managed_club() {
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
        "UPDATE player_role_metrics
         SET goalkeeper_ip = CASE uid
             WHEN 78 THEN 50
             WHEN 79 THEN 80
             WHEN 80 THEN NULL
             ELSE goalkeeper_ip
         END,
         line_holding_keeper_oop = CASE uid
             WHEN 78 THEN 50
             WHEN 79 THEN 80
             WHEN 80 THEN NULL
             ELSE line_holding_keeper_oop
         END
         WHERE snapshot_id = ?1
           AND uid IN (77, 78, 79, 80)",
        params![snapshot_id],
    )
    .expect("set candidate scores");
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let reserve_string_id = team_strings(&depth, PlannerTeam::Reserves)[0].id;
    assign_player(&conn, save_id, reserve_string_id, "goalkeeper", 79)
        .expect("assign reserve player");

    let candidates = get_slot_candidates(&conn, save_id, PlannerTeam::Reserves, "goalkeeper", "")
        .expect("load reserve candidates");

    assert_eq!(
        candidates
            .iter()
            .map(|candidate| candidate.player_uid)
            .collect::<Vec<_>>(),
        [79, 78, 77, 80]
    );
    assert_eq!(candidates[0].combined_score, Some(80));
    assert_eq!(candidates[0].current_club, "Loan FC");
    assert_eq!(
        candidates[0].assignment_location.as_ref().map(|location| (
            location.team.as_str(),
            location.string_id,
            location.lane_id.as_str()
        )),
        Some(("reserves", reserve_string_id, "goalkeeper"))
    );

    let searched = get_slot_candidates(
        &conn,
        save_id,
        PlannerTeam::Reserves,
        "goalkeeper",
        "reserve",
    )
    .expect("search reserve candidates");
    assert_eq!(
        searched
            .iter()
            .map(|candidate| candidate.player_uid)
            .collect::<Vec<_>>(),
        [79, 78]
    );

    let error = get_slot_candidates(
        &conn,
        save_id,
        PlannerTeam::Reserves,
        "goalkeeper",
        &"x".repeat(121),
    )
    .expect_err("reject an unbounded search");
    assert_eq!(error, "Candidate search must be at most 120 characters");
}

#[test]
fn slot_candidates_use_the_selected_lane_weight() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    tactic.lanes[0].ip_weight = 1.0;
    tactic::save_tactic(&conn, save_id, &tactic).expect("save lane weight");
    let snapshot_id: i64 = conn
        .query_row(
            "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
            params![save_id],
            |row| row.get(0),
        )
        .expect("current snapshot");
    conn.execute(
        "UPDATE player_role_metrics
         SET goalkeeper_ip = 80,
             line_holding_keeper_oop = 60
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("set keeper scores");

    let candidates = get_slot_candidates(&conn, save_id, PlannerTeam::Senior, "goalkeeper", "")
        .expect("load candidates");

    assert_eq!(candidates[0].combined_score, Some(80));
}

#[test]
fn missing_team_level_does_not_restrict_the_managed_club_planner_pool() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();

    conn.execute(
        "UPDATE players SET team_level = NULL WHERE current_club = 'Loan FC'",
        [],
    )
    .expect("remove team level");

    for team in [
        PlannerTeam::Senior,
        PlannerTeam::Reserves,
        PlannerTeam::Youth,
    ] {
        assert_eq!(
            get_slot_candidates(&conn, save_id, team, "goalkeeper", "")
                .expect("load managed-club candidates")
                .iter()
                .map(|candidate| candidate.player_uid)
                .collect::<Vec<_>>(),
            [77]
        );
    }
}

#[test]
fn creates_one_default_string_for_each_team() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();

    let depth = get_depth(&conn, save_id).expect("create planner depth");

    assert_eq!(
        depth
            .teams
            .iter()
            .map(|team| (team.team.as_str(), team.strings.len()))
            .collect::<Vec<_>>(),
        [("senior", 1), ("reserves", 1), ("youth", 1)]
    );
    assert!(depth
        .teams
        .iter()
        .all(|team| team.strings[0].string_order == 0));
    assert!(depth
        .teams
        .iter()
        .all(|team| team.strings[0].display_name == "1st string"));
    let reloaded = get_depth(&conn, save_id).expect("reload planner depth");
    assert!(reloaded
        .teams
        .iter()
        .all(|team| team.strings[0].display_name == "1st string"));
}

fn retained_strings(
    depth: &super::depth::PlannerDepth,
    team: PlannerTeam,
) -> Vec<PlannerStringInput> {
    team_strings(depth, team)
        .iter()
        .map(|string| PlannerStringInput {
            id: Some(string.id),
            display_name: string.display_name.clone(),
        })
        .collect()
}

fn add_second_senior_string(conn: &Connection, save_id: i64) -> i64 {
    let depth = get_depth(conn, save_id).expect("load planner depth");
    let inputs = depth
        .teams
        .iter()
        .map(|team| {
            let mut strings = retained_strings(&depth, team.team);
            if team.team == PlannerTeam::Senior {
                strings.push(PlannerStringInput {
                    id: None,
                    display_name: "2nd string".to_string(),
                });
            }
            PlannerTeamInput {
                team: team.team.as_str().to_string(),
                display_name: team.display_name.clone(),
                max_age: team.max_age,
                strings,
            }
        })
        .collect::<Vec<_>>();
    save_team_settings(conn, save_id, &inputs, false).expect("add second senior string");
    let reloaded = get_depth(conn, save_id).expect("reload planner depth");
    team_strings(&reloaded, PlannerTeam::Senior)
        .iter()
        .map(|string| string.id)
        .max()
        .expect("new senior string id")
}

#[test]
fn restored_team_creates_the_ordinal_labeled_string() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("initialize planner depth");
    let senior_strings = retained_strings(&depth, PlannerTeam::Senior);
    let reserves_strings = retained_strings(&depth, PlannerTeam::Reserves);
    save_team_settings(
        &conn,
        save_id,
        &[
            PlannerTeamInput {
                team: "senior".to_string(),
                display_name: "Senior".to_string(),
                max_age: None,
                strings: senior_strings.clone(),
            },
            PlannerTeamInput {
                team: "reserves".to_string(),
                display_name: "Reserves".to_string(),
                max_age: Some(23),
                strings: reserves_strings.clone(),
            },
        ],
        false,
    )
    .expect("remove youth team");
    save_team_settings(
        &conn,
        save_id,
        &[
            PlannerTeamInput {
                team: "senior".to_string(),
                display_name: "Senior".to_string(),
                max_age: None,
                strings: senior_strings,
            },
            PlannerTeamInput {
                team: "reserves".to_string(),
                display_name: "Reserves".to_string(),
                max_age: Some(23),
                strings: reserves_strings,
            },
            PlannerTeamInput {
                team: "youth".to_string(),
                display_name: "Youth".to_string(),
                max_age: Some(18),
                strings: vec![PlannerStringInput {
                    id: None,
                    display_name: String::new(),
                }],
            },
        ],
        false,
    )
    .expect("restore youth team");

    let reloaded = get_depth(&conn, save_id).expect("reload depth");
    let strings = team_strings(&reloaded, PlannerTeam::Youth);
    assert_eq!(strings.len(), 1);
    assert_eq!(strings[0].string_order, 0);
    assert_eq!(strings[0].display_name, "1st string");
}

#[test]
fn direct_depth_commands_reject_an_unavailable_team_without_recreating_it() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("initialize planner depth");
    save_team_settings(
        &conn,
        save_id,
        &[PlannerTeamInput {
            team: "senior".to_string(),
            display_name: "Senior".to_string(),
            max_age: None,
            strings: retained_strings(&depth, PlannerTeam::Senior),
        }],
        false,
    )
    .expect("remove unused teams");

    let candidate_error =
        get_slot_candidates(&conn, save_id, PlannerTeam::Reserves, "goalkeeper", "")
            .expect_err("reject loading candidates for an unavailable team");
    assert!(candidate_error.contains("not available"));
    assert!(!get_depth(&conn, save_id)
        .expect("reload depth")
        .teams
        .iter()
        .any(|team| team.team == PlannerTeam::Reserves));
}

#[test]
fn assignments_report_age_pins_and_players_over_their_teams_age_limit() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    set_player_age(&conn, save_id, 79, Some(24));
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let string_id = team_strings(&depth, PlannerTeam::Reserves)[0].id;
    assign_player(&conn, save_id, string_id, "goalkeeper", 79).expect("assign an older player");
    let reserve_keeper = |conn: &Connection| {
        let depth = get_depth(conn, save_id).expect("reload planner depth");
        team_strings(&depth, PlannerTeam::Reserves)[0].assignments[0].clone()
    };

    let placed_by_hand = reserve_keeper(&conn);
    assert_eq!(placed_by_hand.age, Some(24));
    assert!(placed_by_hand.pinned);
    assert!(placed_by_hand.over_age);

    set_assignment_pinned(&conn, save_id, string_id, "goalkeeper", false).expect("unpin");
    set_player_age(&conn, save_id, 79, Some(23));
    let unpinned = reserve_keeper(&conn);
    assert!(!unpinned.pinned);
    assert!(!unpinned.over_age);
    assert_eq!(assignment_provenance(&conn, 79), "optimizer");

    set_assignment_pinned(&conn, save_id, string_id, "goalkeeper", true).expect("pin again");
    assert!(reserve_keeper(&conn).pinned);
    assert_eq!(
        set_assignment_pinned(&conn, save_id, string_id, "right_back", true)
            .expect_err("reject an empty cell"),
        "Planner cell is empty"
    );
}

#[test]
fn assigning_or_moving_into_an_occupied_cell_leaves_its_player_unassigned() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let senior = team_strings(&depth, PlannerTeam::Senior)[0].id;
    let reserves = team_strings(&depth, PlannerTeam::Reserves)[0].id;
    let placed = |conn: &Connection| {
        conn.prepare(
            "SELECT string_id, player_uid FROM planner_assignments
             WHERE save_id = ?1 ORDER BY player_uid",
        )
        .expect("prepare assignments")
        .query_map([save_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })
        .expect("query assignments")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect assignments")
    };
    assign_player(&conn, save_id, senior, "goalkeeper", 77).expect("fill the senior cell");
    assign_player(&conn, save_id, reserves, "goalkeeper", 79).expect("fill the reserves cell");

    assign_player(&conn, save_id, senior, "goalkeeper", 78)
        .expect("replace with an unassigned player");
    assert_eq!(placed(&conn), [(senior, 78), (reserves, 79)]);

    move_player(&conn, save_id, senior, "goalkeeper", 79)
        .expect("replace with a player from another cell");
    assert_eq!(placed(&conn), [(senior, 79)]);
}

#[test]
fn slot_candidates_report_age_expected_score_and_fit_for_the_team() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute_batch(&format!(
        "UPDATE players
         SET age = 20,
             preferred_foot = 'right',
             positions_json = '{{\"AMR\": 18, \"MR\": 18}}',
             personality_json = '{{\"Professionalism\": 15, \"Ambition\": 15}}',
             attributes_json = '{{\"Determination\": 15}}'
         WHERE snapshot_id = {snapshot_id} AND uid = 78;
         UPDATE player_role_metrics
         SET winger_ip = 60,
             tracking_wide_midfielder_oop = 60,
             potential_winger_ip = 80,
             potential_tracking_wide_midfielder_oop = 80
         WHERE snapshot_id = {snapshot_id} AND uid = 78;
         UPDATE players
         SET age = 24,
             preferred_foot = 'left',
             positions_json = '{{\"AMR\": 13, \"MR\": 18}}'
         WHERE snapshot_id = {snapshot_id} AND uid = 79;
         UPDATE players
         SET positions_json = '{{\"AMR\": 18}}'
         WHERE snapshot_id = {snapshot_id} AND uid = 80;"
    ))
    .expect("set candidate facts");
    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    tactic.lanes[9].preferred_foot = "right".to_string();
    tactic.lanes[9].foot_preference = "strict".to_string();
    tactic::save_tactic(&conn, save_id, &tactic).expect("require a right foot");
    let candidate = |team: PlannerTeam, player_uid: i64| {
        get_slot_candidates(&conn, save_id, team, "right_winger", "")
            .expect("load slot candidates")
            .into_iter()
            .find(|candidate| candidate.player_uid == player_uid)
            .expect("candidate")
    };

    // At 20 with a drive of 15, the player closes 85% of the 20-point gap.
    let suited = candidate(PlannerTeam::Reserves, 78);
    assert_eq!((suited.age, suited.over_age), (Some(20), false));
    assert_eq!(
        (suited.combined_score, suited.expected_combined_score),
        (Some(60), Some(77))
    );
    assert_eq!(
        (suited.position_fit, suited.foot_fit),
        (PositionFit::Suited, FootFit::Matches)
    );

    let older = candidate(PlannerTeam::Reserves, 79);
    assert!(older.over_age);
    assert_eq!(
        (older.position_fit, older.foot_fit),
        (PositionFit::Unfamiliar, FootFit::Excluded)
    );
    assert!(!candidate(PlannerTeam::Senior, 79).over_age);
    assert_eq!(
        candidate(PlannerTeam::Reserves, 80).position_fit,
        PositionFit::Unsuited
    );
}

#[test]
fn assign_and_move_player_persist_manual_provenance() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let first_string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    let second_string_id = add_second_senior_string(&conn, save_id);

    assign_player(&conn, save_id, first_string_id, "goalkeeper", 77).expect("assign player");
    assert_eq!(assignment_provenance(&conn, 77), "manual");

    move_player(&conn, save_id, second_string_id, "goalkeeper", 77).expect("move player");
    assert_eq!(assignment_provenance(&conn, 77), "manual");
}

#[test]
fn unconfirmed_clear_returns_confirmation_before_potential_preflight() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    assign_player(&conn, save_id, string_id, "goalkeeper", 77).expect("assign player");
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "UPDATE player_role_metrics
         SET projection_model_version = 999
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("corrupt compact projection version");
    let before = planner_potential_state(&conn, save_id, snapshot_id);

    let error = clear_all(&conn, save_id, false).expect_err("require confirmation");
    assert_eq!(error, "Clearing all squads requires confirmation");
    assert_eq!(planner_potential_state(&conn, save_id, snapshot_id), before);
}

#[test]
fn clearing_all_requires_confirmation_and_preserves_other_saves_and_settings() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let senior_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    let reserve_id = team_strings(&depth, PlannerTeam::Reserves)[0].id;
    let youth_id = team_strings(&depth, PlannerTeam::Youth)[0].id;
    assign_player(&conn, save_id, senior_id, "goalkeeper", 77).expect("assign senior keeper");
    assign_player(&conn, save_id, reserve_id, "goalkeeper", 79).expect("assign reserve keeper");
    assign_player(&conn, save_id, youth_id, "goalkeeper", 80).expect("assign youth keeper");
    conn.execute(
        "INSERT INTO planner_assignments (
            save_id, string_id, lane_id, player_uid, last_known_name, provenance
         ) VALUES (?1, ?2, ?3, ?4, ?5, 'optimizer')",
        params![save_id, senior_id, "left_back", 78, "Senior optimizer"],
    )
    .expect("assign optimizer row");

    let before_tactic = tactic::get_tactic(&conn, save_id).expect("load tactic before clear");
    let before_managed_club = managed_club_service::get_managed_club(&conn, save_id)
        .expect("load managed club before clear");
    let before_string_ids = get_depth(&conn, save_id)
        .expect("reload before clear")
        .teams
        .into_iter()
        .map(|team| {
            (
                team.team,
                team.strings
                    .into_iter()
                    .map(|planner_string| planner_string.id)
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();

    let error = clear_all(&conn, save_id, false).expect_err("require confirmation");
    assert!(error.contains("requires confirmation"));
    assert_eq!(
        get_depth(&conn, save_id)
            .expect("reload after rejected clear")
            .teams
            .iter()
            .map(|team| team.strings[0].assignments.len())
            .collect::<Vec<_>>(),
        [2, 1, 1]
    );

    conn.execute(
        "INSERT INTO saves (name, is_active) VALUES ('Second save', 0)",
        [],
    )
    .expect("create second save");
    let second_save_id = conn.last_insert_rowid();
    let second_dump_path = temp_dir.path().join("second-save.json");
    std::fs::write(
        &second_dump_path,
        include_str!("../memory_read/fixtures/golden_dump_v9.json"),
    )
    .expect("write second save dump");
    snapshot::ingest::ingest_dump_file_for_save(&mut conn, second_save_id, &second_dump_path)
        .expect("ingest second save");
    managed_club_service::set_managed_club(&conn, second_save_id, "Loan FC", Some(1000))
        .expect("configure second save");
    let second_depth = get_depth(&conn, second_save_id).expect("create second depth");
    let second_string_id = team_strings(&second_depth, PlannerTeam::Senior)[0].id;
    assign_player(&conn, second_save_id, second_string_id, "goalkeeper", 77)
        .expect("assign second-save player");

    clear_all(&conn, save_id, true).expect("clear every team");
    let reloaded = get_depth(&conn, save_id).expect("reload after clear");
    assert!(reloaded.teams.iter().all(|team| team
        .strings
        .iter()
        .all(|planner_string| planner_string.assignments.is_empty())));
    assert_eq!(
        reloaded
            .teams
            .iter()
            .map(|team| {
                (
                    team.team,
                    team.strings
                        .iter()
                        .map(|planner_string| planner_string.id)
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>(),
        before_string_ids
    );
    assert_eq!(reloaded.tactic, before_tactic);
    assert_eq!(
        managed_club_service::get_managed_club(&conn, save_id)
            .expect("load managed club after clear"),
        before_managed_club
    );
    assert_eq!(
        team_strings(
            &get_depth(&conn, second_save_id).expect("reload second save"),
            PlannerTeam::Senior
        )[0]
        .assignments
        .len(),
        1
    );
}

#[test]
fn enforces_player_uniqueness_and_moves_in_one_save() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let first_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    let reserve_id = team_strings(&depth, PlannerTeam::Reserves)[0].id;
    let second_id = add_second_senior_string(&conn, save_id);
    assign_player(&conn, save_id, first_id, "goalkeeper", 77).expect("assign player");

    let error = assign_player(&conn, save_id, second_id, "goalkeeper", 77)
        .expect_err("reject duplicate player");
    assert!(error.contains("already assigned"));
    let error = assign_player(&conn, save_id, reserve_id, "goalkeeper", 77)
        .expect_err("reject duplicate player across teams");
    assert!(error.contains("already assigned"));

    move_player(&conn, save_id, reserve_id, "goalkeeper", 77).expect("move player across teams");
    let reloaded = get_depth(&conn, save_id).expect("reload depth");
    let strings = team_strings(&reloaded, PlannerTeam::Senior);
    assert!(strings
        .iter()
        .any(|string| string.id == first_id && string.assignments.is_empty()));
    assert!(team_strings(&reloaded, PlannerTeam::Reserves)[0]
        .assignments
        .iter()
        .any(|assignment| assignment.player_uid == 77));

    clear_assignment(&conn, save_id, reserve_id, "goalkeeper").expect("clear assignment");
    assert!(team_strings(
        &get_depth(&conn, save_id).expect("reload after clear"),
        PlannerTeam::Senior,
    )
    .iter()
    .all(|string| string.assignments.is_empty()));
}

#[test]
fn preserves_assignment_as_unresolved_when_snapshot_replaces_player() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    assign_player(&conn, save_id, string_id, "goalkeeper", 77).expect("assign player");

    let replacement_path = temp_dir.path().join("replacement.json");
    let replacement = include_str!("../memory_read/fixtures/golden_dump_v9.json")
        .replace("\"uid\": 77", "\"uid\": 78")
        .replace("\"name\": \"Loan Player\"", "\"name\": \"Replacement\"");
    std::fs::write(&replacement_path, replacement).expect("write replacement dump");
    snapshot::ingest::ingest_dump_file(&mut conn, &replacement_path).expect("replace snapshot");

    let reloaded = get_depth(&conn, save_id).expect("reload depth");
    let assignment = &team_strings(&reloaded, PlannerTeam::Senior)[0].assignments[0];
    assert_eq!(assignment.last_known_name, "Golden Fixture Player");
    assert_eq!(assignment.current_name, None);
    assert_eq!(assignment.state, AssignmentState::Unresolved);
    assert_eq!(assignment.combined_score, None);
    assert_eq!(assignment.potential_combined_score, None);
}

#[test]
fn assignment_state_uses_managed_club_membership_not_team_level() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    tactic.lanes[0].ip_weight = 0.25;
    tactic::save_tactic(&conn, save_id, &tactic).expect("save lane weight");
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    assign_player(&conn, save_id, string_id, "goalkeeper", 77).expect("assign player");
    let snapshot_id: i64 = conn
        .query_row(
            "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
            params![save_id],
            |row| row.get(0),
        )
        .expect("current snapshot");
    conn.execute(
        "UPDATE players
         SET ca = 80,
             pa = 170,
             age = 20,
             positions_json = '{\"GK\": 18, \"SW\": null, \"MC\": 14, \"DM\": 0}',
             attributes_json = '{\"Acceleration\": 10}'
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("set projection inputs");

    let tx = conn
        .unchecked_transaction()
        .expect("start potential rebuild");
    potential_scores::replace_player(&tx, snapshot_id, 77).expect("rebuild persisted potential");
    tx.commit().expect("commit potential rebuild");
    conn.execute(
        "UPDATE player_role_metrics
         SET goalkeeper_ip = 80,
             line_holding_keeper_oop = 60
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("set role scores");
    let potential_ip_column =
        crate::features::player_metrics::compact::player_potential_column("goalkeeper_ip")
            .expect("potential ip column");
    let potential_oop_column = crate::features::player_metrics::compact::player_potential_column(
        "line_holding_keeper_oop",
    )
    .expect("potential oop column");
    let (potential_ip_score, potential_oop_score) = conn
        .query_row(
            &format!(
                "SELECT {potential_ip_column}, {potential_oop_column}
                 FROM player_role_metrics
                 WHERE snapshot_id = ?1 AND uid = 77"
            ),
            params![snapshot_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("load persisted potential scores");
    let expected_potential = combine_role_scores(potential_ip_score, potential_oop_score, 0.25);

    let scored = get_depth(&conn, save_id).expect("load score");
    let assignment = &team_strings(&scored, PlannerTeam::Senior)[0].assignments[0];
    assert_eq!(assignment.state, AssignmentState::Resolved);
    assert_eq!(assignment.combined_score, Some(65));
    assert_eq!(assignment.potential_combined_score, expected_potential);

    conn.execute(
        "UPDATE players SET team_level = NULL WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("remove assigned player team level");

    let reloaded = get_depth(&conn, save_id).expect("reload depth");
    let assignment = &team_strings(&reloaded, PlannerTeam::Senior)[0].assignments[0];
    assert_eq!(assignment.state, AssignmentState::Resolved);
    assert_eq!(assignment.combined_score, Some(65));
    assert_eq!(assignment.potential_combined_score, expected_potential);

    conn.execute(
        "UPDATE players SET current_club = 'Other FC' WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("move assigned player outside managed club");

    let reloaded = get_depth(&conn, save_id).expect("reload outside-pool assignment");
    let assignment = &team_strings(&reloaded, PlannerTeam::Senior)[0].assignments[0];
    assert_eq!(assignment.state, AssignmentState::OutsidePool);
    assert_eq!(assignment.combined_score, Some(65));
    assert_eq!(assignment.potential_combined_score, expected_potential);
}

#[test]
fn keeps_potential_combined_score_unavailable_when_selected_attributes_are_missing() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    assign_player(&conn, save_id, string_id, "goalkeeper", 77).expect("assign player");
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "UPDATE player_role_metrics
         SET goalkeeper_ip = 80,
             line_holding_keeper_oop = 60
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("set role scores");

    let resolved = get_depth(&conn, save_id).expect("load depth");
    let assignment = &team_strings(&resolved, PlannerTeam::Senior)[0].assignments[0];
    assert_eq!(assignment.state, AssignmentState::Resolved);
    assert_eq!(assignment.combined_score, Some(70));
    assert_eq!(assignment.potential_combined_score, None);
}

#[test]
fn direct_depth_load_does_not_audit_unassigned_player_potential_state() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    get_depth(&conn, save_id).expect("create planner depth");
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "DELETE FROM player_role_metrics
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("remove compact row");
    let before = planner_potential_state(&conn, save_id, snapshot_id);
    deny_potential_writes(&conn);

    get_depth(&conn, save_id).expect("load empty depth without a player-wide audit");
    assert_eq!(planner_potential_state(&conn, save_id, snapshot_id), before);
}

#[test]
fn depth_rejects_missing_potential_role_without_writes() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    assign_player(&conn, save_id, string_id, "goalkeeper", 77).expect("assign player");
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "DELETE FROM player_role_metrics
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("remove compact row");
    let before = planner_potential_state(&conn, save_id, snapshot_id);
    deny_potential_writes(&conn);

    let error = get_depth(&conn, save_id).expect_err("reject incomplete potential state");
    assert_eq!(error, "Current potential snapshot is incomplete");
    assert_eq!(planner_potential_state(&conn, save_id, snapshot_id), before);
}

#[test]
fn depth_rejects_wrong_version_potential_role_without_writes() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    assign_player(&conn, save_id, string_id, "goalkeeper", 77).expect("assign player");
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "UPDATE player_role_metrics
         SET projection_model_version = 999
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("stale compact projection version");
    let before = planner_potential_state(&conn, save_id, snapshot_id);
    deny_potential_writes(&conn);

    let error = get_depth(&conn, save_id).expect_err("reject stale potential state");
    assert_eq!(error, "Current potential snapshot is incomplete");
    assert_eq!(planner_potential_state(&conn, save_id, snapshot_id), before);
}

#[test]
fn corrupt_potential_state_blocks_assignment_move_before_writes() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let first_string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    let second_string_id = add_second_senior_string(&conn, save_id);
    assign_player(&conn, save_id, first_string_id, "goalkeeper", 77).expect("assign player");
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "UPDATE player_role_metrics
         SET projection_model_version = 999
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("corrupt compact projection version");
    let before = planner_potential_state(&conn, save_id, snapshot_id);
    deny_potential_writes(&conn);

    let error = move_player(&conn, save_id, second_string_id, "goalkeeper", 77)
        .expect_err("reject assignment move");
    assert_eq!(error, "Current potential snapshot is incomplete");
    assert_eq!(planner_potential_state(&conn, save_id, snapshot_id), before);
}

#[test]
fn slot_candidates_reject_missing_current_compact_row_without_writes() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    get_depth(&conn, save_id).expect("create planner depth");
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "DELETE FROM player_role_metrics
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("remove compact row for managed-club candidate");
    let before = planner_potential_state(&conn, save_id, snapshot_id);
    deny_potential_writes(&conn);

    let error = get_slot_candidates(&conn, save_id, PlannerTeam::Senior, "goalkeeper", "")
        .expect_err("reject missing compact row");
    assert_eq!(error, "Current potential snapshot is incomplete");
    assert_eq!(planner_potential_state(&conn, save_id, snapshot_id), before);
}

#[test]
fn slot_candidates_reject_wrong_current_score_model_version_without_writes() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    get_depth(&conn, save_id).expect("create planner depth");
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "UPDATE player_role_metrics
         SET score_model_version = 999
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("stale compact current version for managed-club candidate");
    let before = planner_potential_state(&conn, save_id, snapshot_id);
    deny_potential_writes(&conn);

    let error = get_slot_candidates(&conn, save_id, PlannerTeam::Senior, "goalkeeper", "")
        .expect_err("reject stale compact current state");
    assert_eq!(error, "Current potential snapshot is incomplete");
    assert_eq!(planner_potential_state(&conn, save_id, snapshot_id), before);
}

#[test]
fn assignment_uses_persisted_potential_scores_instead_of_source_projection() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    assign_player(&conn, save_id, string_id, "goalkeeper", 77).expect("assign player");
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "UPDATE players
         SET attributes_json = '{}'
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("diverge source attributes");
    conn.execute(
        "UPDATE player_role_metrics
         SET potential_goalkeeper_ip = 80,
             potential_line_holding_keeper_oop = 60
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("set persisted potential scores");

    let reloaded = get_depth(&conn, save_id).expect("load persisted assignment score");
    let assignment = &team_strings(&reloaded, PlannerTeam::Senior)[0].assignments[0];
    assert_eq!(
        assignment.potential_combined_score,
        combine_role_scores(Some(80), Some(60), 0.5)
    );
}

#[test]
fn assignment_expects_the_share_of_the_gap_the_players_age_and_personality_reach() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    assign_player(&conn, save_id, string_id, "goalkeeper", 77).expect("assign player");
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "UPDATE players
         SET age = 20,
             personality_json = '{\"Professionalism\": 15, \"Ambition\": 15}',
             attributes_json = '{\"Determination\": 15}'
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("set development inputs");
    conn.execute(
        "UPDATE player_role_metrics
         SET goalkeeper_ip = 60,
             line_holding_keeper_oop = 60,
             potential_goalkeeper_ip = 80,
             potential_line_holding_keeper_oop = 80
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("set current and potential scores");

    let reloaded = get_depth(&conn, save_id).expect("load assignment scores");
    let assignment = &team_strings(&reloaded, PlannerTeam::Senior)[0].assignments[0];

    // At 20 with a drive of 15, the player closes 85% of the 20-point gap.
    assert_eq!(assignment.combined_score, Some(60));
    assert_eq!(assignment.expected_combined_score, Some(77));
    assert_eq!(assignment.potential_combined_score, Some(80));
}

#[test]
fn managed_club_and_tactic_updates_preserve_assignments_and_save_isolation() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    assign_player(&conn, save_id, string_id, "goalkeeper", 77).expect("assign player");

    conn.execute(
        "UPDATE players SET current_club = 'Other FC' WHERE current_club = 'Loan FC'",
        [],
    )
    .expect("move player to another available club");
    managed_club_service::set_managed_club(&conn, save_id, "Other FC", Some(1000))
        .expect("replace managed club");
    let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    tactic.lanes[0].ip_role_id = "ball_playing_goalkeeper_ip".to_string();
    tactic::save_tactic(&conn, save_id, &tactic).expect("change tactic role");
    assert_eq!(
        team_strings(
            &get_depth(&conn, save_id).expect("reload depth"),
            PlannerTeam::Senior
        )[0]
        .assignments
        .len(),
        1
    );

    conn.execute(
        "INSERT INTO saves (name, is_active) VALUES ('Second save', 0)",
        [],
    )
    .expect("create second save");
    let second_save_id = conn.last_insert_rowid();
    let second_dump_path = temp_dir.path().join("second-save.json");
    std::fs::write(
        &second_dump_path,
        include_str!("../memory_read/fixtures/golden_dump_v9.json"),
    )
    .expect("write second save dump");
    snapshot::ingest::ingest_dump_file_for_save(&mut conn, second_save_id, &second_dump_path)
        .expect("ingest second save");
    managed_club_service::set_managed_club(&conn, second_save_id, "Loan FC", Some(1000))
        .expect("configure second save");
    let second_depth = get_depth(&conn, second_save_id).expect("create isolated depth");
    let second_string_id = team_strings(&second_depth, PlannerTeam::Senior)[0].id;
    assign_player(&conn, second_save_id, second_string_id, "goalkeeper", 77)
        .expect("assign same player uid in second save");
    assert!(second_depth
        .teams
        .iter()
        .all(|team| team.strings[0].assignments.is_empty()));
}
