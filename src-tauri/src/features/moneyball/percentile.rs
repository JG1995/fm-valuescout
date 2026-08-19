use std::collections::BTreeMap;

use super::{
    catalog::LOWER_IS_BETTER_STATISTIC_KEYS, MoneyballMetricValue, MoneyballStatistics,
    MONEYBALL_STATISTIC_KEYS,
};

pub(crate) type MoneyballNumericStatistics = BTreeMap<String, Option<f64>>;
pub(crate) type MoneyballPercentiles = BTreeMap<String, Option<u8>>;

pub(crate) fn numeric_statistics(statistics: &MoneyballStatistics) -> MoneyballNumericStatistics {
    MONEYBALL_STATISTIC_KEYS
        .iter()
        .map(|key| {
            let value = match statistics.get(*key).copied().flatten() {
                Some(MoneyballMetricValue::Count(value)) => Some(f64::from(value)),
                Some(MoneyballMetricValue::Decimal(value)) if value.is_finite() => Some(value),
                _ => None,
            };
            ((*key).to_string(), value)
        })
        .collect()
}

pub(crate) fn calculate_percentiles(
    statistics_by_player: &BTreeMap<i64, MoneyballNumericStatistics>,
) -> BTreeMap<i64, MoneyballPercentiles> {
    let mut scores: BTreeMap<i64, MoneyballPercentiles> = statistics_by_player
        .keys()
        .map(|player_uid| {
            (
                *player_uid,
                MONEYBALL_STATISTIC_KEYS
                    .iter()
                    .map(|key| ((*key).to_string(), None))
                    .collect(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    for key in MONEYBALL_STATISTIC_KEYS {
        let mut population = statistics_by_player
            .values()
            .filter_map(|statistics| {
                statistics
                    .get(key)
                    .copied()
                    .flatten()
                    .filter(|value| value.is_finite())
            })
            .collect::<Vec<_>>();
        population.sort_by(f64::total_cmp);

        for (player_uid, statistics) in statistics_by_player {
            let Some(value) = statistics
                .get(key)
                .copied()
                .flatten()
                .filter(|value| value.is_finite())
            else {
                continue;
            };
            let score = percentile_score(&population, value, key);
            scores
                .get_mut(player_uid)
                .expect("scores are initialized from statistic player IDs")
                .insert((*key).to_string(), score);
        }
    }

    scores
}

fn percentile_score(population: &[f64], value: f64, key: &str) -> Option<u8> {
    if population.is_empty() {
        return None;
    }
    if population.len() == 1 || population.first() == population.last() {
        return Some(50);
    }

    let rank = population.partition_point(|candidate| *candidate < value);
    let score = ((rank as f64 / (population.len() - 1) as f64) * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8;
    Some(if LOWER_IS_BETTER_STATISTIC_KEYS.contains(&key) {
        100 - score
    } else {
        score
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::calculate_percentiles;

    #[test]
    fn ranks_matched_values_with_lower_bounds_ties_and_inversion() {
        let values = BTreeMap::from([
            (
                1,
                BTreeMap::from([
                    ("goals".to_string(), Some(1.0)),
                    ("minutes_per_goal".to_string(), Some(90.0)),
                ]),
            ),
            (
                2,
                BTreeMap::from([
                    ("goals".to_string(), Some(2.0)),
                    ("minutes_per_goal".to_string(), Some(60.0)),
                ]),
            ),
            (
                3,
                BTreeMap::from([
                    ("goals".to_string(), Some(2.0)),
                    ("minutes_per_goal".to_string(), Some(30.0)),
                ]),
            ),
        ]);

        let scores = calculate_percentiles(&values);

        assert_eq!(scores[&1]["goals"], Some(0));
        assert_eq!(scores[&2]["goals"], Some(50));
        assert_eq!(scores[&3]["goals"], Some(50));
        assert_eq!(scores[&1]["minutes_per_goal"], Some(0));
        assert_eq!(scores[&2]["minutes_per_goal"], Some(50));
        assert_eq!(scores[&3]["minutes_per_goal"], Some(100));
    }

    #[test]
    fn preserves_nulls_and_uses_neutral_scores_for_non_varying_populations() {
        let values = BTreeMap::from([
            (
                1,
                BTreeMap::from([
                    ("goals".to_string(), Some(3.0)),
                    ("assists".to_string(), None),
                ]),
            ),
            (
                2,
                BTreeMap::from([
                    ("goals".to_string(), Some(3.0)),
                    ("assists".to_string(), Some(5.0)),
                ]),
            ),
        ]);

        let scores = calculate_percentiles(&values);

        assert_eq!(scores[&1]["goals"], Some(50));
        assert_eq!(scores[&2]["goals"], Some(50));
        assert_eq!(scores[&1]["assists"], None);
        assert_eq!(scores[&2]["assists"], Some(50));
    }

    #[test]
    fn converts_counts_and_decimals_without_replacing_nulls() {
        use crate::features::moneyball::{MoneyballMetricValue, MoneyballStatistics};

        let statistics = MoneyballStatistics::from([
            ("goals".to_string(), Some(MoneyballMetricValue::Count(3))),
            (
                "average_rating".to_string(),
                Some(MoneyballMetricValue::Decimal(7.25)),
            ),
            ("assists".to_string(), None),
        ]);

        let values = super::numeric_statistics(&statistics);

        assert_eq!(values["goals"], Some(3.0));
        assert_eq!(values["average_rating"], Some(7.25));
        assert_eq!(values["assists"], None);
        assert_eq!(values.len(), 138);
    }
}
