//! Combined IP + OOP role score from per-phase scores and caller weights.
//!
//! Public API stays unused until ingest persistence in this feature.

#![allow(dead_code)]

/// Default IP weight for equal 50/50 blending when callers do not customize weights.
pub const DEFAULT_IP_WEIGHT: f64 = 0.5;

/// Blends in-possession and out-of-possession role scores with caller-supplied weights.
///
/// `oop_weight` is `1 - ip_weight`. Returns `None` when either score is missing or
/// `ip_weight` is outside `[0, 1]` (non-finite values are rejected).
pub fn combine_role_scores(ip: Option<u8>, oop: Option<u8>, ip_weight: f64) -> Option<u8> {
    if !(0.0..=1.0).contains(&ip_weight) {
        return None;
    }

    let ip_score = ip?;
    let oop_score = oop?;
    let blended = (f64::from(ip_score) * ip_weight) + (f64::from(oop_score) * (1.0 - ip_weight));

    Some(blended.round() as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_weights_average_ip_and_oop_scores() {
        assert_eq!(
            combine_role_scores(Some(80), Some(60), DEFAULT_IP_WEIGHT),
            Some(70)
        );
    }

    #[test]
    fn custom_ip_weight_biases_toward_ip_score() {
        assert_eq!(combine_role_scores(Some(100), Some(0), 0.75), Some(75));
    }

    #[test]
    fn custom_oop_weight_biases_toward_oop_score() {
        assert_eq!(combine_role_scores(Some(0), Some(100), 0.25), Some(75));
    }

    #[test]
    fn null_ip_score_returns_none() {
        assert_eq!(combine_role_scores(None, Some(60), DEFAULT_IP_WEIGHT), None);
    }

    #[test]
    fn null_oop_score_returns_none() {
        assert_eq!(combine_role_scores(Some(80), None, DEFAULT_IP_WEIGHT), None);
    }

    #[test]
    fn weight_below_zero_returns_none() {
        assert_eq!(combine_role_scores(Some(80), Some(60), -0.1), None);
    }

    #[test]
    fn weight_above_one_returns_none() {
        assert_eq!(combine_role_scores(Some(80), Some(60), 1.1), None);
    }

    #[test]
    fn non_finite_weight_returns_none() {
        assert_eq!(combine_role_scores(Some(80), Some(60), f64::NAN), None);
        assert_eq!(combine_role_scores(Some(80), Some(60), f64::INFINITY), None);
    }

    #[test]
    fn endpoint_weights_use_single_phase_score() {
        assert_eq!(combine_role_scores(Some(42), Some(99), 0.0), Some(99));
        assert_eq!(combine_role_scores(Some(42), Some(99), 1.0), Some(42));
    }

    #[test]
    fn blended_score_rounds_to_nearest_integer() {
        assert_eq!(
            combine_role_scores(Some(81), Some(60), DEFAULT_IP_WEIGHT),
            Some(71)
        );
    }
}
