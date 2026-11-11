use std::collections::BTreeMap;

use super::tactic::{base_position, TacticLane};
use crate::features::moneyball::league_strength::shifted;
use crate::features::moneyball::role_catalog::definition_for_attribute_role;
use crate::features::moneyball::role_score::{mean_percentile_score, score_role};
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

pub(crate) fn lane_fit_score(
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

/// A slot's Moneyball role score from a player's percentiles: the Moneyball definitions of
/// the slot's IP and OOP roles, blended by its IP weight and adjusted for fit. No score when
/// either role has no Moneyball definition at its position.
pub(crate) fn moneyball_roles_lane_score(
    percentiles: &BTreeMap<String, Option<u8>>,
    player_foot: &str,
    positions: &BTreeMap<String, Option<i64>>,
    lane: &TacticLane,
    league_shift: i16,
) -> Option<u8> {
    let role_score = |role_id: &str, position: &str| {
        definition_for_attribute_role(role_id, base_position(position)).map(|definition| {
            score_role(definition, percentiles).map(|role| shifted(role.score, league_shift))
        })
    };
    let ip_score = role_score(&lane.ip_role_id, &lane.ip_position)?;
    let oop_score = role_score(&lane.oop_role_id, &lane.oop_position)?;
    tactic_adjusted_score(
        ip_score,
        oop_score,
        lane.ip_weight,
        player_foot,
        positions,
        lane,
    )
}

/// A slot's custom Moneyball score: the equal-weight mean of its chosen metrics' percentiles,
/// moved by the player's league shift and adjusted for fit without the IP/OOP blend.
pub(crate) fn custom_metrics_lane_score(
    percentiles: &BTreeMap<String, Option<u8>>,
    player_foot: &str,
    positions: &BTreeMap<String, Option<i64>>,
    lane: &TacticLane,
    league_shift: i16,
) -> Option<u8> {
    lane_fit_score(
        mean_percentile_score(&lane.custom_metric_keys, percentiles)
            .map(|score| shifted(score, league_shift)),
        player_foot,
        positions,
        lane,
    )
}

/// How a player's positions suit a lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PositionFit {
    /// Accomplished in the lane's position in both phases.
    Suited,
    /// Playable, but marked down for a position he is less familiar with.
    Unfamiliar,
    /// Cannot play one of the lane's positions, so Optimize never places him there.
    Unsuited,
}

impl PositionFit {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Suited => "suited",
            Self::Unfamiliar => "unfamiliar",
            Self::Unsuited => "unsuited",
        }
    }
}

/// How a player's preferred foot suits a lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FootFit {
    Matches,
    /// Marked down for the wrong foot.
    Mismatch,
    /// The lane takes only its preferred foot, so Optimize never places him there.
    Excluded,
}

impl FootFit {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Matches => "matches",
            Self::Mismatch => "mismatch",
            Self::Excluded => "excluded",
        }
    }
}

pub(super) fn lane_position_fit(
    positions: &BTreeMap<String, Option<i64>>,
    lane: &TacticLane,
) -> PositionFit {
    match (
        suitable_familiarity(positions, &lane.ip_position),
        suitable_familiarity(positions, &lane.oop_position),
    ) {
        (Some(ip), Some(oop)) if familiarity_penalty(ip) + familiarity_penalty(oop) == 0 => {
            PositionFit::Suited
        }
        (Some(_), Some(_)) => PositionFit::Unfamiliar,
        _ => PositionFit::Unsuited,
    }
}

pub(super) fn lane_foot_fit(player_foot: &str, lane: &TacticLane) -> FootFit {
    match foot_penalty(player_foot, &lane.preferred_foot, &lane.foot_preference) {
        Some(0) => FootFit::Matches,
        Some(_) => FootFit::Mismatch,
        None => FootFit::Excluded,
    }
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
