use std::collections::BTreeMap;

use serde::Serialize;
use tauri::State;

use crate::db::Db;

use super::query::{
    self, MoneyballProfile, MoneyballProfileState, MoneyballRoleContribution, MoneyballRoleScore,
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
