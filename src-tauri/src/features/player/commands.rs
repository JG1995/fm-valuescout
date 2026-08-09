use serde::Serialize;
use tauri::State;

use crate::db::Db;

use super::query::{self, PlayerDetail, PlayerRoleScore};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRoleScoreDto {
    pub role_id: String,
    pub display_name: String,
    pub phase: String,
    pub position_tags: Vec<String>,
    pub score: Option<i64>,
    pub potential_score: Option<i64>,
}

impl From<PlayerRoleScore> for PlayerRoleScoreDto {
    fn from(row: PlayerRoleScore) -> Self {
        Self {
            role_id: row.role_id,
            display_name: row.display_name,
            phase: row.phase,
            position_tags: row.position_tags,
            score: row.score,
            potential_score: row.potential_score,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerDetailDto {
    pub uid: i64,
    pub name: String,
    pub age: Option<i64>,
    pub birth_year: i64,
    pub birth_day_of_year: i64,
    pub nationalities: Vec<String>,
    pub height_cm: Option<i64>,
    pub preferred_foot: String,
    pub positions: std::collections::BTreeMap<String, i64>,
    pub attributes: std::collections::BTreeMap<String, Option<i64>>,
    pub potential_attributes: std::collections::BTreeMap<String, Option<i64>>,
    pub hidden_attributes: std::collections::BTreeMap<String, Option<i64>>,
    pub personality: std::collections::BTreeMap<String, Option<i64>>,
    pub weekly_wage_gbp: Option<i64>,
    pub contract_expiry_year: Option<i64>,
    pub contract_expiry_day_of_year: Option<i64>,
    pub transfer_listed: Option<bool>,
    pub loan_listed: Option<bool>,
    pub not_for_sale: Option<bool>,
    pub set_for_release: Option<bool>,
    pub market_value_gbp: Option<i64>,
    pub reputation_current: Option<i64>,
    pub reputation_world: Option<i64>,
    pub club: Option<String>,
    pub parent_club: Option<String>,
    pub on_loan: Option<bool>,
    pub division: Option<String>,
    pub team_level: Option<String>,
    pub ca: i64,
    pub pa: i64,
    pub role_scores: Vec<PlayerRoleScoreDto>,
}

impl From<PlayerDetail> for PlayerDetailDto {
    fn from(player: PlayerDetail) -> Self {
        Self {
            uid: player.uid,
            name: player.name,
            age: player.age,
            birth_year: player.birth_year,
            birth_day_of_year: player.birth_day_of_year,
            nationalities: player.nationalities,
            height_cm: player.height_cm,
            preferred_foot: player.preferred_foot,
            positions: player.positions,
            attributes: player.attributes,
            potential_attributes: player.potential_attributes,
            hidden_attributes: player.hidden_attributes,
            personality: player.personality,
            weekly_wage_gbp: player.weekly_wage_gbp,
            contract_expiry_year: player.contract_expiry_year,
            contract_expiry_day_of_year: player.contract_expiry_day_of_year,
            transfer_listed: player.transfer_listed,
            loan_listed: player.loan_listed,
            not_for_sale: player.not_for_sale,
            set_for_release: player.set_for_release,
            market_value_gbp: player.market_value_gbp,
            reputation_current: player.reputation_current,
            reputation_world: player.reputation_world,
            club: player.club,
            parent_club: player.parent_club,
            on_loan: player.on_loan,
            division: player.division,
            team_level: player.team_level,
            ca: player.ca,
            pa: player.pa,
            role_scores: player
                .role_scores
                .into_iter()
                .map(PlayerRoleScoreDto::from)
                .collect(),
        }
    }
}

/// Query key for frontend cache: `["player", uid]` — invalidate with snapshot/save keys
/// when Load Data or set_active_save runs (wired in a later commit).
#[tauri::command]
pub fn get_player(uid: i64, db: State<'_, Db>) -> Result<Option<PlayerDetailDto>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let player = query::get_player(&conn, uid)?;
    Ok(player.map(PlayerDetailDto::from))
}
