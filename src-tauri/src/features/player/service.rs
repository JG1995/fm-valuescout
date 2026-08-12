use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::Serialize;
use serde_json::{Map, Value};

use crate::features::memory_read::service::{
    PlayerBoostOperation, PlayerBoostRequestError, PlayerBoostResult as BridgePlayerBoostResult,
    OPERATION_BOOST_CURRENT_ABILITY, OPERATION_WONDERKID_MENTALITY,
};
use crate::features::scoring::{catalog::all_roles, score::score_role};

const MIN_ABILITY: i64 = 1;
const MAX_ABILITY: i64 = 200;
const MENTALITY_ELIGIBILITY_MAXIMUM: i64 = 10;
const MENTALITY_TARGET_MINIMUM: i64 = 11;
const MENTALITY_TARGET_MAXIMUM: i64 = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PreparedPlayerBoost {
    pub(super) snapshot_id: i64,
    pub(super) snapshot_context_token: String,
    pub(super) save_id: i64,
    pub(super) save_context_token: String,
    pub(super) source_request_id: String,
    pub(super) player_uid: u32,
    pub(super) expected_current_ability: i64,
    pub(super) expected_potential_ability: i64,
    pub(super) current_ability_increment: Option<i32>,
    pub(super) target_current_ability: Option<i64>,
    pub(super) expected_ambition: Option<i64>,
    pub(super) expected_professionalism: Option<i64>,
    pub(super) expected_determination: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PlayerBoostContext {
    pub(super) snapshot_id: i64,
    pub(super) snapshot_context_token: String,
    pub(super) save_id: i64,
    pub(super) save_context_token: String,
    pub(super) source_request_id: String,
}

impl PreparedPlayerBoost {
    pub(super) fn context(&self) -> PlayerBoostContext {
        PlayerBoostContext {
            snapshot_id: self.snapshot_id,
            snapshot_context_token: self.snapshot_context_token.clone(),
            save_id: self.save_id,
            save_context_token: self.save_context_token.clone(),
            source_request_id: self.source_request_id.clone(),
        }
    }

    pub(super) fn bridge_operation(&self) -> PlayerBoostOperation {
        match self.current_ability_increment {
            Some(increment) => PlayerBoostOperation::CurrentAbility { increment },
            None => PlayerBoostOperation::WonderkidMentality {
                expected_ambition: self.expected_ambition.map(|value| value as i32),
                expected_professionalism: self.expected_professionalism.map(|value| value as i32),
                expected_determination: self.expected_determination.map(|value| value as i32),
            },
        }
    }

    fn operation_name(&self) -> &'static str {
        if self.current_ability_increment.is_some() {
            OPERATION_BOOST_CURRENT_ABILITY
        } else {
            OPERATION_WONDERKID_MENTALITY
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct VerifiedPlayerBoost {
    pub(super) snapshot_id: i64,
    pub(super) operation: String,
    pub(super) previous_current_ability: Option<i64>,
    pub(super) current_ability: Option<i64>,
    pub(super) potential_ability: Option<i64>,
    pub(super) previous_ambition: Option<i64>,
    pub(super) ambition: Option<i64>,
    pub(super) previous_professionalism: Option<i64>,
    pub(super) professionalism: Option<i64>,
    pub(super) previous_determination: Option<i64>,
    pub(super) determination: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "phase", rename_all = "camelCase")]
pub enum PlayerBoostError {
    Eligibility { kind: String, message: String },
    Bridge { kind: String, message: String },
    LiveValue { message: String },
    SnapshotSync { message: String },
}

impl std::fmt::Display for PlayerBoostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eligibility { message, .. }
            | Self::Bridge { message, .. }
            | Self::LiveValue { message }
            | Self::SnapshotSync { message } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for PlayerBoostError {}

pub(super) fn prepare_current_ability_boost(
    conn: &Connection,
    uid: i64,
) -> Result<PreparedPlayerBoost, PlayerBoostError> {
    let player = capture_player(conn, uid)?;
    validate_ability_snapshot(&player)?;
    let age = player
        .age
        .ok_or_else(|| eligibility_error("unknownAge", "player age is unknown"))?;
    if age < 0 {
        return Err(eligibility_error(
            "invalidSnapshot",
            "player age is invalid; Load Data again",
        ));
    }
    if age >= 29 {
        return Err(eligibility_error(
            "ageIneligible",
            "current ability boosts are unavailable for players aged 29 or older",
        ));
    }

    let increment = if age <= 20 { 5 } else { 10 };
    let target = (player.current_ability + i64::from(increment))
        .min(player.potential_ability)
        .min(MAX_ABILITY);
    if target <= player.current_ability {
        return Err(eligibility_error(
            "currentAbilityAtLimit",
            "current ability has reached its potential limit",
        ));
    }

    Ok(PreparedPlayerBoost {
        snapshot_id: player.snapshot_id,
        snapshot_context_token: player.snapshot_context_token,
        save_id: player.save_id,
        save_context_token: player.save_context_token,
        source_request_id: player.source_request_id,
        player_uid: player.uid,
        expected_current_ability: player.current_ability,
        expected_potential_ability: player.potential_ability,
        current_ability_increment: Some(increment),
        target_current_ability: Some(target),
        expected_ambition: None,
        expected_professionalism: None,
        expected_determination: None,
    })
}

pub(super) fn prepare_wonderkid_mentality_boost(
    conn: &Connection,
    uid: i64,
) -> Result<PreparedPlayerBoost, PlayerBoostError> {
    let player = capture_player(conn, uid)?;
    validate_ability_snapshot(&player)?;
    let mentality = parse_mentality_snapshot(&player.attributes_json, &player.personality_json)
        .map_err(|message| eligibility_error("invalidSnapshot", message))?;

    if !mentality.is_eligible() {
        return Err(eligibility_error(
            "noEligibleMentality",
            "no known mentality attribute is 10 or lower",
        ));
    }

    Ok(PreparedPlayerBoost {
        snapshot_id: player.snapshot_id,
        snapshot_context_token: player.snapshot_context_token,
        save_id: player.save_id,
        save_context_token: player.save_context_token,
        source_request_id: player.source_request_id,
        player_uid: player.uid,
        expected_current_ability: player.current_ability,
        expected_potential_ability: player.potential_ability,
        current_ability_increment: None,
        target_current_ability: None,
        expected_ambition: mentality.ambition,
        expected_professionalism: mentality.professionalism,
        expected_determination: mentality.determination,
    })
}

pub(super) fn map_bridge_error(error: PlayerBoostRequestError) -> PlayerBoostError {
    match error {
        PlayerBoostRequestError::Unconfirmed(message) => PlayerBoostError::SnapshotSync {
            message: format!("FM may have changed. {message}"),
        },
        PlayerBoostRequestError::Failed(message)
            if message == "player values changed in FM; Load Data again"
                || message == "current ability is already at its potential limit" =>
        {
            PlayerBoostError::LiveValue { message }
        }
        PlayerBoostRequestError::Failed(message) => PlayerBoostError::Bridge {
            kind: "rejected".to_string(),
            message,
        },
        PlayerBoostRequestError::UnsupportedPlatform(message) => PlayerBoostError::Bridge {
            kind: "unsupportedPlatform".to_string(),
            message,
        },
        PlayerBoostRequestError::Missing(message) => PlayerBoostError::Bridge {
            kind: "missing".to_string(),
            message,
        },
        PlayerBoostRequestError::Corrupt(message) => PlayerBoostError::Bridge {
            kind: "corrupt".to_string(),
            message,
        },
        PlayerBoostRequestError::Timeout(message) => PlayerBoostError::SnapshotSync {
            message: format!("FM may have changed. Load Data again before retrying ({message})"),
        },
        PlayerBoostRequestError::WriteFailed(message) => PlayerBoostError::Bridge {
            kind: "writeFailed".to_string(),
            message,
        },
        PlayerBoostRequestError::Unavailable(message) => PlayerBoostError::Bridge {
            kind: "unavailable".to_string(),
            message,
        },
    }
}

pub(super) fn reconcile_verified_boost(
    conn: &mut Connection,
    prepared: &PreparedPlayerBoost,
    bridge_result: BridgePlayerBoostResult,
) -> Result<VerifiedPlayerBoost, PlayerBoostError> {
    let verified = verify_bridge_result(prepared, bridge_result)?;
    let tx = conn.transaction().map_err(database_sync_error)?;
    let current = load_current_player_state(&tx, prepared)?;
    ensure_current_player_matches_prepared(&current, prepared)?;

    if prepared.current_ability_increment.is_some() {
        let current_ability = verified
            .current_ability
            .ok_or_else(unconfirmed_result_error)?;
        tx.execute(
            "UPDATE players SET ca = ?1 WHERE snapshot_id = ?2 AND uid = ?3",
            params![
                current_ability,
                prepared.snapshot_id,
                i64::from(prepared.player_uid)
            ],
        )
        .map_err(database_sync_error)?;
    } else {
        reconcile_mentality(&tx, prepared, &verified, &current)?;
    }

    crate::features::player_metrics::potential_cache::invalidate_player_cache(
        &tx,
        prepared.snapshot_id,
        i64::from(prepared.player_uid),
    )
    .map_err(database_sync_error)?;

    tx.commit().map_err(database_sync_error)?;
    Ok(verified)
}

pub(super) fn capture_active_player_boost_context(
    conn: &Connection,
) -> Result<PlayerBoostContext, PlayerBoostError> {
    let snapshot: Option<(i64, i64, String, String, Option<String>, i32)> = conn
        .query_row(
            "SELECT
                s.id,
                s.save_id,
                s.context_token,
                sv.context_token,
                s.bridge_source_request_id,
                s.player_boost_recovery_required
             FROM snapshots s
             INNER JOIN saves sv ON sv.id = s.save_id AND sv.is_active = 1
             WHERE s.is_current = 1
             LIMIT 1",
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
        .map_err(|error| eligibility_error("database", error.to_string()))?;
    let Some((
        snapshot_id,
        save_id,
        snapshot_context_token,
        save_context_token,
        source_request_id,
        recovery_required,
    )) = snapshot
    else {
        return Err(eligibility_error(
            "noSnapshot",
            "Load Data before using player boosts",
        ));
    };
    if recovery_required != 0 {
        return Err(snapshot_sync_error(
            "Load Data before using player boosts; the current snapshot could not be reconciled.",
        ));
    }
    let Some(source_request_id) = source_request_id.filter(|value| !value.trim().is_empty()) else {
        return Err(eligibility_error(
            "missingProvenance",
            "Load Data again before using player boosts",
        ));
    };

    Ok(PlayerBoostContext {
        snapshot_id,
        snapshot_context_token,
        save_id,
        save_context_token,
        source_request_id,
    })
}

pub(super) fn active_player_boost_context_matches(
    conn: &Connection,
    expected: &PlayerBoostContext,
) -> Result<bool, PlayerBoostError> {
    Ok(capture_active_player_boost_context(conn)? == *expected)
}

pub(super) fn require_load_data_for_player_boost(
    conn: &Connection,
    context: &PlayerBoostContext,
) -> Result<(), PlayerBoostError> {
    let changed = conn
        .execute(
            "UPDATE snapshots
             SET player_boost_recovery_required = 1
             WHERE id = ?1 AND context_token = ?2",
            params![context.snapshot_id, &context.snapshot_context_token],
        )
        .map_err(database_sync_error)?;
    if changed == 1 {
        return Ok(());
    }

    Err(snapshot_sync_error(
        "FM may have changed, but FM ValueScout could not preserve the recovery requirement. Load Data again.",
    ))
}

fn capture_player(conn: &Connection, uid: i64) -> Result<CapturedPlayer, PlayerBoostError> {
    let player_uid = u32::try_from(uid).map_err(|_| {
        eligibility_error(
            "invalidPlayer",
            "player identity is invalid for a player boost",
        )
    })?;
    if player_uid == 0 {
        return Err(eligibility_error(
            "invalidPlayer",
            "player identity is invalid for a player boost",
        ));
    }

    let context = capture_active_player_boost_context(conn)?;

    let player = conn
        .query_row(
            "SELECT ca, pa, age, attributes_json, personality_json
             FROM players
             WHERE snapshot_id = ?1 AND uid = ?2",
            params![context.snapshot_id, uid],
            |row| {
                Ok(CapturedPlayer {
                    snapshot_id: context.snapshot_id,
                    snapshot_context_token: context.snapshot_context_token.clone(),
                    save_id: context.save_id,
                    save_context_token: context.save_context_token.clone(),
                    source_request_id: context.source_request_id.clone(),
                    uid: player_uid,
                    current_ability: row.get(0)?,
                    potential_ability: row.get(1)?,
                    age: row.get(2)?,
                    attributes_json: row.get(3)?,
                    personality_json: row.get(4)?,
                })
            },
        )
        .optional()
        .map_err(|error| eligibility_error("database", error.to_string()))?;

    player.ok_or_else(|| {
        eligibility_error(
            "playerNotFound",
            "player is not in the current snapshot; Load Data again",
        )
    })
}

fn validate_ability_snapshot(player: &CapturedPlayer) -> Result<(), PlayerBoostError> {
    if !is_ability(player.current_ability)
        || !is_ability(player.potential_ability)
        || player.current_ability > player.potential_ability
    {
        return Err(eligibility_error(
            "invalidSnapshot",
            "snapshot ability values are invalid; Load Data again",
        ));
    }

    Ok(())
}

fn parse_mentality_snapshot(
    attributes_json: &str,
    personality_json: &str,
) -> Result<MentalitySnapshot, String> {
    let attributes = parse_json_object(attributes_json, "attributes")?;
    let personality = parse_json_object(personality_json, "personality")?;
    Ok(MentalitySnapshot {
        ambition: known_mentality_value(&personality, "Ambition"),
        professionalism: known_mentality_value(&personality, "Professionalism"),
        determination: known_mentality_value(&attributes, "Determination"),
    })
}

fn verify_bridge_result(
    prepared: &PreparedPlayerBoost,
    result: BridgePlayerBoostResult,
) -> Result<VerifiedPlayerBoost, PlayerBoostError> {
    if result.operation != prepared.operation_name()
        || result.outcome != "verified"
        || result.rollback != "not-needed"
    {
        return Err(unconfirmed_result_error());
    }

    let previous_current_ability = required_result_value(result.previous_current_ability)?;
    let current_ability = required_result_value(result.current_ability)?;
    let potential_ability = required_result_value(result.potential_ability)?;
    let expected_current_ability = prepared
        .target_current_ability
        .unwrap_or(prepared.expected_current_ability);
    if previous_current_ability != prepared.expected_current_ability
        || current_ability != expected_current_ability
        || potential_ability != prepared.expected_potential_ability
    {
        return Err(unconfirmed_result_error());
    }

    let (
        previous_ambition,
        ambition,
        previous_professionalism,
        professionalism,
        previous_determination,
        determination,
    ) = if prepared.current_ability_increment.is_some() {
        if result.previous_ambition.is_some()
            || result.ambition.is_some()
            || result.previous_professionalism.is_some()
            || result.professionalism.is_some()
            || result.previous_determination.is_some()
            || result.determination.is_some()
        {
            return Err(unconfirmed_result_error());
        }
        (None, None, None, None, None, None)
    } else {
        (
            prepared.expected_ambition,
            verify_mentality_result(
                prepared.expected_ambition,
                result.previous_ambition.map(i64::from),
                result.ambition.map(i64::from),
            )?,
            prepared.expected_professionalism,
            verify_mentality_result(
                prepared.expected_professionalism,
                result.previous_professionalism.map(i64::from),
                result.professionalism.map(i64::from),
            )?,
            prepared.expected_determination,
            verify_mentality_result(
                prepared.expected_determination,
                result.previous_determination.map(i64::from),
                result.determination.map(i64::from),
            )?,
        )
    };

    Ok(VerifiedPlayerBoost {
        snapshot_id: prepared.snapshot_id,
        operation: result.operation,
        previous_current_ability: Some(previous_current_ability),
        current_ability: Some(current_ability),
        potential_ability: Some(potential_ability),
        previous_ambition,
        ambition,
        previous_professionalism,
        professionalism,
        previous_determination,
        determination,
    })
}

fn required_result_value(value: Option<i32>) -> Result<i64, PlayerBoostError> {
    value.map(i64::from).ok_or_else(unconfirmed_result_error)
}

fn verify_mentality_result(
    expected: Option<i64>,
    previous: Option<i64>,
    current: Option<i64>,
) -> Result<Option<i64>, PlayerBoostError> {
    match expected {
        None if previous.is_none() && current.is_none() => Ok(None),
        None => Err(unconfirmed_result_error()),
        Some(expected) => {
            if previous != Some(expected) {
                return Err(unconfirmed_result_error());
            }
            let Some(current) = current else {
                return Err(unconfirmed_result_error());
            };
            if (expected <= MENTALITY_ELIGIBILITY_MAXIMUM
                && !(MENTALITY_TARGET_MINIMUM..=MENTALITY_TARGET_MAXIMUM).contains(&current))
                || (expected > MENTALITY_ELIGIBILITY_MAXIMUM && current != expected)
            {
                return Err(unconfirmed_result_error());
            }
            Ok(Some(current))
        }
    }
}

fn load_current_player_state(
    tx: &Transaction<'_>,
    prepared: &PreparedPlayerBoost,
) -> Result<CurrentPlayerState, PlayerBoostError> {
    tx.query_row(
        "SELECT s.bridge_source_request_id, p.ca, p.pa, p.attributes_json, p.personality_json
         FROM snapshots s
         INNER JOIN saves sv ON sv.id = s.save_id AND sv.is_active = 1
         INNER JOIN players p ON p.snapshot_id = s.id AND p.uid = ?5
         WHERE s.id = ?1
           AND s.save_id = ?2
           AND s.context_token = ?3
           AND sv.context_token = ?4
           AND s.is_current = 1",
        params![
            prepared.snapshot_id,
            prepared.save_id,
            prepared.snapshot_context_token,
            prepared.save_context_token,
            i64::from(prepared.player_uid)
        ],
        |row| {
            Ok(CurrentPlayerState {
                source_request_id: row.get(0)?,
                current_ability: row.get(1)?,
                potential_ability: row.get(2)?,
                attributes_json: row.get(3)?,
                personality_json: row.get(4)?,
            })
        },
    )
    .optional()
    .map_err(database_sync_error)?
    .ok_or_else(|| {
        snapshot_sync_error("FM changed, but this snapshot is no longer current. Load Data again.")
    })
}

fn ensure_current_player_matches_prepared(
    current: &CurrentPlayerState,
    prepared: &PreparedPlayerBoost,
) -> Result<(), PlayerBoostError> {
    if current.source_request_id.as_deref() != Some(prepared.source_request_id.as_str())
        || current.current_ability != prepared.expected_current_ability
        || current.potential_ability != prepared.expected_potential_ability
    {
        return Err(snapshot_sync_error(
            "FM changed, but the snapshot no longer matches this boost. Load Data again.",
        ));
    }

    Ok(())
}

fn reconcile_mentality(
    tx: &Transaction<'_>,
    prepared: &PreparedPlayerBoost,
    verified: &VerifiedPlayerBoost,
    current: &CurrentPlayerState,
) -> Result<(), PlayerBoostError> {
    let mut attributes =
        parse_json_object(&current.attributes_json, "attributes").map_err(snapshot_sync_error)?;
    let mut personality =
        parse_json_object(&current.personality_json, "personality").map_err(snapshot_sync_error)?;
    let current_mentality = MentalitySnapshot {
        ambition: known_mentality_value(&personality, "Ambition"),
        professionalism: known_mentality_value(&personality, "Professionalism"),
        determination: known_mentality_value(&attributes, "Determination"),
    };
    if current_mentality.ambition != prepared.expected_ambition
        || current_mentality.professionalism != prepared.expected_professionalism
        || current_mentality.determination != prepared.expected_determination
    {
        return Err(snapshot_sync_error(
            "FM changed, but the snapshot no longer matches this boost. Load Data again.",
        ));
    }

    if prepared.expected_ambition.is_some() {
        set_nullable_integer(&mut personality, "Ambition", verified.ambition);
    }
    if prepared.expected_professionalism.is_some() {
        set_nullable_integer(
            &mut personality,
            "Professionalism",
            verified.professionalism,
        );
    }
    if prepared.expected_determination.is_some() {
        set_nullable_integer(&mut attributes, "Determination", verified.determination);
    }
    let attributes_json = serde_json::to_string(&attributes).map_err(database_sync_error)?;
    let personality_json = serde_json::to_string(&personality).map_err(database_sync_error)?;

    tx.execute(
        "UPDATE players
         SET attributes_json = ?1, personality_json = ?2
         WHERE snapshot_id = ?3 AND uid = ?4",
        params![
            attributes_json,
            personality_json,
            prepared.snapshot_id,
            i64::from(prepared.player_uid)
        ],
    )
    .map_err(database_sync_error)?;

    if verified.previous_determination != verified.determination {
        replace_role_scores(tx, prepared.snapshot_id, prepared.player_uid, &attributes)?;
    }

    Ok(())
}

fn replace_role_scores(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    player_uid: u32,
    attributes: &Map<String, Value>,
) -> Result<(), PlayerBoostError> {
    let scoring_attributes = scoring_attributes(attributes).map_err(snapshot_sync_error)?;
    tx.execute(
        "DELETE FROM player_role_scores WHERE snapshot_id = ?1 AND uid = ?2",
        params![snapshot_id, i64::from(player_uid)],
    )
    .map_err(database_sync_error)?;

    let mut statement = tx
        .prepare(
            "INSERT INTO player_role_scores (snapshot_id, uid, role_id, phase, score)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .map_err(database_sync_error)?;
    for role in all_roles() {
        statement
            .execute(params![
                snapshot_id,
                i64::from(player_uid),
                role.role_id,
                role.phase.as_db_str(),
                score_role(&scoring_attributes, role).map(i64::from),
            ])
            .map_err(database_sync_error)?;
    }

    Ok(())
}

fn scoring_attributes(
    attributes: &Map<String, Value>,
) -> Result<HashMap<String, Option<u8>>, String> {
    attributes
        .iter()
        .map(|(key, value)| {
            let value = match value {
                Value::Null => None,
                Value::Number(number) => {
                    let value = number
                        .as_i64()
                        .ok_or_else(|| format!("attribute `{key}` must be an integer or null"))?;
                    Some(
                        u8::try_from(value)
                            .map_err(|_| format!("attribute `{key}` is outside the u8 range"))?,
                    )
                }
                _ => return Err(format!("attribute `{key}` must be an integer or null")),
            };
            Ok((key.clone(), value))
        })
        .collect()
}

fn parse_json_object(json: &str, label: &str) -> Result<Map<String, Value>, String> {
    let value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    value
        .as_object()
        .cloned()
        .ok_or_else(|| format!("{label} must be a JSON object"))
}

fn known_mentality_value(object: &Map<String, Value>, key: &str) -> Option<i64> {
    object
        .get(key)
        .and_then(Value::as_i64)
        .filter(|value| is_mentality(*value))
}

fn set_nullable_integer(object: &mut Map<String, Value>, key: &str, value: Option<i64>) {
    object.insert(
        key.to_string(),
        value.map(Value::from).unwrap_or(Value::Null),
    );
}

fn eligibility_error(kind: &str, message: impl Into<String>) -> PlayerBoostError {
    PlayerBoostError::Eligibility {
        kind: kind.to_string(),
        message: message.into(),
    }
}

fn snapshot_sync_error(message: impl Into<String>) -> PlayerBoostError {
    PlayerBoostError::SnapshotSync {
        message: message.into(),
    }
}

fn database_sync_error(_error: impl std::fmt::Display) -> PlayerBoostError {
    snapshot_sync_error(
        "FM changed, but FM ValueScout could not update its snapshot. Load Data again.",
    )
}

fn unconfirmed_result_error() -> PlayerBoostError {
    snapshot_sync_error(
        "FM may have changed, but the bridge result could not be verified. Load Data again.",
    )
}

fn is_ability(value: i64) -> bool {
    (MIN_ABILITY..=MAX_ABILITY).contains(&value)
}

fn is_mentality(value: i64) -> bool {
    (MIN_ABILITY..=MENTALITY_TARGET_MAXIMUM).contains(&value)
}

#[derive(Debug, Clone)]
struct CapturedPlayer {
    snapshot_id: i64,
    snapshot_context_token: String,
    save_id: i64,
    save_context_token: String,
    source_request_id: String,
    uid: u32,
    current_ability: i64,
    potential_ability: i64,
    age: Option<i64>,
    attributes_json: String,
    personality_json: String,
}

#[derive(Debug, Clone)]
struct CurrentPlayerState {
    source_request_id: Option<String>,
    current_ability: i64,
    potential_ability: i64,
    attributes_json: String,
    personality_json: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MentalitySnapshot {
    ambition: Option<i64>,
    professionalism: Option<i64>,
    determination: Option<i64>,
}

impl MentalitySnapshot {
    fn is_eligible(self) -> bool {
        [self.ambition, self.professionalism, self.determination]
            .into_iter()
            .any(|value| value.is_some_and(|value| value <= MENTALITY_ELIGIBILITY_MAXIMUM))
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use rusqlite::{params, Connection};
    use serde_json::{Map, Value};

    use crate::db::migrations;
    use crate::features::memory_read::service::{
        PlayerBoostRequestError, PlayerBoostResult, OPERATION_BOOST_CURRENT_ABILITY,
        OPERATION_WONDERKID_MENTALITY,
    };
    use crate::features::scoring::catalog::all_roles;
    use crate::features::snapshot::ingest::ingest_dump_file;

    const GOLDEN_FIXTURE: &str = include_str!("../memory_read/fixtures/golden_dump_v6.json");
    const PLAYER_UID: i64 = 77;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Mentality {
        ambition: Option<i64>,
        professionalism: Option<i64>,
        determination: Option<i64>,
    }

    struct SeededPlayer {
        _temp_dir: tempfile::TempDir,
        conn: Connection,
        snapshot_id: i64,
        save_id: i64,
        dump_path: PathBuf,
    }

    fn open_migrated(db_path: &Path) -> Connection {
        let conn = Connection::open(db_path).expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    fn seeded_player(age: Option<i64>, ca: i64, pa: i64, mentality: Mentality) -> SeededPlayer {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("player-boost.db"));
        let mut dump: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse fixture");
        dump["players"][0]["age"] = nullable_value(age);
        dump["players"][0]["ca"] = Value::from(ca);
        dump["players"][0]["pa"] = Value::from(pa);
        let attributes = dump["players"][0]["attributes"]
            .as_object_mut()
            .expect("player attributes");
        attributes.insert(
            "Determination".to_string(),
            nullable_value(mentality.determination),
        );
        let personality = dump["players"][0]["personality"]
            .as_object_mut()
            .expect("player personality");
        personality.insert("Ambition".to_string(), nullable_value(mentality.ambition));
        personality.insert(
            "Professionalism".to_string(),
            nullable_value(mentality.professionalism),
        );
        let dump_path = temp_dir.path().join("player.json");
        std::fs::write(&dump_path, dump.to_string()).expect("write dump");
        let snapshot = ingest_dump_file(&mut conn, &dump_path).expect("ingest player");
        conn.execute(
            "UPDATE snapshots SET bridge_source_request_id = ?1 WHERE id = ?2",
            rusqlite::params!["scan-player-1", snapshot.id],
        )
        .expect("bind source request");

        SeededPlayer {
            _temp_dir: temp_dir,
            conn,
            snapshot_id: snapshot.id,
            save_id: snapshot.save_id,
            dump_path,
        }
    }

    fn nullable_value(value: Option<i64>) -> Value {
        value.map(Value::from).unwrap_or(Value::Null)
    }

    fn verified_ca_result(previous: i64, current: i64, potential: i64) -> PlayerBoostResult {
        PlayerBoostResult {
            operation: OPERATION_BOOST_CURRENT_ABILITY.to_string(),
            outcome: "verified".to_string(),
            rollback: "not-needed".to_string(),
            previous_current_ability: Some(previous as i32),
            current_ability: Some(current as i32),
            potential_ability: Some(potential as i32),
            previous_ambition: None,
            ambition: None,
            previous_professionalism: None,
            professionalism: None,
            previous_determination: None,
            determination: None,
        }
    }

    fn verified_mentality_result(
        current_ability: i64,
        potential_ability: i64,
        previous: Mentality,
        current: Mentality,
    ) -> PlayerBoostResult {
        PlayerBoostResult {
            operation: OPERATION_WONDERKID_MENTALITY.to_string(),
            outcome: "verified".to_string(),
            rollback: "not-needed".to_string(),
            previous_current_ability: Some(current_ability as i32),
            current_ability: Some(current_ability as i32),
            potential_ability: Some(potential_ability as i32),
            previous_ambition: previous.ambition.map(|value| value as i32),
            ambition: current.ambition.map(|value| value as i32),
            previous_professionalism: previous.professionalism.map(|value| value as i32),
            professionalism: current.professionalism.map(|value| value as i32),
            previous_determination: previous.determination.map(|value| value as i32),
            determination: current.determination.map(|value| value as i32),
        }
    }

    fn player_ca(conn: &Connection, snapshot_id: i64) -> i64 {
        conn.query_row(
            "SELECT ca FROM players WHERE snapshot_id = ?1 AND uid = ?2",
            params![snapshot_id, PLAYER_UID],
            |row| row.get(0),
        )
        .expect("read player CA")
    }

    fn player_mentality(conn: &Connection, snapshot_id: i64) -> Mentality {
        let (attributes_json, personality_json): (String, String) = conn
            .query_row(
                "SELECT attributes_json, personality_json
                 FROM players WHERE snapshot_id = ?1 AND uid = ?2",
                params![snapshot_id, PLAYER_UID],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read player mentality");
        let attributes: Map<String, Value> =
            serde_json::from_str(&attributes_json).expect("parse attributes");
        let personality: Map<String, Value> =
            serde_json::from_str(&personality_json).expect("parse personality");
        Mentality {
            ambition: super::known_mentality_value(&personality, "Ambition"),
            professionalism: super::known_mentality_value(&personality, "Professionalism"),
            determination: super::known_mentality_value(&attributes, "Determination"),
        }
    }

    fn assert_eligibility_kind(error: super::PlayerBoostError, expected_kind: &str) {
        match error {
            super::PlayerBoostError::Eligibility { kind, .. } => assert_eq!(kind, expected_kind),
            other => panic!("expected eligibility error {expected_kind}, got {other:?}"),
        }
    }

    fn assert_snapshot_sync(error: super::PlayerBoostError) {
        assert!(
            matches!(error, super::PlayerBoostError::SnapshotSync { .. }),
            "expected snapshot-sync error, got {error:?}"
        );
    }

    #[test]
    fn current_ability_boost_prepares_the_age_twenty_increment_and_source_request() {
        let fixture = seeded_player(
            Some(20),
            150,
            170,
            Mentality {
                ambition: Some(14),
                professionalism: Some(14),
                determination: Some(14),
            },
        );

        let prepared = super::prepare_current_ability_boost(&fixture.conn, PLAYER_UID)
            .expect("prepare CA boost");

        assert_eq!(prepared.current_ability_increment, Some(5));
        assert_eq!(prepared.target_current_ability, Some(155));
        assert_eq!(prepared.source_request_id, "scan-player-1");
        assert_eq!(prepared.expected_current_ability, 150);
        assert_eq!(prepared.expected_potential_ability, 170);
    }

    #[test]
    fn current_ability_boost_uses_ten_from_age_twenty_one_through_twenty_eight() {
        for age in [21, 28] {
            let fixture = seeded_player(
                Some(age),
                168,
                170,
                Mentality {
                    ambition: Some(14),
                    professionalism: Some(14),
                    determination: Some(14),
                },
            );

            let prepared = super::prepare_current_ability_boost(&fixture.conn, PLAYER_UID)
                .expect("prepare CA boost");

            assert_eq!(prepared.current_ability_increment, Some(10));
            assert_eq!(prepared.target_current_ability, Some(170));
        }
    }

    #[test]
    fn current_ability_boost_rejects_unknown_and_non_actionable_snapshot_values() {
        let unknown_age = seeded_player(
            None,
            150,
            170,
            Mentality {
                ambition: Some(14),
                professionalism: Some(14),
                determination: Some(14),
            },
        );
        assert_eligibility_kind(
            super::prepare_current_ability_boost(&unknown_age.conn, PLAYER_UID)
                .expect_err("unknown age must be ineligible"),
            "unknownAge",
        );

        let age_ineligible = seeded_player(
            Some(29),
            150,
            170,
            Mentality {
                ambition: Some(14),
                professionalism: Some(14),
                determination: Some(14),
            },
        );
        assert_eligibility_kind(
            super::prepare_current_ability_boost(&age_ineligible.conn, PLAYER_UID)
                .expect_err("age 29 must be ineligible"),
            "ageIneligible",
        );

        let at_potential = seeded_player(
            Some(22),
            170,
            170,
            Mentality {
                ambition: Some(14),
                professionalism: Some(14),
                determination: Some(14),
            },
        );
        assert_eligibility_kind(
            super::prepare_current_ability_boost(&at_potential.conn, PLAYER_UID)
                .expect_err("CA at PA must be ineligible"),
            "currentAbilityAtLimit",
        );

        let at_maximum = seeded_player(
            Some(22),
            200,
            200,
            Mentality {
                ambition: Some(14),
                professionalism: Some(14),
                determination: Some(14),
            },
        );
        assert_eligibility_kind(
            super::prepare_current_ability_boost(&at_maximum.conn, PLAYER_UID)
                .expect_err("CA 200 must be ineligible"),
            "currentAbilityAtLimit",
        );

        let unknown_potential = seeded_player(
            Some(22),
            150,
            0,
            Mentality {
                ambition: Some(14),
                professionalism: Some(14),
                determination: Some(14),
            },
        );
        assert_eligibility_kind(
            super::prepare_current_ability_boost(&unknown_potential.conn, PLAYER_UID)
                .expect_err("invalid PA must be ineligible"),
            "invalidSnapshot",
        );
    }

    #[test]
    fn current_ability_reconciliation_updates_the_snapshot_and_supports_a_repeat() {
        let mut fixture = seeded_player(
            Some(20),
            150,
            170,
            Mentality {
                ambition: Some(14),
                professionalism: Some(14),
                determination: Some(14),
            },
        );
        let prepared = super::prepare_current_ability_boost(&fixture.conn, PLAYER_UID)
            .expect("prepare CA boost");

        let result = super::reconcile_verified_boost(
            &mut fixture.conn,
            &prepared,
            verified_ca_result(150, 155, 170),
        )
        .expect("reconcile CA boost");

        assert_eq!(result.current_ability, Some(155));
        assert_eq!(player_ca(&fixture.conn, fixture.snapshot_id), 155);
        let repeated = super::prepare_current_ability_boost(&fixture.conn, PLAYER_UID)
            .expect("prepare repeat");
        assert_eq!(repeated.expected_current_ability, 155);
        assert_eq!(repeated.target_current_ability, Some(160));
        assert_eq!(repeated.source_request_id, "scan-player-1");
    }

    #[test]
    fn successful_player_boost_invalidates_that_players_potential_role_cache() {
        let mut fixture = seeded_player(
            Some(20),
            150,
            170,
            Mentality {
                ambition: Some(14),
                professionalism: Some(14),
                determination: Some(14),
            },
        );
        fixture
            .conn
            .execute(
                "INSERT INTO player_potential_role_scores (
                    snapshot_id, uid, role_id, score, projection_model_version
                 ) VALUES (?1, ?2, 'goalkeeper_ip', 80, 1)",
                params![fixture.snapshot_id, PLAYER_UID],
            )
            .expect("seed potential cache");
        let prepared = super::prepare_current_ability_boost(&fixture.conn, PLAYER_UID)
            .expect("prepare CA boost");

        super::reconcile_verified_boost(
            &mut fixture.conn,
            &prepared,
            verified_ca_result(150, 155, 170),
        )
        .expect("reconcile CA boost");

        let cache_rows: i64 = fixture
            .conn
            .query_row(
                "SELECT COUNT(*) FROM player_potential_role_scores
                 WHERE snapshot_id = ?1 AND uid = ?2",
                params![fixture.snapshot_id, PLAYER_UID],
                |row| row.get(0),
            )
            .expect("count invalidated potential cache rows");
        assert_eq!(cache_rows, 0);
    }

    #[test]
    fn player_boost_requires_a_snapshot_bound_to_a_bridge_scan() {
        let fixture = seeded_player(
            Some(21),
            150,
            170,
            Mentality {
                ambition: Some(14),
                professionalism: Some(14),
                determination: Some(14),
            },
        );
        fixture
            .conn
            .execute(
                "UPDATE snapshots SET bridge_source_request_id = NULL WHERE id = ?1",
                [fixture.snapshot_id],
            )
            .expect("clear source binding");

        assert_eligibility_kind(
            super::prepare_current_ability_boost(&fixture.conn, PLAYER_UID)
                .expect_err("unbound snapshot must be ineligible"),
            "missingProvenance",
        );
    }

    #[test]
    fn wonderkid_mentality_prepares_only_snapshot_values_and_requires_an_eligible_field() {
        let fixture = seeded_player(
            Some(19),
            150,
            170,
            Mentality {
                ambition: Some(10),
                professionalism: Some(11),
                determination: None,
            },
        );
        let prepared = super::prepare_wonderkid_mentality_boost(&fixture.conn, PLAYER_UID)
            .expect("prepare Wonderkid Mentality");

        assert_eq!(prepared.expected_ambition, Some(10));
        assert_eq!(prepared.expected_professionalism, Some(11));
        assert_eq!(prepared.expected_determination, None);
        assert_eq!(
            prepared.bridge_operation(),
            super::PlayerBoostOperation::WonderkidMentality {
                expected_ambition: Some(10),
                expected_professionalism: Some(11),
                expected_determination: None,
            }
        );

        let none_eligible = seeded_player(
            Some(19),
            150,
            170,
            Mentality {
                ambition: Some(11),
                professionalism: Some(12),
                determination: None,
            },
        );
        assert_eligibility_kind(
            super::prepare_wonderkid_mentality_boost(&none_eligible.conn, PLAYER_UID)
                .expect_err("all high or unknown values must be ineligible"),
            "noEligibleMentality",
        );
    }

    #[test]
    fn wonderkid_mentality_preserves_missing_and_invalid_snapshot_values() {
        let mut fixture = seeded_player(
            Some(19),
            150,
            170,
            Mentality {
                ambition: Some(10),
                professionalism: Some(11),
                determination: Some(12),
            },
        );
        fixture
            .conn
            .execute(
                "UPDATE players
                 SET attributes_json = ?1, personality_json = ?2
                 WHERE snapshot_id = ?3 AND uid = ?4",
                params![
                    r#"{"Pace":15,"Determination":0}"#,
                    r#"{"Ambition":10}"#,
                    fixture.snapshot_id,
                    PLAYER_UID
                ],
            )
            .expect("set unknown mentality fields");

        let prepared = super::prepare_wonderkid_mentality_boost(&fixture.conn, PLAYER_UID)
            .expect("prepare mixed known and unknown values");

        assert_eq!(prepared.expected_ambition, Some(10));
        assert_eq!(prepared.expected_professionalism, None);
        assert_eq!(prepared.expected_determination, None);

        super::reconcile_verified_boost(
            &mut fixture.conn,
            &prepared,
            verified_mentality_result(
                150,
                170,
                Mentality {
                    ambition: Some(10),
                    professionalism: None,
                    determination: None,
                },
                Mentality {
                    ambition: Some(19),
                    professionalism: None,
                    determination: None,
                },
            ),
        )
        .expect("reconcile mixed known and unknown values");

        let (attributes_json, personality_json): (String, String) = fixture
            .conn
            .query_row(
                "SELECT attributes_json, personality_json
                 FROM players WHERE snapshot_id = ?1 AND uid = ?2",
                params![fixture.snapshot_id, PLAYER_UID],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read reconciled mentality JSON");
        let attributes: Map<String, Value> =
            serde_json::from_str(&attributes_json).expect("parse attributes JSON");
        let personality: Map<String, Value> =
            serde_json::from_str(&personality_json).expect("parse personality JSON");

        assert_eq!(personality.get("Ambition"), Some(&Value::from(19)));
        assert!(!personality.contains_key("Professionalism"));
        assert_eq!(attributes.get("Determination"), Some(&Value::from(0)));
    }

    #[test]
    fn wonderkid_reconciliation_preserves_unknown_and_already_high_values() {
        let mut fixture = seeded_player(
            Some(19),
            150,
            170,
            Mentality {
                ambition: Some(10),
                professionalism: Some(11),
                determination: None,
            },
        );
        let prepared = super::prepare_wonderkid_mentality_boost(&fixture.conn, PLAYER_UID)
            .expect("prepare Wonderkid Mentality");
        let previous = Mentality {
            ambition: Some(10),
            professionalism: Some(11),
            determination: None,
        };
        let current = Mentality {
            ambition: Some(17),
            professionalism: Some(11),
            determination: None,
        };

        let result = super::reconcile_verified_boost(
            &mut fixture.conn,
            &prepared,
            verified_mentality_result(150, 170, previous, current),
        )
        .expect("reconcile Wonderkid Mentality");

        assert_eq!(result.ambition, Some(17));
        assert_eq!(
            player_mentality(&fixture.conn, fixture.snapshot_id),
            current
        );
    }

    #[test]
    fn determination_reconciliation_rewrites_all_current_role_scores() {
        let mut fixture = seeded_player(
            Some(19),
            150,
            170,
            Mentality {
                ambition: Some(11),
                professionalism: Some(12),
                determination: Some(10),
            },
        );
        fixture
            .conn
            .execute_batch(&format!(
                "CREATE TABLE score_rewrites (role_id TEXT NOT NULL);
                 CREATE TRIGGER record_score_rewrite
                 AFTER INSERT ON player_role_scores
                 WHEN NEW.snapshot_id = {}
                 BEGIN
                     INSERT INTO score_rewrites (role_id) VALUES (NEW.role_id);
                 END;",
                fixture.snapshot_id
            ))
            .expect("record role-score rewrites");
        let prepared = super::prepare_wonderkid_mentality_boost(&fixture.conn, PLAYER_UID)
            .expect("prepare Wonderkid Mentality");
        let previous = Mentality {
            ambition: Some(11),
            professionalism: Some(12),
            determination: Some(10),
        };
        let current = Mentality {
            ambition: Some(11),
            professionalism: Some(12),
            determination: Some(20),
        };

        super::reconcile_verified_boost(
            &mut fixture.conn,
            &prepared,
            verified_mentality_result(150, 170, previous, current),
        )
        .expect("reconcile determination");

        let rewrites: i64 = fixture
            .conn
            .query_row("SELECT COUNT(*) FROM score_rewrites", [], |row| row.get(0))
            .expect("count role-score rewrites");
        assert_eq!(rewrites as usize, all_roles().len());
        assert_eq!(
            player_mentality(&fixture.conn, fixture.snapshot_id),
            current
        );
    }

    #[test]
    fn reconciliation_rejects_a_changed_source_request_without_updating_sqlite() {
        let mut fixture = seeded_player(
            Some(21),
            150,
            170,
            Mentality {
                ambition: Some(14),
                professionalism: Some(14),
                determination: Some(14),
            },
        );
        let prepared = super::prepare_current_ability_boost(&fixture.conn, PLAYER_UID)
            .expect("prepare CA boost");
        fixture
            .conn
            .execute(
                "UPDATE snapshots SET bridge_source_request_id = 'scan-other' WHERE id = ?1",
                [fixture.snapshot_id],
            )
            .expect("replace source binding");

        let error = super::reconcile_verified_boost(
            &mut fixture.conn,
            &prepared,
            verified_ca_result(150, 160, 170),
        )
        .expect_err("changed source request must reject reconciliation");

        assert_snapshot_sync(error);
        assert_eq!(player_ca(&fixture.conn, fixture.snapshot_id), 150);
    }

    #[test]
    fn reconciliation_rejects_a_replaced_snapshot_without_updating_the_replacement() {
        let mut fixture = seeded_player(
            Some(21),
            150,
            170,
            Mentality {
                ambition: Some(14),
                professionalism: Some(14),
                determination: Some(14),
            },
        );
        let prepared = super::prepare_current_ability_boost(&fixture.conn, PLAYER_UID)
            .expect("prepare CA boost");
        let replacement =
            ingest_dump_file(&mut fixture.conn, &fixture.dump_path).expect("replace snapshot");
        fixture
            .conn
            .execute(
                "UPDATE snapshots SET bridge_source_request_id = 'scan-replacement' WHERE id = ?1",
                [replacement.id],
            )
            .expect("bind replacement source");

        let error = super::reconcile_verified_boost(
            &mut fixture.conn,
            &prepared,
            verified_ca_result(150, 160, 170),
        )
        .expect_err("replaced snapshot must reject reconciliation");

        assert_snapshot_sync(error);
        assert_eq!(player_ca(&fixture.conn, replacement.id), 150);
    }

    #[test]
    fn reconciliation_rejects_a_deleted_and_reused_snapshot_id() {
        let mut fixture = seeded_player(
            Some(21),
            150,
            170,
            Mentality {
                ambition: Some(14),
                professionalism: Some(14),
                determination: Some(14),
            },
        );
        let prepared = super::prepare_current_ability_boost(&fixture.conn, PLAYER_UID)
            .expect("prepare CA boost");

        fixture
            .conn
            .execute("DELETE FROM snapshots WHERE id = ?1", [fixture.snapshot_id])
            .expect("delete captured snapshot");
        let replacement =
            ingest_dump_file(&mut fixture.conn, &fixture.dump_path).expect("reuse snapshot id");
        assert_eq!(replacement.id, fixture.snapshot_id);
        fixture
            .conn
            .execute(
                "UPDATE snapshots SET bridge_source_request_id = 'scan-player-1' WHERE id = ?1",
                [replacement.id],
            )
            .expect("restore matching bridge source");

        let error = super::reconcile_verified_boost(
            &mut fixture.conn,
            &prepared,
            verified_ca_result(150, 160, 170),
        )
        .expect_err("reused snapshot id must reject reconciliation");

        assert_snapshot_sync(error);
        assert_eq!(player_ca(&fixture.conn, replacement.id), 150);
    }

    #[test]
    fn reconciliation_rejects_an_active_save_switch_without_updating_the_old_save() {
        let mut fixture = seeded_player(
            Some(21),
            150,
            170,
            Mentality {
                ambition: Some(14),
                professionalism: Some(14),
                determination: Some(14),
            },
        );
        let prepared = super::prepare_current_ability_boost(&fixture.conn, PLAYER_UID)
            .expect("prepare CA boost");
        fixture
            .conn
            .execute(
                "INSERT INTO saves (name, is_active) VALUES ('Other save', 0)",
                [],
            )
            .expect("create other save");
        let other_save_id = fixture.conn.last_insert_rowid();
        fixture
            .conn
            .execute(
                "UPDATE saves SET is_active = 0 WHERE id = ?1",
                [fixture.save_id],
            )
            .expect("deactivate original save");
        fixture
            .conn
            .execute(
                "UPDATE saves SET is_active = 1 WHERE id = ?1",
                [other_save_id],
            )
            .expect("activate other save");

        let error = super::reconcile_verified_boost(
            &mut fixture.conn,
            &prepared,
            verified_ca_result(150, 160, 170),
        )
        .expect_err("save switch must reject reconciliation");

        assert_snapshot_sync(error);
        assert_eq!(player_ca(&fixture.conn, fixture.snapshot_id), 150);
    }

    #[test]
    fn determination_reconciliation_rolls_back_player_json_when_role_score_write_fails() {
        let mut fixture = seeded_player(
            Some(19),
            150,
            170,
            Mentality {
                ambition: Some(11),
                professionalism: Some(12),
                determination: Some(10),
            },
        );
        fixture
            .conn
            .execute_batch(&format!(
                "CREATE TRIGGER fail_role_score_insert
                 BEFORE INSERT ON player_role_scores
                 WHEN NEW.snapshot_id = {}
                 BEGIN
                     SELECT RAISE(ABORT, 'forced role-score failure');
                 END;",
                fixture.snapshot_id
            ))
            .expect("create failing role-score trigger");
        let prepared = super::prepare_wonderkid_mentality_boost(&fixture.conn, PLAYER_UID)
            .expect("prepare Wonderkid Mentality");
        let previous = Mentality {
            ambition: Some(11),
            professionalism: Some(12),
            determination: Some(10),
        };
        let current = Mentality {
            ambition: Some(11),
            professionalism: Some(12),
            determination: Some(20),
        };

        let error = super::reconcile_verified_boost(
            &mut fixture.conn,
            &prepared,
            verified_mentality_result(150, 170, previous, current),
        )
        .expect_err("role-score error must roll back the player update");

        assert_snapshot_sync(error);
        assert_eq!(
            player_mentality(&fixture.conn, fixture.snapshot_id),
            previous
        );
        let scores: i64 = fixture
            .conn
            .query_row(
                "SELECT COUNT(*) FROM player_role_scores WHERE snapshot_id = ?1 AND uid = ?2",
                params![fixture.snapshot_id, PLAYER_UID],
                |row| row.get(0),
            )
            .expect("count original role scores");
        assert_eq!(scores as usize, all_roles().len());
    }

    #[test]
    fn reconciliation_refuses_an_unverified_bridge_result_without_updating_sqlite() {
        let mut fixture = seeded_player(
            Some(21),
            150,
            170,
            Mentality {
                ambition: Some(14),
                professionalism: Some(14),
                determination: Some(14),
            },
        );
        let prepared = super::prepare_current_ability_boost(&fixture.conn, PLAYER_UID)
            .expect("prepare CA boost");

        let error = super::reconcile_verified_boost(
            &mut fixture.conn,
            &prepared,
            verified_ca_result(150, 156, 170),
        )
        .expect_err("unexpected result must not update SQLite");

        assert_snapshot_sync(error);
        assert_eq!(player_ca(&fixture.conn, fixture.snapshot_id), 150);
    }

    #[test]
    fn bridge_errors_keep_live_value_and_uncertain_outcomes_distinct() {
        assert!(matches!(
            super::map_bridge_error(PlayerBoostRequestError::Failed(
                "player values changed in FM; Load Data again".to_string(),
            )),
            super::PlayerBoostError::LiveValue { .. }
        ));
        assert!(matches!(
            super::map_bridge_error(PlayerBoostRequestError::Failed(
                "bridge operation rejected the request".to_string(),
            )),
            super::PlayerBoostError::Bridge { .. }
        ));
        assert!(matches!(
            super::map_bridge_error(PlayerBoostRequestError::Unconfirmed(
                "player boost may have changed FM".to_string(),
            )),
            super::PlayerBoostError::SnapshotSync { .. }
        ));
    }
}
