use rusqlite::params;

use crate::features::managed_club::service as managed_club_service;
use crate::features::planner::tactic;
use crate::features::scoring::{
    catalog::{all_roles, DUMP_ATTRIBUTE_KEYS},
    combine::combine_role_scores,
    projection::project_attributes,
    score::score_role,
};
use crate::features::snapshot;

use super::depth::{
    add_string, assign_player, clear_all, clear_assignment, get_depth, get_slot_candidates,
    move_player, remove_string, AssignmentState, PlannerTeam,
};
use super::teams::{save_team_settings, PlannerTeamInput};
use super::test_support::{
    add_picker_candidates, assignment_provenance, current_snapshot_id, open_with_snapshot,
    team_strings,
};

#[test]
fn returns_ranked_candidates_from_the_managed_club_team_level() {
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
         SET score = CASE uid
             WHEN 78 THEN 50
             WHEN 79 THEN 80
             WHEN 80 THEN NULL
             ELSE score
         END
         WHERE snapshot_id = ?1
           AND role_id IN ('goalkeeper_ip', 'line_holding_keeper_oop')",
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
        [79]
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
        [79]
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
        "UPDATE player_role_scores
         SET score = CASE role_id
             WHEN 'goalkeeper_ip' THEN 80
             WHEN 'line_holding_keeper_oop' THEN 60
             ELSE score
         END
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("set keeper scores");

    let candidates = get_slot_candidates(&conn, save_id, PlannerTeam::Senior, "goalkeeper", "")
        .expect("load candidates");

    assert_eq!(candidates[0].combined_score, Some(80));
}

#[test]
fn team_level_maps_each_managed_club_player_to_one_team_pool() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();

    assert_eq!(
        get_slot_candidates(&conn, save_id, PlannerTeam::Senior, "goalkeeper", "")
            .expect("load senior candidates")
            .iter()
            .map(|candidate| candidate.player_uid)
            .collect::<Vec<_>>(),
        [77]
    );
    assert!(
        get_slot_candidates(&conn, save_id, PlannerTeam::Reserves, "goalkeeper", "")
            .expect("load reserve candidates")
            .is_empty()
    );
    assert!(
        get_slot_candidates(&conn, save_id, PlannerTeam::Youth, "goalkeeper", "")
            .expect("load youth candidates")
            .is_empty()
    );

    conn.execute(
        "UPDATE players SET team_level = NULL WHERE current_club = 'Loan FC'",
        [],
    )
    .expect("remove team level");
    assert!(
        get_slot_candidates(&conn, save_id, PlannerTeam::Senior, "goalkeeper", "")
            .expect("exclude unclassified player")
            .is_empty()
    );
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
}

#[test]
fn direct_depth_commands_reject_an_unavailable_team_without_recreating_it() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    get_depth(&conn, save_id).expect("initialize planner depth");
    save_team_settings(
        &conn,
        save_id,
        &[PlannerTeamInput {
            team: "senior".to_string(),
            display_name: "Senior".to_string(),
        }],
        false,
    )
    .expect("remove unused teams");

    let add_error = add_string(&conn, save_id, PlannerTeam::Reserves)
        .expect_err("reject adding a string to an unavailable team");
    assert!(add_error.contains("not available"));
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
fn assign_and_move_player_persist_manual_provenance() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let first_string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    let second_string_id = add_string(&conn, save_id, PlannerTeam::Senior)
        .expect("add destination string")
        .id;

    assign_player(&conn, save_id, first_string_id, "goalkeeper", 77).expect("assign player");
    assert_eq!(assignment_provenance(&conn, 77), "manual");

    move_player(&conn, save_id, second_string_id, "goalkeeper", 77).expect("move player");
    assert_eq!(assignment_provenance(&conn, 77), "manual");
}

#[test]
fn adds_ordered_strings_and_rejects_removing_the_final_string() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let first = get_depth(&conn, save_id).expect("create planner depth");
    let first_senior_id = team_strings(&first, PlannerTeam::Senior)[0].id;

    let added = add_string(&conn, save_id, PlannerTeam::Senior).expect("add string");
    assert_eq!(added.string_order, 1);
    remove_string(&conn, save_id, first_senior_id, false).expect("remove empty string");

    let error = remove_string(&conn, save_id, added.id, false).expect_err("keep the final string");
    assert!(error.contains("at least one string"));
    let reloaded = get_depth(&conn, save_id).expect("reload depth");
    let strings = team_strings(&reloaded, PlannerTeam::Senior);
    assert_eq!(
        strings
            .iter()
            .map(|string| string.string_order)
            .collect::<Vec<_>>(),
        [0]
    );

    let next = add_string(&conn, save_id, PlannerTeam::Senior).expect("add next string");
    assert_eq!(next.string_order, 1);
}

