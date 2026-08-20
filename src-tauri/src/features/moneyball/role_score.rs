use std::collections::BTreeMap;

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
    pub(crate) percentile: u8,
    pub(crate) weighted_contribution: f64,
}

pub(crate) fn score_role(
    definition: &RoleDefinition,
    percentiles: &BTreeMap<String, Option<u8>>,
) -> Option<RoleScore> {
    let total_weight = validate_role_definition(definition).ok()?;
    let mut weighted_sum = 0.0;
    let mut contributions = Vec::with_capacity(definition.metrics.len());

    for metric in &definition.metrics {
        let percentile = percentiles.get(&metric.key).and_then(|value| *value)?;
        let percentile = percentile.min(100);
        let weighted_contribution = f64::from(percentile) * (metric.weight / total_weight);
        weighted_sum += weighted_contribution;
        contributions.push(RoleScoreContribution {
            key: metric.key.clone(),
            weight: metric.weight,
            lower_is_better: metric.inverted,
            percentile,
            weighted_contribution,
        });
    }

    let score = weighted_sum.round().clamp(0.0, 100.0) as u8;
    Some(RoleScore {
        score,
        contributions,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::score_role;
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
        assert_eq!(result.contributions[0].percentile, 40);
        assert_eq!(result.contributions[0].weighted_contribution, 10.0);
        assert_eq!(result.contributions[1].weighted_contribution, 60.0);
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
        assert_eq!(result.contributions[0].percentile, 20);
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
        assert_eq!(result.contributions[0].weighted_contribution, 50.0);
        assert!(result.contributions[0].weighted_contribution.is_finite());
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
        assert_eq!(better.contributions[0].percentile, 100);
        assert_eq!(worse.contributions[0].percentile, 0);
    }
}
