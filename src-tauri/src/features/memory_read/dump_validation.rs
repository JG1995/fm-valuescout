use serde::de::{self, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde_json::error::Category;
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::io::{BufReader, Read};

pub const DUMP_SCHEMA_VERSION: i64 = 9;
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

/// Top-level fields a dump may omit.
const OPTIONAL_TOP_LEVEL_KEYS: &[&str] = &["gameDate", "emptySave"];

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
    "currentClubUid",
    "parentClub",
    "parentClubUid",
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
    "clubUid",
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
#[allow(dead_code)]
pub fn validate_dump_json(json: &str) -> Result<(), DumpValidationError> {
    read_validated_dump(json.as_bytes(), |_| Ok(()), |_| Ok(())).map(|_| ())
}

/// A dump that passed every validation rule: its known top-level fields other than the
/// record arrays, and each record as the caller converted it, in dump order.
pub(crate) struct ValidatedDump<P, S> {
    pub(crate) header: Map<String, Value>,
    pub(crate) players: Vec<P>,
    pub(crate) staff: Vec<S>,
}

/// Reads and validates a dump one record at a time, so the document is never held in memory.
///
/// Each header field is checked as soon as it is read, so an outdated bridge, which writes
/// `schemaVersion` first, fails before any record is examined. Each record is validated,
/// handed to its converter, and dropped. The checks that need later data (required fields,
/// counts, duplicate UIDs, and the manager) run when the root object closes. Nothing is
/// returned unless the whole document is valid. A conversion error, which validation makes
/// unreachable, is reported as a corrupt dump.
pub(crate) fn read_validated_dump<R: Read, P, S>(
    reader: R,
    mut convert_player: impl FnMut(&Map<String, Value>) -> Result<P, String>,
    mut convert_staff: impl FnMut(&Map<String, Value>) -> Result<S, String>,
) -> Result<ValidatedDump<P, S>, DumpValidationError> {
    let mut input = BufReader::new(ContentProbe {
        inner: reader,
        has_content: false,
    });
    let mut dump = DumpReader {
        convert_player: &mut convert_player,
        convert_staff: &mut convert_staff,
        header: Map::new(),
        players: None,
        staff: None,
        failure: None,
        expected_type: ("(root)", "expected JSON object"),
    };
    let parsed = {
        let mut deserializer = serde_json::Deserializer::from_reader(&mut input);
        serde::Deserializer::deserialize_map(&mut deserializer, &mut dump)
            .and_then(|()| deserializer.end())
    };
    if let Err(error) = parsed {
        if let Some(failure) = dump.failure {
            return Err(failure);
        }
        // A read failure keeps its own message, as reading the whole file did before.
        if error.classify() == Category::Io {
            return Err(DumpValidationError::Corrupt(error.to_string()));
        }
        if !input.get_ref().has_content {
            return Err(DumpValidationError::Corrupt("dump is empty".to_string()));
        }
        // Header fields and records parse into `Value` and unknown fields into `IgnoredAny`,
        // which accept any JSON, so a data error can only be the root or a record array
        // having the wrong JSON type.
        if error.classify() == Category::Data {
            let (field, detail) = dump.expected_type;
            return Err(DumpValidationError::WrongType {
                field: field.to_string(),
                detail: detail.to_string(),
            });
        }
        return Err(DumpValidationError::Corrupt(format!(
            "dump is not valid JSON: {error}"
        )));
    }
    dump.finish()
}

/// Records whether any non-whitespace byte was read, so an empty dump keeps its own message.
struct ContentProbe<R> {
    inner: R,
    has_content: bool,
}

impl<R: Read> Read for ContentProbe<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buf)?;
        if !self.has_content {
            self.has_content = buf[..read].iter().any(|byte| !byte.is_ascii_whitespace());
        }
        Ok(read)
    }
}

type Converter<'a, T> = &'a mut dyn FnMut(&Map<String, Value>) -> Result<T, String>;

/// One record array's validated records: their UIDs and converted values, in dump order.
struct Records<T> {
    uids: Vec<u64>,
    items: Vec<T>,
}

