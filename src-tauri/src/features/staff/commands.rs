use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::Db;

use super::filter::{self, FilterAst, FilterRule};
use super::query::{self, SortDir, SortField, StaffPage, StaffPageState, StaffScope, StaffSummary};

#[derive(Deserialize)]
pub struct StaffFilterRuleInput {
    pub field: String,
    pub op: String,
    pub value: serde_json::Value,
}
impl TryFrom<StaffFilterRuleInput> for FilterRule {
    type Error = String;
    fn try_from(value: StaffFilterRuleInput) -> Result<Self, Self::Error> {
        Ok(Self {
            field: value.field,
            op: value.op,
            value: filter::filter_value_from_json(value.value)?,
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffSummaryDto {
    pub uid: i64,
    pub name: Option<String>,
    pub age: Option<i64>,
    pub birth_year: Option<i64>,
    pub birth_day_of_year: Option<i64>,
    pub nationalities: Vec<String>,
    pub nation_uid: Option<i64>,
    pub gender: String,
    pub club: Option<String>,
    pub division: Option<String>,
    pub ca: i64,
    pub pa: i64,
    pub job_id: Option<i64>,
    pub weekly_wage_gbp: Option<i64>,
    pub contract_expiry_year: Option<i64>,
    pub contract_expiry_day_of_year: Option<i64>,
    pub dynamic_values: BTreeMap<String, Option<i64>>,
}
impl From<StaffSummary> for StaffSummaryDto {
    fn from(row: StaffSummary) -> Self {
        Self {
            uid: row.uid,
            name: row.name,
            age: row.age,
            birth_year: row.birth_year,
            birth_day_of_year: row.birth_day_of_year,
            nationalities: row.nationalities,
            nation_uid: row.nation_uid,
            gender: row.gender,
            club: row.club,
            division: row.division,
            ca: row.ca,
            pa: row.pa,
            job_id: row.job_id,
            weekly_wage_gbp: row.weekly_wage_gbp,
            contract_expiry_year: row.contract_expiry_year,
            contract_expiry_day_of_year: row.contract_expiry_day_of_year,
            dynamic_values: row.dynamic_values,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffPageDto {
    pub state: &'static str,
    pub staff: Vec<StaffSummaryDto>,
    pub total: i64,
}
impl From<StaffPage> for StaffPageDto {
    fn from(page: StaffPage) -> Self {
        Self {
            state: match page.state {
                StaffPageState::Ready => "ready",
                StaffPageState::NoCurrentSnapshot => "no_current_snapshot",
                StaffPageState::NoClubFamily => "no_club_family",
            },
            staff: page.staff.into_iter().map(StaffSummaryDto::from).collect(),
            total: page.total,
        }
    }
}

fn parse_filters(
    filters: Option<Vec<StaffFilterRuleInput>>,
    combine: Option<&str>,
) -> Result<Option<FilterAst>, String> {
    filters
        .map(|rules| {
            rules
                .into_iter()
                .map(FilterRule::try_from)
                .collect::<Result<Vec<_>, _>>()
                .and_then(|rules| filter::parse_filter_ast(rules, combine))
        })
        .transpose()
}

#[allow(clippy::too_many_arguments)]
fn run(
    scope: StaffScope,
    offset: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_dir: Option<String>,
    filters: Option<Vec<StaffFilterRuleInput>>,
    filter_combine: Option<String>,
    requested_fields: Option<Vec<String>>,
    db: State<'_, Db>,
) -> Result<StaffPageDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let sort = sort_by
        .as_deref()
        .map(SortField::parse)
        .transpose()?
        .unwrap_or(SortField::DEFAULT);
    let direction = sort_dir
        .as_deref()
        .map(SortDir::parse)
        .transpose()?
        .unwrap_or(SortDir::DEFAULT);
    let filters = parse_filters(filters, filter_combine.as_deref())?;
    query::list_staff(
        &conn,
        scope,
        offset.unwrap_or(0) as usize,
        limit.unwrap_or(query::DEFAULT_PAGE_LIMIT as u32) as usize,
        sort,
        direction,
        filters.as_ref(),
        &requested_fields.unwrap_or_default(),
    )
    .map(StaffPageDto::from)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn search_staff(
    offset: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_dir: Option<String>,
    filters: Option<Vec<StaffFilterRuleInput>>,
    filter_combine: Option<String>,
    requested_fields: Option<Vec<String>>,
    db: State<'_, Db>,
) -> Result<StaffPageDto, String> {
    run(
        StaffScope::Search,
        offset,
        limit,
        sort_by,
        sort_dir,
        filters,
        filter_combine,
        requested_fields,
        db,
    )
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn list_my_staff(
    offset: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_dir: Option<String>,
    requested_fields: Option<Vec<String>>,
    db: State<'_, Db>,
) -> Result<StaffPageDto, String> {
    run(
        StaffScope::MyStaff,
        offset,
        limit,
        sort_by,
        sort_dir,
        None,
        None,
        requested_fields,
        db,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_non_scalar_filter_values() {
        assert!(filter::filter_value_from_json(serde_json::json!({"x":1})).is_err());
        assert!(matches!(
            filter::filter_value_from_json(serde_json::json!(4)).unwrap(),
            filter::FilterValue::Integer(4)
        ));
    }
}
