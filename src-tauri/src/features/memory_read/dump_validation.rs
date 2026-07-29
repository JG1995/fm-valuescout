use std::fs;
use std::path::Path;

use serde_json::Value;

pub const DUMP_SCHEMA_VERSION: i64 = 5;
pub const DUMP_PROTOCOL_VERSION: i64 = 1;

const REQUIRED_TOP_LEVEL_KEYS: &[&str] = &[
    "schemaVersion",
    "generatedAtUtc",
    "gameVersion",
    "supportedGameVersion",
    "bridgeVersion",
    "protocolVersion",
    "gameDateSource",
    "playerCount",
    "players",
];

const REQUIRED_PLAYER_KEYS: &[&str] = &["uid", "ca", "pa", "name"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DumpValidationError {
    Corrupt(String),
    UnsupportedSchemaVersion {
        found: i64,
        expected: i64,
    },
    UnsupportedProtocolVersion {
        found: i64,
        expected: i64,
    },
    MissingField(String),
    WrongType {
        field: String,
        detail: String,
    },
    EmptyPlayers,
    PlayerCountMismatch {
        player_count: i64,
        players_len: usize,
    },
}

impl std::fmt::Display for DumpValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Corrupt(message) => write!(f, "{message}"),
            Self::UnsupportedSchemaVersion { found, expected } => {
                write!(
                    f,
                    "unsupported dump schema version {found}; expected {expected}"
                )
            }
            Self::UnsupportedProtocolVersion { found, expected } => {
                write!(
                    f,
                    "unsupported dump protocol version {found}; expected {expected}"
                )
            }
            Self::MissingField(field) => write!(f, "missing required field `{field}`"),
            Self::WrongType { field, detail } => {
                write!(f, "field `{field}` has wrong type: {detail}")
            }
            Self::EmptyPlayers => write!(
                f,
                "dump has no players; set emptySave=true when playerCount is zero"
            ),
            Self::PlayerCountMismatch {
                player_count,
                players_len,
            } => write!(
                f,
                "playerCount ({player_count}) does not match players length ({players_len})"
            ),
        }
    }
}

impl std::error::Error for DumpValidationError {}

/// Validates that `dump.json` content is ingestible without importing into SQLite.
pub fn validate_dump_json(json: &str) -> Result<(), DumpValidationError> {
    if json.trim().is_empty() {
        return Err(DumpValidationError::Corrupt("dump is empty".to_string()));
    }

    let root: Value = serde_json::from_str(json).map_err(|error| {
        DumpValidationError::Corrupt(format!("dump is not valid JSON: {error}"))
    })?;

    let object = root
        .as_object()
        .ok_or_else(|| DumpValidationError::WrongType {
            field: "(root)".to_string(),
            detail: "expected JSON object".to_string(),
        })?;

    for key in REQUIRED_TOP_LEVEL_KEYS {
        if !object.contains_key(*key) {
            return Err(DumpValidationError::MissingField((*key).to_string()));
        }
    }

    let schema_version = require_i64(object, "schemaVersion")?;
    if schema_version != DUMP_SCHEMA_VERSION {
        return Err(DumpValidationError::UnsupportedSchemaVersion {
            found: schema_version,
            expected: DUMP_SCHEMA_VERSION,
        });
    }

    let protocol_version = require_i64(object, "protocolVersion")?;
    if protocol_version != DUMP_PROTOCOL_VERSION {
        return Err(DumpValidationError::UnsupportedProtocolVersion {
            found: protocol_version,
            expected: DUMP_PROTOCOL_VERSION,
        });
    }

    require_non_empty_string(object, "generatedAtUtc")?;
    require_string(object, "gameVersion")?;
    require_string(object, "supportedGameVersion")?;
    require_string(object, "bridgeVersion")?;
    require_string(object, "gameDateSource")?;

    let player_count = require_i64(object, "playerCount")?;
    if player_count < 0 {
        return Err(DumpValidationError::WrongType {
            field: "playerCount".to_string(),
            detail: "must be >= 0".to_string(),
        });
    }

    let players = object
        .get("players")
        .ok_or_else(|| DumpValidationError::MissingField("players".to_string()))?;
    let players_array = players
        .as_array()
        .ok_or_else(|| DumpValidationError::WrongType {
            field: "players".to_string(),
            detail: "expected array".to_string(),
        })?;

    if player_count as usize != players_array.len() {
        return Err(DumpValidationError::PlayerCountMismatch {
            player_count,
            players_len: players_array.len(),
        });
    }

    let empty_save = object
        .get("emptySave")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if players_array.is_empty() {
        if empty_save && player_count == 0 {
            return Ok(());
        }
        return Err(DumpValidationError::EmptyPlayers);
    }

    for (index, player) in players_array.iter().enumerate() {
        validate_player_object(player, index)?;
    }

    Ok(())
}

pub fn validate_dump_file(path: &Path) -> Result<(), DumpValidationError> {
    let json = fs::read_to_string(path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => {
            DumpValidationError::Corrupt("dump.json not found".to_string())
        }
        _ => DumpValidationError::Corrupt("dump.json could not be read".to_string()),
    })?;
    validate_dump_json(&json)
}

fn validate_player_object(player: &Value, index: usize) -> Result<(), DumpValidationError> {
    let object = player
        .as_object()
        .ok_or_else(|| DumpValidationError::WrongType {
            field: format!("players[{index}]"),
            detail: "expected object".to_string(),
        })?;

    for key in REQUIRED_PLAYER_KEYS {
        if !object.contains_key(*key) {
            return Err(DumpValidationError::MissingField(format!(
                "players[{index}].{key}"
            )));
        }
    }

    require_u64(object, &format!("players[{index}].uid"), "uid")?;
    require_i64_at(object, &format!("players[{index}].ca"), "ca")?;
    require_i64_at(object, &format!("players[{index}].pa"), "pa")?;
    require_string_at(object, &format!("players[{index}].name"), "name")?;

    Ok(())
}

