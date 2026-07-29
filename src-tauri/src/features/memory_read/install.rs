//! Steam FM26 BepInEx plugin path resolution and install-status inspection.
//!
// ponytail: module is public API for bridge-plugin-install IPC in commit 3
// Upgrade to wired commands when `get_bridge_install_status` is registered in commands.rs
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use serde::Serialize;

pub const PLUGIN_DLL_FILE_NAME: &str = "FmDataBridge.dll";
pub const FM26_STEAM_FOLDER_NAME: &str = "Football Manager 26";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeInstallStatus {
    pub plugins_path: String,
    pub plugin_present: bool,
    pub bepinex_present: bool,
    pub plugins_dir_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum BridgeInstallError {
    UnsupportedPlatform(String),
}

impl std::fmt::Display for BridgeInstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedPlatform(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for BridgeInstallError {}

/// Join `…/steamapps/common/Football Manager 26/BepInEx/plugins`.
pub fn plugins_path_from_steam_common(steam_common: &Path) -> PathBuf {
    steam_common
        .join(FM26_STEAM_FOLDER_NAME)
        .join("BepInEx")
        .join("plugins")
}

#[cfg(windows)]
fn default_steam_common_dir() -> PathBuf {
    PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common")
}

/// Resolves the FM26 BepInEx plugins directory (Windows only).
///
/// Resolution order mirrors `./scripts/dev bridge-install`:
/// `FM_BRIDGE_PLUGINS`, then `FM_STEAM_ROOT/BepInEx/plugins`, then the default Steam path.
pub fn resolve_steam_plugins_path() -> Result<PathBuf, BridgeInstallError> {
    #[cfg(windows)]
    {
        if let Ok(override_path) = std::env::var("FM_BRIDGE_PLUGINS") {
            if !override_path.is_empty() {
                return Ok(PathBuf::from(override_path));
            }
        }

        if let Ok(steam_root) = std::env::var("FM_STEAM_ROOT") {
            if !steam_root.is_empty() {
                return Ok(PathBuf::from(steam_root).join("BepInEx").join("plugins"));
            }
        }

        Ok(plugins_path_from_steam_common(&default_steam_common_dir()))
    }

    #[cfg(not(windows))]
    {
        Err(BridgeInstallError::UnsupportedPlatform(
            "FM26 bridge plugin install requires Windows".to_string(),
        ))
    }
}

pub fn inspect_bridge_install_at(plugins_path: &Path) -> BridgeInstallStatus {
    let bepinex_present = plugins_path.parent().is_some_and(|path| path.is_dir());
    let plugins_dir_present = plugins_path.is_dir();
    let plugin_present = plugins_dir_present && plugins_path.join(PLUGIN_DLL_FILE_NAME).is_file();

    BridgeInstallStatus {
        plugins_path: plugins_path.display().to_string(),
        plugin_present,
        bepinex_present,
        plugins_dir_present,
    }
}

pub fn get_bridge_install_status() -> Result<BridgeInstallStatus, BridgeInstallError> {
    let plugins_path = resolve_steam_plugins_path()?;
    Ok(inspect_bridge_install_at(&plugins_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn plugins_path_joins_steam_common_to_fm26_bepinex_plugins() {
        let steam_common = PathBuf::from("C:/Steam/steamapps/common");
        assert_eq!(
            plugins_path_from_steam_common(&steam_common),
            PathBuf::from("C:/Steam/steamapps/common/Football Manager 26/BepInEx/plugins")
        );
    }

    #[test]
    fn install_status_reports_plugin_absent_in_empty_plugins_tree() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let plugins = temp_dir.path().join("BepInEx").join("plugins");
        fs::create_dir_all(&plugins).expect("create plugins");

        let status = inspect_bridge_install_at(&plugins);

        assert!(!status.plugin_present);
        assert!(status.bepinex_present);
        assert!(status.plugins_dir_present);
        assert!(status.plugins_path.contains("plugins"));
    }

    #[test]
    fn install_status_reports_bepinex_absent_when_tree_missing() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let plugins = temp_dir
            .path()
            .join("Football Manager 26")
            .join("BepInEx")
            .join("plugins");

        let status = inspect_bridge_install_at(&plugins);

        assert!(!status.plugin_present);
        assert!(!status.bepinex_present);
        assert!(!status.plugins_dir_present);
    }

    #[test]
    fn install_status_reports_plugin_present_when_dll_exists() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let plugins = temp_dir.path().join("BepInEx").join("plugins");
        fs::create_dir_all(&plugins).expect("create plugins");
        fs::write(plugins.join(PLUGIN_DLL_FILE_NAME), b"fixture").expect("write dll");

        let status = inspect_bridge_install_at(&plugins);

        assert!(status.plugin_present);
        assert!(status.bepinex_present);
        assert!(status.plugins_dir_present);
    }

    #[cfg(windows)]
    #[test]
    fn resolve_steam_plugins_path_uses_default_windows_steam_layout() {
        let expected = plugins_path_from_steam_common(&default_steam_common_dir());
        let actual = resolve_steam_plugins_path().expect("resolve on Windows");
        assert_eq!(actual, expected);
        assert!(actual.ends_with("BepInEx\\plugins"));
        assert!(actual.to_string_lossy().contains(FM26_STEAM_FOLDER_NAME));
    }

    #[cfg(not(windows))]
    #[test]
    fn resolve_steam_plugins_path_is_unsupported_on_non_windows() {
        let error = resolve_steam_plugins_path().expect_err("unsupported platform");
        assert!(matches!(error, BridgeInstallError::UnsupportedPlatform(_)));
        assert!(error.to_string().contains("Windows"));
    }
}
