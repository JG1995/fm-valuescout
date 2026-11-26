use std::collections::HashSet;

use rusqlite::Row;

use crate::features::moneyball::columns::statistic_column_of;
use crate::features::moneyball::comparison::MIN_SEASON_MINUTES;
use crate::features::moneyball::{is_moneyball_statistic_key, role_catalog::builtin_catalog};
use crate::features::scoring::catalog::{all_roles, DUMP_ATTRIBUTE_KEYS};

use super::{
    club_dna::SCORE_MODEL_VERSION,
    compact::{player_current_column, player_potential_column, PLAYER_METRICS_ALIAS},
};

/// FM27 dump position keys (bridge `PositionEntries`).
pub const POSITION_KEYS: &[&str] = &[
    "GK", "SW", "DL", "DC", "DR", "DM", "ML", "MC", "MR", "AML", "AMC", "AMR", "ST", "WBL", "WBR",
];

const POSITION_DISPLAY_ORDER_SQL: &str = "CASE entry.key WHEN 'GK' THEN 0 WHEN 'SW' THEN 1 WHEN 'DR' THEN 2 WHEN 'DC' THEN 3 WHEN 'DL' THEN 4 WHEN 'WBR' THEN 5 WHEN 'DM' THEN 6 WHEN 'WBL' THEN 7 WHEN 'MR' THEN 8 WHEN 'MC' THEN 9 WHEN 'ML' THEN 10 WHEN 'AMR' THEN 11 WHEN 'AMC' THEN 12 WHEN 'AML' THEN 13 WHEN 'ST' THEN 14 ELSE 15 END";

pub const HIDDEN_ATTRIBUTE_KEYS: &[&str] = &[
    "Dirtiness",
    "Consistency",
    "ImportantMatches",
    "InjuryProneness",
    "Versatility",
];

pub const PERSONALITY_KEYS: &[&str] = &[
    "Adaptability",
    "Ambition",
    "Loyalty",
    "Pressure",
    "Professionalism",
    "Sportsmanship",
    "Temperament",
    "Controversy",
];

/// The most columns one request reads. Keep direct IPC input bounded before it can expand a
/// row projection.
pub const MAX_REQUESTED_FIELDS: usize = 256;

pub const TACTIC_CURRENT_PREFIX: &str = "tactic_current.";
pub const TACTIC_POTENTIAL_PREFIX: &str = "tactic_potential.";
pub const TACTIC_MONEYBALL_PREFIX: &str = "tactic_moneyball.";

/// True for a season statistics row, aliased `moneyball`, that counts: a scored row with at
/// least `MIN_SEASON_MINUTES` minutes. A player without one has no statistics, percentiles, or
/// Moneyball Scores, so every rule on them drops him.
pub fn qualified_season_stats_sql() -> String {
    format!(
        "(moneyball.minutes >= {MIN_SEASON_MINUTES} AND moneyball.percentiles_json IS NOT NULL)"
    )
}

/// A statistic of the `moneyball` row, or NULL when that row does not qualify or `key` is
/// outside the catalog.
pub fn moneyball_statistic_sql(key: &str) -> String {
    match statistic_column_of(key) {
        Some(column) => format!(
            "CASE WHEN {} THEN moneyball.{column} END",
            qualified_season_stats_sql()
        ),
        None => "NULL".to_string(),
    }
}

/// A playing-time count of the `moneyball` row: 0 for a player without one.
pub fn moneyball_context_sql(column: &str) -> String {
    format!("COALESCE(moneyball.{column}, 0)")
}

