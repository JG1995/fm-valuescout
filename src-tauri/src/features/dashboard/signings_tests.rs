use serde_json::json;

use crate::features::moneyball::comparison::GroupsSource;
use crate::features::planner::tactic::{default_tactic, save_tactic};
use crate::features::snapshot;

use super::signings::{
    get_signings, IncumbentSource, LineupSlot, Recommendation, ScoreBasis, ScoreMode, Signings,
    SigningsRequest,
};
use super::test_support::{open_with_players, player, Fixture, HOME_CLUB, OTHER_CLUB};

const GENERAL: SigningsRequest<'static> = SigningsRequest {
    mode: ScoreMode::General,
    shortlist_id: None,
    lane_id: None,
    season_stats: true,
};
const MONEYBALL: SigningsRequest<'static> = SigningsRequest {
    mode: ScoreMode::Moneyball,
    ..GENERAL
};

fn signings(fixture: &Fixture, request: SigningsRequest<'_>) -> Signings {
    get_signings(
        &fixture.conn,
        &GroupsSource::Uncached,
        fixture.save_id,
        request,
    )
    .expect("read signings")
    .expect("signings for a managed club")
}

fn slot<'a>(signings: &'a Signings, lane_id: &str) -> &'a LineupSlot {
    signings
        .slots
        .iter()
        .find(|slot| slot.lane.lane_id == lane_id)
        .expect("tactic slot")
}

fn incumbent(signings: &Signings, lane_id: &str) -> Option<(i64, u8, IncumbentSource)> {
    slot(signings, lane_id)
        .incumbent
        .as_ref()
        .map(|(player, source)| (player.uid, player.score, *source))
}

/// Each recommendation as `(uid, lane, score, improvement)`.
fn ranked(recommendations: &[Recommendation]) -> Vec<(i64, &str, u8, u8)> {
    recommendations
        .iter()
        .map(|recommendation| {
            (
                recommendation.uid,
                recommendation.lane_id.as_str(),
                recommendation.score,
                recommendation.improvement,
            )
        })
        .collect()
}

#[test]
fn ranks_shortlisted_players_by_improvement_over_each_slots_incumbent() {
    let fixture = open_with_players(&[
        player(1, "Home Keeper", HOME_CLUB, "GK"),
        player(2, "Home Left Back", HOME_CLUB, "DL"),
        player(10, "Better Keeper", OTHER_CLUB, "GK"),
        player(11, "Better Left Back", OTHER_CLUB, "DL"),
        player(12, "Worse Left Back", OTHER_CLUB, "DL"),
        player(13, "Unlisted Left Back", OTHER_CLUB, "DL"),
    ]);
    fixture.set_slot_score(1, "goalkeeper", 70);
    fixture.set_slot_score(2, "left_back", 60);
    fixture.set_slot_score(10, "goalkeeper", 75);
    fixture.set_slot_score(11, "left_back", 80);
    fixture.set_slot_score(12, "left_back", 55);
    fixture.set_slot_score(13, "left_back", 95);
    let shortlist = fixture.add_shortlist("Targets", &[2, 10, 11, 12]);

    let signings = signings(
        &fixture,
        SigningsRequest {
            shortlist_id: Some(shortlist),
            ..GENERAL
        },
    );

    assert_eq!(
        incumbent(&signings, "goalkeeper"),
        Some((1, 70, IncumbentSource::BestFit))
    );
    assert_eq!(
        incumbent(&signings, "left_back"),
        Some((2, 60, IncumbentSource::BestFit))
    );
    assert_eq!(
        slot(&signings, "left_back").score_basis,
        ScoreBasis::Attributes
    );
    assert_eq!(
        ranked(&signings.recommendations),
        [(11, "left_back", 80, 20), (10, "goalkeeper", 75, 5)]
    );
    assert_eq!(signings.total_recommendations, 2);
    // The club's own shortlisted player is not a transfer target.
    assert_eq!(signings.candidates, Some(3));
    let best = &signings.recommendations[0];
    assert_eq!(best.club.as_deref(), Some(OTHER_CLUB));
    assert_eq!(best.minutes, None);
}

