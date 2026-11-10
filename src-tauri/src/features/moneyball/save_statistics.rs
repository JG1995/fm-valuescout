//! Derives the canonical 138-key Moneyball statistics from one player's save-file season
//! totals. Rates come from the summed counts, never from precomputed save rates, and a key
//! the save cannot source stays null.

use crate::features::save_stats::{SavePlayerSeasonStats, SeasonStat};

use super::persistence::CohortRow;
use super::{MoneyballMetricValue, MoneyballStatistics, MONEYBALL_STATISTIC_KEYS};

/// Canonical counts read directly from one save total.
const COUNTS: &[(&str, SeasonStat)] = &[
    ("goals", SeasonStat::Goals),
    ("shots", SeasonStat::Shots),
    ("shots_from_outside_the_box", SeasonStat::ShotsOutsideBox),
    ("shots_on_target", SeasonStat::ShotsOnTarget),
    ("penalties_taken", SeasonStat::PenaltiesTaken),
    ("penalties_scored", SeasonStat::PenaltiesScored),
    ("free_kicks", SeasonStat::FreeKickShots),
    ("assists", SeasonStat::Assists),
    // FM27 saves store no separate chances-created count.
    ("chances_created", SeasonStat::ClearCutChancesCreated),
    (
        "clear_cut_chances_created",
        SeasonStat::ClearCutChancesCreated,
    ),
    ("key_passes", SeasonStat::KeyPasses),
    ("open_play_key_passes", SeasonStat::OpenPlayKeyPasses),
    ("crosses_attempted", SeasonStat::CrossesAttempted),
    ("crosses_completed", SeasonStat::CrossesCompleted),
    (
        "open_play_crosses_attempted",
        SeasonStat::OpenPlayCrossesAttempted,
    ),
    (
        "open_play_crosses_completed",
        SeasonStat::OpenPlayCrossesCompleted,
    ),
    ("passes_attempted", SeasonStat::PassesAttempted),
    ("passes_completed", SeasonStat::PassesCompleted),
    ("progressive_passes", SeasonStat::ProgressivePasses),
    ("dribbles_made", SeasonStat::Dribbles),
    ("high_intensity_sprints", SeasonStat::HighIntensitySprints),
    ("possession_lost", SeasonStat::PossessionLost),
    ("tackles_attempted", SeasonStat::TacklesAttempted),
    ("tackles_completed", SeasonStat::TacklesCompleted),
    ("key_tackles", SeasonStat::KeyTackles),
    ("interceptions", SeasonStat::Interceptions),
    ("possession_won", SeasonStat::PossessionWon),
    ("pressures_attempted", SeasonStat::PressuresAttempted),
    ("pressures_completed", SeasonStat::PressuresCompleted),
    ("blocks", SeasonStat::Blocks),
    ("shots_blocked_defending", SeasonStat::ShotsBlocked),
    ("clearances", SeasonStat::Clearances),
    ("headers_attempted", SeasonStat::AerialChallengesAttempted),
    ("headers_won", SeasonStat::HeadersWon),
    ("key_headers", SeasonStat::KeyHeaders),
    ("clean_sheets", SeasonStat::CleanSheets),
    ("goals_conceded", SeasonStat::GoalsAllowed),
    ("saves_held", SeasonStat::SavesHeld),
    ("saves_parried", SeasonStat::SavesParried),
    ("saves_tipped", SeasonStat::SavesTipped),
    ("penalties_faced", SeasonStat::PenaltiesFaced),
    ("penalties_saved", SeasonStat::PenaltiesSaved),
    ("fouls_made", SeasonStat::FoulsMade),
    ("fouls_against", SeasonStat::FoulsAgainst),
    ("yellow_cards", SeasonStat::YellowCards),
    ("red_cards", SeasonStat::RedCards),
    ("offsides", SeasonStat::Offsides),
    (
        "mistakes_leading_to_goal",
        SeasonStat::MistakesLeadingToGoal,
    ),
    ("player_of_the_match", SeasonStat::PlayerOfTheMatch),
    ("games_won", SeasonStat::GamesWon),
    ("games_lost", SeasonStat::GamesLost),
    ("team_goals", SeasonStat::TeamGoals),
];

