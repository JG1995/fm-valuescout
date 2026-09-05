use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde_json::Value;

pub const DUMP_SCHEMA_VERSION: i64 = 8;
pub const DUMP_PROTOCOL_VERSION: i64 = 1;

const POSITION_KEYS: &[&str] = &[
    "GK", "SW", "DL", "DC", "DR", "DM", "ML", "MC", "MR", "AML", "AMC", "AMR", "ST", "WBL", "WBR",
];

const REQUIRED_TOP_LEVEL_KEYS: &[&str] = &[
    "schemaVersion",
    "generatedAtUtc",
    "gameVersion",
    "supportedGameVersion",
    "bridgeVersion",
    "protocolVersion",
    "gameDateSource",
    "gameDateBasis",
    "playerDatabaseScope",
    "scanTruncated",
    "maxAccepted",
    "playerCount",
    "players",
    "staffCount",
    "staff",
    "manager",
];

const REQUIRED_PLAYER_KEYS: &[&str] = &[
    "uid",
    "ca",
    "pa",
    "name",
    "birthYear",
    "birthDayOfYear",
    "nationalities",
    "gender",
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
    "nationUid",
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
    "clubReputation",
    "teamType",
];

const REQUIRED_STAFF_KEYS: &[&str] = &["uid", "nationalities", "gender", "ca", "pa", "attributes"];

const REQUIRED_STAFF_NULLABLE_KEYS: &[&str] = &[
    "name",
    "birthYear",
    "birthDayOfYear",
    "age",
    "nationUid",
    "jobId",
    "weeklyWageGbp",
    "contractExpiryYear",
    "contractExpiryDayOfYear",
    "club",
    "division",
];

const STAFF_ATTRIBUTE_KEYS: &[&str] = &[
    "Attacking",
    "Defending",
    "Fitness",
    "Possession",
    "Technical",
    "Tactical",
    "SetPieces",
    "Determination",
    "ManManagement",
    "Motivating",
    "JudgingPlayerAbility",
    "JudgingPlayerPotential",
    "JudgingStaffAbility",
    "Negotiating",
    "TacticalKnowledge",
    "Physiotherapy",
    "SportsScience",
    "Authority",
    "Adaptability",
    "DataAnalysis",
    "WorkingWithYoungsters",
    "GoalkeepingDistribution",
    "GoalkeepingHandling",
    "GoalkeepingReflexes",
];

const VALID_GAME_DATE_SOURCES: &[&str] = &["memory", "derived", "unknown"];
const VALID_GAME_DATE_BASES: &[&str] = &[
    "next-fixture-consensus",
    "birth-cohort-and-system-date",
    "unknown",
];
const VALID_PLAYER_DATABASE_SCOPES: &[&str] = &["men", "women", "both"];
const VALID_GENDERS: &[&str] = &["unknown", "male", "female"];

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
    StaffCountMismatch {
        staff_count: i64,
        staff_len: usize,
    },
    DuplicateUid {
        entity: String,
        uid: u64,
    },
    PlayerStaffUidOverlap {
        uid: u64,
    },
    ManagerNotInStaff {
        uid: u64,
    },
}

impl std::fmt::Display for DumpValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Corrupt(message) => write!(f, "{message}"),
            Self::UnsupportedSchemaVersion { found, expected } => {
                write!(
                    f,
                    "unsupported dump schema version {found}; update the FM bridge plugin and rescan (expected {expected})"
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
            Self::StaffCountMismatch {
                staff_count,
                staff_len,
            } => write!(
                f,
                "staffCount ({staff_count}) does not match staff length ({staff_len})"
            ),
            Self::DuplicateUid { entity, uid } => {
                write!(f, "duplicate {entity} uid {uid}")
            }
            Self::PlayerStaffUidOverlap { uid } => {
                write!(f, "uid {uid} appears in both players and staff")
            }
            Self::ManagerNotInStaff { uid } => {
                write!(f, "manager uid {uid} does not identify a staff record")
            }
        }
    }
}

