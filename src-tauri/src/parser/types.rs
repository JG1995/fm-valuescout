use serde::{Deserialize, Serialize};

/// Player role (e.g., ST, CM, CB, GK)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    ST,
    CF,
    AM,
    W,
    CM,
    DM,
    CB,
    FB,
    GK,
}

/// Side modifier for position
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Left,
    Center,
    Right,
}

/// Position combining role and optional side
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position {
    pub role: Role,
    pub side: Option<Side>,
}

impl Position {
    pub fn new(role: Role, side: Option<Side>) -> Self {
        Self { role, side }
    }
}

/// Footedness with label and optional score (1-5)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Footedness {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<u8>,
}

impl Footedness {
    pub fn new(label: String, score: Option<u8>) -> Self {
        Self { label, score }
    }
}

/// Nationality with name and optional code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Nationality {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl Nationality {
    pub fn new(name: String, code: Option<String>) -> Self {
        Self { name, code }
    }
}

/// Transfer value with low and high range (in GBP)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransferValue {
    pub low: Option<f64>,
    pub high: Option<f64>,
}

impl TransferValue {
    pub fn new(low: Option<f64>, high: Option<f64>) -> Self {
        Self { low, high }
    }

    pub fn is_empty(&self) -> bool {
        self.low.is_none() && self.high.is_none()
    }
}

impl Default for TransferValue {
    fn default() -> Self {
        Self {
            low: None,
            high: None,
        }
    }
}

/// Wage normalized to per-week (in GBP)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Wage {
    pub per_week: Option<f64>,
}

impl Wage {
    pub fn new(per_week: Option<f64>) -> Self {
        Self { per_week }
    }

    pub fn is_empty(&self) -> bool {
        self.per_week.is_none()
    }
}

impl Default for Wage {
    fn default() -> Self {
        Self { per_week: None }
    }
}

/// Attacking statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AttackingStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goals: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goals_per_90: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shots_total: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shots_on_target: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shot_accuracy_pct: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pens_scored: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pens_missed: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free_kicks_scored: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers_scored: Option<u16>,
}

/// Chance creation statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChanceCreationStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assists: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assists_per_90: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_passes: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chances_created: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crosses: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_accuracy_pct: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub through_balls: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_third_entries: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub penalty_area_entries: Option<u16>,
}

/// Movement statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MovementStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub touches: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub touches_per_90: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passes: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passes_per_90: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass_accuracy_pct: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward_passes: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backward_passes: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dribbles: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dribble_success_pct: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ball_recoveries: Option<u16>,
}

/// Defending statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DefendingStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tackles: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tackle_success_pct: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interceptions: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clearances: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers_won: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers_lost: Option<u16>,
}

/// Aerial duel statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AerialStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aerial_duels_won: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aerial_duels_lost: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aerial_win_pct: Option<f64>,
}

/// Goalkeeping statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GoalkeepingStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saves: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_pct: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goals_conceded: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clean_sheets: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub punches: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high_claims: Option<u16>,
}

/// Discipline statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DisciplineStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fouls_conceded: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fouls_won: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yellow_cards: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub red_cards: Option<u16>,
}

/// Match outcome statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MatchOutcomeStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches_played: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minutes: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starts: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bench: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_rating: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motm_awards: Option<u16>,
}

/// Column mapping status during parsing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnStatus {
    pub total_columns: usize,
    pub mapped_columns: usize,
    pub unmapped_columns: Vec<String>,
}

impl ColumnStatus {
    pub fn new(total_columns: usize, mapped_columns: usize, unmapped_columns: Vec<String>) -> Self {
        Self {
            total_columns,
            mapped_columns,
            unmapped_columns,
        }
    }
}

/// Information about a skipped row
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkippedRow {
    pub row_number: usize,
    pub reason: String,
}

impl SkippedRow {
    pub fn new(row_number: usize, reason: String) -> Self {
        Self { row_number, reason }
    }
}

/// Parse warning with metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParseWarning {
    pub row_number: usize,
    pub field: String,
    pub message: String,
}

impl ParseWarning {
    pub fn new(row_number: usize, field: String, message: String) -> Self {
        Self {
            row_number,
            field,
            message,
        }
    }
}

/// Complete result of parsing a CSV file
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParseResult {
    pub players: Vec<ParsedPlayer>,
    pub skipped_rows: Vec<SkippedRow>,
    pub warnings: Vec<ParseWarning>,
    pub column_status: ColumnStatus,
}

impl ParseResult {
    pub fn new(
        players: Vec<ParsedPlayer>,
        skipped_rows: Vec<SkippedRow>,
        warnings: Vec<ParseWarning>,
        column_status: ColumnStatus,
    ) -> Self {
        Self {
            players,
            skipped_rows,
            warnings,
            column_status,
        }
    }

