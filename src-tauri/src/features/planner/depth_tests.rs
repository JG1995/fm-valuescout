use rusqlite::params;

use crate::features::planner::service::{self, ClubSourceInput};
use crate::features::planner::tactic;
use crate::features::snapshot;

use super::depth::{
    add_string, assign_player, clear_assignment, clear_team, get_depth, get_slot_candidates,
    move_player, remove_string, AssignmentState, PlannerTeam,
};
use super::test_support::{
    add_picker_candidates, assignment_provenance, open_with_snapshot, team_strings,
};

#[test]
fn returns_ranked_candidates_from_the_target_team_club_family() {
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
    assign_player(&conn, save_id, reserve_string_id, "goalkeeper", 78)
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
    assert_eq!(candidates[1].combined_score, Some(50));
    assert_eq!(candidates[2].combined_score, None);
    assert_eq!(candidates[3].combined_score, None);
    assert_eq!(candidates[0].current_club, "Loan B FC");
    assert_eq!(
        candidates[1].assignment_location.as_ref().map(|location| (
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
        "b team",
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
fn team_level_does_not_restrict_the_primary_club_pool() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();

    for team_level in [Some("senior"), None] {
        conn.execute(
            "UPDATE players SET team_level = ?1 WHERE current_club = 'Loan FC'",
            params![team_level],
        )
        .expect("set team level");

        for team in [
            PlannerTeam::Senior,
            PlannerTeam::Reserves,
            PlannerTeam::Youth,
        ] {
            let candidates = get_slot_candidates(&conn, save_id, team, "goalkeeper", "")
                .expect("load candidates");

            assert_eq!(
                candidates
                    .iter()
                    .map(|candidate| candidate.player_uid)
                    .collect::<Vec<_>>(),
                [77]
            );
        }
    }

    let depth = get_depth(&conn, save_id).expect("load depth");
    let youth_string_id = team_strings(&depth, PlannerTeam::Youth)[0].id;
    assign_player(&conn, save_id, youth_string_id, "goalkeeper", 77)
        .expect("assign senior-level player to youth planner team");

    let depth = get_depth(&conn, save_id).expect("reload depth");
    assert_eq!(
        team_strings(&depth, PlannerTeam::Youth)[0].assignments[0].state,
        AssignmentState::Resolved
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
fn clearing_a_team_requires_confirmation_and_preserves_other_teams() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let senior_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    let reserve_id = team_strings(&depth, PlannerTeam::Reserves)[0].id;
    assign_player(&conn, save_id, senior_id, "goalkeeper", 77).expect("assign senior keeper");
    assign_player(&conn, save_id, reserve_id, "goalkeeper", 78).expect("assign reserve keeper");
    conn.execute(
        "INSERT INTO planner_assignments (
            save_id, string_id, lane_id, player_uid, last_known_name, provenance
         ) VALUES (?1, ?2, ?3, ?4, ?5, 'optimizer')",
        params![save_id, senior_id, "left_back", 79, "Senior optimizer"],
    )
    .expect("assign senior optimizer row");
    conn.execute(
        "INSERT INTO planner_assignments (
            save_id, string_id, lane_id, player_uid, last_known_name, provenance
         ) VALUES (?1, ?2, ?3, ?4, ?5, 'optimizer')",
        params![save_id, reserve_id, "left_back", 80, "Reserve optimizer"],
    )
    .expect("assign reserve optimizer row");

    let error =
        clear_team(&conn, save_id, PlannerTeam::Senior, false).expect_err("require confirmation");
    assert!(error.contains("requires confirmation"));
    assert_eq!(
        team_strings(
            &get_depth(&conn, save_id).expect("reload after rejected clear"),
            PlannerTeam::Senior,
        )[0]
        .assignments
        .len(),
        2
    );

    clear_team(&conn, save_id, PlannerTeam::Senior, true).expect("clear senior team");
    let reloaded = get_depth(&conn, save_id).expect("reload after clear");
    assert!(team_strings(&reloaded, PlannerTeam::Senior)[0]
        .assignments
        .is_empty());
    assert_eq!(
        team_strings(&reloaded, PlannerTeam::Reserves)[0]
            .assignments
            .iter()
            .map(|assignment| assignment.player_uid)
            .collect::<Vec<_>>(),
        [78, 80]
    );
}

#[test]
fn enforces_player_uniqueness_and_moves_in_one_save() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    service::save_club_family(
        &conn,
        save_id,
        "Loan FC",
        &[ClubSourceInput {
            team: "reserves".to_string(),
            club_name: "Loan FC".to_string(),
            team_level: None,
        }],
    )
    .expect("add reserve source");
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let first_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    let reserve_id = team_strings(&depth, PlannerTeam::Reserves)[0].id;
    let second = add_string(&conn, save_id, PlannerTeam::Senior).expect("add string");
    assign_player(&conn, save_id, first_id, "goalkeeper", 77).expect("assign player");

    let error = assign_player(&conn, save_id, second.id, "goalkeeper", 77)
        .expect_err("reject duplicate player");
    assert!(error.contains("already assigned"));
    let error = assign_player(&conn, save_id, reserve_id, "goalkeeper", 77)
        .expect_err("reject duplicate player across teams");
    assert!(error.contains("already assigned"));

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
    let replacement = include_str!("../memory_read/fixtures/golden_dump_v5.json")
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
}

#[test]
fn resolves_combined_scores_and_marks_current_players_outside_the_pool() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
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
    let scored = get_depth(&conn, save_id).expect("load score");
    let assignment = &team_strings(&scored, PlannerTeam::Senior)[0].assignments[0];
    assert_eq!(assignment.state, AssignmentState::Resolved);
    assert_eq!(assignment.combined_score, Some(70));

    let moved_path = temp_dir.path().join("moved.json");
    let moved = include_str!("../memory_read/fixtures/golden_dump_v5.json").replace(
        "\"currentClub\": \"Loan FC\"",
        "\"currentClub\": \"Other FC\"",
    );
    std::fs::write(&moved_path, moved).expect("write moved dump");
    snapshot::ingest::ingest_dump_file(&mut conn, &moved_path).expect("replace snapshot");

    let reloaded = get_depth(&conn, save_id).expect("reload depth");
    let assignment = &team_strings(&reloaded, PlannerTeam::Senior)[0].assignments[0];
    assert_eq!(assignment.state, AssignmentState::OutsidePool);
    assert_eq!(assignment.combined_score, None);
}

#[test]
fn source_and_tactic_updates_preserve_existing_assignments_and_saves_are_isolated() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    assign_player(&conn, save_id, string_id, "goalkeeper", 77).expect("assign player");

    service::save_club_family(
        &conn,
        save_id,
        "Loan FC",
        &[ClubSourceInput {
            team: "reserves".to_string(),
            club_name: "Loan FC".to_string(),
            team_level: None,
        }],
    )
    .expect("replace sources");
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
        include_str!("../memory_read/fixtures/golden_dump_v5.json"),
    )
    .expect("write second save dump");
    snapshot::ingest::ingest_dump_file_for_save(&mut conn, second_save_id, &second_dump_path)
        .expect("ingest second save");
    service::save_club_family(&conn, second_save_id, "Loan FC", &[])
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
