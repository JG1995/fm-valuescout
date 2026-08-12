use std::collections::HashSet;

use rusqlite::Row;

use crate::features::scoring::catalog::{all_roles, DUMP_ATTRIBUTE_KEYS};

use super::potential_cache::PROJECTION_MODEL_VERSION;

/// FM26 dump position keys (bridge `PositionEntries`).
pub const POSITION_KEYS: &[&str] = &[
    "GK", "SW", "DL", "DC", "DR", "DM", "ML", "MC", "MR", "AML", "AMC", "AMR", "ST", "WBL", "WBR",
];

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

/// The frontend catalog has fewer than 256 selectable metrics. Keep direct IPC
/// input bounded before it can expand a row projection.
pub const MAX_REQUESTED_FIELDS: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricValueKind {
    Integer,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MetricSource {
    Column(&'static str),
    JsonInteger { column: &'static str, key: String },
    Position,
    CurrentRole { role_id: &'static str },
    PotentialRole { role_id: &'static str },
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynamicValue {
    Integer(i64),
    Text(String),
}

impl MetricField {
    pub fn parse(field: &str) -> Result<Self, String> {
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
            return Ok(Self {
                id: field.to_string(),
                kind: MetricValueKind::Integer,
                source: MetricSource::CurrentRole {
                    role_id: catalog_role_id(role_id)?,
                },
            });
        }
        if let Some(role_id) = field.strip_prefix("potential_role.") {
            return Ok(Self {
                id: field.to_string(),
                kind: MetricValueKind::Integer,
                source: MetricSource::PotentialRole {
                    role_id: catalog_role_id(role_id)?,
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
            MetricSource::PotentialRole { role_id } => Some(role_id),
            _ => None,
        }
    }

    /// Returns a trusted SQLite expression relative to the supplied player-table alias.
    pub fn sql_expression(&self, player_alias: &str) -> String {
        match &self.source {
            MetricSource::Column(column) => format!("{player_alias}.{column}"),
            MetricSource::JsonInteger { column, key } => {
                format!("json_extract({player_alias}.{column}, '$.{key}')")
            }
            MetricSource::Position => format!(
                "COALESCE((SELECT group_concat(position_key, ', ') FROM (SELECT entry.key AS position_key FROM json_each({player_alias}.positions_json) AS entry ORDER BY CAST(entry.value AS INTEGER) DESC, entry.key ASC)), '')"
            ),
            MetricSource::CurrentRole { role_id } => format!(
                "(SELECT prs.score FROM player_role_scores prs WHERE prs.snapshot_id = {player_alias}.snapshot_id AND prs.uid = {player_alias}.uid AND prs.role_id = '{role_id}')"
            ),
            MetricSource::PotentialRole { role_id } => format!(
                "(SELECT pprs.score FROM player_potential_role_scores pprs WHERE pprs.snapshot_id = {player_alias}.snapshot_id AND pprs.uid = {player_alias}.uid AND pprs.role_id = '{role_id}' AND pprs.projection_model_version = {PROJECTION_MODEL_VERSION})"
            ),
        }
    }

    pub fn sql_sort_expression(&self, player_alias: &str) -> String {
        let expression = self.sql_expression(player_alias);
        match self.kind {
            MetricValueKind::Integer => expression,
            MetricValueKind::Text => format!("{expression} COLLATE NOCASE"),
        }
    }
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

pub fn read_dynamic_value(
    row: &Row<'_>,
    index: usize,
    metric: &MetricField,
) -> rusqlite::Result<Option<DynamicValue>> {
    match metric.kind() {
        MetricValueKind::Integer => row
            .get::<_, Option<i64>>(index)
            .map(|value| value.map(DynamicValue::Integer)),
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
    fn bounds_raw_requested_field_input() {
        let fields = vec!["position".to_string(); MAX_REQUESTED_FIELDS + 1];

        assert!(parse_requested_fields(&fields).is_err());
    }
}
