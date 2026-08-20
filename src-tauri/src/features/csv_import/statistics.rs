use super::{MoneyballMetricValue, MoneyballPlayer, MoneyballStatistics};
use crate::features::moneyball::MONEYBALL_STATISTIC_KEYS;

const COUNT_SOURCE_KEYS: &[(&str, &str)] = &[
    ("goals", "Goals"),
    ("goals_from_outside_the_box", "Goals From Outside The Box"),
    ("shots", "Shots"),
    ("shots_on_target", "Shots on Target"),
    ("penalties_taken", "Penalties Taken"),
    ("penalties_scored", "Penalties Scored"),
    ("free_kicks", "Free Kick Shots"),
    ("assists", "Assists"),
    ("clear_cut_chances_created", "Clear Cut Chances Created"),
    ("key_passes", "Key Passes"),
    ("crosses_attempted", "Crosses Attempted"),
    ("crosses_completed", "Crosses Completed"),
    ("open_play_crosses_attempted", "Open Play Crosses Attempted"),
    ("open_play_crosses_completed", "Open Play Crosses Completed"),
    ("passes_attempted", "Passes Attempted"),
    ("passes_completed", "Passes Completed"),
    ("progressive_passes", "PsP"),
    ("dribbles_made", "Dribbles"),
    ("tackles_attempted", "Tackles Attempted"),
    ("tackles_completed", "Tackled Completed"),
    ("key_tackles", "Key Tackles"),
    ("interceptions", "Interceptions"),
    ("pressures_attempted", "Pres A"),
    ("pressures_completed", "Pres C"),
    ("blocks", "Blk"),
    ("shots_blocked_defending", "Shts Blckd"),
    ("clearances", "Clearances"),
    ("headers_attempted", "Headers Attempted"),
    ("headers_won", "Headers Won"),
    ("clean_sheets", "Clean Sheets"),
    ("goals_conceded", "Goals Conceded"),
    ("saves_held", "Saves Held"),
    ("saves_parried", "Saves Parried"),
    ("saves_tipped", "Saves Tipped"),
    ("penalties_faced", "Penalties Faced"),
    ("penalties_saved", "Penalties Saved"),
    ("fouls_made", "Fouls Made"),
    ("fouls_against", "Fouls Against"),
    ("yellow_cards", "Yellow Cards"),
    ("red_cards", "Red cards"),
    ("offsides", "Off"),
    ("mistakes_leading_to_goal", "Mistakes Leading to Goals"),
    ("player_of_the_match", "Player of the Match"),
    ("games_won", "Games Won"),
    ("games_drawn", "Games Drawn"),
    ("games_lost", "Games Lost"),
    ("team_goals", "Team Goals"),
];

const DECIMAL_SOURCE_KEYS: &[(&str, &str)] = &[
    ("xg", "xG"),
    ("np-xg", "NP-xG"),
    ("xg-op", "xG-OP"),
    ("xg_per_shot", "xG/shot"),
    (
        "shots_from_outside_the_box_per_90",
        "Shots From Outside The Box Per 90 minutes",
    ),
    ("xa", "xA"),
    ("chances_created_per_90", "Chances Created per 90"),
    ("open_play_key_passes_per_90", "Open Play Key Passes per 90"),
    ("high_intensity_sprints_per_90", "Sprints/90"),
    ("possession_lost_per_90", "Possession Lost per 90"),
    ("possession_won_per_90", "Possession Won per 90"),
    ("headers_lost_per_90", "Headers Lost per 90"),
    ("key_headers_per_90", "Key Headers per 90"),
    ("saves_per_90", "Saves per 90"),
    ("expected_save_percentage", "Expected Save Percentage"),
    ("xgp", "xGP"),
    ("average_rating", "Rating"),
];

