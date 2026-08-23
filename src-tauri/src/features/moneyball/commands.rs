use std::collections::BTreeMap;

use serde::Serialize;
use tauri::State;

use crate::db::Db;

use super::query::{
    self, MoneyballComparisonBasis, MoneyballProfile, MoneyballProfileState,
    MoneyballRoleContribution, MoneyballRoleScore,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyballProfileDto {
    pub state: &'static str,
    pub asking_price_kind: Option<String>,
    pub asking_price_lower_eur: Option<i64>,
    pub asking_price_upper_eur: Option<i64>,
    pub starts: Option<i64>,
    pub substitute_appearances: Option<i64>,
    pub minutes: Option<i64>,
    pub statistics: Option<BTreeMap<String, Option<f64>>>,
    pub percentiles: Option<BTreeMap<String, Option<u8>>>,
    pub role_catalog_version: Option<u32>,
    pub role_scores: Option<Vec<MoneyballRoleScoreDto>>,
    pub comparison_basis: Option<MoneyballComparisonBasisDto>,
}

#[derive(Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum MoneyballComparisonBasisDto {
    Available {
        natural_positions: Vec<String>,
        comparison_player_count: usize,
    },
    UnavailableNoNaturalPosition,
}

impl From<MoneyballComparisonBasis> for MoneyballComparisonBasisDto {
    fn from(basis: MoneyballComparisonBasis) -> Self {
        match basis {
            MoneyballComparisonBasis::Available {
                natural_positions,
                comparison_player_count,
            } => Self::Available {
                natural_positions,
                comparison_player_count,
            },
            MoneyballComparisonBasis::UnavailableNoNaturalPosition => {
                Self::UnavailableNoNaturalPosition
            }
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyballRoleScoreDto {
    pub role_id: String,
    pub display_name: String,
    pub phase: String,
    pub position_family: String,
    pub position_tags: Vec<String>,
    pub score: Option<u8>,
    pub contributions: Vec<MoneyballRoleContributionDto>,
}

impl From<MoneyballRoleScore> for MoneyballRoleScoreDto {
    fn from(role: MoneyballRoleScore) -> Self {
        Self {
            role_id: role.role_id,
            display_name: role.display_name,
            phase: role.phase,
            position_family: role.position_family,
            position_tags: role.position_tags,
            score: role.score,
            contributions: role
                .contributions
                .into_iter()
                .map(MoneyballRoleContributionDto::from)
                .collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyballRoleContributionDto {
    pub metric_key: String,
    pub source_label: String,
    pub weight: f64,
    pub direction: String,
    pub percentile: Option<u8>,
    pub weighted_contribution: Option<f64>,
}

impl From<MoneyballRoleContribution> for MoneyballRoleContributionDto {
    fn from(contribution: MoneyballRoleContribution) -> Self {
        Self {
            metric_key: contribution.metric_key,
            source_label: contribution.source_label,
            weight: contribution.weight,
            direction: contribution.direction,
            percentile: contribution.percentile,
            weighted_contribution: contribution.weighted_contribution,
        }
    }
}

impl From<MoneyballProfile> for MoneyballProfileDto {
    fn from(profile: MoneyballProfile) -> Self {
        Self {
            state: match profile.state {
                MoneyballProfileState::NoData => "noData",
                MoneyballProfileState::NeedsReimport => "needsReimport",
                MoneyballProfileState::Ready => "ready",
            },
            asking_price_kind: profile.asking_price_kind,
            asking_price_lower_eur: profile.asking_price_lower_eur,
            asking_price_upper_eur: profile.asking_price_upper_eur,
            starts: profile.starts,
            substitute_appearances: profile.substitute_appearances,
            minutes: profile.minutes,
            statistics: profile.statistics,
            percentiles: profile.percentiles,
            role_catalog_version: profile.role_catalog_version,
            role_scores: profile
                .role_scores
                .map(|roles| roles.into_iter().map(MoneyballRoleScoreDto::from).collect()),
            comparison_basis: profile
                .comparison_basis
                .map(MoneyballComparisonBasisDto::from),
        }
    }
}

/// Reads the active save's current Moneyball cohort for one current player UID.
#[tauri::command]
pub fn get_player_moneyball(
    uid: i64,
    db: State<'_, Db>,
) -> Result<Option<MoneyballProfileDto>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    query::get_player_moneyball(&conn, uid).map(|profile| profile.map(MoneyballProfileDto::from))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::MoneyballProfileDto;
    use crate::features::moneyball::query::{
        MoneyballComparisonBasis, MoneyballProfile, MoneyballProfileState,
    };

    fn ready_profile(comparison_basis: MoneyballComparisonBasis) -> MoneyballProfile {
        MoneyballProfile {
            state: MoneyballProfileState::Ready,
            asking_price_kind: Some("single".to_owned()),
            asking_price_lower_eur: Some(12_000_000),
            asking_price_upper_eur: None,
            starts: Some(18),
            substitute_appearances: Some(3),
            minutes: Some(1_500),
            statistics: Some(BTreeMap::new()),
            percentiles: Some(BTreeMap::new()),
            role_catalog_version: Some(1),
            role_scores: Some(Vec::new()),
            comparison_basis: Some(comparison_basis),
        }
    }

    #[test]
    fn serializes_available_comparison_basis_with_camel_case_fields() {
        let dto = MoneyballProfileDto::from(ready_profile(MoneyballComparisonBasis::Available {
            natural_positions: vec!["AMR".to_owned(), "ST".to_owned()],
            comparison_player_count: 12,
        }));

        let value = serde_json::to_value(dto).expect("serialize profile");

        assert_eq!(
            value["comparisonBasis"],
            json!({
                "kind": "available",
                "naturalPositions": ["AMR", "ST"],
                "comparisonPlayerCount": 12,
            })
        );
        assert!(value["comparisonBasis"].get("natural_positions").is_none());
        assert!(value["comparisonBasis"]
            .get("comparison_player_count")
            .is_none());
    }

    #[test]
    fn serializes_unavailable_comparison_basis_without_neutral_scores() {
        let mut profile = ready_profile(MoneyballComparisonBasis::UnavailableNoNaturalPosition);
        profile.percentiles = None;
        profile.role_catalog_version = None;
        profile.role_scores = None;

        let dto = MoneyballProfileDto::from(profile);
        let value = serde_json::to_value(dto).expect("serialize profile");

        assert_eq!(
            value["comparisonBasis"],
            json!({ "kind": "unavailableNoNaturalPosition" })
        );
        assert_eq!(value["percentiles"], serde_json::Value::Null);
        assert_eq!(value["roleCatalogVersion"], serde_json::Value::Null);
        assert_eq!(value["roleScores"], serde_json::Value::Null);
    }
}
