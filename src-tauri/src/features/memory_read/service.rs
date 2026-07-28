use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;
pub const APP_FOLDER_NAME: &str = "fm-valuescout";
pub const BRIDGE_FOLDER_NAME: &str = "fm-bridge";
pub const STATUS_FILE_NAME: &str = "status.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeStatus {
    pub protocol_version: u32,
    pub plugin_version: String,
    pub state: String,
    pub updated_at_utc: String,
    pub game_plugin_module_present: bool,
    pub game_assembly_module_present: bool,
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

pub fn bridge_directory_from_local_app_data(local_app_data: &Path) -> PathBuf {
    local_app_data
        .join(APP_FOLDER_NAME)
        .join(BRIDGE_FOLDER_NAME)
}

pub fn status_path(bridge_directory: &Path) -> PathBuf {
    bridge_directory.join(STATUS_FILE_NAME)
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

#[cfg(test)]
mod tests {
    use super::*;

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
