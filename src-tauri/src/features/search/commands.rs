use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::Db;

use super::filter::{self, FilterRule};
use super::query::{
    self, DynamicValue, PlayerSuggestHit, PlayerSummary, SearchPlayersPage, SortDir, SortField,
    DEFAULT_PAGE_LIMIT, DEFAULT_SUGGEST_LIMIT, MAX_PAGE_LIMIT, MAX_SUGGEST_LIMIT,
};

#[derive(Deserialize)]
pub struct FilterRuleInput {
    pub field: String,
    pub op: String,
    pub value: serde_json::Value,
}

impl TryFrom<FilterRuleInput> for FilterRule {
    type Error = String;

    fn try_from(input: FilterRuleInput) -> Result<Self, Self::Error> {
        Ok(FilterRule {
            field: input.field,
            op: input.op,
            value: filter::filter_value_from_json(input.value)?,
        })
    }
}

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
    pub dynamic_values: std::collections::BTreeMap<String, Option<DynamicValueDto>>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum DynamicValueDto {
    Integer(i64),
    Text(String),
}

impl From<DynamicValue> for DynamicValueDto {
    fn from(value: DynamicValue) -> Self {
        match value {
            DynamicValue::Integer(number) => Self::Integer(number),
            DynamicValue::Text(text) => Self::Text(text),
        }
    }
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
            dynamic_values: row
                .dynamic_values
                .into_iter()
                .map(|(key, value)| (key, value.map(DynamicValueDto::from)))
                .collect(),
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
    filters: Option<Vec<FilterRuleInput>>,
    filter_combine: Option<String>,
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
    let filter_ast = match filters {
        None => None,
        Some(rules) => {
            let parsed_rules = rules
                .into_iter()
                .map(FilterRule::try_from)
                .collect::<Result<Vec<_>, _>>()?;
            Some(filter::parse_filter_ast(
                parsed_rules,
                filter_combine.as_deref(),
            )?)
        }
    };
    let page = query::search_players(&conn, offset, limit, sort_by, sort_dir, filter_ast.as_ref())?;
    Ok(SearchPlayersPageDto::from(page))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSuggestHitDto {
    pub uid: i64,
    pub name: String,
    pub ca: i64,
}

impl From<PlayerSuggestHit> for PlayerSuggestHitDto {
    fn from(hit: PlayerSuggestHit) -> Self {
        Self {
            uid: hit.uid,
            name: hit.name,
            ca: hit.ca,
        }
    }
}

#[tauri::command]
pub fn suggest_players(
    query: String,
    limit: Option<u32>,
    db: State<'_, Db>,
) -> Result<Vec<PlayerSuggestHitDto>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let limit = limit
        .map(|value| value as usize)
        .unwrap_or(DEFAULT_SUGGEST_LIMIT)
        .clamp(1, MAX_SUGGEST_LIMIT);
    let hits = query::suggest_players(&conn, &query, limit)?;
    Ok(hits.into_iter().map(PlayerSuggestHitDto::from).collect())
}
