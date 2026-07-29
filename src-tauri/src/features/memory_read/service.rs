use std::fs;
use std::path::{Path, PathBuf};
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeRequest {
    pub protocol_version: u32,
    pub request_id: String,
    pub created_at_utc: String,
    pub operation: String,
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
    format!("req-{millis}")
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
    let request_id = new_request_id();
    let request = BridgeRequest {
        protocol_version: PROTOCOL_VERSION,
        request_id: request_id.clone(),
        created_at_utc: utc_now_rfc3339(),
        operation: OPERATION_FULL_DUMP.to_string(),
    };

    write_player_dump_request(bridge_directory, &request)?;
    wait_for_request_terminal(bridge_directory, &request_id, wait)
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

    const INGESTIBLE_DUMP_FIXTURE: &str = include_str!("fixtures/golden_dump_v5.json");

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
        };

        write_player_dump_request(bridge_dir, &request).expect("write request");

        let json = fs::read_to_string(request_path(bridge_dir)).expect("read request");
        let parsed: BridgeRequest = serde_json::from_str(&json).expect("parse request");
        assert_eq!(parsed.request_id, "req-test");
        assert_eq!(parsed.operation, "full-dump");
        assert_eq!(parsed.protocol_version, 1);
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
            write_status_fixture(&writer_dir, "scanning", Some(&writer_id), None, None);
            thread::sleep(Duration::from_millis(30));
            fs::write(dump_path(&writer_dir), INGESTIBLE_DUMP_FIXTURE).expect("dump");
            write_status_fixture(&writer_dir, "ready", Some(&writer_id), Some(42), None);
        });

        barrier.wait();
        let request = BridgeRequest {
            protocol_version: 1,
            request_id: request_id.clone(),
            created_at_utc: "2026-07-28T18:30:00.000Z".to_string(),
            operation: OPERATION_FULL_DUMP.to_string(),
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
        validate_dump_at_bridge_directory(&bridge_dir).expect("dump ingestible after ready");
    }

    fn write_status_fixture(
        bridge_dir: &Path,
        state: &str,
        request_id: Option<&str>,
        players_found: Option<i32>,
        error: Option<&str>,
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
            scan_truncated: None,
            max_accepted: None,
        };
        let json = serde_json::to_string_pretty(&status).expect("serialize");
        let path = status_path(bridge_dir);
        let temp = path.with_extension("json.tmp");
        fs::write(&temp, &json).expect("write status tmp");
        fs::rename(&temp, &path).expect("rename status");
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
