use rusqlite::params;

use crate::features::snapshot::service;
use crate::features::staff::assignment_targets::{self, StaffAssignmentTargetInput};

use super::depth::{assign_player, get_depth, PlannerTeam};
use super::teams::{get_team_settings, save_team_settings, PlannerTeamInput};
use super::test_support::{
    add_picker_candidates, current_snapshot_id, deny_potential_writes, open_with_snapshot,
    planner_potential_state, team_strings,
};

fn input(team: &str, display_name: &str) -> PlannerTeamInput {
    PlannerTeamInput {
        team: team.to_string(),
        display_name: display_name.to_string(),
    }
}

#[test]
fn replaces_team_names_and_removes_populated_team_after_confirmation() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let depth = get_depth(&conn, save_id).expect("initialize planner depth");
    let reserves_string_id = team_strings(&depth, PlannerTeam::Reserves)[0].id;
    assign_player(&conn, save_id, reserves_string_id, "goalkeeper", 79)
        .expect("assign reserve player");

    let error = save_team_settings(
        &conn,
        save_id,
        &[input("senior", " First Team "), input("youth", "U19")],
        false,
    )
    .expect_err("require confirmation before populated removal");
    assert!(error.contains("confirmation"));
    assert_eq!(
        get_team_settings(&conn, save_id)
            .expect("load settings")
            .len(),
        3
    );
    assert_eq!(
        team_strings(
            &get_depth(&conn, save_id).expect("reload depth"),
            PlannerTeam::Reserves,
        )[0]
        .assignments
        .len(),
        1
    );

    let settings = save_team_settings(
        &conn,
        save_id,
        &[input("senior", " First Team "), input("youth", "U19")],
        true,
    )
    .expect("save confirmed settings");
    assert_eq!(
        settings
            .0
            .iter()
            .map(|setting| (setting.team, setting.display_name.as_str()))
            .collect::<Vec<_>>(),
        [
            (PlannerTeam::Senior, "First Team"),
            (PlannerTeam::Youth, "U19"),
        ]
    );
    let depth = get_depth(&conn, save_id).expect("reload depth after removal");
    assert!(!depth
        .teams
        .iter()
        .any(|team| team.team == PlannerTeam::Reserves));
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM planner_assignments WHERE save_id = ?1",
            [save_id],
            |row| row.get::<_, i64>(0),
        )
        .expect("count assignments"),
        0
    );

    let restored = save_team_settings(
        &conn,
        save_id,
        &[
            input("senior", "First Team"),
            input("reserves", "Reserves"),
            input("youth", "U19"),
        ],
        false,
    )
    .expect("restore reserves");
    assert_eq!(restored.0.len(), 3);
    assert_eq!(
        team_strings(
            &get_depth(&conn, save_id).expect("reload depth after restore"),
            PlannerTeam::Reserves,
        )
        .len(),
        1
    );
}

#[test]
fn target_impact_requires_confirmation_and_confirmed_removal_keeps_other_scopes() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    get_depth(&conn, save_id).expect("initialize planner teams");
    let token = service::capture_active_save_context(&conn)
        .expect("save context")
        .context_token;
    let targets = assignment_targets::get_targets(&conn, &token).expect("expanded targets");
    let inputs = targets
        .targets
        .iter()
        .map(|target| StaffAssignmentTargetInput {
            scope: target.scope.clone(),
            job_id: target.job_id.clone(),
            slot_count: i64::from(
                (target.scope == "reserves" && target.job_id == "manager")
                    || target.scope == "club",
            ),
        })
        .collect::<Vec<_>>();
    assignment_targets::save_targets(&conn, &token, &inputs).expect("save targets");

    let inputs = [input("senior", "Senior"), input("youth", "Youth")];
    let impacts =
        super::teams::planner_team_removal_impacts(&conn, save_id, &inputs).expect("target impact");
    assert_eq!(impacts.len(), 1);
    assert_eq!(impacts[0].assignment_count, 0);
    assert_eq!(
        impacts[0]
            .staffing_targets
            .iter()
            .map(|target| (target.job_id.as_str(), target.slot_count))
            .collect::<Vec<_>>(),
        [("manager", 1)]
    );

    assert!(save_team_settings(&conn, save_id, &inputs, false).is_err());
    save_team_settings(&conn, save_id, &inputs, true).expect("confirmed removal");
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM staff_assignment_targets WHERE save_id = ?1 AND scope = 'reserves'",
            [save_id],
            |row| row.get::<_, i64>(0),
        )
        .expect("removed targets"),
        0
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM staff_assignment_targets WHERE save_id = ?1 AND scope = 'club'",
            [save_id],
            |row| row.get::<_, i64>(0),
        )
        .expect("retained club targets"),
        11
    );
}

