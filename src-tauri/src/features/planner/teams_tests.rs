use rusqlite::params;

use crate::features::snapshot::service;
use crate::features::staff::assignment_targets::{self, StaffAssignmentTargetInput};

use super::depth::{add_string, assign_player, get_depth, PlannerTeam};
use super::teams::{
    get_team_settings, planner_team_removal_impacts, save_team_settings, PlannerStringInput,
    PlannerTeamInput,
};
use super::test_support::{
    add_picker_candidates, current_snapshot_id, deny_potential_writes, open_with_snapshot,
    planner_potential_state, team_strings,
};

fn input(team: &str, display_name: &str) -> PlannerTeamInput {
    PlannerTeamInput {
        team: team.to_string(),
        display_name: display_name.to_string(),
        strings: vec![PlannerStringInput {
            id: None,
            display_name: String::new(),
        }],
    }
}

fn input_with_strings(
    team: &str,
    display_name: &str,
    strings: Vec<PlannerStringInput>,
) -> PlannerTeamInput {
    PlannerTeamInput {
        team: team.to_string(),
        display_name: display_name.to_string(),
        strings,
    }
}

fn retained_string(id: i64, display_name: &str) -> PlannerStringInput {
    PlannerStringInput {
        id: Some(id),
        display_name: display_name.to_string(),
    }
}

fn new_string(display_name: &str) -> PlannerStringInput {
    PlannerStringInput {
        id: None,
        display_name: display_name.to_string(),
    }
}

fn current_strings_input(
    conn: &rusqlite::Connection,
    save_id: i64,
    team: PlannerTeam,
) -> Vec<PlannerStringInput> {
    let depth = get_depth(conn, save_id).expect("load planner depth");
    team_strings(&depth, team)
        .iter()
        .map(|string| retained_string(string.id, &string.display_name))
        .collect()
}

#[test]
fn replaces_team_names_and_removes_populated_team_after_confirmation() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let depth = get_depth(&conn, save_id).expect("initialize planner depth");
    let reserves_string_id = team_strings(&depth, PlannerTeam::Reserves)[0].id;
    let reserves_name = depth
        .teams
        .iter()
        .find(|team| team.team == PlannerTeam::Reserves)
        .expect("reserves team")
        .display_name
        .clone();
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
    assert!(error.contains(&reserves_name), "{error}");
    assert!(error.contains("1 assignment"), "{error}");
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

    let inputs = [
        input_with_strings(
            "senior",
            "Senior",
            current_strings_input(&conn, save_id, PlannerTeam::Senior),
        ),
        input_with_strings(
            "youth",
            "Youth",
            current_strings_input(&conn, save_id, PlannerTeam::Youth),
        ),
    ];
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
        "UPDATE player_role_metrics
         SET projection_model_version = 999
         WHERE snapshot_id = ?1 AND uid = 77",
        params![snapshot_id],
    )
    .expect("corrupt compact projection version");
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
        "UPDATE player_role_metrics
         SET projection_model_version = 999
         WHERE snapshot_id = ?1",
        params![current_snapshot_id(&conn, save_id)],
    )
    .expect("corrupt compact projection version");

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

#[test]
fn structural_save_keeps_stable_ids_and_assignments_across_rename_and_reorder() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let depth = get_depth(&conn, save_id).expect("initialize planner depth");
    let first_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    let second_id = add_string(&conn, save_id, PlannerTeam::Senior)
        .expect("add second string")
        .0
        .id;
    assign_player(&conn, save_id, first_id, "goalkeeper", 77).expect("assign first player");
    assign_player(&conn, save_id, second_id, "left_back", 78).expect("assign second player");

    save_team_settings(
        &conn,
        save_id,
        &[
            input_with_strings(
                "senior",
                "Senior",
                vec![
                    retained_string(second_id, "Alpha"),
                    retained_string(first_id, "Beta"),
                ],
            ),
            input_with_strings(
                "reserves",
                "Reserves",
                current_strings_input(&conn, save_id, PlannerTeam::Reserves),
            ),
            input_with_strings(
                "youth",
                "Youth",
                current_strings_input(&conn, save_id, PlannerTeam::Youth),
            ),
        ],
        false,
    )
    .expect("rename and reorder senior strings");

    let reloaded = get_depth(&conn, save_id).expect("reload depth");
    let strings = team_strings(&reloaded, PlannerTeam::Senior);
    assert_eq!(strings.len(), 2);
    assert_eq!(strings[0].id, second_id);
    assert_eq!(strings[0].string_order, 0);
    assert_eq!(strings[0].display_name, "Alpha");
    assert_eq!(strings[1].id, first_id);
    assert_eq!(strings[1].string_order, 1);
    assert_eq!(strings[1].display_name, "Beta");
    assert_eq!(
        strings[0]
            .assignments
            .iter()
            .map(|assignment| (assignment.player_uid, assignment.lane_id.as_str()))
            .collect::<Vec<_>>(),
        [(78, "left_back")]
    );
    assert_eq!(
        strings[1]
            .assignments
            .iter()
            .map(|assignment| (assignment.player_uid, assignment.lane_id.as_str()))
            .collect::<Vec<_>>(),
        [(77, "goalkeeper")]
    );
}

