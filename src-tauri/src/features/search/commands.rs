use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::Db;

use super::filter::{self, FilterRule};
use super::query::{
    self, ComparisonPool, DynamicValue, PlayerSuggestHit, PlayerSummary, SearchPlayersPage,
    SearchPlayersRequest, SearchView, SortDir, SortField, DEFAULT_PAGE_LIMIT,
    DEFAULT_SUGGEST_LIMIT, MAX_PAGE_LIMIT, MAX_SUGGEST_LIMIT,
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
    pub moneyball_percentiles: std::collections::BTreeMap<String, Option<u8>>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum DynamicValueDto {
    Integer(i64),
    Real(f64),
    Text(String),
}

impl From<DynamicValue> for DynamicValueDto {
    fn from(value: DynamicValue) -> Self {
        match value {
            DynamicValue::Integer(number) => Self::Integer(number),
            DynamicValue::Real(number) => Self::Real(number),
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
            moneyball_percentiles: row.moneyball_percentiles,
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
#[allow(clippy::too_many_arguments)] // Tauri deserializes this established flat query payload.
pub fn search_players(
    offset: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_dir: Option<String>,
    filters: Option<Vec<FilterRuleInput>>,
    filter_combine: Option<String>,
    requested_fields: Option<Vec<String>>,
    search_view: Option<String>,
    comparison_pool: Option<String>,
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
    let view = parse_search_view(search_view.as_deref())?;
    let comparison_pool = parse_comparison_pool(comparison_pool.as_deref(), view)?;
    let sort_by = parse_search_sort_for_view(sort_by.as_deref(), view)?;
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
    let requested_fields = requested_fields.unwrap_or_default();
    let page = query::search_players_in_view(
        &conn,
        SearchPlayersRequest {
            offset,
            limit,
            sort_by,
            sort_dir,
            filter_ast: filter_ast.as_ref(),
            requested_fields: &requested_fields,
            view,
            comparison_pool,
        },
    )?;
    Ok(SearchPlayersPageDto::from(page))
}

fn parse_search_view(value: Option<&str>) -> Result<SearchView, String> {
    match value.unwrap_or("general") {
        "general" => Ok(SearchView::General),
        "moneyball" => Ok(SearchView::Moneyball),
        "shortlist" => Ok(SearchView::Shortlist),
        other => Err(format!("unknown search view: {other}")),
    }
}

fn parse_search_sort_for_view(
    sort_by: Option<&str>,
    view: SearchView,
) -> Result<SortField, String> {
    match (sort_by, view) {
        (None, SearchView::General) => Ok(SortField::DEFAULT),
        (None, SearchView::Shortlist) => Ok(SortField::DEFAULT),
        (None, SearchView::Moneyball) => {
            SortField::parse_for_moneyball("moneyball.average_rating", true)
        }
        (Some(value), SearchView::General) => SortField::parse(value),
        (Some(value), SearchView::Shortlist) => SortField::parse(value),
        (Some(value), SearchView::Moneyball) => SortField::parse_for_moneyball(value, true),
    }
}

fn parse_comparison_pool(value: Option<&str>, view: SearchView) -> Result<ComparisonPool, String> {
    match value.unwrap_or(match view {
        SearchView::General | SearchView::Shortlist => "fullCsv",
        SearchView::Moneyball => "filtered",
    }) {
        "fullCsv" => Ok(ComparisonPool::FullCsv),
        "filtered" => Ok(ComparisonPool::Filtered),
        other => Err(format!("unknown Moneyball comparison pool: {other}")),
    }
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn parses_closed_search_view_and_comparison_pool_inputs() {
        assert_eq!(parse_search_view(None), Ok(SearchView::General));
        assert_eq!(
            parse_search_view(Some("moneyball")),
            Ok(SearchView::Moneyball)
        );
        assert_eq!(
            parse_search_view(Some("shortlist")),
            Ok(SearchView::Shortlist)
        );
        assert!(parse_search_view(Some("history")).is_err());

        assert_eq!(
            parse_comparison_pool(None, SearchView::Moneyball),
            Ok(ComparisonPool::Filtered)
        );
        assert_eq!(
            parse_comparison_pool(Some("fullCsv"), SearchView::Moneyball),
            Ok(ComparisonPool::FullCsv)
        );
        assert!(parse_comparison_pool(Some("allPlayers"), SearchView::Moneyball).is_err());
        assert_eq!(
            parse_comparison_pool(None, SearchView::Shortlist),
            Ok(ComparisonPool::FullCsv)
        );
        assert_eq!(
            parse_comparison_pool(Some("filtered"), SearchView::Shortlist),
            Ok(ComparisonPool::Filtered)
        );
        assert_eq!(
            parse_comparison_pool(None, SearchView::General),
            Ok(ComparisonPool::FullCsv)
        );
    }

    #[test]
    fn shortlist_sort_helper_defaults_and_validates_through_general_path() {
        assert_eq!(
            parse_search_sort_for_view(None, SearchView::Shortlist),
            Ok(SortField::DEFAULT)
        );
        assert_eq!(
            parse_search_sort_for_view(None, SearchView::General),
            Ok(SortField::DEFAULT)
        );
        assert_eq!(SortField::DEFAULT, SortField::Ca);

        assert_eq!(
            parse_search_sort_for_view(Some("pa"), SearchView::Shortlist),
            Ok(SortField::Pa)
        );
        assert_eq!(
            parse_search_sort_for_view(Some("role.deep_lying_playmaker_ip"), SearchView::Shortlist),
            SortField::parse("role.deep_lying_playmaker_ip")
        );
        assert!(
            parse_search_sort_for_view(Some("attr.Acceleration"), SearchView::Shortlist).is_ok()
        );
        assert!(parse_search_sort_for_view(
            Some("potential_role.goalkeeper_ip"),
            SearchView::Shortlist
        )
        .is_ok());
        assert!(parse_search_sort_for_view(Some("club_dna"), SearchView::Shortlist).is_ok());

        let moneyball_err = SortField::parse("moneyball.goals").unwrap_err();
        assert_eq!(
            parse_search_sort_for_view(Some("moneyball.goals"), SearchView::Shortlist).unwrap_err(),
            moneyball_err
        );
        let role_err = SortField::parse("moneyball_role.wbl_wbr_wing_back_ip").unwrap_err();
        assert_eq!(
            parse_search_sort_for_view(
                Some("moneyball_role.wbl_wbr_wing_back_ip"),
                SearchView::Shortlist
            )
            .unwrap_err(),
            role_err
        );
    }

    #[test]
    fn serializes_real_values_and_moneyball_percentiles_in_camel_case() {
        let dto = PlayerSummaryDto::from(PlayerSummary {
            uid: 7,
            name: "Scored player".to_string(),
            age: None,
            birth_year: 2000,
            birth_day_of_year: 1,
            nationalities: Vec::new(),
            club: None,
            division: None,
            ca: 100,
            pa: 120,
            market_value_gbp: None,
            dynamic_values: BTreeMap::from([(
                "moneyball.np-xg".to_string(),
                Some(DynamicValue::Real(0.75)),
            )]),
            moneyball_percentiles: BTreeMap::from([("moneyball.np-xg".to_string(), Some(83))]),
        });

        let value = serde_json::to_value(dto).expect("serialize DTO");
        assert_eq!(value["dynamicValues"]["moneyball.np-xg"], 0.75);
        assert_eq!(value["moneyballPercentiles"]["moneyball.np-xg"], 83);
    }
}
