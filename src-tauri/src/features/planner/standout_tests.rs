use super::standout::load_slot_rankings;
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

    let rankings = load_slot_rankings(&conn, current_snapshot_id(&conn, save_id), &tactic)
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
