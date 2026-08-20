use std::collections::BTreeMap;

use serde::Serialize;
use tauri::State;

use crate::db::Db;

use super::query::{self, MoneyballProfile, MoneyballProfileState};

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
