use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;
pub const APP_FOLDER_NAME: &str = "fm-valuescout";
pub const BRIDGE_FOLDER_NAME: &str = "fm-bridge";
pub const STATUS_FILE_NAME: &str = "status.json";
pub const REQUEST_FILE_NAME: &str = "request.json";
pub const DUMP_FILE_NAME: &str = "dump.json";
pub const OPERATION_FULL_DUMP: &str = "full-dump";
pub const OPERATION_BOOST_CURRENT_ABILITY: &str = "boost-current-ability";
pub const OPERATION_BOOST_STAFF_CURRENT_ABILITY: &str = "boost-staff-current-ability";
pub const OPERATION_WONDERKID_MENTALITY: &str = "wonderkid-mentality";
pub const OPERATION_MOVE_SQUAD_PLAYERS: &str = "move-squad-players";
pub const OPERATION_APPLY_STAFF_CHANGES: &str = "apply-staff-changes";
pub const OPERATION_READ_STAFF_LIMITS: &str = "read-staff-limits";
pub const OPERATION_APPLY_TRAINING: &str = "apply-training";
pub const PLAYER_DATABASE_SCOPE_MEN: &str = "men";

static REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static ACTIVE_BRIDGE_DIRECTORIES: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();

struct BridgeRequestGuard {
    bridge_directory: PathBuf,
}

impl Drop for BridgeRequestGuard {
    fn drop(&mut self) {
        if let Some(active) = ACTIVE_BRIDGE_DIRECTORIES.get() {
            if let Ok(mut directories) = active.lock() {
                directories.remove(&self.bridge_directory);
            }
        }
    }
}

fn default_player_database_scope() -> String {
    PLAYER_DATABASE_SCOPE_MEN.to_string()
}

