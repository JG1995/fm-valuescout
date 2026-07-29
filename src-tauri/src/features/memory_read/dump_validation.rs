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
    "scanTruncated",
    "maxAccepted",
    "playerCount",
    "players",
];

const REQUIRED_PLAYER_KEYS: &[&str] = &[
    "uid",
    "ca",
    "pa",
    "name",
    "birthYear",
    "birthDayOfYear",
    "nationalities",
    "preferredFoot",
    "positions",
    "attributes",
    "hiddenAttributes",
    "personality",
    "reputation",
];

/// Nullable MVP fields that must still be present on each player object.
const REQUIRED_PLAYER_NULLABLE_KEYS: &[&str] = &[
    "age",
    "heightCm",
    "weeklyWageGbp",
    "contractExpiryYear",
    "contractExpiryDayOfYear",
    "transferListed",
    "loanListed",
    "notForSale",
    "setForRelease",
    "marketValueGbp",
    "currentClub",
    "parentClub",
    "onLoan",
    "division",
    "teamLevel",
];

const VALID_GAME_DATE_SOURCES: &[&str] = &["memory", "derived", "unknown"];

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
    require_game_date_source(object, "gameDateSource")?;
    let scan_truncated = require_bool_value(object, "scanTruncated")?;
    require_nullable_non_negative_i64(object, "maxAccepted")?;
    if scan_truncated {
        match object.get("maxAccepted") {
            Some(Value::Number(number)) if number.as_i64().is_some_and(|n| n >= 0) => {}
            _ => {
                return Err(DumpValidationError::WrongType {
                    field: "maxAccepted".to_string(),
                    detail: "must be a non-negative number when scanTruncated is true".to_string(),
                });
            }
        }
    }

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

    for key in REQUIRED_PLAYER_NULLABLE_KEYS {
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
    require_i64_at(object, &format!("players[{index}].birthYear"), "birthYear")?;
    require_i64_at(
        object,
        &format!("players[{index}].birthDayOfYear"),
        "birthDayOfYear",
    )?;
    require_array_at(
        object,
        &format!("players[{index}].nationalities"),
        "nationalities",
    )?;
    require_string_at(
        object,
        &format!("players[{index}].preferredFoot"),
        "preferredFoot",
    )?;
    require_object_at(object, &format!("players[{index}].positions"), "positions")?;
    require_object_at(
        object,
        &format!("players[{index}].attributes"),
        "attributes",
    )?;
    require_object_at(
        object,
        &format!("players[{index}].hiddenAttributes"),
        "hiddenAttributes",
    )?;
    require_object_at(
        object,
        &format!("players[{index}].personality"),
        "personality",
    )?;
    require_object_at(
        object,
        &format!("players[{index}].reputation"),
        "reputation",
    )?;
    validate_reputation_object(
        object.get("reputation").expect("reputation checked above"),
        index,
    )?;
    require_nullable_i64_at(object, &format!("players[{index}].age"), "age")?;
    require_nullable_i64_at(object, &format!("players[{index}].heightCm"), "heightCm")?;
    require_nullable_i64_at(
        object,
        &format!("players[{index}].weeklyWageGbp"),
        "weeklyWageGbp",
    )?;
    require_nullable_i64_at(
        object,
        &format!("players[{index}].contractExpiryYear"),
        "contractExpiryYear",
    )?;
    require_nullable_i64_at(
        object,
        &format!("players[{index}].contractExpiryDayOfYear"),
        "contractExpiryDayOfYear",
    )?;
    require_nullable_bool_at(
        object,
        &format!("players[{index}].transferListed"),
        "transferListed",
    )?;
    require_nullable_bool_at(
        object,
        &format!("players[{index}].loanListed"),
        "loanListed",
    )?;
    require_nullable_bool_at(
        object,
        &format!("players[{index}].notForSale"),
        "notForSale",
    )?;
    require_nullable_bool_at(
        object,
        &format!("players[{index}].setForRelease"),
        "setForRelease",
    )?;
    require_nullable_i64_at(
        object,
        &format!("players[{index}].marketValueGbp"),
        "marketValueGbp",
    )?;
    require_nullable_string_at(
        object,
        &format!("players[{index}].currentClub"),
        "currentClub",
    )?;
    require_nullable_string_at(
        object,
        &format!("players[{index}].parentClub"),
        "parentClub",
    )?;
    require_nullable_bool_at(object, &format!("players[{index}].onLoan"), "onLoan")?;
    require_nullable_string_at(object, &format!("players[{index}].division"), "division")?;
    require_nullable_string_at(object, &format!("players[{index}].teamLevel"), "teamLevel")?;
    validate_int_or_null_map(
        object.get("attributes").expect("attributes checked above"),
        &format!("players[{index}].attributes"),
    )?;
    validate_int_or_null_map(
        object
            .get("hiddenAttributes")
            .expect("hiddenAttributes checked above"),
        &format!("players[{index}].hiddenAttributes"),
    )?;
    validate_int_or_null_map(
        object
            .get("personality")
            .expect("personality checked above"),
        &format!("players[{index}].personality"),
    )?;

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

fn require_bool_value(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<bool, DumpValidationError> {
    match object.get(field) {
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(DumpValidationError::WrongType {
            field: field.to_string(),
            detail: "expected boolean".to_string(),
        }),
        None => Err(DumpValidationError::MissingField(field.to_string())),
    }
}

fn require_game_date_source(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<(), DumpValidationError> {
    let value = object.get(field).and_then(Value::as_str).ok_or_else(|| {
        DumpValidationError::WrongType {
            field: field.to_string(),
            detail: "expected string".to_string(),
        }
    })?;
    if VALID_GAME_DATE_SOURCES.contains(&value) {
        Ok(())
    } else {
        Err(DumpValidationError::WrongType {
            field: field.to_string(),
            detail: "expected memory, derived, or unknown".to_string(),
        })
    }
}

fn require_nullable_i64_at(
    object: &serde_json::Map<String, Value>,
    display_field: &str,
    key: &str,
) -> Result<(), DumpValidationError> {
    match object.get(key) {
        Some(Value::Null) | Some(Value::Number(_)) => Ok(()),
        Some(_) => Err(DumpValidationError::WrongType {
            field: display_field.to_string(),
            detail: "expected number or null".to_string(),
        }),
        None => Err(DumpValidationError::MissingField(display_field.to_string())),
    }
}

fn require_nullable_bool_at(
    object: &serde_json::Map<String, Value>,
    display_field: &str,
    key: &str,
) -> Result<(), DumpValidationError> {
    match object.get(key) {
        Some(Value::Null) | Some(Value::Bool(_)) => Ok(()),
        Some(_) => Err(DumpValidationError::WrongType {
            field: display_field.to_string(),
            detail: "expected boolean or null".to_string(),
        }),
        None => Err(DumpValidationError::MissingField(display_field.to_string())),
    }
}

fn require_nullable_string_at(
    object: &serde_json::Map<String, Value>,
    display_field: &str,
    key: &str,
) -> Result<(), DumpValidationError> {
    match object.get(key) {
        Some(Value::Null) | Some(Value::String(_)) => Ok(()),
        Some(_) => Err(DumpValidationError::WrongType {
            field: display_field.to_string(),
            detail: "expected string or null".to_string(),
        }),
        None => Err(DumpValidationError::MissingField(display_field.to_string())),
    }
}

fn validate_reputation_object(reputation: &Value, index: usize) -> Result<(), DumpValidationError> {
    let object = reputation
        .as_object()
        .ok_or_else(|| DumpValidationError::WrongType {
            field: format!("players[{index}].reputation"),
            detail: "expected object".to_string(),
        })?;
    for key in ["current", "world"] {
        if !object.contains_key(key) {
            return Err(DumpValidationError::MissingField(format!(
                "players[{index}].reputation.{key}"
            )));
        }
        require_nullable_i64_at(object, &format!("players[{index}].reputation.{key}"), key)?;
    }
    Ok(())
}

fn validate_int_or_null_map(map: &Value, display_field: &str) -> Result<(), DumpValidationError> {
    let object = map
        .as_object()
        .ok_or_else(|| DumpValidationError::WrongType {
            field: display_field.to_string(),
            detail: "expected object".to_string(),
        })?;
    for (key, value) in object {
        match value {
            Value::Null | Value::Number(_) => {}
            _ => {
                return Err(DumpValidationError::WrongType {
                    field: format!("{display_field}.{key}"),
                    detail: "expected number or null".to_string(),
                });
            }
        }
    }
    Ok(())
}

fn require_nullable_non_negative_i64(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<(), DumpValidationError> {
    match object.get(field) {
        Some(Value::Null) => Ok(()),
        Some(value) => {
            let number = value
                .as_i64()
                .ok_or_else(|| DumpValidationError::WrongType {
                    field: field.to_string(),
                    detail: "expected number or null".to_string(),
                })?;
            if number < 0 {
                return Err(DumpValidationError::WrongType {
                    field: field.to_string(),
                    detail: "must be >= 0 or null".to_string(),
                });
            }
            Ok(())
        }
        None => Err(DumpValidationError::MissingField(field.to_string())),
    }
}

fn require_array_at(
    object: &serde_json::Map<String, Value>,
    display_field: &str,
    key: &str,
) -> Result<(), DumpValidationError> {
    match object.get(key) {
        Some(Value::Array(_)) => Ok(()),
        Some(_) => Err(DumpValidationError::WrongType {
            field: display_field.to_string(),
            detail: "expected array".to_string(),
        }),
        None => Err(DumpValidationError::MissingField(display_field.to_string())),
    }
}

fn require_object_at(
    object: &serde_json::Map<String, Value>,
    display_field: &str,
    key: &str,
) -> Result<(), DumpValidationError> {
    match object.get(key) {
        Some(Value::Object(_)) => Ok(()),
        Some(_) => Err(DumpValidationError::WrongType {
            field: display_field.to_string(),
            detail: "expected object".to_string(),
        }),
        None => Err(DumpValidationError::MissingField(display_field.to_string())),
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
    fn rejects_missing_scan_truncated() {
        let json = GOLDEN_FIXTURE.replace("\"scanTruncated\": false,\n  ", "");

        let error = validate_dump_json(&json).expect_err("missing scanTruncated");

        assert!(
            matches!(error, DumpValidationError::MissingField(field) if field == "scanTruncated")
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
  "scanTruncated": false,
  "maxAccepted": null,
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
  "scanTruncated": false,
  "maxAccepted": null,
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
    fn rejects_player_missing_reputation_object() {
        let json =
            GOLDEN_FIXTURE.replace("\"reputation\": { \"current\": 120, \"world\": 110 },", "");

        let error = validate_dump_json(&json).expect_err("missing reputation");

        assert!(matches!(
            error,
            DumpValidationError::MissingField(field) if field == "players[0].reputation"
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
    fn rejects_invalid_game_date_source() {
        let json = GOLDEN_FIXTURE.replace(
            "\"gameDateSource\": \"memory\"",
            "\"gameDateSource\": \"guess\"",
        );

        let error = validate_dump_json(&json).expect_err("invalid gameDateSource");

        assert!(matches!(
            error,
            DumpValidationError::WrongType { field, .. } if field == "gameDateSource"
        ));
    }

    #[test]
    fn rejects_scan_truncated_true_without_max_accepted() {
        let json = GOLDEN_FIXTURE
            .replace("\"scanTruncated\": false", "\"scanTruncated\": true")
            .replace("\"maxAccepted\": 500", "\"maxAccepted\": null");

        let error = validate_dump_json(&json).expect_err("truncated without cap");

        assert!(matches!(
            error,
            DumpValidationError::WrongType { field, .. } if field == "maxAccepted"
        ));
    }

    #[test]
    fn rejects_player_missing_current_club_key() {
        let json = GOLDEN_FIXTURE.replace("\"currentClub\": \"Loan FC\",", "");

        let error = validate_dump_json(&json).expect_err("missing currentClub");

        assert!(matches!(
            error,
            DumpValidationError::MissingField(field) if field == "players[0].currentClub"
        ));
    }

    #[test]
    fn rejects_attribute_map_with_string_value() {
        let json = GOLDEN_FIXTURE.replace("\"Acceleration\": 14", "\"Acceleration\": \"fast\"");

        let error = validate_dump_json(&json).expect_err("bad attribute value");

        assert!(matches!(
            error,
            DumpValidationError::WrongType { field, .. } if field == "players[0].attributes.Acceleration"
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
