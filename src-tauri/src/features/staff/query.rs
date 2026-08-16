use std::collections::BTreeMap;

use rusqlite::{params_from_iter, types::Value, Connection, OptionalExtension, Row};

use super::filter::{compile_filters, FilterAst};
use super::metrics::{parse_requested_fields, MetricField};

pub const DEFAULT_PAGE_LIMIT: usize = 50;
pub const MAX_PAGE_LIMIT: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffScope {
    Search,
    MyStaff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffPageState {
    Ready,
    NoCurrentSnapshot,
    NoClubFamily,
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
    Nationality,
    Club,
    Division,
    Ca,
    Pa,
    JobId,
    Wage,
    ContractYear,
    Dynamic(MetricField),
}
impl SortField {
    pub const DEFAULT: Self = Self::Ca;
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "name" => Ok(Self::Name),
            "age" => Ok(Self::Age),
            "nationality" => Ok(Self::Nationality),
            "club" => Ok(Self::Club),
            "division" => Ok(Self::Division),
            "ca" => Ok(Self::Ca),
            "pa" => Ok(Self::Pa),
            "job_id" => Ok(Self::JobId),
            "wage" => Ok(Self::Wage),
            "contract_year" => Ok(Self::ContractYear),
            other => Ok(Self::Dynamic(MetricField::parse(other)?)),
        }
    }
    fn expr(&self) -> String {
        match self {
            Self::Name => "staff.name COLLATE NOCASE".into(),
            Self::Age => "staff.age".into(),
            Self::Nationality => "staff.nationalities_json COLLATE NOCASE".into(),
            Self::Club => "staff.club COLLATE NOCASE".into(),
            Self::Division => "staff.division COLLATE NOCASE".into(),
            Self::Ca => "staff.ca".into(),
            Self::Pa => "staff.pa".into(),
            Self::JobId => "staff.job_id".into(),
            Self::Wage => "staff.weekly_wage_gbp".into(),
            Self::ContractYear => "staff.contract_expiry_year".into(),
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffPage {
    pub state: StaffPageState,
    pub staff: Vec<StaffSummary>,
    pub total: i64,
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
    requested_fields: &[String],
) -> Result<StaffPage, String> {
    let dynamic_fields = parse_requested_fields(requested_fields)?;
    let compiled = filters
        .map(|filters| compile_filters(filters, if scope == StaffScope::MyStaff { 3 } else { 2 }))
        .transpose()?;
    let context: Option<(i64, i64)> = conn.query_row(
        "SELECT snapshots.id, snapshots.save_id FROM snapshots INNER JOIN saves ON saves.id = snapshots.save_id AND saves.is_active = 1 WHERE snapshots.is_current = 1 LIMIT 1",
        [], |row| Ok((row.get(0)?, row.get(1)?)),
    ).optional().map_err(|error| error.to_string())?;
    let Some((snapshot_id, save_id)) = context else {
        return Ok(empty(StaffPageState::NoCurrentSnapshot));
    };

    if scope == StaffScope::MyStaff {
        let configured: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM planner_club_sources WHERE save_id = ?1)",
                [save_id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        if !configured {
            return Ok(empty(StaffPageState::NoClubFamily));
        }
    }

    let mut binds = vec![Value::Integer(snapshot_id)];
    let mut where_sql = "staff.snapshot_id = ?1".to_string();
    if scope == StaffScope::MyStaff {
        binds.push(Value::Integer(save_id));
        where_sql.push_str(" AND EXISTS(SELECT 1 FROM planner_club_sources source WHERE source.save_id = ?2 AND source.club_name = staff.club)");
    }
    if let Some(compiled) = compiled {
        if !compiled.sql.is_empty() {
            where_sql.push_str(" AND ");
            where_sql.push_str(&compiled.sql);
            binds.extend(compiled.params);
        }
    }

    let total: i64 = conn
        .query_row(
            &format!("SELECT COUNT(*) FROM staff WHERE {where_sql}"),
            params_from_iter(binds.iter()),
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    let limit = i64::try_from(limit.clamp(1, MAX_PAGE_LIMIT))
        .map_err(|_| "staff limit out of range".to_string())?;
    let offset = i64::try_from(offset).map_err(|_| "staff offset out of range".to_string())?;
    let expr = sort.expr();
    let order = format!(
        "ORDER BY ({expr}) IS NULL ASC, {expr} {}, staff.uid ASC",
        direction.sql()
    );
    let mut sql = String::from("SELECT staff.uid, staff.name, staff.age, staff.birth_year, staff.birth_day_of_year, staff.nationalities_json, staff.nation_uid, staff.gender, staff.club, staff.division, staff.ca, staff.pa, staff.job_id, staff.weekly_wage_gbp, staff.contract_expiry_year, staff.contract_expiry_day_of_year");
    for field in &dynamic_fields {
        sql.push_str(", ");
        sql.push_str(&field.sql_expression("staff"));
    }
    let limit_index = binds.len() + 1;
    let offset_index = binds.len() + 2;
    sql.push_str(&format!(
        " FROM staff WHERE {where_sql} {order} LIMIT ?{limit_index} OFFSET ?{offset_index}"
    ));
    binds.push(Value::Integer(limit));
    binds.push(Value::Integer(offset));
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let staff = statement
        .query_map(params_from_iter(binds.iter()), |row| {
            map_staff(row, &dynamic_fields)
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(StaffPage {
        state: StaffPageState::Ready,
        staff,
        total,
    })
}

fn empty(state: StaffPageState) -> StaffPage {
    StaffPage {
        state,
        staff: Vec::new(),
        total: 0,
    }
}
fn map_staff(row: &Row<'_>, fields: &[MetricField]) -> rusqlite::Result<StaffSummary> {
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::features::staff::filter::{parse_filter_ast, FilterRule, FilterValue};
    fn open() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", true).unwrap();
        migrations::apply(&conn).unwrap();
        conn
    }
    fn seed(conn: &Connection, family: bool) {
        conn.execute_batch("INSERT INTO saves (id,name,is_active) VALUES (1,'Save',1); INSERT INTO snapshots (id,save_id,is_current,schema_version,generated_at_utc,game_version,supported_game_version,bridge_version,protocol_version,game_date_source,scan_truncated,player_count) VALUES (1,1,1,8,'now','26.3','26.3','0.4',1,'unknown',0,0),(2,1,0,8,'old','26.3','26.3','0.4',1,'unknown',0,0); INSERT INTO staff (snapshot_id,uid,name,age,nationalities_json,gender,ca,pa,staff_attributes_json,club) VALUES (1,1,'Alpha',40,'[\"DEN\"]','male',100,120,'{\"Authority\":18}','Club A'),(1,2,'Beta',41,'[\"SWE\"]','female',110,130,'{\"Authority\":15}','Club B'),(1,3,'Gamma',42,'[]','unknown',90,100,'{\"Authority\":null}','Other'),(2,9,'Old',50,'[]','unknown',200,200,'{}','Club A'); INSERT INTO staff_role_scores (snapshot_id,uid,role_id,score) VALUES (1,1,'coach_fitness',80),(1,2,'coach_fitness',70);").unwrap();
        if family {
            conn.execute_batch("INSERT INTO planner_club_settings (save_id,primary_club) VALUES (1,'Club A'); INSERT INTO planner_club_sources (save_id,team,club_name,is_primary) VALUES (1,'senior','Club A',1),(1,'reserves','Club B',0);").unwrap();
        }
    }
    #[test]
    fn search_is_all_current_while_my_staff_uses_every_family_club() {
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
            &[],
        )
        .unwrap();
        assert_eq!(
            search.staff.iter().map(|s| s.uid).collect::<Vec<_>>(),
            [1, 2, 3]
        );
        assert_eq!(mine.staff.iter().map(|s| s.uid).collect::<Vec<_>>(), [1, 2]);
        let second_page = list_staff(
            &conn,
            StaffScope::MyStaff,
            1,
            1,
            SortField::Name,
            SortDir::Asc,
            None,
            &[],
        )
        .unwrap();
        assert_eq!(second_page.total, 2);
        assert_eq!(second_page.staff[0].uid, 2);
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
            &["attr.Authority".into(), "role.coach_fitness".into()],
        )
        .unwrap();
        assert_eq!(page.staff.len(), 1);
        assert_eq!(page.staff[0].dynamic_values["attr.Authority"], Some(18));
        assert_eq!(page.staff[0].dynamic_values["role.coach_fitness"], Some(80));
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
                &[]
            )
            .unwrap()
            .state,
            StaffPageState::NoClubFamily
        );
        let page = list_staff(
            &conn,
            StaffScope::Search,
            1,
            1,
            SortField::Ca,
            SortDir::Desc,
            None,
            &[],
        )
        .unwrap();
        assert_eq!(page.total, 3);
        assert_eq!(page.staff.len(), 1);
        assert_eq!(page.staff[0].uid, 1);
    }
    #[test]
    fn query_plan_uses_staff_score_owner_index() {
        let conn = open();
        seed(&conn, true);
        let plan:String=conn.query_row("EXPLAIN QUERY PLAN SELECT score FROM staff_role_scores WHERE snapshot_id=1 AND uid=1 AND role_id='coach_fitness'",[],|row|row.get(3)).unwrap();
        assert!(
            plan.contains("sqlite_autoindex_staff_role_scores_1") || plan.contains("INDEX"),
            "{plan}"
        );
    }
}
