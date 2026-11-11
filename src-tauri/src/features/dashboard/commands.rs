use serde::Serialize;
use tauri::State;

use crate::blocking;
use crate::db::Db;
use crate::features::snapshot::service::active_save_id;

use super::signings::{
    self, IncumbentSource, LineupPlayer, LineupSlot, OtherLane, Recommendation, ScoreBasis,
    ScoreMode, Signings, SigningsRequest,
};
use super::squad::{self, ExpiringContract, ExpiringContracts, Prospect};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineupPlayerDto {
    pub uid: i64,
    pub name: String,
    pub score: u8,
}

impl From<LineupPlayer> for LineupPlayerDto {
    fn from(value: LineupPlayer) -> Self {
        Self {
            uid: value.uid,
            name: value.name,
            score: value.score,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncumbentDto {
    pub uid: i64,
    pub name: String,
    pub score: u8,
    /// `planner` for the Planner's first-string player, `best_fit` for an allocated one.
    pub source: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineupSlotDto {
    pub lane_id: String,
    pub ip_position: String,
    pub ip_role_id: String,
    pub oop_position: String,
    pub oop_role_id: String,
    /// `attributes`, `moneyball_custom`, `moneyball_roles`, or `unavailable`.
    pub score_basis: &'static str,
    pub incumbent: Option<IncumbentDto>,
    pub cover: Option<LineupPlayerDto>,
    pub unscored_planner_player: Option<String>,
}

impl From<LineupSlot> for LineupSlotDto {
    fn from(value: LineupSlot) -> Self {
        Self {
            lane_id: value.lane.lane_id,
            ip_position: value.lane.ip_position,
            ip_role_id: value.lane.ip_role_id,
            oop_position: value.lane.oop_position,
            oop_role_id: value.lane.oop_role_id,
            score_basis: match value.score_basis {
                ScoreBasis::Attributes => "attributes",
                ScoreBasis::MoneyballCustom => "moneyball_custom",
                ScoreBasis::MoneyballRoles => "moneyball_roles",
                ScoreBasis::Unavailable => "unavailable",
            },
            incumbent: value.incumbent.map(|(player, source)| IncumbentDto {
                uid: player.uid,
                name: player.name,
                score: player.score,
                source: match source {
                    IncumbentSource::Planner => "planner",
                    IncumbentSource::BestFit => "best_fit",
                },
            }),
            cover: value.cover.map(Into::into),
            unscored_planner_player: value.unscored_planner_player,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OtherLaneDto {
    pub lane_id: String,
    pub improvement: u8,
}

impl From<OtherLane> for OtherLaneDto {
    fn from(value: OtherLane) -> Self {
        Self {
            lane_id: value.lane_id,
            improvement: value.improvement,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationDto {
    pub uid: i64,
    pub name: String,
    pub age: Option<i64>,
    pub club: Option<String>,
    pub club_uid: Option<i64>,
    pub market_value_gbp: Option<i64>,
    pub minutes: Option<i64>,
    pub lane_id: String,
    pub score: u8,
    pub improvement: u8,
    pub other_lanes: Vec<OtherLaneDto>,
}

impl From<Recommendation> for RecommendationDto {
    fn from(value: Recommendation) -> Self {
        Self {
            uid: value.uid,
            name: value.name,
            age: value.age,
            club: value.club,
            club_uid: value.club_uid,
            market_value_gbp: value.market_value_gbp,
            minutes: value.minutes,
            lane_id: value.lane_id,
            score: value.score,
            improvement: value.improvement,
            other_lanes: value.other_lanes.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSigningsDto {
    pub tactic_is_default: bool,
    pub scores_available: bool,
    pub slots: Vec<LineupSlotDto>,
    pub shortlisted_players: usize,
    pub total_recommendations: usize,
    pub recommendations: Vec<RecommendationDto>,
}

impl From<Signings> for DashboardSigningsDto {
    fn from(value: Signings) -> Self {
        Self {
            tactic_is_default: value.tactic_is_default,
            scores_available: value.scores_available,
            slots: value.slots.into_iter().map(Into::into).collect(),
            shortlisted_players: value.shortlisted_players,
            total_recommendations: value.total_recommendations,
            recommendations: value.recommendations.into_iter().map(Into::into).collect(),
        }
    }
}

/// The active save's starting XI and recommended signings in a score mode (`general` or
/// `moneyball`), or `null` without a current snapshot or a managed club.
#[tauri::command]
pub async fn get_dashboard_signings(
    mode: String,
    shortlist_id: Option<i64>,
    lane_id: Option<String>,
    db: State<'_, Db>,
) -> Result<Option<DashboardSigningsDto>, String> {
    let db = db.inner().clone();
    blocking::run(move || read_dashboard_signings(&db, &mode, shortlist_id, lane_id.as_deref()))
        .await
}

fn read_dashboard_signings(
    db: &Db,
    mode: &str,
    shortlist_id: Option<i64>,
    lane_id: Option<&str>,
) -> Result<Option<DashboardSigningsDto>, String> {
    let request = SigningsRequest {
        mode: ScoreMode::parse(mode)?,
        shortlist_id,
        lane_id,
    };
    let conn = db.read()?;
    Ok(signings::get_signings(&conn, active_save_id(&conn)?, request)?.map(Into::into))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpiringContractDto {
    pub uid: i64,
    pub name: String,
    pub age: Option<i64>,
    pub ca: i64,
    pub market_value_gbp: Option<i64>,
    pub weekly_wage_gbp: Option<i64>,
    pub expiry_date: String,
    pub days_remaining: Option<i64>,
}

impl From<ExpiringContract> for ExpiringContractDto {
    fn from(value: ExpiringContract) -> Self {
        Self {
            uid: value.uid,
            name: value.name,
            age: value.age,
            ca: value.ca,
            market_value_gbp: value.market_value_gbp,
            weekly_wage_gbp: value.weekly_wage_gbp,
            expiry_date: value.expiry_date,
            days_remaining: value.days_remaining,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpiringContractsDto {
    pub window_months: Option<u32>,
    pub total: usize,
    pub players: Vec<ExpiringContractDto>,
}

impl From<ExpiringContracts> for ExpiringContractsDto {
    fn from(value: ExpiringContracts) -> Self {
        Self {
            window_months: value.window_months,
            total: value.total,
            players: value.players.into_iter().map(Into::into).collect(),
        }
    }
}

/// The managed club's contracts that end soonest, or `null` without a current snapshot or a
/// managed club.
#[tauri::command]
pub async fn get_dashboard_expiring_contracts(
    db: State<'_, Db>,
) -> Result<Option<ExpiringContractsDto>, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let conn = db.read()?;
        Ok(squad::get_expiring_contracts(&conn, active_save_id(&conn)?)?.map(Into::into))
    })
    .await
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProspectDto {
    pub uid: i64,
    pub name: String,
    pub age: i64,
    pub lane_id: String,
    pub ip_position: String,
    pub ip_role_id: String,
    pub oop_position: String,
    pub oop_role_id: String,
    pub current_score: u8,
    pub potential_score: u8,
}

impl From<Prospect> for ProspectDto {
    fn from(value: Prospect) -> Self {
        Self {
            uid: value.uid,
            name: value.name,
            age: value.age,
            lane_id: value.lane.lane_id,
            ip_position: value.lane.ip_position,
            ip_role_id: value.lane.ip_role_id,
            oop_position: value.lane.oop_position,
            oop_role_id: value.lane.oop_role_id,
            current_score: value.current_score,
            potential_score: value.potential_score,
        }
    }
}

/// The managed club's young players with the highest potential in the tactic, or `null`
/// without a current snapshot or a managed club.
#[tauri::command]
pub async fn get_dashboard_prospects(
    db: State<'_, Db>,
) -> Result<Option<Vec<ProspectDto>>, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let conn = db.read()?;
        Ok(squad::get_prospects(&conn, active_save_id(&conn)?)?
            .map(|prospects| prospects.into_iter().map(Into::into).collect()))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::managed_club::service::set_managed_club;

    #[test]
    fn signings_answer_on_a_reader_without_storing_planner_defaults() {
        let (_temp_dir, db) = crate::db::open_with_golden_snapshot();
        {
            let conn = db.0.lock().expect("lock writer");
            let save_id = active_save_id(&conn).expect("active save");
            set_managed_club(&conn, save_id, "Loan FC", Some(1000))
                .expect("configure managed club");
        }

        let signings = crate::db::read_while_writer_is_held(&db, |db| {
            read_dashboard_signings(db, "general", None, None)
        })
        .expect("signings on a reader")
        .expect("signings for a managed club");

        assert!(signings.tactic_is_default);
        assert_eq!(signings.slots.len(), 11);
        assert_eq!(
            read_dashboard_signings(&db, "potential", None, None).map(|_| ()),
            Err("unknown dashboard score mode: potential".to_string())
        );
    }
}