    pub fn success_count(&self) -> usize {
        self.players.len()
    }

    pub fn skip_count(&self) -> usize {
        self.skipped_rows.len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
}

/// Complete parsed player data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedPlayer {
    // Identity fields
    pub uid: String,
    pub name: String,
    pub nationalities: Vec<Nationality>,

    // Position and role
    pub position: Position,
    pub footedness: Footedness,

    // Match outcomes
    pub match_outcomes: MatchOutcomeStats,

    // Attacking stats
    pub attacking: AttackingStats,

    // Chance creation stats
    pub chance_creation: ChanceCreationStats,

    // Movement stats
    pub movement: MovementStats,

    // Defending stats
    pub defending: DefendingStats,

    // Aerial stats
    pub aerial: AerialStats,

    // Goalkeeping stats (only for GKs)
    pub goalkeeping: GoalkeepingStats,

    // Discipline stats
    pub discipline: DisciplineStats,

    // Financial info
    pub transfer_value: TransferValue,
    pub wage: Wage,

    // In-game date snapshot
    pub snapshot_date: Option<String>,
}

impl ParsedPlayer {
    pub fn empty() -> Self {
        Self {
            uid: String::new(),
            name: String::new(),
            nationalities: Vec::new(),
            position: Position::new(Role::GK, None),
            footedness: Footedness::new(String::new(), None),
            match_outcomes: MatchOutcomeStats::default(),
            attacking: AttackingStats::default(),
            chance_creation: ChanceCreationStats::default(),
            movement: MovementStats::default(),
            defending: DefendingStats::default(),
            aerial: AerialStats::default(),
            goalkeeping: GoalkeepingStats::default(),
            discipline: DisciplineStats::default(),
            transfer_value: TransferValue::default(),
            wage: Wage::default(),
            snapshot_date: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_footedness_serde_round_trip() {
        let pos = Position::new(Role::ST, Some(Side::Center));
        let serialized = serde_json::to_string(&pos).unwrap();
        let deserialized: Position = serde_json::from_str(&serialized).unwrap();
        assert_eq!(pos, deserialized);

        let footedness = Footedness::new("Right".to_string(), Some(5));
        let serialized = serde_json::to_string(&footedness).unwrap();
        let deserialized: Footedness = serde_json::from_str(&serialized).unwrap();
        assert_eq!(footedness, deserialized);
    }

    #[test]
    fn test_parsed_player_empty_skeleton() {
        let player = ParsedPlayer::empty();
        assert!(player.uid.is_empty());
        assert!(player.name.is_empty());
        assert!(player.nationalities.is_empty());
        assert_eq!(player.position.role, Role::GK);
        assert!(player.footedness.label.is_empty());
        assert!(player.match_outcomes.matches_played.is_none());
        assert!(player.attacking.goals.is_none());
        assert!(player.transfer_value.is_empty());
        assert!(player.wage.is_empty());
    }

    #[test]
    fn test_parse_result_serialization() {
        let result = ParseResult::new(
            vec![],
            vec![SkippedRow::new(1, "Missing UID".to_string())],
            vec![ParseWarning::new(2, "Position".to_string(), "Unknown role".to_string())],
            ColumnStatus::new(80, 75, vec!["unknown_col".to_string()]),
        );

        let serialized = serde_json::to_string(&result).unwrap();
        let deserialized: ParseResult = serde_json::from_str(&serialized).unwrap();
        assert_eq!(result, deserialized);
    }

    #[test]
    fn test_nationality_with_and_without_code() {
        let with_code = Nationality::new("England".to_string(), Some("ENG".to_string()));
        let without_code = Nationality::new("England".to_string(), None);

        assert_eq!(with_code.code, Some("ENG".to_string()));
        assert_eq!(without_code.code, None);

        // Both should serialize/deserialize correctly
        let serialized = serde_json::to_string(&with_code).unwrap();
        let deserialized: Nationality = serde_json::from_str(&serialized).unwrap();
        assert_eq!(with_code, deserialized);

        let serialized = serde_json::to_string(&without_code).unwrap();
        let deserialized: Nationality = serde_json::from_str(&serialized).unwrap();
        assert_eq!(without_code, deserialized);
    }

    #[test]
    fn test_transfer_value_and_wage_defaults() {
        let transfer_value = TransferValue::default();
        assert!(transfer_value.is_empty());
        assert!(transfer_value.low.is_none());
        assert!(transfer_value.high.is_none());

        let wage = Wage::default();
        assert!(wage.is_empty());
        assert!(wage.per_week.is_none());

        // Test with values
        let transfer_value = TransferValue::new(Some(1000.0), Some(5000.0));
        assert!(!transfer_value.is_empty());

        let wage = Wage::new(Some(500.0));
        assert!(!wage.is_empty());
    }
}