#[test]
fn structural_save_swaps_adjacent_orders_without_unique_constraint_failure() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let depth = get_depth(&conn, save_id).expect("initialize planner depth");
    let first_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    let second_id = add_string(&conn, save_id, PlannerTeam::Senior)
        .expect("add second string")
        .0
        .id;
    assign_player(&conn, save_id, first_id, "goalkeeper", 77).expect("assign first player");
    assign_player(&conn, save_id, second_id, "left_back", 78).expect("assign second player");

    save_team_settings(
        &conn,
        save_id,
        &[
            input_with_strings(
                "senior",
                "Senior",
                vec![
                    retained_string(second_id, "2nd string"),
                    retained_string(first_id, "1st string"),
                ],
            ),
            input_with_strings(
                "reserves",
                "Reserves",
                current_strings_input(&conn, save_id, PlannerTeam::Reserves),
            ),
            input_with_strings(
                "youth",
                "Youth",
                current_strings_input(&conn, save_id, PlannerTeam::Youth),
            ),
        ],
        false,
    )
    .expect("swap adjacent string orders");

    let reloaded = get_depth(&conn, save_id).expect("reload depth");
    let strings = team_strings(&reloaded, PlannerTeam::Senior);
    assert_eq!(
        strings
            .iter()
            .map(|string| (string.id, string.string_order))
            .collect::<Vec<_>>(),
        [(second_id, 0), (first_id, 1)]
    );
    assert!(strings
        .iter()
        .flat_map(|string| &string.assignments)
        .any(|assignment| assignment.player_uid == 77));
    assert!(strings
        .iter()
        .flat_map(|string| &string.assignments)
        .any(|assignment| assignment.player_uid == 78));
}

#[test]
fn structural_save_removes_only_the_excluded_strings_assignments() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let depth = get_depth(&conn, save_id).expect("initialize planner depth");
    let first_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    let second_id = add_string(&conn, save_id, PlannerTeam::Senior)
        .expect("add second string")
        .0
        .id;
    assign_player(&conn, save_id, first_id, "goalkeeper", 77).expect("assign first player");
    assign_player(&conn, save_id, second_id, "left_back", 78).expect("assign second player");

    save_team_settings(
        &conn,
        save_id,
        &[
            input_with_strings(
                "senior",
                "Senior",
                vec![retained_string(first_id, "1st string")],
            ),
            input_with_strings(
                "reserves",
                "Reserves",
                current_strings_input(&conn, save_id, PlannerTeam::Reserves),
            ),
            input_with_strings(
                "youth",
                "Youth",
                current_strings_input(&conn, save_id, PlannerTeam::Youth),
            ),
        ],
        true,
    )
    .expect("remove second string with confirmation");

    let reloaded = get_depth(&conn, save_id).expect("reload depth");
    let strings = team_strings(&reloaded, PlannerTeam::Senior);
    assert_eq!(strings.len(), 1);
    assert_eq!(strings[0].id, first_id);
    assert_eq!(strings[0].string_order, 0);
    assert_eq!(
        strings[0]
            .assignments
            .iter()
            .map(|assignment| assignment.player_uid)
            .collect::<Vec<_>>(),
        [77]
    );
    assert_eq!(
        conn.query_row(
            "SELECT COUNT(*) FROM planner_assignments WHERE save_id = ?1",
            [save_id],
            |row| row.get::<_, i64>(0),
        )
        .expect("count assignments"),
        1
    );
}