const PER_90_COUNT_PAIRS: &[(&str, &str)] = &[
    ("goals", "goals_per_90"),
    (
        "goals_from_outside_the_box",
        "goals_from_outside_the_box_per_90",
    ),
    ("shots", "shots_per_90"),
    ("shots_on_target", "shots_on_target_per_90"),
    ("penalties_taken", "penalties_taken_per_90"),
    ("penalties_scored", "penalties_scored_per_90"),
    ("free_kicks", "free_kicks_per_90"),
    ("assists", "assists_per_90"),
    (
        "clear_cut_chances_created",
        "clear_cut_chances_created_per_90",
    ),
    ("key_passes", "key_passes_per_90"),
    ("crosses_attempted", "crosses_attempted_per_90"),
    ("crosses_completed", "crosses_completed_per_90"),
    (
        "open_play_crosses_attempted",
        "open_play_crosses_attempted_per_90",
    ),
    (
        "open_play_crosses_completed",
        "open_play_crosses_completed_per_90",
    ),
    ("passes_attempted", "passes_attempted_per_90"),
    ("passes_completed", "passes_completed_per_90"),
    ("progressive_passes", "progressive_passes_per_90"),
    ("dribbles_made", "dribbles_made_per_90"),
    ("tackles_attempted", "tackles_attempted_per_90"),
    ("tackles_completed", "tackles_completed_per_90"),
    ("key_tackles", "key_tackles_per_90"),
    ("interceptions", "interceptions_per_90"),
    ("pressures_attempted", "pressures_attempted_per_90"),
    ("pressures_completed", "pressures_completed_per_90"),
    ("blocks", "blocks_per_90"),
    ("shots_blocked_defending", "shots_blocked_defending_per_90"),
    ("clearances", "clearances_per_90"),
    ("headers_attempted", "headers_attempted_per_90"),
    ("headers_won", "headers_won_per_90"),
    ("clean_sheets", "clean_sheets_per_90"),
    ("goals_conceded", "goals_conceded_per_90"),
    ("saves_held", "saves_held_per_90"),
    ("saves_parried", "saves_parried_per_90"),
    ("saves_tipped", "saves_tipped_per_90"),
    ("penalties_faced", "penalties_faced_per_90"),
    ("penalties_saved", "penalties_saved_per_90"),
    ("fouls_made", "fouls_made_per_90"),
    ("fouls_against", "fouls_against_per_90"),
    ("yellow_cards", "yellow_cards_per_90"),
    ("red_cards", "red_cards_per_90"),
    ("offsides", "offsides_per_90"),
    (
        "mistakes_leading_to_goal",
        "mistakes_leading_to_goal_per_90",
    ),
    ("player_of_the_match", "player_of_the_match_per_90"),
    ("team_goals", "team_goals_per_90"),
];

const PER_90_DECIMAL_PAIRS: &[(&str, &str)] = &[
    ("xg", "xg_per_90"),
    ("np-xg", "np-xg_per_90"),
    ("xg-op", "xg-op_per_90"),
    ("xa", "xa_per_90"),
    ("distance_covered", "distance_covered_per_90"),
    ("xgp", "xgp_per_90"),
];

const TOTALS_FROM_PER_90: &[(&str, &str)] = &[
    (
        "shots_from_outside_the_box_per_90",
        "shots_from_outside_the_box",
    ),
    ("chances_created_per_90", "chances_created"),
    ("open_play_key_passes_per_90", "open_play_key_passes"),
    ("high_intensity_sprints_per_90", "high_intensity_sprints"),
    ("possession_lost_per_90", "possession_lost"),
    ("possession_won_per_90", "possession_won"),
    ("headers_lost_per_90", "headers_lost"),
    ("key_headers_per_90", "key_headers"),
    ("saves_per_90", "saves"),
];

const RATIO_FIELDS: &[(&str, &str, &str)] = &[
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
];

