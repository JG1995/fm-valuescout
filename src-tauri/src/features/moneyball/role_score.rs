use std::collections::BTreeMap;

use super::catalog::is_save_unavailable_statistic_key;
use super::role_catalog::{validate_role_definition, RoleDefinition};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RoleScore {
    pub(crate) score: u8,
    pub(crate) contributions: Vec<RoleScoreContribution>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RoleScoreContribution {
    pub(crate) key: String,
    pub(crate) weight: f64,
    pub(crate) lower_is_better: bool,
    pub(crate) percentile: Option<u8>,
    pub(crate) weighted_contribution: Option<f64>,
    pub(crate) available_from_source: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RoleScoreDetails {
    pub(crate) score: Option<u8>,
    pub(crate) contributions: Vec<RoleScoreContribution>,
}

/// Explains a role score as the weighted mean of its metric percentiles.
/// Metrics that save files cannot provide are skipped, and the remaining
/// weights are rescaled to sum to one.
pub(crate) fn explain_role(
    definition: &RoleDefinition,
    percentiles: &BTreeMap<String, Option<u8>>,
) -> Option<RoleScoreDetails> {
    validate_role_definition(definition).ok()?;
    let available_weight = definition
        .metrics
        .iter()
        .filter(|metric| !is_save_unavailable_statistic_key(&metric.key))
        .map(|metric| metric.weight)
        .sum::<f64>();
    let mut weighted_sum = 0.0;
    let mut complete = available_weight > 0.0;
    let mut contributions = Vec::with_capacity(definition.metrics.len());

    for metric in &definition.metrics {
        let available_from_source = !is_save_unavailable_statistic_key(&metric.key);
        let percentile = if available_from_source {
            percentiles
                .get(&metric.key)
                .and_then(|value| *value)
                .map(|value| value.min(100))
        } else {
            None
        };
        let weighted_contribution = percentile.map(|value| {
            let contribution = f64::from(value) * (metric.weight / available_weight);
            weighted_sum += contribution;
            contribution
        });
        complete &= !available_from_source || weighted_contribution.is_some();
        contributions.push(RoleScoreContribution {
            key: metric.key.clone(),
            weight: metric.weight,
            lower_is_better: metric.inverted,
            percentile,
            weighted_contribution,
            available_from_source,
        });
    }

    Some(RoleScoreDetails {
        score: complete.then(|| weighted_sum.round().clamp(0.0, 100.0) as u8),
        contributions,
    })
}

pub(crate) fn score_role(
    definition: &RoleDefinition,
    percentiles: &BTreeMap<String, Option<u8>>,
) -> Option<RoleScore> {
    let details = explain_role(definition, percentiles)?;
    let score = details.score?;
    Some(RoleScore {
        score,
        contributions: details.contributions,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{explain_role, score_role};
    use crate::features::moneyball::percentile::calculate_percentiles;
    use crate::features::moneyball::role_catalog::{RoleDefinition, RoleMetric, RolePhase};

    fn definition(metrics: Vec<RoleMetric>) -> RoleDefinition {
        RoleDefinition {
            id: "test_role_ip".to_owned(),
            display_name: "Test Role".to_owned(),
            phase: RolePhase::InPossession,
            position_family: "test".to_owned(),
            position_tags: vec!["MC".to_owned()],
            attribute_role_id: None,
            metrics,
        }
    }

    fn metric(key: &str, weight: f64, inverted: bool) -> RoleMetric {
        RoleMetric {
            source_label: None,
            key: key.to_owned(),
            weight,
            inverted,
        }
    }

    #[test]
    fn computes_weighted_score_and_metric_explanations() {
        let definition = definition(vec![
            metric("goals", 0.25, false),
            metric("assists", 0.75, false),
        ]);
        let percentiles = BTreeMap::from([
            ("goals".to_owned(), Some(40)),
            ("assists".to_owned(), Some(80)),
        ]);

        let result = score_role(&definition, &percentiles).expect("score should be calculable");

        assert_eq!(result.score, 70);
        assert_eq!(result.contributions[0].percentile, Some(40));
        assert_eq!(result.contributions[0].weighted_contribution, Some(10.0));
        assert_eq!(result.contributions[1].weighted_contribution, Some(60.0));
        assert!(result
            .contributions
            .iter()
            .all(|contribution| contribution.available_from_source));
    }

    #[test]
    fn rounds_the_final_weighted_mean() {
        let definition = definition(vec![
            metric("goals", 0.5, false),
            metric("assists", 0.5, false),
        ]);
        let percentiles = BTreeMap::from([
            ("goals".to_owned(), Some(80)),
            ("assists".to_owned(), Some(81)),
        ]);

        assert_eq!(score_role(&definition, &percentiles).unwrap().score, 81);
    }

    #[test]
    fn preserves_a_valid_zero_percentile() {
        let definition = definition(vec![metric("goals", 1.0, false)]);
        let percentiles = BTreeMap::from([("goals".to_owned(), Some(0))]);

        assert_eq!(score_role(&definition, &percentiles).unwrap().score, 0);
    }

    #[test]
    fn returns_none_for_missing_or_null_input() {
        let definition = definition(vec![metric("goals", 1.0, false)]);

        assert!(score_role(&definition, &BTreeMap::new()).is_none());
        assert!(score_role(&definition, &BTreeMap::from([("goals".to_owned(), None)])).is_none());
    }

    #[test]
    fn does_not_invert_an_already_inverted_percentile() {
        let definition = definition(vec![metric("minutes_per_goal", 1.0, true)]);
        let percentiles = BTreeMap::from([("minutes_per_goal".to_owned(), Some(20))]);

        let result = score_role(&definition, &percentiles).expect("score should be calculable");

        assert_eq!(result.score, 20);
        assert_eq!(result.contributions[0].percentile, Some(20));
        assert!(result.contributions[0].lower_is_better);
    }

    #[test]
    fn rejects_a_zero_weight_total() {
        let definition = definition(vec![metric("goals", 0.0, false)]);
        let percentiles = BTreeMap::from([("goals".to_owned(), Some(50))]);

        assert!(score_role(&definition, &percentiles).is_none());
    }

    #[test]
    fn keeps_finite_large_weights_finite_by_normalizing_before_multiplication() {
        let definition = definition(vec![metric("goals", 1e307, false)]);
        let percentiles = BTreeMap::from([("goals".to_owned(), Some(50))]);

        let result = score_role(&definition, &percentiles).expect("score should be calculable");

        assert_eq!(result.score, 50);
        assert_eq!(result.contributions[0].weighted_contribution, Some(50.0));
        assert!(result.contributions[0]
            .weighted_contribution
            .expect("finite contribution")
            .is_finite());
    }

    #[test]
    fn composes_lower_raw_values_into_higher_role_contributions_once() {
        let definition = definition(vec![metric("minutes_per_goal", 1.0, true)]);
        let raw_values = BTreeMap::from([
            (
                1,
                BTreeMap::from([("minutes_per_goal".to_owned(), Some(30.0))]),
            ),
            (
                2,
                BTreeMap::from([("minutes_per_goal".to_owned(), Some(60.0))]),
            ),
        ]);
        let percentiles = calculate_percentiles(&raw_values);

        let better = score_role(&definition, &percentiles[&1]).expect("score should be calculable");
        let worse = score_role(&definition, &percentiles[&2]).expect("score should be calculable");

        assert_eq!(better.score, 100);
        assert_eq!(worse.score, 0);
        assert_eq!(better.contributions[0].percentile, Some(100));
        assert_eq!(worse.contributions[0].percentile, Some(0));
    }

    #[test]
    fn explanation_keeps_available_contributions_when_one_metric_is_null() {
        let definition = definition(vec![
            metric("goals", 0.5, false),
            metric("assists", 0.5, false),
        ]);
        let percentiles =
            BTreeMap::from([("goals".to_owned(), Some(80)), ("assists".to_owned(), None)]);

        let result = explain_role(&definition, &percentiles).expect("valid definition");

        assert_eq!(result.score, None);
        assert_eq!(result.contributions[0].percentile, Some(80));
        assert_eq!(result.contributions[0].weighted_contribution, Some(40.0));
        assert_eq!(result.contributions[1].percentile, None);
        assert_eq!(result.contributions[1].weighted_contribution, None);
    }

    fn pressing_role() -> RoleDefinition {
        definition(vec![
            metric("pressures_completed_per_90", 0.35, false),
            metric("high_intensity_sprints_per_90", 0.25, false),
            metric("possession_won_per_90", 0.2, false),
            metric("tackles_completed_per_90", 0.15, false),
            metric("interceptions_per_90", 0.05, false),
        ])
    }

    fn pressing_percentiles() -> BTreeMap<String, Option<u8>> {
        BTreeMap::from([
            ("pressures_completed_per_90".to_owned(), None),
            ("high_intensity_sprints_per_90".to_owned(), Some(90)),
            ("possession_won_per_90".to_owned(), Some(60)),
            ("tackles_completed_per_90".to_owned(), Some(40)),
            ("interceptions_per_90".to_owned(), Some(20)),
        ])
    }

    #[test]
    fn scores_a_role_without_unavailable_metrics_by_rescaling_remaining_weights() {
        let result = score_role(&pressing_role(), &pressing_percentiles())
            .expect("remaining metrics should score");

        // (0.25 * 90 + 0.2 * 60 + 0.15 * 40 + 0.05 * 20) / 0.65 = 63.85
        assert_eq!(result.score, 64);
        let pressures = &result.contributions[0];
        assert!(!pressures.available_from_source);
        assert_eq!(pressures.weight, 0.35);
        assert_eq!(pressures.percentile, None);
        assert_eq!(pressures.weighted_contribution, None);
        let sprints = &result.contributions[1];
        assert!(sprints.available_from_source);
        assert_eq!(sprints.weight, 0.25);
        let sprint_contribution = sprints.weighted_contribution.expect("sprint contribution");
        assert!((sprint_contribution - 22.5 / 0.65).abs() < 1e-9);
        let total = result
            .contributions
            .iter()
            .filter_map(|contribution| contribution.weighted_contribution)
            .sum::<f64>();
        assert!((total - 41.5 / 0.65).abs() < 1e-9);
    }

    #[test]
    fn ignores_a_stored_percentile_for_an_unavailable_metric() {
        let mut percentiles = pressing_percentiles();
        percentiles.insert("pressures_completed_per_90".to_owned(), Some(100));

        let result = score_role(&pressing_role(), &percentiles).expect("score");

        assert_eq!(result.score, 64);
        assert_eq!(result.contributions[0].percentile, None);
    }

    #[test]
    fn a_missing_available_percentile_still_withholds_the_score() {
        let mut percentiles = pressing_percentiles();
        percentiles.insert("tackles_completed_per_90".to_owned(), None);

        assert!(score_role(&pressing_role(), &percentiles).is_none());
        let details = explain_role(&pressing_role(), &percentiles).expect("valid definition");
        assert_eq!(details.score, None);
    }

    #[test]
    fn a_role_of_only_unavailable_metrics_has_no_score() {
        let definition = definition(vec![
            metric("pressures_attempted_per_90", 0.5, false),
            metric("pressures_completed_per_90", 0.5, false),
        ]);
        let percentiles = BTreeMap::from([
            ("pressures_attempted_per_90".to_owned(), Some(70)),
            ("pressures_completed_per_90".to_owned(), Some(70)),
        ]);

        assert!(score_role(&definition, &percentiles).is_none());
        let details = explain_role(&definition, &percentiles).expect("valid definition");
        assert_eq!(details.score, None);
        assert!(details
            .contributions
            .iter()
            .all(|contribution| !contribution.available_from_source));
    }
}