/// Default wait for the bridge to finish a dump after a request is written.
pub const DEFAULT_DUMP_WAIT_TIMEOUT: Duration = Duration::from_secs(120);
pub const DEFAULT_DUMP_POLL_INTERVAL: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeStatus {
    pub protocol_version: u32,
    pub plugin_version: String,
    pub state: String,
    pub updated_at_utc: String,
    pub game_plugin_module_present: bool,
    pub game_assembly_module_present: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub players_found: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_boosts_supported: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_boost: Option<PlayerBoostResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staff_boosts_supported: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staff_boost: Option<StaffBoostResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub squad_moves_supported: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub squad_moves: Option<SquadMoveResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staff_changes_supported: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staff_changes: Option<StaffChangeResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staff_limits_supported: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staff_limits: Option<StaffLimitsResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub training_supported: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub training: Option<TrainingResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeRequest {
    pub protocol_version: u32,
    pub request_id: String,
    pub created_at_utc: String,
    pub operation: String,
    #[serde(default = "default_player_database_scope")]
    pub player_database_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DumpRequestResult {
    pub request_id: String,
    pub state: String,
    pub players_found: Option<i32>,
    pub dump_present: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerBoostResult {
    pub operation: String,
    pub outcome: String,
    pub rollback: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_current_ability: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_ability: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub potential_ability: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_ambition: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ambition: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_professionalism: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub professionalism: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_determination: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub determination: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffBoostResult {
    pub operation: String,
    pub outcome: String,
    pub rollback: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_current_ability: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_ability: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub potential_ability: Option<i32>,
}

/// Each requested squad move's outcome, in request order. It carries no player identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SquadMoveResult {
    #[serde(default)]
    pub moves: Vec<SquadMoveOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SquadMoveOutcome {
    /// `moved`, `already-there`, `not-found`, `not-in-club-teams`, `no-target-team`, or
    /// `not-confirmed`.
    pub outcome: String,
    /// FM's team type the player is in afterwards, when the bridge read it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_type: Option<i32>,
    /// The level of `team_type`: `senior`, `reserve`, or `youth`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_level: Option<String>,
}

/// One player to move to the managed club's team at a level: `senior`, `reserve`, or `youth`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SquadMoveItem {
    pub player_uid: u32,
    pub target_level: &'static str,
}

/// The bridge's answer to a squad-move request: each move's outcome, and the error that stopped
/// the request early when one did. FM may have made some moves before such an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SquadMoveResponse {
    pub moves: Vec<SquadMoveOutcome>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct SquadMoveRequest<'a> {
    protocol_version: u32,
    request_id: String,
    created_at_utc: String,
    operation: &'static str,
    player_database_scope: &'static str,
    source_request_id: &'a str,
    club_uid: u32,
    squad_moves: &'a [SquadMoveItem],
}

/// The most players one squad-move request may move; the bridge enforces the same bound.
pub const MAX_SQUAD_MOVES: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerBoostOperation {
    CurrentAbility {
        increment: i32,
    },
    WonderkidMentality {
        expected_ambition: Option<i32>,
        expected_professionalism: Option<i32>,
        expected_determination: Option<i32>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlayerBoostRequest {
    protocol_version: u32,
    request_id: String,
    created_at_utc: String,
    operation: String,
    player_database_scope: String,
    source_request_id: String,
    player_uid: u32,
    expected_current_ability: i32,
    expected_potential_ability: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_ability_increment: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_ambition: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_professionalism: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_determination: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StaffBoostRequest {
    protocol_version: u32,
    request_id: String,
    created_at_utc: String,
    operation: String,
    player_database_scope: String,
    source_request_id: String,
    staff_uid: u32,
    expected_current_ability: i32,
    expected_potential_ability: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum BridgeStatusError {
    UnsupportedPlatform(String),
    Missing(String),
    Corrupt(String),
    UnsupportedVersion(String),
}

impl std::fmt::Display for BridgeStatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedPlatform(message)
            | Self::Missing(message)
            | Self::Corrupt(message)
            | Self::UnsupportedVersion(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for BridgeStatusError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum DumpRequestError {
    UnsupportedPlatform(String),
    Missing(String),
    Corrupt(String),
    Timeout(String),
    WriteFailed(String),
}

impl std::fmt::Display for DumpRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedPlatform(message)
            | Self::Missing(message)
            | Self::Corrupt(message)
            | Self::Timeout(message)
            | Self::WriteFailed(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for DumpRequestError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum PlayerBoostRequestError {
    UnsupportedPlatform(String),
    Missing(String),
    Corrupt(String),
    Timeout(String),
    WriteFailed(String),
    Unavailable(String),
    Failed(String),
    Unconfirmed(String),
}

impl std::fmt::Display for PlayerBoostRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedPlatform(message)
            | Self::Missing(message)
            | Self::Corrupt(message)
            | Self::Timeout(message)
            | Self::WriteFailed(message)
            | Self::Unavailable(message)
            | Self::Failed(message)
            | Self::Unconfirmed(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for PlayerBoostRequestError {}

pub type StaffBoostRequestError = PlayerBoostRequestError;

impl From<BridgeStatusError> for DumpRequestError {
    fn from(value: BridgeStatusError) -> Self {
        match value {
            BridgeStatusError::UnsupportedPlatform(message) => Self::UnsupportedPlatform(message),
            BridgeStatusError::Missing(message) => Self::Missing(message),
            BridgeStatusError::Corrupt(message) => Self::Corrupt(message),
            BridgeStatusError::UnsupportedVersion(message) => Self::Corrupt(message),
        }
    }
}

impl From<BridgeStatusError> for PlayerBoostRequestError {
    fn from(value: BridgeStatusError) -> Self {
        match value {
            BridgeStatusError::UnsupportedPlatform(message) => Self::UnsupportedPlatform(message),
            BridgeStatusError::Missing(message) => Self::Missing(message),
            BridgeStatusError::Corrupt(message)
            | BridgeStatusError::UnsupportedVersion(message) => Self::Corrupt(message),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DumpWaitConfig {
    pub timeout: Duration,
    pub poll_interval: Duration,
}

impl Default for DumpWaitConfig {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_DUMP_WAIT_TIMEOUT,
            poll_interval: DEFAULT_DUMP_POLL_INTERVAL,
        }
    }
}

pub fn bridge_directory_from_local_app_data(local_app_data: &Path) -> PathBuf {
    local_app_data
        .join(APP_FOLDER_NAME)
        .join(BRIDGE_FOLDER_NAME)
}

pub fn status_path(bridge_directory: &Path) -> PathBuf {
    bridge_directory.join(STATUS_FILE_NAME)
}

pub fn request_path(bridge_directory: &Path) -> PathBuf {
    bridge_directory.join(REQUEST_FILE_NAME)
}

pub fn dump_path(bridge_directory: &Path) -> PathBuf {
    bridge_directory.join(DUMP_FILE_NAME)
}

/// Resolves `%LOCALAPPDATA%\fm-valuescout\fm-bridge` on Windows.
/// Non-Windows hosts always return [`BridgeStatusError::UnsupportedPlatform`].
pub fn resolve_bridge_directory() -> Result<PathBuf, BridgeStatusError> {
    Ok(bridge_directory_from_local_app_data(&local_app_data_dir()?))
}

fn local_app_data_dir() -> Result<PathBuf, BridgeStatusError> {
    #[cfg(windows)]
    {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .ok_or_else(|| {
                BridgeStatusError::UnsupportedPlatform(
                    "LOCALAPPDATA is not set; cannot resolve bridge directory".to_string(),
                )
            })
    }

    #[cfg(not(windows))]
    {
        Err(BridgeStatusError::UnsupportedPlatform(
            "FM27 memory read requires Windows".to_string(),
        ))
    }
}

pub fn read_bridge_status(bridge_directory: &Path) -> Result<BridgeStatus, BridgeStatusError> {
    let path = status_path(bridge_directory);
    let json = fs::read_to_string(&path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => {
            BridgeStatusError::Missing("status.json not found".to_string())
        }
        _ => BridgeStatusError::Corrupt("status.json could not be read".to_string()),
    })?;
    parse_bridge_status(&json)
}

pub fn parse_bridge_status(json: &str) -> Result<BridgeStatus, BridgeStatusError> {
    if json.trim().is_empty() {
        return Err(BridgeStatusError::Corrupt(
            "status.json is empty".to_string(),
        ));
    }

    let status: BridgeStatus = serde_json::from_str(json).map_err(|error| {
        BridgeStatusError::Corrupt(format!("status.json is not valid JSON: {error}"))
    })?;

    if status.protocol_version != PROTOCOL_VERSION {
        return Err(BridgeStatusError::UnsupportedVersion(format!(
            "unsupported bridge protocol version {}; expected {PROTOCOL_VERSION}",
            status.protocol_version
        )));
    }

    Ok(status)
}

pub fn new_request_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let sequence = REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("req-{millis}-{sequence}")
}

pub fn utc_now_rfc3339() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0));
    format_unix_secs_as_rfc3339(duration.as_secs(), duration.subsec_millis())
}

fn format_unix_secs_as_rfc3339(secs: u64, millis: u32) -> String {
    // ponytail: hand-rolled UTC civil date for request timestamps
    // Upgrade to the `time` crate if TTL clock skew or DST bugs appear
    let days = (secs / 86_400) as i64;
    let day_secs = (secs % 86_400) as u32;
    let (year, month, day) = civil_from_days(days);
    let hour = day_secs / 3600;
    let minute = (day_secs % 3600) / 60;
    let second = day_secs % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z")
}

/// Howard's civil_from_days — days since Unix epoch → Y-M-D.
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

pub fn write_player_dump_request(
    bridge_directory: &Path,
    request: &BridgeRequest,
) -> Result<PathBuf, DumpRequestError> {
    fs::create_dir_all(bridge_directory).map_err(|error| {
        DumpRequestError::WriteFailed(format!("could not create bridge directory: {error}"))
    })?;

    let path = request_path(bridge_directory);
    let temp_path = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(request).map_err(|error| {
        DumpRequestError::WriteFailed(format!("could not serialize request: {error}"))
    })?;

    fs::write(&temp_path, json).map_err(|error| {
        DumpRequestError::WriteFailed(format!("could not write request temp file: {error}"))
    })?;
    fs::rename(&temp_path, &path).map_err(|error| {
        DumpRequestError::WriteFailed(format!("could not replace request.json: {error}"))
    })?;

    Ok(path)
}

/// Writes `request.json` and polls `status.json` until this request reaches a terminal state.
pub fn request_player_dump(
    bridge_directory: &Path,
    wait: DumpWaitConfig,
) -> Result<DumpRequestResult, DumpRequestError> {
    let _bridge_request_guard =
        acquire_bridge_request_guard(bridge_directory).map_err(DumpRequestError::WriteFailed)?;

    let request_id = new_request_id();
    let request = BridgeRequest {
        protocol_version: PROTOCOL_VERSION,
        request_id: request_id.clone(),
        created_at_utc: utc_now_rfc3339(),
        operation: OPERATION_FULL_DUMP.to_string(),
        player_database_scope: PLAYER_DATABASE_SCOPE_MEN.to_string(),
    };

    write_player_dump_request(bridge_directory, &request)?;
    wait_for_request_terminal(bridge_directory, &request_id, wait)
}

pub fn request_player_boost_from_local_app_data(
    source_request_id: &str,
    player_uid: u32,
    expected_current_ability: i32,
    expected_potential_ability: i32,
    operation: PlayerBoostOperation,
    wait: DumpWaitConfig,
) -> Result<PlayerBoostResult, PlayerBoostRequestError> {
    let bridge_directory = resolve_bridge_directory().map_err(PlayerBoostRequestError::from)?;
    request_player_boost(
        &bridge_directory,
        source_request_id,
        player_uid,
        expected_current_ability,
        expected_potential_ability,
        operation,
        wait,
    )
}

pub fn request_player_boost(
    bridge_directory: &Path,
    source_request_id: &str,
    player_uid: u32,
    expected_current_ability: i32,
    expected_potential_ability: i32,
    operation: PlayerBoostOperation,
    wait: DumpWaitConfig,
) -> Result<PlayerBoostResult, PlayerBoostRequestError> {
    validate_player_boost_request(
        source_request_id,
        player_uid,
        expected_current_ability,
        expected_potential_ability,
        &operation,
    )?;
    let _bridge_request_guard = acquire_bridge_request_guard(bridge_directory)
        .map_err(PlayerBoostRequestError::Unavailable)?;
    ensure_player_boost_is_available(bridge_directory)?;

    let request_id = new_request_id();
    let request = PlayerBoostRequest::from_operation(
        request_id.clone(),
        source_request_id.to_string(),
        player_uid,
        expected_current_ability,
        expected_potential_ability,
        operation,
    );
    write_player_boost_request(bridge_directory, &request)?;

    match wait_for_player_boost_terminal(bridge_directory, &request_id, wait) {
        Ok(result) => Ok(result),
        Err(error @ PlayerBoostRequestError::Failed(_))
        | Err(error @ PlayerBoostRequestError::Unconfirmed(_)) => Err(error),
        Err(error) => Err(PlayerBoostRequestError::Unconfirmed(format!(
            "could not confirm the player boost result; Load Data again before retrying ({error})"
        ))),
    }
}

pub fn request_staff_boost_from_local_app_data(
    source_request_id: &str,
    staff_uid: u32,
    expected_current_ability: i32,
    expected_potential_ability: i32,
    wait: DumpWaitConfig,
) -> Result<StaffBoostResult, StaffBoostRequestError> {
    let bridge_directory = resolve_bridge_directory().map_err(PlayerBoostRequestError::from)?;
    request_staff_boost(
        &bridge_directory,
        source_request_id,
        staff_uid,
        expected_current_ability,
        expected_potential_ability,
        wait,
    )
}

pub fn request_staff_boost(
    bridge_directory: &Path,
    source_request_id: &str,
    staff_uid: u32,
    expected_current_ability: i32,
    expected_potential_ability: i32,
    wait: DumpWaitConfig,
) -> Result<StaffBoostResult, StaffBoostRequestError> {
    validate_staff_boost_request(
        source_request_id,
        staff_uid,
        expected_current_ability,
        expected_potential_ability,
    )?;
    let _bridge_request_guard = acquire_bridge_request_guard(bridge_directory)
        .map_err(PlayerBoostRequestError::Unavailable)?;
    ensure_staff_boost_is_available(bridge_directory)?;

    let request_id = new_request_id();
    let request = StaffBoostRequest {
        protocol_version: PROTOCOL_VERSION,
        request_id: request_id.clone(),
        created_at_utc: utc_now_rfc3339(),
        operation: OPERATION_BOOST_STAFF_CURRENT_ABILITY.to_string(),
        player_database_scope: PLAYER_DATABASE_SCOPE_MEN.to_string(),
        source_request_id: source_request_id.to_string(),
        staff_uid,
        expected_current_ability,
        expected_potential_ability,
    };
    write_staff_boost_request(bridge_directory, &request)?;

    match wait_for_staff_boost_terminal(bridge_directory, &request_id, wait) {
        Ok(result) => Ok(result),
        Err(error @ PlayerBoostRequestError::Failed(_))
        | Err(error @ PlayerBoostRequestError::Unconfirmed(_)) => Err(error),
        Err(error) => Err(PlayerBoostRequestError::Unconfirmed(format!(
            "could not confirm the staff boost result; Load Data again before retrying ({error})"
        ))),
    }
}

pub fn request_squad_moves_from_local_app_data(
    source_request_id: &str,
    club_uid: u32,
    moves: &[SquadMoveItem],
    wait: DumpWaitConfig,
) -> Result<SquadMoveResponse, PlayerBoostRequestError> {
    let bridge_directory = resolve_bridge_directory().map_err(PlayerBoostRequestError::from)?;
    request_squad_moves(&bridge_directory, source_request_id, club_uid, moves, wait)
}

/// Asks the bridge to move the managed club's players between its teams and waits for each
/// move's outcome.
pub fn request_squad_moves(
    bridge_directory: &Path,
    source_request_id: &str,
    club_uid: u32,
    moves: &[SquadMoveItem],
    wait: DumpWaitConfig,
) -> Result<SquadMoveResponse, PlayerBoostRequestError> {
    if source_request_id.trim().is_empty() || club_uid == 0 {
        return Err(PlayerBoostRequestError::WriteFailed(
            "squad moves need the live scan and the managed club".to_string(),
        ));
    }
    if moves.is_empty() || moves.len() > MAX_SQUAD_MOVES {
        return Err(PlayerBoostRequestError::WriteFailed(format!(
            "squad moves must list 1 through {MAX_SQUAD_MOVES} players"
        )));
    }
    let _bridge_request_guard = acquire_bridge_request_guard(bridge_directory)
        .map_err(PlayerBoostRequestError::Unavailable)?;
    let status = read_bridge_status(bridge_directory).map_err(PlayerBoostRequestError::from)?;
    if status.squad_moves_supported != Some(true) {
        return Err(PlayerBoostRequestError::Unavailable(
            SQUAD_MOVES_UNAVAILABLE.to_string(),
        ));
    }

    let request_id = new_request_id();
    let request = SquadMoveRequest {
        protocol_version: PROTOCOL_VERSION,
        request_id: request_id.clone(),
        created_at_utc: utc_now_rfc3339(),
        operation: OPERATION_MOVE_SQUAD_PLAYERS,
        player_database_scope: PLAYER_DATABASE_SCOPE_MEN,
        source_request_id,
        club_uid,
        squad_moves: moves,
    };
    write_bridge_request(bridge_directory, &request)?;
    wait_for_squad_moves_terminal(bridge_directory, &request_id, moves.len(), wait)
}

/// Why squad moves are unavailable when the bridge does not offer them.
pub const SQUAD_MOVES_UNAVAILABLE: &str =
    "Load Data with FM open on this save before moving squads";

fn write_bridge_request<T: Serialize>(
    bridge_directory: &Path,
    request: &T,
) -> Result<(), PlayerBoostRequestError> {
    fs::create_dir_all(bridge_directory).map_err(|error| {
        PlayerBoostRequestError::WriteFailed(format!("could not create bridge directory: {error}"))
    })?;
    let path = request_path(bridge_directory);
    let temp_path = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(request).map_err(|error| {
        PlayerBoostRequestError::WriteFailed(format!("could not serialize bridge request: {error}"))
    })?;
    fs::write(&temp_path, json).map_err(|error| {
        PlayerBoostRequestError::WriteFailed(format!(
            "could not write bridge request temp file: {error}"
        ))
    })?;
    fs::rename(&temp_path, &path).map_err(|error| {
        PlayerBoostRequestError::WriteFailed(format!("could not replace bridge request: {error}"))
    })
}

pub fn wait_for_squad_moves_terminal(
    bridge_directory: &Path,
    request_id: &str,
    move_count: usize,
    wait: DumpWaitConfig,
) -> Result<SquadMoveResponse, PlayerBoostRequestError> {
    let deadline = Instant::now() + wait.timeout;
    loop {
        match read_bridge_status(bridge_directory) {
            Ok(status)
                if status.request_id.as_deref() == Some(request_id)
                    && is_terminal_state(&status.state) =>
            {
                let error = (status.state == "failed").then(|| {
                    status
                        .error
                        .clone()
                        .unwrap_or_else(|| "squad moves failed without a bridge error".to_string())
                });
                return match status.squad_moves {
                    Some(result) if result.moves.len() == move_count => Ok(SquadMoveResponse {
                        moves: result.moves,
                        error,
                    }),
                    Some(_) => Err(PlayerBoostRequestError::Corrupt(
                        "the bridge reported a different number of squad moves than were sent"
                            .to_string(),
                    )),
                    None => Err(PlayerBoostRequestError::Failed(error.unwrap_or_else(
                        || "the bridge finished the squad moves without their outcomes".to_string(),
                    ))),
                };
            }
            Ok(_) => {}
            Err(BridgeStatusError::Missing(_)) => {}
            Err(error) => return Err(error.into()),
        }

        if Instant::now() >= deadline {
            return Err(PlayerBoostRequestError::Timeout(
                "timed out waiting for FM to finish the squad moves; Load Data to see FM's squads"
                    .to_string(),
            ));
        }

        thread::sleep(wait.poll_interval);
    }
}

impl PlayerBoostRequest {
    fn from_operation(
        request_id: String,
        source_request_id: String,
        player_uid: u32,
        expected_current_ability: i32,
        expected_potential_ability: i32,
        operation: PlayerBoostOperation,
    ) -> Self {
        let (
            operation,
            current_ability_increment,
            expected_ambition,
            expected_professionalism,
            expected_determination,
        ) = match operation {
            PlayerBoostOperation::CurrentAbility { increment } => (
                OPERATION_BOOST_CURRENT_ABILITY.to_string(),
                Some(increment),
                None,
                None,
                None,
            ),
            PlayerBoostOperation::WonderkidMentality {
                expected_ambition,
                expected_professionalism,
                expected_determination,
            } => (
                OPERATION_WONDERKID_MENTALITY.to_string(),
                None,
                expected_ambition,
                expected_professionalism,
                expected_determination,
            ),
        };

        Self {
            protocol_version: PROTOCOL_VERSION,
            request_id,
            created_at_utc: utc_now_rfc3339(),
            operation,
            player_database_scope: PLAYER_DATABASE_SCOPE_MEN.to_string(),
            source_request_id,
            player_uid,
            expected_current_ability,
            expected_potential_ability,
            current_ability_increment,
            expected_ambition,
            expected_professionalism,
            expected_determination,
        }
    }
}

fn validate_player_boost_request(
    source_request_id: &str,
    player_uid: u32,
    expected_current_ability: i32,
    expected_potential_ability: i32,
    operation: &PlayerBoostOperation,
) -> Result<(), PlayerBoostRequestError> {
    if source_request_id.trim().is_empty() {
        return Err(PlayerBoostRequestError::WriteFailed(
            "source request ID is required for player boosts".to_string(),
        ));
    }
    if player_uid == 0 {
        return Err(PlayerBoostRequestError::WriteFailed(
            "player UID is required for player boosts".to_string(),
        ));
    }
    if !is_ability(expected_current_ability)
        || !is_ability(expected_potential_ability)
        || expected_current_ability > expected_potential_ability
    {
        return Err(PlayerBoostRequestError::WriteFailed(
            "expected current ability and potential ability must be 1 through 200 with CA not above PA"
                .to_string(),
        ));
    }

    match operation {
        PlayerBoostOperation::CurrentAbility { increment: 1..=20 } => {}
        PlayerBoostOperation::CurrentAbility { .. } => {
            return Err(PlayerBoostRequestError::WriteFailed(
                "current ability increment must be 1 through 20".to_string(),
            ));
        }
        PlayerBoostOperation::WonderkidMentality {
            expected_ambition,
            expected_professionalism,
            expected_determination,
        } => {
            let values = [
                expected_ambition,
                expected_professionalism,
                expected_determination,
            ];
            if values
                .iter()
                .any(|value| value.is_some_and(|value| !is_mentality(value)))
            {
                return Err(PlayerBoostRequestError::WriteFailed(
                    "Wonderkid Mentality values must be null or 1 through 20".to_string(),
                ));
            }
            if !values
                .iter()
                .any(|value| value.is_some_and(|value| value <= 10))
            {
                return Err(PlayerBoostRequestError::WriteFailed(
                    "Wonderkid Mentality requires a known value from 1 through 10".to_string(),
                ));
            }
        }
    }

    Ok(())
}

fn validate_staff_boost_request(
    source_request_id: &str,
    staff_uid: u32,
    expected_current_ability: i32,
    expected_potential_ability: i32,
) -> Result<(), StaffBoostRequestError> {
    if source_request_id.trim().is_empty() {
        return Err(PlayerBoostRequestError::WriteFailed(
            "source request ID is required for staff boosts".to_string(),
        ));
    }
    if staff_uid == 0 {
        return Err(PlayerBoostRequestError::WriteFailed(
            "staff UID is required for staff boosts".to_string(),
        ));
    }
    if !is_ability(expected_current_ability)
        || !is_ability(expected_potential_ability)
        || expected_current_ability > expected_potential_ability
    {
        return Err(PlayerBoostRequestError::WriteFailed(
            "expected current ability and potential ability must be 1 through 200 with CA not above PA"
                .to_string(),
        ));
    }

    Ok(())
}

fn ensure_player_boost_is_available(
    bridge_directory: &Path,
) -> Result<(), PlayerBoostRequestError> {
    let status = read_bridge_status(bridge_directory).map_err(PlayerBoostRequestError::from)?;
    if status.player_boosts_supported != Some(true) {
        return Err(PlayerBoostRequestError::Unavailable(
            "Load Data again before using player boosts".to_string(),
        ));
    }

    Ok(())
}

fn ensure_staff_boost_is_available(bridge_directory: &Path) -> Result<(), StaffBoostRequestError> {
    let status = read_bridge_status(bridge_directory).map_err(PlayerBoostRequestError::from)?;
    if status.staff_boosts_supported != Some(true) {
        return Err(PlayerBoostRequestError::Unavailable(
            "Load Data again before using staff boosts".to_string(),
        ));
    }

    Ok(())
}

fn write_player_boost_request(
    bridge_directory: &Path,
    request: &PlayerBoostRequest,
) -> Result<PathBuf, PlayerBoostRequestError> {
    fs::create_dir_all(bridge_directory).map_err(|error| {
        PlayerBoostRequestError::WriteFailed(format!(
            "could not create bridge directory for player boost: {error}"
        ))
    })?;

    let path = request_path(bridge_directory);
    let temp_path = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(request).map_err(|error| {
        PlayerBoostRequestError::WriteFailed(format!(
            "could not serialize player boost request: {error}"
        ))
    })?;
    fs::write(&temp_path, json).map_err(|error| {
        PlayerBoostRequestError::WriteFailed(format!(
            "could not write player boost request temp file: {error}"
        ))
    })?;
    fs::rename(&temp_path, &path).map_err(|error| {
        PlayerBoostRequestError::WriteFailed(format!(
            "could not replace player boost request: {error}"
        ))
    })?;

    Ok(path)
}

fn write_staff_boost_request(
    bridge_directory: &Path,
    request: &StaffBoostRequest,
) -> Result<PathBuf, StaffBoostRequestError> {
    fs::create_dir_all(bridge_directory).map_err(|error| {
        PlayerBoostRequestError::WriteFailed(format!(
            "could not create bridge directory for staff boost: {error}"
        ))
    })?;

    let path = request_path(bridge_directory);
    let temp_path = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(request).map_err(|error| {
        PlayerBoostRequestError::WriteFailed(format!(
            "could not serialize staff boost request: {error}"
        ))
    })?;
    fs::write(&temp_path, json).map_err(|error| {
        PlayerBoostRequestError::WriteFailed(format!(
            "could not write staff boost request temp file: {error}"
        ))
    })?;
    fs::rename(&temp_path, &path).map_err(|error| {
        PlayerBoostRequestError::WriteFailed(format!(
            "could not replace staff boost request: {error}"
        ))
    })?;

    Ok(path)
}

pub fn wait_for_player_boost_terminal(
    bridge_directory: &Path,
    request_id: &str,
    wait: DumpWaitConfig,
) -> Result<PlayerBoostResult, PlayerBoostRequestError> {
    let deadline = Instant::now() + wait.timeout;

    loop {
        match read_bridge_status(bridge_directory) {
            Ok(status)
                if status.request_id.as_deref() == Some(request_id)
                    && is_terminal_state(&status.state) =>
            {
                if status.state == "failed" {
                    let message = status.error.unwrap_or_else(|| {
                        "player boost failed without a bridge error".to_string()
                    });
                    if status
                        .player_boost
                        .as_ref()
                        .is_some_and(|boost| boost.rollback == "unverified")
                        || (status.player_boost.is_none()
                            && !is_known_pre_write_player_boost_failure(&message))
                    {
                        return Err(PlayerBoostRequestError::Unconfirmed(
                            format!(
                                "player boost may have changed FM before its result could be verified; Load Data again before retrying ({message})"
                            ),
                        ));
                    }
                    return Err(PlayerBoostRequestError::Failed(message));
                }

                let boost = status.player_boost.ok_or_else(|| {
                    PlayerBoostRequestError::Corrupt(
                        "bridge reported a ready player boost without a verified result"
                            .to_string(),
                    )
                })?;
                if boost.outcome != "verified" || boost.rollback != "not-needed" {
                    return Err(PlayerBoostRequestError::Unconfirmed(
                        status
                            .error
                            .unwrap_or_else(|| "player boost was not verified".to_string()),
                    ));
                }
                return Ok(boost);
            }
            Ok(_) => {}
            Err(BridgeStatusError::Missing(_)) => {}
            Err(error) => return Err(error.into()),
        }

        if Instant::now() >= deadline {
            return Err(PlayerBoostRequestError::Timeout(format!(
                "timed out waiting for player boost request {request_id}"
            )));
        }

        thread::sleep(wait.poll_interval);
    }
}

pub fn wait_for_staff_boost_terminal(
    bridge_directory: &Path,
    request_id: &str,
    wait: DumpWaitConfig,
) -> Result<StaffBoostResult, StaffBoostRequestError> {
    let deadline = Instant::now() + wait.timeout;

    loop {
        match read_bridge_status(bridge_directory) {
            Ok(status)
                if status.request_id.as_deref() == Some(request_id)
                    && is_terminal_state(&status.state) =>
            {
                if status.state == "failed" {
                    let message = status
                        .error
                        .unwrap_or_else(|| "staff boost failed without a bridge error".to_string());
                    if status
                        .staff_boost
                        .as_ref()
                        .is_some_and(|boost| boost.rollback == "unverified")
                        || (status.staff_boost.is_none()
                            && !is_known_pre_write_staff_boost_failure(&message))
                    {
                        return Err(PlayerBoostRequestError::Unconfirmed(format!(
                            "staff boost may have changed FM before its result could be verified; Load Data again before retrying ({message})"
                        )));
                    }
                    return Err(PlayerBoostRequestError::Failed(message));
                }

                let boost = status.staff_boost.ok_or_else(|| {
                    PlayerBoostRequestError::Corrupt(
                        "bridge reported a ready staff boost without a verified result".to_string(),
                    )
                })?;
                if boost.outcome != "verified" || boost.rollback != "not-needed" {
                    return Err(PlayerBoostRequestError::Unconfirmed(
                        status
                            .error
                            .unwrap_or_else(|| "staff boost was not verified".to_string()),
                    ));
                }
                return Ok(boost);
            }
            Ok(_) => {}
            Err(BridgeStatusError::Missing(_)) => {}
            Err(error) => return Err(error.into()),
        }

        if Instant::now() >= deadline {
            return Err(PlayerBoostRequestError::Timeout(format!(
                "timed out waiting for staff boost request {request_id}"
            )));
        }

        thread::sleep(wait.poll_interval);
    }
}

fn is_ability(value: i32) -> bool {
    (1..=200).contains(&value)
}

fn is_mentality(value: i32) -> bool {
    (1..=20).contains(&value)
}

fn is_known_pre_write_player_boost_failure(message: &str) -> bool {
    matches!(
        message,
        "invalid player boost request; update the app and retry"
            | "this FM build is not approved for player boosts; update the bridge plugin and Load Data"
            | "Load Data again before using player boosts"
            | "player was not found in the latest live scan; Load Data again"
            | "player values changed in FM; Load Data again"
            | "player identity changed in FM; Load Data again"
            | "could not safely read the player; Load Data again"
            | "player values are not valid for this boost; Load Data again"
            | "current ability is already at its potential limit"
            | "player boost cancelled before it started"
            | "could not detect FM game_plugin.dll version; refusing player boost (fail closed)"
            | "bridge work is already in progress; retry the request"
    )
}

fn is_known_pre_write_staff_boost_failure(message: &str) -> bool {
    matches!(
        message,
        "invalid staff boost request; update the app and retry"
            | "this FM build is not approved for staff boosts; update the bridge plugin and Load Data"
            | "Load Data again before using staff boosts"
            | "staff member was not found in the latest live scan; Load Data again"
            | "staff values changed in FM; Load Data again"
            | "staff identity changed in FM; Load Data again"
            | "could not safely read the staff member; Load Data again"
            | "staff values are not valid for this boost; Load Data again"
            | "current ability is already at its potential limit"
            | "staff boost cancelled before it started"
            | "could not detect FM game_plugin.dll version; refusing staff boost (fail closed)"
            | "bridge work is already in progress; retry the request"
    )
}

fn acquire_bridge_request_guard(bridge_directory: &Path) -> Result<BridgeRequestGuard, String> {
    let active = ACTIVE_BRIDGE_DIRECTORIES.get_or_init(|| Mutex::new(HashSet::new()));
    let mut directories = active
        .lock()
        .map_err(|_| "bridge request state is unavailable".to_string())?;
    let bridge_directory = bridge_directory.to_path_buf();
    if !directories.insert(bridge_directory.clone()) {
        return Err("a bridge request is already in progress; wait for it to finish".to_string());
    }

    Ok(BridgeRequestGuard { bridge_directory })
}

pub fn wait_for_request_terminal(
    bridge_directory: &Path,
    request_id: &str,
    wait: DumpWaitConfig,
) -> Result<DumpRequestResult, DumpRequestError> {
    let deadline = Instant::now() + wait.timeout;

    loop {
        match read_bridge_status(bridge_directory) {
            Ok(status)
                if status.request_id.as_deref() == Some(request_id)
                    && is_terminal_state(&status.state) =>
            {
                let dump_present = dump_path(bridge_directory).is_file();
                return Ok(DumpRequestResult {
                    request_id: request_id.to_string(),
                    state: status.state,
                    players_found: status.players_found,
                    dump_present,
                    error: status.error,
                });
            }
            Ok(_) => {}
            Err(BridgeStatusError::Missing(_)) => {}
            Err(error) => return Err(error.into()),
        }

        if Instant::now() >= deadline {
            return Err(DumpRequestError::Timeout(format!(
                "timed out waiting for dump request {request_id}"
            )));
        }

        thread::sleep(wait.poll_interval);
    }
}

fn is_terminal_state(state: &str) -> bool {
    matches!(state, "ready" | "failed")
}

/// Production IPC entry: resolve LocalAppData bridge dir, then request + wait.
pub fn request_player_dump_from_local_app_data(
    wait: DumpWaitConfig,
) -> Result<DumpRequestResult, DumpRequestError> {
    let bridge_directory = resolve_bridge_directory().map_err(DumpRequestError::from)?;
    request_player_dump(&bridge_directory, wait)
}

/// Each requested staff change's outcome, in request order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffChangeResult {
    #[serde(default)]
    pub changes: Vec<StaffChangeOutcome>,
}

/// What FM did with one staff change, and the person's contract afterwards as the bridge read it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffChangeOutcome {
    /// `sacked`, `hired`, `changed`, `left-unemployed`, `not-found`, `not-at-club`,
    /// `no-target-team`, or `not-confirmed`.
    pub outcome: String,
    /// The contract's club; absent when the person has no contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub club_uid: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub club: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub division: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weekly_wage_gbp: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_expiry_year: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_expiry_day_of_year: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_type: Option<i32>,
    /// The level of `team_type`: `senior`, `reserve`, or `youth`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_level: Option<String>,
}

/// One staff change at the managed club: `sack` ends the contract, `hire` brings the person in
/// with FM's job type at a team level, and `change` does both.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffChangeItem {
    pub staff_uid: u32,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_type: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_level: Option<&'static str>,
}

/// The bridge's answer to a staff-change request: each change's outcome, and the error that
/// stopped the request early when one did. FM may have made some changes before such an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffChangeResponse {
    pub changes: Vec<StaffChangeOutcome>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StaffChangeRequest<'a> {
    protocol_version: u32,
    request_id: String,
    created_at_utc: String,
    operation: &'static str,
    player_database_scope: &'static str,
    source_request_id: &'a str,
    club_uid: u32,
    staff_changes: &'a [StaffChangeItem],
}

/// The most staff one staff-change request may change; the bridge enforces the same bound.
pub const MAX_STAFF_CHANGES: usize = 100;

/// Why staff changes are unavailable when the bridge does not offer them.
pub const STAFF_CHANGES_UNAVAILABLE: &str =
    "Load Data with FM open on this save before changing staff in FM";

pub fn request_staff_changes_from_local_app_data(
    source_request_id: &str,
    club_uid: u32,
    changes: &[StaffChangeItem],
    wait: DumpWaitConfig,
) -> Result<StaffChangeResponse, PlayerBoostRequestError> {
    let bridge_directory = resolve_bridge_directory().map_err(PlayerBoostRequestError::from)?;
    request_staff_changes(
        &bridge_directory,
        source_request_id,
        club_uid,
        changes,
        wait,
    )
}

/// Asks the bridge to sack, hire, and re-hire the managed club's staff and waits for each
/// change's outcome.
pub fn request_staff_changes(
    bridge_directory: &Path,
    source_request_id: &str,
    club_uid: u32,
    changes: &[StaffChangeItem],
    wait: DumpWaitConfig,
) -> Result<StaffChangeResponse, PlayerBoostRequestError> {
    if source_request_id.trim().is_empty() || club_uid == 0 {
        return Err(PlayerBoostRequestError::WriteFailed(
            "staff changes need the live scan and the managed club".to_string(),
        ));
    }
    if changes.is_empty() || changes.len() > MAX_STAFF_CHANGES {
        return Err(PlayerBoostRequestError::WriteFailed(format!(
            "staff changes must list 1 through {MAX_STAFF_CHANGES} staff"
        )));
    }
    let _bridge_request_guard = acquire_bridge_request_guard(bridge_directory)
        .map_err(PlayerBoostRequestError::Unavailable)?;
    let status = read_bridge_status(bridge_directory).map_err(PlayerBoostRequestError::from)?;
    if status.staff_changes_supported != Some(true) {
        return Err(PlayerBoostRequestError::Unavailable(
            STAFF_CHANGES_UNAVAILABLE.to_string(),
        ));
    }

    let request_id = new_request_id();
    let request = StaffChangeRequest {
        protocol_version: PROTOCOL_VERSION,
        request_id: request_id.clone(),
        created_at_utc: utc_now_rfc3339(),
        operation: OPERATION_APPLY_STAFF_CHANGES,
        player_database_scope: PLAYER_DATABASE_SCOPE_MEN,
        source_request_id,
        club_uid,
        staff_changes: changes,
    };
    write_bridge_request(bridge_directory, &request)?;
    wait_for_staff_changes_terminal(bridge_directory, &request_id, changes.len(), wait)
}

pub fn wait_for_staff_changes_terminal(
    bridge_directory: &Path,
    request_id: &str,
    change_count: usize,
    wait: DumpWaitConfig,
) -> Result<StaffChangeResponse, PlayerBoostRequestError> {
    let deadline = Instant::now() + wait.timeout;
    loop {
        match read_bridge_status(bridge_directory) {
            Ok(status)
                if status.request_id.as_deref() == Some(request_id)
                    && is_terminal_state(&status.state) =>
            {
                let error = (status.state == "failed").then(|| {
                    status.error.clone().unwrap_or_else(|| {
                        "staff changes failed without a bridge error".to_string()
                    })
                });
                return match status.staff_changes {
                    Some(result) if result.changes.len() == change_count => {
                        Ok(StaffChangeResponse {
                            changes: result.changes,
                            error,
                        })
                    }
                    Some(_) => Err(PlayerBoostRequestError::Corrupt(
                        "the bridge reported a different number of staff changes than were sent"
                            .to_string(),
                    )),
                    None => Err(PlayerBoostRequestError::Failed(error.unwrap_or_else(
                        || {
                            "the bridge finished the staff changes without their outcomes"
                                .to_string()
                        },
                    ))),
                };
            }
            Ok(_) => {}
            Err(BridgeStatusError::Missing(_)) => {}
            Err(error) => return Err(error.into()),
        }

        if Instant::now() >= deadline {
            return Err(PlayerBoostRequestError::Timeout(
                "timed out waiting for FM to finish the staff changes; Load Data to see FM's staff"
                    .to_string(),
            ));
        }

        thread::sleep(wait.poll_interval);
    }
}

/// The board's staff limits for the managed club's first, reserve, and youth teams, as the
/// bridge read them from FM's Staff overview channels.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffLimitsResult {
    #[serde(default)]
    pub teams: Vec<StaffLimitsTeam>,
}

