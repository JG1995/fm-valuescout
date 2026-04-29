use crate::parser;
use crate::parser::types::{ParseResult, ParsedPlayer};
use crate::storage;

/// Parse a CSV file. Pure function — no side effects.
/// Returns ParseResult with players, skipped rows, warnings, and column status.
#[tauri::command]
pub fn parse_csv(file_path: String, _in_game_date: String) -> Result<ParseResult, String> {
    parser::parse_csv(&file_path)
}

/// Persist parsed players to the database.
/// Idempotent: players with the same UID + in_game_date are skipped silently.
#[tauri::command]
pub fn save_import(players: Vec<ParsedPlayer>, in_game_date: String) -> Result<(), String> {
    storage::save_import(players, &in_game_date)
}
