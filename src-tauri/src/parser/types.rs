use serde::{Deserialize, Serialize};

// ── Enums ──────────────────────────────────────────────────────────────

/// Position role from FM position string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    GK,
    D,
    WB,
    DM,
    M,
    AM,
    ST,
}

/// Side qualifier from FM position string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    L,
    C,
    R,
}

/// A single parsed position entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub role: Role,
    pub sides: Vec<Side>,
}

/// Footedness label and numeric score (1-5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Footedness {
    pub label: String,
    pub score: u8,
}

/// Nationality with optional 3-letter code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nationality {
    pub code: Option<String>,
    pub name: String,
}

// ── Error / warning types ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedRow {
    pub row_number: usize,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseWarning {
    pub row_number: usize,
    pub field: String,
    pub message: String,
}

// ── Stat category structs ─────────────────────────────────────────────

/// Attacking stats (goals, shots, xG variants).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackingStats {
    pub goals: Option<f64>,
    pub goals_from_outside_box: Option<f64>,
    pub xg: Option<f64>,
    pub np_xg: Option<f64>,
    pub xg_overperformance: Option<f64>,
    pub xg_per_shot: Option<f64>,
    pub shots: Option<f64>,
    pub shots_from_outside_box_per_90: Option<f64>,
    pub shots_on_target: Option<f64>,
    pub penalties_taken: Option<f64>,
    pub penalties_scored: Option<f64>,
    pub free_kick_shots: Option<f64>,
    // Per-90 computed
    pub goals_per_90: Option<f64>,
    pub xg_per_90: Option<f64>,
    pub np_xg_per_90: Option<f64>,
    pub shots_per_90: Option<f64>,
    pub shots_on_target_per_90: Option<f64>,
}

/// Chance creation / passing stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChanceCreationStats {
    pub assists: Option<f64>,
    pub xa: Option<f64>,
    pub chances_created_per_90: Option<f64>,
    pub clear_cut_chances: Option<f64>,
    pub key_passes: Option<f64>,
    pub open_play_key_passes_per_90: Option<f64>,
    pub crosses_attempted: Option<f64>,
    pub crosses_completed: Option<f64>,
    pub open_play_crosses_attempted: Option<f64>,
    pub open_play_crosses_completed: Option<f64>,
    pub passes_attempted: Option<f64>,
    pub passes_completed: Option<f64>,
    pub progressive_passes: Option<f64>,
    pub pass_completion_rate: Option<f64>,
    // Per-90 computed
    pub assists_per_90: Option<f64>,
    pub xa_per_90: Option<f64>,
    pub key_passes_per_90: Option<f64>,
    pub progressive_passes_per_90: Option<f64>,
}

/// Movement / physical stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementStats {
    pub dribbles: Option<f64>,
    pub distance_km: Option<f64>,
    pub sprints_per_90: Option<f64>,
    pub possession_lost_per_90: Option<f64>,
    // Per-90 computed
    pub dribbles_per_90: Option<f64>,
    pub distance_per_90: Option<f64>,
}

/// Defending stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefendingStats {
    pub tackles_attempted: Option<f64>,
    pub tackles_completed: Option<f64>,
    pub key_tackles: Option<f64>,
    pub interceptions: Option<f64>,
    pub possession_won_per_90: Option<f64>,
    pub pressures_attempted: Option<f64>,
    pub pressures_completed: Option<f64>,
    pub blocks: Option<f64>,
    pub shots_blocked: Option<f64>,
    pub clearances: Option<f64>,
    // Per-90 computed
    pub tackles_per_90: Option<f64>,
    pub interceptions_per_90: Option<f64>,
    pub pressures_per_90: Option<f64>,
    pub clearances_per_90: Option<f64>,
    pub tackle_completion_rate: Option<f64>,
    pub pressure_completion_rate: Option<f64>,
}

/// Aerial / heading stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AerialStats {
    pub aerial_challenges_attempted: Option<f64>,
    pub aerial_challenges_won: Option<f64>,
    pub aerial_challenges_lost_per_90: Option<f64>,
    pub key_headers_per_90: Option<f64>,
    // Per-90 computed
    pub aerial_challenge_rate: Option<f64>,
    pub aerial_duels_per_90: Option<f64>,
}

