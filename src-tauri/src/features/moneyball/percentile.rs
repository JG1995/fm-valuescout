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
    let columns = MONEYBALL_STATISTIC_KEYS
        .iter()
        .map(|key| {
            let values = statistics_by_player
                .values()
                .map(|statistics| statistics.get(*key).copied().flatten())
                .collect::<Vec<_>>();
            rank_column(&values, key)
        })
        .collect::<Vec<_>>();

    statistics_by_player
        .keys()
        .enumerate()
        .map(|(position, player_uid)| {
            let percentiles = MONEYBALL_STATISTIC_KEYS
                .iter()
                .zip(&columns)
                .map(|(key, column)| ((*key).to_string(), column[position]))
                .collect();
            (*player_uid, percentiles)
        })
        .collect()
}

/// Ranks one metric's values against each other, returning their percentiles in the same
/// order. Absent and non-finite values are not ranked and score `None`.
pub(crate) fn rank_column(values: &[Option<f64>], key: &str) -> Vec<Option<u8>> {
    let mut population = values
        .iter()
        .flatten()
        .copied()
        .filter(|value| value.is_finite())
        .collect::<Vec<_>>();
    population.sort_by(f64::total_cmp);

    values
        .iter()
        .map(|value| {
            value
                .filter(|value| value.is_finite())
                .and_then(|value| percentile_score(&population, value, key))
        })
        .collect()
}

/// One player's percentiles against `cohort`, which includes that player. Both hold values
/// in `MONEYBALL_STATISTIC_KEYS` order. The result equals that player's entry from
/// `calculate_percentiles`, without ranking every other cohort member.
pub(crate) fn subject_percentiles<'a>(
    subject: &[Option<f64>],
    cohort: impl IntoIterator<Item = &'a Vec<Option<f64>>>,
) -> MoneyballPercentiles {
    // Per metric: values below the subject's, values present, and their range.
    let mut tallies = vec![(0_usize, 0_usize, f64::INFINITY, f64::NEG_INFINITY); subject.len()];
    for peer in cohort {
        for ((tally, value), subject_value) in tallies.iter_mut().zip(peer).zip(subject) {
            let Some(value) = value.filter(|value| value.is_finite()) else {
                continue;
            };
            if subject_value.is_some_and(|subject_value| value < subject_value) {
                tally.0 += 1;
            }
            tally.1 += 1;
            tally.2 = tally.2.min(value);
            tally.3 = tally.3.max(value);
        }
    }

    MONEYBALL_STATISTIC_KEYS
        .iter()
        .zip(subject)
        .zip(tallies)
        .map(|((key, value), (below, present, min, max))| {
            let score = value
                .filter(|value| value.is_finite())
                .and_then(|_| ranked_score(below, present, min == max, key));
            ((*key).to_string(), score)
        })
        .collect()
}

fn percentile_score(population: &[f64], value: f64, key: &str) -> Option<u8> {
    ranked_score(
        population.partition_point(|candidate| *candidate < value),
        population.len(),
        population.first() == population.last(),
        key,
    )
}