#[test]
fn populated_string_requires_confirmation_and_deletes_only_its_assignments() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let populated_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    let remaining = add_string(&conn, save_id, PlannerTeam::Senior).expect("add string");
    assign_player(&conn, save_id, populated_id, "goalkeeper", 77).expect("assign player");

    let error = remove_string(&conn, save_id, populated_id, false)
        .expect_err("require populated confirmation");
    assert!(error.contains("requires confirmation"));
    assert_eq!(
        team_strings(
            &get_depth(&conn, save_id).expect("reload depth"),
            PlannerTeam::Senior
        )[0]
        .assignments
        .len(),
        1
    );

    remove_string(&conn, save_id, populated_id, true).expect("remove confirmed string");
    let reloaded = get_depth(&conn, save_id).expect("reload depth");
    let strings = team_strings(&reloaded, PlannerTeam::Senior);
    assert_eq!(strings.len(), 1);
    assert_eq!(strings[0].id, remaining.id);
    assert!(strings[0].assignments.is_empty());
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
        include_str!("../memory_read/fixtures/golden_dump_v8.json"),
    )
    .expect("write second save dump");
    snapshot::ingest::ingest_dump_file_for_save(&mut conn, second_save_id, &second_dump_path)
        .expect("ingest second save");
    managed_club_service::set_managed_club(&conn, second_save_id, "Loan FC")
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
    let second = add_string(&conn, save_id, PlannerTeam::Senior).expect("add string");
    assign_player(&conn, save_id, first_id, "goalkeeper", 77).expect("assign player");

    let error = assign_player(&conn, save_id, second.id, "goalkeeper", 77)
        .expect_err("reject duplicate player");
    assert!(error.contains("already assigned"));
    let error = assign_player(&conn, save_id, reserve_id, "goalkeeper", 77)
        .expect_err("reject player outside reserve team level");
    assert!(error.contains("not available"));

    move_player(&conn, save_id, second.id, "goalkeeper", 77).expect("move player");
    let reloaded = get_depth(&conn, save_id).expect("reload depth");
    let strings = team_strings(&reloaded, PlannerTeam::Senior);
    assert!(strings
        .iter()
        .any(|string| string.id == first_id && string.assignments.is_empty()));
    assert!(strings.iter().any(|string| {
        string.id == second.id
            && string
                .assignments
                .iter()
                .any(|assignment| assignment.player_uid == 77)
    }));

    clear_assignment(&conn, save_id, second.id, "goalkeeper").expect("clear assignment");
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
    let replacement = include_str!("../memory_read/fixtures/golden_dump_v8.json")
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
fn combines_selected_lane_potential_scores_for_resolved_and_outside_pool_players() {
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
    let attributes = DUMP_ATTRIBUTE_KEYS
        .iter()
        .map(|key| ((*key).to_string(), Some(10)))
        .collect();
    conn.execute(
        "UPDATE players
         SET ca = 80,
             pa = 170,
             age = 20,
             positions_json = '{\"GK\": 18, \"SW\": null, \"MC\": 14, \"DM\": 0}',
             attributes_json = ?1
         WHERE snapshot_id = ?2 AND uid = 77",
        params![
            serde_json::to_string(&attributes).expect("serialize attributes"),
            snapshot_id
        ],
    )
    .expect("set projection inputs");
    conn.execute(
        "UPDATE player_role_scores
         SET score = CASE role_id
             WHEN 'goalkeeper_ip' THEN 80
             WHEN 'line_holding_keeper_oop' THEN 60
             ELSE score
         END
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("set role scores");

    let projected_attributes =
        project_attributes(&attributes, 80, 170, Some(20), [("GK", Some(20))]);
    let ip_role = all_roles()
        .iter()
        .find(|role| role.role_id == "goalkeeper_ip")
        .expect("goalkeeper role");
    let oop_role = all_roles()
        .iter()
        .find(|role| role.role_id == "line_holding_keeper_oop")
        .expect("line-holding keeper role");
    let expected_potential = combine_role_scores(
        score_role(&projected_attributes, ip_role),
        score_role(&projected_attributes, oop_role),
        0.25,
    );

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
        "UPDATE player_role_scores
         SET score = CASE role_id
             WHEN 'goalkeeper_ip' THEN 80
             WHEN 'line_holding_keeper_oop' THEN 60
             ELSE score
         END
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
    managed_club_service::set_managed_club(&conn, save_id, "Other FC")
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
        include_str!("../memory_read/fixtures/golden_dump_v8.json"),
    )
    .expect("write second save dump");
    snapshot::ingest::ingest_dump_file_for_save(&mut conn, second_save_id, &second_dump_path)
        .expect("ingest second save");
    managed_club_service::set_managed_club(&conn, second_save_id, "Loan FC")
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