/// Goalkeeping stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalkeepingStats {
    pub clean_sheets: Option<f64>,
    pub goals_conceded: Option<f64>,
    pub saves_per_90: Option<f64>,
    pub expected_save_pct: Option<f64>,
    pub expected_goals_prevented: Option<f64>,
    pub saves_held: Option<f64>,
    pub saves_parried: Option<f64>,
    pub saves_tipped: Option<f64>,
    pub penalties_faced: Option<f64>,
    pub penalties_saved: Option<f64>,
}

/// Discipline stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisciplineStats {
    pub fouls_made: Option<f64>,
    pub fouls_against: Option<f64>,
    pub yellow_cards: Option<f64>,
    pub red_cards: Option<f64>,
    pub offsides: Option<f64>,
    pub mistakes_leading_to_goal: Option<f64>,
    // Per-90 computed
    pub fouls_made_per_90: Option<f64>,
    pub fouls_against_per_90: Option<f64>,
}

/// Match outcome / general stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchOutcomeStats {
    pub average_rating: Option<f64>,
    pub player_of_the_match: Option<f64>,
    pub games_won: Option<f64>,
    pub games_drawn: Option<f64>,
    pub games_lost: Option<f64>,
    pub team_goals: Option<f64>,
    // Per-90 computed
    pub win_rate: Option<f64>,
}

// ── Financial types ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferValue {
    pub currency_symbol: Option<String>,
    pub low: Option<f64>,
    pub high: Option<f64>,
    pub raw: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wage {
    pub currency_symbol: Option<String>,
    pub raw_value: Option<f64>,
    pub wage_per_week: Option<f64>,
    pub denomination: Option<String>,
    pub raw: Option<String>,
}

// ── Core player record ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedPlayer {
    pub uid: u32,
    pub name: String,
    pub nationality: Option<Nationality>,
    pub second_nationality: Option<Nationality>,
    pub club: Option<String>,
    pub age: Option<u16>,
    pub positions: Vec<Position>,
    pub height: Option<u16>,
    pub left_foot: Option<Footedness>,
    pub right_foot: Option<Footedness>,
    pub ca: Option<u16>,
    pub pa: Option<u16>,
    pub transfer_value: TransferValue,
    pub wage: Wage,
    pub contract_expires: Option<String>, // ISO date string
    pub appearances_started: Option<u16>,
    pub appearances_sub: Option<u16>,
    pub minutes: Option<u16>,
    pub attacking: AttackingStats,
    pub chance_creation: ChanceCreationStats,
    pub movement: MovementStats,
    pub defending: DefendingStats,
    pub aerial: AerialStats,
    pub goalkeeping: GoalkeepingStats,
    pub discipline: DisciplineStats,
    pub match_outcome: MatchOutcomeStats,
}

// ── Result type ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnStatus {
    pub name: String,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    pub players: Vec<ParsedPlayer>,
    pub skipped_rows: Vec<SkippedRow>,
    pub warnings: Vec<ParseWarning>,
    pub columns_found: Vec<ColumnStatus>,
    pub columns_missing: Vec<String>,
    pub total_rows: usize,
}

