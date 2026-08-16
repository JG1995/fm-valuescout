use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::Db;
use crate::features::memory_read::service::{
    request_staff_boost_from_local_app_data, DumpWaitConfig, StaffBoostResult,
};
use crate::features::player::boost_gate;

use super::filter::{self, FilterAst, FilterRule};
use super::query::{
    self, SortDir, SortField, StaffDetail, StaffPage, StaffPageState, StaffRoleScore, StaffScope,
    StaffSummary,
};
use super::service::{self, PreparedStaffBoost, StaffBoostError, VerifiedStaffBoost};

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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffRoleScoreDto {
    pub role_id: String,
    pub display_name: String,
    pub score: Option<i64>,
}

impl From<StaffRoleScore> for StaffRoleScoreDto {
    fn from(role: StaffRoleScore) -> Self {
        Self {
            role_id: role.role_id,
            display_name: role.display_name,
            score: role.score,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffDetailDto {
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
    pub role_scores: Vec<StaffRoleScoreDto>,
}

impl From<StaffDetail> for StaffDetailDto {
    fn from(staff: StaffDetail) -> Self {
        Self {
            uid: staff.uid,
            name: staff.name,
            age: staff.age,
            birth_year: staff.birth_year,
            birth_day_of_year: staff.birth_day_of_year,
            nationalities: staff.nationalities,
            nation_uid: staff.nation_uid,
            gender: staff.gender,
            club: staff.club,
            division: staff.division,
            ca: staff.ca,
            pa: staff.pa,
            job_id: staff.job_id,
            weekly_wage_gbp: staff.weekly_wage_gbp,
            contract_expiry_year: staff.contract_expiry_year,
            contract_expiry_day_of_year: staff.contract_expiry_day_of_year,
            attributes: staff.attributes,
            hidden_information_revealed: staff.hidden_information_revealed,
            role_scores: staff
                .role_scores
                .into_iter()
                .map(StaffRoleScoreDto::from)
                .collect(),
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

#[tauri::command]
pub fn get_staff(uid: i64, db: State<'_, Db>) -> Result<Option<StaffDetailDto>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    query::get_staff(&conn, uid).map(|staff| staff.map(StaffDetailDto::from))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Debug)]
pub struct StaffBoostResultDto {
    pub snapshot_id: i64,
    pub operation: String,
    pub previous_current_ability: i64,
    pub current_ability: i64,
    pub potential_ability: i64,
}

impl From<VerifiedStaffBoost> for StaffBoostResultDto {
    fn from(result: VerifiedStaffBoost) -> Self {
        Self {
            snapshot_id: result.snapshot_id,
            operation: result.operation,
            previous_current_ability: result.previous_current_ability,
            current_ability: result.current_ability,
            potential_ability: result.potential_ability,
        }
    }
}

#[tauri::command]
pub fn boost_staff_current_ability(
    uid: i64,
    db: State<'_, Db>,
) -> Result<StaffBoostResultDto, StaffBoostError> {
    execute_staff_boost_with(uid, db.inner(), request_local_staff_boost)
}

fn execute_staff_boost_with<F>(
    uid: i64,
    db: &Db,
    request_bridge_boost: F,
) -> Result<StaffBoostResultDto, StaffBoostError>
where
    F: FnOnce(&PreparedStaffBoost) -> Result<StaffBoostResult, StaffBoostError>,
{
    let _boost_guard =
        boost_gate::acquire_boost_gate().map_err(|message| StaffBoostError::Bridge {
            kind: "inProgress".to_string(),
            message,
        })?;
    let prepared = {
        let conn = db.0.lock().map_err(|_| StaffBoostError::Eligibility {
            kind: "databaseUnavailable".to_string(),
            message: "could not read the current snapshot for this staff boost".to_string(),
        })?;
        service::prepare_current_ability_boost(&conn, uid)?
    };
    let result = match request_bridge_boost(&prepared) {
        Ok(result) => result,
        Err(error @ StaffBoostError::SnapshotSync { .. }) => {
            mark_recovery_required(db, &prepared)?;
            return Err(error);
        }
        Err(error) => return Err(error),
    };
    let reconciled = {
        let mut conn = db.0.lock().map_err(|_| StaffBoostError::SnapshotSync {
            message: "FM changed, but the local database is unavailable. Load Data again."
                .to_string(),
        })?;
        service::reconcile_verified_boost(&mut conn, &prepared, result)
    };
    match reconciled {
        Ok(result) => Ok(result.into()),
        Err(error) => {
            mark_recovery_required(db, &prepared)?;
            Err(error)
        }
    }
}

fn request_local_staff_boost(
    prepared: &PreparedStaffBoost,
) -> Result<StaffBoostResult, StaffBoostError> {
    request_staff_boost_from_local_app_data(
        &prepared.source_request_id,
        prepared.staff_uid,
        prepared.expected_current_ability as i32,
        prepared.expected_potential_ability as i32,
        DumpWaitConfig::default(),
    )
    .map_err(service::map_bridge_error)
}

fn mark_recovery_required(db: &Db, prepared: &PreparedStaffBoost) -> Result<(), StaffBoostError> {
    let conn = db.0.lock().map_err(|_| StaffBoostError::SnapshotSync {
        message:
            "FM may have changed, but the recovery requirement could not be saved. Load Data again."
                .to_string(),
    })?;
    service::require_load_data_for_boost(&conn, &prepared.context())
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use rusqlite::Connection;

    use super::*;
    use crate::db::migrations;
    use crate::features::memory_read::service::OPERATION_BOOST_STAFF_CURRENT_ABILITY;
    use crate::features::player::boost_gate::BOOST_TEST_GATE;
    use crate::features::player::service::prepare_current_ability_boost_for_test;
    use crate::features::snapshot::ingest::ingest_dump_file;

    const GOLDEN_FIXTURE: &str = include_str!("../memory_read/fixtures/golden_dump_v8.json");

    fn seeded_db(ca: i64, pa: i64) -> (tempfile::TempDir, Db) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn =
            Connection::open(temp_dir.path().join("staff-command.db")).expect("open test database");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("foreign keys");
        migrations::apply(&conn).expect("migrations");
        let mut dump: serde_json::Value = serde_json::from_str(GOLDEN_FIXTURE).expect("fixture");
        dump["staff"][0]["ca"] = serde_json::Value::from(ca);
        dump["staff"][0]["pa"] = serde_json::Value::from(pa);
        let path = temp_dir.path().join("staff.json");
        std::fs::write(&path, dump.to_string()).expect("write fixture");
        let snapshot = ingest_dump_file(&mut conn, &path).expect("ingest fixture");
        conn.execute(
            "UPDATE snapshots SET bridge_source_request_id = 'scan-staff-command'
             WHERE id = ?1",
            [snapshot.id],
        )
        .expect("bind scan");
        (temp_dir, Db(Mutex::new(conn)))
    }

    fn verified_result(prepared: &PreparedStaffBoost, current_ability: i32) -> StaffBoostResult {
        StaffBoostResult {
            operation: OPERATION_BOOST_STAFF_CURRENT_ABILITY.to_string(),
            outcome: "verified".to_string(),
            rollback: "not-needed".to_string(),
            previous_current_ability: Some(prepared.expected_current_ability as i32),
            current_ability: Some(current_ability),
            potential_ability: Some(prepared.expected_potential_ability as i32),
        }
    }
    #[test]
    fn rejects_non_scalar_filter_values() {
        assert!(filter::filter_value_from_json(serde_json::json!({"x":1})).is_err());
        assert!(matches!(
            filter::filter_value_from_json(serde_json::json!(4)).unwrap(),
            filter::FilterValue::Integer(4)
        ));
    }

    #[test]
    fn staff_and_player_boosts_share_one_application_gate() {
        let _test_guard = BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let gate = boost_gate::acquire_boost_gate().expect("hold shared boost gate");
        let db = Db(Mutex::new(
            Connection::open_in_memory().expect("open test database"),
        ));
        let error = execute_staff_boost_with(88, &db, |_| {
            panic!("staff request must not reach the bridge while the shared gate is held")
        })
        .expect_err("concurrent boost must be rejected");
        drop(gate);

        assert!(matches!(
            error,
            StaffBoostError::Bridge { ref kind, .. } if kind == "inProgress"
        ));
    }

    #[test]
    fn uncertain_and_unreconciled_results_latch_recovery_for_staff_and_players() {
        let _test_guard = BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        for failure in ["uncertain", "unreconciled"] {
            let (_temp, db) = seeded_db(115, 140);
            let error = execute_staff_boost_with(88, &db, |prepared| match failure {
                "uncertain" => Err(StaffBoostError::SnapshotSync {
                    message: "FM may have changed before verification".to_string(),
                }),
                _ => Ok(verified_result(prepared, 126)),
            })
            .expect_err("unsafe outcome must fail");
            assert!(matches!(error, StaffBoostError::SnapshotSync { .. }));

            let conn = db.0.lock().expect("lock database");
            let recovery: i64 = conn
                .query_row(
                    "SELECT boost_recovery_required FROM snapshots WHERE is_current = 1",
                    [],
                    |row| row.get(0),
                )
                .expect("read recovery");
            assert_eq!(recovery, 1, "{failure} must latch recovery");
            assert!(matches!(
                service::prepare_current_ability_boost(&conn, 88),
                Err(StaffBoostError::SnapshotSync { .. })
            ));
            assert!(prepare_current_ability_boost_for_test(&conn, 77).is_err());
        }
    }

    #[test]
    fn proven_no_write_failures_leave_shared_recovery_clear() {
        let _test_guard = BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        for error in [
            StaffBoostError::Bridge {
                kind: "rejected".to_string(),
                message: "Load Data again before using staff boosts".to_string(),
            },
            StaffBoostError::LiveValue {
                message: "staff values changed in FM; Load Data again".to_string(),
            },
        ] {
            let (_temp, db) = seeded_db(115, 140);
            execute_staff_boost_with(88, &db, |_| Err(error))
                .expect_err("proven no-write failure must be returned");
            let conn = db.0.lock().expect("lock database");
            let recovery: i64 = conn
                .query_row(
                    "SELECT boost_recovery_required FROM snapshots WHERE is_current = 1",
                    [],
                    |row| row.get(0),
                )
                .expect("read recovery");
            assert_eq!(recovery, 0);
            assert!(service::prepare_current_ability_boost(&conn, 88).is_ok());
            assert!(prepare_current_ability_boost_for_test(&conn, 77).is_ok());
        }
    }
}
