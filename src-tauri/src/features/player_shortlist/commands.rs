use tauri::State;

use crate::blocking;
use crate::db::Db;

use super::{PlayerShortlistChoice, PlayerShortlistError};

/// Adds the FM shortlist file at `path` to save `save_id`, identified by `context_token`, or
/// re-reads it when the save already links that path.
#[tauri::command]
pub async fn choose_player_shortlist(
    save_id: i64,
    context_token: String,
    path: String,
    db: State<'_, Db>,
) -> Result<PlayerShortlistChoice, PlayerShortlistError> {
    let db = db.inner().clone();
    blocking::run(move || super::choose_player_shortlist(&db, save_id, &context_token, &path)).await
}

/// Removes shortlist `shortlist_id` of save `save_id`, identified by `context_token`.
#[tauri::command]
pub async fn remove_player_shortlist(
    save_id: i64,
    context_token: String,
    shortlist_id: i64,
    db: State<'_, Db>,
) -> Result<(), PlayerShortlistError> {
    let db = db.inner().clone();
    blocking::run(move || {
        super::remove_player_shortlist(&db, save_id, &context_token, shortlist_id)
    })
    .await
}