#[test]
fn open_slots_take_distinct_club_players_and_the_rest_are_cover() {
    let fixture = open_with_players(&[
        player(1, "First Centre Back", HOME_CLUB, "DC"),
        player(2, "Second Centre Back", HOME_CLUB, "DC"),
        player(3, "Third Centre Back", HOME_CLUB, "DC"),
        player(10, "Target Centre Back", OTHER_CLUB, "DC"),
    ]);
    for (uid, score) in [(1, 80), (2, 70), (3, 60), (10, 75)] {
        fixture.set_slot_score(uid, "left_centre_back", score);
    }

    let signings = signings(&fixture, GENERAL);

    let mut starters = ["left_centre_back", "right_centre_back"]
        .map(|lane_id| incumbent(&signings, lane_id).expect("incumbent"));
    starters.sort_by_key(|&(uid, _, _)| uid);
    assert_eq!(
        starters,
        [
            (1, 80, IncumbentSource::BestFit),
            (2, 70, IncumbentSource::BestFit)
        ]
    );
    for lane_id in ["left_centre_back", "right_centre_back"] {
        let cover = slot(&signings, lane_id).cover.as_ref().expect("cover");
        assert_eq!((cover.uid, cover.score), (3, 60));
    }
    // The target beats only the weaker starter, so that slot is the recommendation.
    let recommendations = ranked(&signings.recommendations);
    assert_eq!(recommendations.len(), 1);
    assert_eq!(
        (
            recommendations[0].0,
            recommendations[0].2,
            recommendations[0].3
        ),
        (10, 75, 5)
    );
    assert!(signings.recommendations[0].other_lanes.is_empty());
}

#[test]
fn the_planner_first_string_player_is_the_slots_incumbent() {
    let fixture = open_with_players(&[
        player(1, "First Centre Back", HOME_CLUB, "DC"),
        player(2, "Second Centre Back", HOME_CLUB, "DC"),
        player(3, "Planner Centre Back", HOME_CLUB, "DC"),
        player(4, "Planner Keeper", HOME_CLUB, "GK"),
        player(5, "Home Left Back", HOME_CLUB, "DL"),
        player(10, "Target Centre Back", OTHER_CLUB, "DC"),
    ]);
    for (uid, score) in [(1, 80), (2, 70), (3, 60), (10, 75)] {
        fixture.set_slot_score(uid, "left_centre_back", score);
    }
    fixture.set_slot_score(5, "left_back", 65);
    fixture.assign_first_string("left_centre_back", 3);
    // A keeper has no score at left back, so the Planner's pick cannot be compared there.
    fixture.assign_first_string("left_back", 4);

    let signings = signings(&fixture, GENERAL);

    assert_eq!(
        incumbent(&signings, "left_centre_back"),
        Some((3, 60, IncumbentSource::Planner))
    );
    assert_eq!(
        incumbent(&signings, "right_centre_back"),
        Some((1, 80, IncumbentSource::BestFit))
    );
    assert_eq!(
        incumbent(&signings, "left_back"),
        Some((5, 65, IncumbentSource::BestFit))
    );
    assert_eq!(
        slot(&signings, "left_back")
            .unscored_planner_player
            .as_deref(),
        Some("Planner Keeper")
    );
    assert_eq!(
        slot(&signings, "left_centre_back").unscored_planner_player,
        None
    );
    assert_eq!(
        ranked(&signings.recommendations),
        [(10, "left_centre_back", 75, 15)]
    );
}

#[test]
fn a_slot_without_an_incumbent_counts_the_whole_score_and_a_slot_filter_narrows_the_ranking() {
    let fixture = open_with_players(&[
        player(1, "First Centre Back", HOME_CLUB, "DC"),
        player(2, "Second Centre Back", HOME_CLUB, "DC"),
        player(10, "Target Centre Back", OTHER_CLUB, "DC"),
        player(11, "Target Striker", OTHER_CLUB, "ST"),
    ]);
    for (uid, score) in [(1, 80), (2, 70), (10, 85)] {
        fixture.set_slot_score(uid, "left_centre_back", score);
    }
    fixture.set_slot_score(11, "centre_forward", 66);

    let all_slots = signings(&fixture, GENERAL);

    assert_eq!(incumbent(&all_slots, "centre_forward"), None);
    assert_eq!(
        (
            all_slots.recommendations[0].uid,
            all_slots.recommendations[0].improvement
        ),
        (11, 66)
    );
    let centre_back = &all_slots.recommendations[1];
    assert_eq!((centre_back.uid, centre_back.improvement), (10, 15));
    assert_eq!(centre_back.other_lanes.len(), 1);
    assert_eq!(centre_back.other_lanes[0].improvement, 5);
    let stronger_slot = centre_back.other_lanes[0].lane_id.clone();

    let one_slot = signings(
        &fixture,
        SigningsRequest {
            lane_id: Some(&stronger_slot),
            ..GENERAL
        },
    );

    assert_eq!(
        ranked(&one_slot.recommendations),
        [(10, stronger_slot.as_str(), 85, 5)]
    );
    assert!(one_slot.recommendations[0].other_lanes.is_empty());
    assert_eq!(
        get_signings(
            &fixture.conn,
            &GroupsSource::Uncached,
            fixture.save_id,
            SigningsRequest {
                lane_id: Some("sweeper"),
                ..GENERAL
            }
        ),
        Err("unknown tactic lane: sweeper".to_string())
    );
}