impl ParsedPlayer {
    /// Create a ParsedPlayer with all optional fields set to None/empty.
    /// Used as a builder base during row parsing.
    pub fn empty(uid: u32, name: String, positions: Vec<Position>) -> Self {
        Self {
            uid,
            name,
            nationality: None,
            second_nationality: None,
            club: None,
            age: None,
            positions,
            height: None,
            left_foot: None,
            right_foot: None,
            ca: None,
            pa: None,
            transfer_value: TransferValue {
                currency_symbol: None,
                low: None,
                high: None,
                raw: None,
            },
            wage: Wage {
                currency_symbol: None,
                raw_value: None,
                wage_per_week: None,
                denomination: None,
                raw: None,
            },
            contract_expires: None,
            appearances_started: None,
            appearances_sub: None,
            minutes: None,
            attacking: AttackingStats {
                goals: None,
                goals_from_outside_box: None,
                xg: None,
                np_xg: None,
                xg_overperformance: None,
                xg_per_shot: None,
                shots: None,
                shots_from_outside_box_per_90: None,
                shots_on_target: None,
                penalties_taken: None,
                penalties_scored: None,
                free_kick_shots: None,
                goals_per_90: None,
                xg_per_90: None,
                np_xg_per_90: None,
                shots_per_90: None,
                shots_on_target_per_90: None,
            },
            chance_creation: ChanceCreationStats {
                assists: None,
                xa: None,
                chances_created_per_90: None,
                clear_cut_chances: None,
                key_passes: None,
                open_play_key_passes_per_90: None,
                crosses_attempted: None,
                crosses_completed: None,
                open_play_crosses_attempted: None,
                open_play_crosses_completed: None,
                passes_attempted: None,
                passes_completed: None,
                progressive_passes: None,
                pass_completion_rate: None,
                assists_per_90: None,
                xa_per_90: None,
                key_passes_per_90: None,
                progressive_passes_per_90: None,
            },
            movement: MovementStats {
                dribbles: None,
                distance_km: None,
                sprints_per_90: None,
                possession_lost_per_90: None,
                dribbles_per_90: None,
                distance_per_90: None,
            },
            defending: DefendingStats {
                tackles_attempted: None,
                tackles_completed: None,
                key_tackles: None,
                interceptions: None,
                possession_won_per_90: None,
                pressures_attempted: None,
                pressures_completed: None,
                blocks: None,
                shots_blocked: None,
                clearances: None,
                tackles_per_90: None,
                interceptions_per_90: None,
                pressures_per_90: None,
                clearances_per_90: None,
                tackle_completion_rate: None,
                pressure_completion_rate: None,
            },
            aerial: AerialStats {
                aerial_challenges_attempted: None,
                aerial_challenges_won: None,
                aerial_challenges_lost_per_90: None,
                key_headers_per_90: None,
                aerial_challenge_rate: None,
                aerial_duels_per_90: None,
            },
            goalkeeping: GoalkeepingStats {
                clean_sheets: None,
                goals_conceded: None,
                saves_per_90: None,
                expected_save_pct: None,
                expected_goals_prevented: None,
                saves_held: None,
                saves_parried: None,
                saves_tipped: None,
                penalties_faced: None,
                penalties_saved: None,
            },
            discipline: DisciplineStats {
                fouls_made: None,
                fouls_against: None,
                yellow_cards: None,
                red_cards: None,
                offsides: None,
                mistakes_leading_to_goal: None,
                fouls_made_per_90: None,
                fouls_against_per_90: None,
            },
            match_outcome: MatchOutcomeStats {
                average_rating: None,
                player_of_the_match: None,
                games_won: None,
                games_drawn: None,
                games_lost: None,
                team_goals: None,
                win_rate: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_position_roundtrip() {
        let pos = Position {
            role: Role::AM,
            sides: vec![Side::L, Side::C],
        };
        let json = serde_json::to_string(&pos).unwrap();
        let back: Position = serde_json::from_str(&json).unwrap();
        assert_eq!(pos, back);
    }

    #[test]
    fn serde_footedness_roundtrip() {
        let f = Footedness {
            label: "Very Strong".to_string(),
            score: 5,
        };
        let json = serde_json::to_string(&f).unwrap();
        let back: Footedness = serde_json::from_str(&json).unwrap();
        assert_eq!(f, back);
    }

    #[test]
    fn empty_player_has_required_fields() {
        let p = ParsedPlayer::empty(12345, "Test Player".to_string(), vec![Position {
            role: Role::ST,
            sides: vec![Side::C],
        }]);
        assert_eq!(p.uid, 12345);
        assert_eq!(p.name, "Test Player");
        assert_eq!(p.positions.len(), 1);
        assert!(p.age.is_none());
        assert!(p.minutes.is_none());
    }

    #[test]
    fn parse_result_serializable() {
        let result = ParseResult {
            players: vec![],
            skipped_rows: vec![SkippedRow {
                row_number: 5,
                reason: "Missing UID".to_string(),
            }],
            warnings: vec![],
            columns_found: vec![],
            columns_missing: vec![],
            total_rows: 10,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: ParseResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.total_rows, back.total_rows);
    }
}