impl std::error::Error for DumpValidationError {}

/// Validates that `dump.json` content is ingestible without importing into SQLite.
pub fn validate_dump_json(json: &str) -> Result<(), DumpValidationError> {
    parse_and_validate_dump(json).map(|_| ())
}

/// Parses and validates dump JSON once, returning the root value for reuse by ingest.
pub fn parse_and_validate_dump(json: &str) -> Result<Value, DumpValidationError> {
    if json.trim().is_empty() {
        return Err(DumpValidationError::Corrupt("dump is empty".to_string()));
    }

    let root: Value = serde_json::from_str(json).map_err(|error| {
        DumpValidationError::Corrupt(format!("dump is not valid JSON: {error}"))
    })?;
    validate_dump_value(&root)?;
    Ok(root)
}

/// Validates an already-parsed dump document.
pub fn validate_dump_value(root: &Value) -> Result<(), DumpValidationError> {
    let object = root
        .as_object()
        .ok_or_else(|| DumpValidationError::WrongType {
            field: "(root)".to_string(),
            detail: "expected JSON object".to_string(),
        })?;

    let schema_version = require_i64(object, "schemaVersion")?;
    if schema_version != DUMP_SCHEMA_VERSION {
        return Err(DumpValidationError::UnsupportedSchemaVersion {
            found: schema_version,
            expected: DUMP_SCHEMA_VERSION,
        });
    }

    for key in REQUIRED_TOP_LEVEL_KEYS {
        if !object.contains_key(*key) {
            return Err(DumpValidationError::MissingField((*key).to_string()));
        }
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
    require_optional_game_date(object, "gameDate")?;
    require_game_date_source(object, "gameDateSource")?;
    require_game_date_basis(object, "gameDateBasis")?;
    require_player_database_scope(object, "playerDatabaseScope")?;
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

    let staff_count = require_i64(object, "staffCount")?;
    if staff_count < 0 {
        return Err(DumpValidationError::WrongType {
            field: "staffCount".to_string(),
            detail: "must be >= 0".to_string(),
        });
    }
    let staff = object
        .get("staff")
        .ok_or_else(|| DumpValidationError::MissingField("staff".to_string()))?;
    let staff_array = staff
        .as_array()
        .ok_or_else(|| DumpValidationError::WrongType {
            field: "staff".to_string(),
            detail: "expected array".to_string(),
        })?;
    if staff_count as usize != staff_array.len() {
        return Err(DumpValidationError::StaffCountMismatch {
            staff_count,
            staff_len: staff_array.len(),
        });
    }

    let empty_save = object
        .get("emptySave")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if players_array.is_empty() {
        if empty_save
            && player_count == 0
            && staff_array.is_empty()
            && object.get("manager") == Some(&Value::Null)
        {
            return Ok(());
        }
        return Err(DumpValidationError::EmptyPlayers);
    }

    let mut player_uids = HashSet::with_capacity(players_array.len());
    for (index, player) in players_array.iter().enumerate() {
        let uid = validate_player_object(player, index)?;
        if !player_uids.insert(uid) {
            return Err(DumpValidationError::DuplicateUid {
                entity: "player".to_string(),
                uid,
            });
        }
    }

    let mut staff_uids = HashSet::with_capacity(staff_array.len());
    for (index, staff) in staff_array.iter().enumerate() {
        let uid = validate_staff_object(staff, index)?;
        if !staff_uids.insert(uid) {
            return Err(DumpValidationError::DuplicateUid {
                entity: "staff".to_string(),
                uid,
            });
        }
        if player_uids.contains(&uid) {
            return Err(DumpValidationError::PlayerStaffUidOverlap { uid });
        }
    }

    validate_manager(object.get("manager"), &staff_uids)?;

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

fn validate_player_object(player: &Value, index: usize) -> Result<u64, DumpValidationError> {
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

    let uid = require_u64(object, &format!("players[{index}].uid"), "uid")?;
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
    require_nullable_u64_at(object, &format!("players[{index}].nationUid"), "nationUid")?;
    require_gender_at(object, &format!("players[{index}].gender"), "gender")?;
    require_string_at(
        object,
        &format!("players[{index}].preferredFoot"),
        "preferredFoot",
    )?;
    let positions = object
        .get("positions")
        .ok_or_else(|| DumpValidationError::MissingField(format!("players[{index}].positions")))?;
    validate_positions_map(positions, &format!("players[{index}].positions"))?;
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
    require_nullable_i64_at(
        object,
        &format!("players[{index}].clubReputation"),
        "clubReputation",
    )?;
    require_nullable_i64_at(object, &format!("players[{index}].teamType"), "teamType")?;
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

    Ok(uid)
}

fn validate_staff_object(staff: &Value, index: usize) -> Result<u64, DumpValidationError> {
    let object = staff
        .as_object()
        .ok_or_else(|| DumpValidationError::WrongType {
            field: format!("staff[{index}]"),
            detail: "expected object".to_string(),
        })?;

    for key in REQUIRED_STAFF_KEYS {
        if !object.contains_key(*key) {
            return Err(DumpValidationError::MissingField(format!(
                "staff[{index}].{key}"
            )));
        }
    }
    for key in REQUIRED_STAFF_NULLABLE_KEYS {
        if !object.contains_key(*key) {
            return Err(DumpValidationError::MissingField(format!(
                "staff[{index}].{key}"
            )));
        }
    }

    let uid = require_u64(object, &format!("staff[{index}].uid"), "uid")?;
    require_nullable_string_at(object, &format!("staff[{index}].name"), "name")?;
    require_nullable_i64_at(object, &format!("staff[{index}].birthYear"), "birthYear")?;
    require_nullable_i64_at(
        object,
        &format!("staff[{index}].birthDayOfYear"),
        "birthDayOfYear",
    )?;
    require_nullable_i64_at(object, &format!("staff[{index}].age"), "age")?;
    require_array_at(
        object,
        &format!("staff[{index}].nationalities"),
        "nationalities",
    )?;
    require_nullable_u64_at(object, &format!("staff[{index}].nationUid"), "nationUid")?;
    require_gender_at(object, &format!("staff[{index}].gender"), "gender")?;
    require_i64_at(object, &format!("staff[{index}].ca"), "ca")?;
    require_i64_at(object, &format!("staff[{index}].pa"), "pa")?;
    validate_fixed_staff_attribute_map(
        object.get("attributes").expect("attributes checked above"),
        &format!("staff[{index}].attributes"),
    )?;
    require_nullable_i64_at(object, &format!("staff[{index}].jobId"), "jobId")?;
    require_nullable_i64_at(
        object,
        &format!("staff[{index}].weeklyWageGbp"),
        "weeklyWageGbp",
    )?;
    require_nullable_i64_at(
        object,
        &format!("staff[{index}].contractExpiryYear"),
        "contractExpiryYear",
    )?;
    require_nullable_i64_at(
        object,
        &format!("staff[{index}].contractExpiryDayOfYear"),
        "contractExpiryDayOfYear",
    )?;
    require_nullable_string_at(object, &format!("staff[{index}].club"), "club")?;
    require_nullable_string_at(object, &format!("staff[{index}].division"), "division")?;

    Ok(uid)
}

fn validate_manager(
    manager: Option<&Value>,
    staff_uids: &HashSet<u64>,
) -> Result<(), DumpValidationError> {
    let Some(manager) = manager else {
        return Err(DumpValidationError::MissingField("manager".to_string()));
    };
    if manager.is_null() {
        return Ok(());
    }
    let object = manager
        .as_object()
        .ok_or_else(|| DumpValidationError::WrongType {
            field: "manager".to_string(),
            detail: "expected object or null".to_string(),
        })?;
    let uid = require_u64(object, "manager.uid", "uid")?;
    require_non_empty_string_at(object, "manager.name", "name")?;
    require_nullable_string_at(object, "manager.club", "club")?;
    require_nullable_i64_at(object, "manager.clubReputation", "clubReputation")?;
    if !staff_uids.contains(&uid) {
        return Err(DumpValidationError::ManagerNotInStaff { uid });
    }
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
    require_non_empty_string_at(object, field, field)
}

fn require_non_empty_string_at(
    object: &serde_json::Map<String, Value>,
    display_field: &str,
    key: &str,
) -> Result<(), DumpValidationError> {
    match object.get(key) {
        Some(Value::String(value)) if !value.trim().is_empty() => Ok(()),
        Some(Value::String(_)) => Err(DumpValidationError::WrongType {
            field: display_field.to_string(),
            detail: "expected non-empty string".to_string(),
        }),
        Some(_) => Err(DumpValidationError::WrongType {
            field: display_field.to_string(),
            detail: "expected string".to_string(),
        }),
        None => Err(DumpValidationError::MissingField(display_field.to_string())),
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

fn require_optional_game_date(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<(), DumpValidationError> {
    let Some(value) = object.get(field) else {
        return Ok(());
    };
    let Value::String(date) = value else {
        if value.is_null() {
            return Ok(());
        }
        return Err(DumpValidationError::WrongType {
            field: field.to_string(),
            detail: "expected YYYY-MM-DD or null".to_string(),
        });
    };
    if canonical_game_date(date) {
        Ok(())
    } else {
        Err(DumpValidationError::WrongType {
            field: field.to_string(),
            detail: "expected YYYY-MM-DD".to_string(),
        })
    }
}

pub(crate) fn canonical_game_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if bytes
        .iter()
        .enumerate()
        .any(|(index, byte)| !matches!(index, 4 | 7) && !byte.is_ascii_digit())
    {
        return false;
    }

    let year = decimal(&bytes[0..4]);
    let month = decimal(&bytes[5..7]);
    let day = decimal(&bytes[8..10]);
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => return false,
    };

    year > 0 && (1..=days_in_month).contains(&day)
}

fn decimal(bytes: &[u8]) -> u16 {
    bytes
        .iter()
        .fold(0, |value, byte| value * 10 + u16::from(byte - b'0'))
}

fn require_game_date_basis(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<(), DumpValidationError> {
    require_enum_value(
        object,
        field,
        VALID_GAME_DATE_BASES,
        "expected next-fixture-consensus, birth-cohort-and-system-date, or unknown",
    )
}

fn require_player_database_scope(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<(), DumpValidationError> {
    require_enum_value(
        object,
        field,
        VALID_PLAYER_DATABASE_SCOPES,
        "expected men, women, or both",
    )
}

fn require_gender_at(
    object: &serde_json::Map<String, Value>,
    display_field: &str,
    key: &str,
) -> Result<(), DumpValidationError> {
    require_enum_value_at(
        object,
        display_field,
        key,
        VALID_GENDERS,
        "expected unknown, male, or female",
    )
}

fn require_enum_value(
    object: &serde_json::Map<String, Value>,
    field: &str,
    valid_values: &[&str],
    detail: &str,
) -> Result<(), DumpValidationError> {
    require_enum_value_at(object, field, field, valid_values, detail)
}

fn require_enum_value_at(
    object: &serde_json::Map<String, Value>,
    display_field: &str,
    key: &str,
    valid_values: &[&str],
    detail: &str,
) -> Result<(), DumpValidationError> {
    let value =
        object
            .get(key)
            .and_then(Value::as_str)
            .ok_or_else(|| DumpValidationError::WrongType {
                field: display_field.to_string(),
                detail: "expected string".to_string(),
            })?;
    if valid_values.contains(&value) {
        Ok(())
    } else {
        Err(DumpValidationError::WrongType {
            field: display_field.to_string(),
            detail: detail.to_string(),
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

fn require_nullable_u64_at(
    object: &serde_json::Map<String, Value>,
    display_field: &str,
    key: &str,
) -> Result<(), DumpValidationError> {
    match object.get(key) {
        Some(Value::Null) => Ok(()),
        Some(Value::Number(number)) if number.as_u64().is_some() => Ok(()),
        Some(_) => Err(DumpValidationError::WrongType {
            field: display_field.to_string(),
            detail: "expected non-negative integer or null".to_string(),
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

fn validate_positions_map(map: &Value, display_field: &str) -> Result<(), DumpValidationError> {
    let object = map
        .as_object()
        .ok_or_else(|| DumpValidationError::WrongType {
            field: display_field.to_string(),
            detail: "expected object".to_string(),
        })?;

    for key in POSITION_KEYS {
        if !object.contains_key(*key) {
            return Err(DumpValidationError::MissingField(format!(
                "{display_field}.{key}"
            )));
        }
    }

    for (key, value) in object {
        if !POSITION_KEYS.contains(&key.as_str()) {
            return Err(DumpValidationError::WrongType {
                field: format!("{display_field}.{key}"),
                detail: "unexpected position key".to_string(),
            });
        }

        if !value.is_null()
            && !value
                .as_i64()
                .is_some_and(|value| (0..=20).contains(&value))
        {
            return Err(DumpValidationError::WrongType {
                field: format!("{display_field}.{key}"),
                detail: "expected integer from 0 to 20 or null".to_string(),
            });
        }
    }

    Ok(())
}

fn validate_fixed_staff_attribute_map(
    map: &Value,
    display_field: &str,
) -> Result<(), DumpValidationError> {
    let object = map
        .as_object()
        .ok_or_else(|| DumpValidationError::WrongType {
            field: display_field.to_string(),
            detail: "expected object".to_string(),
        })?;

    for key in STAFF_ATTRIBUTE_KEYS {
        if !object.contains_key(*key) {
            return Err(DumpValidationError::MissingField(format!(
                "{display_field}.{key}"
            )));
        }
    }

    for (key, value) in object {
        if !STAFF_ATTRIBUTE_KEYS.contains(&key.as_str()) {
            return Err(DumpValidationError::WrongType {
                field: format!("{display_field}.{key}"),
                detail: "unexpected staff attribute".to_string(),
            });
        }
        if !value.is_null()
            && !value
                .as_i64()
                .is_some_and(|value| (1..=20).contains(&value))
        {
            return Err(DumpValidationError::WrongType {
                field: format!("{display_field}.{key}"),
                detail: "expected integer from 1 to 20 or null".to_string(),
            });
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

    const GOLDEN_FIXTURE: &str = include_str!("fixtures/golden_dump_v8.json");
    const STALE_V7_FIXTURE: &str = include_str!("fixtures/golden_dump_v7.json");
    const STALE_V6_FIXTURE: &str = include_str!("fixtures/golden_dump_v6.json");
    const STALE_V5_FIXTURE: &str = include_str!("fixtures/golden_dump_v5.json");

    fn fixture_with_positions(positions: Value) -> String {
        let mut root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse v8 fixture");
        root["players"][0]["positions"] = positions;
        root.to_string()
    }

    #[test]
    fn golden_fixture_passes_ingestibility_validation() {
        validate_dump_json(GOLDEN_FIXTURE).expect("golden dump v8 should be ingestible");
    }

    #[test]
    fn rejects_stale_schema_v7_with_plugin_update_and_rescan_instruction() {
        let error = validate_dump_json(STALE_V7_FIXTURE).expect_err("stale schema v7");

        assert!(error
            .to_string()
            .contains("update the FM bridge plugin and rescan"));
        assert!(matches!(
            error,
            DumpValidationError::UnsupportedSchemaVersion {
                found: 7,
                expected: 8
            }
        ));
    }

    #[test]
    fn rejects_stale_schema_v6_with_plugin_update_and_rescan_instruction() {
        let error = validate_dump_json(STALE_V6_FIXTURE).expect_err("stale schema v6");

        assert!(error
            .to_string()
            .contains("update the FM bridge plugin and rescan"));
        assert!(matches!(
            error,
            DumpValidationError::UnsupportedSchemaVersion {
                found: 6,
                expected: 8
            }
        ));
    }

    #[test]
    fn rejects_stale_schema_v5_with_plugin_update_and_rescan_instruction() {
        let error = validate_dump_json(STALE_V5_FIXTURE).expect_err("stale schema v5");

        assert!(error
            .to_string()
            .contains("update the FM bridge plugin and rescan"));
        assert!(matches!(
            error,
            DumpValidationError::UnsupportedSchemaVersion {
                found: 5,
                expected: 8
            }
        ));
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let json = GOLDEN_FIXTURE.replace("\"schemaVersion\": 8", "\"schemaVersion\": 4");

        let error = validate_dump_json(&json).expect_err("schema v4");

        assert!(matches!(
            error,
            DumpValidationError::UnsupportedSchemaVersion {
                found: 4,
                expected: 8
            }
        ));
    }

    #[test]
    fn rejects_positions_with_a_missing_canonical_key() {
        let mut positions = serde_json::from_str::<Value>(
            r#"{"GK":null,"SW":null,"DL":null,"DC":null,"DR":null,"DM":null,"ML":null,"MC":0,"MR":17,"AML":null,"AMC":14,"AMR":20,"ST":null,"WBL":null}"#,
        )
        .expect("positions");
        let json = fixture_with_positions(positions.take());

        let error = validate_dump_json(&json).expect_err("missing WBR");

        assert!(matches!(
            error,
            DumpValidationError::MissingField(field) if field == "players[0].positions.WBR"
        ));
    }

    #[test]
    fn rejects_positions_with_an_extra_key() {
        let mut positions = serde_json::from_str::<Value>(
            r#"{"GK":null,"SW":null,"DL":null,"DC":null,"DR":null,"DM":null,"ML":null,"MC":0,"MR":17,"AML":null,"AMC":14,"AMR":20,"ST":null,"WBL":null,"WBR":null,"CB":15}"#,
        )
        .expect("positions");
        let json = fixture_with_positions(positions.take());

        let error = validate_dump_json(&json).expect_err("extra CB");

        assert!(matches!(
            error,
            DumpValidationError::WrongType { field, .. } if field == "players[0].positions.CB"
        ));
    }

    #[test]
    fn rejects_positions_outside_the_inclusive_zero_to_twenty_range() {
        for (key, value) in [("MC", -1), ("MR", 21)] {
            let mut positions: Value = serde_json::from_str(
                r#"{"GK":null,"SW":null,"DL":null,"DC":null,"DR":null,"DM":null,"ML":null,"MC":0,"MR":17,"AML":null,"AMC":14,"AMR":20,"ST":null,"WBL":null,"WBR":null}"#,
            )
            .expect("positions");
            positions[key] = Value::from(value);

            let error = validate_dump_json(&fixture_with_positions(positions))
                .expect_err("out-of-range position");

            assert!(matches!(
                error,
                DumpValidationError::WrongType { field, .. } if field == format!("players[0].positions.{key}")
            ));
        }
    }

    #[test]
    fn rejects_fractional_and_string_position_values() {
        for value in [Value::from(1.5), Value::from("20"), Value::Bool(true)] {
            let mut positions: Value = serde_json::from_str(
                r#"{"GK":null,"SW":null,"DL":null,"DC":null,"DR":null,"DM":null,"ML":null,"MC":0,"MR":17,"AML":null,"AMC":14,"AMR":20,"ST":null,"WBL":null,"WBR":null}"#,
            )
            .expect("positions");
            positions["AMC"] = value;

            let error = validate_dump_json(&fixture_with_positions(positions))
                .expect_err("invalid position value");

            assert!(matches!(
                error,
                DumpValidationError::WrongType { field, .. } if field == "players[0].positions.AMC"
            ));
        }
    }

    #[test]
    fn rejects_missing_required_top_level_key() {
        let json = GOLDEN_FIXTURE.replace("\"gameDateSource\": \"derived\",", "");

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
  "schemaVersion": 8,
  "generatedAtUtc": "2026-07-29T10:00:00.000Z",
  "gameVersion": "26.3.2",
  "supportedGameVersion": "26.3",
  "bridgeVersion": "0.1.0",
  "protocolVersion": 1,
  "gameDateSource": "unknown",
  "gameDateBasis": "unknown",
  "playerDatabaseScope": "men",
  "scanTruncated": false,
  "maxAccepted": null,
  "playerCount": 0,
  "players": [],
  "staffCount": 0,
  "staff": [],
  "manager": null
}"#;

        let error = validate_dump_json(json).expect_err("empty players");

        assert!(matches!(error, DumpValidationError::EmptyPlayers));
    }

    #[test]
    fn accepts_explicit_empty_save_marker() {
        let json = r#"{
  "schemaVersion": 8,
  "generatedAtUtc": "2026-07-29T10:00:00.000Z",
  "gameVersion": "26.3.2",
  "supportedGameVersion": "26.3",
  "bridgeVersion": "0.1.0",
  "protocolVersion": 1,
  "gameDateSource": "unknown",
  "gameDateBasis": "unknown",
  "playerDatabaseScope": "men",
  "scanTruncated": false,
  "maxAccepted": null,
  "emptySave": true,
  "playerCount": 0,
  "players": [],
  "staffCount": 0,
  "staff": [],
  "manager": null
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
            "\"gameDateSource\": \"derived\"",
            "\"gameDateSource\": \"guess\"",
        );

        let error = validate_dump_json(&json).expect_err("invalid gameDateSource");

        assert!(matches!(
            error,
            DumpValidationError::WrongType { field, .. } if field == "gameDateSource"
        ));
    }

    #[test]
    fn rejects_a_noncanonical_game_date() {
        let json = GOLDEN_FIXTURE.replace(
            "\"gameDate\": \"2026-08-14\"",
            "\"gameDate\": \"2026-02-30\"",
        );

        let error = validate_dump_json(&json).expect_err("invalid game date");

        assert!(matches!(
            error,
            DumpValidationError::WrongType { field, .. } if field == "gameDate"
        ));
    }

    #[test]
    fn rejects_scan_truncated_true_without_max_accepted() {
        let json = GOLDEN_FIXTURE.replace("\"scanTruncated\": false", "\"scanTruncated\": true");

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
    fn rejects_staff_count_mismatch() {
        let json = GOLDEN_FIXTURE.replace("\"staffCount\": 1", "\"staffCount\": 2");

        let error = validate_dump_json(&json).expect_err("staff count mismatch");

        assert!(matches!(
            error,
            DumpValidationError::StaffCountMismatch {
                staff_count: 2,
                staff_len: 1
            }
        ));
    }

    #[test]
    fn rejects_staff_missing_fixed_scoring_attributes() {
        for key in ["Authority", "Adaptability"] {
            let mut root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse fixture");
            root["staff"][0]["attributes"]
                .as_object_mut()
                .expect("staff attributes")
                .remove(key);

            let error = validate_dump_json(&root.to_string()).expect_err("missing staff attribute");

            assert!(
                matches!(error, DumpValidationError::MissingField(field) if field == format!("staff[0].attributes.{key}"))
            );
        }
    }

    #[test]
    fn rejects_staff_scoring_attributes_outside_one_to_twenty() {
        for (key, value) in [("Authority", 0), ("Adaptability", 21)] {
            let mut root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse fixture");
            root["staff"][0]["attributes"][key] = Value::from(value);

            let error = validate_dump_json(&root.to_string()).expect_err("invalid staff attribute");

            assert!(
                matches!(error, DumpValidationError::WrongType { field, .. } if field == format!("staff[0].attributes.{key}"))
            );
        }
    }

    #[test]
    fn rejects_renamed_staff_attribute() {
        let mut root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse fixture");
        let attributes = root["staff"][0]["attributes"]
            .as_object_mut()
            .expect("staff attributes");
        let attacking = attributes.remove("Attacking").expect("Attacking attribute");
        attributes.insert("Attack".to_string(), attacking);

        let error = validate_dump_json(&root.to_string()).expect_err("renamed staff attribute");

        assert!(
            matches!(error, DumpValidationError::MissingField(field) if field == "staff[0].attributes.Attacking")
        );
    }

    #[test]
    fn rejects_unknown_staff_attribute() {
        let mut root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse fixture");
        root["staff"][0]["attributes"]
            .as_object_mut()
            .expect("staff attributes")
            .insert("Address".to_string(), Value::from(123));

        let error = validate_dump_json(&root.to_string()).expect_err("unknown staff attribute");

        assert!(
            matches!(error, DumpValidationError::WrongType { field, .. } if field == "staff[0].attributes.Address")
        );
    }

    #[test]
    fn rejects_duplicate_player_and_staff_uids() {
        for (entity, count_key) in [("players", "playerCount"), ("staff", "staffCount")] {
            let mut root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse fixture");
            let duplicate = root[entity][0].clone();
            root[entity]
                .as_array_mut()
                .expect("fixture array")
                .push(duplicate);
            root[count_key] = Value::from(2);

            let error = validate_dump_json(&root.to_string()).expect_err("duplicate uid");

            match (entity, error) {
                (
                    "players",
                    DumpValidationError::DuplicateUid {
                        entity: found,
                        uid: 77,
                    },
                ) => {
                    assert_eq!(found, "player");
                }
                (
                    "staff",
                    DumpValidationError::DuplicateUid {
                        entity: found,
                        uid: 88,
                    },
                ) => {
                    assert_eq!(found, "staff");
                }
                (_, error) => panic!("unexpected duplicate validation error: {error:?}"),
            }
        }
    }

    #[test]
    fn rejects_player_and_staff_uid_overlap() {
        let mut root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse fixture");
        root["staff"][0]["uid"] = Value::from(77);
        root["manager"]["uid"] = Value::from(77);

        let error =
            validate_dump_json(&root.to_string()).expect_err("player and staff uid overlap");

        assert!(matches!(
            error,
            DumpValidationError::PlayerStaffUidOverlap { uid: 77 }
        ));
    }

    #[test]
    fn rejects_manager_without_matching_staff_record() {
        let mut root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse fixture");
        root["manager"]["uid"] = Value::from(999);

        let error = validate_dump_json(&root.to_string()).expect_err("manager without staff");

        assert!(matches!(
            error,
            DumpValidationError::ManagerNotInStaff { uid: 999 }
        ));
    }

    #[test]
    fn rejects_invalid_scope_and_gender_values() {
        let invalid_scope = GOLDEN_FIXTURE.replace(
            "\"playerDatabaseScope\": \"men\"",
            "\"playerDatabaseScope\": \"mixed\"",
        );
        let scope_error = validate_dump_json(&invalid_scope).expect_err("invalid scope");
        assert!(matches!(
            scope_error,
            DumpValidationError::WrongType { field, .. } if field == "playerDatabaseScope"
        ));

        let invalid_gender =
            GOLDEN_FIXTURE.replace("\"gender\": \"male\"", "\"gender\": \"other\"");
        let gender_error = validate_dump_json(&invalid_gender).expect_err("invalid gender");
        assert!(matches!(
            gender_error,
            DumpValidationError::WrongType { field, .. } if field == "players[0].gender"
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
