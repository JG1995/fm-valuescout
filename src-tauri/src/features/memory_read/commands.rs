use super::install::{self, BridgeInstallError, BridgeInstallStatus};
use super::service::{
    self, BridgeStatus, BridgeStatusError, DumpRequestError, DumpRequestResult, DumpWaitConfig,
};
use tauri::AppHandle;

#[tauri::command]
pub fn get_bridge_status() -> Result<BridgeStatus, BridgeStatusError> {
    let bridge_directory = service::resolve_bridge_directory()?;
    service::read_bridge_status(&bridge_directory)
}

#[tauri::command]
pub fn request_player_dump() -> Result<DumpRequestResult, DumpRequestError> {
    service::request_player_dump_from_local_app_data(DumpWaitConfig::default())
}

#[tauri::command]
pub fn get_bridge_install_status() -> Result<BridgeInstallStatus, BridgeInstallError> {
    install::get_bridge_install_status()
}

#[tauri::command]
pub fn install_bridge_plugin(app: AppHandle) -> Result<BridgeInstallStatus, BridgeInstallError> {
    let source_dll = install::resolve_bundled_plugin_dll(&app)?;
    install::install_bridge_plugin(&source_dll)
}

#[tauri::command]
pub fn remove_bridge_plugin() -> Result<BridgeInstallStatus, BridgeInstallError> {
    install::remove_bridge_plugin()
}