impl<T> Records<T> {
    fn new() -> Self {
        Self {
            uids: Vec::new(),
            items: Vec::new(),
        }
    }
}

struct DumpReader<'a, P, S> {
    convert_player: Converter<'a, P>,
    convert_staff: Converter<'a, S>,
    header: Map<String, Value>,
    players: Option<Records<P>>,
    staff: Option<Records<S>>,
    /// The validation error that stopped parsing.
    failure: Option<DumpValidationError>,
    /// The field whose JSON type the parser checks next, and the error detail for it.
    expected_type: (&'static str, &'static str),
}

/// Keeps `error` as the reason parsing stopped and returns a parser error that stops it.
fn stop<E: de::Error>(failure: &mut Option<DumpValidationError>, error: DumpValidationError) -> E {
    *failure = Some(error);
    E::custom("dump validation failed")
}

impl<'de, P, S> Visitor<'de> for &mut DumpReader<'_, P, S> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a dump object")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        while let Some(key) = map.next_key::<String>()? {
            let duplicate = match key.as_str() {
                "players" => self.players.is_some(),
                "staff" => self.staff.is_some(),
                _ => self.header.contains_key(&key),
            };
            if duplicate {
                return Err(stop(
                    &mut self.failure,
                    DumpValidationError::Corrupt(format!("duplicate top-level field `{key}`")),
                ));
            }
            match key.as_str() {
                "players" => {
                    self.expected_type = ("players", "expected array");
                    map.next_value_seed(RecordsSeed {
                        records: self.players.insert(Records::new()),
                        failure: &mut self.failure,
                        validate: validate_player_object,
                        convert: &mut *self.convert_player,
                    })?;
                }
                "staff" => {
                    self.expected_type = ("staff", "expected array");
                    map.next_value_seed(RecordsSeed {
                        records: self.staff.insert(Records::new()),
                        failure: &mut self.failure,
                        validate: validate_staff_object,
                        convert: &mut *self.convert_staff,
                    })?;
                }
                known
                    if REQUIRED_TOP_LEVEL_KEYS.contains(&known)
                        || OPTIONAL_TOP_LEVEL_KEYS.contains(&known) =>
                {
                    let value = map.next_value::<Value>()?;
                    self.header.insert(key.clone(), value);
                    check_header_field(&self.header, &key)
                        .map_err(|error| stop(&mut self.failure, error))?;
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(())
    }
}

impl<P, S> DumpReader<'_, P, S> {
    /// Runs the checks that need the whole document, in the order the dump has always been
    /// validated, then releases the records.
    fn finish(self) -> Result<ValidatedDump<P, S>, DumpValidationError> {
        let header = &self.header;
        check_schema_version(header)?;
        for key in REQUIRED_TOP_LEVEL_KEYS {
            let present = match *key {
                "players" => self.players.is_some(),
                "staff" => self.staff.is_some(),
                _ => header.contains_key(*key),
            };
            if !present {
                return Err(DumpValidationError::MissingField((*key).to_string()));
            }
        }
        check_protocol_version(header)?;
        require_non_empty_string(header, "generatedAtUtc")?;
        require_string(header, "gameVersion")?;
        require_string(header, "supportedGameVersion")?;
        require_string(header, "bridgeVersion")?;
        require_optional_game_date(header, "gameDate")?;
        require_game_date_source(header, "gameDateSource")?;
        require_game_date_basis(header, "gameDateBasis")?;
        require_player_database_scope(header, "playerDatabaseScope")?;
        check_scan_truncation(header)?;

        let player_count = require_count(header, "playerCount")?;
        let players = self.players.expect("required fields checked above");
        if player_count as usize != players.items.len() {
            return Err(DumpValidationError::PlayerCountMismatch {
                player_count,
                players_len: players.items.len(),
            });
        }

        let staff_count = require_count(header, "staffCount")?;
        let staff = self.staff.expect("required fields checked above");
        if staff_count as usize != staff.items.len() {
            return Err(DumpValidationError::StaffCountMismatch {
                staff_count,
                staff_len: staff.items.len(),
            });
        }

        let empty_save = header
            .get("emptySave")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        if players.items.is_empty() {
            if empty_save
                && player_count == 0
                && staff.items.is_empty()
                && header.get("manager") == Some(&Value::Null)
            {
                return Ok(ValidatedDump {
                    header: self.header,
                    players: players.items,
                    staff: staff.items,
                });
            }
            return Err(DumpValidationError::EmptyPlayers);
        }

        let mut player_uids = HashSet::with_capacity(players.uids.len());
        for &uid in &players.uids {
            if !player_uids.insert(uid) {
                return Err(DumpValidationError::DuplicateUid {
                    entity: "player".to_string(),
                    uid,
                });
            }
        }

        let mut staff_uids = HashSet::with_capacity(staff.uids.len());
        for &uid in &staff.uids {
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

        validate_manager(header.get("manager"), &staff_uids)?;

        Ok(ValidatedDump {
            header: self.header,
            players: players.items,
            staff: staff.items,
        })
    }
}

