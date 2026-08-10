use std::collections::{BTreeMap, HashMap};

use csv::{ReaderBuilder, StringRecord, Trim};

use super::{
    CsvImportError, MoneyballAppearances, MoneyballMetricValue, MoneyballPlayer,
    MoneyballTransferValue, MoneyballWage, YouthTrackerAttribute, YouthTrackerHiddenAttribute,
    YouthTrackerPlayer,
};

const MONEYBALL_REQUIRED_HEADERS: &[&[&str]] = &[
    &["Unique ID"],
    &["Player"],
    &["Nation"],
    &["2nd Nat"],
    &["Club"],
    &["Position"],
    &["Age"],
    &["Height"],
    &["Left Foot"],
    &["Right Foot"],
    &["Transfer Value"],
    &["Wage"],
    &["Expires"],
    &["Appearances"],
    &["Minutes"],
    &["Goals"],
    &["Goals From Outside The Box"],
    &["xG"],
    &["NP-xG"],
    &["xG-OP"],
    &["xG/shot"],
    &["Shots"],
    &["Shots From Outside The Box Per 90 minutes"],
    &["Shots on Target", "ShT"],
    &["Penalties Taken", "Pens"],
    &["Penalties Scored", "Pens S"],
    &["Free Kick Shots"],
    &["Assists"],
    &["xA"],
    &["Chances Created per 90", "Ch C/90"],
    &["Clear Cut Chances Created", "CCC"],
    &["Key Passes", "Key"],
    &["Open Play Key Passes per 90", "OP-KP/90"],
    &["Crosses Attempted", "Cr A"],
    &["Crosses Completed", "Cr C"],
    &["Open Play Crosses Attempted", "OP-Crs A"],
    &["Open Play Crosses Completed", "OP-Crs C"],
    &["Passes Attempted", "Pas A"],
    &["Passes Completed", "Ps C", "Pas C"],
    &["PsP"],
    &["Dribbles", "Drb"],
    &["Distance"],
    &["Sprints/90"],
    &["Possession Lost per 90", "Poss Lost/90"],
    &["Tackles Attempted", "Tck A"],
    &["Tackled Completed", "Tackles Completed", "Tck C"],
    &["Key Tackles", "K Tck"],
    &["Interceptions", "Itc"],
    &["Possession Won per 90", "Poss Won/90"],
    &["Pres A"],
    &["Pres C"],
    &["Blk"],
    &["Shts Blckd"],
    &["Clearances"],
    &["Headers Attempted", "Hdrs A"],
    &["Headers Won", "Hdrs"],
    &["Headers Lost per 90", "Hdrs L/90"],
    &["Key Headers per 90", "K Hdrs/90"],
    &["Clean Sheets"],
    &["Goals Conceded"],
    &["Saves per 90", "Saves/90"],
    &["Expected Save Percentage", "xSv %"],
    &["xGP"],
    &["Saves Held", "Svh"],
    &["Saves Parried", "Svp"],
    &["Saves Tipped", "Svt"],
    &["Penalties Faced", "Pens Faced"],
    &["Penalties Saved", "Pens Saved"],
    &["Fouls Made"],
    &["Fouls Against"],
    &["Yellow Cards", "Yel"],
    &["Red cards"],
    &["Off"],
    &["Mistakes Leading to Goals", "MLG"],
    &["Rating"],
    &["Player of the Match", "PoM"],
    &["Games Won"],
    &["Games Drawn"],
    &["Games Lost"],
    &["Team Goals"],
];

const MONEYBALL_OPTIONAL_HEADERS: &[&[&str]] = &[
    &["Division"],
    &["CA"],
    &["PA"],
    &["Asking Price"],
    &["Save Percentage", "Sv %"],
];

const MONEYBALL_SIGNATURE_HEADERS: &[&str] = &[
    "Transfer Value",
    "Wage",
    "Expires",
    "Goals From Outside The Box",
    "xG",
    "NP-xG",
    "xG-OP",
    "xG/shot",
    "Shots From Outside The Box Per 90 minutes",
    "Expected Save Percentage",
    "Team Goals",
];

const MONEYBALL_COUNT_METRICS: &[&str] = &[
    "Goals",
    "Goals From Outside The Box",
    "Shots",
    "Shots on Target",
    "Penalties Taken",
    "Penalties Scored",
    "Free Kick Shots",
    "Assists",
    "Clear Cut Chances Created",
    "Key Passes",
    "Crosses Attempted",
    "Crosses Completed",
    "Open Play Crosses Attempted",
    "Open Play Crosses Completed",
    "Passes Attempted",
    "Passes Completed",
    "PsP",
    "Dribbles",
    "Tackles Attempted",
    "Tackled Completed",
    "Key Tackles",
    "Interceptions",
    "Pres A",
    "Pres C",
    "Blk",
    "Shts Blckd",
    "Clearances",
    "Headers Attempted",
    "Headers Won",
    "Clean Sheets",
    "Goals Conceded",
    "Saves Held",
    "Saves Parried",
    "Saves Tipped",
    "Penalties Faced",
    "Penalties Saved",
    "Fouls Made",
    "Fouls Against",
    "Yellow Cards",
    "Red cards",
    "Off",
    "Mistakes Leading to Goals",
    "Player of the Match",
    "Games Won",
    "Games Drawn",
    "Games Lost",
    "Team Goals",
];

const HIDDEN_ATTRIBUTE_HEADERS: &[(YouthTrackerHiddenAttribute, &str)] = &[
    (YouthTrackerHiddenAttribute::Ambition, "Ambition"),
    (YouthTrackerHiddenAttribute::Consistency, "Consistency"),
    (
        YouthTrackerHiddenAttribute::ImportantMatches,
        "Important Matches",
    ),
    (
        YouthTrackerHiddenAttribute::InjuryProneness,
        "Injury Proneness",
    ),
    (
        YouthTrackerHiddenAttribute::Professionalism,
        "Professionalism",
    ),
];

