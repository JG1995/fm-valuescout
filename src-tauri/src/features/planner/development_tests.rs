use rusqlite::Connection;

use super::commands::read_planner_development;
use super::depth::{assign_player, get_depth, PlannerTeam};
use super::development::{
    choose_slot, development_slots, load_settings, same_training, set_max_age, set_optimize_for,
    set_position_familiarity, set_training_basis, DevelopmentSlots, OptimizeFor, TrainingBasis,
};
use super::fit::PositionRule;
use super::tactic::{self, PlannerTactic};
use super::test_support::{
    current_snapshot_id, open_with_snapshot, seed_players, set_lane_scores, set_player_age,
    set_player_club, set_player_positions, team_strings,
};

const WINGER_AND_STRIKER: &str = r#"{"AMR": 18, "MR": 18, "ST": 18}"#;

/// Player 77, aged 19 at the managed club, plays right winger and centre forward. He scores
/// higher on the wing now, but his potential there is the lowest of the save's four wingers,
/// while his potential up front is the highest of its three strikers.
fn winger_who_will_be_a_striker() -> (tempfile::TempDir, Connection, i64) {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    seed_players(
        &temp_dir,
        &mut conn,
        save_id,
        &[77, 201, 202, 203, 204, 205],
    );
    for (uid, potential) in [(201, 80), (202, 85), (203, 90)] {
        set_player_positions(&conn, save_id, uid, r#"{"AMR": 18, "MR": 18}"#);
        set_lane_scores(&conn, save_id, uid, "right_winger", 70, potential);
        set_player_age(&conn, save_id, uid, Some(30));
    }
    for (uid, potential) in [(204, 60), (205, 70)] {
        set_player_positions(&conn, save_id, uid, r#"{"ST": 18}"#);
        set_lane_scores(&conn, save_id, uid, "centre_forward", 60, potential);
        set_player_age(&conn, save_id, uid, Some(30));
    }
    set_player_positions(&conn, save_id, 77, WINGER_AND_STRIKER);
    set_lane_scores(&conn, save_id, 77, "right_winger", 70, 75);
    set_lane_scores(&conn, save_id, 77, "centre_forward", 60, 85);
    set_player_age(&conn, save_id, 77, Some(19));
    (temp_dir, conn, save_id)
}

fn slots(conn: &Connection, save_id: i64, tactic: &PlannerTactic) -> DevelopmentSlots {
    let settings = load_settings(conn, save_id).expect("settings");
    development_slots(
        conn,
        save_id,
        current_snapshot_id(conn, save_id),
        tactic,
        &settings,
    )
    .expect("development slots")
}

fn lane_index(tactic: &PlannerTactic, lane_id: &str) -> usize {
    tactic
        .lanes
        .iter()
        .position(|lane| lane.lane_id == lane_id)
        .expect("tactic lane")
}

#[test]
fn a_player_is_suggested_the_slot_where_his_potential_stands_out() {
    let (_temp_dir, conn, save_id) = winger_who_will_be_a_striker();
    let tactic = tactic::get_tactic(&conn, save_id).expect("tactic");

    let slots = slots(&conn, save_id, &tactic);
    let development = slots.of(77).expect("a 19-year-old develops");

    let striker = lane_index(&tactic, "centre_forward");
    let winger = lane_index(&tactic, "right_winger");
    assert_eq!(development.suggested, Some(striker));
    assert_eq!(development.chosen, Some(striker));
    assert!(!development.locked);
    assert_eq!(slots.lane_of(77), Some(striker));
    assert_eq!(development.chosen_scores(), Some((60, 85)));

    // The slots he can play come first, best potential rank first: he outranks both other
    // strikers, and stands level only with himself among the four wingers.
    let playable = development
        .options
        .iter()
        .filter_map(|option| Some((option.lane_index, option.potential_rank?)))
        .collect::<Vec<_>>();
    assert_eq!(playable.len(), 2);
    assert_eq!(playable[0].0, striker);
    assert!((playable[0].1 - 2.5 / 3.0).abs() < 1e-9);
    assert_eq!(playable[1].0, winger);
    assert!((playable[1].1 - 0.125).abs() < 1e-9);

    // A slot whose roles he has scores for is offered as a position to learn: the left wing
    // asks for the winger's roles. Slots that train the same are offered once, and the
    // goalkeeper's never.
    let learnable = &development.options[2..];
    assert_eq!(
        learnable
            .iter()
            .map(|option| (option.lane_index, option.potential_rank))
            .collect::<Vec<_>>(),
        vec![(lane_index(&tactic, "left_winger"), None)]
    );
    assert!(development
        .options
        .iter()
        .all(|option| tactic.lanes[option.lane_index].lane_id != "goalkeeper"));
    for (index, option) in development.options.iter().enumerate() {
        for other in &development.options[index + 1..] {
            assert!(!same_training(
                &tactic.lanes[option.lane_index],
                &tactic.lanes[other.lane_index]
            ));
        }
    }
}

#[test]
fn without_position_familiarity_a_slot_is_suggested_in_any_position() {
    let (_temp_dir, conn, save_id) = winger_who_will_be_a_striker();
    // He has a striker's scores but can no longer play up front.
    set_player_positions(&conn, save_id, 77, r#"{"AMR": 18, "MR": 18}"#);
    let tactic = tactic::get_tactic(&conn, save_id).expect("tactic");
    let striker = lane_index(&tactic, "centre_forward");
    let winger = lane_index(&tactic, "right_winger");
    let option = |development: &super::development::PlayerDevelopment, lane_index: usize| {
        development
            .options
            .iter()
            .find(|option| option.lane_index == lane_index)
            .map(|option| (option.playable, option.potential_rank))
            .expect("an option for the lane")
    };

    // By position he develops on the wing, the only slot he can play, and the striker's slot
    // is a position to learn.
    let by_position = slots(&conn, save_id, &tactic)
        .of(77)
        .cloned()
        .expect("development");
    assert_eq!(by_position.suggested, Some(winger));
    assert_eq!(option(&by_position, striker), (false, None));

    set_position_familiarity(&conn, save_id, false).expect("stop holding positions against him");
    assert_eq!(
        load_settings(&conn, save_id)
            .expect("settings")
            .position_rule,
        PositionRule::AnyPosition
    );

    // On role scores alone his potential outranks both of the save's strikers, so that is
    // where he is growing, though he still cannot play there today.
    let anywhere = slots(&conn, save_id, &tactic)
        .of(77)
        .cloned()
        .expect("development");
    assert_eq!(anywhere.suggested, Some(striker));
    assert_eq!(anywhere.chosen, Some(striker));
    assert_eq!(anywhere.options[0].lane_index, striker);
    assert_eq!(option(&anywhere, striker), (false, Some(1.0)));
    assert_eq!(option(&anywhere, winger), (true, Some(0.125)));
    // Every slot his roles are scored for is ranked now, so none is left as only to learn.
    assert!(anywhere
        .options
        .iter()
        .all(|option| option.potential_rank.is_some()));
}

#[test]
fn a_chosen_slot_replaces_the_suggestion_until_it_is_cleared() {
    let (_temp_dir, conn, save_id) = winger_who_will_be_a_striker();
    let tactic = tactic::get_tactic(&conn, save_id).expect("tactic");
    let snapshot_id = current_snapshot_id(&conn, save_id);
    let striker = lane_index(&tactic, "centre_forward");
    // A position he has yet to learn: the left wing asks for his winger's roles.
    let left_winger = lane_index(&tactic, "left_winger");

    choose_slot(&conn, save_id, snapshot_id, 77, Some("left_winger")).expect("choose the left");
    let development = slots(&conn, save_id, &tactic)
        .of(77)
        .cloned()
        .expect("development");
    assert_eq!(development.chosen, Some(left_winger));
    assert_eq!(development.suggested, Some(striker));
    assert!(development.locked);

    // A second choice replaces the first.
    choose_slot(&conn, save_id, snapshot_id, 77, Some("right_winger")).expect("choose winger");
    assert_eq!(
        slots(&conn, save_id, &tactic).lane_of(77),
        Some(lane_index(&tactic, "right_winger"))
    );

    choose_slot(&conn, save_id, snapshot_id, 77, None).expect("clear the choice");
    let development = slots(&conn, save_id, &tactic)
        .of(77)
        .cloned()
        .expect("development");
    assert_eq!(development.chosen, Some(striker));
    assert!(!development.locked);
}

#[test]
fn a_choice_needs_a_slot_of_the_tactic_and_a_player_at_the_club() {
    let (_temp_dir, conn, save_id) = winger_who_will_be_a_striker();
    let tactic = tactic::get_tactic(&conn, save_id).expect("tactic");
    let snapshot_id = current_snapshot_id(&conn, save_id);

    assert_eq!(
        choose_slot(&conn, save_id, snapshot_id, 77, Some("sweeper")),
        Err("Unknown tactic lane `sweeper`".to_string())
    );
    set_player_club(&conn, save_id, 201, "Elsewhere FC");
    assert_eq!(
        choose_slot(&conn, save_id, snapshot_id, 201, Some("left_winger")),
        Err("The player is not at the managed club".to_string())
    );
    // A goalkeeper's slot is no choice for an outfield player, so the suggestion stands.
    choose_slot(&conn, save_id, snapshot_id, 77, Some("goalkeeper")).expect("store the choice");
    let development = slots(&conn, save_id, &tactic)
        .of(77)
        .cloned()
        .expect("development");
    assert_eq!(
        development.chosen,
        Some(lane_index(&tactic, "centre_forward"))
    );
    assert!(!development.locked);
}

#[test]
fn only_players_up_to_the_age_limit_develop() {
    let (_temp_dir, conn, save_id) = winger_who_will_be_a_striker();
    let tactic = tactic::get_tactic(&conn, save_id).expect("tactic");

    // The peers are 30, and the default limit is 23.
    assert!(slots(&conn, save_id, &tactic).of(201).is_none());
    set_player_age(&conn, save_id, 77, Some(24));
    assert!(slots(&conn, save_id, &tactic).of(77).is_none());
    set_max_age(&conn, save_id, 24).expect("raise the limit");
    assert!(slots(&conn, save_id, &tactic).of(77).is_some());
    set_player_age(&conn, save_id, 77, None);
    assert!(slots(&conn, save_id, &tactic).of(77).is_none());

    for out_of_range in [14, 29] {
        assert!(set_max_age(&conn, save_id, out_of_range).is_err());
    }
    assert_eq!(load_settings(&conn, save_id).expect("settings").max_age, 24);
}

#[test]
fn slots_that_train_the_same_count_as_one_and_follow_the_planner() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    seed_players(&temp_dir, &mut conn, save_id, &[77]);
    set_player_positions(&conn, save_id, 77, r#"{"DC": 18}"#);
    set_lane_scores(&conn, save_id, 77, "left_centre_back", 60, 80);
    set_player_age(&conn, save_id, 77, Some(19));
    let tactic = tactic::get_tactic(&conn, save_id).expect("tactic");
    let left = lane_index(&tactic, "left_centre_back");
    let right = lane_index(&tactic, "right_centre_back");
    assert!(same_training(&tactic.lanes[left], &tactic.lanes[right]));
    assert!(!same_training(
        &tactic.lanes[lane_index(&tactic, "left_winger")],
        &tactic.lanes[lane_index(&tactic, "right_winger")]
    ));

    // Level in both, he is offered the earlier one.
    let development = slots(&conn, save_id, &tactic)
        .of(77)
        .cloned()
        .expect("development");
    let centre_backs = development
        .options
        .iter()
        .filter(|option| same_training(&tactic.lanes[option.lane_index], &tactic.lanes[left]))
        .count();
    assert_eq!(centre_backs, 1);
    assert_eq!(development.chosen, Some(left));

    // In the Planner's right centre-back slot, that is the one he develops in.
    let depth = get_depth(&conn, save_id).expect("planner depth");
    let senior = team_strings(&depth, PlannerTeam::Senior)[0].id;
    assign_player(&conn, save_id, senior, "right_centre_back", 77).expect("place player");
    assert_eq!(slots(&conn, save_id, &tactic).lane_of(77), Some(right));
}

#[test]
fn a_save_starts_on_planner_training_and_the_strongest_squads() {
    let (_temp_dir, conn, save_id) = open_with_snapshot();
    let settings = load_settings(&conn, save_id).expect("settings");
    assert_eq!(settings.training_basis, TrainingBasis::Planner);
    assert_eq!(settings.optimize_for, OptimizeFor::Strongest);
    assert_eq!(settings.max_age, 23);
    assert_eq!(settings.position_rule, PositionRule::Familiarity);

    set_training_basis(&conn, save_id, TrainingBasis::Development).expect("train by development");
    set_optimize_for(&conn, save_id, OptimizeFor::Development).expect("optimize for development");
    let settings = load_settings(&conn, save_id).expect("settings");
    assert_eq!(settings.training_basis, TrainingBasis::Development);
    assert_eq!(settings.optimize_for, OptimizeFor::Development);
}

#[test]
fn the_board_reads_each_developing_players_slots() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    seed_players(&temp_dir, &mut conn, save_id, &[77, 78]);
    for (uid, age) in [(77, 19), (78, 30)] {
        set_player_positions(&conn, save_id, uid, r#"{"DC": 18}"#);
        set_lane_scores(&conn, save_id, uid, "left_centre_back", 60, 80);
        set_player_age(&conn, save_id, uid, Some(age));
    }
    get_depth(&conn, save_id).expect("create the board");
    set_optimize_for(&conn, save_id, OptimizeFor::Development).expect("optimize for development");

    let development =
        read_planner_development(&crate::db::Db::new(conn)).expect("read development");

    assert_eq!(development.optimize_for, "development");
    assert_eq!(development.max_age, 23);
    // Only the 19-year-old develops, and either centre-back slot is his.
    assert_eq!(
        development
            .players
            .iter()
            .map(|player| (player.player_uid, player.lane_ids.clone()))
            .collect::<Vec<_>>(),
        vec![(
            77,
            vec![
                "left_centre_back".to_string(),
                "right_centre_back".to_string()
            ]
        )]
    );
}
