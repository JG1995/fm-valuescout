use crate::parser::types::ParsedPlayer;

/// Persist imported players to the database.
/// Currently a stub — will be implemented with the persistence layer.
/// Idempotent: skips players with same UID + in_game_date.
pub fn save_import(players: Vec<ParsedPlayer>, _in_game_date: &str) -> Result<(), String> {
    // TODO: Implement actual persistence when storage layer is built.
    // For now, just validate the input and return success.
    if players.is_empty() {
        return Ok(());
    }
    // Stub: accept all players, no actual storage
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::types::{Position, Role, Side};

    #[test]
    fn save_import_accepts_empty() {
        assert!(save_import(vec![], "2026-01-01").is_ok());
    }

    #[test]
    fn save_import_stub_accepts_players() {
        let players = vec![ParsedPlayer::empty(1, "Test".to_string(), vec![Position {
            role: Role::ST,
            sides: vec![Side::C],
        }])];
        assert!(save_import(players, "2026-01-01").is_ok());
    }
}