/// One team's staff groups; a team FM sent nothing for has none.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffLimitsTeam {
    /// `senior`, `reserve`, or `youth`.
    pub level: String,
    pub team_type: i32,
    #[serde(default)]
    pub groups: Vec<StaffLimitsGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffLimitsGroup {
    /// `coaching`, `recruitment`, or `medical`.
    pub group: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<i32>,
    pub max: i32,
    #[serde(default)]
    pub jobs: Vec<StaffLimitsJob>,
}

/// The board's limit for one job; `job_type` is FM's job type, as `staff.job_id` holds it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffLimitsJob {
    pub job_type: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<i32>,
    pub max: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StaffLimitsRequest<'a> {
    protocol_version: u32,
    request_id: String,
    created_at_utc: String,
    operation: &'static str,
    player_database_scope: &'static str,
    source_request_id: &'a str,
    club_uid: u32,
}

/// Why staff limits cannot be read when the bridge does not offer them.
pub const STAFF_LIMITS_UNAVAILABLE: &str =
    "the bridge plugin cannot read FM's staff limits; update it from Settings";

/// How long Load Data waits for the bridge to read the staff limits. The read itself ends
/// within about six seconds once FM's main thread runs it.
pub const STAFF_LIMITS_WAIT: DumpWaitConfig = DumpWaitConfig {
    timeout: Duration::from_secs(30),
    poll_interval: Duration::from_millis(100),
};