pub const TACTIC_LANE_IDS: [&str; 11] = [
    "goalkeeper",
    "left_back",
    "left_centre_back",
    "right_centre_back",
    "right_back",
    "defensive_midfielder",
    "left_central_midfielder",
    "right_central_midfielder",
    "left_winger",
    "right_winger",
    "centre_forward",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TacticGroup {
    Current,
    Potential,
    /// The slot's Moneyball Score: its chosen metrics' mean percentile, or the Moneyball
    /// definitions of its roles when it has no chosen metrics.
    Moneyball,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricValueKind {
    Integer,
    Real,
    Text,
}

/// Exact persisted Club DNA score identity for one validated sort.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClubDnaSortIdentity {
    pub definition_version: Option<i64>,
    pub score_model_version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MetricSource {
    Column(&'static str),
    JsonInteger {
        column: &'static str,
        key: String,
    },
    MoneyballContext {
        column: &'static str,
    },
    MoneyballStatistic {
        key: String,
    },
    MoneyballRole {
        role_id: String,
    },
    Position,
    CurrentRole {
        role_id: &'static str,
        column: String,
    },
    PotentialRole {
        role_id: &'static str,
        column: String,
    },
    ClubDna,
    Tactic {
        group: TacticGroup,
        lane_id: String,
    },
}

impl MetricSource {
    fn current_role_id(&self) -> Option<&'static str> {
        match self {
            Self::CurrentRole { role_id, .. } => Some(role_id),
            _ => None,
        }
    }
}

/// Bound parameter positions for an exact persisted Club DNA score identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClubDnaSqlBindings {
    pub(crate) definition_version: usize,
    pub(crate) score_model_version: usize,
}

impl ClubDnaSqlBindings {
    pub fn new(definition_version: usize, score_model_version: usize) -> Self {
        Self {
            definition_version,
            score_model_version,
        }
    }

    fn score_expression(self, player_alias: &str) -> String {
        format!(
            "(SELECT cds.score FROM club_dna_scores cds WHERE cds.snapshot_id = {player_alias}.snapshot_id AND cds.uid = {player_alias}.uid AND cds.definition_version = ?{} AND cds.score_model_version = ?{})",
            self.definition_version, self.score_model_version,
        )
    }
}

/// One Rust-validated display or sort metric.
///
/// SQL is constructed only from these closed sources, never from raw WebView input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricField {
    id: String,
    kind: MetricValueKind,
    source: MetricSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DynamicValue {
    Integer(i64),
    Real(f64),
    Text(String),
}

impl MetricField {
    /// A player field: no season statistics and no tactic columns.
    pub fn parse(field: &str) -> Result<Self, String> {
        Self::parse_inner(field, false)
    }

    /// A Player Search field: any player field, season statistic, playing-time count,
    /// Moneyball Score, or tactic column.
    pub fn parse_search(field: &str) -> Result<Self, String> {
        Self::parse_inner(field, true)
    }

    fn parse_inner(field: &str, search: bool) -> Result<Self, String> {
        if search {
            if let Some(tactic) = parse_tactic_field(field)? {
                return Ok(tactic);
            }
        } else if field.starts_with(TACTIC_CURRENT_PREFIX)
            || field.starts_with(TACTIC_POTENTIAL_PREFIX)
            || field.starts_with(TACTIC_MONEYBALL_PREFIX)
        {
            return Err(format!("unknown player metric: {field}"));
        }
        if let Some(role_id) = parse_moneyball_role_id(field, search)? {
            return Ok(Self {
                id: field.to_string(),
                kind: MetricValueKind::Integer,
                source: MetricSource::MoneyballRole { role_id },
            });
        }
        if field == "club_dna" {
            return Ok(Self {
                id: field.to_string(),
                kind: MetricValueKind::Integer,
                source: MetricSource::ClubDna,
            });
        }
        if let Some((column, kind)) = scalar_metric(field) {
            return Ok(Self {
                id: field.to_string(),
                kind,
                source: MetricSource::Column(column),
            });
        }
        if field == "position" {
            return Ok(Self {
                id: field.to_string(),
                kind: MetricValueKind::Text,
                source: MetricSource::Position,
            });
        }
        if let Some(key) = attribute_key(field)? {
            return Ok(json_integer_metric(field, "attributes_json", key));
        }
        if let Some(key) = hidden_attribute_key(field)? {
            return Ok(json_integer_metric(field, "hidden_attributes_json", key));
        }
        if let Some(key) = personality_key(field)? {
            return Ok(json_integer_metric(field, "personality_json", key));
        }
        if let Some(key) = field.strip_prefix("pos.") {
            let Some(canonical) = POSITION_KEYS
                .iter()
                .copied()
                .find(|candidate| *candidate == key)
            else {
                return Err(format!("unknown position key: {key}"));
            };
            return Ok(Self {
                id: field.to_string(),
                kind: MetricValueKind::Integer,
                source: MetricSource::JsonInteger {
                    column: "positions_json",
                    key: canonical.to_string(),
                },
            });
        }
        if let Some(role_id) = field.strip_prefix("role.") {
            let role_id = catalog_role_id(role_id)?;
            let column = player_current_column(role_id)?.to_string();
            return Ok(Self {
                id: field.to_string(),
                kind: MetricValueKind::Integer,
                source: MetricSource::CurrentRole { role_id, column },
            });
        }
        if let Some(role_id) = field.strip_prefix("potential_role.") {
            let role_id = catalog_role_id(role_id)?;
            let column = player_potential_column(role_id)?;
            return Ok(Self {
                id: field.to_string(),
                kind: MetricValueKind::Integer,
                source: MetricSource::PotentialRole { role_id, column },
            });
        }
        if search {
            if let Some(column) = moneyball_context_column(field) {
                return Ok(Self {
                    id: field.to_string(),
                    kind: MetricValueKind::Integer,
                    source: MetricSource::MoneyballContext { column },
                });
            }
        }
        if let Some(key) = field.strip_prefix("moneyball.") {
            if !search {
                return Err(format!("unknown player metric: {field}"));
            }
            if !is_moneyball_statistic_key(key) {
                return Err(format!("unknown Moneyball metric: {key}"));
            }
            return Ok(Self {
                id: field.to_string(),
                kind: MetricValueKind::Real,
                source: MetricSource::MoneyballStatistic {
                    key: key.to_string(),
                },
            });
        }
        Err(format!("unknown player metric: {field}"))
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn kind(&self) -> MetricValueKind {
        self.kind
    }

    pub fn is_basic_table_field(&self) -> bool {
        matches!(
            self.id.as_str(),
            "name" | "age" | "nationality" | "club" | "division" | "ca" | "pa" | "value"
        )
    }

    pub fn potential_role_id(&self) -> Option<&'static str> {
        match self.source {
            MetricSource::PotentialRole { role_id, .. } => Some(role_id),
            _ => None,
        }
    }

    pub fn current_role_id(&self) -> Option<&'static str> {
        self.source.current_role_id()
    }

    pub fn club_dna_sort_identity(
        &self,
        definition_version: Option<i64>,
    ) -> Option<ClubDnaSortIdentity> {
        self.is_club_dna().then_some(ClubDnaSortIdentity {
            definition_version,
            score_model_version: SCORE_MODEL_VERSION,
        })
    }

    pub fn moneyball_key(&self) -> Option<&str> {
        match &self.source {
            MetricSource::MoneyballStatistic { key } => Some(key),
            _ => None,
        }
    }

    pub fn moneyball_role_id(&self) -> Option<&str> {
        match &self.source {
            MetricSource::MoneyballRole { role_id } => Some(role_id),
            _ => None,
        }
    }

    /// Whether its SQL reads the season statistics row aliased `moneyball`.
    pub fn reads_season_stats(&self) -> bool {
        matches!(
            self.source,
            MetricSource::MoneyballContext { .. } | MetricSource::MoneyballStatistic { .. }
        )
    }

    /// Whether its value is a Moneyball Score computed in Rust from ranked statistics: a
    /// role's or a tactic slot's.
    pub fn is_moneyball_score(&self) -> bool {
        matches!(
            self.source,
            MetricSource::MoneyballRole { .. }
                | MetricSource::Tactic {
                    group: TacticGroup::Moneyball,
                    ..
                }
        )
    }

    pub fn is_club_dna(&self) -> bool {
        matches!(self.source, MetricSource::ClubDna)
    }

    #[allow(dead_code)]
    pub fn is_tactic_field(&self) -> bool {
        matches!(self.source, MetricSource::Tactic { .. })
    }

    #[allow(dead_code)]
    pub fn is_tactic_current(&self) -> bool {
        matches!(
            self.source,
            MetricSource::Tactic {
                group: TacticGroup::Current,
                ..
            }
        )
    }

    #[allow(dead_code)]
    pub fn tactic_lane_id(&self) -> Option<&str> {
        match &self.source {
            MetricSource::Tactic { lane_id, .. } => Some(lane_id),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn tactic_group(&self) -> Option<TacticGroup> {
        match &self.source {
            MetricSource::Tactic { group, .. } => Some(group.clone()),
            _ => None,
        }
    }

    /// Returns a trusted SQLite expression relative to the supplied player-table alias.
    #[cfg(test)]
    pub fn sql_expression(&self, player_alias: &str) -> String {
        self.sql_expression_with_club_dna(player_alias, None)
    }

    pub fn sql_expression_with_club_dna(
        &self,
        player_alias: &str,
        club_dna_bindings: Option<ClubDnaSqlBindings>,
    ) -> String {
        match &self.source {
            MetricSource::Column(column) => format!("{player_alias}.{column}"),
            MetricSource::JsonInteger { column, key } => {
                format!("json_extract({player_alias}.{column}, '$.{key}')")
            }
            MetricSource::MoneyballContext { column } => moneyball_context_sql(column),
            MetricSource::MoneyballStatistic { key } => moneyball_statistic_sql(key),
            // Moneyball Scores are computed in Rust from ranked statistics.
            MetricSource::MoneyballRole { .. } => "NULL".to_string(),
            MetricSource::Position => format!(
                "COALESCE((SELECT group_concat(position_key, ', ') FROM (SELECT entry.key AS position_key FROM json_each({player_alias}.positions_json) AS entry WHERE entry.type = 'integer' AND entry.value > 0 ORDER BY entry.value DESC, {POSITION_DISPLAY_ORDER_SQL}, entry.key ASC)), '')"
            ),
            MetricSource::CurrentRole { column, .. } | MetricSource::PotentialRole { column, .. } => {
                format!("{PLAYER_METRICS_ALIAS}.{column}")
            }
            MetricSource::ClubDna => club_dna_bindings
                .map(|bindings| bindings.score_expression(player_alias))
                .unwrap_or_else(|| "NULL".to_string()),
            MetricSource::Tactic { .. } => "NULL".to_string(),
        }
    }

    pub fn sql_sort_expression_with_club_dna(
        &self,
        player_alias: &str,
        club_dna_bindings: Option<ClubDnaSqlBindings>,
    ) -> String {
        let expression = self.sql_expression_with_club_dna(player_alias, club_dna_bindings);
        match self.kind {
            MetricValueKind::Integer | MetricValueKind::Real => expression,
            MetricValueKind::Text => format!("{expression} COLLATE NOCASE"),
        }
    }
}

pub fn moneyball_context_column(field: &str) -> Option<&'static str> {
    match field {
        "moneyball.starts" => Some("starts"),
        "moneyball.substitute_appearances" => Some("substitute_appearances"),
        "moneyball.minutes" => Some("minutes"),
        _ => None,
    }
}

pub fn parse_moneyball_role_id(field: &str, moneyball: bool) -> Result<Option<String>, String> {
    let Some(role_id) = field.strip_prefix("moneyball_role.") else {
        return Ok(None);
    };
    if !moneyball {
        return Err(format!("unknown player metric: {field}"));
    }
    let catalog = builtin_catalog()?;
    if catalog
        .definitions
        .iter()
        .any(|definition| definition.id == role_id)
    {
        Ok(Some(role_id.to_string()))
    } else {
        Err(format!("unknown Moneyball role: {role_id}"))
    }
}

/// A tactic slot's Role Score (`tactic_current.*`), Potential Role Score
/// (`tactic_potential.*`), or Moneyball Score (`tactic_moneyball.*`).
fn parse_tactic_field(field: &str) -> Result<Option<MetricField>, String> {
    let (group, lane_id) = if let Some(lane) = field.strip_prefix(TACTIC_CURRENT_PREFIX) {
        (TacticGroup::Current, lane)
    } else if let Some(lane) = field.strip_prefix(TACTIC_POTENTIAL_PREFIX) {
        (TacticGroup::Potential, lane)
    } else if let Some(lane) = field.strip_prefix(TACTIC_MONEYBALL_PREFIX) {
        (TacticGroup::Moneyball, lane)
    } else {
        return Ok(None);
    };
    if !TACTIC_LANE_IDS.contains(&lane_id) {
        return Err(format!("unknown player metric: {field}"));
    }
    Ok(Some(MetricField {
        id: field.to_string(),
        kind: MetricValueKind::Integer,
        source: MetricSource::Tactic {
            group,
            lane_id: lane_id.to_string(),
        },
    }))
}

/// Validates request fields and retains their first-seen order.
pub fn parse_requested_fields(fields: &[String]) -> Result<Vec<MetricField>, String> {
    if fields.len() > MAX_REQUESTED_FIELDS {
        return Err(format!(
            "requested field count exceeds maximum of {MAX_REQUESTED_FIELDS}"
        ));
    }
    let mut seen = HashSet::new();
    let mut parsed = Vec::new();
    for field in fields {
        let metric = MetricField::parse(field)?;
        if seen.insert(metric.id.clone()) {
            parsed.push(metric);
        }
    }
    Ok(parsed)
}

/// `parse_requested_fields` for Player Search's fields.
pub fn parse_search_requested_fields(fields: &[String]) -> Result<Vec<MetricField>, String> {
    if fields.len() > MAX_REQUESTED_FIELDS {
        return Err(format!(
            "requested field count exceeds maximum of {MAX_REQUESTED_FIELDS}"
        ));
    }
    let mut seen = HashSet::new();
    let mut parsed = Vec::new();
    for field in fields {
        let metric = MetricField::parse_search(field)?;
        if seen.insert(metric.id.clone()) {
            parsed.push(metric);
        }
    }
    Ok(parsed)
}

pub fn read_dynamic_value(
    row: &Row<'_>,
    index: usize,
    metric: &MetricField,
) -> rusqlite::Result<Option<DynamicValue>> {
    match metric.kind() {
        MetricValueKind::Integer => row
            .get::<_, Option<i64>>(index)
            .map(|value| value.map(DynamicValue::Integer)),
        MetricValueKind::Real => row
            .get::<_, Option<f64>>(index)
            .map(|value| value.map(DynamicValue::Real)),
        MetricValueKind::Text => row
            .get::<_, Option<String>>(index)
            .map(|value| value.map(DynamicValue::Text)),
    }
}

pub fn catalog_role_id(role_id: &str) -> Result<&'static str, String> {
    all_roles()
        .iter()
        .find(|candidate| candidate.role_id == role_id)
        .map(|role| role.role_id)
        .ok_or_else(|| format!("unknown role id: {role_id}"))
}

