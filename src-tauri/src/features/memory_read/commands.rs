use std::sync::{Mutex, MutexGuard};

use super::install::{self, BridgeInstallError, BridgeInstallStatus};
use super::service::{
    self, BridgeStatus, BridgeStatusError, DumpRequestError, DumpRequestResult, DumpWaitConfig,
};
use crate::blocking;
use tauri::AppHandle;

/// Keeps dump requests, plugin installs, and plugin removals from overlapping, as they
/// never did while every command ran on the main thread.
static BRIDGE_FILES: Mutex<()> = Mutex::new(());

fn lock_bridge_files() -> MutexGuard<'static, ()> {
    BRIDGE_FILES
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[tauri::command]
pub async fn get_bridge_status() -> Result<BridgeStatus, BridgeStatusError> {
    blocking::run(move || {
        let bridge_directory = service::resolve_bridge_directory()?;
        service::read_bridge_status(&bridge_directory)
    })
    .await
}

#[tauri::command]
pub async fn request_player_dump() -> Result<DumpRequestResult, DumpRequestError> {
    blocking::run(move || {
        let _bridge_files = lock_bridge_files();
        service::request_player_dump_from_local_app_data(DumpWaitConfig::default())
    })
    .await
}

#[tauri::command]
pub async fn get_bridge_install_status() -> Result<BridgeInstallStatus, BridgeInstallError> {
    blocking::run(install::get_bridge_install_status).await
}

#[tauri::command]
pub async fn install_bridge_plugin(
    app: AppHandle,
) -> Result<BridgeInstallStatus, BridgeInstallError> {
    blocking::run(move || {
        let _bridge_files = lock_bridge_files();
        let source_dll = install::resolve_bundled_plugin_dll(&app)?;
        install::install_bridge_plugin(&source_dll)
    })
    .await
}

#[tauri::command]
pub async fn remove_bridge_plugin() -> Result<BridgeInstallStatus, BridgeInstallError> {
    blocking::run(|| {
        let _bridge_files = lock_bridge_files();
        install::remove_bridge_plugin()
    })
    .await
}