const ATTRIBUTE_HEADERS: &[(YouthTrackerAttribute, &str)] = &[
    (YouthTrackerAttribute::Corners, "Corners"),
    (YouthTrackerAttribute::Crossing, "Crossing"),
    (YouthTrackerAttribute::Dribbling, "Dribbling"),
    (YouthTrackerAttribute::Finishing, "Finishing"),
    (YouthTrackerAttribute::FirstTouch, "First Touch"),
    (YouthTrackerAttribute::FreeKickTaking, "Free Kick Taking"),
    (YouthTrackerAttribute::Heading, "Heading"),
    (YouthTrackerAttribute::LongShots, "Long Shots"),
    (YouthTrackerAttribute::LongThrows, "Long Throws"),
    (YouthTrackerAttribute::Marking, "Marking"),
    (YouthTrackerAttribute::Passing, "Passing"),
    (YouthTrackerAttribute::PenaltyTaking, "Penalty Taking"),
    (YouthTrackerAttribute::Tackling, "Tackling"),
    (YouthTrackerAttribute::Technique, "Technique"),
    (YouthTrackerAttribute::Aggression, "Aggression"),
    (YouthTrackerAttribute::Anticipation, "Anticipation"),
    (YouthTrackerAttribute::Bravery, "Bravery"),
    (YouthTrackerAttribute::Composure, "Composure"),
    (YouthTrackerAttribute::Concentration, "Concentration"),
    (YouthTrackerAttribute::Decisions, "Decisions"),
    (YouthTrackerAttribute::Flair, "Flair"),
    (YouthTrackerAttribute::Leadership, "Leadership"),
    (YouthTrackerAttribute::OffTheBall, "Off The Ball"),
    (YouthTrackerAttribute::Positioning, "Positioning"),
    (YouthTrackerAttribute::TeamWork, "Team Work"),
    (YouthTrackerAttribute::Vision, "Vision"),
    (YouthTrackerAttribute::WorkRate, "Work Rate"),
    (YouthTrackerAttribute::Acceleration, "Acceleration"),
    (YouthTrackerAttribute::Agility, "Agility"),
    (YouthTrackerAttribute::Balance, "Balance"),
    (YouthTrackerAttribute::JumpingReach, "Jumping Reach"),
    (YouthTrackerAttribute::NaturalFitness, "Natural Fitness"),
    (YouthTrackerAttribute::Pace, "Pace"),
    (YouthTrackerAttribute::Stamina, "Stamina"),
    (YouthTrackerAttribute::Strength, "Strength"),
    (YouthTrackerAttribute::AerialReach, "Aerial Reach"),
    (YouthTrackerAttribute::CommandOfArea, "Command Of Area"),
    (YouthTrackerAttribute::Communication, "Communication"),
    (YouthTrackerAttribute::Eccentricity, "Eccentricity"),
    (YouthTrackerAttribute::Handling, "Handling"),
    (YouthTrackerAttribute::Kicking, "Kicking"),
    (YouthTrackerAttribute::OneOnOnes, "One On Ones"),
    (YouthTrackerAttribute::Punching, "Punching"),
    (YouthTrackerAttribute::Reflexes, "Reflexes"),
    (
        YouthTrackerAttribute::RushingOutTendency,
        "Rushing Out (Tendency)",
    ),
    (YouthTrackerAttribute::Throwing, "Throwing"),
];