fn scalar_metric(field: &str) -> Option<(&'static str, MetricValueKind)> {
    match field {
        "name" => Some(("name", MetricValueKind::Text)),
        "club" => Some(("current_club", MetricValueKind::Text)),
        "division" => Some(("division", MetricValueKind::Text)),
        "parent_club" => Some(("parent_club", MetricValueKind::Text)),
        "nationality" => Some(("nationalities_json", MetricValueKind::Text)),
        "preferred_foot" => Some(("preferred_foot", MetricValueKind::Text)),
        "team_level" => Some(("team_level", MetricValueKind::Text)),
        "age" => Some(("age", MetricValueKind::Integer)),
        "ca" => Some(("ca", MetricValueKind::Integer)),
        "pa" => Some(("pa", MetricValueKind::Integer)),
        "height" => Some(("height_cm", MetricValueKind::Integer)),
        "wage" => Some(("weekly_wage_gbp", MetricValueKind::Integer)),
        "value" => Some(("market_value_gbp", MetricValueKind::Integer)),
        "reputation" => Some(("reputation_current", MetricValueKind::Integer)),
        "world_reputation" => Some(("reputation_world", MetricValueKind::Integer)),
        "birth_year" => Some(("birth_year", MetricValueKind::Integer)),
        "contract_year" => Some(("contract_expiry_year", MetricValueKind::Integer)),
        "transfer_listed" => Some(("transfer_listed", MetricValueKind::Integer)),
        "loan_listed" => Some(("loan_listed", MetricValueKind::Integer)),
        "not_for_sale" => Some(("not_for_sale", MetricValueKind::Integer)),
        "set_for_release" => Some(("set_for_release", MetricValueKind::Integer)),
        "on_loan" => Some(("on_loan", MetricValueKind::Integer)),
        _ => None,
    }
}

