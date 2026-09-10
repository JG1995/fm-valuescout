use super::index::GraphicsKind;
use super::runtime::{picker, GraphicsRuntime, GraphicsStatus, ResolveResult};
use crate::db::Db;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_graphics_status(runtime: State<'_, GraphicsRuntime>) -> GraphicsStatus {
    runtime.status()
}

#[tauri::command]
pub fn choose_graphics_root(
    app: AppHandle,
    db: State<'_, Db>,
    runtime: State<'_, GraphicsRuntime>,
) -> Result<GraphicsStatus, String> {
    let Some(root) = picker(&app)? else {
        return Ok(runtime.status());
    };
    if root.as_os_str().is_empty() {
        return Ok(runtime.status());
    }
    if let Some((generation, root)) = runtime.persist_transition(&db.0, Some(root))? {
        runtime.scan_reserved(generation, root);
    }
    Ok(runtime.status())
}

#[tauri::command]
pub fn clear_graphics_root(
    db: State<'_, Db>,
    runtime: State<'_, GraphicsRuntime>,
) -> Result<GraphicsStatus, String> {
    if let Some((generation, root)) = runtime.persist_transition(&db.0, None)? {
        runtime.scan_reserved(generation, root);
    }
    Ok(runtime.status())
}

#[tauri::command]
pub fn rescan_graphics(
    db: State<'_, Db>,
    runtime: State<'_, GraphicsRuntime>,
) -> Result<GraphicsStatus, String> {
    if let Some((generation, root)) = runtime.begin_rescan(&db.0)? {
        runtime.scan_reserved(generation, root);
    }
    Ok(runtime.status())
}

#[tauri::command]
pub fn resolve_graphics(
    kind: GraphicsKindDto,
    uid: u32,
    runtime: State<'_, GraphicsRuntime>,
) -> ResolveResult {
    runtime.resolve(kind.into(), uid)
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GraphicsKindDto {
    PersonPortrait,
    ClubLogo,
    ClubIcon,
}
impl From<GraphicsKindDto> for GraphicsKind {
    fn from(v: GraphicsKindDto) -> Self {
        match v {
            GraphicsKindDto::PersonPortrait => Self::PersonPortrait,
            GraphicsKindDto::ClubLogo => Self::ClubLogo,
            GraphicsKindDto::ClubIcon => Self::ClubIcon,
        }
    }
}
