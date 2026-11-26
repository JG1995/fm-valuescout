use std::sync::Arc;

use super::fit::PositionRule;
use super::standout::{load_slot_rankings, slot_rankings};
use super::tactic;
use super::test_support::{
    current_snapshot_id, open_with_snapshot, seed_players, set_lane_scores, set_player_positions,
};

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn a_score_ranks_against_the_players_who_can_play_the_slot() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    seed_players(&temp_dir, &mut conn, save_id, &[77, 201, 202, 203, 204]);
    for (player_uid, current, potential) in
        [(201, 50, 90), (202, 60, 60), (203, 60, 70), (204, 80, 80)]
    {
        set_player_positions(&conn, save_id, player_uid, r#"{"AMR": 18, "MR": 18}"#);
        set_lane_scores(
            &conn,
            save_id,
            player_uid,
            "right_winger",
            current,
            potential,
        );
    }
    // A striker with a winger's scores does not count among the right wingers.
    set_player_positions(&conn, save_id, 77, r#"{"ST": 18}"#);
    set_lane_scores(&conn, save_id, 77, "right_winger", 99, 99);
    set_lane_scores(&conn, save_id, 77, "centre_forward", 40, 40);
    let tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
    let lane_index = |lane_id: &str| {
        tactic
            .lanes
            .iter()
            .position(|lane| lane.lane_id == lane_id)
            .expect("tactic lane")
    };

    let rankings = load_slot_rankings(
        &conn,
        current_snapshot_id(&conn, save_id),
        &tactic,
        PositionRule::Familiarity,
    )
    .expect("rank the save");

    // Players below count in full and players level count half.
    let right_winger = rankings.lane(lane_index("right_winger"));
    assert_close(right_winger.current.rank(50), 0.125);
    assert_close(right_winger.current.rank(55), 0.25);
    assert_close(right_winger.current.rank(60), 0.5);
    assert_close(right_winger.current.rank(80), 0.875);
    assert_close(right_winger.potential.rank(80), 0.625);
    assert_close(right_winger.ip_current.rank(80), 0.875);
    assert_close(right_winger.oop_potential.rank(90), 0.875);
    // The only centre-forward ranks in the middle, as does any score where nobody plays.
    assert_close(
        rankings.lane(lane_index("centre_forward")).current.rank(40),
        0.5,
    );
    assert_close(
        rankings.lane(lane_index("goalkeeper")).current.rank(70),
        0.5,
    );
}

#[test]
fn readers_keep_the_rankings_until_role_scores_or_the_tactic_change() {
    let (temp_dir, db) = crate::db::open_with_golden_snapshot();
    let (save_id, snapshot_id, tactic) = {
        let conn = db.0.lock().expect("lock writer");
        let save_id =
            crate::features::snapshot::service::active_save_id(&conn).expect("active save");
        set_player_positions(&conn, save_id, 77, r#"{"AMR": 18, "MR": 18}"#);
        set_lane_scores(&conn, save_id, 77, "right_winger", 60, 60);
        (
            save_id,
            current_snapshot_id(&conn, save_id),
            tactic::get_tactic(&conn, save_id).expect("load tactic"),
        )
    };
    let right_winger = tactic
        .lanes
        .iter()
        .position(|lane| lane.lane_id == "right_winger")
        .expect("tactic lane");
    let rank = |tactic: &tactic::PlannerTactic, rule| {
        slot_rankings(&db.read().expect("reader"), snapshot_id, tactic, rule)
            .expect("rank the save")
    };
    let first = rank(&tactic, PositionRule::Familiarity);
    assert_close(first.lane(right_winger).current.rank(70), 1.0);

    // A write that leaves the role scores alone, here a save setting, advances the data
    // generation but keeps the rankings. So does a change that no app write makes: these
    // scores were set behind the code that counts score writes.
    super::development::set_max_age(&db.0.lock().expect("lock writer"), save_id, 21)
        .expect("change a setting");
    let outside = rusqlite::Connection::open(temp_dir.path().join(crate::db::APP_DB_FILE))
        .expect("open outside the app");
    set_lane_scores(&outside, save_id, 77, "right_winger", 80, 80);
    drop(outside);
    assert!(Arc::ptr_eq(
        &first,
        &rank(&tactic, PositionRule::Familiarity)
    ));
    // The other position rule and another tactic have rankings of their own.
    assert!(!Arc::ptr_eq(
        &first,
        &rank(&tactic, PositionRule::AnyPosition)
    ));
    let mut heavier = tactic.clone();
    heavier.lanes[right_winger].ip_weight = 0.9;
    assert!(!Arc::ptr_eq(
        &first,
        &rank(&heavier, PositionRule::Familiarity)
    ));

    // The app rewrites a player's role scores after a boost, and the next reader ranks again:
    // the rewrite replaces the 80 set above with his scores from his attributes.
    {
        let mut conn = db.0.lock().expect("lock writer");
        let tx = conn.transaction().expect("begin");
        crate::features::player_metrics::potential_scores::replace_player(&tx, snapshot_id, 77)
            .expect("rewrite role scores");
        tx.commit().expect("commit");
    }
    let ranked_again = rank(&tactic, PositionRule::Familiarity);
    assert!(!Arc::ptr_eq(&first, &ranked_again));
}
