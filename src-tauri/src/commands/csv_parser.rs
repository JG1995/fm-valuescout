use crate::parser;
use crate::parser::types::ParseResult;

/// Parse a CSV file. Pure function — no side effects.
/// Returns ParseResult with players, skipped rows, warnings, and column status.
#[tauri::command]
pub fn parse_csv(file_path: String, _in_game_date: String) -> Result<ParseResult, String> {
    parser::parse_csv(&file_path)
}
