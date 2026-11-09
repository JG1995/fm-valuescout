use serde::{Deserialize, Serialize};
use tauri::State;

use crate::blocking;
use crate::db::result_cache::{parse_table_id, TableRegistry, TableTicket};
use crate::db::Db;

use super::filter::{self, FilterAst, FilterRule};
use super::query::{
    self, ComparisonPool, DynamicValue, GeneralOrderCache, PlayerSummary, SearchPlayersPage,
    SearchPlayersPageState, SearchPlayersRequest, SearchView, SortDir, SortField,
    DEFAULT_PAGE_LIMIT, MAX_PAGE_LIMIT,
};
use super::suggest::{
    self, PlayerSuggestHit, SuggestionCache, DEFAULT_SUGGEST_LIMIT, MAX_SUGGEST_LIMIT,
};

/// Search's in-memory results, kept as app state. Clones share them.
#[derive(Clone)]
pub struct SearchCaches {
    pub suggestions: SuggestionCache,
    /// General table orders: at most four queries, about 8 MB each at 1,000,000 players.
    pub general_orders: GeneralOrderCache,
}

impl Default for SearchCaches {
    fn default() -> Self {
        Self {
            suggestions: SuggestionCache::new(1),
            general_orders: GeneralOrderCache::new(4),
        }
    }
}

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
    pub current_club_uid: Option<i64>,
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
            current_club_uid: row.current_club_uid,
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
    pub state: &'static str,
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
            state: match page.state {
                SearchPlayersPageState::Ready => "ready",
                SearchPlayersPageState::NoCurrentSnapshot => "no_current_snapshot",
                SearchPlayersPageState::NoShortlist => "no_shortlist",
            },
        }
    }
}

#[tauri::command]
#[allow(clippy::too_many_arguments)] // Tauri deserializes this established flat query payload.
pub async fn search_players(
    offset: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_dir: Option<String>,
    filters: Option<Vec<FilterRuleInput>>,
    filter_combine: Option<String>,
    requested_fields: Option<Vec<String>>,
    search_view: Option<String>,
    comparison_pool: Option<String>,
    shortlist_only: Option<bool>,
    table_id: Option<String>,
    db: State<'_, Db>,
    caches: State<'_, SearchCaches>,
    registry: State<'_, TableRegistry>,
) -> Result<SearchPlayersPageDto, String> {
    let search = PlayerSearch::parse(
        offset,
        limit,
        sort_by,
        sort_dir,
        filters,
        filter_combine,
        requested_fields,
        search_view,
        comparison_pool,
        shortlist_only,
    )?;
    let table = table_id.as_deref().map(parse_table_id).transpose()?;
    // Moneyball keeps its uncached path, which nothing supersedes yet.
    let ticket = table
        .filter(|_| search.view == SearchView::General)
        .map(|table| registry.arrive(table, &query::general_query_fingerprint(&search.request())));
    let db = db.inner().clone();
    let orders = caches.general_orders.clone();
    let registry = registry.inner().clone();
    blocking::run(move || read_players_page(&db, &orders, &registry, ticket.as_ref(), &search))
        .await
}

/// A validated `search_players` request.
struct PlayerSearch {
    offset: usize,
    limit: usize,
    sort_by: SortField,
    sort_dir: SortDir,
    filter_ast: Option<FilterAst>,
    requested_fields: Vec<String>,
    view: SearchView,
    comparison_pool: ComparisonPool,
    shortlist_only: bool,
}

impl PlayerSearch {
    #[allow(clippy::too_many_arguments)]
    fn parse(
        offset: Option<u32>,
        limit: Option<u32>,
        sort_by: Option<String>,
        sort_dir: Option<String>,
        filters: Option<Vec<FilterRuleInput>>,
        filter_combine: Option<String>,
        requested_fields: Option<Vec<String>>,
        search_view: Option<String>,
        comparison_pool: Option<String>,
        shortlist_only: Option<bool>,
    ) -> Result<Self, String> {
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
        Ok(Self {
            offset: offset.unwrap_or(0) as usize,
            limit: limit
                .map(|value| value as usize)
                .unwrap_or(DEFAULT_PAGE_LIMIT)
                .clamp(1, MAX_PAGE_LIMIT),
            sort_by,
            sort_dir,
            filter_ast,
            requested_fields: requested_fields.unwrap_or_default(),
            view,
            comparison_pool,
            shortlist_only: shortlist_only.unwrap_or(false),
        })
    }

    fn request(&self) -> SearchPlayersRequest<'_> {
        SearchPlayersRequest {
            offset: self.offset,
            limit: self.limit,
            sort_by: self.sort_by.clone(),
            sort_dir: self.sort_dir,
            filter_ast: self.filter_ast.as_ref(),
            requested_fields: &self.requested_fields,
            view: self.view,
            comparison_pool: self.comparison_pool,
            shortlist_only: self.shortlist_only,
        }
    }
}

fn read_players_page(
    db: &Db,
    orders: &GeneralOrderCache,
    registry: &TableRegistry,
    ticket: Option<&TableTicket>,
    search: &PlayerSearch,
) -> Result<SearchPlayersPageDto, String> {
    let page = match search.view {
        SearchView::General => {
            query::search_general_players(db, orders, registry, ticket, search.request())?
        }
        SearchView::Moneyball => query::search_players_in_view(&*db.read()?, search.request())?,
    };
    Ok(SearchPlayersPageDto::from(page))
}

fn parse_search_view(value: Option<&str>) -> Result<SearchView, String> {
    match value.unwrap_or("general") {
        "general" => Ok(SearchView::General),
        "moneyball" => Ok(SearchView::Moneyball),
        other => Err(format!("unknown search view: {other}")),
    }
}