type AttributeColumn<T> = (T, &'static str, Option<usize>);

#[derive(Debug)]
struct HeaderMap {
    uid: usize,
    name: usize,
    age: Option<usize>,
    best_position: Option<usize>,
    positions: Option<usize>,
    nationality: Option<usize>,
    ca: Option<usize>,
    pa: Option<usize>,
    hidden_attributes: Vec<AttributeColumn<YouthTrackerHiddenAttribute>>,
    height: Option<usize>,
    determination: Option<usize>,
    personality: Option<usize>,
    preferred_foot: Option<usize>,
    all_time_appearances: Option<usize>,
    international_appearances: Option<usize>,
    all_time_goals: Option<usize>,
    assists: Option<usize>,
    attributes: Vec<AttributeColumn<YouthTrackerAttribute>>,
}

impl HeaderMap {
    fn from_headers(headers: &StringRecord) -> Result<Self, CsvImportError> {
        let headers = headers.iter().map(normalize_header).collect::<Vec<_>>();
        let uid = find_header(&headers, &["Unique ID", "UID", "Id"], "Unique ID")?
            .ok_or(CsvImportError::MissingRequiredHeader("Unique ID"))?;
        let name = find_header(&headers, &["Player", "Name", "Full Name"], "Player")?
            .ok_or(CsvImportError::MissingRequiredHeader("Player"))?;

        Ok(Self {
            uid,
            name,
            age: find_header(&headers, &["Age"], "Age")?,
            best_position: find_preferred_header(
                &headers,
                &[&["Best Pos", "Best Position"], &["Position"], &["Pos"]],
                "Best Position",
            )?,
            positions: find_header(&headers, &["Position"], "Position")?,
            nationality: find_header(&headers, &["Nation", "Nationality"], "Nation")?,
            ca: find_header(&headers, &["CA", "Current Ability"], "Current Ability")?,
            pa: find_header(
                &headers,
                &["PA", "Potential Ability", "Potential"],
                "Potential Ability",
            )?,
            hidden_attributes: attribute_columns(&headers, HIDDEN_ATTRIBUTE_HEADERS)?,
            height: find_header(&headers, &["Height"], "Height")?,
            determination: find_header(&headers, &["Determination", "Det"], "Determination")?,
            personality: find_header(&headers, &["Personality"], "Personality")?,
            preferred_foot: find_header(&headers, &["Preferred Foot", "Foot"], "Preferred Foot")?,
            all_time_appearances: find_preferred_header(
                &headers,
                &[
                    &["AT Apps"],
                    &["Apps", "Appearances", "Games"],
                    &["All Time Apps", "Career Apps"],
                ],
                "All-time appearances",
            )?,
            international_appearances: find_preferred_header(
                &headers,
                &[
                    &["Int Apps"],
                    &["Caps", "International Caps", "Intl Caps"],
                    &["International Apps"],
                ],
                "International appearances",
            )?,
            all_time_goals: find_preferred_header(
                &headers,
                &[
                    &["AT Gls"],
                    &["Goals", "Gls", "Scored"],
                    &["All Time Goals", "Career Goals"],
                ],
                "All-time goals",
            )?,
            assists: find_header(&headers, &["Assists", "Ast"], "Assists")?,
            attributes: attribute_columns(&headers, ATTRIBUTE_HEADERS)?,
        })
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn parse_youth_tracker(input: &str) -> Result<Vec<YouthTrackerPlayer>, CsvImportError> {
    parse_youth_tracker_with_row_limit(input, None)
}

fn parse_youth_tracker_with_row_limit(
    input: &str,
    max_rows: Option<usize>,
) -> Result<Vec<YouthTrackerPlayer>, CsvImportError> {
    let input = input.strip_prefix('﻿').unwrap_or(input);
    if input.trim().is_empty() {
        return Err(CsvImportError::EmptyInput);
    }

    let delimiter = detect_delimiter(input)?;
    let mut reader = csv_reader(input, delimiter);
    let headers = reader
        .headers()
        .map_err(|_| CsvImportError::MalformedCsv { row: 1 })?
        .clone();
    let columns = HeaderMap::from_headers(&headers)?;

    let mut seen_uids = HashMap::new();
    let mut players = Vec::new();

    for (record_index, record) in reader.records().enumerate() {
        if let Some(limit) = max_rows.filter(|limit| players.len() >= *limit) {
            return Err(CsvImportError::TooManyRows { limit });
        }
        let row = record_index + 2;
        let record = record.map_err(|_| CsvImportError::MalformedCsv { row })?;
        let player = parse_player(&record, &columns, row)?;

        if let Some(first_row) = seen_uids.insert(player.uid, row) {
            return Err(CsvImportError::DuplicateUid { first_row, row });
        }

        players.push(player);
    }

    Ok(players)
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ParsedCsv {
    YouthTracker(Vec<YouthTrackerPlayer>),
    Moneyball(Vec<MoneyballPlayer>),
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn parse_csv(input: &str) -> Result<ParsedCsv, CsvImportError> {
    parse_csv_with_row_limit(input, None)
}

pub(crate) fn parse_csv_with_row_limit(
    input: &str,
    max_rows: Option<usize>,
) -> Result<ParsedCsv, CsvImportError> {
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    if input.trim().is_empty() {
        return Err(CsvImportError::EmptyInput);
    }

    let delimiter = detect_delimiter(input)?;
    let headers = csv_reader(input, delimiter)
        .headers()
        .map_err(|_| CsvImportError::MalformedCsv { row: 1 })?
        .clone();
    if is_moneyball_candidate(&headers) {
        if delimiter != b';' {
            return Err(CsvImportError::UnsupportedDialect);
        }
        return parse_moneyball_with_row_limit(input, max_rows).map(ParsedCsv::Moneyball);
    }

    parse_youth_tracker_with_row_limit(input, max_rows).map(ParsedCsv::YouthTracker)
}

#[derive(Debug)]
struct MoneyballHeaderMap {
    columns: BTreeMap<&'static str, usize>,
}

impl MoneyballHeaderMap {
    fn from_headers(headers: &StringRecord) -> Result<Self, CsvImportError> {
        let mut columns = BTreeMap::new();
        for aliases in MONEYBALL_REQUIRED_HEADERS {
            let label = aliases[0];
            let column = find_moneyball_header(headers, aliases, label)?
                .ok_or(CsvImportError::MissingRequiredHeader(label))?;
            columns.insert(label, column);
        }
        for aliases in MONEYBALL_OPTIONAL_HEADERS {
            let label = aliases[0];
            if let Some(column) = find_moneyball_header(headers, aliases, label)? {
                columns.insert(label, column);
            }
        }
        Ok(Self { columns })
    }

    fn value<'a>(&self, record: &'a StringRecord, label: &'static str) -> &'a str {
        value(record, self.columns.get(label).copied())
    }
}

fn find_moneyball_header(
    headers: &StringRecord,
    aliases: &[&str],
    label: &'static str,
) -> Result<Option<usize>, CsvImportError> {
    let matches = headers
        .iter()
        .enumerate()
        .filter_map(|(index, header)| aliases.contains(&header).then_some(index))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => Ok(None),
        [index] => Ok(Some(*index)),
        _ => Err(CsvImportError::DuplicateHeader(label)),
    }
}

fn is_moneyball_candidate(headers: &StringRecord) -> bool {
    MONEYBALL_SIGNATURE_HEADERS
        .iter()
        .filter(|header| headers.iter().any(|value| value == **header))
        .count()
        >= 2
}

fn parse_moneyball_with_row_limit(
    input: &str,
    max_rows: Option<usize>,
) -> Result<Vec<MoneyballPlayer>, CsvImportError> {
    let mut reader = csv_reader(input, b';');
    let headers = reader
        .headers()
        .map_err(|_| CsvImportError::MalformedCsv { row: 1 })?
        .clone();
    let columns = MoneyballHeaderMap::from_headers(&headers)?;
    let mut seen_uids = HashMap::new();
    let mut players = Vec::new();

    for (record_index, record) in reader.records().enumerate() {
        if let Some(limit) = max_rows.filter(|limit| players.len() >= *limit) {
            return Err(CsvImportError::TooManyRows { limit });
        }
        let row = record_index + 2;
        let record = record.map_err(|_| CsvImportError::MalformedCsv { row })?;
        let player = parse_moneyball_player(&record, &columns, row)?;
        if let Some(first_row) = seen_uids.insert(player.uid, row) {
            return Err(CsvImportError::DuplicateUid { first_row, row });
        }
        players.push(player);
    }
    Ok(players)
}

fn parse_moneyball_player(
    record: &StringRecord,
    columns: &MoneyballHeaderMap,
    row: usize,
) -> Result<MoneyballPlayer, CsvImportError> {
    let mut metrics = BTreeMap::new();
    for header in MONEYBALL_COUNT_METRICS {
        metrics.insert(
            (*header).to_string(),
            parse_optional_u32(columns.value(record, header), row, header)?
                .map(MoneyballMetricValue::Count),
        );
    }
    for header in moneyball_decimal_metrics() {
        metrics.insert(
            (*header).to_string(),
            parse_optional_decimal(columns.value(record, header), row, header)?
                .map(MoneyballMetricValue::Decimal),
        );
    }

    Ok(MoneyballPlayer {
        uid: parse_uid(columns.value(record, "Unique ID"), row)?,
        name: optional_text(columns.value(record, "Player")),
        nation: optional_text(columns.value(record, "Nation")),
        second_nation: optional_text(columns.value(record, "2nd Nat")),
        club: optional_text(columns.value(record, "Club")),
        division: optional_text(columns.value(record, "Division")),
        position: optional_text(columns.value(record, "Position")),
        age: parse_optional_u8(columns.value(record, "Age"), row, "Age")?,
        height_centimeters: parse_optional_height(columns.value(record, "Height"), row)?,
        left_foot: optional_text(columns.value(record, "Left Foot")),
        right_foot: optional_text(columns.value(record, "Right Foot")),
        ca: parse_optional_ability(columns.value(record, "CA"), row, "CA")?,
        pa: parse_optional_ability(columns.value(record, "PA"), row, "PA")?,
        transfer_value: parse_optional_transfer_value(
            columns.value(record, "Transfer Value"),
            row,
            "Transfer Value",
        )?,
        asking_price: parse_optional_transfer_value(
            columns.value(record, "Asking Price"),
            row,
            "Asking Price",
        )?,
        wage: parse_optional_wage(columns.value(record, "Wage"), row)?,
        expires: optional_text(columns.value(record, "Expires")),
        appearances: parse_optional_moneyball_appearances(
            columns.value(record, "Appearances"),
            row,
        )?,
        minutes: parse_optional_u32(columns.value(record, "Minutes"), row, "Minutes")?,
        distance_kilometers: parse_optional_distance(columns.value(record, "Distance"), row)?,
        metrics,
    })
}

fn moneyball_decimal_metrics() -> &'static [&'static str] {
    &[
        "xG",
        "NP-xG",
        "xG-OP",
        "xG/shot",
        "Shots From Outside The Box Per 90 minutes",
        "xA",
        "Chances Created per 90",
        "Open Play Key Passes per 90",
        "Sprints/90",
        "Possession Lost per 90",
        "Possession Won per 90",
        "Headers Lost per 90",
        "Key Headers per 90",
        "Saves per 90",
        "Save Percentage",
        "Expected Save Percentage",
        "xGP",
        "Rating",
    ]
}

fn csv_reader(input: &str, delimiter: u8) -> csv::Reader<&[u8]> {
    ReaderBuilder::new()
        .delimiter(delimiter)
        .trim(Trim::All)
        .flexible(false)
        .from_reader(input.as_bytes())
}

fn detect_delimiter(input: &str) -> Result<u8, CsvImportError> {
    let semicolon_columns = header_width(input, b';')?;
    let comma_columns = header_width(input, b',')?;

    match semicolon_columns.cmp(&comma_columns) {
        std::cmp::Ordering::Greater if semicolon_columns > 1 => Ok(b';'),
        std::cmp::Ordering::Less if comma_columns > 1 => Ok(b','),
        _ => Err(CsvImportError::UnsupportedDialect),
    }
}

fn header_width(input: &str, delimiter: u8) -> Result<usize, CsvImportError> {
    let mut reader = csv_reader(input, delimiter);
    reader
        .headers()
        .map(StringRecord::len)
        .map_err(|_| CsvImportError::MalformedCsv { row: 1 })
}

fn parse_player(
    record: &StringRecord,
    columns: &HeaderMap,
    row: usize,
) -> Result<YouthTrackerPlayer, CsvImportError> {
    let uid = parse_uid(value(record, Some(columns.uid)), row)?;

    let hidden_attributes = columns
        .hidden_attributes
        .iter()
        .map(|(attribute, field, column)| {
            Ok((
                *attribute,
                parse_optional_rating(value(record, *column), row, field)?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, CsvImportError>>()?;

    let attributes = columns
        .attributes
        .iter()
        .map(|(attribute, field, column)| {
            Ok((
                *attribute,
                parse_optional_rating(value(record, *column), row, field)?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, CsvImportError>>()?;

    Ok(YouthTrackerPlayer {
        uid,
        name: optional_text(value(record, Some(columns.name))),
        age: parse_optional_u8(value(record, columns.age), row, "Age")?,
        best_position: optional_text(value(record, columns.best_position)),
        positions: optional_text(value(record, columns.positions)),
        nationality: optional_text(value(record, columns.nationality)),
        ca: parse_optional_ability(value(record, columns.ca), row, "Current Ability")?,
        pa: parse_optional_ability(value(record, columns.pa), row, "Potential Ability")?,
        hidden_attributes,
        height: optional_text(value(record, columns.height)),
        determination: parse_optional_rating(
            value(record, columns.determination),
            row,
            "Determination",
        )?,
        personality: optional_text(value(record, columns.personality)),
        preferred_foot: optional_text(value(record, columns.preferred_foot)),
        all_time_appearances: parse_optional_appearances(
            value(record, columns.all_time_appearances),
            row,
            "All-time appearances",
        )?,
        international_appearances: parse_optional_appearances(
            value(record, columns.international_appearances),
            row,
            "International appearances",
        )?,
        all_time_goals: parse_optional_u32(
            value(record, columns.all_time_goals),
            row,
            "All-time goals",
        )?,
        assists: parse_optional_u32(value(record, columns.assists), row, "Assists")?,
        attributes,
    })
}

fn value(record: &StringRecord, column: Option<usize>) -> &str {
    column
        .and_then(|column| record.get(column))
        .unwrap_or_default()
}

fn parse_uid(value: &str, row: usize) -> Result<u32, CsvImportError> {
    let uid = parse_optional_u32(value, row, "Unique ID")?.ok_or(CsvImportError::InvalidValue {
        row,
        field: "Unique ID",
        expected: "a positive u32",
    })?;

    if uid == 0 {
        return Err(CsvImportError::InvalidValue {
            row,
            field: "Unique ID",
            expected: "a positive u32",
        });
    }

    Ok(uid)
}

fn parse_optional_ability(
    value: &str,
    row: usize,
    field: &'static str,
) -> Result<Option<u8>, CsvImportError> {
    parse_optional_bounded_u8(value, row, field, 1, 200, "an integer from 1 to 200")
}

fn parse_optional_rating(
    value: &str,
    row: usize,
    field: &'static str,
) -> Result<Option<u8>, CsvImportError> {
    parse_optional_bounded_u8(value, row, field, 1, 20, "an integer from 1 to 20")
}

fn parse_optional_bounded_u8(
    value: &str,
    row: usize,
    field: &'static str,
    minimum: u8,
    maximum: u8,
    expected: &'static str,
) -> Result<Option<u8>, CsvImportError> {
    if is_unavailable(value) {
        return Ok(None);
    }

    let value = value
        .parse::<u8>()
        .map_err(|_| CsvImportError::InvalidValue {
            row,
            field,
            expected,
        })?;

    if !(minimum..=maximum).contains(&value) {
        return Err(CsvImportError::InvalidValue {
            row,
            field,
            expected,
        });
    }

    Ok(Some(value))
}

fn parse_optional_u8(
    value: &str,
    row: usize,
    field: &'static str,
) -> Result<Option<u8>, CsvImportError> {
    if is_unavailable(value) {
        return Ok(None);
    }

    value
        .parse::<u8>()
        .map(Some)
        .map_err(|_| CsvImportError::InvalidValue {
            row,
            field,
            expected: "a whole number",
        })
}

fn parse_optional_u32(
    value: &str,
    row: usize,
    field: &'static str,
) -> Result<Option<u32>, CsvImportError> {
    if is_unavailable(value) {
        return Ok(None);
    }

    value
        .parse::<u32>()
        .map(Some)
        .map_err(|_| CsvImportError::InvalidValue {
            row,
            field,
            expected: if field == "Unique ID" {
                "a positive u32"
            } else {
                "a non-negative whole number"
            },
        })
}

fn parse_optional_appearances(
    value: &str,
    row: usize,
    field: &'static str,
) -> Result<Option<u32>, CsvImportError> {
    if is_unavailable(value) {
        return Ok(None);
    }

    if let Some((starts, substitutes)) = value.split_once('(') {
        let substitutes =
            substitutes
                .trim()
                .strip_suffix(')')
                .ok_or(CsvImportError::InvalidValue {
                    row,
                    field,
                    expected: "a whole number or N (M)",
                })?;
        let starts = parse_required_u32(starts.trim(), row, field)?;
        let substitutes = parse_required_u32(substitutes.trim(), row, field)?;
        return starts
            .checked_add(substitutes)
            .map(Some)
            .ok_or(CsvImportError::InvalidValue {
                row,
                field,
                expected: "a whole number or N (M)",
            });
    }

    parse_required_u32(value, row, field).map(Some)
}

fn parse_required_u32(value: &str, row: usize, field: &'static str) -> Result<u32, CsvImportError> {
    value
        .parse::<u32>()
        .map_err(|_| CsvImportError::InvalidValue {
            row,
            field,
            expected: "a whole number or N (M)",
        })
}

fn parse_optional_decimal(
    value: &str,
    row: usize,
    field: &'static str,
) -> Result<Option<f64>, CsvImportError> {
    if is_unavailable(value) {
        return Ok(None);
    }
    value
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .map(Some)
        .ok_or(CsvImportError::InvalidValue {
            row,
            field,
            expected: "a dot-decimal number",
        })
}

fn parse_optional_height(value: &str, row: usize) -> Result<Option<u16>, CsvImportError> {
    if is_unavailable(value) {
        return Ok(None);
    }
    value
        .strip_suffix(" cm")
        .and_then(|value| value.parse::<u16>().ok())
        .map(Some)
        .ok_or(CsvImportError::InvalidValue {
            row,
            field: "Height",
            expected: "a centimeter value",
        })
}

fn parse_optional_distance(value: &str, row: usize) -> Result<Option<f64>, CsvImportError> {
    if is_unavailable(value) {
        return Ok(None);
    }
    value
        .strip_suffix("km")
        .and_then(|value| value.trim().parse::<f64>().ok())
        .filter(|value| value.is_finite())
        .map(Some)
        .ok_or(CsvImportError::InvalidValue {
            row,
            field: "Distance",
            expected: "a kilometer value",
        })
}

fn parse_optional_moneyball_appearances(
    value: &str,
    row: usize,
) -> Result<Option<MoneyballAppearances>, CsvImportError> {
    if is_unavailable(value) {
        return Ok(None);
    }
    let (starts, substitutes) = match value.split_once('(') {
        Some((starts, substitutes)) => (
            parse_required_u32(starts.trim(), row, "Appearances")?,
            parse_required_u32(
                substitutes
                    .trim()
                    .strip_suffix(')')
                    .ok_or(CsvImportError::InvalidValue {
                        row,
                        field: "Appearances",
                        expected: "a whole number or N (M)",
                    })?,
                row,
                "Appearances",
            )?,
        ),
        None => (parse_required_u32(value, row, "Appearances")?, 0),
    };
    Ok(Some(MoneyballAppearances {
        starts,
        substitutes,
    }))
}

fn parse_optional_transfer_value(
    value: &str,
    row: usize,
    field: &'static str,
) -> Result<Option<MoneyballTransferValue>, CsvImportError> {
    if is_unavailable(value) {
        return Ok(None);
    }
    if value == "Not for Sale" {
        return Ok(Some(MoneyballTransferValue::NotForSale));
    }
    let transfer_value = match value.split_once(" - ") {
        Some((lower, upper)) => MoneyballTransferValue::Range {
            lower_euros: parse_euro_amount(lower, row, field)?,
            upper_euros: parse_euro_amount(upper, row, field)?,
        },
        None => MoneyballTransferValue::Single {
            euros: parse_euro_amount(value, row, field)?,
        },
    };
    Ok(Some(transfer_value))
}

fn parse_optional_wage(value: &str, row: usize) -> Result<Option<MoneyballWage>, CsvImportError> {
    if is_unavailable(value) {
        return Ok(None);
    }
    let euros = value
        .strip_suffix(" p/w")
        .ok_or(CsvImportError::InvalidValue {
            row,
            field: "Wage",
            expected: "a euro weekly wage",
        })
        .and_then(|value| parse_euro_amount(value, row, "Wage"))?;
    Ok(Some(MoneyballWage {
        euros_per_week: euros,
    }))
}

fn parse_euro_amount(value: &str, row: usize, field: &'static str) -> Result<u64, CsvImportError> {
    let value = value
        .strip_prefix('€')
        .ok_or(CsvImportError::InvalidValue {
            row,
            field,
            expected: "a euro amount or range",
        })?;
    let (number, multiplier) = match value.chars().last() {
        Some('K') => (&value[..value.len() - 1], 1_000_u64),
        Some('M') => (&value[..value.len() - 1], 1_000_000_u64),
        _ => (value, 1_u64),
    };
    let (whole, fraction) = number.split_once('.').unwrap_or((number, ""));
    let whole = whole.parse::<u64>().ok();
    let fraction_euros = if fraction.is_empty() {
        Some(0)
    } else {
        fraction
            .len()
            .try_into()
            .ok()
            .and_then(|digits: u32| 10_u64.checked_pow(digits))
            .zip(fraction.parse::<u64>().ok())
            .and_then(|(scale, fraction)| {
                fraction
                    .checked_mul(multiplier)
                    .map(|scaled| (scale, scaled))
            })
            .and_then(|(scale, scaled)| (scaled % scale == 0).then_some(scaled / scale))
    };
    whole
        .and_then(|whole| whole.checked_mul(multiplier))
        .zip(fraction_euros)
        .and_then(|(whole, fraction)| whole.checked_add(fraction))
        .ok_or(CsvImportError::InvalidValue {
            row,
            field,
            expected: "a euro amount or range",
        })
}

fn optional_text(value: &str) -> Option<String> {
    (!is_unavailable(value)).then(|| value.to_string())
}

fn is_unavailable(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "" | "-" | "n/a" | "na" | "—"
    )
}

fn attribute_columns<T: Copy>(
    headers: &[String],
    definitions: &[(T, &'static str)],
) -> Result<Vec<AttributeColumn<T>>, CsvImportError> {
    definitions
        .iter()
        .map(|(attribute, header)| {
            Ok((
                *attribute,
                *header,
                find_header(headers, &[*header], header)?,
            ))
        })
        .collect()
}

fn find_preferred_header(
    headers: &[String],
    alternatives: &[&[&str]],
    label: &'static str,
) -> Result<Option<usize>, CsvImportError> {
    for aliases in alternatives {
        if let Some(index) = find_header(headers, aliases, label)? {
            return Ok(Some(index));
        }
    }

    Ok(None)
}

fn find_header(
    headers: &[String],
    aliases: &[&str],
    label: &'static str,
) -> Result<Option<usize>, CsvImportError> {
    let matches = headers
        .iter()
        .enumerate()
        .filter_map(|(index, header)| {
            aliases
                .iter()
                .any(|alias| header == &normalize_header(alias))
                .then_some(index)
        })
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [] => Ok(None),
        [index] => Ok(Some(*index)),
        _ => Err(CsvImportError::DuplicateHeader(label)),
    }
}

fn normalize_header(header: &str) -> String {
    header.trim().trim_start_matches('﻿').to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MONZA_EXPORT: &str = include_str!("fixtures/2030_07_01_Full_Squad_CA_PA_Monza.csv");
    const MONEYBALL_EXPORT: &str = include_str!("fixtures/moneyball_stats.csv");

    #[test]
    fn parses_the_pinned_moneyball_export_before_youth_tracker() {
        let ParsedCsv::Moneyball(players) = parse_csv(MONEYBALL_EXPORT).expect("parse Moneyball")
        else {
            panic!("detect Moneyball export");
        };

        assert_eq!(
            MONEYBALL_EXPORT
                .lines()
                .next()
                .expect("fixture header")
                .split(';')
                .count(),
            84
        );
        assert_eq!(players.len(), 75);
        assert!(players.iter().all(|player| player.uid > 0));

        let player = players
            .iter()
            .find(|player| player.uid == 2_002_188_319)
            .expect("Alessandro Willeit");
        assert_eq!(player.name.as_deref(), Some("Alessandro Willeit"));
        assert_eq!(player.height_centimeters, Some(181));
        assert_eq!(
            player.transfer_value,
            Some(MoneyballTransferValue::Range {
                lower_euros: 30_000_000,
                upper_euros: 35_000_000,
            })
        );
        assert_eq!(
            player.appearances,
            Some(MoneyballAppearances {
                starts: 38,
                substitutes: 0,
            })
        );
        assert_eq!(player.distance_kilometers, Some(205.1));
        assert_eq!(
            player.metric("Save Percentage"),
            Some(MoneyballMetricValue::Decimal(84.0))
        );
        assert_eq!(
            player.metric("Passes Completed"),
            Some(MoneyballMetricValue::Count(1_118))
        );
    }

    #[test]
    fn rejects_moneyball_near_match_missing_a_required_group() {
        let input = remove_column(MONEYBALL_EXPORT, "xG/shot");

        assert_eq!(
            parse_csv(&input).expect_err("missing required Moneyball header must fail"),
            CsvImportError::MissingRequiredHeader("xG/shot")
        );
    }

    #[test]
    fn rejects_an_incomplete_moneyball_export_instead_of_parsing_it_as_youth() {
        let input = remove_column(&remove_column(MONEYBALL_EXPORT, "Wage"), "xG");

        assert_eq!(
            parse_csv(&input).expect_err("incomplete Moneyball export must fail"),
            CsvImportError::MissingRequiredHeader("Wage")
        );
    }

    #[test]
    fn rejects_malformed_moneyball_values_and_duplicate_uids() {
        let malformed = MONEYBALL_EXPORT.replacen("€19.5M - €25M", "not money", 1);
        assert_eq!(
            parse_csv(&malformed).expect_err("malformed transfer value must fail"),
            CsvImportError::InvalidValue {
                row: 2,
                field: "Transfer Value",
                expected: "a euro amount or range",
            }
        );

        let duplicate = format!(
            "{MONEYBALL_EXPORT}{}\n",
            MONEYBALL_EXPORT.lines().nth(1).expect("first player")
        );
        assert_eq!(
            parse_csv(&duplicate).expect_err("duplicate UID must fail"),
            CsvImportError::DuplicateUid {
                first_row: 2,
                row: 77,
            }
        );
    }

    #[test]
    fn rejects_fractional_euros_and_labels_optional_asking_price_errors() {
        let fractional_euros = MONEYBALL_EXPORT.replacen("€19.5M - €25M", "€1.2345K - €2M", 1);
        assert_eq!(
            parse_csv(&fractional_euros).expect_err("fractional euros must fail"),
            CsvImportError::InvalidValue {
                row: 2,
                field: "Transfer Value",
                expected: "a euro amount or range",
            }
        );

        let asking_price = append_column(MONEYBALL_EXPORT, "Asking Price", "not money");
        assert_eq!(
            parse_csv(&asking_price).expect_err("malformed asking price must fail"),
            CsvImportError::InvalidValue {
                row: 2,
                field: "Asking Price",
                expected: "a euro amount or range",
            }
        );
    }

    #[test]
    fn supports_bom_aliases_optional_groups_and_nulls() {
        let aliased = format!(
            "\u{feff}{}",
            MONEYBALL_EXPORT
                .replacen("Shots on Target", "ShT", 1)
                .replacen("Passes Completed", "Ps C", 1)
        );
        let ParsedCsv::Moneyball(players) = parse_csv(&aliased).expect("parse aliased Moneyball")
        else {
            panic!("detect Moneyball export");
        };
        assert_eq!(
            players[3].metric("Shots on Target"),
            Some(MoneyballMetricValue::Count(0))
        );

        let optional_groups_removed = ["Division", "CA", "PA", "Save Percentage"]
            .into_iter()
            .fold(MONEYBALL_EXPORT.to_string(), |input, header| {
                remove_column(&input, header)
            });
        let ParsedCsv::Moneyball(players) =
            parse_csv(&optional_groups_removed).expect("optional Moneyball groups may be absent")
        else {
            panic!("detect Moneyball export");
        };
        assert_eq!(players[0].division, None);
        assert_eq!(players[0].ca, None);
        assert_eq!(players[0].pa, None);
        assert_eq!(players[0].metric("Save Percentage"), None);

        let null_transfer = MONEYBALL_EXPORT.replacen("€19.5M - €25M", "-", 1);
        let ParsedCsv::Moneyball(players) = parse_csv(&null_transfer).expect("preserve nulls")
        else {
            panic!("detect Moneyball export");
        };
        assert_eq!(players[0].transfer_value, None);
    }

    #[test]
    fn rejects_a_comma_delimited_moneyball_signature_and_non_finite_metrics() {
        assert_eq!(
            parse_csv(&MONEYBALL_EXPORT.replacen(';', ",", 84))
                .expect_err("Moneyball is semicolon-delimited"),
            CsvImportError::UnsupportedDialect
        );

        let non_finite = MONEYBALL_EXPORT.replacen("0.0", "NaN", 1);
        assert_eq!(
            parse_csv(&non_finite).expect_err("non-finite metric must fail"),
            CsvImportError::InvalidValue {
                row: 2,
                field: "Shots From Outside The Box Per 90 minutes",
                expected: "a dot-decimal number",
            }
        );

        let non_finite_distance = MONEYBALL_EXPORT.replacen("0.0km", "NaNkm", 1);
        assert_eq!(
            parse_csv(&non_finite_distance).expect_err("non-finite distance must fail"),
            CsvImportError::InvalidValue {
                row: 2,
                field: "Distance",
                expected: "a kilometer value",
            }
        );
    }

    #[test]
    fn detects_youth_tracker_after_the_moneyball_signature_check() {
        assert!(matches!(
            parse_csv(MONZA_EXPORT).expect("parse Youth Tracker"),
            ParsedCsv::YouthTracker(_)
        ));
    }

    fn remove_column(input: &str, header: &str) -> String {
        let column = input
            .lines()
            .next()
            .expect("CSV header")
            .split(';')
            .position(|value| value == header)
            .expect("header exists");
        input
            .lines()
            .map(|line| {
                line.split(';')
                    .enumerate()
                    .filter(|(index, _)| *index != column)
                    .map(|(_, value)| value)
                    .collect::<Vec<_>>()
                    .join(";")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn append_column(input: &str, header: &str, first_row_value: &str) -> String {
        input
            .lines()
            .enumerate()
            .map(|(index, line)| match index {
                0 => format!("{line};{header}"),
                1 => format!("{line};{first_row_value}"),
                _ => format!("{line};-"),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn parses_the_pinned_monza_youth_export() {
        let players = parse_youth_tracker(MONZA_EXPORT).expect("parse pinned Monza export");

        assert_eq!(
            MONZA_EXPORT
                .lines()
                .next()
                .expect("fixture header")
                .split(';')
                .count(),
            66
        );
        assert_eq!(players.len(), 74);
        assert!(players.iter().all(|player| player.uid > 0));

        let player = players
            .iter()
            .find(|player| player.uid == 2_002_402_173)
            .expect("Andrea Bisceglia");
        assert_eq!(player.name.as_deref(), Some("Andrea Bisceglia"));
        assert_eq!(player.ca, Some(83));
        assert_eq!(player.pa, Some(147));
        assert_eq!(
            player.hidden_attribute(YouthTrackerHiddenAttribute::Ambition),
            Some(15)
        );
        assert_eq!(player.attribute(YouthTrackerAttribute::Passing), Some(7));
        assert_eq!(player.attribute(YouthTrackerAttribute::Reflexes), Some(14));
        assert_eq!(player.international_appearances, Some(0));
    }

    #[test]
    fn parses_bom_comma_aliases_and_quoted_multiline_names() {
        let players = parse_youth_tracker(
            "\u{feff}unique id,Name,Current Ability,Potential,AT Apps,Int Apps,AT Gls,Passing\n42,\"Jane, \"\"JJ\"\"\nDoe\",120,150,\"1 (4)\",2,3,14\n",
        )
        .expect("parse comma export");

        assert_eq!(players.len(), 1);
        let player = &players[0];
        assert_eq!(player.name.as_deref(), Some("Jane, \"JJ\"\nDoe"));
        assert_eq!(player.ca, Some(120));
        assert_eq!(player.pa, Some(150));
        assert_eq!(player.all_time_appearances, Some(5));
        assert_eq!(player.international_appearances, Some(2));
        assert_eq!(player.all_time_goals, Some(3));
        assert_eq!(player.attribute(YouthTrackerAttribute::Passing), Some(14));
    }

    #[test]
    fn preserves_blank_and_unavailable_exported_values_as_null() {
        let players = parse_youth_tracker(
            "Unique ID;Player;CA;PA;Passing\n42;Blank values;;;\n43;Unavailable values;N/A;-;—\n",
        )
        .expect("parse unavailable values");

        for player in players {
            assert_eq!(player.ca, None);
            assert_eq!(player.pa, None);
            assert_eq!(player.attribute(YouthTrackerAttribute::Passing), None);
        }
    }

    #[test]
    fn does_not_bind_partial_headers_to_typed_fields() {
        let players = parse_youth_tracker(
            "Unique ID;Player;International Apps;Average Rating\n42;Player;7;6.8\n",
        )
        .expect("parse explicit international appearances");

        let player = &players[0];
        assert_eq!(player.international_appearances, Some(7));
        assert_eq!(player.nationality, None);
        assert_eq!(player.age, None);
    }

    #[test]
    fn rejects_malformed_populated_values() {
        let error = parse_youth_tracker("Unique ID;Player;CA\n42;Player;not-a-number\n")
            .expect_err("malformed CA must fail");

        assert_eq!(
            error,
            CsvImportError::InvalidValue {
                row: 2,
                field: "Current Ability",
                expected: "an integer from 1 to 200",
            }
        );
    }

    #[test]
    fn rejects_invalid_and_out_of_range_uids() {
        let zero_error =
            parse_youth_tracker("Unique ID;Player\n0;Player\n").expect_err("zero UID must fail");
        assert_eq!(
            zero_error,
            CsvImportError::InvalidValue {
                row: 2,
                field: "Unique ID",
                expected: "a positive u32",
            }
        );

        let oversized_error = parse_youth_tracker("Unique ID;Player\n4294967296;Player\n")
            .expect_err("oversized UID must fail");
        assert_eq!(
            oversized_error,
            CsvImportError::InvalidValue {
                row: 2,
                field: "Unique ID",
                expected: "a positive u32",
            }
        );
    }

    #[test]
    fn rejects_duplicate_uids() {
        let error = parse_youth_tracker("Unique ID;Player\n42;First\n42;Second\n")
            .expect_err("duplicate UID must fail");

        assert_eq!(
            error,
            CsvImportError::DuplicateUid {
                first_row: 2,
                row: 3,
            }
        );
    }

    #[test]
    fn rejects_rows_with_a_different_width_from_the_header() {
        let error = parse_youth_tracker("Unique ID;Player;CA\n42;Player\n")
            .expect_err("short record must fail");

        assert_eq!(error, CsvImportError::MalformedCsv { row: 2 });
    }

    #[test]
    fn requires_a_player_name_header() {
        let error =
            parse_youth_tracker("Unique ID;CA\n42;100\n").expect_err("name header must be present");

        assert_eq!(error, CsvImportError::MissingRequiredHeader("Player"));
    }
}