/// Scores a value that `below` of the `population` present values are lower than.
/// `uniform` means every present value is equal.
fn ranked_score(below: usize, population: usize, uniform: bool, key: &str) -> Option<u8> {
    if population == 0 {
        return None;
    }
    if population == 1 || uniform {
        return Some(50);
    }

    let score = ((below as f64 / (population - 1) as f64) * 100.0)
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

    use super::{calculate_percentiles, MoneyballNumericStatistics, MoneyballPercentiles};
    use crate::features::moneyball::{
        catalog::LOWER_IS_BETTER_STATISTIC_KEYS, MONEYBALL_STATISTIC_KEYS,
    };

    // The pre-column implementation, copied verbatim as the equivalence oracle.
    fn previous_calculate_percentiles(
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
                let score = previous_percentile_score(&population, value, key);
                scores
                    .get_mut(player_uid)
                    .expect("scores are initialized from statistic player IDs")
                    .insert((*key).to_string(), score);
            }
        }

        scores
    }

    fn previous_percentile_score(population: &[f64], value: f64, key: &str) -> Option<u8> {
        previous_ranked_score(
            population.partition_point(|candidate| *candidate < value),
            population.len(),
            population.first() == population.last(),
            key,
        )
    }

    fn previous_ranked_score(
        below: usize,
        population: usize,
        uniform: bool,
        key: &str,
    ) -> Option<u8> {
        if population == 0 {
            return None;
        }
        if population == 1 || uniform {
            return Some(50);
        }

        let score = ((below as f64 / (population - 1) as f64) * 100.0)
            .round()
            .clamp(0.0, 100.0) as u8;
        Some(if LOWER_IS_BETTER_STATISTIC_KEYS.contains(&key) {
            100 - score
        } else {
            score
        })
    }

    /// SplitMix64, so the randomized comparison is reproducible without a dependency.
    struct Generator(u64);

    impl Generator {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }

        fn below(&mut self, bound: u64) -> u64 {
            self.next() % bound
        }
    }

    #[test]
    fn column_ranking_matches_the_previous_algorithm_on_random_cohorts() {
        const SPECIAL: [f64; 6] = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.0, 0.0, 1e300];
        let mut generator = Generator(0x5EED_2026);

        for cohort in 0..300 {
            let players = generator.below(64) as usize;
            // Per metric: varied values with ties, nulls, missing keys, and non-finite values;
            // a uniform population; one reported value; or no values.
            let shapes = MONEYBALL_STATISTIC_KEYS
                .iter()
                .map(|_| (generator.below(4), generator.below(6) as f64 - 1.5))
                .collect::<Vec<_>>();
            let lone_player = generator.below(players.max(1) as u64) as usize;

            let mut statistics_by_player = BTreeMap::new();
            for position in 0..players {
                let mut statistics = MoneyballNumericStatistics::new();
                for (key, (shape, shared)) in MONEYBALL_STATISTIC_KEYS.iter().zip(&shapes) {
                    let value = match shape {
                        0 => match generator.below(10) {
                            0 => continue,
                            1 => None,
                            2 => Some(SPECIAL[generator.below(6) as usize]),
                            3 => Some(generator.next() as f64 / 1e15),
                            _ => Some(generator.below(7) as f64 / 2.0),
                        },
                        1 => (generator.below(5) != 0).then_some(*shared),
                        2 => (position == lone_player).then_some(*shared),
                        _ => None,
                    };
                    statistics.insert((*key).to_string(), value);
                }
                let uid = generator.next() as i64;
                statistics_by_player.insert(uid, statistics);
            }

            assert_eq!(
                calculate_percentiles(&statistics_by_player),
                previous_calculate_percentiles(&statistics_by_player),
                "cohort {cohort}"
            );
        }
    }

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

    #[test]
    fn ranks_one_player_exactly_as_the_whole_cohort_ranking_does() {
        use crate::features::moneyball::catalog::LOWER_IS_BETTER_STATISTIC_KEYS;
        use crate::features::moneyball::MONEYBALL_STATISTIC_KEYS;

        // Metrics cycle through varied values with ties and nulls (every lower-is-better
        // metric included), one shared value, a single reported value, and no values.
        let value = |uid: usize, index: usize| -> Option<f64> {
            let key = MONEYBALL_STATISTIC_KEYS[index];
            if LOWER_IS_BETTER_STATISTIC_KEYS.contains(&key) || index % 4 == 0 {
                match (uid * 7 + index * 3) % 11 {
                    0 => None,
                    rest => Some((rest % 5) as f64 + 0.5),
                }
            } else if index % 4 == 1 {
                Some(3.0)
            } else if index % 4 == 2 {
                (uid == 5).then_some(1.25)
            } else {
                None
            }
        };
        let rows = (0..40)
            .map(|uid| {
                (0..MONEYBALL_STATISTIC_KEYS.len())
                    .map(|index| value(uid, index))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let keyed = rows
            .iter()
            .enumerate()
            .map(|(uid, row)| {
                (
                    uid as i64,
                    MONEYBALL_STATISTIC_KEYS
                        .iter()
                        .zip(row)
                        .map(|(key, value)| ((*key).to_string(), *value))
                        .collect(),
                )
            })
            .collect::<BTreeMap<_, _>>();

        let expected = calculate_percentiles(&keyed);

        for (uid, row) in rows.iter().enumerate() {
            assert_eq!(
                super::subject_percentiles(row, &rows),
                expected[&(uid as i64)],
                "player {uid}"
            );
        }
    }
}