/// Canonical decimals read directly from one save total, in public units.
const DECIMALS: &[(&str, SeasonStat)] = &[
    ("xg", SeasonStat::ExpectedGoals),
    ("np-xg", SeasonStat::NonPenaltyExpectedGoals),
    ("xa", SeasonStat::ExpectedAssists),
    ("distance_covered", SeasonStat::DistanceKm),
    ("xgp", SeasonStat::ExpectedGoalsPrevented),
];

/// (ratio, numerator, denominator), as the CSV derivation defines them.
const RATIOS: &[(&str, &str, &str)] = &[
    (
        "penalties_scored_ratio",
        "penalties_scored",
        "penalties_taken",
    ),
    (
        "cross_completion_ratio",
        "crosses_completed",
        "crosses_attempted",
    ),
    (
        "open_play_cross_completion_ratio",
        "open_play_crosses_completed",
        "open_play_crosses_attempted",
    ),
    (
        "pass_completion_ratio",
        "passes_completed",
        "passes_attempted",
    ),
    (
        "tackle_completion_ratio",
        "tackles_completed",
        "tackles_attempted",
    ),
    (
        "pressure_success_ratio",
        "pressures_completed",
        "pressures_attempted",
    ),
    ("headers_won_ratio", "headers_won", "headers_attempted"),
    (
        "penalties_saved_ratio",
        "penalties_saved",
        "penalties_faced",
    ),
    ("xg_per_shot", "xg", "shots"),
];

const PER_90_SUFFIX: &str = "_per_90";

/// The player's cohort row, keyed by the game's Unique ID.
pub(crate) fn cohort_row(player: &SavePlayerSeasonStats) -> CohortRow {
    CohortRow {
        player_uid: i64::from(player.unique_id),
        starts: i64::from(player.starts),
        substitute_appearances: i64::from(player.substitute_appearances),
        minutes: i64::from(player.minutes),
        statistics: canonical_statistics(player),
    }
}

fn canonical_statistics(player: &SavePlayerSeasonStats) -> MoneyballStatistics {
    // Save totals are sums of unsigned stored counts.
    let count = |stat| player.total(stat).map(|total| total as u32);
    let saves = [
        SeasonStat::SavesHeld,
        SeasonStat::SavesParried,
        SeasonStat::SavesTipped,
    ]
    .into_iter()
    .map(count)
    .sum::<Option<u32>>();
    let headers_lost = count(SeasonStat::AerialChallengesAttempted)
        .zip(count(SeasonStat::HeadersWon))
        .and_then(|(attempted, won)| attempted.checked_sub(won));
    let average_rating = (player.rated_appearances > 0)
        .then(|| player.rating_sum / f64::from(player.rated_appearances));
    let xg_overperformance = player
        .total(SeasonStat::Goals)
        .zip(player.total(SeasonStat::ExpectedGoals))
        .map(|(goals, expected_goals)| goals - expected_goals);

    let totals = COUNTS
        .iter()
        .map(|(key, stat)| (*key, count(*stat).map(MoneyballMetricValue::Count)))
        .chain(
            DECIMALS
                .iter()
                .map(|(key, stat)| (*key, player.total(*stat).map(MoneyballMetricValue::Decimal))),
        )
        .chain([
            ("saves", saves.map(MoneyballMetricValue::Count)),
            (
                "headers_lost",
                headers_lost.map(MoneyballMetricValue::Count),
            ),
            (
                "games_drawn",
                games_drawn(player).map(MoneyballMetricValue::Count),
            ),
            (
                "average_rating",
                average_rating.map(MoneyballMetricValue::Decimal),
            ),
            (
                "xg-op",
                xg_overperformance.map(MoneyballMetricValue::Decimal),
            ),
            (
                "expected_save_percentage",
                expected_save_percentage(player).map(MoneyballMetricValue::Decimal),
            ),
        ]);
    let mut statistics: MoneyballStatistics = MONEYBALL_STATISTIC_KEYS
        .iter()
        .map(|key| ((*key).to_string(), None))
        .collect();
    for (key, value) in totals {
        statistics.insert(key.to_string(), value);
    }

    derive_rates(&mut statistics, player);
    statistics
}

/// The appearances that were neither won nor lost: a line stores no draws.
///
/// A competition the game does not simulate in detail stores no team results, so a player
/// with no win, loss, or team goal has no known draws rather than one for every appearance.
fn games_drawn(player: &SavePlayerSeasonStats) -> Option<u32> {
    let count = |stat| player.total(stat).map(|total| total as u32);
    let won = count(SeasonStat::GamesWon)?;
    let lost = count(SeasonStat::GamesLost)?;
    let team_goals = count(SeasonStat::TeamGoals)?;
    if won + lost + team_goals == 0 {
        return None;
    }
    (player.starts + player.substitute_appearances).checked_sub(won + lost)
}

