use serde::{Deserialize, Serialize};
use tauri::State;

use crate::blocking;
use crate::db::result_cache::{parse_table_id, TableRegistry, TableTicket};
use crate::db::Db;
use crate::features::moneyball::comparison::ComparisonGroupsCache;

use super::filter::{self, FilterAst, FilterRule};
use super::query::{
    self, Comparison, DynamicValue, PlayerOrderCache, PlayerSummary, SearchPlayersPage,
    SearchPlayersPageState, SearchPlayersRequest, SortDir, SortField, DEFAULT_PAGE_LIMIT,
    MAX_PAGE_LIMIT,
};
use super::suggest::{
    self, PlayerSuggestHit, SuggestionCache, DEFAULT_SUGGEST_LIMIT, MAX_SUGGEST_LIMIT,
};

/// Search's in-memory results, kept as app state. Clones share them.
#[derive(Clone)]
pub struct SearchCaches {
    pub suggestions: SuggestionCache,
    /// Table orders: at most four queries, about 8 MB each at 1,000,000 players.
    pub orders: PlayerOrderCache,
}

impl Default for SearchCaches {
    fn default() -> Self {
        Self {
            suggestions: SuggestionCache::new(1),
            orders: PlayerOrderCache::new(4),
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
    /// The in-game date of the kept statistics the player's season values come from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moneyball_stats_date: Option<String>,
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
            moneyball_stats_date: row.moneyball_stats_date,
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
    comparison: Option<String>,
    shortlist_id: Option<i64>,
    table_id: Option<String>,
    db: State<'_, Db>,
    caches: State<'_, SearchCaches>,
    groups: State<'_, ComparisonGroupsCache>,
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
        comparison,
        shortlist_id,
    )?;
    let table = table_id.as_deref().map(parse_table_id).transpose()?;
    let ticket = table
        .map(|table| registry.arrive(table, &query::search_query_fingerprint(&search.request())));
    let db = db.inner().clone();
    let caches = caches.inner().clone();
    let groups = groups.inner().clone();
    let registry = registry.inner().clone();
    blocking::run(move || {
        read_players_page(&db, &caches, &groups, &registry, ticket.as_ref(), &search)
    })
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
    comparison: Comparison,
    shortlist_id: Option<i64>,
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
        comparison: Option<String>,
        shortlist_id: Option<i64>,
    ) -> Result<Self, String> {
        let comparison = comparison
            .as_deref()
            .map_or(Ok(Comparison::Own), Comparison::parse)?;
        let sort_by = sort_by
            .as_deref()
            .map_or(Ok(SortField::DEFAULT), SortField::parse)?;
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
            comparison,
            shortlist_id,
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
            comparison: self.comparison,
            shortlist_id: self.shortlist_id,
        }
    }
}

fn read_players_page(
    db: &Db,
    caches: &SearchCaches,
    groups: &ComparisonGroupsCache,
    registry: &TableRegistry,
    ticket: Option<&TableTicket>,
    search: &PlayerSearch,
) -> Result<SearchPlayersPageDto, String> {
    let page = query::search_players(
        db,
        &caches.orders,
        groups,
        registry,
        ticket,
        search.request(),
    )?;
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
    use crate::features::moneyball::comparison::{ComparisonGroup, PositionFamily};

    #[test]
    fn parses_closed_comparison_inputs() {
        assert_eq!(Comparison::parse("own"), Ok(Comparison::Own));
        assert_eq!(
            Comparison::parse("central_defender"),
            Ok(Comparison::Group(ComparisonGroup::family(
                PositionFamily::CentralDefender
            )))
        );
        assert_eq!(
            Comparison::parse("winger+wide_midfielder"),
            Ok(Comparison::Group(
                ComparisonGroup::slot("AML", "ML").expect("slot group")
            ))
        );
        for invalid in [
            "filtered",
            "DC",
            "winger+winger",
            "winger+",
            "winger+wide_midfielder+striker",
        ] {
            assert!(Comparison::parse(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn sorts_default_to_ca_and_accept_every_search_field() {
        let parse = |sort: Option<&str>| {
            PlayerSearch::parse(
                None,
                None,
                sort.map(str::to_string),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .map(|search| search.sort_by)
        };
        assert_eq!(parse(None), Ok(SortField::Ca));
        for field in [
            "pa",
            "role.deep_lying_playmaker_ip",
            "attr.Acceleration",
            "potential_role.goalkeeper_ip",
            "club_dna",
            "moneyball.goals",
            "moneyball.minutes",
            "moneyball_role.wbl_wbr_wing_back_ip",
            "tactic_moneyball.goalkeeper",
        ] {
            assert!(parse(Some(field)).is_ok(), "{field}");
        }
        assert!(parse(Some("tactic_custom.goalkeeper")).is_err());
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

        let (page, hits) = crate::db::read_while_writer_is_held(&db, |db| {
            let page = |fields: &[&str]| {
                let search = PlayerSearch::parse(
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(fields.iter().map(|field| field.to_string()).collect()),
                    None,
                    None,
                )?;
                let registry = TableRegistry::default();
                let ticket = registry.arrive("search", "first page");
                read_players_page(
                    db,
                    &SearchCaches::default(),
                    &ComparisonGroupsCache::default(),
                    &registry,
                    Some(&ticket),
                    &search,
                )
            };
            (
                page(&["tactic_current.goalkeeper", "tactic_moneyball.goalkeeper"]),
                read_suggestions(db, &SuggestionCache::new(1), "golden", None),
            )
        });

        let page = page.expect("page on a reader");
        assert_eq!(page.state, "ready");
        assert_eq!(page.players.len(), 1);
        for field in ["tactic_current.goalkeeper", "tactic_moneyball.goalkeeper"] {
            assert!(
                page.players[0].dynamic_values.contains_key(field),
                "{field}"
            );
        }
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
            moneyball_stats_date: Some("2031-05-20".to_string()),
        });

        let value = serde_json::to_value(dto).expect("serialize DTO");
        assert_eq!(value["dynamicValues"]["moneyball.np-xg"], 0.75);
        assert_eq!(value["moneyballPercentiles"]["moneyball.np-xg"], 83);
        assert_eq!(value["moneyballStatsDate"], "2031-05-20");
    }
}