pub(crate) fn canonical_statistics(player: &MoneyballPlayer) -> MoneyballStatistics {
    let mut statistics: MoneyballStatistics = MONEYBALL_STATISTIC_KEYS
        .iter()
        .map(|key| ((*key).to_string(), None))
        .collect();

    for (key, source) in COUNT_SOURCE_KEYS {
        write_count(&mut statistics, key, source_count(player, source));
    }
    for (key, source) in DECIMAL_SOURCE_KEYS {
        write_decimal(&mut statistics, key, source_decimal(player, source));
    }
    write_decimal(
        &mut statistics,
        "distance_covered",
        player.distance_kilometers.filter(|value| value.is_finite()),
    );

    for (total_key, per_90_key) in PER_90_COUNT_PAIRS {
        let value = per_90(count(&statistics, total_key).map(f64::from), player.minutes);
        write_decimal(&mut statistics, per_90_key, value);
    }
    for (total_key, per_90_key) in PER_90_DECIMAL_PAIRS {
        let value = per_90(decimal(&statistics, total_key), player.minutes);
        write_decimal(&mut statistics, per_90_key, value);
    }
    for (per_90_key, total_key) in TOTALS_FROM_PER_90 {
        let value = from_per_90(decimal(&statistics, per_90_key), player.minutes);
        write_count(&mut statistics, total_key, value);
    }
    for (ratio_key, numerator_key, denominator_key) in RATIO_FIELDS {
        let value = ratio(
            count(&statistics, numerator_key),
            count(&statistics, denominator_key),
        );
        write_decimal(&mut statistics, ratio_key, value);
    }

    let goals = count(&statistics, "goals");
    let assists = count(&statistics, "assists");
    write_decimal(
        &mut statistics,
        "minutes_per_goal",
        minutes_per_events(player.minutes, goals.map(u64::from)),
    );
    write_decimal(
        &mut statistics,
        "minutes_per_goal_or_assist",
        match (player.minutes, goals, assists) {
            (Some(minutes), Some(goals), Some(assists)) => {
                minutes_per_events(Some(minutes), Some(u64::from(goals) + u64::from(assists)))
            }
            _ => None,
        },
    );
    write_decimal(
        &mut statistics,
        "minutes_per_assist",
        minutes_per_events(player.minutes, assists.map(u64::from)),
    );
    let clean_sheets = count(&statistics, "clean_sheets");
    write_decimal(
        &mut statistics,
        "clean_sheets_ratio",
        clean_sheets_ratio(clean_sheets, player),
    );
    let saves = count(&statistics, "saves");
    let goals_conceded = count(&statistics, "goals_conceded");
    write_decimal(
        &mut statistics,
        "save_ratio",
        save_ratio(
            source_decimal(player, "Save Percentage"),
            saves,
            goals_conceded,
        ),
    );
    let wins = count(&statistics, "games_won");
    let draws = count(&statistics, "games_drawn");
    let losses = count(&statistics, "games_lost");
    write_decimal(
        &mut statistics,
        "game_win_ratio",
        game_win_ratio(wins, draws, losses),
    );

    statistics
}

fn source_count(player: &MoneyballPlayer, source: &str) -> Option<u32> {
    match player.metric(source) {
        Some(MoneyballMetricValue::Count(value)) => Some(value),
        _ => None,
    }
}

fn source_decimal(player: &MoneyballPlayer, source: &str) -> Option<f64> {
    match player.metric(source) {
        Some(MoneyballMetricValue::Decimal(value)) if value.is_finite() => Some(value),
        _ => None,
    }
}

fn write_count(statistics: &mut MoneyballStatistics, key: &str, value: Option<u32>) {
    statistics.insert(key.to_string(), value.map(MoneyballMetricValue::Count));
}

fn write_decimal(statistics: &mut MoneyballStatistics, key: &str, value: Option<f64>) {
    statistics.insert(
        key.to_string(),
        value
            .filter(|value| value.is_finite())
            .map(MoneyballMetricValue::Decimal),
    );
}

fn count(statistics: &MoneyballStatistics, key: &str) -> Option<u32> {
    match statistics.get(key).copied().flatten() {
        Some(MoneyballMetricValue::Count(value)) => Some(value),
        _ => None,
    }
}

fn decimal(statistics: &MoneyballStatistics, key: &str) -> Option<f64> {
    match statistics.get(key).copied().flatten() {
        Some(MoneyballMetricValue::Decimal(value)) if value.is_finite() => Some(value),
        _ => None,
    }
}