/// The percentage of the on-target shots faced that their xG expects to be saved, cut to a
/// whole number as the game shows it: 87.73 is 87.
///
/// The save stores this xG in hundredths, and whole hundredths keep an exact percentage
/// exact: 0.68 xG from two shots is 66, where `100.0 * (1.0 - 0.68 / 2.0)` is just below it.
fn expected_save_percentage(player: &SavePlayerSeasonStats) -> Option<f64> {
    let shots_faced = player.total(SeasonStat::ShotsOnTargetFaced)? as u64;
    let xg_hundredths = (player.total(SeasonStat::ExpectedGoalsFaced)? * 100.0).round() as u64;
    let expected_saves_hundredths = (100 * shots_faced).checked_sub(xg_hundredths)?;
    (shots_faced > 0).then(|| (expected_saves_hundredths / shots_faced) as f64)
}

/// Fills per-90 values, ratios, and minutes-per-event values from the totals already set.
fn derive_rates(statistics: &mut MoneyballStatistics, player: &SavePlayerSeasonStats) {
    let value = |statistics: &MoneyballStatistics, key: &str| -> Option<f64> {
        match statistics.get(key).copied().flatten() {
            Some(MoneyballMetricValue::Count(value)) => Some(f64::from(value)),
            Some(MoneyballMetricValue::Decimal(value)) => Some(value),
            None => None,
        }
    };
    let quotient = |numerator: Option<f64>, denominator: Option<f64>| {
        numerator
            .zip(denominator.filter(|denominator| *denominator > 0.0))
            .map(|(numerator, denominator)| numerator / denominator)
            .filter(|value| value.is_finite())
            .map(MoneyballMetricValue::Decimal)
    };
    let minutes = Some(f64::from(player.minutes));

    let mut derived: Vec<(&str, Option<MoneyballMetricValue>)> = Vec::new();
    for key in MONEYBALL_STATISTIC_KEYS {
        if let Some(total_key) = key.strip_suffix(PER_90_SUFFIX) {
            let per_90 = quotient(
                value(statistics, total_key).map(|total| total * 90.0),
                minutes,
            );
            derived.push((key, per_90));
        }
    }
    for (key, numerator, denominator) in RATIOS {
        derived.push((
            *key,
            quotient(value(statistics, numerator), value(statistics, denominator)),
        ));
    }
    let goals = value(statistics, "goals");
    let assists = value(statistics, "assists");
    let appearances = f64::from(player.starts) + f64::from(player.substitute_appearances);
    let saves = value(statistics, "saves");
    let shots_faced = saves
        .zip(value(statistics, "goals_conceded"))
        .map(|(saves, conceded)| saves + conceded);
    let games_won = value(statistics, "games_won");
    let games = [
        games_won,
        value(statistics, "games_drawn"),
        value(statistics, "games_lost"),
    ]
    .into_iter()
    .sum::<Option<f64>>();
    derived.extend([
        ("minutes_per_goal", quotient(minutes, goals)),
        ("minutes_per_assist", quotient(minutes, assists)),
        (
            "minutes_per_goal_or_assist",
            quotient(
                minutes,
                goals.zip(assists).map(|(goals, assists)| goals + assists),
            ),
        ),
        (
            "clean_sheets_ratio",
            quotient(value(statistics, "clean_sheets"), Some(appearances)),
        ),
        ("save_ratio", quotient(saves, shots_faced)),
        ("game_win_ratio", quotient(games_won, games)),
    ]);
    for (key, metric) in derived {
        statistics.insert(key.to_string(), metric);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::moneyball::catalog::{
        is_save_unavailable_statistic_key, SAVE_UNAVAILABLE_STATISTIC_KEYS,
    };

    fn decimal(statistics: &MoneyballStatistics, key: &str) -> f64 {
        match statistics[key] {
            Some(MoneyballMetricValue::Decimal(value)) => value,
            other => panic!("{key} is {other:?}, not a decimal"),
        }
    }

    fn count(statistics: &MoneyballStatistics, key: &str) -> u32 {
        match statistics[key] {
            Some(MoneyballMetricValue::Count(value)) => value,
            other => panic!("{key} is {other:?}, not a count"),
        }
    }

    fn assert_close(actual: f64, expected: f64, key: &str) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "{key}: {actual} != {expected}"
        );
    }

    #[test]
    fn combined_counts_weighted_rating_and_rates_follow_the_canonical_rules() {
        let mut player = SavePlayerSeasonStats::for_test(1_500, |stat| match stat {
            SeasonStat::ClearCutChancesCreated => Some(6.0),
            SeasonStat::SavesHeld => Some(10.0),
            SeasonStat::SavesParried => Some(5.0),
            SeasonStat::SavesTipped => Some(3.0),
            SeasonStat::GoalsAllowed => Some(6.0),
            SeasonStat::AerialChallengesAttempted => Some(40.0),
            SeasonStat::HeadersWon => Some(25.0),
            SeasonStat::Goals => Some(4.0),
            SeasonStat::Assists => Some(2.0),
            SeasonStat::DistanceKm => Some(108.0),
            SeasonStat::PassesAttempted => Some(500.0),
            SeasonStat::PassesCompleted => Some(400.0),
            SeasonStat::ExpectedGoals => Some(2.5),
            SeasonStat::PressuresAttempted => Some(40.0),
            SeasonStat::PressuresCompleted => Some(10.0),
            SeasonStat::TeamGoals => Some(18.0),
            SeasonStat::GamesWon => Some(6.0),
            SeasonStat::GamesLost => Some(3.0),
            SeasonStat::PenaltiesFaced => Some(4.0),
            SeasonStat::PenaltiesSaved => Some(1.0),
            _ => None,
        });
        player.minutes = 900;
        player.starts = 9;
        player.substitute_appearances = 1;
        player.rated_appearances = 8;
        player.rating_sum = 56.8;

        let statistics = cohort_row(&player).statistics;

        assert_eq!(count(&statistics, "chances_created"), 6);
        assert_close(decimal(&statistics, "chances_created_per_90"), 0.6, "ccc");
        assert_eq!(count(&statistics, "saves"), 18);
        assert_close(decimal(&statistics, "saves_per_90"), 1.8, "saves");
        assert_close(decimal(&statistics, "save_ratio"), 0.75, "save ratio");
        assert_eq!(count(&statistics, "headers_lost"), 15);
        assert_close(decimal(&statistics, "headers_won_ratio"), 0.625, "headers");
        assert_close(decimal(&statistics, "average_rating"), 7.1, "rating");
        assert_close(decimal(&statistics, "goals_per_90"), 0.4, "goals");
        assert_close(decimal(&statistics, "distance_covered"), 108.0, "distance");
        assert_close(
            decimal(&statistics, "distance_covered_per_90"),
            10.8,
            "distance",
        );
        assert_close(decimal(&statistics, "pass_completion_ratio"), 0.8, "passes");
        assert_close(decimal(&statistics, "minutes_per_goal"), 225.0, "mpg");
        assert_close(
            decimal(&statistics, "minutes_per_goal_or_assist"),
            150.0,
            "mpga",
        );
        assert_eq!(count(&statistics, "pressures_attempted"), 40);
        assert_close(
            decimal(&statistics, "pressures_completed_per_90"),
            1.0,
            "pressures",
        );
        assert_close(
            decimal(&statistics, "pressure_success_ratio"),
            0.25,
            "pressures",
        );
        assert_close(decimal(&statistics, "team_goals_per_90"), 1.8, "team goals");
        assert_eq!(count(&statistics, "games_won"), 6);
        assert_eq!(count(&statistics, "games_lost"), 3);
        // Ten appearances, six won and three lost.
        assert_eq!(count(&statistics, "games_drawn"), 1);
        assert_close(decimal(&statistics, "game_win_ratio"), 0.6, "win ratio");
        assert_close(decimal(&statistics, "xg-op"), 1.5, "xG-OP");
        assert_close(decimal(&statistics, "xg-op_per_90"), 0.15, "xG-OP");
        assert_close(
            decimal(&statistics, "penalties_faced_per_90"),
            0.4,
            "penalties",
        );
        assert_close(
            decimal(&statistics, "penalties_saved_ratio"),
            0.25,
            "penalties",
        );
        // No shots stored: rates over them stay null rather than zero.
        assert_eq!(statistics["shots"], None);
        assert_eq!(statistics["shots_per_90"], None);
        assert_eq!(statistics["xg_per_shot"], None);
        for key in SAVE_UNAVAILABLE_STATISTIC_KEYS {
            assert_eq!(statistics[key], None, "{key}");
        }
    }

    #[test]
    fn rating_and_rates_are_null_without_rated_appearances_or_minutes() {
        let player = SavePlayerSeasonStats::for_test(1_500, |_| Some(0.0));

        let statistics = cohort_row(&player).statistics;

        assert_eq!(statistics["average_rating"], None);
        assert_eq!(statistics["goals"], Some(MoneyballMetricValue::Count(0)));
        assert_eq!(statistics["goals_per_90"], None);
        assert_eq!(statistics["minutes_per_goal"], None);
        assert_eq!(statistics["clean_sheets_ratio"], None);
    }

    #[test]
    fn expected_save_percentage_is_cut_to_a_whole_number_as_the_game_shows_it() {
        let expected_save_percentage = |expected_goals_faced: f64, shots_on_target_faced: f64| {
            let player = SavePlayerSeasonStats::for_test(1_500, |stat| match stat {
                SeasonStat::ExpectedGoalsFaced => Some(expected_goals_faced),
                SeasonStat::ShotsOnTargetFaced => Some(shots_on_target_faced),
                _ => None,
            });
            cohort_row(&player).statistics["expected_save_percentage"]
        };
        let whole = |percentage: f64| Some(MoneyballMetricValue::Decimal(percentage));

        // 87.73, which the game shows as 87.
        assert_eq!(expected_save_percentage(19.14, 156.0), whole(87.0));
        // 85.9, which the game shows as 85.
        assert_eq!(expected_save_percentage(1.41, 10.0), whole(85.0));
        // Exactly 66, which floating-point arithmetic puts just below 66.
        assert_eq!(expected_save_percentage(0.68, 2.0), whole(66.0));
        assert_eq!(expected_save_percentage(0.0, 0.0), None);
    }

    #[test]
    fn games_drawn_and_the_win_ratio_are_null_without_stored_team_results() {
        let team_results = |won: f64, lost: f64, team_goals: f64| {
            let mut player = SavePlayerSeasonStats::for_test(1_500, |stat| match stat {
                SeasonStat::GamesWon => Some(won),
                SeasonStat::GamesLost => Some(lost),
                SeasonStat::TeamGoals => Some(team_goals),
                _ => None,
            });
            player.minutes = 450;
            player.starts = 5;
            cohort_row(&player).statistics
        };

        // A competition the game does not simulate in detail stores no team results.
        let untracked = team_results(0.0, 0.0, 0.0);
        assert_eq!(count(&untracked, "games_won"), 0);
        assert_eq!(untracked["games_drawn"], None);
        assert_eq!(untracked["game_win_ratio"], None);

        let all_drawn = team_results(0.0, 0.0, 4.0);
        assert_eq!(count(&all_drawn, "games_drawn"), 5);
        assert_close(decimal(&all_drawn, "game_win_ratio"), 0.0, "win ratio");

        // More results than appearances cannot be split into draws.
        let inconsistent = team_results(4.0, 3.0, 9.0);
        assert_eq!(inconsistent["games_drawn"], None);
        assert_eq!(inconsistent["game_win_ratio"], None);
    }

    #[test]
    fn a_fully_populated_player_fills_every_key_the_save_can_source() {
        let mut player = SavePlayerSeasonStats::for_test(1_501, |stat| match stat {
            SeasonStat::GoalsOutsideBox => None,
            // Headers lost needs more attempts than wins.
            SeasonStat::AerialChallengesAttempted => Some(30.0),
            // Twelve wins leave room for draws in 22 appearances.
            SeasonStat::GamesLost => Some(5.0),
            _ => Some(12.0),
        });
        player.minutes = 1_800;
        player.starts = 20;
        player.substitute_appearances = 2;
        player.rated_appearances = 20;
        player.rating_sum = 140.0;

        let row = cohort_row(&player);

        assert_eq!(row.statistics.len(), MONEYBALL_STATISTIC_KEYS.len());
        for key in MONEYBALL_STATISTIC_KEYS {
            assert_eq!(
                row.statistics[key].is_some(),
                !is_save_unavailable_statistic_key(key),
                "{key}"
            );
        }
        assert_eq!(
            (
                row.player_uid,
                row.starts,
                row.substitute_appearances,
                row.minutes
            ),
            (1_501, 20, 2, 1_800)
        );
    }
}