fn require_i64_at(
    object: &serde_json::Map<String, Value>,
    display_field: &str,
    key: &str,
) -> Result<i64, DumpValidationError> {
    let value = object
        .get(key)
        .ok_or_else(|| DumpValidationError::MissingField(display_field.to_string()))?;
    value
        .as_i64()
        .ok_or_else(|| DumpValidationError::WrongType {
            field: display_field.to_string(),
            detail: "expected number".to_string(),
        })
}

fn require_i64(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<i64, DumpValidationError> {
    require_i64_at(object, field, field)
}

fn require_u64(
    object: &serde_json::Map<String, Value>,
    display_field: &str,
    field: &str,
) -> Result<u64, DumpValidationError> {
    let value = require_i64_at(object, display_field, field)?;
    u64::try_from(value).map_err(|_| DumpValidationError::WrongType {
        field: display_field.to_string(),
        detail: "expected non-negative integer".to_string(),
    })
}

fn require_string_at(
    object: &serde_json::Map<String, Value>,
    display_field: &str,
    key: &str,
) -> Result<(), DumpValidationError> {
    match object.get(key) {
        Some(Value::String(_)) => Ok(()),
        Some(_) => Err(DumpValidationError::WrongType {
            field: display_field.to_string(),
            detail: "expected string".to_string(),
        }),
        None => Err(DumpValidationError::MissingField(display_field.to_string())),
    }
}

fn require_string(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<(), DumpValidationError> {
    require_string_at(object, field, field)
}

fn require_non_empty_string(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<(), DumpValidationError> {
    match object.get(field) {
        Some(Value::String(value)) if !value.trim().is_empty() => Ok(()),
        Some(Value::String(_)) => Err(DumpValidationError::WrongType {
            field: field.to_string(),
            detail: "expected non-empty string".to_string(),
        }),
        Some(_) => Err(DumpValidationError::WrongType {
            field: field.to_string(),
            detail: "expected string".to_string(),
        }),
        None => Err(DumpValidationError::MissingField(field.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOLDEN_FIXTURE: &str = include_str!("fixtures/golden_dump_v5.json");

    #[test]
    fn golden_fixture_passes_ingestibility_validation() {
        validate_dump_json(GOLDEN_FIXTURE).expect("golden dump v5 should be ingestible");
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let json = GOLDEN_FIXTURE.replace("\"schemaVersion\": 5", "\"schemaVersion\": 4");

        let error = validate_dump_json(&json).expect_err("schema v4");

        assert!(matches!(
            error,
            DumpValidationError::UnsupportedSchemaVersion {
                found: 4,
                expected: 5
            }
        ));
    }

    #[test]
    fn rejects_missing_required_top_level_key() {
        let json = GOLDEN_FIXTURE.replace("\"gameDateSource\": \"memory\",", "");

        let error = validate_dump_json(&json).expect_err("missing gameDateSource");

        assert!(
            matches!(error, DumpValidationError::MissingField(field) if field == "gameDateSource")
        );
    }

    #[test]
    fn rejects_empty_players_without_empty_save_marker() {
        let json = r#"{
  "schemaVersion": 5,
  "generatedAtUtc": "2026-07-29T10:00:00.000Z",
  "gameVersion": "26.3.2",
  "supportedGameVersion": "26.3",
  "bridgeVersion": "0.1.0",
  "protocolVersion": 1,
  "gameDateSource": "unknown",
  "playerCount": 0,
  "players": []
}"#;

        let error = validate_dump_json(json).expect_err("empty players");

        assert!(matches!(error, DumpValidationError::EmptyPlayers));
    }

    #[test]
    fn accepts_explicit_empty_save_marker() {
        let json = r#"{
  "schemaVersion": 5,
  "generatedAtUtc": "2026-07-29T10:00:00.000Z",
  "gameVersion": "26.3.2",
  "supportedGameVersion": "26.3",
  "bridgeVersion": "0.1.0",
  "protocolVersion": 1,
  "gameDateSource": "unknown",
  "emptySave": true,
  "playerCount": 0,
  "players": []
}"#;

        validate_dump_json(json).expect("emptySave marker should validate");
    }

    #[test]
    fn rejects_player_count_mismatch() {
        let json = GOLDEN_FIXTURE.replace("\"playerCount\": 1", "\"playerCount\": 2");

        let error = validate_dump_json(&json).expect_err("count mismatch");

        assert!(matches!(
            error,
            DumpValidationError::PlayerCountMismatch {
                player_count: 2,
                players_len: 1
            }
        ));
    }

    #[test]
    fn rejects_player_missing_uid() {
        let json = GOLDEN_FIXTURE.replace("\"uid\": 77,", "");

        let error = validate_dump_json(&json).expect_err("missing uid");

        assert!(matches!(
            error,
            DumpValidationError::MissingField(field) if field == "players[0].uid"
        ));
    }

    #[test]
    fn rejects_player_ca_wrong_type_with_indexed_field_path() {
        let json = GOLDEN_FIXTURE.replace("\"ca\": 150", "\"ca\": \"bad\"");

        let error = validate_dump_json(&json).expect_err("ca wrong type");

        assert!(matches!(
            error,
            DumpValidationError::WrongType { field, .. } if field == "players[0].ca"
        ));
    }

    #[test]
    fn validate_dump_file_reads_golden_fixture_from_disk() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let path = temp_dir.path().join("dump.json");
        fs::write(&path, GOLDEN_FIXTURE).expect("write fixture");

        validate_dump_file(&path).expect("file validation");
    }
}