pub fn request_staff_limits_from_local_app_data(
    source_request_id: &str,
    club_uid: u32,
    wait: DumpWaitConfig,
) -> Result<StaffLimitsResult, PlayerBoostRequestError> {
    let bridge_directory = resolve_bridge_directory().map_err(PlayerBoostRequestError::from)?;
    request_staff_limits(&bridge_directory, source_request_id, club_uid, wait)
}

/// Asks the bridge to read the board's staff limits for the managed club's teams and waits
/// for them. Nothing in FM changes.
pub fn request_staff_limits(
    bridge_directory: &Path,
    source_request_id: &str,
    club_uid: u32,
    wait: DumpWaitConfig,
) -> Result<StaffLimitsResult, PlayerBoostRequestError> {
    if source_request_id.trim().is_empty() || club_uid == 0 {
        return Err(PlayerBoostRequestError::WriteFailed(
            "staff limits need the live scan and the managed club".to_string(),
        ));
    }
    let _bridge_request_guard = acquire_bridge_request_guard(bridge_directory)
        .map_err(PlayerBoostRequestError::Unavailable)?;
    let status = read_bridge_status(bridge_directory).map_err(PlayerBoostRequestError::from)?;
    if status.staff_limits_supported != Some(true) {
        return Err(PlayerBoostRequestError::Unavailable(
            STAFF_LIMITS_UNAVAILABLE.to_string(),
        ));
    }

    let request_id = new_request_id();
    let request = StaffLimitsRequest {
        protocol_version: PROTOCOL_VERSION,
        request_id: request_id.clone(),
        created_at_utc: utc_now_rfc3339(),
        operation: OPERATION_READ_STAFF_LIMITS,
        player_database_scope: PLAYER_DATABASE_SCOPE_MEN,
        source_request_id,
        club_uid,
    };
    write_bridge_request(bridge_directory, &request)?;
    wait_for_staff_limits_terminal(bridge_directory, &request_id, wait)
}

pub fn wait_for_staff_limits_terminal(
    bridge_directory: &Path,
    request_id: &str,
    wait: DumpWaitConfig,
) -> Result<StaffLimitsResult, PlayerBoostRequestError> {
    let deadline = Instant::now() + wait.timeout;
    loop {
        match read_bridge_status(bridge_directory) {
            Ok(status)
                if status.request_id.as_deref() == Some(request_id)
                    && is_terminal_state(&status.state) =>
            {
                if status.state == "failed" {
                    return Err(PlayerBoostRequestError::Failed(
                        status.error.unwrap_or_else(|| {
                            "the staff limit read failed without a bridge error".to_string()
                        }),
                    ));
                }
                return status.staff_limits.ok_or_else(|| {
                    PlayerBoostRequestError::Corrupt(
                        "the bridge finished the staff limit read without the limits".to_string(),
                    )
                });
            }
            Ok(_) => {}
            Err(BridgeStatusError::Missing(_)) => {}
            Err(error) => return Err(error.into()),
        }

        if Instant::now() >= deadline {
            return Err(PlayerBoostRequestError::Timeout(
                "timed out waiting for FM to read the staff limits".to_string(),
            ));
        }

        thread::sleep(wait.poll_interval);
    }
}

/// Each requested training change's outcome, in request order, with the training FM holds for
/// the player afterwards. It carries no player identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingResult {
    #[serde(default)]
    pub players: Vec<TrainingOutcome>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingOutcome {
    /// `applied`, `not-found`, `not-at-club`, or `not-confirmed`.
    pub outcome: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_position: Option<i32>,
    /// FM's in-possession role value; 0 trains the position itself.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_role: Option<u64>,
    /// FM's name for the in-possession training, such as "Half Back".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_role_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oop_role: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oop_role_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_name: Option<String>,
}

/// One player's training in FM27's own IDs. Each part is optional; a change sets at least one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingChangeItem {
    pub player_uid: u32,
    /// FM's position mask for the in-possession role, such as 16 for D (C).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_position: Option<i32>,
    /// FM's in-possession role value: one bit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_role: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oop_role: Option<i32>,
    /// FM's additional focus, or -1 to clear it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus: Option<i32>,
}

/// The bridge's answer to a training request: each change's outcome, and the error that stopped
/// the request early when one did. FM may have taken some changes before such an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrainingResponse {
    pub players: Vec<TrainingOutcome>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct TrainingRequest<'a> {
    protocol_version: u32,
    request_id: String,
    created_at_utc: String,
    operation: &'static str,
    player_database_scope: &'static str,
    source_request_id: &'a str,
    club_uid: u32,
    training_changes: &'a [TrainingChangeItem],
}

/// The most players one training request may change, enough for a whole club with all its
/// teams; the bridge enforces the same bound.
pub const MAX_TRAINING_CHANGES: usize = 250;

/// Why training cannot be set when the bridge's live scan does not offer it.
pub const TRAINING_NEEDS_LOAD_DATA: &str =
    "Load Data with FM open on this save before setting training";

/// Why training cannot be set when the bridge plugin predates it.
pub const TRAINING_NEEDS_PLUGIN_UPDATE: &str =
    "Update the bridge plugin from Settings, restart FM and Load Data to set training in FM";

/// None when the bridge offers training changes for its live scan; otherwise why not.
pub fn training_unavailable_reason(status: &BridgeStatus) -> Option<&'static str> {
    match status.training_supported {
        Some(true) => None,
        Some(false) => Some(TRAINING_NEEDS_LOAD_DATA),
        None => Some(TRAINING_NEEDS_PLUGIN_UPDATE),
    }
}

pub fn request_training_from_local_app_data(
    source_request_id: &str,
    club_uid: u32,
    changes: &[TrainingChangeItem],
    wait: DumpWaitConfig,
) -> Result<TrainingResponse, PlayerBoostRequestError> {
    let bridge_directory = resolve_bridge_directory().map_err(PlayerBoostRequestError::from)?;
    request_training(
        &bridge_directory,
        source_request_id,
        club_uid,
        changes,
        wait,
    )
}

