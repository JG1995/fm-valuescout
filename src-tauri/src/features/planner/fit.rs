use std::collections::BTreeMap;

use super::tactic::{base_position, TacticLane};
use crate::features::scoring::combine::combine_role_scores;

pub(super) fn phase_fit_score(
    score: Option<u8>,
    player_foot: &str,
    positions: &BTreeMap<String, Option<i64>>,
    position: &str,
    preferred_foot: &str,
    foot_preference: &str,
) -> Option<u8> {
    let score = score?;
    let familiarity = suitable_familiarity(positions, position)?;
    let foot_penalty = foot_penalty(player_foot, preferred_foot, foot_preference)?;
    Some(adjust_score(
        score,
        foot_penalty + familiarity_penalty(familiarity),
    ))
}

pub(super) fn lane_fit_score(
    score: Option<u8>,
    player_foot: &str,
    positions: &BTreeMap<String, Option<i64>>,
    lane: &TacticLane,
) -> Option<u8> {
    let ip_familiarity = suitable_familiarity(positions, &lane.ip_position)?;
    let oop_familiarity = suitable_familiarity(positions, &lane.oop_position)?;
    let score = score?;
    let foot_penalty = foot_penalty(player_foot, &lane.preferred_foot, &lane.foot_preference)?;
    Some(adjust_score(
        score,
        foot_penalty + familiarity_penalty(ip_familiarity) + familiarity_penalty(oop_familiarity),
    ))
}

pub(crate) fn tactic_adjusted_score(
    ip_score: Option<u8>,
    oop_score: Option<u8>,
    ip_weight: f64,
    player_foot: &str,
    positions: &BTreeMap<String, Option<i64>>,
    lane: &TacticLane,
) -> Option<u8> {
    let blended = combine_role_scores(ip_score, oop_score, ip_weight)?;
    lane_fit_score(Some(blended), player_foot, positions, lane)
}

pub(super) fn foot_matches(player_foot: &str, preferred_foot: &str) -> bool {
    match preferred_foot {
        "any" => true,
        "left" => matches!(player_foot, "left" | "either"),
        "right" => matches!(player_foot, "right" | "either"),
        "both" => player_foot == "either",
        _ => false,
    }
}

fn suitable_familiarity(positions: &BTreeMap<String, Option<i64>>, position: &str) -> Option<i64> {
    positions
        .get(base_position(position))
        .copied()
        .flatten()
        .filter(|familiarity| *familiarity >= 12)
}

fn foot_penalty(player_foot: &str, preferred_foot: &str, foot_preference: &str) -> Option<u8> {
    if foot_matches(player_foot, preferred_foot) {
        Some(0)
    } else if foot_preference == "strict" {
        None
    } else {
        Some(5)
    }
}

fn familiarity_penalty(familiarity: i64) -> u8 {
    u8::from(familiarity < 16) * 5
}

fn adjust_score(score: u8, penalty: u8) -> u8 {
    score.saturating_sub(penalty)
}