#[test]
fn structural_save_rejects_populated_string_removal_without_confirmation() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let depth = get_depth(&conn, save_id).expect("initialize planner depth");
    let first_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    let first_name = team_strings(&depth, PlannerTeam::Senior)[0]
        .display_name
        .clone();
    assign_player(&conn, save_id, first_id, "goalkeeper", 77).expect("assign player");
    let snapshot_id = current_snapshot_id(&conn, save_id);
    let before = planner_potential_state(&conn, save_id, snapshot_id);

    let error = save_team_settings(
        &conn,
        save_id,
        &[
            input_with_strings("senior", "Senior", vec![new_string("Fresh")]),
            input_with_strings(
                "reserves",
                "Reserves",
                current_strings_input(&conn, save_id, PlannerTeam::Reserves),
            ),
            input_with_strings(
                "youth",
                "Youth",
                current_strings_input(&conn, save_id, PlannerTeam::Youth),
            ),
        ],
        false,
    )
    .expect_err("require confirmation before populated string removal");
    assert!(error.contains("confirmation"), "{error}");
    assert!(error.contains(&first_name), "{error}");
    assert!(error.contains("1 assignment"), "{error}");
    assert_eq!(planner_potential_state(&conn, save_id, snapshot_id), before);
}

#[test]
fn structural_save_derives_blank_new_names_and_preserves_custom_names() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("initialize planner depth");
    let first_id = team_strings(&depth, PlannerTeam::Senior)[0].id;

    save_team_settings(
        &conn,
        save_id,
        &[
            input_with_strings(
                "senior",
                "Senior",
                vec![
                    retained_string(first_id, "First"),
                    new_string(""),
                    new_string("   "),
                    new_string("  Custom  "),
                ],
            ),
            input_with_strings(
                "reserves",
                "Reserves",
                current_strings_input(&conn, save_id, PlannerTeam::Reserves),
            ),
            input_with_strings(
                "youth",
                "Youth",
                current_strings_input(&conn, save_id, PlannerTeam::Youth),
            ),
        ],
        false,
    )
    .expect("derive blank new-string names");

    let reloaded = get_depth(&conn, save_id).expect("reload depth");
    assert_eq!(
        team_strings(&reloaded, PlannerTeam::Senior)
            .iter()
            .map(|string| (string.string_order, string.display_name.as_str()))
            .collect::<Vec<_>>(),
        [
            (0, "First"),
            (1, "2nd string"),
            (2, "3rd string"),
            (3, "Custom"),
        ]
    );
}

#[test]
fn structural_save_rejects_invalid_string_names_without_mutating_rows() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let depth = get_depth(&conn, save_id).expect("initialize planner depth");
    let first_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    let snapshot_id = current_snapshot_id(&conn, save_id);
    let before = planner_potential_state(&conn, save_id, snapshot_id);
    let senior_name =
        |strings: Vec<PlannerStringInput>| input_with_strings("senior", "Senior", strings);

    for (strings, expected) in [
        (vec![], "at least one string"),
        (vec![retained_string(first_id, " ")], "must not be empty"),
        (
            vec![retained_string(first_id, &"x".repeat(41))],
            "at most 40",
        ),
        (vec![new_string("Alpha"), new_string("alpha")], "unique"),
        (vec![new_string("2nd string"), new_string("")], "unique"),
    ] {
        let error = save_team_settings(
            &conn,
            save_id,
            &[
                senior_name(strings),
                input_with_strings(
                    "reserves",
                    "Reserves",
                    current_strings_input(&conn, save_id, PlannerTeam::Reserves),
                ),
                input_with_strings(
                    "youth",
                    "Youth",
                    current_strings_input(&conn, save_id, PlannerTeam::Youth),
                ),
            ],
            false,
        )
        .expect_err("reject invalid string names");
        assert!(error.contains(expected), "{error}");
    }
    assert_eq!(planner_potential_state(&conn, save_id, snapshot_id), before);
}

