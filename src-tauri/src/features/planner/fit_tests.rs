use std::collections::BTreeMap;

use super::fit::{foot_matches, lane_fit_score, phase_fit_score};
use super::tactic::default_tactic;

fn positions(entries: &[(&str, i64)]) -> BTreeMap<String, Option<i64>> {
    entries
        .iter()
        .map(|(position, familiarity)| ((*position).to_string(), Some(*familiarity)))
        .collect()
}

#[test]
fn phase_fit_uses_the_existing_familiarity_thresholds() {
    for (familiarity, expected) in [(11, None), (12, Some(45)), (15, Some(45)), (16, Some(50))] {
        assert_eq!(
            phase_fit_score(
                Some(50),
                "right",
                &positions(&[("MC", familiarity)]),
                "MC",
                "any",
                "preferred",
            ),
            expected,
            "familiarity {familiarity}",
        );
    }
}

#[test]
fn phase_fit_normalizes_qualified_positions_to_their_base_position() {
    assert_eq!(
        phase_fit_score(
            Some(50),
            "right",
            &positions(&[("MC", 15)]),
            "MCR",
            "any",
            "preferred",
        ),
        Some(45),
    );
}

#[test]
fn phase_fit_applies_a_soft_preferred_foot_penalty_once() {
    assert_eq!(
        phase_fit_score(
            Some(50),
            "left",
            &positions(&[("MC", 16)]),
            "MC",
            "right",
            "preferred",
        ),
        Some(45),
    );
}

#[test]
fn phase_fit_rejects_a_strict_preferred_foot_mismatch() {
    assert_eq!(
        phase_fit_score(
            Some(50),
            "left",
            &positions(&[("MC", 16)]),
            "MC",
            "right",
            "strict",
        ),
        None,
    );
}

#[test]
fn phase_fit_saturates_penalties_at_zero() {
    assert_eq!(
        phase_fit_score(
            Some(3),
            "left",
            &positions(&[("MC", 12)]),
            "MC",
            "right",
            "preferred",
        ),
        Some(0),
    );
}

#[test]
fn linked_lane_counts_familiarity_penalties_for_both_repeated_positions() {
    let lane = default_tactic().lanes[6].clone();

    assert_eq!(
        lane_fit_score(Some(50), "right", &positions(&[("MC", 15)]), &lane,),
        Some(40),
    );
}

#[test]
fn linked_lane_applies_a_soft_preferred_foot_penalty_once() {
    let mut lane = default_tactic().lanes[9].clone();
    lane.preferred_foot = "right".to_string();
    lane.foot_preference = "preferred".to_string();

    assert_eq!(
        lane_fit_score(
            Some(50),
            "left",
            &positions(&[("AMR", 16), ("MR", 16)]),
            &lane,
        ),
        Some(45),
    );
}

#[test]
fn linked_lane_requires_both_phase_positions_to_be_suitable() {
    let lane = default_tactic().lanes[9].clone();

    assert_eq!(
        lane_fit_score(
            Some(50),
            "right",
            &positions(&[("AMR", 18), ("MR", 11)]),
            &lane,
        ),
        None,
    );
}

#[test]
fn linked_lane_rejects_an_unsuitable_ip_position() {
    let lane = default_tactic().lanes[9].clone();

    assert_eq!(
        lane_fit_score(
            Some(50),
            "right",
            &positions(&[("AMR", 11), ("MR", 18)]),
            &lane,
        ),
        None,
    );
}

#[test]
fn foot_matching_preserves_two_footed_rules() {
    for (player_foot, preferred_foot, expected) in [
        ("left", "any", true),
        ("right", "any", true),
        ("either", "any", true),
        ("", "any", true),
        ("left", "left", true),
        ("right", "left", false),
        ("either", "left", true),
        ("", "left", false),
        ("left", "right", false),
        ("right", "right", true),
        ("either", "right", true),
        ("right", "both", false),
        ("left", "both", false),
        ("either", "both", true),
        ("", "both", false),
    ] {
        assert_eq!(
            foot_matches(player_foot, preferred_foot),
            expected,
            "{player_foot:?} against {preferred_foot:?}",
        );
    }
}
