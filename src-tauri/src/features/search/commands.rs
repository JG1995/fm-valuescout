use serde::Serialize;
use tauri::State;

use crate::db::Db;

use super::query::{
    self, PlayerSummary, SearchPlayersPage, SortDir, SortField, DEFAULT_PAGE_LIMIT, MAX_PAGE_LIMIT,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSummaryDto {
    pub uid: i64,
    pub name: String,
    pub age: Option<i64>,
    pub birth_year: i64,
    pub birth_day_of_year: i64,
    pub nationalities: Vec<String>,
    pub club: Option<String>,
    pub division: Option<String>,
    pub ca: i64,
    pub pa: i64,
    pub market_value_gbp: Option<i64>,
}

impl From<PlayerSummary> for PlayerSummaryDto {
    fn from(row: PlayerSummary) -> Self {
        Self {
            uid: row.uid,
            name: row.name,
            age: row.age,
            birth_year: row.birth_year,
            birth_day_of_year: row.birth_day_of_year,
            nationalities: row.nationalities,
            club: row.club,
            division: row.division,
            ca: row.ca,
            pa: row.pa,
            market_value_gbp: row.market_value_gbp,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPlayersPageDto {
    pub players: Vec<PlayerSummaryDto>,
    pub total: i64,
}

impl From<SearchPlayersPage> for SearchPlayersPageDto {
    fn from(page: SearchPlayersPage) -> Self {
        Self {
            players: page
                .players
                .into_iter()
                .map(PlayerSummaryDto::from)
                .collect(),
            total: page.total,
        }
    }
}

#[tauri::command]
pub fn search_players(
    offset: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_dir: Option<String>,
    db: State<'_, Db>,
) -> Result<SearchPlayersPageDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let offset = offset.unwrap_or(0) as usize;
    let limit = limit
        .map(|value| value as usize)
        .unwrap_or(DEFAULT_PAGE_LIMIT)
        .clamp(1, MAX_PAGE_LIMIT);
    let sort_by = match sort_by.as_deref() {
        None => SortField::DEFAULT,
        Some(value) => SortField::parse(value)?,
    };
    let sort_dir = match sort_dir.as_deref() {
        None => SortDir::DEFAULT,
        Some(value) => SortDir::parse(value)?,
    };
    let page = query::search_players(&conn, offset, limit, sort_by, sort_dir)?;
    Ok(SearchPlayersPageDto::from(page))
}