pub fn attribute_key(field: &str) -> Result<Option<&'static str>, String> {
    catalog_prefixed_key(field, "attr.", DUMP_ATTRIBUTE_KEYS)
}

pub fn hidden_attribute_key(field: &str) -> Result<Option<&'static str>, String> {
    catalog_prefixed_key(field, "hidden.", HIDDEN_ATTRIBUTE_KEYS)
}

pub fn personality_key(field: &str) -> Result<Option<&'static str>, String> {
    catalog_prefixed_key(field, "personality.", PERSONALITY_KEYS)
}

fn json_integer_metric(field: &str, column: &'static str, key: &str) -> MetricField {
    MetricField {
        id: field.to_string(),
        kind: MetricValueKind::Integer,
        source: MetricSource::JsonInteger {
            column,
            key: key.to_string(),
        },
    }
}

fn catalog_prefixed_key(
    field: &str,
    prefix: &str,
    known_keys: &[&'static str],
) -> Result<Option<&'static str>, String> {
    let Some(key) = field.strip_prefix(prefix) else {
        return Ok(None);
    };
    known_keys
        .iter()
        .copied()
        .find(|candidate| *candidate == key)
        .map(Some)
        .ok_or_else(|| format!("unknown player metric: {field}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_deduplicates_and_keeps_requested_order() {
        let fields = parse_requested_fields(&[
            "position".to_string(),
            "attr.Acceleration".to_string(),
            "position".to_string(),
            "potential_role.goalkeeper_ip".to_string(),
        ])
        .expect("parse requested fields");

        assert_eq!(
            fields.iter().map(MetricField::id).collect::<Vec<_>>(),
            [
                "position",
                "attr.Acceleration",
                "potential_role.goalkeeper_ip",
            ]
        );
        assert!(fields[2].potential_role_id().is_some());
    }

    #[test]
    fn rejects_unsafe_or_unknown_ids() {
        assert!(MetricField::parse("attr.Acceleration;DROP").is_err());
        assert!(MetricField::parse("attr.NotARealMetric").is_err());
        assert!(MetricField::parse("hidden.NotARealMetric").is_err());
        assert!(MetricField::parse("personality.NotARealMetric").is_err());
        assert!(MetricField::parse("role.not_a_role").is_err());
    }

    #[test]
    fn resolves_validated_role_metrics_to_trusted_compact_columns() {
        let current =
            MetricField::parse("role.deep_lying_playmaker_ip").expect("parse current role metric");
        let potential = MetricField::parse("potential_role.line_holding_keeper_oop")
            .expect("parse potential role metric");

        assert_eq!(
            current.sql_expression("players"),
            "player_metrics.deep_lying_playmaker_ip"
        );
        assert_eq!(
            potential.sql_expression("players"),
            "player_metrics.potential_line_holding_keeper_oop"
        );
        assert_eq!(
            potential.potential_role_id(),
            Some("line_holding_keeper_oop")
        );
        assert!(MetricField::parse("ca")
            .expect("parse scalar metric")
            .potential_role_id()
            .is_none());
    }

    #[test]
    fn exposes_validated_current_role_sort_identity() {
        let metric = MetricField::parse("role.deep_lying_playmaker_ip").expect("parse role metric");

        assert_eq!(metric.current_role_id(), Some("deep_lying_playmaker_ip"));
        assert!(MetricField::parse("ca")
            .expect("parse scalar metric")
            .current_role_id()
            .is_none());
    }

    #[test]
    fn exposes_club_dna_as_a_nullable_integer_metric() {
        let metric = MetricField::parse("club_dna").expect("parse Club DNA metric");
        let expression =
            metric.sql_expression_with_club_dna("players", Some(ClubDnaSqlBindings::new(2, 3)));

        assert_eq!(metric.kind(), MetricValueKind::Integer);
        assert!(metric.is_club_dna());
        assert_eq!(
            metric.club_dna_sort_identity(Some(2)),
            Some(ClubDnaSortIdentity {
                definition_version: Some(2),
                score_model_version: SCORE_MODEL_VERSION,
            })
        );
        assert_eq!(
            metric.club_dna_sort_identity(None),
            Some(ClubDnaSortIdentity {
                definition_version: None,
                score_model_version: SCORE_MODEL_VERSION,
            })
        );
        assert!(MetricField::parse("ca")
            .expect("parse scalar metric")
            .club_dna_sort_identity(Some(2))
            .is_none());
        assert!(expression.contains("club_dna_scores"));
        assert!(expression.contains("cds.snapshot_id = players.snapshot_id"));
        assert!(expression.contains("cds.definition_version = ?2"));
        assert!(expression.contains("cds.score_model_version = ?3"));
    }

    #[test]
    fn bounds_raw_requested_field_input() {
        let fields = vec!["position".to_string(); MAX_REQUESTED_FIELDS + 1];

        assert!(parse_requested_fields(&fields).is_err());
    }

    #[test]
    fn accepts_a_hyphenated_moneyball_metric() {
        let metric = MetricField::parse_search("moneyball.np-xg").expect("moneyball metric");

        assert_eq!(metric.id(), "moneyball.np-xg");
        assert!(metric.reads_season_stats());
    }

    #[test]
    fn statistics_count_only_from_a_qualified_sample() {
        let metric = MetricField::parse_search("moneyball.np-xg").expect("moneyball metric");

        assert_eq!(
            metric.sql_expression("players"),
            "CASE WHEN (moneyball.minutes >= 450 AND moneyball.percentiles_json IS NOT NULL) \
             THEN moneyball.np_xg END"
        );
        assert_eq!(moneyball_statistic_sql("np-xg'); --"), "NULL");
    }

    #[test]
    fn playing_time_is_a_search_field_that_counts_a_missing_row_as_zero() {
        assert!(MetricField::parse("moneyball.minutes").is_err());
        let metric = MetricField::parse_search("moneyball.minutes").expect("playing time");

        assert_eq!(metric.kind(), MetricValueKind::Integer);
        assert_eq!(
            metric.sql_expression("players"),
            "COALESCE(moneyball.minutes, 0)"
        );
        assert!(metric.moneyball_key().is_none());
        assert!(metric.reads_season_stats());
    }

    #[test]
    fn accepts_only_valid_moneyball_role_fields_in_search() {
        let metric = MetricField::parse_search("moneyball_role.mc_central_midfielder_ip")
            .expect("Moneyball role field");

        assert_eq!(metric.kind(), MetricValueKind::Integer);
        assert_eq!(metric.moneyball_role_id(), Some("mc_central_midfielder_ip"));
        assert!(metric.is_moneyball_score());
        assert!(MetricField::parse_search("moneyball_role.not_a_role").is_err());
        assert!(MetricField::parse("moneyball_role.mc_central_midfielder_ip").is_err());
    }

    #[test]
    fn search_accepts_player_fields_and_season_fields_together() {
        let fields = parse_search_requested_fields(&[
            "attr.Acceleration".to_string(),
            "role.target_forward_ip".to_string(),
            "club_dna".to_string(),
            "moneyball.np-xg".to_string(),
            "moneyball.minutes".to_string(),
            "moneyball_role.mc_central_midfielder_ip".to_string(),
        ])
        .expect("merged fields");

        assert_eq!(fields.len(), 6);
    }

    #[test]
    fn tactic_fields_search_accepts_every_group_and_canonical_lane_as_integer_null() {
        for lane in TACTIC_LANE_IDS {
            for (prefix, expected_group) in [
                (TACTIC_CURRENT_PREFIX, TacticGroup::Current),
                (TACTIC_POTENTIAL_PREFIX, TacticGroup::Potential),
                (TACTIC_MONEYBALL_PREFIX, TacticGroup::Moneyball),
            ] {
                let field = format!("{prefix}{lane}");
                let parsed = MetricField::parse_search(&field)
                    .unwrap_or_else(|e| panic!("should parse {field}: {e}"));
                assert_eq!(parsed.id(), field);
                assert_eq!(parsed.kind(), MetricValueKind::Integer);
                assert!(parsed.is_tactic_field());
                assert_eq!(parsed.tactic_lane_id(), Some(lane));
                assert_eq!(parsed.tactic_group(), Some(expected_group.clone()));
                assert_eq!(
                    parsed.is_moneyball_score(),
                    expected_group == TacticGroup::Moneyball
                );
                assert_eq!(parsed.sql_expression("players"), "NULL");
                let req = parse_search_requested_fields(std::slice::from_ref(&field))
                    .expect("requested fields");
                assert_eq!(req[0].id(), field);
            }
        }
        let scalar = MetricField::parse("ca").expect("scalar");
        assert!(!scalar.is_tactic_field());
        assert!(scalar.tactic_lane_id().is_none());
    }

    #[test]
    fn tactic_fields_are_search_only_and_the_old_custom_prefix_is_gone() {
        for field in ["tactic_current.goalkeeper", "tactic_moneyball.goalkeeper"] {
            assert_eq!(
                MetricField::parse(field).unwrap_err(),
                format!("unknown player metric: {field}")
            );
        }
        assert!(parse_requested_fields(&["tactic_current.goalkeeper".to_string()]).is_err());
        assert!(crate::features::planner::squad::SquadSortField::parse(
            "tactic_current.goalkeeper"
        )
        .is_err());
        assert!(MetricField::parse_search("tactic_custom.goalkeeper").is_err());
        assert_eq!(
            MetricField::parse_search("tactic_moneyball.not_a_lane").unwrap_err(),
            "unknown player metric: tactic_moneyball.not_a_lane"
        );
    }

    #[test]
    fn tactic_fields_reject_invalid_suffix() {
        for field in [
            "tactic_current.",
            "tactic_current.Goalkeeper",
            "tactic_current.goalkeeper;DROP",
        ] {
            let err = MetricField::parse_search(field).unwrap_err();
            assert!(
                err.starts_with("unknown player metric:"),
                "unexpected {err}"
            );
            assert!(err.contains(field));
        }
    }

    #[test]
    fn tactic_fields_dedup_and_order_via_search_parser() {
        let fields = parse_search_requested_fields(&[
            "tactic_current.goalkeeper".to_string(),
            "ca".to_string(),
            "tactic_moneyball.goalkeeper".to_string(),
            "moneyball.np-xg".to_string(),
            "tactic_current.goalkeeper".to_string(),
        ])
        .expect("dedup");
        assert_eq!(
            fields.iter().map(MetricField::id).collect::<Vec<_>>(),
            [
                "tactic_current.goalkeeper",
                "ca",
                "tactic_moneyball.goalkeeper",
                "moneyball.np-xg"
            ]
        );
    }

    #[test]
    fn tactic_fields_bounded_by_max_requested_fields_via_search_parser() {
        let max = vec!["tactic_current.goalkeeper".to_string(); MAX_REQUESTED_FIELDS];
        assert!(parse_search_requested_fields(&max).is_ok());
        let too_many = vec!["tactic_current.goalkeeper".to_string(); MAX_REQUESTED_FIELDS + 1];
        assert!(parse_search_requested_fields(&too_many)
            .unwrap_err()
            .contains("exceeds maximum"));
        assert!(parse_requested_fields(&too_many).is_err());
    }

    #[test]
    fn tactic_lane_ids_match_planner_default() {
        let planner_ids = crate::features::planner::tactic::DEFAULT_LANE_IDS;
        assert_eq!(TACTIC_LANE_IDS.len(), planner_ids.len());
        for (left, right) in TACTIC_LANE_IDS.iter().zip(planner_ids.iter()) {
            assert_eq!(left, right);
        }
    }
}