/// Streams one record array: each element is validated, converted, and dropped in turn.
struct RecordsSeed<'a, T> {
    records: &'a mut Records<T>,
    failure: &'a mut Option<DumpValidationError>,
    validate: fn(&Value, usize) -> Result<u64, DumpValidationError>,
    convert: Converter<'a, T>,
}

impl<'de, T> DeserializeSeed<'de> for RecordsSeed<'_, T> {
    type Value = ();

    fn deserialize<D: de::Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, T> Visitor<'de> for RecordsSeed<'_, T> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an array of records")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        while let Some(record) = seq.next_element::<Value>()? {
            let index = self.records.items.len();
            let converted = (self.validate)(&record, index).and_then(|uid| {
                let object = record.as_object().expect("validated records are objects");
                (self.convert)(object)
                    .map(|item| (uid, item))
                    .map_err(DumpValidationError::Corrupt)
            });
            let (uid, item) = converted.map_err(|error| stop(self.failure, error))?;
            self.records.uids.push(uid);
            self.records.items.push(item);
        }
        Ok(())
    }
}

/// Checks one header field as soon as it is read, with the rule the whole document gets.
/// The manager is checked when the root closes, because it must name a staff record.
fn check_header_field(header: &Map<String, Value>, key: &str) -> Result<(), DumpValidationError> {
    match key {
        "schemaVersion" => check_schema_version(header),
        "protocolVersion" => check_protocol_version(header),
        "generatedAtUtc" => require_non_empty_string(header, key),
        "gameVersion" | "supportedGameVersion" | "bridgeVersion" => require_string(header, key),
        "gameDate" => require_optional_game_date(header, key),
        "gameDateSource" => require_game_date_source(header, key),
        "gameDateBasis" => require_game_date_basis(header, key),
        "playerDatabaseScope" => require_player_database_scope(header, key),
        "scanTruncated" | "maxAccepted" => check_scan_truncation(header),
        "playerCount" | "staffCount" => require_count(header, key).map(|_| ()),
        _ => Ok(()),
    }
}

fn check_schema_version(header: &Map<String, Value>) -> Result<(), DumpValidationError> {
    let schema_version = require_i64(header, "schemaVersion")?;
    if schema_version != DUMP_SCHEMA_VERSION {
        return Err(DumpValidationError::UnsupportedSchemaVersion {
            found: schema_version,
            expected: DUMP_SCHEMA_VERSION,
        });
    }
    Ok(())
}

fn check_protocol_version(header: &Map<String, Value>) -> Result<(), DumpValidationError> {
    let protocol_version = require_i64(header, "protocolVersion")?;
    if protocol_version != DUMP_PROTOCOL_VERSION {
        return Err(DumpValidationError::UnsupportedProtocolVersion {
            found: protocol_version,
            expected: DUMP_PROTOCOL_VERSION,
        });
    }
    Ok(())
}