fn per_90(value: Option<f64>, minutes: Option<u32>) -> Option<f64> {
    match (value, minutes) {
        (Some(value), Some(minutes)) if minutes > 0 => {
            let per_90 = value * 90.0 / f64::from(minutes);
            per_90.is_finite().then_some(per_90)
        }
        _ => None,
    }
}

fn from_per_90(value: Option<f64>, minutes: Option<u32>) -> Option<u32> {
    match (value, minutes) {
        (Some(value), Some(minutes)) if minutes > 0 => {
            let rounded = (value * f64::from(minutes) / 90.0).round();
            (rounded.is_finite() && rounded >= 0.0 && rounded <= f64::from(u32::MAX))
                .then_some(rounded as u32)
        }
        _ => None,
    }
}

fn ratio(numerator: Option<u32>, denominator: Option<u32>) -> Option<f64> {
    match (numerator, denominator) {
        (Some(numerator), Some(denominator)) if denominator > 0 => {
            let ratio = f64::from(numerator) / f64::from(denominator);
            ratio.is_finite().then_some(ratio)
        }
        _ => None,
    }
}

fn minutes_per_events(minutes: Option<u32>, events: Option<u64>) -> Option<f64> {
    match (minutes, events) {
        (Some(minutes), Some(events)) if events > 0 => {
            let value = f64::from(minutes) / events as f64;
            value.is_finite().then_some(value)
        }
        _ => None,
    }
}

fn clean_sheets_ratio(clean_sheets: Option<u32>, player: &MoneyballPlayer) -> Option<f64> {
    match (clean_sheets, player.appearances.as_ref()) {
        (Some(clean_sheets), Some(appearances)) => {
            let appearances = u64::from(appearances.starts) + u64::from(appearances.substitutes);
            (appearances > 0)
                .then_some(f64::from(clean_sheets) / appearances as f64)
                .filter(|value| value.is_finite())
        }
        _ => None,
    }
}

fn save_ratio(
    exported_percentage: Option<f64>,
    saves: Option<u32>,
    goals_conceded: Option<u32>,
) -> Option<f64> {
    match exported_percentage {
        Some(percentage) if percentage > 0.0 => {
            let ratio = percentage / 100.0;
            ratio.is_finite().then_some(ratio)
        }
        _ => match (saves, goals_conceded) {
            (Some(saves), Some(goals_conceded)) => {
                let denominator = u64::from(saves) + u64::from(goals_conceded);
                (denominator > 0)
                    .then_some(f64::from(saves) / denominator as f64)
                    .filter(|value| value.is_finite())
            }
            _ => None,
        },
    }
}

