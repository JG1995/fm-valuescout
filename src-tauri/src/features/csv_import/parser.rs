use std::collections::{BTreeMap, HashMap};

use csv::{ReaderBuilder, StringRecord, Trim};

use super::{
    CsvImportError, YouthTrackerAttribute, YouthTrackerHiddenAttribute, YouthTrackerPlayer,
};

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
