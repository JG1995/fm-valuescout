use crate::parser::types::ParsedPlayer;

/// Compute all derived metrics for a player.
/// Mutates the player's stat category structs in place.
/// - Per-90: value / minutes * 90. If minutes = 0 or None → None.
/// - Ratios: numerator / denominator. If denominator = 0 or None → None.
pub fn compute_metrics(player: &mut ParsedPlayer) {
    let minutes: Option<f64> = player.minutes.map(|m| m as f64);

    // ── Attacking per-90 ───────────────────────────────────────────────
    player.attacking.goals_per_90 = per_90(player.attacking.goals, minutes);
    player.attacking.xg_per_90 = per_90(player.attacking.xg, minutes);
    player.attacking.np_xg_per_90 = per_90(player.attacking.np_xg, minutes);
    player.attacking.shots_per_90 = per_90(player.attacking.shots, minutes);
    player.attacking.shots_on_target_per_90 = per_90(player.attacking.shots_on_target, minutes);

    // ── Chance creation per-90 + ratios ────────────────────────────────
    player.chance_creation.assists_per_90 = per_90(player.chance_creation.assists, minutes);
    player.chance_creation.xa_per_90 = per_90(player.chance_creation.xa, minutes);
    player.chance_creation.key_passes_per_90 = per_90(player.chance_creation.key_passes, minutes);
    player.chance_creation.progressive_passes_per_90 = per_90(player.chance_creation.progressive_passes, minutes);
    player.chance_creation.pass_completion_rate = ratio(
        player.chance_creation.passes_completed,
        player.chance_creation.passes_attempted,
    );

    // ── Movement per-90 ────────────────────────────────────────────────
    player.movement.dribbles_per_90 = per_90(player.movement.dribbles, minutes);
    player.movement.distance_per_90 = per_90(player.movement.distance_km, minutes);

    // ── Defending per-90 + ratios ──────────────────────────────────────
    player.defending.tackles_per_90 = per_90(player.defending.tackles_completed, minutes);
    player.defending.interceptions_per_90 = per_90(player.defending.interceptions, minutes);
    player.defending.pressures_per_90 = per_90(player.defending.pressures_attempted, minutes);
    player.defending.clearances_per_90 = per_90(player.defending.clearances, minutes);
    player.defending.tackle_completion_rate = ratio(
        player.defending.tackles_completed,
        player.defending.tackles_attempted,
    );
    player.defending.pressure_completion_rate = ratio(
        player.defending.pressures_completed,
        player.defending.pressures_attempted,
    );

    // ── Aerial ratios ──────────────────────────────────────────────────
    player.aerial.aerial_challenge_rate = ratio(
        player.aerial.aerial_challenges_won,
        player.aerial.aerial_challenges_attempted,
    );
    player.aerial.aerial_duels_per_90 = per_90(player.aerial.aerial_challenges_attempted, minutes);

    // ── Discipline per-90 ──────────────────────────────────────────────
    player.discipline.fouls_made_per_90 = per_90(player.discipline.fouls_made, minutes);
    player.discipline.fouls_against_per_90 = per_90(player.discipline.fouls_against, minutes);

    // ── Match outcome ──────────────────────────────────────────────────
    let total_games = safe_add(
        player.match_outcome.games_won,
        player.match_outcome.games_drawn,
        player.match_outcome.games_lost,
    );
    player.match_outcome.win_rate = ratio(player.match_outcome.games_won, total_games);
}

/// Compute per-90 value: (value / minutes) * 90.
/// Returns None if either input is None or if minutes is 0.
fn per_90(value: Option<f64>, minutes: Option<f64>) -> Option<f64> {
    match (value, minutes) {
        (Some(v), Some(m)) if m > 0.0 => Some(v / m * 90.0),
        _ => None,
    }
}

/// Compute ratio: numerator / denominator.
/// Returns None if either input is None or denominator is 0.
fn ratio(numerator: Option<f64>, denominator: Option<f64>) -> Option<f64> {
    match (numerator, denominator) {
        (Some(n), Some(d)) if d > 0.0 => Some(n / d),
        _ => None,
    }
}