/// Asks the bridge to set the managed club's players' training and waits for each change's
/// outcome.
pub fn request_training(
    bridge_directory: &Path,
    source_request_id: &str,
    club_uid: u32,
    changes: &[TrainingChangeItem],
    wait: DumpWaitConfig,
) -> Result<TrainingResponse, PlayerBoostRequestError> {
    if source_request_id.trim().is_empty() || club_uid == 0 {
        return Err(PlayerBoostRequestError::WriteFailed(
            "training changes need the live scan and the managed club".to_string(),
        ));
    }
    if changes.is_empty() || changes.len() > MAX_TRAINING_CHANGES {
        return Err(PlayerBoostRequestError::WriteFailed(format!(
            "training changes must list 1 through {MAX_TRAINING_CHANGES} players"
        )));
    }
    let _bridge_request_guard = acquire_bridge_request_guard(bridge_directory)
        .map_err(PlayerBoostRequestError::Unavailable)?;
    let status = read_bridge_status(bridge_directory).map_err(PlayerBoostRequestError::from)?;
    if let Some(reason) = training_unavailable_reason(&status) {
        return Err(PlayerBoostRequestError::Unavailable(reason.to_string()));
    }

    let request_id = new_request_id();
    let request = TrainingRequest {
        protocol_version: PROTOCOL_VERSION,
        request_id: request_id.clone(),
        created_at_utc: utc_now_rfc3339(),
        operation: OPERATION_APPLY_TRAINING,
        player_database_scope: PLAYER_DATABASE_SCOPE_MEN,
        source_request_id,
        club_uid,
        training_changes: changes,
    };
    write_bridge_request(bridge_directory, &request)?;
    wait_for_training_terminal(bridge_directory, &request_id, changes.len(), wait)
}