#[test]
fn corrupt_potential_state_blocks_confirmed_team_removal_before_writes() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let depth = get_depth(&conn, save_id).expect("initialize planner depth");
    let reserves_string_id = team_strings(&depth, PlannerTeam::Reserves)[0].id;
    assign_player(&conn, save_id, reserves_string_id, "goalkeeper", 79)
        .expect("assign reserve player");
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "UPDATE players
         SET potential_projection_model_version = 999
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("corrupt projected map version");
    let before = planner_potential_state(&conn, save_id, snapshot_id);
    deny_potential_writes(&conn);

    let error = save_team_settings(
        &conn,
        save_id,
        &[input("senior", "Senior"), input("youth", "Youth")],
        true,
    )
    .expect_err("reject destructive team removal");
    assert_eq!(error, "Current potential snapshot is incomplete");
    assert_eq!(planner_potential_state(&conn, save_id, snapshot_id), before);
}

#[test]
fn rejects_invalid_team_settings_without_mutating_existing_rows() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    get_depth(&conn, save_id).expect("initialize planner depth");
    conn.execute(
        "UPDATE player_potential_role_scores
         SET projection_model_version = 999
         WHERE snapshot_id = ?1 AND uid = 77 AND role_id = 'goalkeeper_ip'",
        params![current_snapshot_id(&conn, save_id)],
    )
    .expect("corrupt potential state");

    for (inputs, expected) in [
        (vec![], "at least one"),
        (
            vec![
                input("senior", "Senior"),
                input("reserves", "Reserves"),
                input("youth", "Youth"),
                input("senior", "Fourth"),
            ],
            "at most 3",
        ),
        (vec![input("unknown", "Unknown")], "Unknown planner team"),
        (
            vec![input("senior", " "), input("reserves", "Reserves")],
            "must not be empty",
        ),
        (vec![input("senior", "A"), input("reserves", "a")], "unique"),
        (vec![input("senior", &"x".repeat(41))], "at most 40"),
    ] {
        let error = save_team_settings(&conn, save_id, &inputs, false)
            .expect_err("reject invalid team settings");
        assert!(error.contains(expected), "{error}");
    }

    assert_eq!(
        get_team_settings(&conn, save_id)
            .expect("load settings")
            .len(),
        3
    );
}

#[test]
fn swaps_existing_display_names_without_transient_unique_constraint_failure() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    get_depth(&conn, save_id).expect("initialize planner depth");

    let settings = save_team_settings(
        &conn,
        save_id,
        &[
            input("senior", "Reserves"),
            input("reserves", "Senior"),
            input("youth", "Youth"),
        ],
        false,
    )
    .expect("swap existing display names");

    assert_eq!(
        settings
            .0
            .iter()
            .map(|setting| (setting.team, setting.display_name.as_str()))
            .collect::<Vec<_>>(),
        [
            (PlannerTeam::Senior, "Reserves"),
            (PlannerTeam::Reserves, "Senior"),
            (PlannerTeam::Youth, "Youth"),
        ]
    );

    let sentinel_names = save_team_settings(
        &conn,
        save_id,
        &[
            input("senior", "__planner_team_reserves"),
            input("reserves", "__planner_team_youth"),
            input("youth", "__planner_team_senior"),
        ],
        false,
    )
    .expect("allow display names in the internal replacement namespace");
    assert_eq!(
        sentinel_names
            .0
            .iter()
            .map(|setting| (setting.team, setting.display_name.as_str()))
            .collect::<Vec<_>>(),
        [
            (PlannerTeam::Senior, "__planner_team_reserves"),
            (PlannerTeam::Reserves, "__planner_team_youth"),
            (PlannerTeam::Youth, "__planner_team_senior"),
        ]
    );
}

#[test]
fn keeps_one_team_and_isolates_settings_per_save() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    get_depth(&conn, save_id).expect("initialize first depth");

    let error =
        save_team_settings(&conn, save_id, &[], false).expect_err("do not remove every team");
    assert!(error.contains("at least one"));

    let only_senior = save_team_settings(&conn, save_id, &[input("senior", "Senior")], false)
        .expect("keep one configured team");
    assert_eq!(only_senior.0.len(), 1);

    conn.execute(
        "INSERT INTO saves (name, is_active) VALUES ('Second save', 0)",
        [],
    )
    .expect("create second save");
    let second_save_id = conn.last_insert_rowid();
    let second = save_team_settings(&conn, second_save_id, &[input("youth", "U19")], false)
        .expect("configure second save");
    assert_eq!(second.0.len(), 1);
    assert_eq!(
        get_team_settings(&conn, save_id)
            .expect("load first settings")
            .len(),
        1
    );
}