/// Add up to three optional f64 values. Returns Some(sum) if any is Some.
fn safe_add(a: Option<f64>, b: Option<f64>, c: Option<f64>) -> Option<f64> {
    let vals = [a, b, c].into_iter().filter_map(|v| v);
    let mut sum = 0.0;
    let mut any = false;
    for v in vals {
        sum += v;
        any = true;
    }
    if any { Some(sum) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::types::*;

    fn make_player_with_minutes(minutes: u16) -> ParsedPlayer {
        let mut p = ParsedPlayer::empty(1, "Test".to_string(), vec![Position {
            role: Role::ST,
            sides: vec![Side::C],
        }]);
        p.minutes = Some(minutes);
        p.attacking.goals = Some(10.0);
        p.attacking.xg = Some(8.5);
        p.attacking.shots = Some(50.0);
        p.attacking.shots_on_target = Some(25.0);
        p.chance_creation.assists = Some(5.0);
        p.chance_creation.passes_attempted = Some(500.0);
        p.chance_creation.passes_completed = Some(400.0);
        p.movement.dribbles = Some(30.0);
        p.defending.tackles_attempted = Some(20.0);
        p.defending.tackles_completed = Some(15.0);
        p.defending.pressures_attempted = Some(100.0);
        p.defending.pressures_completed = Some(40.0);
        p.aerial.aerial_challenges_attempted = Some(50.0);
        p.aerial.aerial_challenges_won = Some(30.0);
        p.discipline.fouls_made = Some(12.0);
        p.match_outcome.games_won = Some(15.0);
        p.match_outcome.games_drawn = Some(5.0);
        p.match_outcome.games_lost = Some(10.0);
        p
    }

    #[test]
    fn per_90_normal() {
        let mut p = make_player_with_minutes(900);
        compute_metrics(&mut p);
        // 10 goals in 900 minutes → 1.0 per 90
        assert!((p.attacking.goals_per_90.unwrap() - 1.0).abs() < 0.001);
        // 8.5 xG in 900 minutes → 0.85 per 90
        assert!((p.attacking.xg_per_90.unwrap() - 0.85).abs() < 0.001);
    }

    #[test]
    fn per_90_zero_minutes() {
        let mut p = make_player_with_minutes(0);
        compute_metrics(&mut p);
        assert!(p.attacking.goals_per_90.is_none());
        assert!(p.chance_creation.assists_per_90.is_none());
    }

    #[test]
    fn ratio_normal() {
        let mut p = make_player_with_minutes(900);
        compute_metrics(&mut p);
        // 400/500 = 0.8
        assert!((p.chance_creation.pass_completion_rate.unwrap() - 0.8).abs() < 0.001);
        // 15/20 = 0.75
        assert!((p.defending.tackle_completion_rate.unwrap() - 0.75).abs() < 0.001);
    }

    #[test]
    fn ratio_zero_denominator() {
        let mut p = ParsedPlayer::empty(1, "Test".to_string(), vec![Position {
            role: Role::GK,
            sides: vec![],
        }]);
        p.minutes = Some(900);
        p.chance_creation.passes_attempted = Some(0.0);
        p.chance_creation.passes_completed = Some(0.0);
        compute_metrics(&mut p);
        assert!(p.chance_creation.pass_completion_rate.is_none());
    }

    #[test]
    fn ratio_none_inputs() {
        let mut p = ParsedPlayer::empty(1, "Test".to_string(), vec![Position {
            role: Role::GK,
            sides: vec![],
        }]);
        p.minutes = Some(900);
        compute_metrics(&mut p);
        assert!(p.chance_creation.pass_completion_rate.is_none());
        assert!(p.defending.tackle_completion_rate.is_none());
    }

    #[test]
    fn win_rate() {
        let mut p = make_player_with_minutes(900);
        compute_metrics(&mut p);
        // 15 won / 30 total = 0.5
        assert!((p.match_outcome.win_rate.unwrap() - 0.5).abs() < 0.001);
    }

    #[test]
    fn aerial_challenge_rate() {
        let mut p = make_player_with_minutes(900);
        compute_metrics(&mut p);
        // 30/50 = 0.6
        assert!((p.aerial.aerial_challenge_rate.unwrap() - 0.6).abs() < 0.001);
    }

    #[test]
    fn none_minutes_all_per90_none() {
        let mut p = ParsedPlayer::empty(1, "Test".to_string(), vec![Position {
            role: Role::ST,
            sides: vec![Side::C],
        }]);
        // minutes is None by default
        p.attacking.goals = Some(5.0);
        compute_metrics(&mut p);
        assert!(p.attacking.goals_per_90.is_none());
    }

    #[test]
    fn pressure_completion_rate() {
        let mut p = make_player_with_minutes(900);
        compute_metrics(&mut p);
        // 40/100 = 0.4
        assert!((p.defending.pressure_completion_rate.unwrap() - 0.4).abs() < 0.001);
    }
}
