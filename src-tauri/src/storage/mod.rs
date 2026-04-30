use crate::parser::types::ParsedPlayer;

/// Persist imported players to the database.
/// Currently an honest stub — returns an error until implemented.
/// Idempotent: skips players with same UID + in_game_date.
pub fn save_import(_players: Vec<ParsedPlayer>, _in_game_date: &str) -> Result<(), String> {
    Err("Storage is not yet implemented. Your data has not been saved.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::types::{Position, Role, Side};

    #[test]
    fn save_import_rejects_empty() {
        assert!(save_import(vec![], "2026-01-01").is_err());
    }

    #[test]
    fn save_import_stub_rejects_players() {
        let players = vec![ParsedPlayer::empty(1, "Test".to_string(), vec![Position {
            role: Role::ST,
            sides: vec![Side::C],
        }])];
        let result = save_import(players, "2026-01-01");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not yet implemented"));
    }
}
