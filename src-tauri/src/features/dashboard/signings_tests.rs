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
};
const MONEYBALL: SigningsRequest<'static> = SigningsRequest {
    mode: ScoreMode::Moneyball,
    ..GENERAL
};

fn signings(fixture: &Fixture, request: SigningsRequest<'_>) -> Signings {
    get_signings(&fixture.conn, fixture.save_id, request)
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
    fixture.add_shortlist("Targets", &[2, 10, 11, 12]);

    let signings = signings(&fixture, GENERAL);

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
    assert_eq!(signings.shortlisted_players, 3);
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
    fixture.add_shortlist("Targets", &[10]);

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
    fixture.add_shortlist("Targets", &[10]);

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
    fixture.add_shortlist("Targets", &[10, 11]);

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
    fixture.add_shortlist("Targets", &[10, 11, 12]);

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
    fixture.set_percentiles(1, 50, &[("goals_per_90", 40), ("xg_per_90", 20)]);
    fixture.set_percentiles(2, 60, &[]);
    fixture.set_percentiles(10, 50, &[("goals_per_90", 90), ("xg_per_90", 70)]);
    fixture.set_percentiles(11, 75, &[]);
    fixture.add_shortlist("Targets", &[10, 11, 12]);

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
    assert_eq!(
        incumbent(&signings, "centre_forward"),
        Some((1, 30, IncumbentSource::BestFit))
    );
    assert_eq!(
        incumbent(&signings, "goalkeeper"),
        Some((2, 60, IncumbentSource::BestFit))
    );
    assert_eq!(
        ranked(&signings.recommendations),
        [(10, "centre_forward", 80, 50), (11, "goalkeeper", 75, 15)]
    );
    assert_eq!(signings.recommendations[0].minutes, Some(900));
}

#[test]
fn moneyball_mode_marks_a_signing_down_for_playing_in_a_weaker_league() {
    let fixture = open_with_players(&[
        player(1, "Home Keeper", HOME_CLUB, "GK"),
        player(10, "Target Keeper", OTHER_CLUB, "GK"),
    ]);
    fixture.set_percentiles(1, 60, &[]);
    fixture.set_percentiles(10, 75, &[]);
    fixture.add_shortlist("Targets", &[10]);
    fixture.set_player_column(1, "division", "Top");
    fixture.set_player_column(10, "division", "Second");
    fixture.measure_leagues(&[("Top", 150.0), ("Second", 140.0)]);

    let signings = signings(&fixture, MONEYBALL);

    // The managed club's league is the reference, so its own keeper keeps his 60. The
    // target's 75 loses half of the 10 points between the leagues.
    assert_eq!(
        incumbent(&signings, "goalkeeper"),
        Some((1, 60, IncumbentSource::BestFit))
    );
    assert_eq!(
        ranked(&signings.recommendations),
        [(10, "goalkeeper", 70, 10)]
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
    fixture.add_shortlist("Targets", &[10]);

    let signings = signings(&fixture, MONEYBALL);

    assert!(!signings.scores_available);
    assert_eq!(incumbent(&signings, "goalkeeper"), None);
    assert!(signings.recommendations.is_empty());
}

#[test]
fn compares_one_shortlist_or_all_of_the_saves_and_never_another_saves() {
    let fixture = open_with_players(&[
        player(1, "Home Keeper", HOME_CLUB, "GK"),
        player(10, "First Target", OTHER_CLUB, "GK"),
        player(11, "Second Target", OTHER_CLUB, "GK"),
    ]);
    fixture.set_slot_score(1, "goalkeeper", 50);
    fixture.set_slot_score(10, "goalkeeper", 60);
    fixture.set_slot_score(11, "goalkeeper", 70);
    let first = fixture.add_shortlist("First", &[10]);
    fixture.add_shortlist("Second", &[11]);
    let other_save = snapshot::service::create_save(&fixture.conn, "Other save")
        .expect("create save")
        .id;
    fixture
        .conn
        .execute(
            "INSERT INTO player_shortlists (id, save_id, file_path, name)
             VALUES (900, ?1, NULL, 'Foreign')",
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
                shortlist_id,
                ..GENERAL
            },
        );
        signings
            .recommendations
            .iter()
            .map(|recommendation| recommendation.uid)
            .collect::<Vec<_>>()
    };

    assert_eq!(with_shortlist(None), [11, 10]);
    assert_eq!(with_shortlist(Some(first)), [10]);
    assert_eq!(with_shortlist(Some(900)), Vec::<i64>::new());
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
        get_signings(&fixture.conn, fixture.save_id, GENERAL),
        Ok(None)
    );
}