fn parse_search_sort_for_view(
    sort_by: Option<&str>,
    view: SearchView,
) -> Result<SortField, String> {
    match (sort_by, view) {
        (None, SearchView::General) => Ok(SortField::DEFAULT),
        (None, SearchView::Moneyball) => {
            SortField::parse_for_moneyball("moneyball.average_rating", true)
        }
        (Some(value), SearchView::General) => SortField::parse(value),
        (Some(value), SearchView::Moneyball) => SortField::parse_for_moneyball(value, true),
    }
}

fn parse_comparison_pool(value: Option<&str>, view: SearchView) -> Result<ComparisonPool, String> {
    match value.unwrap_or(match view {
        SearchView::General => "fullCsv",
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
pub async fn suggest_players(
    query: String,
    limit: Option<u32>,
    db: State<'_, Db>,
    caches: State<'_, SearchCaches>,
) -> Result<Vec<PlayerSuggestHitDto>, String> {
    let db = db.inner().clone();
    let cache = caches.suggestions.clone();
    blocking::run(move || read_suggestions(&db, &cache, &query, limit)).await
}

fn read_suggestions(
    db: &Db,
    cache: &SuggestionCache,
    query: &str,
    limit: Option<u32>,
) -> Result<Vec<PlayerSuggestHitDto>, String> {
    let limit = limit
        .map(|value| value as usize)
        .unwrap_or(DEFAULT_SUGGEST_LIMIT)
        .clamp(1, MAX_SUGGEST_LIMIT);
    let hits = suggest::suggest_players(db, cache, query, limit)?;
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
        assert!(parse_search_view(Some("shortlist")).is_err());
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
            parse_comparison_pool(None, SearchView::General),
            Ok(ComparisonPool::FullCsv)
        );
    }

    #[test]
    fn general_sort_helper_defaults_and_validates_through_general_path() {
        assert_eq!(
            parse_search_sort_for_view(None, SearchView::General),
            Ok(SortField::DEFAULT)
        );
        assert_eq!(SortField::DEFAULT, SortField::Ca);

        assert_eq!(
            parse_search_sort_for_view(Some("pa"), SearchView::General),
            Ok(SortField::Pa)
        );
        assert_eq!(
            parse_search_sort_for_view(Some("role.deep_lying_playmaker_ip"), SearchView::General),
            SortField::parse("role.deep_lying_playmaker_ip")
        );
        assert!(parse_search_sort_for_view(Some("attr.Acceleration"), SearchView::General).is_ok());
        assert!(parse_search_sort_for_view(
            Some("potential_role.goalkeeper_ip"),
            SearchView::General
        )
        .is_ok());
        assert!(parse_search_sort_for_view(Some("club_dna"), SearchView::General).is_ok());

        let moneyball_err = SortField::parse("moneyball.goals").unwrap_err();
        assert_eq!(
            parse_search_sort_for_view(Some("moneyball.goals"), SearchView::General).unwrap_err(),
            moneyball_err
        );
        let role_err = SortField::parse("moneyball_role.wbl_wbr_wing_back_ip").unwrap_err();
        assert_eq!(
            parse_search_sort_for_view(
                Some("moneyball_role.wbl_wbr_wing_back_ip"),
                SearchView::General
            )
            .unwrap_err(),
            role_err
        );
    }

    #[test]
    fn serializes_search_page_state_in_camel_case_snake_state_strings() {
        let ready = serde_json::to_value(SearchPlayersPageDto::from(SearchPlayersPage {
            players: Vec::new(),
            total: 0,
            state: SearchPlayersPageState::Ready,
        }))
        .expect("serialize ready page");
        assert_eq!(ready["state"], "ready");

        let no_shortlist = serde_json::to_value(SearchPlayersPageDto::from(SearchPlayersPage {
            players: Vec::new(),
            total: 0,
            state: SearchPlayersPageState::NoShortlist,
        }))
        .expect("serialize no shortlist page");
        assert_eq!(no_shortlist["state"], "no_shortlist");

        let no_snapshot = serde_json::to_value(SearchPlayersPageDto::from(SearchPlayersPage {
            players: Vec::new(),
            total: 0,
            state: SearchPlayersPageState::NoCurrentSnapshot,
        }))
        .expect("serialize no snapshot page");
        assert_eq!(no_snapshot["state"], "no_current_snapshot");
    }

    #[test]
    fn player_search_and_suggestions_answer_on_a_reader() {
        let (_temp_dir, db) = crate::db::open_with_golden_snapshot();

        let (general, moneyball, hits) = crate::db::read_while_writer_is_held(&db, |db| {
            let page = |view: &str, field: &str| {
                let search = PlayerSearch::parse(
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(vec![field.to_string()]),
                    Some(view.to_string()),
                    None,
                    None,
                )?;
                let registry = TableRegistry::default();
                let ticket = registry.arrive("search", "first page");
                read_players_page(
                    db,
                    &GeneralOrderCache::new(4),
                    &registry,
                    Some(&ticket),
                    &search,
                )
            };
            (
                page("general", "tactic_current.goalkeeper"),
                page("moneyball", "tactic_custom.goalkeeper"),
                read_suggestions(db, &SuggestionCache::new(1), "golden", None),
            )
        });

        let general = general.expect("General page on a reader");
        assert_eq!(general.state, "ready");
        assert_eq!(general.players.len(), 1);
        assert!(general.players[0]
            .dynamic_values
            .contains_key("tactic_current.goalkeeper"));
        assert_eq!(
            moneyball.expect("Moneyball page on a reader").state,
            "ready"
        );
        let hits = hits.expect("suggestions on a reader");
        assert_eq!(hits.iter().map(|hit| hit.uid).collect::<Vec<_>>(), vec![77]);
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
            current_club_uid: None,
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
