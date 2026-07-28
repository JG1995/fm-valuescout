use super::service::{self, BridgeStatus, BridgeStatusError};

#[tauri::command]
pub fn get_bridge_status() -> Result<BridgeStatus, BridgeStatusError> {
    let bridge_directory = service::resolve_bridge_directory()?;
    service::read_bridge_status(&bridge_directory)
}
