use super::service::{
    self, BridgeStatus, BridgeStatusError, DumpRequestError, DumpRequestResult, DumpWaitConfig,
};

#[tauri::command]
pub fn get_bridge_status() -> Result<BridgeStatus, BridgeStatusError> {
    let bridge_directory = service::resolve_bridge_directory()?;
    service::read_bridge_status(&bridge_directory)
}

#[tauri::command]
pub fn request_player_dump() -> Result<DumpRequestResult, DumpRequestError> {
    service::request_player_dump_from_local_app_data(DumpWaitConfig::default())
}
