//! Steam FM26 BepInEx plugin path resolution and install-status inspection.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{path::BaseDirectory, AppHandle, Manager};

pub const PLUGIN_DLL_FILE_NAME: &str = "FmDataBridge.dll";
/// Path under `bundle.resources` — must match `tauri.conf.json` entry.
pub const BUNDLED_PLUGIN_DLL_RESOURCE: &str = "resources/FmDataBridge.dll";
#[cfg_attr(not(windows), allow(dead_code))]
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
    BepinexMissing(String),
    SourceMissing(String),
    WriteFailed(String),
    RemoveFailed(String),
}

impl std::fmt::Display for BridgeInstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedPlatform(message)
            | Self::BepinexMissing(message)
            | Self::SourceMissing(message)
            | Self::WriteFailed(message)
            | Self::RemoveFailed(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for BridgeInstallError {}

/// Join `…/steamapps/common/Football Manager 26/BepInEx/plugins`.
#[cfg_attr(not(windows), allow(dead_code))]
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

/// Resolves the bundled `FmDataBridge.dll` shipped as a Tauri resource.
pub fn resolve_bundled_plugin_dll(app: &AppHandle) -> Result<PathBuf, BridgeInstallError> {
    let path = app
        .path()
        .resolve(BUNDLED_PLUGIN_DLL_RESOURCE, BaseDirectory::Resource)
        .map_err(|error| {
            BridgeInstallError::SourceMissing(format!(
                "could not resolve bundled plugin DLL: {error}"
            ))
        })?;

    if !path.is_file() {
        return Err(BridgeInstallError::SourceMissing(format!(
            "bundled plugin DLL not found at {}",
            path.display()
        )));
    }

    Ok(path)
}

fn ensure_plugins_dir(plugins_path: &Path) -> Result<(), BridgeInstallError> {
    let bepinex_path = plugins_path
        .parent()
        .ok_or_else(|| BridgeInstallError::BepinexMissing("invalid plugins path".to_string()))?;

    if !bepinex_path.is_dir() {
        return Err(BridgeInstallError::BepinexMissing(format!(
            "BepInEx not found at {}",
            bepinex_path.display()
        )));
    }

    if !plugins_path.is_dir() {
        fs::create_dir_all(plugins_path).map_err(|error| {
            BridgeInstallError::WriteFailed(format!("could not create plugins directory: {error}"))
        })?;
    }

    Ok(())
}

/// Copies `source_dll` into `plugins_path` as `FmDataBridge.dll`.
///
/// Creates `plugins` only when the BepInEx parent directory already exists.
pub fn install_bridge_plugin_at(
    source_dll: &Path,
    plugins_path: &Path,
) -> Result<BridgeInstallStatus, BridgeInstallError> {
    if !source_dll.is_file() {
        return Err(BridgeInstallError::SourceMissing(format!(
            "plugin DLL not found at {}",
            source_dll.display()
        )));
    }

    ensure_plugins_dir(plugins_path)?;

    let destination = plugins_path.join(PLUGIN_DLL_FILE_NAME);
    fs::copy(source_dll, &destination).map_err(|error| {
        BridgeInstallError::WriteFailed(format!("could not copy plugin DLL: {error}"))
    })?;

    Ok(inspect_bridge_install_at(plugins_path))
}

pub fn install_bridge_plugin(source_dll: &Path) -> Result<BridgeInstallStatus, BridgeInstallError> {
    let plugins_path = resolve_steam_plugins_path()?;
    install_bridge_plugin_at(source_dll, &plugins_path)
}

/// Deletes only `FmDataBridge.dll` under `plugins_path` when present.
pub fn remove_bridge_plugin_at(
    plugins_path: &Path,
) -> Result<BridgeInstallStatus, BridgeInstallError> {
    let plugin_path = plugins_path.join(PLUGIN_DLL_FILE_NAME);
    if plugin_path.is_file() {
        fs::remove_file(&plugin_path).map_err(|error| {
            BridgeInstallError::RemoveFailed(format!("could not remove plugin DLL: {error}"))
        })?;
    }

    Ok(inspect_bridge_install_at(plugins_path))
}

pub fn remove_bridge_plugin() -> Result<BridgeInstallStatus, BridgeInstallError> {
    let plugins_path = resolve_steam_plugins_path()?;
    remove_bridge_plugin_at(&plugins_path)
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

    fn temp_plugins_tree(temp_dir: &Path) -> PathBuf {
        let plugins = temp_dir.join("BepInEx").join("plugins");
        fs::create_dir_all(&plugins).expect("create plugins");
        plugins
    }

    fn write_fixture_dll(path: &Path, contents: &[u8]) -> PathBuf {
        let fixture = path.join("fixture-source.dll");
        fs::write(&fixture, contents).expect("write fixture");
        fixture
    }

    #[test]
    fn install_copies_fixture_dll_into_plugins_tree() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let plugins = temp_plugins_tree(temp_dir.path());
        let source = write_fixture_dll(temp_dir.path(), b"fixture-v1");

        let status = install_bridge_plugin_at(&source, &plugins).expect("install");

        assert!(status.plugin_present);
        let installed = plugins.join(PLUGIN_DLL_FILE_NAME);
        assert!(installed.is_file());
        assert_eq!(fs::read(&installed).expect("read installed"), b"fixture-v1");
    }

    #[test]
    fn install_overwrites_existing_plugin_dll() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let plugins = temp_plugins_tree(temp_dir.path());
        let existing = plugins.join(PLUGIN_DLL_FILE_NAME);
        fs::write(&existing, b"old").expect("write old dll");
        let source = write_fixture_dll(temp_dir.path(), b"new");

        let status = install_bridge_plugin_at(&source, &plugins).expect("install");

        assert!(status.plugin_present);
        assert_eq!(fs::read(&existing).expect("read installed"), b"new");
    }

    #[test]
    fn install_creates_plugins_dir_when_bepinex_exists() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bepinex = temp_dir.path().join("BepInEx");
        fs::create_dir_all(&bepinex).expect("create bepinex");
        let plugins = bepinex.join("plugins");
        let source = write_fixture_dll(temp_dir.path(), b"fixture");

        let status = install_bridge_plugin_at(&source, &plugins).expect("install");

        assert!(plugins.is_dir());
        assert!(status.plugin_present);
        assert!(status.plugins_dir_present);
    }

    #[test]
    fn install_fails_closed_when_bepinex_parent_missing() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let plugins = temp_dir
            .path()
            .join("Football Manager 26")
            .join("BepInEx")
            .join("plugins");
        let source = write_fixture_dll(temp_dir.path(), b"fixture");

        let error = install_bridge_plugin_at(&source, &plugins).expect_err("missing bepinex");

        assert!(matches!(error, BridgeInstallError::BepinexMissing(_)));
        assert!(error.to_string().contains("BepInEx"));
        assert!(!plugins.join(PLUGIN_DLL_FILE_NAME).exists());
    }

    #[test]
    fn install_fails_when_source_dll_missing() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let plugins = temp_plugins_tree(temp_dir.path());
        let missing = temp_dir.path().join("missing.dll");

        let error = install_bridge_plugin_at(&missing, &plugins).expect_err("missing source");

        assert!(matches!(error, BridgeInstallError::SourceMissing(_)));
        assert!(!plugins.join(PLUGIN_DLL_FILE_NAME).exists());
    }

    #[test]
    fn remove_deletes_only_fm_data_bridge_dll() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let plugins = temp_plugins_tree(temp_dir.path());
        fs::write(plugins.join(PLUGIN_DLL_FILE_NAME), b"ours").expect("write ours");
        fs::write(plugins.join("OtherPlugin.dll"), b"other").expect("write other");

        let status = remove_bridge_plugin_at(&plugins).expect("remove");

        assert!(!status.plugin_present);
        assert!(!plugins.join(PLUGIN_DLL_FILE_NAME).exists());
        assert!(plugins.join("OtherPlugin.dll").is_file());
    }

    #[test]
    fn remove_succeeds_when_plugin_dll_absent() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let plugins = temp_plugins_tree(temp_dir.path());

        let status = remove_bridge_plugin_at(&plugins).expect("remove when absent");

        assert!(!status.plugin_present);
        assert!(status.plugins_dir_present);
    }
}
