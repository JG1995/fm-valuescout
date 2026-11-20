use super::depth::{assign_player, get_depth, PlannerTeam};
use super::loans::{loan_benefit, loan_candidates};
use super::test_support::{
    add_picker_candidates, loan_in, loan_out, open_with_snapshot, set_development, set_player_age,
    set_right_winger, team_strings,
};

#[test]
fn a_loan_needs_growth_and_helps_most_near_the_senior_squad_and_far_from_minutes() {
    // Without growth to come, a loan develops nothing.
    assert_eq!(loan_benefit(0.0, Some(0.0), None), 0);
    // Full growth, level with the senior squad, and in no squad at all.
    assert_eq!(loan_benefit(12.0, Some(0.0), None), 100);
    assert_eq!(loan_benefit(30.0, None, Some(2)), 100);
    // Twenty points behind the senior squad and starting for a development team.
    assert_eq!(loan_benefit(12.0, Some(20.0), Some(0)), 25);
    // Half the growth, halfway to the senior squad, and second in line.
    assert_eq!(loan_benefit(6.0, Some(10.0), Some(1)), 31);
    // A player better than the weakest senior player is as close as one level with him.
    assert_eq!(loan_benefit(12.0, Some(-5.0), None), 100);
}

#[test]
fn loan_candidates_are_18_to_23_outside_the_senior_squad_with_most_to_gain_first() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    set_right_winger(&conn, save_id, 77, 80, 80);
    set_player_age(&conn, save_id, 77, Some(21));
    set_right_winger(&conn, save_id, 78, 70, 90);
    set_development(&conn, save_id, 78, 19, [15, 15, 15]);
    set_right_winger(&conn, save_id, 79, 60, 62);
    set_development(&conn, save_id, 79, 22, [10, 10, 10]);
    set_right_winger(&conn, save_id, 80, 60, 90);
    set_player_age(&conn, save_id, 80, Some(17));
    let depth = get_depth(&conn, save_id).expect("create planner depth");
    let string_id = |team| team_strings(&depth, team)[0].id;
    assign_player(
        &conn,
        save_id,
        string_id(PlannerTeam::Senior),
        "right_winger",
        77,
    )
    .expect("place the senior winger");
    assign_player(
        &conn,
        save_id,
        string_id(PlannerTeam::Reserves),
        "right_winger",
        78,
    )
    .expect("place the reserve winger");

    let candidates = loan_candidates(&conn, save_id).expect("rank loan candidates");

    // The senior winger and the 17-year-old are left out. At 19 with a drive of 15, uid 78
    // closes 95% of his gap: full growth, ten points behind the senior winger, and a starter
    // for the reserves. At 22, uid 79 gains one point and trails the senior winger by twenty.
    assert_eq!(
        candidates
            .iter()
            .map(|candidate| {
                (
                    candidate.player_uid,
                    candidate
                        .place
                        .as_ref()
                        .map(|place| (place.team, place.string_name.as_str())),
                    candidate.lane_id.as_str(),
                    (candidate.current_score, candidate.expected_score),
                    candidate.benefit,
                    candidate.recommended,
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                78,
                Some((PlannerTeam::Reserves, "1st string")),
                "right_winger",
                (70, 89),
                50,
                true
            ),
            (79, None, "right_winger", (60, 61), 4, false),
        ]
    );
}

#[test]
fn players_on_loan_either_way_are_no_loan_candidates() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    for uid in [78, 79, 80] {
        set_right_winger(&conn, save_id, uid, 60, 90);
        set_development(&conn, save_id, uid, 19, [15, 15, 15]);
    }
    get_depth(&conn, save_id).expect("create planner depth");
    let candidate_uids = |conn| {
        loan_candidates(conn, save_id)
            .expect("rank loan candidates")
            .iter()
            .map(|candidate| candidate.player_uid)
            .collect::<Vec<_>>()
    };
    assert_eq!(candidate_uids(&conn), [78, 79, 80]);

    loan_in(&conn, save_id, 78);
    loan_out(&conn, save_id, 79);

    assert_eq!(candidate_uids(&conn), [80]);
}
