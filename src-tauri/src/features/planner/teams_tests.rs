use super::depth::{assign_player, get_depth, PlannerTeam};
use super::teams::{get_team_settings, save_team_settings, PlannerTeamInput};
use super::test_support::{add_picker_candidates, open_with_snapshot, team_strings};

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
    assert_eq!(restored.len(), 3);
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
fn rejects_invalid_team_settings_without_mutating_existing_rows() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    get_depth(&conn, save_id).expect("initialize planner depth");

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
    assert_eq!(only_senior.len(), 1);

    conn.execute(
        "INSERT INTO saves (name, is_active) VALUES ('Second save', 0)",
        [],
    )
    .expect("create second save");
    let second_save_id = conn.last_insert_rowid();
    let second = save_team_settings(&conn, second_save_id, &[input("youth", "U19")], false)
        .expect("configure second save");
    assert_eq!(second.len(), 1);
    assert_eq!(
        get_team_settings(&conn, save_id)
            .expect("load first settings")
            .len(),
        1
    );
}
