use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::features::memory_read::service::{
    PlayerBoostRequestError, StaffBoostResult as BridgeStaffBoostResult,
    OPERATION_BOOST_STAFF_CURRENT_ABILITY,
};

const MIN_ABILITY: i64 = 1;
const MAX_ABILITY: i64 = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PreparedStaffBoost {
    pub(super) snapshot_id: i64,
    pub(super) snapshot_context_token: String,
    pub(super) save_id: i64,
    pub(super) save_context_token: String,
    pub(super) source_request_id: String,
    pub(super) staff_uid: u32,
    pub(super) expected_current_ability: i64,
    pub(super) expected_potential_ability: i64,
    pub(super) target_current_ability: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaffBoostContext {
    pub(super) snapshot_id: i64,
    pub(super) snapshot_context_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaffBoostBatchContext {
    pub(super) snapshot_id: i64,
    pub(super) snapshot_context_token: String,
    pub(super) save_id: i64,
    pub(super) save_context_token: String,
    pub(super) source_request_id: String,
}

impl StaffBoostBatchContext {
    pub(super) fn recovery_context(&self) -> StaffBoostContext {
        StaffBoostContext {
            snapshot_id: self.snapshot_id,
            snapshot_context_token: self.snapshot_context_token.clone(),
        }
    }
}

impl PreparedStaffBoost {
    pub(super) fn context(&self) -> StaffBoostContext {
        StaffBoostContext {
            snapshot_id: self.snapshot_id,
            snapshot_context_token: self.snapshot_context_token.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct VerifiedStaffBoost {
    pub(super) snapshot_id: i64,
    pub(super) operation: String,
    pub(super) previous_current_ability: i64,
    pub(super) current_ability: i64,
    pub(super) potential_ability: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "phase", rename_all = "camelCase")]
pub enum StaffBoostError {
    Eligibility { kind: String, message: String },
    Bridge { kind: String, message: String },
    LiveValue { message: String },
    SnapshotSync { message: String },
}

impl std::fmt::Display for StaffBoostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eligibility { message, .. }
            | Self::Bridge { message, .. }
            | Self::LiveValue { message }
            | Self::SnapshotSync { message } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for StaffBoostError {}

pub(super) fn prepare_current_ability_boost(
    conn: &Connection,
    uid: i64,
) -> Result<PreparedStaffBoost, StaffBoostError> {
    let staff_uid = u32::try_from(uid)
        .ok()
        .filter(|uid| *uid != 0)
        .ok_or_else(|| eligibility_error("invalidStaff", "staff identity is invalid"))?;
    let context = capture_boost_context(conn)?;
    let staff: Option<(i64, i64)> = conn
        .query_row(
            "SELECT ca, pa FROM staff WHERE snapshot_id = ?1 AND uid = ?2",
            params![context.snapshot_id, uid],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(database_eligibility_error)?;
    let (current_ability, potential_ability) = staff.ok_or_else(|| {
        eligibility_error(
            "staffNotFound",
            "staff member is not in the current snapshot; Load Data again",
        )
    })?;
    if !(MIN_ABILITY..=MAX_ABILITY).contains(&current_ability)
        || !(MIN_ABILITY..=MAX_ABILITY).contains(&potential_ability)
        || current_ability > potential_ability
    {
        return Err(eligibility_error(
            "invalidSnapshot",
            "snapshot ability values are invalid; Load Data again",
        ));
    }
    let target_current_ability = (current_ability + 10)
        .min(potential_ability)
        .min(MAX_ABILITY);
    if target_current_ability <= current_ability {
        return Err(eligibility_error(
            "currentAbilityAtLimit",
            "current ability has reached its potential limit",
        ));
    }

    Ok(PreparedStaffBoost {
        snapshot_id: context.snapshot_id,
        snapshot_context_token: context.snapshot_context_token,
        save_id: context.save_id,
        save_context_token: context.save_context_token,
        source_request_id: context.source_request_id,
        staff_uid,
        expected_current_ability: current_ability,
        expected_potential_ability: potential_ability,
        target_current_ability,
    })
}

pub(super) fn map_bridge_error(error: PlayerBoostRequestError) -> StaffBoostError {
    match error {
        PlayerBoostRequestError::Unconfirmed(message) => {
            snapshot_sync_error(format!("FM may have changed. {message}"))
        }
        PlayerBoostRequestError::Timeout(message) => snapshot_sync_error(format!(
            "FM may have changed. Load Data again before retrying ({message})"
        )),
        PlayerBoostRequestError::Failed(message)
            if message == "staff values changed in FM; Load Data again"
                || message == "current ability is already at its potential limit" =>
        {
            StaffBoostError::LiveValue { message }
        }
        PlayerBoostRequestError::Failed(message) => bridge_error("rejected", message),
        PlayerBoostRequestError::UnsupportedPlatform(message) => {
            bridge_error("unsupportedPlatform", message)
        }
        PlayerBoostRequestError::Missing(message) => bridge_error("missing", message),
        PlayerBoostRequestError::Corrupt(message) => bridge_error("corrupt", message),
        PlayerBoostRequestError::WriteFailed(message) => bridge_error("writeFailed", message),
        PlayerBoostRequestError::Unavailable(message) => bridge_error("unavailable", message),
    }
}

pub(super) fn reconcile_verified_boost(
    conn: &mut Connection,
    prepared: &PreparedStaffBoost,
    result: BridgeStaffBoostResult,
) -> Result<VerifiedStaffBoost, StaffBoostError> {
    let verified = verify_result(prepared, result)?;
    let tx = conn.transaction().map_err(database_sync_error)?;
    let current: Option<(Option<String>, i64, i64)> = tx
        .query_row(
            "SELECT s.bridge_source_request_id, st.ca, st.pa
             FROM snapshots s
             INNER JOIN saves sv ON sv.id = s.save_id AND sv.is_active = 1
             INNER JOIN staff st ON st.snapshot_id = s.id AND st.uid = ?5
             WHERE s.id = ?1 AND s.save_id = ?2 AND s.context_token = ?3
               AND sv.context_token = ?4 AND s.is_current = 1",
            params![
                prepared.snapshot_id,
                prepared.save_id,
                prepared.snapshot_context_token,
                prepared.save_context_token,
                i64::from(prepared.staff_uid)
            ],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(database_sync_error)?;
    let Some((source_request_id, current_ability, potential_ability)) = current else {
        return Err(snapshot_sync_error(
            "FM changed, but this snapshot is no longer current. Load Data again.",
        ));
    };
    if source_request_id.as_deref() != Some(prepared.source_request_id.as_str())
        || current_ability != prepared.expected_current_ability
        || potential_ability != prepared.expected_potential_ability
    {
        return Err(snapshot_sync_error(
            "FM changed, but the snapshot no longer matches this boost. Load Data again.",
        ));
    }
    tx.execute(
        "UPDATE staff SET ca = ?1 WHERE snapshot_id = ?2 AND uid = ?3",
        params![
            verified.current_ability,
            prepared.snapshot_id,
            i64::from(prepared.staff_uid)
        ],
    )
    .map_err(database_sync_error)?;
    tx.commit().map_err(database_sync_error)?;
    Ok(verified)
}

pub(super) fn require_load_data_for_boost(
    conn: &Connection,
    context: &StaffBoostContext,
) -> Result<(), StaffBoostError> {
    let changed = conn
        .execute(
            "UPDATE snapshots SET boost_recovery_required = 1
             WHERE id = ?1 AND context_token = ?2",
            params![context.snapshot_id, context.snapshot_context_token],
        )
        .map_err(database_sync_error)?;
    if changed == 1 {
        Ok(())
    } else {
        Err(snapshot_sync_error(
            "FM may have changed, but FM ValueScout could not preserve the recovery requirement. Load Data again.",
        ))
    }
}

pub(super) fn capture_boost_context(
    conn: &Connection,
) -> Result<StaffBoostBatchContext, StaffBoostError> {
    let context: Option<(i64, i64, String, String, Option<String>, i32)> = conn
        .query_row(
            "SELECT s.id, s.save_id, s.context_token, sv.context_token,
                    s.bridge_source_request_id, s.boost_recovery_required
             FROM snapshots s
             INNER JOIN saves sv ON sv.id = s.save_id AND sv.is_active = 1
             WHERE s.is_current = 1 LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .optional()
        .map_err(database_eligibility_error)?;
    let Some((snapshot_id, save_id, snapshot_context_token, save_context_token, source, recovery)) =
        context
    else {
        return Err(eligibility_error(
            "noSnapshot",
            "Load Data before using staff boosts",
        ));
    };
    if recovery != 0 {
        return Err(snapshot_sync_error(
            "Load Data before using staff boosts; the current snapshot could not be reconciled.",
        ));
    }
    let source_request_id = source
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            eligibility_error(
                "missingProvenance",
                "Load Data again before using staff boosts",
            )
        })?;
    Ok(StaffBoostBatchContext {
        snapshot_id,
        save_id,
        snapshot_context_token,
        save_context_token,
        source_request_id,
    })
}

pub(super) fn boost_context_matches(
    conn: &Connection,
    expected: &StaffBoostBatchContext,
) -> Result<bool, StaffBoostError> {
    Ok(capture_boost_context(conn)? == *expected)
}

fn verify_result(
    prepared: &PreparedStaffBoost,
    result: BridgeStaffBoostResult,
) -> Result<VerifiedStaffBoost, StaffBoostError> {
    let values = (
        result.previous_current_ability.map(i64::from),
        result.current_ability.map(i64::from),
        result.potential_ability.map(i64::from),
    );
    if result.operation != OPERATION_BOOST_STAFF_CURRENT_ABILITY
        || result.outcome != "verified"
        || result.rollback != "not-needed"
        || values
            != (
                Some(prepared.expected_current_ability),
                Some(prepared.target_current_ability),
                Some(prepared.expected_potential_ability),
            )
    {
        return Err(snapshot_sync_error(
            "the bridge result could not be reconciled; Load Data again before retrying",
        ));
    }
    Ok(VerifiedStaffBoost {
        snapshot_id: prepared.snapshot_id,
        operation: result.operation,
        previous_current_ability: values.0.expect("verified above"),
        current_ability: values.1.expect("verified above"),
        potential_ability: values.2.expect("verified above"),
    })
}

fn eligibility_error(kind: &str, message: impl Into<String>) -> StaffBoostError {
    StaffBoostError::Eligibility {
        kind: kind.to_string(),
        message: message.into(),
    }
}

fn bridge_error(kind: &str, message: String) -> StaffBoostError {
    StaffBoostError::Bridge {
        kind: kind.to_string(),
        message,
    }
}

fn snapshot_sync_error(message: impl Into<String>) -> StaffBoostError {
    StaffBoostError::SnapshotSync {
        message: message.into(),
    }
}

fn database_eligibility_error(error: rusqlite::Error) -> StaffBoostError {
    eligibility_error("database", error.to_string())
}

fn database_sync_error(error: impl std::fmt::Display) -> StaffBoostError {
    snapshot_sync_error(error.to_string())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use serde_json::Value;

    use crate::db::migrations;
    use crate::features::memory_read::service::{
        StaffBoostResult, OPERATION_BOOST_STAFF_CURRENT_ABILITY,
    };
    use crate::features::snapshot::ingest::ingest_dump_file;

    use super::{PreparedStaffBoost, StaffBoostError};

    const GOLDEN_FIXTURE: &str = include_str!("../memory_read/fixtures/golden_dump_v8.json");
    const STAFF_UID: i64 = 88;

    fn seeded_staff(ca: i64, pa: i64) -> (tempfile::TempDir, Connection) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = Connection::open(temp_dir.path().join("staff-boost.db")).expect("open db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("foreign keys");
        migrations::apply(&conn).expect("migrations");
        let mut dump: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("fixture");
        dump["staff"][0]["ca"] = Value::from(ca);
        dump["staff"][0]["pa"] = Value::from(pa);
        let path = temp_dir.path().join("staff.json");
        std::fs::write(&path, dump.to_string()).expect("write fixture");
        let snapshot = ingest_dump_file(&mut conn, &path).expect("ingest fixture");
        conn.execute(
            "UPDATE snapshots SET bridge_source_request_id = 'scan-staff-1' WHERE id = ?1",
            [snapshot.id],
        )
        .expect("bind scan");
        (temp_dir, conn)
    }

    fn verified(prepared: &PreparedStaffBoost) -> StaffBoostResult {
        StaffBoostResult {
            operation: OPERATION_BOOST_STAFF_CURRENT_ABILITY.to_string(),
            outcome: "verified".to_string(),
            rollback: "not-needed".to_string(),
            previous_current_ability: Some(prepared.expected_current_ability as i32),
            current_ability: Some(prepared.target_current_ability as i32),
            potential_ability: Some(prepared.expected_potential_ability as i32),
        }
    }

    #[test]
    fn staff_boost_is_always_ten_and_caps_at_potential() {
        let (_temp, conn) = seeded_staff(115, 140);
        let prepared = super::prepare_current_ability_boost(&conn, STAFF_UID).expect("prepare");
        assert_eq!(prepared.target_current_ability, 125);

        let (_temp, conn) = seeded_staff(135, 140);
        let prepared = super::prepare_current_ability_boost(&conn, STAFF_UID).expect("prepare");
        assert_eq!(prepared.target_current_ability, 140);

        let (_temp, conn) = seeded_staff(140, 140);
        let error = super::prepare_current_ability_boost(&conn, STAFF_UID)
            .expect_err("at potential must be rejected");
        assert!(matches!(
            error,
            StaffBoostError::Eligibility { ref kind, .. } if kind == "currentAbilityAtLimit"
        ));
    }

    #[test]
    fn verified_boost_updates_only_staff_ca_and_keeps_role_scores() {
        let (_temp, mut conn) = seeded_staff(115, 140);
        let prepared = super::prepare_current_ability_boost(&conn, STAFF_UID).expect("prepare");
        let scores_before: Vec<(String, i64)> = conn
            .prepare(
                "SELECT role_id, score FROM staff_role_scores
                 WHERE snapshot_id = ?1 AND uid = ?2 ORDER BY role_id",
            )
            .expect("prepare scores")
            .query_map(rusqlite::params![prepared.snapshot_id, STAFF_UID], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .expect("query scores")
            .collect::<Result<_, _>>()
            .expect("collect scores");

        let result = verified(&prepared);
        super::reconcile_verified_boost(&mut conn, &prepared, result).expect("reconcile");

        let ca: i64 = conn
            .query_row(
                "SELECT ca FROM staff WHERE snapshot_id = ?1 AND uid = ?2",
                rusqlite::params![prepared.snapshot_id, STAFF_UID],
                |row| row.get(0),
            )
            .expect("read ca");
        let scores_after: Vec<(String, i64)> = conn
            .prepare(
                "SELECT role_id, score FROM staff_role_scores
                 WHERE snapshot_id = ?1 AND uid = ?2 ORDER BY role_id",
            )
            .expect("prepare scores")
            .query_map(rusqlite::params![prepared.snapshot_id, STAFF_UID], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .expect("query scores")
            .collect::<Result<_, _>>()
            .expect("collect scores");
        assert_eq!(ca, 125);
        assert_eq!(scores_after, scores_before);
    }

    #[test]
    fn stale_reconciliation_requires_load_data_and_blocks_later_staff_boosts() {
        let (_temp, mut conn) = seeded_staff(115, 140);
        let prepared = super::prepare_current_ability_boost(&conn, STAFF_UID).expect("prepare");
        conn.execute(
            "UPDATE staff SET ca = 116 WHERE snapshot_id = ?1 AND uid = ?2",
            rusqlite::params![prepared.snapshot_id, STAFF_UID],
        )
        .expect("make snapshot stale");
        let error = super::reconcile_verified_boost(&mut conn, &prepared, verified(&prepared))
            .expect_err("stale snapshot must fail");
        assert!(matches!(error, StaffBoostError::SnapshotSync { .. }));
        super::require_load_data_for_boost(&conn, &prepared.context()).expect("mark recovery");
        assert!(matches!(
            super::prepare_current_ability_boost(&conn, STAFF_UID),
            Err(StaffBoostError::SnapshotSync { .. })
        ));
    }
}