#[test]
fn structural_save_rejects_untrusted_string_ids_before_any_mutation() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let depth = get_depth(&conn, save_id).expect("initialize planner depth");
    let senior_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    let reserves_id = team_strings(&depth, PlannerTeam::Reserves)[0].id;
    conn.execute(
        "INSERT INTO saves (name, is_active) VALUES ('Second save', 0)",
        [],
    )
    .expect("create second save");
    let second_save_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO planner_teams (save_id, team, display_name) VALUES (?1, 'senior', 'Senior')",
        [second_save_id],
    )
    .expect("seed second-save team");
    conn.execute(
        "INSERT INTO planner_strings (save_id, team, string_order, display_name)
         VALUES (?1, 'senior', 0, '1st string')",
        [second_save_id],
    )
    .expect("seed second-save string");
    let foreign_id = conn.last_insert_rowid();
    let snapshot_id = current_snapshot_id(&conn, save_id);
    let before = planner_potential_state(&conn, save_id, snapshot_id);

    for (strings, expected) in [
        (
            vec![
                retained_string(senior_id, "1st string"),
                retained_string(senior_id, "2nd string"),
            ],
            "must be unique",
        ),
        (vec![retained_string(999_999, "Ghost")], "not found"),
        (
            vec![retained_string(foreign_id, "Foreign")],
            "does not belong to this save",
        ),
        (
            vec![retained_string(reserves_id, "Wrong team")],
            "does not belong",
        ),
    ] {
        let error = save_team_settings(
            &conn,
            save_id,
            &[
                input_with_strings("senior", "Senior", strings),
                input_with_strings(
                    "reserves",
                    "Reserves",
                    current_strings_input(&conn, save_id, PlannerTeam::Reserves),
                ),
                input_with_strings(
                    "youth",
                    "Youth",
                    current_strings_input(&conn, save_id, PlannerTeam::Youth),
                ),
            ],
            true,
        )
        .expect_err("reject untrusted string IDs");
        assert!(error.contains(expected), "{error}");
    }
    assert_eq!(planner_potential_state(&conn, save_id, snapshot_id), before);
}

#[test]
fn removal_impacts_name_excluded_strings_with_assignment_counts() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let depth = get_depth(&conn, save_id).expect("initialize planner depth");
    let senior_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
    let senior_name = team_strings(&depth, PlannerTeam::Senior)[0]
        .display_name
        .clone();
    let extra = add_string(&conn, save_id, PlannerTeam::Senior)
        .expect("add second string")
        .0;
    let extra_id = extra.id;
    let extra_name = extra.display_name.clone();
    assign_player(&conn, save_id, extra_id, "goalkeeper", 77).expect("assign player");

    let impacts = planner_team_removal_impacts(
        &conn,
        save_id,
        &[
            input_with_strings(
                "senior",
                "Senior",
                vec![retained_string(senior_id, &senior_name)],
            ),
            input_with_strings(
                "reserves",
                "Reserves",
                current_strings_input(&conn, save_id, PlannerTeam::Reserves),
            ),
            input_with_strings(
                "youth",
                "Youth",
                current_strings_input(&conn, save_id, PlannerTeam::Youth),
            ),
        ],
    )
    .expect("read removal impacts");
    assert_eq!(impacts.len(), 1);
    assert_eq!(impacts[0].team, PlannerTeam::Senior);
    assert_eq!(impacts[0].assignment_count, 1);
    assert_eq!(
        impacts[0]
            .strings
            .iter()
            .map(|impact| {
                (
                    impact.string_id,
                    impact.display_name.as_str(),
                    impact.assignment_count,
                )
            })
            .collect::<Vec<_>>(),
        [(extra_id, extra_name.as_str(), 1)]
    );

    let team_impacts = planner_team_removal_impacts(
        &conn,
        save_id,
        &[
            input_with_strings(
                "senior",
                "Senior",
                current_strings_input(&conn, save_id, PlannerTeam::Senior),
            ),
            input_with_strings(
                "youth",
                "Youth",
                current_strings_input(&conn, save_id, PlannerTeam::Youth),
            ),
        ],
    )
    .expect("read team removal impacts");
    assert_eq!(team_impacts.len(), 1);
    assert_eq!(team_impacts[0].team, PlannerTeam::Reserves);
    assert_eq!(team_impacts[0].strings.len(), 1);
}