#[test]
fn names_each_slots_best_signing_whichever_slot_suits_the_player_most() {
    let fixture = open_with_players(&[
        player(1, "First Centre Back", HOME_CLUB, "DC"),
        player(2, "Second Centre Back", HOME_CLUB, "DC"),
        player(3, "Home Keeper", HOME_CLUB, "GK"),
        player(10, "Target Centre Back", OTHER_CLUB, "DC"),
        player(11, "Lesser Centre Back", OTHER_CLUB, "DC"),
        player(12, "Worse Keeper", OTHER_CLUB, "GK"),
    ]);
    for (uid, score) in [(1, 70), (2, 60), (10, 85), (11, 65)] {
        fixture.set_slot_score(uid, "left_centre_back", score);
    }
    fixture.set_slot_score(3, "goalkeeper", 70);
    fixture.set_slot_score(12, "goalkeeper", 60);

    // A slot filter narrows the ranking, never the per-slot answers.
    for request in [
        GENERAL,
        SigningsRequest {
            lane_id: Some("goalkeeper"),
            ..GENERAL
        },
    ] {
        let signings = signings(&fixture, request);

        // The target is listed once in the ranking, yet is the best signing for both slots.
        let mut centre_backs = ["left_centre_back", "right_centre_back"].map(|lane_id| {
            let best = slot(&signings, lane_id)
                .best_signing
                .as_ref()
                .expect("best signing");
            assert_eq!(best.lane_id, lane_id);
            (best.uid, best.score, best.improvement)
        });
        centre_backs.sort();
        assert_eq!(centre_backs, [(10, 85, 15), (10, 85, 25)]);
        assert_eq!(slot(&signings, "goalkeeper").best_signing, None);
        assert_eq!(slot(&signings, "left_back").best_signing, None);
    }
    assert_eq!(
        ranked(&signings(&fixture, GENERAL).recommendations)
            .iter()
            .map(|&(uid, _, _, improvement)| (uid, improvement))
            .collect::<Vec<_>>(),
        [(10, 25), (11, 5)]
    );
}

#[test]
fn moneyball_mode_scores_a_slot_by_its_custom_metrics_or_else_its_roles() {
    let fixture = open_with_players(&[
        player(1, "Home Striker", HOME_CLUB, "ST"),
        player(2, "Home Keeper", HOME_CLUB, "GK"),
        player(10, "Target Striker", OTHER_CLUB, "ST"),
        player(11, "Target Keeper", OTHER_CLUB, "GK"),
        player(12, "Keeper Without Stats", OTHER_CLUB, "GK"),
    ]);
    let mut tactic = default_tactic();
    tactic
        .lanes
        .iter_mut()
        .find(|lane| lane.lane_id == "centre_forward")
        .expect("centre forward")
        .custom_metric_keys = vec!["goals_per_90".to_string(), "xg_per_90".to_string()];
    save_tactic(&fixture.conn, fixture.save_id, &tactic).expect("save tactic");
    // Attribute scores would rank the home striker first; Moneyball mode must ignore them.
    fixture.set_slot_score(1, "centre_forward", 99);
    // The home striker ranks above the target at everything but the slot's custom metrics.
    fixture.set_season_stats(1, 80.0, &[("goals_per_90", 1.0), ("xg_per_90", 1.0)]);
    fixture.set_season_stats(10, 20.0, &[("goals_per_90", 9.0), ("xg_per_90", 9.0)]);
    fixture.set_season_stats(2, 40.0, &[]);
    fixture.set_season_stats(11, 60.0, &[]);

    let signings = signings(&fixture, MONEYBALL);

    assert!(signings.scores_available);
    assert!(!signings.tactic_is_default);
    assert_eq!(
        slot(&signings, "centre_forward").score_basis,
        ScoreBasis::MoneyballCustom
    );
    assert_eq!(
        slot(&signings, "goalkeeper").score_basis,
        ScoreBasis::MoneyballRoles
    );
    // Each pair of peers ranks at 0 and 100: the custom metrics put the target striker
    // first, and the goalkeeper slot's roles the target keeper.
    assert_eq!(
        incumbent(&signings, "centre_forward"),
        Some((1, 0, IncumbentSource::BestFit))
    );
    assert_eq!(
        incumbent(&signings, "goalkeeper"),
        Some((2, 0, IncumbentSource::BestFit))
    );
    assert_eq!(
        ranked(&signings.recommendations),
        [
            (11, "goalkeeper", 100, 100),
            (10, "centre_forward", 100, 100)
        ]
    );
    assert_eq!(signings.recommendations[0].minutes, Some(900));
}