fn game_win_ratio(wins: Option<u32>, draws: Option<u32>, losses: Option<u32>) -> Option<f64> {
    match (wins, draws, losses) {
        (Some(wins), Some(draws), Some(losses)) => {
            let games = u64::from(wins) + u64::from(draws) + u64::from(losses);
            (games > 0)
                .then_some(f64::from(wins) / games as f64)
                .filter(|value| value.is_finite())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::super::{
        parser::{parse_csv, ParsedCsv},
        MoneyballAppearances, MoneyballMetricValue, MoneyballPlayer,
    };

    const MONEYBALL_EXPORT: &str = include_str!("fixtures/moneyball_stats.csv");
    const CANONICAL_STATISTIC_KEYS: &str =
        include_str!("fixtures/moneyball_canonical_statistics.txt");

    #[test]
    fn derives_the_exact_pinned_moneyball_catalogue() {
        let statistics = player_from_fixture().canonical_statistics();
        let expected = CANONICAL_STATISTIC_KEYS.lines().collect::<BTreeSet<_>>();
        let actual = statistics
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();

        assert_eq!(expected.len(), 138);
        assert_eq!(statistics.len(), 138);
        assert_eq!(actual, expected);
        assert!(statistics.values().flatten().all(|value| match value {
            MoneyballMetricValue::Count(_) => true,
            MoneyballMetricValue::Decimal(value) => value.is_finite(),
        }));
    }

    #[test]
    fn maps_every_exported_source_metric_to_its_legacy_key() {
        let mut player = player_from_fixture();
        let count_mappings = [
            ("Goals", "goals"),
            ("Goals From Outside The Box", "goals_from_outside_the_box"),
            ("Shots", "shots"),
            ("Shots on Target", "shots_on_target"),
            ("Penalties Taken", "penalties_taken"),
            ("Penalties Scored", "penalties_scored"),
            ("Free Kick Shots", "free_kicks"),
            ("Assists", "assists"),
            ("Clear Cut Chances Created", "clear_cut_chances_created"),
            ("Key Passes", "key_passes"),
            ("Crosses Attempted", "crosses_attempted"),
            ("Crosses Completed", "crosses_completed"),
            ("Open Play Crosses Attempted", "open_play_crosses_attempted"),
            ("Open Play Crosses Completed", "open_play_crosses_completed"),
            ("Passes Attempted", "passes_attempted"),
            ("Passes Completed", "passes_completed"),
            ("PsP", "progressive_passes"),
            ("Dribbles", "dribbles_made"),
            ("Tackles Attempted", "tackles_attempted"),
            ("Tackled Completed", "tackles_completed"),
            ("Key Tackles", "key_tackles"),
            ("Interceptions", "interceptions"),
            ("Pres A", "pressures_attempted"),
            ("Pres C", "pressures_completed"),
            ("Blk", "blocks"),
            ("Shts Blckd", "shots_blocked_defending"),
            ("Clearances", "clearances"),
            ("Headers Attempted", "headers_attempted"),
            ("Headers Won", "headers_won"),
            ("Clean Sheets", "clean_sheets"),
            ("Goals Conceded", "goals_conceded"),
            ("Saves Held", "saves_held"),
            ("Saves Parried", "saves_parried"),
            ("Saves Tipped", "saves_tipped"),
            ("Penalties Faced", "penalties_faced"),
            ("Penalties Saved", "penalties_saved"),
            ("Fouls Made", "fouls_made"),
            ("Fouls Against", "fouls_against"),
            ("Yellow Cards", "yellow_cards"),
            ("Red cards", "red_cards"),
            ("Off", "offsides"),
            ("Mistakes Leading to Goals", "mistakes_leading_to_goal"),
            ("Player of the Match", "player_of_the_match"),
            ("Games Won", "games_won"),
            ("Games Drawn", "games_drawn"),
            ("Games Lost", "games_lost"),
            ("Team Goals", "team_goals"),
        ];
        let decimal_mappings = [
            ("xG", "xg"),
            ("NP-xG", "np-xg"),
            ("xG-OP", "xg-op"),
            ("xG/shot", "xg_per_shot"),
            (
                "Shots From Outside The Box Per 90 minutes",
                "shots_from_outside_the_box_per_90",
            ),
            ("xA", "xa"),
            ("Chances Created per 90", "chances_created_per_90"),
            ("Open Play Key Passes per 90", "open_play_key_passes_per_90"),
            ("Sprints/90", "high_intensity_sprints_per_90"),
            ("Possession Lost per 90", "possession_lost_per_90"),
            ("Possession Won per 90", "possession_won_per_90"),
            ("Headers Lost per 90", "headers_lost_per_90"),
            ("Key Headers per 90", "key_headers_per_90"),
            ("Saves per 90", "saves_per_90"),
            ("Expected Save Percentage", "expected_save_percentage"),
            ("xGP", "xgp"),
            ("Rating", "average_rating"),
        ];

        for (index, (source, _)) in count_mappings.iter().enumerate() {
            player.metrics.insert(
                (*source).to_string(),
                Some(MoneyballMetricValue::Count((index + 1) as u32)),
            );
        }
        for (index, (source, _)) in decimal_mappings.iter().enumerate() {
            player.metrics.insert(
                (*source).to_string(),
                Some(MoneyballMetricValue::Decimal(index as f64 + 0.5)),
            );
        }
        player.distance_kilometers = Some(123.5);

        let statistics = player.canonical_statistics();

        for (index, (_, key)) in count_mappings.iter().enumerate() {
            assert_eq!(
                statistic(&statistics, key),
                Some(MoneyballMetricValue::Count((index + 1) as u32))
            );
        }
        for (index, (_, key)) in decimal_mappings.iter().enumerate() {
            assert_eq!(
                statistic(&statistics, key),
                Some(MoneyballMetricValue::Decimal(index as f64 + 0.5))
            );
        }
        assert_eq!(
            statistic(&statistics, "distance_covered"),
            Some(MoneyballMetricValue::Decimal(123.5))
        );
    }

    #[test]
    fn derives_every_legacy_per_ninety_partner_from_totals() {
        let mut player = player_from_fixture();
        player.metrics.clear();
        player.minutes = Some(900);
        let count_mappings = [
            ("Goals", "goals_per_90"),
            (
                "Goals From Outside The Box",
                "goals_from_outside_the_box_per_90",
            ),
            ("Shots", "shots_per_90"),
            ("Shots on Target", "shots_on_target_per_90"),
            ("Penalties Taken", "penalties_taken_per_90"),
            ("Penalties Scored", "penalties_scored_per_90"),
            ("Free Kick Shots", "free_kicks_per_90"),
            ("Assists", "assists_per_90"),
            (
                "Clear Cut Chances Created",
                "clear_cut_chances_created_per_90",
            ),
            ("Key Passes", "key_passes_per_90"),
            ("Crosses Attempted", "crosses_attempted_per_90"),
            ("Crosses Completed", "crosses_completed_per_90"),
            (
                "Open Play Crosses Attempted",
                "open_play_crosses_attempted_per_90",
            ),
            (
                "Open Play Crosses Completed",
                "open_play_crosses_completed_per_90",
            ),
            ("Passes Attempted", "passes_attempted_per_90"),
            ("Passes Completed", "passes_completed_per_90"),
            ("PsP", "progressive_passes_per_90"),
            ("Dribbles", "dribbles_made_per_90"),
            ("Tackles Attempted", "tackles_attempted_per_90"),
            ("Tackled Completed", "tackles_completed_per_90"),
            ("Key Tackles", "key_tackles_per_90"),
            ("Interceptions", "interceptions_per_90"),
            ("Pres A", "pressures_attempted_per_90"),
            ("Pres C", "pressures_completed_per_90"),
            ("Blk", "blocks_per_90"),
            ("Shts Blckd", "shots_blocked_defending_per_90"),
            ("Clearances", "clearances_per_90"),
            ("Headers Attempted", "headers_attempted_per_90"),
            ("Headers Won", "headers_won_per_90"),
            ("Clean Sheets", "clean_sheets_per_90"),
            ("Goals Conceded", "goals_conceded_per_90"),
            ("Saves Held", "saves_held_per_90"),
            ("Saves Parried", "saves_parried_per_90"),
            ("Saves Tipped", "saves_tipped_per_90"),
            ("Penalties Faced", "penalties_faced_per_90"),
            ("Penalties Saved", "penalties_saved_per_90"),
            ("Fouls Made", "fouls_made_per_90"),
            ("Fouls Against", "fouls_against_per_90"),
            ("Yellow Cards", "yellow_cards_per_90"),
            ("Red cards", "red_cards_per_90"),
            ("Off", "offsides_per_90"),
            (
                "Mistakes Leading to Goals",
                "mistakes_leading_to_goal_per_90",
            ),
            ("Player of the Match", "player_of_the_match_per_90"),
            ("Team Goals", "team_goals_per_90"),
        ];
        let decimal_mappings = [
            ("xG", "xg_per_90"),
            ("NP-xG", "np-xg_per_90"),
            ("xG-OP", "xg-op_per_90"),
            ("xA", "xa_per_90"),
            ("xGP", "xgp_per_90"),
        ];

        for (index, (source, _)) in count_mappings.iter().enumerate() {
            set_count(&mut player, source, ((index + 1) * 10) as u32);
        }
        for (index, (source, _)) in decimal_mappings.iter().enumerate() {
            set_decimal(&mut player, source, (index + 1) as f64 * 10.0);
        }
        player.distance_kilometers = Some(60.0);

        let statistics = player.canonical_statistics();

        for (index, (_, key)) in count_mappings.iter().enumerate() {
            assert_decimal(&statistics, key, (index + 1) as f64);
        }
        for (index, (_, key)) in decimal_mappings.iter().enumerate() {
            assert_decimal(&statistics, key, (index + 1) as f64);
        }
        assert_decimal(&statistics, "distance_covered_per_90", 6.0);
    }

    #[test]
    fn applies_the_pinned_derived_statistic_rules() {
        let mut player = player_from_fixture();
        player.metrics.clear();
        player.minutes = Some(900);
        player.appearances = Some(MoneyballAppearances {
            starts: 2,
            substitutes: 3,
        });
        player.distance_kilometers = Some(10.0);

        set_count(&mut player, "Goals", 10);
        set_count(&mut player, "Assists", 5);
        set_decimal(&mut player, "xG", 10.0);
        set_decimal(
            &mut player,
            "Shots From Outside The Box Per 90 minutes",
            0.825,
        );
        set_decimal(&mut player, "Chances Created per 90", 0.25);
        set_decimal(&mut player, "Open Play Key Passes per 90", 0.25);
        set_decimal(&mut player, "Sprints/90", 0.825);
        set_decimal(&mut player, "Possession Lost per 90", 0.825);
        set_decimal(&mut player, "Possession Won per 90", 0.825);
        set_decimal(&mut player, "Headers Lost per 90", 0.825);
        set_decimal(&mut player, "Key Headers per 90", 0.825);
        set_decimal(&mut player, "Saves per 90", 1.0);
        set_decimal(&mut player, "Save Percentage", 0.0);
        set_count(&mut player, "Penalties Taken", 4);
        set_count(&mut player, "Penalties Scored", 0);
        set_count(&mut player, "Crosses Attempted", 6);
        set_count(&mut player, "Crosses Completed", 3);
        set_count(&mut player, "Open Play Crosses Attempted", 8);
        set_count(&mut player, "Open Play Crosses Completed", 2);
        set_count(&mut player, "Passes Attempted", 10);
        set_count(&mut player, "Passes Completed", 9);
        set_count(&mut player, "Tackles Attempted", 5);
        set_count(&mut player, "Tackled Completed", 4);
        set_count(&mut player, "Pres A", 6);
        set_count(&mut player, "Pres C", 3);
        set_count(&mut player, "Headers Attempted", 7);
        set_count(&mut player, "Headers Won", 0);
        set_count(&mut player, "Clean Sheets", 0);
        set_count(&mut player, "Goals Conceded", 5);
        set_count(&mut player, "Penalties Faced", 2);
        set_count(&mut player, "Penalties Saved", 1);
        set_count(&mut player, "Games Won", 2);
        set_count(&mut player, "Games Drawn", 1);
        set_count(&mut player, "Games Lost", 1);
        set_count(&mut player, "Team Goals", 5);

        let statistics = player.canonical_statistics();

        assert_decimal(&statistics, "goals_per_90", 1.0);
        assert_decimal(&statistics, "xg_per_90", 1.0);
        assert_decimal(&statistics, "distance_covered_per_90", 1.0);
        assert_eq!(
            statistic(&statistics, "shots_from_outside_the_box"),
            Some(MoneyballMetricValue::Count(8))
        );
        assert_eq!(
            statistic(&statistics, "chances_created"),
            Some(MoneyballMetricValue::Count(3))
        );
        assert_eq!(
            statistic(&statistics, "open_play_key_passes"),
            Some(MoneyballMetricValue::Count(3))
        );
        for key in [
            "high_intensity_sprints",
            "possession_lost",
            "possession_won",
            "headers_lost",
            "key_headers",
        ] {
            assert_eq!(
                statistic(&statistics, key),
                Some(MoneyballMetricValue::Count(8))
            );
        }
        assert_eq!(
            statistic(&statistics, "saves"),
            Some(MoneyballMetricValue::Count(10))
        );
        assert_decimal(&statistics, "penalties_scored_ratio", 0.0);
        assert_decimal(&statistics, "cross_completion_ratio", 0.5);
        assert_decimal(&statistics, "open_play_cross_completion_ratio", 0.25);
        assert_decimal(&statistics, "pass_completion_ratio", 0.9);
        assert_decimal(&statistics, "tackle_completion_ratio", 0.8);
        assert_decimal(&statistics, "pressure_success_ratio", 0.5);
        assert_decimal(&statistics, "headers_won_ratio", 0.0);
        assert_decimal(&statistics, "penalties_saved_ratio", 0.5);
        assert_decimal(&statistics, "clean_sheets_ratio", 0.0);
        assert_decimal(&statistics, "save_ratio", 2.0 / 3.0);
        assert_decimal(&statistics, "game_win_ratio", 0.5);
        assert_decimal(&statistics, "minutes_per_goal", 90.0);
        assert_decimal(&statistics, "minutes_per_goal_or_assist", 60.0);
        assert_decimal(&statistics, "minutes_per_assist", 180.0);
        assert_decimal(&statistics, "team_goals_per_90", 0.5);

        set_decimal(&mut player, "Save Percentage", 84.0);
        assert_decimal(&player.canonical_statistics(), "save_ratio", 0.84);
    }

    #[test]
    fn preserves_null_and_zero_boundaries_and_never_returns_non_finite_values() {
        let mut player = player_from_fixture();
        player.metrics.clear();
        player.minutes = Some(0);
        set_count(&mut player, "Goals", 0);
        set_count(&mut player, "Penalties Taken", 0);
        set_count(&mut player, "Penalties Scored", 0);
        set_decimal(&mut player, "xG", f64::MAX);
        set_decimal(&mut player, "Save Percentage", 0.0);

        let statistics = player.canonical_statistics();

        assert_eq!(
            statistic(&statistics, "goals"),
            Some(MoneyballMetricValue::Count(0))
        );
        assert_eq!(statistic(&statistics, "goals_per_90"), None);
        assert_eq!(statistic(&statistics, "xg_per_90"), None);
        assert_eq!(statistic(&statistics, "penalties_scored_ratio"), None);
        assert_eq!(statistic(&statistics, "save_ratio"), None);
        assert!(statistics.values().flatten().all(|value| match value {
            MoneyballMetricValue::Count(_) => true,
            MoneyballMetricValue::Decimal(value) => value.is_finite(),
        }));
    }

    fn player_from_fixture() -> MoneyballPlayer {
        let ParsedCsv::Moneyball(mut players) =
            parse_csv(MONEYBALL_EXPORT).expect("parse Moneyball")
        else {
            panic!("detect Moneyball export");
        };
        players.remove(0)
    }

    fn set_count(player: &mut MoneyballPlayer, source: &str, value: u32) {
        player
            .metrics
            .insert(source.to_string(), Some(MoneyballMetricValue::Count(value)));
    }

    fn set_decimal(player: &mut MoneyballPlayer, source: &str, value: f64) {
        player.metrics.insert(
            source.to_string(),
            Some(MoneyballMetricValue::Decimal(value)),
        );
    }

    fn statistic(
        statistics: &std::collections::BTreeMap<String, Option<MoneyballMetricValue>>,
        key: &str,
    ) -> Option<MoneyballMetricValue> {
        statistics.get(key).copied().flatten()
    }

    fn assert_decimal(
        statistics: &std::collections::BTreeMap<String, Option<MoneyballMetricValue>>,
        key: &str,
        expected: f64,
    ) {
        let Some(MoneyballMetricValue::Decimal(actual)) = statistic(statistics, key) else {
            panic!("{key} must be a decimal statistic");
        };
        assert!(
            (actual - expected).abs() < 1e-12,
            "{key}: {actual} != {expected}"
        );
    }
}