pub fn wait_for_training_terminal(
    bridge_directory: &Path,
    request_id: &str,
    change_count: usize,
    wait: DumpWaitConfig,
) -> Result<TrainingResponse, PlayerBoostRequestError> {
    let deadline = Instant::now() + wait.timeout;
    loop {
        match read_bridge_status(bridge_directory) {
            Ok(status)
                if status.request_id.as_deref() == Some(request_id)
                    && is_terminal_state(&status.state) =>
            {
                let error = (status.state == "failed").then(|| {
                    status.error.clone().unwrap_or_else(|| {
                        "the training changes failed without a bridge error".to_string()
                    })
                });
                return match status.training {
                    Some(result) if result.players.len() == change_count => Ok(TrainingResponse {
                        players: result.players,
                        error,
                    }),
                    Some(_) => Err(PlayerBoostRequestError::Corrupt(
                        "the bridge reported a different number of training changes than were sent"
                            .to_string(),
                    )),
                    None => Err(PlayerBoostRequestError::Failed(error.unwrap_or_else(
                        || {
                            "the bridge finished the training changes without their outcomes"
                                .to_string()
                        },
                    ))),
                };
            }
            Ok(_) => {}
            Err(BridgeStatusError::Missing(_)) => {}
            Err(error) => return Err(error.into()),
        }

        if Instant::now() >= deadline {
            return Err(PlayerBoostRequestError::Timeout(
                "timed out waiting for FM to set the training; check it in FM".to_string(),
            ));
        }

        thread::sleep(wait.poll_interval);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    const INGESTIBLE_DUMP_FIXTURE: &str = include_str!("fixtures/golden_dump_v11.json");

    const HAPPY_STATUS_JSON: &str = r#"{
  "protocolVersion": 1,
  "pluginVersion": "0.1.0",
  "state": "idle",
  "updatedAtUtc": "2026-07-28T15:00:00+00:00",
  "gamePluginModulePresent": true,
  "gameAssemblyModulePresent": false
}"#;

    #[test]
    fn bridge_directory_joins_local_app_data_fm_valuescout_fm_bridge() {
        let local = PathBuf::from("local-app-data");
        let bridge = bridge_directory_from_local_app_data(&local);
        assert_eq!(
            bridge,
            PathBuf::from("local-app-data")
                .join("fm-valuescout")
                .join("fm-bridge")
        );
    }

    #[test]
    fn parse_idle_status_includes_versioned_contract_fields() {
        let status = parse_bridge_status(HAPPY_STATUS_JSON).expect("parse status");

        assert_eq!(status.protocol_version, 1);
        assert_eq!(status.plugin_version, "0.1.0");
        assert_eq!(status.state, "idle");
        assert_eq!(status.updated_at_utc, "2026-07-28T15:00:00+00:00");
        assert!(status.game_plugin_module_present);
        assert!(!status.game_assembly_module_present);
        assert_eq!(status.request_id, None);
    }

    #[test]
    fn read_returns_status_when_fixture_file_exists() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bridge_dir = temp_dir.path();
        fs::write(status_path(bridge_dir), HAPPY_STATUS_JSON).expect("write fixture");

        let status = read_bridge_status(bridge_dir).expect("read status");

        assert_eq!(status.state, "idle");
        assert_eq!(status.plugin_version, "0.1.0");
        assert!(status.game_plugin_module_present);
    }

    #[test]
    fn read_returns_missing_when_status_file_absent() {
        let temp_dir = tempfile::tempdir().expect("temp dir");

        let error = read_bridge_status(temp_dir.path()).expect_err("missing status");

        assert!(matches!(error, BridgeStatusError::Missing(_)));
    }

    #[test]
    fn read_returns_corrupt_when_status_file_is_not_utf8() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bridge_dir = temp_dir.path();
        fs::write(status_path(bridge_dir), [0xff, 0xfe, 0xfd]).expect("write binary fixture");

        let error = read_bridge_status(bridge_dir).expect_err("corrupt status");

        assert!(matches!(error, BridgeStatusError::Corrupt(_)));
    }

    #[test]
    fn parse_returns_corrupt_for_empty_payload() {
        let error = parse_bridge_status("").expect_err("empty");
        assert!(matches!(error, BridgeStatusError::Corrupt(_)));
    }

    #[test]
    fn parse_returns_corrupt_for_invalid_json() {
        let error = parse_bridge_status("{not-json").expect_err("corrupt");
        assert!(matches!(error, BridgeStatusError::Corrupt(_)));
    }

    #[test]
    fn parse_returns_unsupported_version_for_unknown_protocol() {
        let json = r#"{
  "protocolVersion": 99,
  "pluginVersion": "0.1.0",
  "state": "idle",
  "updatedAtUtc": "2026-07-28T15:00:00+00:00",
  "gamePluginModulePresent": false,
  "gameAssemblyModulePresent": false
}"#;

        let error = parse_bridge_status(json).expect_err("unsupported version");

        assert!(matches!(error, BridgeStatusError::UnsupportedVersion(_)));
    }

    #[test]
    fn write_request_uses_versioned_full_dump_shape() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bridge_dir = temp_dir.path();
        let request = BridgeRequest {
            protocol_version: 1,
            request_id: "req-test".to_string(),
            created_at_utc: "2026-07-28T18:30:00.000Z".to_string(),
            operation: OPERATION_FULL_DUMP.to_string(),
            player_database_scope: PLAYER_DATABASE_SCOPE_MEN.to_string(),
        };

        write_player_dump_request(bridge_dir, &request).expect("write request");

        let json = fs::read_to_string(request_path(bridge_dir)).expect("read request");
        let parsed: BridgeRequest = serde_json::from_str(&json).expect("parse request");
        assert_eq!(parsed.request_id, "req-test");
        assert_eq!(parsed.operation, "full-dump");
        assert_eq!(parsed.protocol_version, 1);
        assert_eq!(parsed.player_database_scope, PLAYER_DATABASE_SCOPE_MEN);
        assert!(
            json.contains("\"playerDatabaseScope\": \"men\""),
            "production request must default playerDatabaseScope to men, got: {json}"
        );
        let wire: serde_json::Value = serde_json::from_str(&json).expect("parse request JSON");
        assert!(
            wire.get("maxAccepted").is_none(),
            "full-dump request must omit maxAccepted, got: {json}"
        );
    }

    #[test]
    fn request_player_dump_defaults_to_unlimited_max_accepted() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bridge_dir = temp_dir.path().to_path_buf();
        let barrier = Arc::new(Barrier::new(2));

        let writer_dir = bridge_dir.clone();
        let writer_barrier = Arc::clone(&barrier);
        thread::spawn(move || {
            writer_barrier.wait();
            loop {
                if request_path(&writer_dir).is_file() {
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
            let json = fs::read_to_string(request_path(&writer_dir)).expect("read request");
            let request: BridgeRequest = serde_json::from_str(&json).expect("parse request");
            assert_eq!(request.player_database_scope, PLAYER_DATABASE_SCOPE_MEN);
            let wire: serde_json::Value = serde_json::from_str(&json).expect("parse request JSON");
            assert!(
                wire.get("maxAccepted").is_none(),
                "production request must omit maxAccepted, got: {json}"
            );
            write_status_fixture(
                &writer_dir,
                "ready",
                Some(&request.request_id),
                Some(1),
                None,
            );
            fs::write(dump_path(&writer_dir), INGESTIBLE_DUMP_FIXTURE).expect("dump");
        });

        barrier.wait();
        let result = request_player_dump(
            &bridge_dir,
            DumpWaitConfig {
                timeout: Duration::from_secs(2),
                poll_interval: Duration::from_millis(20),
            },
        )
        .expect("request dump");
        assert_eq!(result.state, "ready");
    }

    #[test]
    fn wait_returns_ready_when_terminal_dump_is_present_even_if_content_is_invalid() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bridge_dir = temp_dir.path().to_path_buf();
        let request_id = "req-watch-ready".to_string();
        let barrier = Arc::new(Barrier::new(2));

        let writer_dir = bridge_dir.clone();
        let writer_barrier = Arc::clone(&barrier);
        let writer_id = request_id.clone();
        thread::spawn(move || {
            writer_barrier.wait();
            // Simulate bridge: see request, scan, then ready + dump.
            loop {
                if request_path(&writer_dir).is_file() {
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
            write_status_fixture(&writer_dir, "scanning", Some(&writer_id), None, None);
            thread::sleep(Duration::from_millis(30));
            fs::write(dump_path(&writer_dir), "{\"schemaVersion\":99}").expect("dump");
            // The ready status in an older shape, with fields this reader no longer knows.
            let ready = status_fixture("ready", Some(&writer_id), Some(42), None);
            let mut old_status = serde_json::to_value(&ready).expect("status JSON");
            old_status["scanTruncated"] = serde_json::Value::Bool(false);
            old_status["maxAccepted"] = serde_json::Value::from(500);
            write_status_json(&writer_dir, &old_status.to_string());
        });

        barrier.wait();
        let request = BridgeRequest {
            protocol_version: 1,
            request_id: request_id.clone(),
            created_at_utc: "2026-07-28T18:30:00.000Z".to_string(),
            operation: OPERATION_FULL_DUMP.to_string(),
            player_database_scope: PLAYER_DATABASE_SCOPE_MEN.to_string(),
        };
        write_player_dump_request(&bridge_dir, &request).expect("write");

        let result = wait_for_request_terminal(
            &bridge_dir,
            &request_id,
            DumpWaitConfig {
                timeout: Duration::from_secs(2),
                poll_interval: Duration::from_millis(20),
            },
        )
        .expect("wait ready");

        assert_eq!(result.request_id, request_id);
        assert_eq!(result.state, "ready");
        assert_eq!(result.players_found, Some(42));
        assert!(result.dump_present);
        assert!(result.error.is_none());
    }

    #[test]
    fn wait_returns_failed_status_without_requiring_dump() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bridge_dir = temp_dir.path().to_path_buf();
        let request_id = "req-watch-failed";

        write_status_fixture(
            &bridge_dir,
            "failed",
            Some(request_id),
            None,
            Some("scan produced zero player candidates"),
        );

        let result = wait_for_request_terminal(
            &bridge_dir,
            request_id,
            DumpWaitConfig {
                timeout: Duration::from_millis(200),
                poll_interval: Duration::from_millis(20),
            },
        )
        .expect("wait failed");

        assert_eq!(result.state, "failed");
        assert!(!result.dump_present);
        assert!(result
            .error
            .as_deref()
            .unwrap_or("")
            .contains("zero player"));
    }

    #[test]
    fn wait_times_out_when_status_never_reaches_terminal() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bridge_dir = temp_dir.path();
        write_status_fixture(bridge_dir, "idle", None, None, None);

        let error = wait_for_request_terminal(
            bridge_dir,
            "req-never",
            DumpWaitConfig {
                timeout: Duration::from_millis(80),
                poll_interval: Duration::from_millis(20),
            },
        )
        .expect_err("timeout");

        assert!(matches!(error, DumpRequestError::Timeout(_)));
    }

    #[test]
    fn wait_ignores_terminal_status_for_other_request_ids() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bridge_dir = temp_dir.path().to_path_buf();
        let ours = "req-ours".to_string();
        let barrier = Arc::new(Barrier::new(2));

        write_status_fixture(&bridge_dir, "ready", Some("req-stale-other"), Some(9), None);

        let writer_dir = bridge_dir.clone();
        let writer_barrier = Arc::clone(&barrier);
        let writer_id = ours.clone();
        thread::spawn(move || {
            writer_barrier.wait();
            thread::sleep(Duration::from_millis(40));
            fs::write(dump_path(&writer_dir), INGESTIBLE_DUMP_FIXTURE).expect("dump");
            write_status_fixture(&writer_dir, "ready", Some(&writer_id), Some(3), None);
        });

        barrier.wait();
        let result = wait_for_request_terminal(
            &bridge_dir,
            &ours,
            DumpWaitConfig {
                timeout: Duration::from_secs(2),
                poll_interval: Duration::from_millis(20),
            },
        )
        .expect("wait ours");

        assert_eq!(result.request_id, "req-ours");
        assert_eq!(result.players_found, Some(3));
    }

    #[test]
    fn player_boost_request_accepts_current_ability_increments_from_one_through_twenty() {
        let validate = |increment| {
            super::validate_player_boost_request(
                "scan-1",
                77,
                99,
                172,
                &PlayerBoostOperation::CurrentAbility { increment },
            )
        };

        for increment in [1, 20] {
            validate(increment).expect("increment within 1 through 20");
        }
        for increment in [0, 21] {
            assert!(
                validate(increment).is_err(),
                "increment {increment} must be rejected"
            );
        }
    }

    #[test]
    fn player_boost_request_serializes_only_the_closed_ca_operation_fields() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let request = PlayerBoostRequest::from_operation(
            "boost-ca-1".to_string(),
            "scan-1".to_string(),
            77,
            99,
            172,
            PlayerBoostOperation::CurrentAbility { increment: 5 },
        );

        write_player_boost_request(temp_dir.path(), &request).expect("write boost request");

        let json = fs::read_to_string(request_path(temp_dir.path())).expect("read boost request");
        let value: serde_json::Value = serde_json::from_str(&json).expect("parse boost request");
        assert_eq!(value["protocolVersion"], 1);
        assert_eq!(value["operation"], OPERATION_BOOST_CURRENT_ABILITY);
        assert_eq!(value["sourceRequestId"], "scan-1");
        assert_eq!(value["playerUid"], 77);
        assert_eq!(value["expectedCurrentAbility"], 99);
        assert_eq!(value["expectedPotentialAbility"], 172);
        assert_eq!(value["currentAbilityIncrement"], 5);
        assert_eq!(value["playerDatabaseScope"], PLAYER_DATABASE_SCOPE_MEN);
        assert!(value.get("expectedAmbition").is_none());
        assert!(value.get("expectedProfessionalism").is_none());
        assert!(value.get("expectedDetermination").is_none());
    }

    #[test]
    fn player_boost_request_submits_null_mentality_expectations_as_unchanged_fields() {
        let request = PlayerBoostRequest::from_operation(
            "boost-mentality-1".to_string(),
            "scan-1".to_string(),
            77,
            99,
            172,
            PlayerBoostOperation::WonderkidMentality {
                expected_ambition: Some(10),
                expected_professionalism: Some(12),
                expected_determination: None,
            },
        );
        let json = serde_json::to_value(request).expect("serialize boost request");

        assert_eq!(json["operation"], OPERATION_WONDERKID_MENTALITY);
        assert_eq!(json["expectedAmbition"], 10);
        assert_eq!(json["expectedProfessionalism"], 12);
        assert!(json.get("expectedDetermination").is_none());
        assert!(json.get("currentAbilityIncrement").is_none());
    }

    #[test]
    fn staff_boost_request_serializes_uid_and_expected_values_without_an_increment() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let request = StaffBoostRequest {
            protocol_version: PROTOCOL_VERSION,
            request_id: "staff-boost-1".to_string(),
            created_at_utc: "2026-08-16T10:00:00.000Z".to_string(),
            operation: OPERATION_BOOST_STAFF_CURRENT_ABILITY.to_string(),
            player_database_scope: PLAYER_DATABASE_SCOPE_MEN.to_string(),
            source_request_id: "scan-1".to_string(),
            staff_uid: 88,
            expected_current_ability: 115,
            expected_potential_ability: 140,
        };
        write_staff_boost_request(temp_dir.path(), &request).expect("write staff boost request");

        let value: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(request_path(temp_dir.path())).expect("read request"),
        )
        .expect("parse request");
        assert_eq!(value["operation"], OPERATION_BOOST_STAFF_CURRENT_ABILITY);
        assert_eq!(value["staffUid"], 88);
        assert_eq!(value["expectedCurrentAbility"], 115);
        assert_eq!(value["expectedPotentialAbility"], 140);
        assert!(value.get("playerUid").is_none());
        assert!(value.get("currentAbilityIncrement").is_none());
    }

    #[test]
    fn staff_boost_requires_advertised_support_before_writing_a_request() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        write_staff_boost_status_fixture(
            temp_dir.path(),
            "ready",
            Some("scan-1"),
            None,
            false,
            None,
        );

        let error = request_staff_boost(
            temp_dir.path(),
            "scan-1",
            88,
            115,
            140,
            DumpWaitConfig {
                timeout: Duration::from_millis(100),
                poll_interval: Duration::from_millis(10),
            },
        )
        .expect_err("bridge without staff boost support must be rejected");
        assert!(matches!(error, PlayerBoostRequestError::Unavailable(_)));
        assert!(!request_path(temp_dir.path()).exists());
    }

    #[test]
    fn staff_boost_accepts_only_a_verified_terminal_result() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        write_staff_boost_status_fixture(
            temp_dir.path(),
            "ready",
            Some("staff-boost-1"),
            None,
            true,
            Some(StaffBoostResult {
                operation: OPERATION_BOOST_STAFF_CURRENT_ABILITY.to_string(),
                outcome: "verified".to_string(),
                rollback: "not-needed".to_string(),
                previous_current_ability: Some(115),
                current_ability: Some(125),
                potential_ability: Some(140),
            }),
        );

        let result = wait_for_staff_boost_terminal(
            temp_dir.path(),
            "staff-boost-1",
            DumpWaitConfig {
                timeout: Duration::from_millis(100),
                poll_interval: Duration::from_millis(10),
            },
        )
        .expect("verified staff boost");
        assert_eq!(result.current_ability, Some(125));
    }

    fn write_squad_move_status_fixture(
        bridge_dir: &Path,
        state: &str,
        request_id: &str,
        supported: bool,
        squad_moves: Option<SquadMoveResult>,
        error: Option<&str>,
    ) {
        let status = BridgeStatus {
            protocol_version: PROTOCOL_VERSION,
            plugin_version: "0.31.1".to_string(),
            state: state.to_string(),
            updated_at_utc: "2026-11-20T12:00:00.000Z".to_string(),
            game_plugin_module_present: true,
            game_assembly_module_present: true,
            request_id: Some(request_id.to_string()),
            players_found: None,
            error: error.map(str::to_string),
            player_boosts_supported: Some(supported),
            player_boost: None,
            staff_boosts_supported: None,
            staff_boost: None,
            squad_moves_supported: Some(supported),
            squad_moves,
            staff_changes_supported: None,
            staff_changes: None,
            staff_limits_supported: None,
            staff_limits: None,
            training_supported: None,
            training: None,
        };
        write_status(bridge_dir, &status);
    }

    fn outcome(outcome: &str, team_type: Option<i32>) -> SquadMoveOutcome {
        SquadMoveOutcome {
            outcome: outcome.to_string(),
            team_type,
            team_level: team_type.map(|_| "reserve".to_string()),
        }
    }

    const QUICK_WAIT: DumpWaitConfig = DumpWaitConfig {
        timeout: Duration::from_millis(100),
        poll_interval: Duration::from_millis(10),
    };

    #[test]
    fn squad_moves_require_advertised_support_before_writing_a_request() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        write_squad_move_status_fixture(temp_dir.path(), "ready", "scan-1", false, None, None);

        let error = request_squad_moves(
            temp_dir.path(),
            "scan-1",
            439,
            &[SquadMoveItem {
                player_uid: 77,
                target_level: "reserve",
            }],
            QUICK_WAIT,
        )
        .expect_err("a bridge without squad moves must be rejected");

        assert_eq!(
            error,
            PlayerBoostRequestError::Unavailable(SQUAD_MOVES_UNAVAILABLE.to_string())
        );
        assert!(!request_path(temp_dir.path()).exists());
    }

    #[test]
    fn squad_move_request_names_the_club_and_each_move() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let moves = [
            SquadMoveItem {
                player_uid: 77,
                target_level: "reserve",
            },
            SquadMoveItem {
                player_uid: 78,
                target_level: "youth",
            },
        ];
        write_bridge_request(
            temp_dir.path(),
            &SquadMoveRequest {
                protocol_version: PROTOCOL_VERSION,
                request_id: "move-1".to_string(),
                created_at_utc: "2026-11-20T12:00:00.000Z".to_string(),
                operation: OPERATION_MOVE_SQUAD_PLAYERS,
                player_database_scope: PLAYER_DATABASE_SCOPE_MEN,
                source_request_id: "scan-1",
                club_uid: 439,
                squad_moves: &moves,
            },
        )
        .expect("write squad move request");

        let value: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(request_path(temp_dir.path())).expect("read request"),
        )
        .expect("parse request");
        assert_eq!(value["operation"], OPERATION_MOVE_SQUAD_PLAYERS);
        assert_eq!(value["sourceRequestId"], "scan-1");
        assert_eq!(value["clubUid"], 439);
        assert_eq!(
            value["squadMoves"],
            serde_json::json!([
                { "playerUid": 77, "targetLevel": "reserve" },
                { "playerUid": 78, "targetLevel": "youth" }
            ])
        );
    }

    #[test]
    fn squad_move_wait_keeps_the_outcomes_of_a_request_that_stopped_early() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        write_squad_move_status_fixture(
            temp_dir.path(),
            "failed",
            "move-1",
            true,
            Some(SquadMoveResult {
                moves: vec![
                    outcome("moved", Some(10)),
                    outcome("not-confirmed", Some(0)),
                ],
            }),
            Some("FM rejected a squad move: boom"),
        );

        let response = wait_for_squad_moves_terminal(temp_dir.path(), "move-1", 2, QUICK_WAIT)
            .expect("outcomes of a partial request");

        assert_eq!(response.moves[0], outcome("moved", Some(10)));
        assert_eq!(
            response.error.as_deref(),
            Some("FM rejected a squad move: boom")
        );
    }

    fn write_training_status_fixture(
        bridge_dir: &Path,
        state: &str,
        request_id: &str,
        supported: Option<bool>,
        training: Option<TrainingResult>,
        error: Option<&str>,
    ) {
        let status = BridgeStatus {
            protocol_version: PROTOCOL_VERSION,
            plugin_version: "0.32.0".to_string(),
            state: state.to_string(),
            updated_at_utc: "2026-11-20T12:00:00.000Z".to_string(),
            game_plugin_module_present: true,
            game_assembly_module_present: true,
            request_id: Some(request_id.to_string()),
            players_found: None,
            error: error.map(str::to_string),
            player_boosts_supported: Some(true),
            player_boost: None,
            staff_boosts_supported: None,
            staff_boost: None,
            squad_moves_supported: Some(true),
            squad_moves: None,
            staff_changes_supported: None,
            staff_changes: None,
            staff_limits_supported: None,
            staff_limits: None,
            training_supported: supported,
            training,
        };
        write_status(bridge_dir, &status);
    }

    #[test]
    fn training_names_the_missing_step_before_writing_a_request() {
        for (supported, reason) in [
            (Some(false), TRAINING_NEEDS_LOAD_DATA),
            (None, TRAINING_NEEDS_PLUGIN_UPDATE),
        ] {
            let temp_dir = tempfile::tempdir().expect("temp dir");
            write_training_status_fixture(
                temp_dir.path(),
                "ready",
                "scan-1",
                supported,
                None,
                None,
            );

            let error = request_training(
                temp_dir.path(),
                "scan-1",
                439,
                &[TrainingChangeItem {
                    player_uid: 77,
                    ip_position: None,
                    ip_role: None,
                    oop_role: None,
                    focus: Some(19),
                }],
                QUICK_WAIT,
            )
            .expect_err("a bridge without training must be rejected");

            assert_eq!(
                error,
                PlayerBoostRequestError::Unavailable(reason.to_string())
            );
            assert!(!request_path(temp_dir.path()).exists());
        }
    }

    #[test]
    fn training_request_sends_only_the_parts_each_change_sets() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let changes = [
            TrainingChangeItem {
                player_uid: 77,
                ip_position: Some(1024),
                ip_role: Some(144_115_188_075_855_872),
                oop_role: Some(-14),
                focus: Some(19),
            },
            TrainingChangeItem {
                player_uid: 78,
                ip_position: None,
                ip_role: None,
                oop_role: None,
                focus: Some(-1),
            },
        ];
        write_bridge_request(
            temp_dir.path(),
            &TrainingRequest {
                protocol_version: PROTOCOL_VERSION,
                request_id: "training-1".to_string(),
                created_at_utc: "2026-11-20T12:00:00.000Z".to_string(),
                operation: OPERATION_APPLY_TRAINING,
                player_database_scope: PLAYER_DATABASE_SCOPE_MEN,
                source_request_id: "scan-1",
                club_uid: 439,
                training_changes: &changes,
            },
        )
        .expect("write training request");

        let value: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(request_path(temp_dir.path())).expect("read request"),
        )
        .expect("parse request");
        assert_eq!(value["operation"], "apply-training");
        assert_eq!(value["clubUid"], 439);
        assert_eq!(
            value["trainingChanges"],
            serde_json::json!([
                { "playerUid": 77, "ipPosition": 1024, "ipRole": 144_115_188_075_855_872_u64, "oopRole": -14, "focus": 19 },
                { "playerUid": 78, "focus": -1 }
            ])
        );
    }

    #[test]
    fn training_wait_keeps_the_outcomes_of_a_request_that_stopped_early() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let applied = TrainingOutcome {
            outcome: "applied".to_string(),
            ip_role: Some(144_115_188_075_855_872),
            ip_role_name: Some("Midfield Playmaker".to_string()),
            ..TrainingOutcome::default()
        };
        write_training_status_fixture(
            temp_dir.path(),
            "failed",
            "training-1",
            Some(true),
            Some(TrainingResult {
                players: vec![applied.clone()],
            }),
            Some("FM rejected an action: boom"),
        );

        let response = wait_for_training_terminal(temp_dir.path(), "training-1", 1, QUICK_WAIT)
            .expect("outcomes of a partial request");

        assert_eq!(response.players, vec![applied]);
        assert_eq!(
            response.error.as_deref(),
            Some("FM rejected an action: boom")
        );
        assert!(matches!(
            wait_for_training_terminal(temp_dir.path(), "training-1", 2, QUICK_WAIT),
            Err(PlayerBoostRequestError::Corrupt(_))
        ));
    }

    fn write_staff_change_status_fixture(
        bridge_dir: &Path,
        state: &str,
        request_id: &str,
        supported: bool,
        staff_changes: Option<StaffChangeResult>,
        error: Option<&str>,
    ) {
        let status = BridgeStatus {
            protocol_version: PROTOCOL_VERSION,
            plugin_version: "0.31.1".to_string(),
            state: state.to_string(),
            updated_at_utc: "2026-11-20T12:00:00.000Z".to_string(),
            game_plugin_module_present: true,
            game_assembly_module_present: true,
            request_id: Some(request_id.to_string()),
            players_found: None,
            error: error.map(str::to_string),
            player_boosts_supported: Some(supported),
            player_boost: None,
            staff_boosts_supported: Some(supported),
            staff_boost: None,
            squad_moves_supported: Some(supported),
            squad_moves: None,
            staff_changes_supported: Some(supported),
            staff_changes,
            staff_limits_supported: None,
            staff_limits: None,
            training_supported: None,
            training: None,
        };
        write_status(bridge_dir, &status);
    }

    fn write_staff_limits_status_fixture(
        bridge_dir: &Path,
        state: &str,
        request_id: &str,
        supported: bool,
        staff_limits: Option<StaffLimitsResult>,
        error: Option<&str>,
    ) {
        let mut status = parse_bridge_status(HAPPY_STATUS_JSON).expect("parse status");
        status.state = state.to_string();
        status.request_id = Some(request_id.to_string());
        status.error = error.map(str::to_string);
        status.staff_limits_supported = Some(supported);
        status.staff_limits = staff_limits;
        write_status(bridge_dir, &status);
    }

    #[test]
    fn staff_limits_require_advertised_support_before_writing_a_request() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        write_staff_limits_status_fixture(temp_dir.path(), "ready", "scan-1", false, None, None);

        let error = request_staff_limits(temp_dir.path(), "scan-1", 680, QUICK_WAIT)
            .expect_err("a bridge without staff limits must be rejected");

        assert_eq!(
            error,
            PlayerBoostRequestError::Unavailable(STAFF_LIMITS_UNAVAILABLE.to_string())
        );
        assert!(!request_path(temp_dir.path()).exists());
    }

    #[test]
    fn staff_limit_wait_returns_the_limits_or_the_bridge_error() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let limits: StaffLimitsResult = serde_json::from_value(serde_json::json!({
            "teams": [{
                "level": "senior",
                "teamType": 0,
                "groups": [{
                    "group": "coaching",
                    "name": "Coaching Team",
                    "count": 28,
                    "max": 28,
                    "jobs": [{ "jobType": 2, "name": "Coaches", "count": 13, "max": 13 }]
                }]
            }]
        }))
        .expect("parse limits");
        write_staff_limits_status_fixture(
            temp_dir.path(),
            "ready",
            "limits-1",
            true,
            Some(limits.clone()),
            None,
        );
        assert_eq!(
            wait_for_staff_limits_terminal(temp_dir.path(), "limits-1", QUICK_WAIT),
            Ok(limits)
        );

        write_staff_limits_status_fixture(
            temp_dir.path(),
            "failed",
            "limits-2",
            true,
            None,
            Some("FM sent no staff limits for the first team"),
        );
        assert_eq!(
            wait_for_staff_limits_terminal(temp_dir.path(), "limits-2", QUICK_WAIT),
            Err(PlayerBoostRequestError::Failed(
                "FM sent no staff limits for the first team".to_string()
            ))
        );
    }

    #[test]
    fn staff_changes_require_advertised_support_before_writing_a_request() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        write_staff_change_status_fixture(temp_dir.path(), "ready", "scan-1", false, None, None);

        let error = request_staff_changes(
            temp_dir.path(),
            "scan-1",
            680,
            &[StaffChangeItem {
                staff_uid: 41,
                kind: "sack",
                job_type: None,
                target_level: None,
            }],
            QUICK_WAIT,
        )
        .expect_err("a bridge without staff changes must be rejected");

        assert_eq!(
            error,
            PlayerBoostRequestError::Unavailable(STAFF_CHANGES_UNAVAILABLE.to_string())
        );
        assert!(!request_path(temp_dir.path()).exists());
    }

    #[test]
    fn staff_change_request_names_the_club_and_each_change() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let changes = [
            StaffChangeItem {
                staff_uid: 41,
                kind: "sack",
                job_type: None,
                target_level: None,
            },
            StaffChangeItem {
                staff_uid: 42,
                kind: "change",
                job_type: Some(2),
                target_level: Some("youth"),
            },
        ];
        write_bridge_request(
            temp_dir.path(),
            &StaffChangeRequest {
                protocol_version: PROTOCOL_VERSION,
                request_id: "staff-1".to_string(),
                created_at_utc: "2026-11-20T12:00:00.000Z".to_string(),
                operation: OPERATION_APPLY_STAFF_CHANGES,
                player_database_scope: PLAYER_DATABASE_SCOPE_MEN,
                source_request_id: "scan-1",
                club_uid: 680,
                staff_changes: &changes,
            },
        )
        .expect("write staff change request");

        let value: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(request_path(temp_dir.path())).expect("read request"),
        )
        .expect("parse request");
        assert_eq!(value["operation"], OPERATION_APPLY_STAFF_CHANGES);
        assert_eq!(value["clubUid"], 680);
        assert_eq!(
            value["staffChanges"],
            serde_json::json!([
                { "staffUid": 41, "kind": "sack" },
                { "staffUid": 42, "kind": "change", "jobType": 2, "targetLevel": "youth" }
            ])
        );
    }

    #[test]
    fn staff_change_wait_keeps_outcomes_and_rejects_a_mismatched_count() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let hired = StaffChangeOutcome {
            outcome: "hired".to_string(),
            club_uid: Some(680),
            job_id: Some(2),
            weekly_wage_gbp: Some(6388),
            team_type: Some(10),
            team_level: Some("reserve".to_string()),
            ..StaffChangeOutcome::default()
        };
        write_staff_change_status_fixture(
            temp_dir.path(),
            "failed",
            "staff-1",
            true,
            Some(StaffChangeResult {
                changes: vec![hired.clone()],
            }),
            Some("FM rejected an action: boom"),
        );

        let response = wait_for_staff_changes_terminal(temp_dir.path(), "staff-1", 1, QUICK_WAIT)
            .expect("outcomes of a partial request");
        assert_eq!(response.changes, vec![hired]);
        assert_eq!(
            response.error.as_deref(),
            Some("FM rejected an action: boom")
        );
        assert!(matches!(
            wait_for_staff_changes_terminal(temp_dir.path(), "staff-1", 2, QUICK_WAIT),
            Err(PlayerBoostRequestError::Corrupt(_))
        ));
    }

    #[test]
    fn squad_move_wait_rejects_missing_or_mismatched_outcomes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        write_squad_move_status_fixture(
            temp_dir.path(),
            "ready",
            "move-1",
            true,
            Some(SquadMoveResult {
                moves: vec![outcome("moved", Some(10))],
            }),
            None,
        );
        assert!(matches!(
            wait_for_squad_moves_terminal(temp_dir.path(), "move-1", 2, QUICK_WAIT),
            Err(PlayerBoostRequestError::Corrupt(_))
        ));

        write_squad_move_status_fixture(
            temp_dir.path(),
            "failed",
            "move-2",
            true,
            None,
            Some("the managed club was not found in FM; Load Data again"),
        );
        assert_eq!(
            wait_for_squad_moves_terminal(temp_dir.path(), "move-2", 1, QUICK_WAIT),
            Err(PlayerBoostRequestError::Failed(
                "the managed club was not found in FM; Load Data again".to_string()
            ))
        );
    }

    #[test]
    fn player_boost_requires_advertised_support_before_writing_a_request() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        write_player_boost_status_fixture(
            temp_dir.path(),
            "ready",
            Some("scan-1"),
            None,
            false,
            None,
        );

        let error = request_player_boost(
            temp_dir.path(),
            "scan-1",
            77,
            99,
            172,
            PlayerBoostOperation::CurrentAbility { increment: 5 },
            DumpWaitConfig {
                timeout: Duration::from_millis(100),
                poll_interval: Duration::from_millis(10),
            },
        )
        .expect_err("bridge without boost support must be rejected");

        assert!(matches!(error, PlayerBoostRequestError::Unavailable(_)));
        assert!(!request_path(temp_dir.path()).exists());
    }

    #[test]
    fn bridge_requests_for_the_same_directory_cannot_overwrite_each_other() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let _guard = acquire_bridge_request_guard(temp_dir.path()).expect("hold bridge request");

        let dump_error = request_player_dump(
            temp_dir.path(),
            DumpWaitConfig {
                timeout: Duration::from_millis(100),
                poll_interval: Duration::from_millis(10),
            },
        )
        .expect_err("second dump request must not replace the in-flight request");
        assert!(matches!(dump_error, DumpRequestError::WriteFailed(_)));

        let boost_error = request_player_boost(
            temp_dir.path(),
            "scan-1",
            77,
            99,
            172,
            PlayerBoostOperation::CurrentAbility { increment: 5 },
            DumpWaitConfig {
                timeout: Duration::from_millis(100),
                poll_interval: Duration::from_millis(10),
            },
        )
        .expect_err("boost request must not replace the in-flight request");
        assert!(matches!(
            boost_error,
            PlayerBoostRequestError::Unavailable(_)
        ));
        assert!(!request_path(temp_dir.path()).exists());
    }

    #[test]
    fn player_boost_accepts_a_supported_status_from_a_prior_boost_for_repeat_actions() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bridge_dir = temp_dir.path().to_path_buf();
        write_player_boost_status_fixture(
            &bridge_dir,
            "ready",
            Some("boost-ca-prior"),
            None,
            true,
            Some(verified_ca_status_result(99, 104, 172)),
        );

        let responder_dir = bridge_dir.clone();
        let responder = thread::spawn(move || {
            loop {
                if request_path(&responder_dir).is_file() {
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
            let request: serde_json::Value = serde_json::from_str(
                &fs::read_to_string(request_path(&responder_dir)).expect("read boost request"),
            )
            .expect("parse boost request");
            assert_eq!(request["sourceRequestId"], "scan-1");
            assert_eq!(request["expectedCurrentAbility"], 104);
            write_player_boost_status_fixture(
                &responder_dir,
                "ready",
                request["requestId"].as_str(),
                None,
                true,
                Some(verified_ca_status_result(104, 109, 172)),
            );
        });

        let result = request_player_boost(
            &bridge_dir,
            "scan-1",
            77,
            104,
            172,
            PlayerBoostOperation::CurrentAbility { increment: 5 },
            DumpWaitConfig {
                timeout: Duration::from_secs(2),
                poll_interval: Duration::from_millis(20),
            },
        )
        .expect("repeat boost request");
        responder.join().expect("join boost responder");

        assert_eq!(result.current_ability, Some(109));
        assert_eq!(result.potential_ability, Some(172));
    }

    #[test]
    fn player_boost_accepts_a_preserved_live_index_after_a_failed_scan() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bridge_dir = temp_dir.path().to_path_buf();
        write_player_boost_status_fixture(
            &bridge_dir,
            "failed",
            Some("scan-failed"),
            Some("scan failed without replacing the prior dump"),
            true,
            None,
        );

        let responder_dir = bridge_dir.clone();
        let responder = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(1);
            while !request_path(&responder_dir).is_file() {
                if Instant::now() >= deadline {
                    return false;
                }
                thread::sleep(Duration::from_millis(10));
            }
            let request: serde_json::Value = serde_json::from_str(
                &fs::read_to_string(request_path(&responder_dir)).expect("read boost request"),
            )
            .expect("parse boost request");
            assert_eq!(request["sourceRequestId"], "scan-1");
            write_player_boost_status_fixture(
                &responder_dir,
                "ready",
                request["requestId"].as_str(),
                None,
                true,
                Some(verified_ca_status_result(99, 104, 172)),
            );
            true
        });

        let result = request_player_boost(
            &bridge_dir,
            "scan-1",
            77,
            99,
            172,
            PlayerBoostOperation::CurrentAbility { increment: 5 },
            DumpWaitConfig {
                timeout: Duration::from_secs(2),
                poll_interval: Duration::from_millis(20),
            },
        );
        assert!(responder.join().expect("join boost responder"));

        let result = result.expect("preserved live index boost request");
        assert_eq!(result.current_ability, Some(104));
        assert_eq!(result.potential_ability, Some(172));
    }

    #[test]
    fn player_boost_timeout_is_an_uncertain_outcome_that_requires_a_fresh_load() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        write_player_boost_status_fixture(
            temp_dir.path(),
            "ready",
            Some("scan-1"),
            None,
            true,
            None,
        );

        let error = request_player_boost(
            temp_dir.path(),
            "scan-1",
            77,
            99,
            172,
            PlayerBoostOperation::CurrentAbility { increment: 5 },
            DumpWaitConfig {
                timeout: Duration::from_millis(50),
                poll_interval: Duration::from_millis(10),
            },
        )
        .expect_err("timeout must not be exposed as a safe retry");

        assert!(matches!(error, PlayerBoostRequestError::Unconfirmed(_)));
        assert!(request_path(temp_dir.path()).exists());
    }

    #[test]
    fn player_boost_partial_rollback_is_an_uncertain_outcome() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bridge_dir = temp_dir.path().to_path_buf();
        write_player_boost_status_fixture(&bridge_dir, "ready", Some("scan-1"), None, true, None);

        let responder_dir = bridge_dir.clone();
        let responder = thread::spawn(move || {
            loop {
                if request_path(&responder_dir).is_file() {
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
            let request: serde_json::Value = serde_json::from_str(
                &fs::read_to_string(request_path(&responder_dir)).expect("read boost request"),
            )
            .expect("parse boost request");
            write_player_boost_status_fixture(
                &responder_dir,
                "failed",
                request["requestId"].as_str(),
                Some("player boost could not verify rollback; Load Data again before making another change"),
                false,
                Some(PlayerBoostResult {
                    operation: OPERATION_BOOST_CURRENT_ABILITY.to_string(),
                    outcome: "partial-unverified".to_string(),
                    rollback: "unverified".to_string(),
                    previous_current_ability: Some(99),
                    current_ability: None,
                    potential_ability: Some(172),
                    previous_ambition: None,
                    ambition: None,
                    previous_professionalism: None,
                    professionalism: None,
                    previous_determination: None,
                    determination: None,
                }),
            );
        });

        let error = request_player_boost(
            &bridge_dir,
            "scan-1",
            77,
            99,
            172,
            PlayerBoostOperation::CurrentAbility { increment: 5 },
            DumpWaitConfig {
                timeout: Duration::from_secs(2),
                poll_interval: Duration::from_millis(20),
            },
        )
        .expect_err("partial rollback must not be a retryable bridge failure");
        responder.join().expect("join boost responder");

        assert!(matches!(error, PlayerBoostRequestError::Unconfirmed(_)));
    }

    #[test]
    fn player_boost_unexpected_failed_status_is_an_uncertain_outcome() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bridge_dir = temp_dir.path().to_path_buf();
        write_player_boost_status_fixture(&bridge_dir, "ready", Some("scan-1"), None, true, None);

        let responder_dir = bridge_dir.clone();
        let responder = thread::spawn(move || {
            loop {
                if request_path(&responder_dir).is_file() {
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
            let request: serde_json::Value = serde_json::from_str(
                &fs::read_to_string(request_path(&responder_dir)).expect("read boost request"),
            )
            .expect("parse boost request");
            write_player_boost_status_fixture(
                &responder_dir,
                "failed",
                request["requestId"].as_str(),
                Some("player boost failed unexpectedly"),
                false,
                None,
            );
        });

        let error = request_player_boost(
            &bridge_dir,
            "scan-1",
            77,
            99,
            172,
            PlayerBoostOperation::CurrentAbility { increment: 5 },
            DumpWaitConfig {
                timeout: Duration::from_secs(2),
                poll_interval: Duration::from_millis(20),
            },
        )
        .expect_err("unexpected bridge failure must not be retryable");
        responder.join().expect("join boost responder");

        assert!(matches!(error, PlayerBoostRequestError::Unconfirmed(_)));
    }

    #[test]
    fn player_boost_ready_status_without_a_verified_result_is_uncertain() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        write_player_boost_status_fixture(
            temp_dir.path(),
            "ready",
            Some("boost-ca-1"),
            Some("player boost was not verified"),
            false,
            Some(PlayerBoostResult {
                operation: OPERATION_BOOST_CURRENT_ABILITY.to_string(),
                outcome: "failed".to_string(),
                rollback: "not-needed".to_string(),
                previous_current_ability: Some(99),
                current_ability: None,
                potential_ability: Some(172),
                previous_ambition: None,
                ambition: None,
                previous_professionalism: None,
                professionalism: None,
                previous_determination: None,
                determination: None,
            }),
        );

        let error = wait_for_player_boost_terminal(
            temp_dir.path(),
            "boost-ca-1",
            DumpWaitConfig {
                timeout: Duration::from_millis(100),
                poll_interval: Duration::from_millis(10),
            },
        )
        .expect_err("unverified ready result must not be accepted");

        assert!(matches!(error, PlayerBoostRequestError::Unconfirmed(_)));
    }

    fn write_status_fixture(
        bridge_dir: &Path,
        state: &str,
        request_id: Option<&str>,
        players_found: Option<i32>,
        error: Option<&str>,
    ) {
        write_status(
            bridge_dir,
            &status_fixture(state, request_id, players_found, error),
        );
    }

    fn status_fixture(
        state: &str,
        request_id: Option<&str>,
        players_found: Option<i32>,
        error: Option<&str>,
    ) -> BridgeStatus {
        BridgeStatus {
            protocol_version: 1,
            plugin_version: "0.1.0".to_string(),
            state: state.to_string(),
            updated_at_utc: "2026-07-28T18:30:00+00:00".to_string(),
            game_plugin_module_present: true,
            game_assembly_module_present: true,
            request_id: request_id.map(str::to_string),
            players_found,
            error: error.map(str::to_string),
            player_boosts_supported: None,
            player_boost: None,
            staff_boosts_supported: None,
            staff_boost: None,
            squad_moves_supported: None,
            squad_moves: None,
            staff_changes_supported: None,
            staff_changes: None,
            staff_limits_supported: None,
            staff_limits: None,
            training_supported: None,
            training: None,
        }
    }

    fn write_player_boost_status_fixture(
        bridge_dir: &Path,
        state: &str,
        request_id: Option<&str>,
        error: Option<&str>,
        player_boosts_supported: bool,
        player_boost: Option<PlayerBoostResult>,
    ) {
        let status = BridgeStatus {
            protocol_version: 1,
            plugin_version: "0.1.0".to_string(),
            state: state.to_string(),
            updated_at_utc: "2026-07-28T18:30:00+00:00".to_string(),
            game_plugin_module_present: true,
            game_assembly_module_present: true,
            request_id: request_id.map(str::to_string),
            players_found: None,
            error: error.map(str::to_string),
            player_boosts_supported: Some(player_boosts_supported),
            player_boost,
            staff_boosts_supported: None,
            staff_boost: None,
            squad_moves_supported: None,
            squad_moves: None,
            staff_changes_supported: None,
            staff_changes: None,
            staff_limits_supported: None,
            staff_limits: None,
            training_supported: None,
            training: None,
        };
        write_status(bridge_dir, &status);
    }

    fn write_staff_boost_status_fixture(
        bridge_dir: &Path,
        state: &str,
        request_id: Option<&str>,
        error: Option<&str>,
        staff_boosts_supported: bool,
        staff_boost: Option<StaffBoostResult>,
    ) {
        let status = BridgeStatus {
            protocol_version: 1,
            plugin_version: "0.1.0".to_string(),
            state: state.to_string(),
            updated_at_utc: "2026-08-16T10:00:00+00:00".to_string(),
            game_plugin_module_present: true,
            game_assembly_module_present: true,
            request_id: request_id.map(str::to_string),
            players_found: None,
            error: error.map(str::to_string),
            player_boosts_supported: None,
            player_boost: None,
            staff_boosts_supported: Some(staff_boosts_supported),
            staff_boost,
            squad_moves_supported: None,
            squad_moves: None,
            staff_changes_supported: None,
            staff_changes: None,
            staff_limits_supported: None,
            staff_limits: None,
            training_supported: None,
            training: None,
        };
        write_status(bridge_dir, &status);
    }

    fn write_status(bridge_dir: &Path, status: &BridgeStatus) {
        let json = serde_json::to_string_pretty(&status).expect("serialize");
        write_status_json(bridge_dir, &json);
    }

    /// Replaces `status.json` in one step, as the bridge does, so a reader polling it never
    /// sees it empty or half written.
    fn write_status_json(bridge_dir: &Path, json: &str) {
        let path = status_path(bridge_dir);
        let temp = path.with_extension("json.tmp");
        fs::write(&temp, json).expect("write status tmp");
        fs::rename(&temp, &path).expect("rename status");
    }

    fn verified_ca_status_result(
        previous_current_ability: i32,
        current_ability: i32,
        potential_ability: i32,
    ) -> PlayerBoostResult {
        PlayerBoostResult {
            operation: OPERATION_BOOST_CURRENT_ABILITY.to_string(),
            outcome: "verified".to_string(),
            rollback: "not-needed".to_string(),
            previous_current_ability: Some(previous_current_ability),
            current_ability: Some(current_ability),
            potential_ability: Some(potential_ability),
            previous_ambition: None,
            ambition: None,
            previous_professionalism: None,
            professionalism: None,
            previous_determination: None,
            determination: None,
        }
    }

    #[test]
    fn utc_epoch_formats_as_rfc3339() {
        assert_eq!(
            format_unix_secs_as_rfc3339(0, 0),
            "1970-01-01T00:00:00.000Z"
        );
        assert_eq!(
            format_unix_secs_as_rfc3339(1_753_725_600, 123),
            "2025-07-28T18:00:00.123Z"
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn resolve_bridge_directory_is_unsupported_on_non_windows() {
        let error = resolve_bridge_directory().expect_err("unsupported platform");
        assert!(matches!(error, BridgeStatusError::UnsupportedPlatform(_)));
        assert!(error.to_string().contains("Windows"));
    }

    #[cfg(windows)]
    #[test]
    fn resolve_bridge_directory_uses_local_app_data_on_windows() {
        let expected = bridge_directory_from_local_app_data(Path::new(
            &std::env::var_os("LOCALAPPDATA").expect("LOCALAPPDATA"),
        ));
        let actual = resolve_bridge_directory().expect("resolve on Windows");
        assert_eq!(actual, expected);
    }
}