/// Checks whichever of `scanTruncated` and `maxAccepted` has been read, and their combined
/// rule once both have.
fn check_scan_truncation(header: &Map<String, Value>) -> Result<(), DumpValidationError> {
    let scan_truncated = match header.get("scanTruncated") {
        Some(_) => require_bool_value(header, "scanTruncated")?,
        None => false,
    };
    let Some(max_accepted) = header.get("maxAccepted") else {
        return Ok(());
    };
    require_nullable_non_negative_i64(header, "maxAccepted")?;
    if scan_truncated && max_accepted.is_null() {
        return Err(DumpValidationError::WrongType {
            field: "maxAccepted".to_string(),
            detail: "must be a non-negative number when scanTruncated is true".to_string(),
        });
    }
    Ok(())
}

fn require_count(header: &Map<String, Value>, field: &str) -> Result<i64, DumpValidationError> {
    let count = require_i64(header, field)?;
    if count < 0 {
        return Err(DumpValidationError::WrongType {
            field: field.to_string(),
            detail: "must be >= 0".to_string(),
        });
    }
    Ok(count)
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
    require_nullable_positive_u64_at(
        object,
        &format!("players[{index}].currentClubUid"),
        "currentClubUid",
    )?;
    require_nullable_positive_u64_at(
        object,
        &format!("players[{index}].parentClubUid"),
        "parentClubUid",
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
    require_nullable_positive_u64_at(object, &format!("staff[{index}].clubUid"), "clubUid")?;
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
    require_nullable_positive_u64_at(object, "manager.clubUid", "clubUid")?;
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

fn require_nullable_positive_u64_at(
    object: &serde_json::Map<String, Value>,
    display_field: &str,
    key: &str,
) -> Result<(), DumpValidationError> {
    match object.get(key) {
        Some(Value::Null) => Ok(()),
        Some(Value::Number(number)) if number.as_u64().is_some_and(|value| value > 0) => Ok(()),
        Some(_) => Err(DumpValidationError::WrongType {
            field: display_field.to_string(),
            detail: "expected positive integer or null".to_string(),
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

    const GOLDEN_FIXTURE: &str = include_str!("fixtures/golden_dump_v9.json");
    const STALE_V8_FIXTURE: &str = include_str!("fixtures/golden_dump_v8.json");
    const STALE_V7_FIXTURE: &str = include_str!("fixtures/golden_dump_v7.json");
    const STALE_V6_FIXTURE: &str = include_str!("fixtures/golden_dump_v6.json");
    const STALE_V5_FIXTURE: &str = include_str!("fixtures/golden_dump_v5.json");

    fn fixture_with_positions(positions: Value) -> String {
        let mut root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse v9 fixture");
        root["players"][0]["positions"] = positions;
        root.to_string()
    }

    #[test]
    fn golden_fixture_passes_ingestibility_validation() {
        validate_dump_json(GOLDEN_FIXTURE).expect("golden dump v9 should be ingestible");
    }

    #[test]
    fn rejects_malformed_documents_with_whole_document_messages() {
        let with_field = |field: &str, value: Value| {
            let mut root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse v9 fixture");
            root[field] = value;
            root.to_string()
        };
        let truncated = &GOLDEN_FIXTURE[..GOLDEN_FIXTURE.find("\"attributes\"").unwrap()];
        let trailing = format!("{GOLDEN_FIXTURE}x");
        let cases = [
            ("", "dump is empty"),
            (" \n", "dump is empty"),
            ("[]", "field `(root)` has wrong type: expected JSON object"),
            (
                "null",
                "field `(root)` has wrong type: expected JSON object",
            ),
            ("5", "field `(root)` has wrong type: expected JSON object"),
            (
                &with_field("players", serde_json::json!({})),
                "field `players` has wrong type: expected array",
            ),
            (
                &with_field("staff", Value::Null),
                "field `staff` has wrong type: expected array",
            ),
            (
                truncated,
                "dump is not valid JSON: EOF while parsing a value at line 46 column 6",
            ),
            (
                &trailing,
                "dump is not valid JSON: trailing characters at line 135 column 1",
            ),
        ];
        for (json, expected) in cases {
            let error = validate_dump_json(json).expect_err("malformed dump");
            assert_eq!(error.to_string(), expected, "input {json:?}");
        }
    }

    #[test]
    fn reports_a_failed_read_as_the_read_error() {
        struct FailingReader;
        impl Read for FailingReader {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("disk unavailable"))
            }
        }
        let error = read_validated_dump(FailingReader, |_| Ok(()), |_| Ok(()))
            .err()
            .expect("read failure");
        assert_eq!(
            error,
            DumpValidationError::Corrupt("disk unavailable".to_string())
        );
    }

    #[test]
    fn accepts_nullable_club_uids_and_rejects_non_positive_values() {
        let mut root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse v9 fixture");
        root["players"][0]["currentClubUid"] = Value::Null;
        root["players"][0]["parentClubUid"] = Value::Null;
        root["staff"][0]["clubUid"] = Value::Null;
        root["manager"]["clubUid"] = Value::Null;
        validate_dump_json(&root.to_string()).expect("null club UIDs should validate");

        for (path, value) in [
            ("currentClubUid", Value::from(0)),
            ("parentClubUid", Value::from(-1)),
        ] {
            let mut invalid = root.clone();
            invalid["players"][0][path] = value;
            let error =
                validate_dump_json(&invalid.to_string()).expect_err("non-positive club UID");
            assert!(
                matches!(error, DumpValidationError::WrongType { field, .. } if field == format!("players[0].{path}"))
            );
        }
    }

    #[test]
    fn rejects_stale_schema_v8_with_plugin_update_and_rescan_instruction() {
        // The v8 records fail today's record rules, so the version must be checked first.
        let stale: Value = serde_json::from_str(STALE_V8_FIXTURE).expect("parse v8 fixture");
        assert!(validate_player_object(&stale["players"][0], 0).is_err());

        let error = validate_dump_json(STALE_V8_FIXTURE).expect_err("stale schema v8");
        assert!(error
            .to_string()
            .contains("update the FM bridge plugin and rescan"));
        assert!(matches!(
            error,
            DumpValidationError::UnsupportedSchemaVersion {
                found: 8,
                expected: 9
            }
        ));
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
                expected: 9
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
                expected: 9
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
                expected: 9
            }
        ));
    }

    #[test]
    fn validates_a_dump_whose_header_follows_its_records() {
        let golden: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse v9 fixture");
        let header_last = |root: &Value| {
            let fields = root.as_object().expect("fixture object");
            let header = fields
                .iter()
                .filter(|(key, _)| !matches!(key.as_str(), "players" | "staff"))
                .map(|(key, value)| format!("{}:{value}", Value::from(key.as_str())))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                r#"{{"players":{},"staff":{},{header}}}"#,
                root["players"], root["staff"]
            )
        };

        validate_dump_json(&header_last(&golden)).expect("header after records validates");

        let mut stale = golden.clone();
        stale["schemaVersion"] = Value::from(8);
        assert!(matches!(
            validate_dump_json(&header_last(&stale)).expect_err("stale schema after records"),
            DumpValidationError::UnsupportedSchemaVersion {
                found: 8,
                expected: 9
            }
        ));

        let mut miscounted = golden;
        miscounted["staffCount"] = Value::from(2);
        assert!(matches!(
            validate_dump_json(&header_last(&miscounted)).expect_err("count after records"),
            DumpValidationError::StaffCountMismatch {
                staff_count: 2,
                staff_len: 1
            }
        ));
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let json = GOLDEN_FIXTURE.replace("\"schemaVersion\": 9", "\"schemaVersion\": 4");

        let error = validate_dump_json(&json).expect_err("schema v4");

        assert!(matches!(
            error,
            DumpValidationError::UnsupportedSchemaVersion {
                found: 4,
                expected: 9
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
        let mut root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse v9 fixture");
        root.as_object_mut()
            .expect("fixture object")
            .remove("scanTruncated");

        let error = validate_dump_json(&root.to_string()).expect_err("missing scanTruncated");

        assert!(
            matches!(error, DumpValidationError::MissingField(field) if field == "scanTruncated")
        );
    }

    #[test]
    fn rejects_empty_players_without_empty_save_marker() {
        let json = r#"{
  "schemaVersion": 9,
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
  "schemaVersion": 9,
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
        let mut root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse v9 fixture");
        root["players"][0]
            .as_object_mut()
            .expect("player object")
            .remove("reputation");
        let json = root.to_string();

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
}
