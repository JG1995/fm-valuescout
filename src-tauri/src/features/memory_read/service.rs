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
pub const OPERATION_WONDERKID_MENTALITY: &str = "wonderkid-mentality";
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
    pub scan_truncated: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_accepted: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_boosts_supported: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_boost: Option<PlayerBoostResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeRequest {
    pub protocol_version: u32,
    pub request_id: String,
    pub created_at_utc: String,
    pub operation: String,
    /// Optional accepted-player cap. `None` means unlimited (serialized as JSON `null`).
    pub max_accepted: Option<i32>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_accepted: Option<i32>,
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
            "FM26 memory read requires Windows".to_string(),
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
///
/// `max_accepted`: `None` = unlimited (production Load Data default); `Some(n)` stops after `n`
/// accepted players (tests / diagnostic caps).
pub fn request_player_dump(
    bridge_directory: &Path,
    wait: DumpWaitConfig,
) -> Result<DumpRequestResult, DumpRequestError> {
    request_player_dump_with_limit(bridge_directory, wait, None)
}

pub fn request_player_dump_with_limit(
    bridge_directory: &Path,
    wait: DumpWaitConfig,
    max_accepted: Option<i32>,
) -> Result<DumpRequestResult, DumpRequestError> {
    if let Some(limit) = max_accepted {
        if limit <= 0 {
            return Err(DumpRequestError::WriteFailed(
                "maxAccepted must be null or a positive integer".to_string(),
            ));
        }
    }
    let _bridge_request_guard =
        acquire_bridge_request_guard(bridge_directory).map_err(DumpRequestError::WriteFailed)?;

    let request_id = new_request_id();
    let request = BridgeRequest {
        protocol_version: PROTOCOL_VERSION,
        request_id: request_id.clone(),
        created_at_utc: utc_now_rfc3339(),
        operation: OPERATION_FULL_DUMP.to_string(),
        max_accepted,
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
        PlayerBoostOperation::CurrentAbility { increment: 5 | 10 } => {}
        PlayerBoostOperation::CurrentAbility { .. } => {
            return Err(PlayerBoostRequestError::WriteFailed(
                "current ability increment must be 5 or 10".to_string(),
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
                if dump_present {
                    if let Err(error) = validate_dump_at_bridge_directory(bridge_directory) {
                        log::warn!("dump.json failed ingestibility validation: {error}");
                    }
                }
                return Ok(DumpRequestResult {
                    request_id: request_id.to_string(),
                    state: status.state,
                    players_found: status.players_found,
                    dump_present,
                    error: status.error,
                    scan_truncated: status.scan_truncated,
                    max_accepted: status.max_accepted,
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

/// Validates `dump.json` under the bridge directory (ingest pre-check for feature 2).
pub fn validate_dump_at_bridge_directory(
    bridge_directory: &Path,
) -> Result<(), super::dump_validation::DumpValidationError> {
    super::dump_validation::validate_dump_file(&dump_path(bridge_directory))
}

/// Production IPC entry: resolve LocalAppData bridge dir, then request + wait.
pub fn request_player_dump_from_local_app_data(
    wait: DumpWaitConfig,
) -> Result<DumpRequestResult, DumpRequestError> {
    let bridge_directory = resolve_bridge_directory().map_err(DumpRequestError::from)?;
    request_player_dump(&bridge_directory, wait)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    const INGESTIBLE_DUMP_FIXTURE: &str = include_str!("fixtures/golden_dump_v6.json");

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
            max_accepted: None,
            player_database_scope: PLAYER_DATABASE_SCOPE_MEN.to_string(),
        };

        write_player_dump_request(bridge_dir, &request).expect("write request");

        let json = fs::read_to_string(request_path(bridge_dir)).expect("read request");
        let parsed: BridgeRequest = serde_json::from_str(&json).expect("parse request");
        assert_eq!(parsed.request_id, "req-test");
        assert_eq!(parsed.operation, "full-dump");
        assert_eq!(parsed.protocol_version, 1);
        assert_eq!(parsed.max_accepted, None);
        assert_eq!(parsed.player_database_scope, PLAYER_DATABASE_SCOPE_MEN);
        assert!(
            json.contains("\"playerDatabaseScope\": \"men\""),
            "production request must default playerDatabaseScope to men, got: {json}"
        );
        assert!(
            json.contains("\"maxAccepted\": null") || !json.contains("maxAccepted"),
            "unlimited request must omit maxAccepted or set it null, got: {json}"
        );
    }

    #[test]
    fn write_request_includes_positive_max_accepted_cap() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bridge_dir = temp_dir.path();
        let request = BridgeRequest {
            protocol_version: 1,
            request_id: "req-cap".to_string(),
            created_at_utc: "2026-07-28T18:30:00.000Z".to_string(),
            operation: OPERATION_FULL_DUMP.to_string(),
            max_accepted: Some(500),
            player_database_scope: PLAYER_DATABASE_SCOPE_MEN.to_string(),
        };

        write_player_dump_request(bridge_dir, &request).expect("write request");

        let json = fs::read_to_string(request_path(bridge_dir)).expect("read request");
        let parsed: BridgeRequest = serde_json::from_str(&json).expect("parse request");
        assert_eq!(parsed.max_accepted, Some(500));
        assert!(
            json.contains("\"maxAccepted\": 500"),
            "capped request must serialize maxAccepted, got: {json}"
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
            assert_eq!(request.max_accepted, None);
            assert_eq!(request.player_database_scope, PLAYER_DATABASE_SCOPE_MEN);
            assert!(
                json.contains("\"maxAccepted\": null") || !json.contains("maxAccepted"),
                "production request must be unlimited, got: {json}"
            );
            write_status_fixture(
                &writer_dir,
                "ready",
                Some(&request.request_id),
                Some(1),
                None,
                Some(false),
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
        assert_eq!(result.max_accepted, None);
    }

    #[test]
    fn request_player_dump_with_limit_rejects_non_positive_cap() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let error = request_player_dump_with_limit(
            temp_dir.path(),
            DumpWaitConfig {
                timeout: Duration::from_millis(50),
                poll_interval: Duration::from_millis(10),
            },
            Some(0),
        )
        .expect_err("non-positive cap");
        assert!(matches!(error, DumpRequestError::WriteFailed(_)));
        assert!(!request_path(temp_dir.path()).is_file());
    }

    #[test]
    fn wait_returns_ready_when_status_reaches_terminal_for_request() {
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
            write_status_fixture(
                &writer_dir,
                "scanning",
                Some(&writer_id),
                None,
                None,
                None,
                None,
            );
            thread::sleep(Duration::from_millis(30));
            fs::write(dump_path(&writer_dir), INGESTIBLE_DUMP_FIXTURE).expect("dump");
            write_status_fixture(
                &writer_dir,
                "ready",
                Some(&writer_id),
                Some(42),
                None,
                Some(false),
                Some(500),
            );
        });

        barrier.wait();
        let request = BridgeRequest {
            protocol_version: 1,
            request_id: request_id.clone(),
            created_at_utc: "2026-07-28T18:30:00.000Z".to_string(),
            operation: OPERATION_FULL_DUMP.to_string(),
            max_accepted: Some(500),
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

        assert_eq!(result.state, "ready");
        assert_eq!(result.players_found, Some(42));
        assert!(result.dump_present);
        assert!(result.error.is_none());
        assert_eq!(result.scan_truncated, Some(false));
        assert_eq!(result.max_accepted, Some(500));
        validate_dump_at_bridge_directory(&bridge_dir).expect("dump ingestible after ready");
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
            None,
            None,
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
        write_status_fixture(bridge_dir, "idle", None, None, None, None, None);

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

        write_status_fixture(
            &bridge_dir,
            "ready",
            Some("req-stale-other"),
            Some(9),
            None,
            None,
            None,
        );

        let writer_dir = bridge_dir.clone();
        let writer_barrier = Arc::clone(&barrier);
        let writer_id = ours.clone();
        thread::spawn(move || {
            writer_barrier.wait();
            thread::sleep(Duration::from_millis(40));
            fs::write(dump_path(&writer_dir), INGESTIBLE_DUMP_FIXTURE).expect("dump");
            write_status_fixture(
                &writer_dir,
                "ready",
                Some(&writer_id),
                Some(3),
                None,
                None,
                None,
            );
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
        validate_dump_at_bridge_directory(&bridge_dir).expect("dump ingestible after ready");
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
        scan_truncated: Option<bool>,
        max_accepted: Option<i32>,
    ) {
        let status = BridgeStatus {
            protocol_version: 1,
            plugin_version: "0.1.0".to_string(),
            state: state.to_string(),
            updated_at_utc: "2026-07-28T18:30:00+00:00".to_string(),
            game_plugin_module_present: true,
            game_assembly_module_present: true,
            request_id: request_id.map(str::to_string),
            players_found,
            error: error.map(str::to_string),
            scan_truncated,
            max_accepted,
            player_boosts_supported: None,
            player_boost: None,
        };
        write_status(bridge_dir, &status);
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
            scan_truncated: None,
            max_accepted: None,
            player_boosts_supported: Some(player_boosts_supported),
            player_boost,
        };
        write_status(bridge_dir, &status);
    }

    fn write_status(bridge_dir: &Path, status: &BridgeStatus) {
        let json = serde_json::to_string_pretty(&status).expect("serialize");
        let path = status_path(bridge_dir);
        let temp = path.with_extension("json.tmp");
        fs::write(&temp, &json).expect("write status tmp");
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