#[test]
fn moneyball_scores_rank_within_the_roles_families_and_the_slots_group() {
    let fixture = open_with_players(&[
        player(1, "Home Wide Player", HOME_CLUB, "AML"),
        player(2, "Home Keeper", HOME_CLUB, "GK"),
        player(10, "Target Wide Player", OTHER_CLUB, "AML"),
        player(11, "Target Keeper", OTHER_CLUB, "GK"),
        player(13, "Winger", OTHER_CLUB, "AML"),
        player(14, "Wide Midfielder", OTHER_CLUB, "ML"),
        player(15, "Striker", OTHER_CLUB, "ST"),
    ]);
    for uid in [1, 10] {
        fixture.set_player_column(uid, "positions_json", r#"{"AML": 20, "ML": 20}"#);
    }
    let mut tactic = default_tactic();
    tactic
        .lanes
        .iter_mut()
        .find(|lane| lane.lane_id == "left_winger")
        .expect("left winger")
        .custom_metric_keys = vec!["goals_per_90".to_string()];
    save_tactic(&fixture.conn, fixture.save_id, &tactic).expect("save tactic");
    for (uid, goals_per_90) in [(1, 2.0), (10, 6.0), (13, 4.0), (14, 8.0)] {
        fixture.set_season_stats(uid, 50.0, &[("goals_per_90", goals_per_90)]);
    }
    fixture.set_season_stats(2, 40.0, &[]);
    fixture.set_season_stats(11, 60.0, &[]);
    // Better than everyone at everything, but neither a keeper nor a wide player.
    fixture.set_season_stats(15, 90.0, &[("goals_per_90", 100.0)]);
    let slot_signings = |lane_id| {
        signings(
            &fixture,
            SigningsRequest {
                lane_id: Some(lane_id),
                ..MONEYBALL
            },
        )
    };

    // The AML/ML slot ranks among wingers and wide midfielders together: 2, 4, 6, and 8, so
    // 6 is at the 67th percentile. Wingers alone would put it at 100, everyone at 50.
    assert_eq!(
        ranked(&slot_signings("left_winger").recommendations),
        [(10, "left_winger", 67, 67)]
    );
    // The goalkeeper's roles rank among keepers only, so the striker does not count.
    assert_eq!(
        ranked(&slot_signings("goalkeeper").recommendations),
        [(11, "goalkeeper", 100, 100)]
    );
}

#[test]
fn moneyball_mode_marks_a_signing_down_for_playing_in_a_weaker_league() {
    let fixture = open_with_players(&[
        player(1, "Home Keeper", HOME_CLUB, "GK"),
        player(10, "Target Keeper", OTHER_CLUB, "GK"),
    ]);
    fixture.set_season_stats(1, 60.0, &[]);
    fixture.set_season_stats(10, 75.0, &[]);
    fixture.set_player_column(1, "division", "Top");
    fixture.set_player_column(10, "division", "Second");
    fixture.measure_leagues(&[("Top", 150.0), ("Second", 140.0)]);

    let signings = signings(&fixture, MONEYBALL);

    // The managed club's league is the reference, so its own keeper keeps his 0. The
    // target's 100 loses half of the 10 points between the leagues.
    assert_eq!(
        incumbent(&signings, "goalkeeper"),
        Some((1, 0, IncumbentSource::BestFit))
    );
    assert_eq!(
        ranked(&signings.recommendations),
        [(10, "goalkeeper", 95, 95)]
    );
}

#[test]
fn moneyball_mode_without_season_stats_reports_that_no_scores_exist() {
    let fixture = open_with_players(&[
        player(1, "Home Keeper", HOME_CLUB, "GK"),
        player(10, "Target Keeper", OTHER_CLUB, "GK"),
    ]);
    fixture.set_slot_score(1, "goalkeeper", 70);
    fixture.set_slot_score(10, "goalkeeper", 90);

    let signings = signings(&fixture, MONEYBALL);

    assert!(!signings.scores_available);
    assert_eq!(incumbent(&signings, "goalkeeper"), None);
    assert!(signings.recommendations.is_empty());
}

#[test]
fn without_a_shortlist_the_candidates_are_the_clubs_realistic_signings() {
    let fixture = open_with_players(&[
        player(1, "Home Keeper", HOME_CLUB, "GK"),
        player(10, "Keeper Within Reach", OTHER_CLUB, "GK"),
        player(11, "Keeper Out Of Reach", OTHER_CLUB, "GK"),
        player(12, "Home Keeper On Loan", OTHER_CLUB, "GK"),
    ]);
    for (uid, score) in [(1, 50), (10, 60), (11, 90), (12, 80)] {
        fixture.set_slot_score(uid, "goalkeeper", score);
    }
    // Far above the club's level, so the club could not sign him.
    fixture.set_player_column(11, "ca", 200);
    // The club's own player, out on loan, is not a transfer target.
    fixture.set_player_column(12, "on_loan", 1);
    fixture.set_player_column(12, "parent_club", HOME_CLUB);
    fixture.set_player_column(12, "parent_club_uid", 1000);
    let dreams = fixture.add_shortlist("Dreams", &[11]);

    let realistic = signings(&fixture, GENERAL);

    assert_eq!(
        ranked(&realistic.recommendations),
        [(10, "goalkeeper", 60, 10)]
    );
    // Realistic signings are not counted.
    assert_eq!(realistic.candidates, None);
    assert_eq!(
        slot(&realistic, "goalkeeper")
            .best_signing
            .as_ref()
            .map(|best| best.uid),
        Some(10)
    );

    // A shortlist is the manager's own choice, so its players are compared as they stand.
    let shortlisted = signings(
        &fixture,
        SigningsRequest {
            shortlist_id: Some(dreams),
            ..GENERAL
        },
    );
    assert_eq!(
        ranked(&shortlisted.recommendations),
        [(11, "goalkeeper", 90, 40)]
    );
    assert_eq!(shortlisted.candidates, Some(1));

    // A keeper nobody within reach improves on still leaves realistic signings to compare.
    fixture.set_slot_score(1, "goalkeeper", 95);
    let none_better = signings(&fixture, GENERAL);
    assert!(none_better.recommendations.is_empty());
    assert_eq!(none_better.candidates, None);
}

#[test]
fn a_realistic_signing_better_in_one_phase_only_can_still_improve_a_slot() {
    let fixture = open_with_players(&[
        player(1, "Home Keeper", HOME_CLUB, "GK"),
        player(10, "Keeper Better In Possession", OTHER_CLUB, "GK"),
        player(11, "Keeper Better Out Of Possession", OTHER_CLUB, "GK"),
        player(12, "Keeper Better In Neither", OTHER_CLUB, "GK"),
    ]);
    fixture.set_slot_score(1, "goalkeeper", 60);
    fixture.set_slot_role_scores(10, "goalkeeper", 90, 40);
    fixture.set_slot_role_scores(11, "goalkeeper", 50, 74);
    fixture.set_slot_role_scores(12, "goalkeeper", 60, 60);

    assert_eq!(
        ranked(&signings(&fixture, GENERAL).recommendations),
        [(10, "goalkeeper", 65, 5), (11, "goalkeeper", 62, 2)]
    );
}

#[test]
fn realistic_signings_in_moneyball_mode_are_those_with_season_stats() {
    let fixture = open_with_players(&[
        player(1, "Home Keeper", HOME_CLUB, "GK"),
        player(10, "Keeper With Stats", OTHER_CLUB, "GK"),
        player(11, "Keeper Without Stats", OTHER_CLUB, "GK"),
        player(12, "Keeper Out Of Reach", OTHER_CLUB, "GK"),
    ]);
    fixture.set_season_stats(1, 60.0, &[]);
    fixture.set_season_stats(10, 75.0, &[]);
    fixture.set_season_stats(12, 90.0, &[]);
    fixture.set_player_column(12, "ca", 200);

    let with_stats = signings(&fixture, MONEYBALL);

    // The three keepers with season stats rank at 0, 50, and 100.
    assert_eq!(
        ranked(&with_stats.recommendations),
        [(10, "goalkeeper", 50, 50)]
    );
    assert_eq!(with_stats.candidates, None);

    // A player without season stats has no Moneyball score to compare.
    fixture
        .conn
        .execute(
            "DELETE FROM player_moneyball_stats WHERE player_uid = 10",
            [],
        )
        .expect("remove the target's stats");
    let without_stats = signings(&fixture, MONEYBALL);
    assert!(without_stats.recommendations.is_empty());
    assert_eq!(without_stats.candidates, Some(0));
}

#[test]
fn a_club_without_a_recorded_uid_has_no_realistic_signings() {
    let fixture = open_with_players(&[
        player(1, "Home Keeper", HOME_CLUB, "GK"),
        player(10, "Target Keeper", OTHER_CLUB, "GK"),
    ]);
    fixture.set_slot_score(1, "goalkeeper", 50);
    fixture.set_slot_score(10, "goalkeeper", 60);
    // A club chosen before club UIDs were recorded.
    fixture
        .conn
        .execute("UPDATE managed_club_settings SET club_uid = NULL", [])
        .expect("clear club uid");

    let signings = signings(&fixture, GENERAL);

    assert_eq!(
        incumbent(&signings, "goalkeeper"),
        Some((1, 50, IncumbentSource::BestFit))
    );
    assert_eq!(signings.candidates, Some(0));
    assert!(signings.recommendations.is_empty());
}

#[test]
fn compares_one_of_the_saves_shortlists_and_never_another_saves() {
    let fixture = open_with_players(&[
        player(1, "Home Keeper", HOME_CLUB, "GK"),
        player(10, "First Target", OTHER_CLUB, "GK"),
        player(11, "Second Target", OTHER_CLUB, "GK"),
    ]);
    fixture.set_slot_score(1, "goalkeeper", 50);
    fixture.set_slot_score(10, "goalkeeper", 60);
    fixture.set_slot_score(11, "goalkeeper", 70);
    let first = fixture.add_shortlist("First", &[10]);
    let second = fixture.add_shortlist("Second", &[11]);
    let other_save = snapshot::service::create_save(&fixture.conn, "Other save")
        .expect("create save")
        .id;
    fixture
        .conn
        .execute(
            "INSERT INTO player_shortlists (id, save_id, name) VALUES (900, ?1, 'Foreign')",
            [other_save],
        )
        .expect("insert foreign shortlist");
    fixture
        .conn
        .execute(
            "INSERT INTO player_shortlist_entries (shortlist_id, player_uid) VALUES (900, 10)",
            [],
        )
        .expect("insert foreign entry");
    let with_shortlist = |shortlist_id| {
        let signings = signings(
            &fixture,
            SigningsRequest {
                shortlist_id: Some(shortlist_id),
                ..GENERAL
            },
        );
        signings
            .recommendations
            .iter()
            .map(|recommendation| recommendation.uid)
            .collect::<Vec<_>>()
    };

    assert_eq!(with_shortlist(first), [10]);
    assert_eq!(with_shortlist(second), [11]);
    assert_eq!(with_shortlist(900), Vec::<i64>::new());
}

#[test]
fn reads_the_default_tactic_without_storing_it() {
    let fixture = open_with_players(&[player(1, "Home Keeper", HOME_CLUB, "GK")]);

    let signings = signings(&fixture, GENERAL);

    assert!(signings.tactic_is_default);
    assert_eq!(signings.slots.len(), 11);
    let stored_lanes: i64 = fixture
        .conn
        .query_row("SELECT COUNT(*) FROM planner_tactic_lanes", [], |row| {
            row.get(0)
        })
        .expect("count lanes");
    assert_eq!(stored_lanes, 0);
}

#[test]
fn has_no_signings_without_a_managed_club() {
    let fixture = open_with_players(&[player(1, "Home Keeper", HOME_CLUB, "GK")]);
    fixture
        .conn
        .execute("DELETE FROM managed_club_settings", [])
        .expect("clear managed club");

    assert_eq!(
        get_signings(
            &fixture.conn,
            &GroupsSource::Uncached,
            fixture.save_id,
            GENERAL
        ),
        Ok(None)
    );
}

#[test]
fn a_signing_is_recommended_only_for_slots_whose_recruitment_filters_he_passes() {
    let fixture = open_with_players(&[
        player(1, "Home Centre Back", HOME_CLUB, "DC"),
        player(2, "Other Home Centre Back", HOME_CLUB, "DC"),
        player(10, "Right-Footed Target", OTHER_CLUB, "DC"),
        player(11, "Left-Footed Target", OTHER_CLUB, "DC"),
    ]);
    for (uid, score) in [(1, 50), (2, 50), (10, 70), (11, 80)] {
        fixture.set_slot_score(uid, "left_centre_back", score);
    }
    fixture.set_player_column(10, "preferred_foot", "right");
    fixture.set_player_column(11, "preferred_foot", "left");
    fixture.set_recruitment_filters(
        "left_centre_back",
        &[
            ("preferred_foot", "in", json!(["right"])),
            ("height", "gte", json!(150)),
        ],
    );
    let shortlist = fixture.add_shortlist("Targets", &[10, 11]);

    for request in [
        GENERAL,
        SigningsRequest {
            shortlist_id: Some(shortlist),
            ..GENERAL
        },
    ] {
        let all_slots = signings(&fixture, request);
        // The left-footed target fails the left centre-back's filters, so he moves to the
        // unfiltered right centre-back slot and nobody counts as filtered out.
        assert_eq!(
            ranked(&all_slots.recommendations),
            [
                (11, "right_centre_back", 80, 30),
                (10, "left_centre_back", 70, 20)
            ]
        );
        assert_eq!(all_slots.filtered_out, 0);
        assert_eq!(
            slot(&all_slots, "left_centre_back")
                .best_signing
                .as_ref()
                .map(|best| best.uid),
            Some(10)
        );
        assert_eq!(
            slot(&all_slots, "left_centre_back")
                .lane
                .recruitment_filters
                .len(),
            2
        );

        let one_slot = signings(
            &fixture,
            SigningsRequest {
                lane_id: Some("left_centre_back"),
                ..request
            },
        );
        assert_eq!(
            ranked(&one_slot.recommendations),
            [(10, "left_centre_back", 70, 20)]
        );
        assert_eq!(one_slot.filtered_out, 1);
    }
}

#[test]
fn a_percentile_filter_ranks_a_signing_among_the_slots_position_peers() {
    let fixture = open_with_players(&[
        player(1, "Home Centre Back", HOME_CLUB, "DC"),
        player(2, "Other Home Centre Back", HOME_CLUB, "DC"),
        player(10, "Progressive Target", OTHER_CLUB, "DC"),
        player(11, "Average Target", OTHER_CLUB, "DC"),
        player(12, "Target Without Stats", OTHER_CLUB, "DC"),
        player(13, "Peer Centre Back", OTHER_CLUB, "DC"),
        player(14, "Peer At Sixteen", OTHER_CLUB, "DM"),
        player(15, "Striker At Fifteen", OTHER_CLUB, "ST"),
    ]);
    for (uid, score) in [(1, 50), (2, 50), (10, 70), (11, 70), (12, 70)] {
        fixture.set_slot_score(uid, "left_centre_back", score);
    }
    fixture.set_player_column(14, "positions_json", r#"{"DM": 20, "DC": 16}"#);
    fixture.set_player_column(15, "positions_json", r#"{"ST": 20, "DC": 15}"#);
    for (uid, progressive_passes) in [(10, 40.0), (11, 20.0), (13, 10.0), (14, 5.0), (15, 100.0)] {
        fixture.set_season_stats(uid, 50.0, &[]);
        fixture.set_statistics(uid, &[("progressive_passes", progressive_passes)]);
    }
    fixture.set_recruitment_filters(
        "left_centre_back",
        &[
            ("moneyball.progressive_passes", "pct_gte", json!(60)),
            ("moneyball.minutes", "gte", json!(900)),
        ],
    );

    let signings = signings(
        &fixture,
        SigningsRequest {
            lane_id: Some("left_centre_back"),
            ..GENERAL
        },
    );

    // The peers are 5, 10, 20, and 40: the striker's familiarity of 15 leaves him out, and
    // the 16 of the defensive midfielder brings him in. 20 then ranks at the 67th percentile;
    // with the striker, or without the midfielder, it would rank at the 50th. The target
    // without season statistics cannot be ranked.
    let mut recommended = signings
        .recommendations
        .iter()
        .map(|recommendation| recommendation.uid)
        .collect::<Vec<_>>();
    recommended.sort_unstable();
    assert_eq!(recommended, [10, 11]);
    assert_eq!(signings.filtered_out, 1);
}

#[test]
fn moneyball_mode_applies_a_role_score_filter() {
    let fixture = open_with_players(&[
        player(1, "Home Centre Back", HOME_CLUB, "DC"),
        player(2, "Other Home Centre Back", HOME_CLUB, "DC"),
        player(10, "Strong Target", OTHER_CLUB, "DC"),
        player(11, "Weak Target", OTHER_CLUB, "DC"),
    ]);
    for (uid, level) in [(1, 40.0), (2, 40.0), (10, 80.0), (11, 80.0)] {
        fixture.set_season_stats(uid, level, &[]);
    }
    fixture.set_slot_score(10, "left_centre_back", 70);
    fixture.set_slot_score(11, "left_centre_back", 50);
    let centre_back = default_tactic().lanes[2].ip_role_id.clone();
    fixture.set_recruitment_filters(
        "left_centre_back",
        &[(format!("role.{centre_back}").as_str(), "gte", json!(60))],
    );

    let signings = signings(
        &fixture,
        SigningsRequest {
            lane_id: Some("left_centre_back"),
            ..MONEYBALL
        },
    );

    assert_eq!(
        signings
            .recommendations
            .iter()
            .map(|recommendation| recommendation.uid)
            .collect::<Vec<_>>(),
        [10]
    );
    assert_eq!(signings.filtered_out, 1);
}

fn uids(recommendations: &[Recommendation]) -> Vec<i64> {
    let mut uids = recommendations
        .iter()
        .map(|recommendation| recommendation.uid)
        .collect::<Vec<_>>();
    uids.sort_unstable();
    uids
}

#[test]
fn season_stats_hold_back_short_samples_and_switch_off_with_their_filters() {
    let fixture = open_with_players(&[
        player(1, "Home Centre Back", HOME_CLUB, "DC"),
        player(2, "Other Home Centre Back", HOME_CLUB, "DC"),
        player(10, "Regular Target", OTHER_CLUB, "DC"),
        player(11, "Short Sample Target", OTHER_CLUB, "DC"),
        player(12, "Target Without Stats", OTHER_CLUB, "DC"),
        player(13, "Peer Centre Back", OTHER_CLUB, "DC"),
    ]);
    for (uid, score) in [(1, 50), (2, 50), (10, 70), (11, 70), (12, 70)] {
        fixture.set_slot_score(uid, "left_centre_back", score);
    }
    for (uid, minutes, progressive_passes) in [(10, 600, 40.0), (11, 300, 100.0), (13, 900, 10.0)] {
        fixture.set_season_stats(uid, 50.0, &[]);
        fixture.set_season_minutes(uid, minutes);
        fixture.set_statistics(uid, &[("progressive_passes", progressive_passes)]);
    }
    fixture.set_recruitment_filters(
        "left_centre_back",
        &[("moneyball.progressive_passes", "pct_gte", json!(60))],
    );
    let left_centre_back = SigningsRequest {
        lane_id: Some("left_centre_back"),
        ..GENERAL
    };

    // The peers are 10 and 40: the 300-minute sample is too short to count, so 40 ranks at
    // the 100th percentile, not the 50th. Players under 450 minutes cannot be recommended.
    let applied = signings(&fixture, left_centre_back);
    assert!(applied.season_stats_available);
    assert!(applied.season_stats_applied);
    assert_eq!(uids(&applied.recommendations), [10]);
    assert_eq!(
        (applied.filtered_out, applied.under_minimum_minutes),
        (2, 2)
    );
    // The minimum holds for every slot, filtered or not.
    let all_slots = signings(&fixture, GENERAL);
    assert_eq!(uids(&all_slots.recommendations), [10]);
    assert_eq!(all_slots.under_minimum_minutes, 2);

    let off = signings(
        &fixture,
        SigningsRequest {
            season_stats: false,
            ..left_centre_back
        },
    );
    assert!(!off.season_stats_applied);
    assert_eq!(uids(&off.recommendations), [10, 11, 12]);
    assert_eq!((off.filtered_out, off.under_minimum_minutes), (0, 0));
}

#[test]
fn moneyball_mode_always_applies_season_stats() {
    let fixture = open_with_players(&[
        player(1, "Home Centre Back", HOME_CLUB, "DC"),
        player(2, "Other Home Centre Back", HOME_CLUB, "DC"),
        player(10, "Regular Target", OTHER_CLUB, "DC"),
        player(11, "Short Sample Target", OTHER_CLUB, "DC"),
    ]);
    for (uid, level) in [(1, 40.0), (2, 40.0), (10, 80.0), (11, 90.0)] {
        fixture.set_season_stats(uid, level, &[]);
    }
    fixture.set_season_minutes(11, 300);

    let signings = signings(
        &fixture,
        SigningsRequest {
            lane_id: Some("left_centre_back"),
            season_stats: false,
            ..MONEYBALL
        },
    );

    assert!(signings.season_stats_applied);
    assert_eq!(uids(&signings.recommendations), [10]);
    assert_eq!(signings.under_minimum_minutes, 1);
}

#[test]
fn without_season_stats_the_stat_filters_are_skipped_and_the_rest_apply() {
    let fixture = open_with_players(&[
        player(1, "Home Centre Back", HOME_CLUB, "DC"),
        player(2, "Other Home Centre Back", HOME_CLUB, "DC"),
        player(10, "Tall Target", OTHER_CLUB, "DC"),
        player(11, "Short Target", OTHER_CLUB, "DC"),
    ]);
    for (uid, score) in [(1, 50), (2, 50), (10, 70), (11, 70)] {
        fixture.set_slot_score(uid, "left_centre_back", score);
    }
    fixture.set_player_column(10, "height_cm", 190);
    fixture.set_player_column(11, "height_cm", 175);
    fixture.set_recruitment_filters(
        "left_centre_back",
        &[
            ("height", "gte", json!(183)),
            ("moneyball.progressive_passes", "pct_gte", json!(60)),
        ],
    );

    let signings = signings(
        &fixture,
        SigningsRequest {
            lane_id: Some("left_centre_back"),
            ..GENERAL
        },
    );

    assert!(!signings.season_stats_available);
    assert!(!signings.season_stats_applied);
    assert_eq!(uids(&signings.recommendations), [10]);
    assert_eq!(
        (signings.filtered_out, signings.under_minimum_minutes),
        (1, 0)
    );
}
