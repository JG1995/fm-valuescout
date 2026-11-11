use std::collections::{BTreeMap, HashMap};

use rusqlite::{params_from_iter, types::Value, Connection, OptionalExtension, Row};

use crate::db::result_cache::{Cancellation, ResultCache, TableRegistry, TableTicket};
use crate::db::Db;

use super::filter::{compile_filters, FilterAst};
use super::metrics::{parse_requested_fields, MetricField};
use super::scoring::{
    all_staff_roles, assert_read_models_complete, staff_metrics_join, staff_role_column,
    STAFF_METRICS_ALIAS,
};

pub const DEFAULT_PAGE_LIMIT: usize = 50;
pub const MAX_PAGE_LIMIT: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffScope {
    Search,
    MyStaff,
    Shortlist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffPageState {
    Ready,
    NoCurrentSnapshot,
    NoManagedClub,
    NoShortlist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}
impl SortDir {
    pub const DEFAULT: Self = Self::Desc;
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            _ => Err(format!("unknown staff sort direction: {value}")),
        }
    }
    fn sql(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortField {
    Name,
    Age,
    BirthYear,
    BirthDayOfYear,
    Nationality,
    NationUid,
    Gender,
    Club,
    Division,
    Ca,
    Pa,
    JobId,
    Wage,
    ContractYear,
    ContractDayOfYear,
    PreferredJob,
    CoachingQualifications,
    Dynamic(MetricField),
}
impl SortField {
    pub const DEFAULT: Self = Self::Ca;
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "name" => Ok(Self::Name),
            "age" => Ok(Self::Age),
            "birth_year" => Ok(Self::BirthYear),
            "birth_day_of_year" => Ok(Self::BirthDayOfYear),
            "nationality" => Ok(Self::Nationality),
            "nation_uid" => Ok(Self::NationUid),
            "gender" => Ok(Self::Gender),
            "club" => Ok(Self::Club),
            "division" => Ok(Self::Division),
            "ca" => Ok(Self::Ca),
            "pa" => Ok(Self::Pa),
            "job_id" => Ok(Self::JobId),
            "wage" => Ok(Self::Wage),
            "contract_year" => Ok(Self::ContractYear),
            "contract_day" => Ok(Self::ContractDayOfYear),
            "preferred_job" => Ok(Self::PreferredJob),
            "coaching_qualifications" => Ok(Self::CoachingQualifications),
            other => Ok(Self::Dynamic(MetricField::parse(other)?)),
        }
    }
    fn expr(&self) -> String {
        match self {
            Self::Name => "staff.name COLLATE NOCASE".into(),
            Self::Age => "staff.age".into(),
            Self::BirthYear => "staff.birth_year".into(),
            Self::BirthDayOfYear => "staff.birth_day_of_year".into(),
            Self::Nationality => "staff.nationalities_json COLLATE NOCASE".into(),
            Self::NationUid => "staff.nation_uid".into(),
            Self::Gender => "staff.gender COLLATE NOCASE".into(),
            Self::Club => "staff.club COLLATE NOCASE".into(),
            Self::Division => "staff.division COLLATE NOCASE".into(),
            Self::Ca => "staff.ca".into(),
            Self::Pa => "staff.pa".into(),
            Self::JobId => "staff.job_id".into(),
            Self::Wage => "staff.weekly_wage_gbp".into(),
            Self::ContractYear => "staff.contract_expiry_year".into(),
            Self::ContractDayOfYear => "staff.contract_expiry_day_of_year".into(),
            Self::PreferredJob => "shortlist.preferred_job COLLATE NOCASE".into(),
            Self::CoachingQualifications => {
                "shortlist.coaching_qualifications COLLATE NOCASE".into()
            }
            Self::Dynamic(field) => field.sql_expression("staff"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffSummary {
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
    pub shortlist: Option<StaffShortlistMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffShortlistMetadata {
    pub preferred_job: String,
    pub coaching_qualifications: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffPage {
    pub state: StaffPageState,
    pub staff: Vec<StaffSummary>,
    pub total: i64,
    pub preferred_job_options: Vec<String>,
}

pub fn list_my_staff_uids(
    conn: &Connection,
    save_id: i64,
    snapshot_id: i64,
) -> Result<Option<Vec<i64>>, String> {
    let configured: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM managed_club_settings WHERE save_id = ?1)",
            [save_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !configured {
        return Ok(None);
    }

    let mut statement = conn
        .prepare(
            "SELECT DISTINCT staff.uid
             FROM staff
             WHERE staff.snapshot_id = ?1
               AND staff.club = (
                   SELECT club_name FROM managed_club_settings WHERE save_id = ?2
               )
             ORDER BY staff.uid ASC",
        )
        .map_err(|error| error.to_string())?;
    let staff_uids = statement
        .query_map(rusqlite::params![snapshot_id, save_id], |row| row.get(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(Some(staff_uids))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffRoleScore {
    pub role_id: String,
    pub display_name: String,
    pub score: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffDetail {
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
    pub attributes: BTreeMap<String, Option<i64>>,
    pub hidden_information_revealed: bool,
    pub role_scores: Vec<StaffRoleScore>,
}

/// Load one staff member from the active save's effective current snapshot.
pub fn get_staff(conn: &Connection, uid: i64) -> Result<Option<StaffDetail>, String> {
    let context: Option<(i64, i64)> = conn
        .query_row(
            "SELECT snapshots.id, saves.reveal_hidden_information
             FROM snapshots
             INNER JOIN saves ON saves.id = snapshots.save_id AND saves.is_active = 1
             WHERE snapshots.is_current = 1
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let Some((snapshot_id, hidden_information_revealed)) = context else {
        return Ok(None);
    };

    let staff = conn
        .query_row(
            "SELECT uid, name, age, birth_year, birth_day_of_year, nationalities_json,
                    nation_uid, gender, club, division, ca, pa, job_id, weekly_wage_gbp,
                    contract_expiry_year, contract_expiry_day_of_year, staff_attributes_json
             FROM staff WHERE snapshot_id = ?1 AND uid = ?2",
            rusqlite::params![snapshot_id, uid],
            |row| map_staff_detail(row, hidden_information_revealed == 1),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let Some(mut staff) = staff else {
        return Ok(None);
    };
    // Profile must reject missing or wrong-version compact state rather than
    // returning partial nulls.
    assert_read_models_complete(conn, snapshot_id)?;
    staff.role_scores = load_compact_staff_scores(conn, snapshot_id, uid)?;
    Ok(Some(staff))
}

fn map_staff_detail(
    row: &Row<'_>,
    hidden_information_revealed: bool,
) -> rusqlite::Result<StaffDetail> {
    let nationalities_json: String = row.get(5)?;
    let attributes_json: String = row.get(16)?;
    Ok(StaffDetail {
        uid: row.get(0)?,
        name: row.get(1)?,
        age: row.get(2)?,
        birth_year: row.get(3)?,
        birth_day_of_year: row.get(4)?,
        nationalities: serde_json::from_str(&nationalities_json).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                5,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        nation_uid: row.get(6)?,
        gender: row.get(7)?,
        club: row.get(8)?,
        division: row.get(9)?,
        ca: row.get(10)?,
        pa: row.get(11)?,
        job_id: row.get(12)?,
        weekly_wage_gbp: row.get(13)?,
        contract_expiry_year: row.get(14)?,
        contract_expiry_day_of_year: row.get(15)?,
        attributes: serde_json::from_str(&attributes_json).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                16,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        hidden_information_revealed,
        role_scores: Vec::new(),
    })
}

fn load_compact_staff_scores(
    conn: &Connection,
    snapshot_id: i64,
    uid: i64,
) -> Result<Vec<StaffRoleScore>, String> {
    let roles = all_staff_roles();
    let columns = roles
        .iter()
        .map(|role| staff_role_column(role.role_id))
        .collect::<Result<Vec<_>, _>>()?;
    let metric_columns = columns
        .iter()
        .map(|column| format!("{STAFF_METRICS_ALIAS}.{column}"))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT {STAFF_METRICS_ALIAS}.score_model_version, {metric_columns}
         FROM staff s{} WHERE s.snapshot_id = ?1 AND s.uid = ?2",
        staff_metrics_join("s")
    );
    let row = conn
        .query_row(&sql, rusqlite::params![snapshot_id, uid], |row| {
            let has_row = row.get::<_, Option<i64>>(0)?.is_some();
            let scores = (0..roles.len())
                .map(|index| row.get::<_, Option<i64>>(index + 1))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((has_row, scores))
        })
        .optional()
        .map_err(|error| error.to_string())?;
    let Some((has_row, scores)) = row else {
        return Err("Current compact staff snapshot is incomplete".to_string());
    };
    if !has_row {
        return Err("Current compact staff snapshot is incomplete".to_string());
    }
    Ok(roles
        .iter()
        .zip(scores)
        .map(|(role, score)| StaffRoleScore {
            role_id: role.role_id.to_string(),
            display_name: role.display_name.to_string(),
            score,
        })
        .collect())
}

#[allow(clippy::too_many_arguments)] // Keeps the internal query seam aligned with the flat Tauri page request.
pub fn list_staff(
    conn: &Connection,
    scope: StaffScope,
    offset: usize,
    limit: usize,
    sort: SortField,
    direction: SortDir,
    filters: Option<&FilterAst>,
    shortlist_only: bool,
    preferred_job: Option<&str>,
    unemployed_only: bool,
    requested_fields: &[String],
) -> Result<StaffPage, String> {
    let preferred_jobs = preferred_job.into_iter().collect::<Vec<_>>();
    list_staff_with_shortlist(
        conn,
        scope,
        offset,
        limit,
        sort,
        direction,
        filters,
        shortlist_only,
        &preferred_jobs,
        unemployed_only,
        requested_fields,
    )
}

#[allow(clippy::too_many_arguments)]
fn list_staff_with_shortlist(
    conn: &Connection,
    scope: StaffScope,
    offset: usize,
    limit: usize,
    sort: SortField,
    direction: SortDir,
    filters: Option<&FilterAst>,
    shortlist_only: bool,
    preferred_jobs: &[&str],
    unemployed_only: bool,
    requested_fields: &[String],
) -> Result<StaffPage, String> {
    let shortlisted = shortlisted_query(scope, &sort, shortlist_only)?;
    let dynamic_fields = parse_requested_fields(requested_fields)?;
    let has_role_field = dynamic_fields
        .iter()
        .any(|field| matches!(field, MetricField::Role(_)));
    let query = match plan_staff_query(
        conn,
        scope,
        &sort,
        direction,
        filters,
        shortlisted,
        preferred_jobs,
        unemployed_only,
        has_role_field,
    )? {
        Ok(query) => query,
        Err(state) => return Ok(empty(state)),
    };
    let StaffQuery {
        save_id,
        from_sql,
        where_sql,
        mut binds,
        order_sql,
        ..
    } = query;

    let total: i64 = conn
        .query_row(
            &format!("SELECT COUNT(*) FROM {from_sql} WHERE {where_sql}"),
            params_from_iter(binds.iter()),
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    let limit = i64::try_from(limit.clamp(1, MAX_PAGE_LIMIT))
        .map_err(|_| "staff limit out of range".to_string())?;
    let offset = i64::try_from(offset).map_err(|_| "staff offset out of range".to_string())?;
    let mut sql = staff_select_sql(&dynamic_fields, shortlisted);
    let limit_index = binds.len() + 1;
    let offset_index = binds.len() + 2;
    sql.push_str(&format!(
        " FROM {from_sql} WHERE {where_sql} {order_sql} LIMIT ?{limit_index} OFFSET ?{offset_index}"
    ));
    binds.push(Value::Integer(limit));
    binds.push(Value::Integer(offset));
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let staff = statement
        .query_map(params_from_iter(binds.iter()), |row| {
            map_staff(row, &dynamic_fields, shortlisted)
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let preferred_job_options = if shortlisted {
        read_preferred_job_options(conn, save_id)?
    } else {
        Vec::new()
    };
    Ok(StaffPage {
        state: StaffPageState::Ready,
        staff,
        total,
        preferred_job_options,
    })
}

/// Whether a query reads the shortlist. Only a shortlisted query may sort by a shortlist
/// column.
fn shortlisted_query(
    scope: StaffScope,
    sort: &SortField,
    shortlist_only: bool,
) -> Result<bool, String> {
    // The flagged core path reuses the Shortlist composition: shortlist JOIN,
    // shortlist predicates, then general filters, before count, sort, paging.
    let shortlisted = scope == StaffScope::Shortlist || shortlist_only;
    if matches!(
        sort,
        SortField::PreferredJob | SortField::CoachingQualifications
    ) && !shortlisted
    {
        return Err("shortlist columns can only sort Shortlist".to_string());
    }
    Ok(shortlisted)
}

/// The SQL that decides one Staff table query's rows and their order, before paging.
struct StaffQuery {
    snapshot_id: i64,
    save_id: i64,
    from_sql: String,
    where_sql: String,
    binds: Vec<Value>,
    order_sql: String,
}

/// Builds a query's source, predicate, and order for the active save's current snapshot, or
/// the setup state that leaves it empty. `join_metrics` joins compact role metrics for the
/// caller's requested fields; the sort and filters join them on their own.
#[allow(clippy::too_many_arguments)]
fn plan_staff_query(
    conn: &Connection,
    scope: StaffScope,
    sort: &SortField,
    direction: SortDir,
    filters: Option<&FilterAst>,
    shortlisted: bool,
    preferred_jobs: &[&str],
    unemployed_only: bool,
    join_metrics: bool,
) -> Result<Result<StaffQuery, StaffPageState>, String> {
    // Filter placeholders follow the shortlist binds: one for the shortlist
    // save plus one for the Preferred Job predicate when each applies.
    let filter_start = (if scope == StaffScope::MyStaff { 3 } else { 2 })
        + usize::from(shortlisted)
        + usize::from(!preferred_jobs.is_empty() && shortlisted);
    let compiled = filters
        .map(|filters| compile_filters(filters, filter_start))
        .transpose()?;
    let context: Option<(i64, i64)> = conn.query_row(
        "SELECT snapshots.id, snapshots.save_id FROM snapshots INNER JOIN saves ON saves.id = snapshots.save_id AND saves.is_active = 1 WHERE snapshots.is_current = 1 LIMIT 1",
        [], |row| Ok((row.get(0)?, row.get(1)?)),
    ).optional().map_err(|error| error.to_string())?;
    let Some((snapshot_id, save_id)) = context else {
        return Ok(Err(StaffPageState::NoCurrentSnapshot));
    };
    let sort_needs_metrics = matches!(sort, SortField::Dynamic(MetricField::Role(_)));
    let filter_needs_metrics =
        filters.is_some_and(|ast| ast.rules.iter().any(|rule| rule.field.starts_with("role.")));
    let needs_metrics = join_metrics || sort_needs_metrics || filter_needs_metrics;
    // Every staff table read requires a complete compact snapshot: a missing or
    // wrong-version `staff_role_metrics` row for the effective current snapshot
    // must surface as an error instead of silently returning raw staff rows,
    // regardless of which fields, filters, or sorts the page requested.
    assert_read_models_complete(conn, snapshot_id)?;

    if scope == StaffScope::MyStaff {
        let configured: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM managed_club_settings WHERE save_id = ?1)",
                [save_id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        if !configured {
            return Ok(Err(StaffPageState::NoManagedClub));
        }
    }

    let mut binds = vec![Value::Integer(snapshot_id)];
    let mut where_sql = "staff.snapshot_id = ?1".to_string();
    let mut from_sql = "staff".to_string();
    if needs_metrics {
        from_sql.push_str(&staff_metrics_join("staff"));
    }
    if scope == StaffScope::MyStaff {
        binds.push(Value::Integer(save_id));
        where_sql.push_str(
            " AND staff.club = (SELECT club_name FROM managed_club_settings WHERE save_id = ?2)",
        );
    }
    if shortlisted {
        let has_shortlist: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM staff_shortlist_entries WHERE save_id = ?1)",
                [save_id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        if !has_shortlist {
            return Ok(Err(StaffPageState::NoShortlist));
        }
        binds.push(Value::Integer(save_id));
        from_sql.push_str(&format!(
            " INNER JOIN staff_shortlist_entries shortlist ON shortlist.staff_uid = staff.uid AND shortlist.save_id = ?{}",
            binds.len()
        ));
        if let Some(preferred_job) = preferred_jobs.first() {
            binds.push(Value::Text((*preferred_job).to_string()));
            where_sql.push_str(&format!(
                " AND shortlist.preferred_job COLLATE NOCASE = ?{}",
                binds.len()
            ));
        }
        if unemployed_only {
            // The shortlist holds no job: a person is unemployed when the snapshot has no
            // club for them.
            where_sql.push_str(" AND staff.club IS NULL");
        }
    }
    if let Some(compiled) = compiled {
        if !compiled.sql.is_empty() {
            where_sql.push_str(" AND ");
            where_sql.push_str(&compiled.sql);
            binds.extend(compiled.params);
        }
    }
    let expr = sort.expr();
    let order_sql = format!(
        "ORDER BY ({expr}) IS NULL ASC, {expr} {}, staff.uid ASC",
        direction.sql()
    );
    Ok(Ok(StaffQuery {
        snapshot_id,
        save_id,
        from_sql,
        where_sql,
        binds,
        order_sql,
    }))
}

/// Today's display columns, the requested fields, and the shortlist metadata, in
/// [`map_staff`]'s order.
fn staff_select_sql(dynamic_fields: &[MetricField], shortlisted: bool) -> String {
    let mut sql = String::from("SELECT staff.uid, staff.name, staff.age, staff.birth_year, staff.birth_day_of_year, staff.nationalities_json, staff.nation_uid, staff.gender, staff.club, staff.division, staff.ca, staff.pa, staff.job_id, staff.weekly_wage_gbp, staff.contract_expiry_year, staff.contract_expiry_day_of_year");
    for field in dynamic_fields {
        sql.push_str(", ");
        sql.push_str(&field.sql_expression("staff"));
    }
    if shortlisted {
        sql.push_str(", shortlist.preferred_job, shortlist.coaching_qualifications");
    }
    sql
}

fn read_preferred_job_options(conn: &Connection, save_id: i64) -> Result<Vec<String>, String> {
    conn.prepare("SELECT DISTINCT preferred_job FROM staff_shortlist_entries WHERE save_id = ?1 ORDER BY preferred_job COLLATE NOCASE ASC")
        .map_err(|error| error.to_string())?
        .query_map([save_id], |row| row.get(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

/// One Staff Search page request: the query that decides rows and order, and the window and
/// fields to read.
pub struct StaffSearchRequest<'a> {
    pub offset: usize,
    pub limit: usize,
    pub sort: SortField,
    pub direction: SortDir,
    pub filters: Option<&'a FilterAst>,
    pub shortlist_only: bool,
    pub preferred_job: Option<&'a str>,
    pub unemployed_only: bool,
    pub requested_fields: &'a [String],
}

/// One Staff Search query's complete ordering, computed once per data generation.
pub struct StaffOrder {
    /// The data generation of the reader that computed it.
    generation: u64,
    state: StaffPageState,
    snapshot_id: i64,
    save_id: i64,
    shortlisted: bool,
    uids: Vec<i64>,
    preferred_job_options: Vec<String>,
}

/// Staff Search's computed orders, keyed by [`staff_query_fingerprint`].
pub type StaffOrderCache = ResultCache<StaffOrder>;

/// Identifies a Staff Search query by every input that changes its rows or their order. The
/// offset, limit, and requested fields only choose which rows and columns a window shows.
pub fn staff_query_fingerprint(request: &StaffSearchRequest<'_>) -> String {
    format!(
        "{:?}|{:?}|{:?}|{}|{:?}|{}",
        request.sort,
        request.direction,
        request.filters,
        request.shortlist_only,
        request.preferred_job,
        request.unemployed_only,
    )
}

/// Serves one Staff Search window from its query's computed order, computing the order on a
/// miss. With a ticket, the computation is its table's in `registry`, so a newer request for
/// the table can supersede it. Returns the same page as [`list_staff`] with the Search scope.
pub fn search_staff_page(
    db: &Db,
    orders: &StaffOrderCache,
    registry: &TableRegistry,
    ticket: Option<&TableTicket>,
    request: StaffSearchRequest<'_>,
) -> Result<StaffPage, String> {
    // Fields are validated before the query, as `list_staff` does.
    let dynamic_fields = parse_requested_fields(request.requested_fields)?;
    let fingerprint = staff_query_fingerprint(&request);
    let mut generation = db.generation();
    loop {
        let order = orders.get_or_compute(&fingerprint, generation, || {
            let reader = db.read()?;
            let order = registry.run(ticket, &reader, |cancellation| {
                compute_staff_order(&reader, reader.generation(), &request, cancellation)
            })?;
            Ok::<_, String>((reader.generation(), order))
        })?;
        let reader = db.read()?;
        if reader.generation() > order.generation {
            // Data changed since the order was read, so the window reads a fresh order.
            generation = reader.generation();
            continue;
        }
        return read_staff_window(&reader, &order, &request, &dynamic_fields);
    }
}

/// Computes a Staff Search query's complete key-only ordering with `list_staff`'s snapshot,
/// shortlist, filter, join, and sort SQL.
fn compute_staff_order(
    conn: &Connection,
    generation: u64,
    request: &StaffSearchRequest<'_>,
    cancellation: &Cancellation,
) -> Result<StaffOrder, String> {
    #[cfg(test)]
    tests::ORDER_COMPUTATIONS.with(|count| count.set(count.get() + 1));
    let shortlisted = shortlisted_query(StaffScope::Search, &request.sort, request.shortlist_only)?;
    let preferred_jobs = request.preferred_job.into_iter().collect::<Vec<_>>();
    let empty = |state| StaffOrder {
        generation,
        state,
        snapshot_id: 0,
        save_id: 0,
        shortlisted,
        uids: Vec::new(),
        preferred_job_options: Vec::new(),
    };
    let query = match plan_staff_query(
        conn,
        StaffScope::Search,
        &request.sort,
        request.direction,
        request.filters,
        shortlisted,
        &preferred_jobs,
        request.unemployed_only,
        false,
    )? {
        Ok(query) => query,
        Err(state) => return Ok(empty(state)),
    };

    #[cfg(test)]
    if let Some(hook) = tests::BEFORE_ORDER_QUERY.with(|hook| hook.borrow_mut().take()) {
        hook();
    }
    cancellation.check()?;
    let StaffQuery {
        snapshot_id,
        save_id,
        from_sql,
        where_sql,
        binds,
        order_sql,
    } = query;
    let uids = conn
        .prepare(&format!(
            "SELECT staff.uid FROM {from_sql} WHERE {where_sql} {order_sql}"
        ))
        .map_err(|error| error.to_string())?
        .query_map(params_from_iter(binds.iter()), |row| row.get(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<i64>, _>>()
        .map_err(|error| error.to_string())?;
    cancellation.check()?;
    let preferred_job_options = if shortlisted {
        read_preferred_job_options(conn, save_id)?
    } else {
        Vec::new()
    };
    Ok(StaffOrder {
        generation,
        state: StaffPageState::Ready,
        snapshot_id,
        save_id,
        shortlisted,
        uids,
        preferred_job_options,
    })
}

/// Reads today's display columns, the requested fields, and each row's shortlist metadata for
/// one window of a computed order, in the order's sequence. The order's reader checked the
/// compact snapshot, and this reader reads the same generation.
fn read_staff_window(
    conn: &Connection,
    order: &StaffOrder,
    request: &StaffSearchRequest<'_>,
    dynamic_fields: &[MetricField],
) -> Result<StaffPage, String> {
    if order.state != StaffPageState::Ready {
        return Ok(empty(order.state));
    }
    let limit = request.limit.clamp(1, MAX_PAGE_LIMIT);
    let start = request.offset.min(order.uids.len());
    let window = &order.uids[start..(start + limit).min(order.uids.len())];

    let mut sql = staff_select_sql(dynamic_fields, order.shortlisted);
    sql.push_str(" FROM staff");
    if dynamic_fields
        .iter()
        .any(|field| matches!(field, MetricField::Role(_)))
    {
        sql.push_str(&staff_metrics_join("staff"));
    }
    let mut binds = vec![Value::Integer(order.snapshot_id)];
    if order.shortlisted {
        binds.push(Value::Integer(order.save_id));
        sql.push_str(
            " INNER JOIN staff_shortlist_entries shortlist ON shortlist.staff_uid = staff.uid AND shortlist.save_id = ?2",
        );
    }
    let first_uid_index = binds.len() + 1;
    let uid_placeholders = (0..window.len())
        .map(|index| format!("?{}", first_uid_index + index))
        .collect::<Vec<_>>()
        .join(", ");
    binds.extend(window.iter().map(|uid| Value::Integer(*uid)));
    sql.push_str(&format!(
        " WHERE staff.snapshot_id = ?1 AND staff.uid IN ({uid_placeholders})"
    ));

    let mut rows = conn
        .prepare(&sql)
        .map_err(|error| error.to_string())?
        .query_map(params_from_iter(binds.iter()), |row| {
            map_staff(row, dynamic_fields, order.shortlisted)
        })
        .map_err(|error| error.to_string())?
        .map(|row| row.map(|staff| (staff.uid, staff)))
        .collect::<Result<HashMap<_, _>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(StaffPage {
        state: StaffPageState::Ready,
        staff: window.iter().filter_map(|uid| rows.remove(uid)).collect(),
        total: i64::try_from(order.uids.len()).map_err(|error| error.to_string())?,
        preferred_job_options: order.preferred_job_options.clone(),
    })
}

fn empty(state: StaffPageState) -> StaffPage {
    StaffPage {
        state,
        staff: Vec::new(),
        total: 0,
        preferred_job_options: Vec::new(),
    }
}
fn map_staff(
    row: &Row<'_>,
    fields: &[MetricField],
    has_shortlist: bool,
) -> rusqlite::Result<StaffSummary> {
    let json: String = row.get(5)?;
    let nationalities = serde_json::from_str(&json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let mut dynamic_values = BTreeMap::new();
    for (index, field) in fields.iter().enumerate() {
        dynamic_values.insert(field.id(), row.get(16 + index)?);
    }
    Ok(StaffSummary {
        uid: row.get(0)?,
        name: row.get(1)?,
        age: row.get(2)?,
        birth_year: row.get(3)?,
        birth_day_of_year: row.get(4)?,
        nationalities,
        nation_uid: row.get(6)?,
        gender: row.get(7)?,
        club: row.get(8)?,
        division: row.get(9)?,
        ca: row.get(10)?,
        pa: row.get(11)?,
        job_id: row.get(12)?,
        weekly_wage_gbp: row.get(13)?,
        contract_expiry_year: row.get(14)?,
        contract_expiry_day_of_year: row.get(15)?,
        dynamic_values,
        shortlist: if has_shortlist {
            Some(StaffShortlistMetadata {
                preferred_job: row.get(16 + fields.len())?,
                coaching_qualifications: row.get(17 + fields.len())?,
            })
        } else {
            None
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::features::staff::filter::{parse_filter_ast, FilterRule, FilterValue};
    use crate::features::{
        player::{query as player_query, service as player_service},
        player_metrics::potential_scores::rebuild_snapshot,
    };
    use std::cell::{Cell, RefCell};

    thread_local! {
        /// Staff Search order computations run on this thread.
        pub(super) static ORDER_COMPUTATIONS: Cell<usize> = const { Cell::new(0) };
        /// Runs once, just before the next Staff Search order query on this thread.
        pub(super) static BEFORE_ORDER_QUERY: RefCell<Option<Box<dyn FnOnce()>>> =
            const { RefCell::new(None) };
    }

    fn open() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", true).unwrap();
        migrations::apply(&conn).unwrap();
        conn
    }
    fn seed(conn: &Connection, managed_club: bool) {
        conn.execute_batch("INSERT INTO saves (id,name,is_active) VALUES (1,'Save',1); INSERT INTO snapshots (id,save_id,is_current,schema_version,generated_at_utc,game_version,supported_game_version,bridge_version,protocol_version,game_date_source,scan_truncated,player_count) VALUES (1,1,1,8,'now','26.3','26.3','0.4',1,'unknown',0,0),(2,1,0,8,'old','26.3','26.3','0.4',1,'unknown',0,0); INSERT INTO staff (snapshot_id,uid,name,age,nationalities_json,gender,ca,pa,staff_attributes_json,club) VALUES (1,1,'Alpha',40,'[\"DEN\"]','male',100,120,'{\"Authority\":18}','Club A'),(1,2,'Beta',41,'[\"SWE\"]','female',110,130,'{\"Authority\":15}','Club B'),(1,3,'Gamma',42,'[]','unknown',90,100,'{\"Authority\":null}','Other'),(2,9,'Old',50,'[]','unknown',200,200,'{}','Club A'); INSERT INTO staff_role_metrics (snapshot_id, uid, score_model_version, coach_fitness) VALUES (1, 1, 1, 80), (1, 2, 1, 70), (1, 3, 1, NULL);").unwrap();
        if managed_club {
            conn.execute_batch(
                "INSERT INTO managed_club_settings (save_id,club_name) VALUES (1,'Club A');",
            )
            .unwrap();
        }
    }
    #[test]
    fn search_is_all_current_while_my_staff_uses_the_exact_managed_club() {
        let conn = open();
        seed(&conn, true);
        let search = list_staff(
            &conn,
            StaffScope::Search,
            0,
            50,
            SortField::Name,
            SortDir::Asc,
            None,
            false,
            None,
            false,
            &[],
        )
        .unwrap();
        let mine = list_staff(
            &conn,
            StaffScope::MyStaff,
            0,
            50,
            SortField::Name,
            SortDir::Asc,
            None,
            false,
            None,
            false,
            &[],
        )
        .unwrap();
        assert_eq!(
            search.staff.iter().map(|s| s.uid).collect::<Vec<_>>(),
            [1, 2, 3]
        );
        assert_eq!(mine.staff.iter().map(|s| s.uid).collect::<Vec<_>>(), [1]);
        let second_page = list_staff(
            &conn,
            StaffScope::MyStaff,
            0,
            1,
            SortField::Name,
            SortDir::Asc,
            None,
            false,
            None,
            false,
            &[],
        )
        .unwrap();
        assert_eq!(second_page.total, 1);
        assert_eq!(second_page.staff[0].uid, 1);
    }

    #[test]
    fn shortlist_joins_active_save_metadata_and_filters_before_paging() {
        let conn = open();
        seed(&conn, false);
        conn.execute_batch(
            "INSERT INTO staff_shortlist_entries (
                save_id, staff_uid, preferred_job, coaching_qualifications
             ) VALUES
                (1, 1, 'Physio', 'Continental Pro'),
                (1, 2, 'Scout', 'National C'),
                (1, 3, 'Scout', 'National B');
             UPDATE staff SET club = NULL WHERE snapshot_id = 1 AND uid IN (1, 2);",
        )
        .expect("seed shortlist");

        let all = list_staff(
            &conn,
            StaffScope::Search,
            0,
            1,
            SortField::Name,
            SortDir::Asc,
            None,
            true,
            None,
            false,
            &[],
        )
        .expect("list shortlist");
        assert_eq!(all.total, 3);
        assert_eq!(all.staff[0].uid, 1);
        assert_eq!(
            all.staff[0].shortlist.as_ref().expect("shortlist metadata"),
            &StaffShortlistMetadata {
                preferred_job: "Physio".to_string(),
                coaching_qualifications: "Continental Pro".to_string(),
            }
        );
        assert_eq!(all.preferred_job_options, ["Physio", "Scout"]);

        let unemployed_scouts = list_staff(
            &conn,
            StaffScope::Search,
            0,
            50,
            SortField::Name,
            SortDir::Asc,
            None,
            true,
            Some("Scout"),
            true,
            &[],
        )
        .expect("filter shortlist");
        assert_eq!(unemployed_scouts.total, 1);
        assert_eq!(unemployed_scouts.staff[0].uid, 2);

        let unemployed = list_staff(
            &conn,
            StaffScope::Search,
            0,
            50,
            SortField::Name,
            SortDir::Asc,
            None,
            true,
            None,
            true,
            &[],
        )
        .expect("all unemployed shortlist staff");
        assert_eq!(
            unemployed
                .staff
                .iter()
                .map(|staff| staff.uid)
                .collect::<Vec<_>>(),
            [1, 2]
        );

        let case_insensitive = list_staff(
            &conn,
            StaffScope::Search,
            0,
            50,
            SortField::Name,
            SortDir::Asc,
            None,
            true,
            Some("scout"),
            false,
            &[],
        )
        .expect("case-insensitive job filter");
        assert_eq!(case_insensitive.total, 2);
    }

    #[test]
    fn flagged_core_search_returns_shortlist_rows_and_composes_core_filters() {
        let conn = open();
        seed(&conn, false);
        conn.execute_batch(
            "INSERT INTO staff_shortlist_entries (
                save_id, staff_uid, preferred_job, coaching_qualifications
             ) VALUES
                (1, 1, 'Physio', 'Continental Pro'),
                (1, 2, 'Scout', 'National C'),
                (1, 3, 'Scout', 'National B');
             UPDATE staff SET club = NULL WHERE snapshot_id = 1 AND uid IN (1, 2);",
        )
        .expect("seed shortlist");

        let flagged = list_staff(
            &conn,
            StaffScope::Search,
            0,
            50,
            SortField::Name,
            SortDir::Asc,
            None,
            true,
            None,
            false,
            &[],
        )
        .expect("flagged core search");
        assert_eq!(flagged.state, StaffPageState::Ready);
        assert_eq!(flagged.total, 3);
        assert_eq!(
            flagged
                .staff
                .iter()
                .map(|staff| staff.uid)
                .collect::<Vec<_>>(),
            [1, 2, 3]
        );
        assert_eq!(
            flagged.staff[0]
                .shortlist
                .as_ref()
                .expect("shortlist metadata"),
            &StaffShortlistMetadata {
                preferred_job: "Physio".to_string(),
                coaching_qualifications: "Continental Pro".to_string(),
            }
        );
        assert_eq!(flagged.preferred_job_options, ["Physio", "Scout"]);

        let sorted = list_staff(
            &conn,
            StaffScope::Search,
            0,
            50,
            SortField::PreferredJob,
            SortDir::Desc,
            None,
            true,
            None,
            false,
            &[],
        )
        .expect("flagged shortlist sort");
        assert_eq!(
            sorted
                .staff
                .iter()
                .map(|staff| staff.uid)
                .collect::<Vec<_>>(),
            [2, 3, 1]
        );

        let ast = parse_filter_ast(
            vec![FilterRule {
                field: "ca".into(),
                op: "gt".into(),
                value: FilterValue::Integer(95),
            }],
            None,
        )
        .unwrap();
        let combined = list_staff(
            &conn,
            StaffScope::Search,
            0,
            1,
            SortField::Name,
            SortDir::Asc,
            Some(&ast),
            true,
            Some("Scout"),
            true,
            &[],
        )
        .expect("flagged shortlist with preferred job, unemployment, core filter, paging");
        assert_eq!(combined.total, 1);
        assert_eq!(
            combined
                .staff
                .iter()
                .map(|staff| staff.uid)
                .collect::<Vec<_>>(),
            [2]
        );
        assert_eq!(
            combined.staff[0]
                .shortlist
                .as_ref()
                .expect("shortlist metadata"),
            &StaffShortlistMetadata {
                preferred_job: "Scout".to_string(),
                coaching_qualifications: "National C".to_string(),
            }
        );
    }

    #[test]
    fn shortlist_distinguishes_setup_states_and_keeps_saved_rows_when_current_staff_changes() {
        let conn = open();
        assert_eq!(
            list_staff(
                &conn,
                StaffScope::Search,
                0,
                50,
                SortField::Ca,
                SortDir::Desc,
                None,
                true,
                None,
                false,
                &[]
            )
            .expect("empty database state")
            .state,
            StaffPageState::NoCurrentSnapshot
        );
        seed(&conn, false);
        assert_eq!(
            list_staff(
                &conn,
                StaffScope::Search,
                0,
                50,
                SortField::Ca,
                SortDir::Desc,
                None,
                true,
                None,
                false,
                &[]
            )
            .expect("no shortlist state")
            .state,
            StaffPageState::NoShortlist
        );
        conn.execute(
            "INSERT INTO staff_shortlist_entries (
                save_id, staff_uid, preferred_job, coaching_qualifications
             ) VALUES (1, 1, 'Physio', 'A')",
            [],
        )
        .expect("seed shortlist");
        conn.execute(
            "INSERT INTO staff_shortlist_entries (
                save_id, staff_uid, preferred_job, coaching_qualifications
             ) VALUES (1, 3, 'Scout', 'B')",
            [],
        )
        .expect("seed departed entry");
        conn.execute_batch(
            "UPDATE snapshots SET is_current = 0 WHERE id = 1;
             UPDATE snapshots SET is_current = 1 WHERE id = 2;
             INSERT INTO staff (
                snapshot_id, uid, name, age, nationalities_json, gender, ca, pa,
                staff_attributes_json, club
             ) VALUES (2, 1, 'Replacement', 30, '[]', 'unknown', 150, 160, '{}', 'New Club');
             INSERT INTO staff_role_metrics (snapshot_id, uid, score_model_version)
             VALUES (2, 1, 1), (2, 9, 1);",
        )
        .expect("replace current snapshot");

        let replacement = list_staff(
            &conn,
            StaffScope::Search,
            0,
            50,
            SortField::Ca,
            SortDir::Desc,
            None,
            true,
            None,
            false,
            &[],
        )
        .expect("current replacement staff");
        assert_eq!(replacement.state, StaffPageState::Ready);
        assert_eq!(replacement.total, 1);
        assert_eq!(replacement.staff[0].name.as_deref(), Some("Replacement"));
        assert_eq!(
            replacement.staff[0]
                .shortlist
                .as_ref()
                .unwrap()
                .preferred_job,
            "Physio"
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM staff_shortlist_entries WHERE save_id = 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .expect("saved row remains"),
            2
        );
    }

    #[test]
    fn shortlist_scopes_its_sorts_and_role_projection_to_the_bounded_query() {
        let conn = open();
        seed(&conn, false);
        conn.execute_batch(
            "INSERT INTO staff_shortlist_entries (
                save_id, staff_uid, preferred_job, coaching_qualifications
             ) VALUES
                (1, 1, 'Zulu', 'Zulu qualification'),
                (1, 2, 'Alpha', 'Alpha qualification');
             INSERT INTO saves (id, name, is_active) VALUES (2, 'Inactive', 0);
             INSERT INTO staff_shortlist_entries (
                save_id, staff_uid, preferred_job, coaching_qualifications
             ) VALUES (2, 1, 'Wrong save', 'Wrong qualification');
             UPDATE staff_role_metrics SET coach_fitness = 70 WHERE snapshot_id = 1 AND uid = 1;
             UPDATE staff_role_metrics SET coach_fitness = 80 WHERE snapshot_id = 1 AND uid = 2;",
        )
        .expect("seed shortlist");
        for sort in [SortField::PreferredJob, SortField::CoachingQualifications] {
            let sorted = list_staff(
                &conn,
                StaffScope::Search,
                0,
                50,
                sort.clone(),
                SortDir::Asc,
                None,
                true,
                None,
                false,
                &[],
            )
            .expect("sort shortlist column");
            assert_eq!(
                sorted
                    .staff
                    .iter()
                    .map(|staff| staff.uid)
                    .collect::<Vec<_>>(),
                [2, 1]
            );
            assert!(list_staff(
                &conn,
                StaffScope::Search,
                0,
                50,
                sort,
                SortDir::Asc,
                None,
                false,
                None,
                false,
                &[],
            )
            .is_err());
        }
        let scores = list_staff(
            &conn,
            StaffScope::Search,
            0,
            1,
            SortField::parse("role.coach_fitness").unwrap(),
            SortDir::Desc,
            None,
            true,
            None,
            false,
            &["role.coach_fitness".to_string()],
        )
        .expect("role score page");
        assert_eq!(scores.total, 2);
        assert_eq!(scores.staff[0].uid, 2);
        assert_eq!(
            scores.staff[0].dynamic_values["role.coach_fitness"],
            Some(80)
        );
        assert!(list_staff(
            &conn,
            StaffScope::Search,
            0,
            50,
            SortField::Ca,
            SortDir::Desc,
            None,
            true,
            None,
            false,
            &["role.unknown".to_string()],
        )
        .is_err());
    }
    #[test]
    fn returns_requested_metrics_and_applies_bound_filters() {
        let conn = open();
        seed(&conn, true);
        let ast = parse_filter_ast(
            vec![FilterRule {
                field: "role.coach_fitness".into(),
                op: "gt".into(),
                value: FilterValue::Integer(75),
            }],
            None,
        )
        .unwrap();
        let page = list_staff(
            &conn,
            StaffScope::Search,
            0,
            50,
            SortField::Ca,
            SortDir::Desc,
            Some(&ast),
            false,
            None,
            false,
            &["attr.Authority".into(), "role.coach_fitness".into()],
        )
        .unwrap();
        assert_eq!(page.staff.len(), 1);
        assert_eq!(page.staff[0].dynamic_values["attr.Authority"], Some(18));
        assert_eq!(page.staff[0].dynamic_values["role.coach_fitness"], Some(80));
    }

    #[test]
    fn staff_detail_is_current_only_and_returns_catalog_order_with_null_scores() {
        let conn = open();
        assert!(get_staff(&conn, 1)
            .expect("lookup without an active snapshot")
            .is_none());
        seed(&conn, true);
        conn.execute_batch(
            "INSERT INTO saves (id,name,is_active) VALUES (2,'Inactive',0);
             INSERT INTO snapshots (id,save_id,is_current,schema_version,generated_at_utc,game_version,supported_game_version,bridge_version,protocol_version,game_date_source,scan_truncated,player_count)
             VALUES (3,2,1,8,'now','26.3','26.3','0.4',1,'unknown',0,0);
             INSERT INTO staff (snapshot_id,uid,name,age,nationalities_json,gender,ca,pa,staff_attributes_json,club)
             VALUES (3,1,'Inactive Alpha',40,'[]','unknown',200,200,'{}','Other');",
        )
        .expect("insert inactive save staff");

        let detail = get_staff(&conn, 1).expect("get staff").expect("staff");
        assert_eq!(detail.name.as_deref(), Some("Alpha"));
        assert_eq!(detail.attributes["Authority"], Some(18));
        assert!(detail.hidden_information_revealed);
        assert_eq!(detail.role_scores.len(), all_staff_roles().len());
        assert_eq!(detail.role_scores[0].role_id, "assistant_manager");
        assert_eq!(detail.role_scores[0].score, None);
        assert_eq!(
            detail
                .role_scores
                .iter()
                .find(|role| role.role_id == "coach_fitness")
                .and_then(|role| role.score),
            Some(80)
        );
        assert!(get_staff(&conn, 9)
            .expect("historical staff lookup")
            .is_none());
        assert!(get_staff(&conn, 999)
            .expect("missing staff lookup")
            .is_none());
        assert_eq!(
            get_staff(&conn, 3)
                .expect("get nullable staff")
                .expect("nullable staff")
                .attributes["Authority"],
            None
        );
    }

    #[test]
    fn staff_detail_reads_the_shared_active_save_visibility_without_redaction() {
        let conn = open();
        seed(&conn, true);
        conn.execute(
            "UPDATE saves SET reveal_hidden_information = 0 WHERE id = 1",
            [],
        )
        .expect("conceal information");

        let detail = get_staff(&conn, 1).expect("get staff").expect("staff");
        assert!(!detail.hidden_information_revealed);
        assert_eq!(detail.pa, 120);
        assert_eq!(detail.attributes["Authority"], Some(18));
        assert_eq!(detail.role_scores.len(), 21);
    }

    #[test]
    fn one_visibility_setter_is_observed_by_player_and_staff_details() {
        let conn = open();
        seed(&conn, true);
        conn.execute(
            "INSERT INTO players (
                snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                nationalities_json, preferred_foot, positions_json, attributes_json,
                hidden_attributes_json, personality_json
             ) VALUES (1, 77, 100, 120, 'Player', 2000, 1, '[]', 'right', '{}', '{}', '{}', '{}')",
            [],
        )
        .expect("insert player");
        let tx = conn
            .unchecked_transaction()
            .expect("start potential materialization transaction");
        rebuild_snapshot(&tx, 1).expect("materialize player potential state");
        tx.commit().expect("commit player potential state");

        assert!(
            player_query::get_player(&conn, 77)
                .expect("get player")
                .expect("player")
                .hidden_information_revealed
        );
        assert!(
            get_staff(&conn, 1)
                .expect("get staff")
                .expect("staff")
                .hidden_information_revealed
        );

        assert!(
            !player_service::set_hidden_information_revealed(&conn, false)
                .expect("conceal shared information")
        );

        let player = player_query::get_player(&conn, 77)
            .expect("get concealed player")
            .expect("player");
        let staff = get_staff(&conn, 1)
            .expect("get concealed staff")
            .expect("staff");
        assert!(!player.hidden_information_revealed);
        assert!(!staff.hidden_information_revealed);
        assert_eq!(player.pa, 120);
        assert_eq!(staff.pa, 120);
    }
    #[test]
    fn injection_shaped_text_is_bound_as_data() {
        let conn = open();
        seed(&conn, true);
        let ast = parse_filter_ast(
            vec![FilterRule {
                field: "name".into(),
                op: "contains".into(),
                value: FilterValue::Text("%' OR 1=1 --".into()),
            }],
            None,
        )
        .unwrap();
        let page = list_staff(
            &conn,
            StaffScope::Search,
            0,
            50,
            SortField::Name,
            SortDir::Asc,
            Some(&ast),
            false,
            None,
            false,
            &[],
        )
        .unwrap();
        assert_eq!(page.total, 0);
    }
    #[test]
    fn combines_filters_with_or_and_sorts_scores_with_nulls_last() {
        let conn = open();
        seed(&conn, true);
        let ast = parse_filter_ast(
            vec![
                FilterRule {
                    field: "club".into(),
                    op: "is".into(),
                    value: FilterValue::Text("Other".into()),
                },
                FilterRule {
                    field: "ca".into(),
                    op: "gt".into(),
                    value: FilterValue::Integer(105),
                },
            ],
            Some("or"),
        )
        .unwrap();
        let filtered = list_staff(
            &conn,
            StaffScope::Search,
            0,
            50,
            SortField::Name,
            SortDir::Asc,
            Some(&ast),
            false,
            None,
            false,
            &[],
        )
        .unwrap();
        assert_eq!(
            filtered
                .staff
                .iter()
                .map(|staff| staff.uid)
                .collect::<Vec<_>>(),
            [2, 3]
        );

        let sorted = list_staff(
            &conn,
            StaffScope::Search,
            0,
            50,
            SortField::parse("role.coach_fitness").unwrap(),
            SortDir::Desc,
            None,
            false,
            None,
            false,
            &[],
        )
        .unwrap();
        assert_eq!(
            sorted
                .staff
                .iter()
                .map(|staff| staff.uid)
                .collect::<Vec<_>>(),
            [1, 2, 3]
        );
    }

    #[test]
    fn accepts_every_configurable_basic_staff_sort_field() {
        for field in [
            "name",
            "age",
            "birth_year",
            "birth_day_of_year",
            "nationality",
            "nation_uid",
            "gender",
            "club",
            "division",
            "ca",
            "pa",
            "wage",
            "contract_year",
            "contract_day",
            "job_id",
        ] {
            assert!(SortField::parse(field).is_ok(), "rejected {field}");
        }
    }
    #[test]
    fn reports_setup_states_and_bounds_stable_pages() {
        let conn = open();
        assert_eq!(
            list_staff(
                &conn,
                StaffScope::Search,
                0,
                50,
                SortField::Ca,
                SortDir::Desc,
                None,
                false,
                None,
                false,
                &[]
            )
            .unwrap()
            .state,
            StaffPageState::NoCurrentSnapshot
        );
        seed(&conn, false);
        assert_eq!(
            list_staff(
                &conn,
                StaffScope::MyStaff,
                0,
                50,
                SortField::Ca,
                SortDir::Desc,
                None,
                false,
                None,
                false,
                &[]
            )
            .unwrap()
            .state,
            StaffPageState::NoManagedClub
        );
        let page = list_staff(
            &conn,
            StaffScope::Search,
            1,
            1,
            SortField::Ca,
            SortDir::Desc,
            None,
            false,
            None,
            false,
            &[],
        )
        .unwrap();
        assert_eq!(page.total, 3);
        assert_eq!(page.staff.len(), 1);
        assert_eq!(page.staff[0].uid, 1);
    }
    #[test]
    fn compact_profile_rejects_missing_and_wrong_version_without_row_multiplication() {
        let conn = open();
        seed(&conn, true);
        // Missing compact row must fail the read.
        conn.execute(
            "DELETE FROM staff_role_metrics WHERE snapshot_id = 1 AND uid = 1",
            [],
        )
        .expect("delete compact row");
        assert_eq!(
            get_staff(&conn, 1).unwrap_err(),
            "Current compact staff snapshot is incomplete"
        );
        // Wrong version must also fail.
        conn.execute(
            "INSERT INTO staff_role_metrics (snapshot_id, uid, score_model_version, coach_fitness) VALUES (1, 1, 999, 80)",
            [],
        )
        .expect("insert wrong version");
        assert_eq!(
            get_staff(&conn, 1).unwrap_err(),
            "Current compact staff snapshot is incomplete"
        );
        // List with a role filter must reject the incomplete snapshot.
        let ast = parse_filter_ast(
            vec![FilterRule {
                field: "role.coach_fitness".into(),
                op: "gt".into(),
                value: FilterValue::Integer(10),
            }],
            None,
        )
        .unwrap();
        assert_eq!(
            list_staff(
                &conn,
                StaffScope::Search,
                0,
                50,
                SortField::Ca,
                SortDir::Desc,
                Some(&ast),
                false,
                None,
                false,
                &[]
            )
            .unwrap_err(),
            "Current compact staff snapshot is incomplete"
        );
        // Scalar-only table reads (no role field, filter, or sort) must reject
        // the same incomplete state instead of returning raw staff rows.
        assert_eq!(
            list_staff(
                &conn,
                StaffScope::Search,
                0,
                50,
                SortField::Ca,
                SortDir::Desc,
                None,
                false,
                None,
                false,
                &[]
            )
            .unwrap_err(),
            "Current compact staff snapshot is incomplete"
        );
        // One row per staff: compact join must not multiply rows.
        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM staff s LEFT JOIN staff_role_metrics m ON m.snapshot_id = s.snapshot_id AND m.uid = s.uid AND m.score_model_version = 1 WHERE s.snapshot_id = 1",
                [],
                |row| row.get(0),
            )
            .expect("count compact join");
        assert_eq!(total, 3);
    }

    struct StaffShape {
        name: &'static str,
        sort: &'static str,
        dir: SortDir,
        filters: Option<FilterAst>,
        fields: Vec<String>,
        shortlist_only: bool,
        preferred_job: Option<&'static str>,
        unemployed_only: bool,
    }

    impl StaffShape {
        fn new(name: &'static str, sort: &'static str, dir: SortDir, fields: &[&str]) -> Self {
            Self {
                name,
                sort,
                dir,
                filters: None,
                fields: fields.iter().map(|field| field.to_string()).collect(),
                shortlist_only: false,
                preferred_job: None,
                unemployed_only: false,
            }
        }

        fn filtered(self, rules: Vec<(&str, &str, FilterValue)>, combine: &str) -> Self {
            let rules = rules
                .into_iter()
                .map(|(field, op, value)| FilterRule {
                    field: field.into(),
                    op: op.into(),
                    value,
                })
                .collect();
            Self {
                filters: Some(parse_filter_ast(rules, Some(combine)).expect("filters")),
                ..self
            }
        }

        fn shortlisted(self, preferred_job: Option<&'static str>, unemployed_only: bool) -> Self {
            Self {
                shortlist_only: true,
                preferred_job,
                unemployed_only,
                ..self
            }
        }

        fn request(&self, offset: usize, limit: usize) -> StaffSearchRequest<'_> {
            StaffSearchRequest {
                offset,
                limit,
                sort: SortField::parse(self.sort).expect("sort"),
                direction: self.dir,
                filters: self.filters.as_ref(),
                shortlist_only: self.shortlist_only,
                preferred_job: self.preferred_job,
                unemployed_only: self.unemployed_only,
                requested_fields: &self.fields,
            }
        }

        fn uncached(&self, db: &Db, offset: usize, limit: usize) -> Result<StaffPage, String> {
            list_staff(
                &db.read().expect("reader"),
                StaffScope::Search,
                offset,
                limit,
                SortField::parse(self.sort).expect("sort"),
                self.dir,
                self.filters.as_ref(),
                self.shortlist_only,
                self.preferred_job,
                self.unemployed_only,
                &self.fields,
            )
        }
    }

    /// A database file with readers whose staff tie and miss values across the Staff sorts,
    /// beside an older snapshot and an inactive save's shortlist.
    fn open_staff_equivalence_db() -> (tempfile::TempDir, Db) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = crate::db::open(&temp_dir.path().join(crate::db::APP_DB_FILE)).expect("open db");
        let writer = db.0.lock().expect("lock writer");
        seed(&writer, false);
        writer
            .execute_batch(
                r#"INSERT INTO staff (
                    snapshot_id, uid, name, age, nationalities_json, gender, ca, pa,
                    staff_attributes_json, club
                 ) VALUES
                    (1, 4, 'alpha', 40, '["ENG"]', 'male', 110, 120, '{"Authority":12}', NULL),
                    (1, 5, 'Delta', 39, '[]', 'male', 100, 140, '{"Authority":12}', 'Club B'),
                    (1, 6, 'echo', 38, '["DEN"]', 'female', 90, 90, '{}', 'Club A'),
                    (1, 7, 'Foxtrot', 37, '[]', 'male', 110, 110, '{"Authority":20}', NULL),
                    (1, 8, 'golf', 36, '[]', 'unknown', 120, 125, '{"Authority":5}', 'Other');
                 INSERT INTO staff_role_metrics (snapshot_id, uid, score_model_version, coach_fitness)
                 VALUES (1, 4, 1, 80), (1, 5, 1, NULL), (1, 6, 1, 55), (1, 7, 1, 70), (1, 8, 1, 80);
                 INSERT INTO saves (id, name, is_active) VALUES (2, 'Inactive', 0);
                 INSERT INTO staff_shortlist_entries (
                    save_id, staff_uid, preferred_job, coaching_qualifications
                 ) VALUES (2, 4, 'Wrong save', 'Wrong qualification');"#,
            )
            .expect("seed staff");
        drop(writer);
        (temp_dir, db)
    }

    /// Joins three-row cached windows and compares each with the uncached page at its offset.
    fn assert_cached_windows_match(db: &Db, orders: &StaffOrderCache, shape: &StaffShape) {
        let registry = TableRegistry::default();
        let complete = shape
            .uncached(db, 0, MAX_PAGE_LIMIT)
            .expect("uncached complete page");
        let mut joined = Vec::new();
        for offset in (0..complete.staff.len() + 3).step_by(3) {
            let cached = search_staff_page(db, orders, &registry, None, shape.request(offset, 3))
                .unwrap_or_else(|error| panic!("{}: {error}", shape.name));
            let uncached = shape.uncached(db, offset, 3).expect("uncached page");
            assert_eq!(cached, uncached, "{} at offset {offset}", shape.name);
            joined.extend(cached.staff);
        }
        assert_eq!(joined, complete.staff, "{} joined windows", shape.name);
    }

    #[test]
    fn cached_staff_windows_equal_uncached_pages_for_every_shape() {
        let (_temp_dir, db) = open_staff_equivalence_db();
        // The name-sorted shapes differ only in the shortlist flag, Preferred Job, and the
        // unemployed filter, so one cache holding them all proves each is in the fingerprint.
        let orders = StaffOrderCache::new(32);
        let shortlist_shapes = [
            StaffShape::new("shortlist by name", "name", SortDir::Asc, &[])
                .shortlisted(None, false),
            StaffShape::new("unemployed by name", "name", SortDir::Asc, &[])
                .shortlisted(None, true),
            StaffShape::new("shortlist", "ca", SortDir::Desc, &["role.coach_fitness"])
                .shortlisted(None, false),
            StaffShape::new("Preferred Job sort", "preferred_job", SortDir::Asc, &[])
                .shortlisted(None, false),
            StaffShape::new("Preferred Job", "name", SortDir::Asc, &["attr.Authority"])
                .shortlisted(Some("scout"), false),
            StaffShape::new(
                "unemployed only",
                "coaching_qualifications",
                SortDir::Desc,
                &[],
            )
            .shortlisted(None, true),
            StaffShape::new("shortlist filters", "ca", SortDir::Asc, &[])
                .shortlisted(Some("Scout"), true)
                .filtered(vec![("ca", "gt", FilterValue::Integer(95))], "and"),
        ];
        let shapes = [
            StaffShape::new("CA", "ca", SortDir::Desc, &[]),
            StaffShape::new("name", "name", SortDir::Asc, &["role.coach_fitness"]),
            StaffShape::new(
                "role",
                "role.coach_fitness",
                SortDir::Desc,
                &["role.coach_fitness", "attr.Authority"],
            ),
            StaffShape::new("attribute", "attr.Authority", SortDir::Asc, &[]),
            StaffShape::new("club", "club", SortDir::Asc, &[]),
            StaffShape::new("AND filter", "pa", SortDir::Desc, &[]).filtered(
                vec![
                    ("ca", "gt", FilterValue::Integer(95)),
                    ("role.coach_fitness", "gt", FilterValue::Integer(60)),
                ],
                "and",
            ),
            StaffShape::new("OR filter", "age", SortDir::Asc, &["role.coach_fitness"]).filtered(
                vec![
                    ("ca", "eq", FilterValue::Integer(90)),
                    ("role.coach_fitness", "gt", FilterValue::Integer(75)),
                ],
                "or",
            ),
            StaffShape::new("empty", "ca", SortDir::Desc, &[])
                .filtered(vec![("ca", "gt", FilterValue::Integer(999))], "and"),
        ];
        // Without entries, every shortlist shape reports the missing shortlist.
        for shape in shapes.iter().chain(&shortlist_shapes) {
            assert_cached_windows_match(&db, &orders, shape);
        }

        db.0.lock()
            .expect("lock writer")
            .execute_batch(
                "INSERT INTO staff_shortlist_entries (
                    save_id, staff_uid, preferred_job, coaching_qualifications
                 ) VALUES
                    (1, 1, 'Physio', 'Continental Pro'),
                    (1, 2, 'Scout', 'National C'),
                    (1, 3, 'scout', 'National B'),
                    (1, 4, 'Scout', 'National B'),
                    (1, 7, 'Analyst', 'None'),
                    (1, 99, 'Scout', 'Departed');",
            )
            .expect("seed shortlist");
        for shape in shapes.iter().chain(&shortlist_shapes) {
            assert_cached_windows_match(&db, &orders, shape);
        }

        let shortlist_sort_without_shortlist =
            StaffShape::new("shortlist sort", "preferred_job", SortDir::Asc, &[]);
        assert_eq!(
            search_staff_page(
                &db,
                &orders,
                &TableRegistry::default(),
                None,
                shortlist_sort_without_shortlist.request(0, 3)
            ),
            shortlist_sort_without_shortlist.uncached(&db, 0, 3)
        );
    }

    #[test]
    fn later_staff_windows_reuse_the_order_until_a_write() {
        let (_temp_dir, db) = open_staff_equivalence_db();
        let orders = StaffOrderCache::new(4);
        let registry = TableRegistry::default();
        let shape = StaffShape::new("CA", "ca", SortDir::Desc, &[]);
        let with_column = StaffShape::new("CA", "ca", SortDir::Desc, &["role.coach_fitness"]);
        ORDER_COMPUTATIONS.with(|count| count.set(0));

        search_staff_page(&db, &orders, &registry, None, shape.request(0, 3))
            .expect("first window");
        search_staff_page(&db, &orders, &registry, None, shape.request(6, 3))
            .expect("later window");
        search_staff_page(&db, &orders, &registry, None, with_column.request(3, 3))
            .expect("window with another column");
        assert_eq!(ORDER_COMPUTATIONS.with(Cell::get), 1);

        db.0.lock()
            .expect("lock writer")
            .execute("UPDATE staff SET ca = 200 WHERE uid = 6", [])
            .expect("change a staff member");
        let fresh = search_staff_page(&db, &orders, &registry, None, shape.request(0, 3))
            .expect("window after a write");
        assert_eq!(ORDER_COMPUTATIONS.with(Cell::get), 2);
        assert_eq!(fresh.staff[0].uid, 6);
        assert_eq!(fresh, shape.uncached(&db, 0, 3).expect("uncached page"));
    }

    #[test]
    fn a_newer_staff_query_supersedes_a_running_one_and_completes() {
        let (_temp_dir, db) = open_staff_equivalence_db();
        let orders = StaffOrderCache::new(4);
        let registry = TableRegistry::default();
        let older = StaffShape::new("CA", "ca", SortDir::Desc, &[]);
        let newer = || StaffShape::new("name", "name", SortDir::Asc, &[]);
        let older_ticket = registry.arrive(
            "staff-search",
            &staff_query_fingerprint(&older.request(0, 3)),
        );

        let (sender, receiver) = std::sync::mpsc::channel();
        {
            let (db, orders, registry) = (db.clone(), orders.clone(), registry.clone());
            BEFORE_ORDER_QUERY.with(|hook| {
                *hook.borrow_mut() = Some(Box::new(move || {
                    let newer = newer();
                    let ticket = registry.arrive(
                        "staff-search",
                        &staff_query_fingerprint(&newer.request(0, 3)),
                    );
                    sender
                        .send(search_staff_page(
                            &db,
                            &orders,
                            &registry,
                            Some(&ticket),
                            newer.request(0, 3),
                        ))
                        .expect("send newer page");
                }));
            });
        }

        let superseded = search_staff_page(
            &db,
            &orders,
            &registry,
            Some(&older_ticket),
            older.request(0, 3),
        );
        assert_eq!(
            superseded,
            Err(crate::db::result_cache::SUPERSEDED.to_string())
        );
        assert_eq!(
            receiver.recv().expect("newer page"),
            newer().uncached(&db, 0, 3)
        );
    }
}
