use super::depth::PlannerTeam;
use super::outlook::{expected_score, slot_value, Development};

fn development(age: i64, professionalism: i64, ambition: i64, determination: i64) -> Development {
    Development {
        age: Some(age),
        professionalism: Some(professionalism),
        ambition: Some(ambition),
        determination: Some(determination),
    }
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn reach_scales_the_time_a_player_has_left_by_his_drive() {
    assert_close(development(18, 15, 15, 15).reach(), 1.0);
    assert_close(development(18, 10, 10, 10).reach(), 0.8);
    assert_close(development(18, 5, 5, 5).reach(), 0.6);
    assert_close(development(22, 15, 15, 15).reach(), 0.6);
    // A hard worker closes more of the gap late than his age alone allows.
    assert_close(development(22, 20, 20, 20).reach(), 0.72);
    assert_close(development(18, 20, 20, 20).reach(), 1.0);
    assert_close(development(29, 20, 20, 20).reach(), 0.0);
}

#[test]
fn reach_weighs_professionalism_over_ambition_over_determination() {
    let professional = development(18, 20, 1, 1).reach();
    let ambitious = development(18, 1, 20, 1).reach();
    let determined = development(18, 1, 1, 20).reach();

    assert!(professional > ambitious);
    assert!(ambitious > determined);
}

#[test]
fn reach_counts_a_missing_attribute_as_average_and_an_unknown_age_as_no_growth() {
    let unknown_personality = Development {
        age: Some(18),
        professionalism: None,
        ambition: None,
        determination: None,
    };
    let unknown_age = Development {
        age: None,
        ..development(18, 20, 20, 20)
    };

    assert_close(unknown_personality.reach(), 0.8);
    assert_close(unknown_age.reach(), 0.0);
}

#[test]
fn expected_score_closes_the_reachable_share_of_the_gap_to_potential() {
    assert_close(expected_score(58, 86, &development(16, 14, 12, 13)), 84.152);
    // A potential below the current score promises no growth.
    assert_close(expected_score(80, 60, &development(16, 14, 12, 13)), 80.0);
}

#[test]
fn the_senior_team_starts_a_prospect_over_a_declining_veteran_but_not_over_a_prime_player() {
    let senior = |current, potential, development: Development| {
        slot_value(PlannerTeam::Senior, current, potential, &development, false)
    };
    let prospect = senior(70, 84, development(20, 15, 14, 12));
    let veteran = senior(74, 74, development(33, 10, 10, 10));
    let prime = senior(74, 74, development(27, 10, 10, 10));
    let unprofessional_prospect = senior(70, 84, development(20, 5, 8, 9));

    assert_eq!(
        [prospect, veteran, prime, unprofessional_prospect],
        [729, 725, 740, 719]
    );
}

#[test]
fn the_reserves_start_the_higher_ceiling_over_the_better_player_today() {
    let reserves = |current, potential, age| {
        slot_value(
            PlannerTeam::Reserves,
            current,
            potential,
            &development(age, 14, 12, 13),
            false,
        )
    };

    assert_eq!([reserves(58, 86, 16), reserves(66, 72, 20)], [737, 689]);
}

#[test]
fn each_team_weighs_the_same_outlook_more_the_younger_its_squad() {
    let value = |team| slot_value(team, 60, 80, &development(17, 15, 15, 15), false);

    assert_eq!(
        [
            value(PlannerTeam::Senior),
            value(PlannerTeam::Reserves),
            value(PlannerTeam::Youth)
        ],
        [650, 720, 770]
    );
}

#[test]
fn decline_starts_three_years_later_for_goalkeepers_and_stops_at_ten_points() {
    let senior = |age, goalkeeper| {
        slot_value(
            PlannerTeam::Senior,
            74,
            74,
            &development(age, 10, 10, 10),
            goalkeeper,
        )
    };

    assert_eq!(
        [senior(30, false), senior(33, false), senior(40, false)],
        [740, 725, 715]
    );
    assert_eq!(
        [senior(33, true), senior(36, true), senior(45, true)],
        [740, 725, 715]
    );
}
