//! Synthetic save containers for tests, written from observed format facts.
//!
//! Ported from fmsave 0.10.0 (commit a71abe9b7b38e35affa05c49c3619ff05eb21795, MIT
//! License, Copyright (c) 2026 Randal S. Olson): `tests/fixtures/container.py`, the FM27
//! `game_info` and structured summary fixtures in `tests/test_version.py`, the name pools and
//! player records of `tests/fixtures/game_db.py`, and `tests/fixtures/season_stats.py` with
//! FM27's line marker and length. See `THIRD_PARTY_NOTICES.md`.
//!
//! Nothing here uses the reader's own constants or parsers, so a wrong offset in the reader
//! fails tests built here. The containers hold fictional content, not game saves.

use std::path::{Path, PathBuf};

pub(super) const FM27_BUILD: &str = "27.0.0+2486695";
pub(super) const UNREGISTERED_FM27_BUILD: &str = "27.1.0+3000001";

const FILE_MAGIC: &[u8] = b"\x02\x01fmf.";
const HEADER_SIZE: usize = 26;
const SKIPPABLE_FRAME_MAGIC: u32 = 0x184D_2A50;

pub(super) fn length_prefixed(text: &str) -> Vec<u8> {
    let mut bytes = (text.len() as u32).to_le_bytes().to_vec();
    bytes.extend_from_slice(text.as_bytes());
    bytes
}

pub(super) fn packed_date(day_of_year: u16, year: u16, time_slot: u16) -> [u8; 4] {
    let mut bytes = [0; 4];
    bytes[..2].copy_from_slice(&(day_of_year | time_slot << 9).to_le_bytes());
    bytes[2..].copy_from_slice(&year.to_le_bytes());
    bytes
}

pub(super) fn section_body(extension: &str, schema: u16, payload: &[u8]) -> Vec<u8> {
    let mut body = b"\x03\x01".to_vec();
    body.extend(extension.bytes().rev());
    body.extend_from_slice(&schema.to_le_bytes());
    body.extend_from_slice(payload);
    body
}

pub(super) fn compress(bytes: &[u8]) -> Vec<u8> {
    zstd::bulk::compress(bytes, 3).expect("compress fixture frame")
}

/// A valid, empty Zstandard frame whose header claims `content_size` bytes, so a bound can be
/// tested against a declared size without producing that output.
pub(super) fn frame_declaring_content_size(content_size: u64) -> Vec<u8> {
    let mut frame = 0xFD2F_B528_u32.to_le_bytes().to_vec();
    // An eight-byte content size in a single-segment frame, with no window descriptor.
    frame.push(0xE0);
    frame.extend_from_slice(&content_size.to_le_bytes());
    // One last, raw, empty block.
    frame.extend_from_slice(&[0x01, 0x00, 0x00]);
    frame
}

pub(super) fn skippable_frame(payload: &[u8]) -> Vec<u8> {
    let mut frame = SKIPPABLE_FRAME_MAGIC.to_le_bytes().to_vec();
    frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    frame.extend_from_slice(payload);
    frame
}

fn put_u32(body: &mut [u8], at: usize, value: u32) {
    body[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

/// Invented schema-47 `game_info` fields for `build`. The filename count sits
/// `list_after_db_version` bytes after the database version, seven bytes before the clock,
/// which holds day 60 of 2031.
pub(super) fn fm27_game_info_body(
    build: &str,
    schema: u16,
    list_after_db_version: usize,
    filenames: &[&str],
) -> Vec<u8> {
    let number: u32 = build.split('+').nth(1).unwrap().parse().unwrap();
    let build_numbers = [2_329_565, number, number];
    let game_date_offset = list_after_db_version + 7;
    let late_build_number_offset = list_after_db_version + 215;
    let migration_names = ["EXAMPLE_PATCH_fm27"];

    let version_field = length_prefixed("26.2.0+0");
    let base = 8 + version_field.len();
    let names_offset = base + late_build_number_offset + 8;
    let mut body = section_body(".dat", schema, &version_field);
    body.resize(names_offset, 0);
    put_u32(&mut body, base + 34, build_numbers[0]);
    put_u32(&mut body, base + 38, build_numbers[1]);
    body[base + game_date_offset..base + game_date_offset + 4]
        .copy_from_slice(&packed_date(60, 2031, 66));
    put_u32(&mut body, base + late_build_number_offset, build_numbers[2]);
    put_u32(
        &mut body,
        base + late_build_number_offset + 4,
        migration_names.len() as u32,
    );
    for name in migration_names {
        body.extend(length_prefixed(name));
    }
    let count_at = base + game_date_offset - 7;
    put_u32(&mut body, count_at, filenames.len() as u32);
    let listed: Vec<u8> = filenames
        .iter()
        .flat_map(|name| length_prefixed(name))
        .collect();
    body.splice(count_at + 4..count_at + 4, listed);
    body
}

/// Invented summary fields with variable strings before the structured clock.
pub(super) fn structured_summary_body(
    version: &str,
    schema: u16,
    summary_date: [u8; 4],
) -> Vec<u8> {
    let divisions = ["Example Division"];
    let mut payload = length_prefixed("EXAMPLE,SETUP");
    payload.extend(length_prefixed(version));
    payload.extend_from_slice(&(divisions.len() as u32).to_le_bytes());
    for division in divisions {
        payload.extend(length_prefixed(division));
    }
    payload.extend_from_slice(&[0; 8]);
    payload.extend(length_prefixed("Alex Example"));
    payload.extend(length_prefixed("Northbridge FC"));
    payload.extend_from_slice(&5001_u32.to_le_bytes());
    payload.extend_from_slice(&summary_date);
    section_body(".dat", schema, &payload)
}

pub(super) struct SectionFrame {
    pub name: &'static str,
    pub extension: &'static str,
    pub body: Vec<u8>,
    pub unlisted_frames_after: Vec<Vec<u8>>,
}

impl SectionFrame {
    pub(super) fn new(name: &'static str, body: Vec<u8>) -> Self {
        Self {
            name,
            extension: ".dat",
            body,
            unlisted_frames_after: Vec::new(),
        }
    }
}

/// The sections of fmsave's default fragment, with FM27 metadata for `build`. The summary
/// clock holds day 61 of 2031.
pub(super) fn fm27_sections(build: &str) -> Vec<SectionFrame> {
    vec![
        SectionFrame::new("game_info", fm27_game_info_body(build, 47, 167, &[])),
        SectionFrame::new("memory_pools", section_body(".dat", 3, &[0; 64])),
        SectionFrame::new(
            "save_game_summary",
            structured_summary_body(build, 31, packed_date(61, 2031, 0)),
        ),
        SectionFrame::new(
            "game_db",
            section_body(".dat", 4000, &(0..=255).collect::<Vec<u8>>()),
        ),
        SectionFrame {
            unlisted_frames_after: vec![
                compress(&[b'A'; 32]),
                compress(&[b'B'; 48]),
                skippable_frame(b"skip"),
            ],
            ..SectionFrame::new("non_pl_hist_ls", section_body(".dat", 5, &[0; 16]))
        },
        SectionFrame::new("humans", section_body(".dat", 21, &[0; 32])),
        SectionFrame {
            extension: ".cmt",
            ..SectionFrame::new("tc_history_dt", section_body(".cmt", 7, &[0; 24]))
        },
    ]
}

/// A directory entry's (compressed, decompressed) sizes; `None` keeps the real size.
pub(super) type DeclaredSizes = (Option<u64>, Option<u64>);

/// A fictional container holding `sections` and one `.apm` attachment. `declared_sizes`
/// overrides an entry's sizes, keyed by name and extension.
pub(super) fn build_container(
    sections: &[SectionFrame],
    declared_sizes: &[(&str, DeclaredSizes)],
) -> Vec<u8> {
    let mut stream = Vec::new();
    let mut directory_rows: Vec<(String, String, u64, u64, u64)> = Vec::new();
    let mut add_listed_frame = |stream: &mut Vec<u8>, name: &str, extension: &str, body: &[u8]| {
        let compressed = compress(body);
        let relative_offset = stream.len() as u64;
        stream.extend_from_slice(&compressed);
        let key = format!("{name}{extension}");
        let (compressed_size, decompressed_size) = declared_sizes
            .iter()
            .find(|(declared_key, _)| *declared_key == key)
            .map_or((None, None), |(_, sizes)| *sizes);
        let compressed_size = compressed_size.unwrap_or(compressed.len() as u64);
        let decompressed_size = decompressed_size.unwrap_or(body.len() as u64);
        directory_rows.push((
            name.to_owned(),
            extension.to_owned(),
            relative_offset,
            compressed_size,
            decompressed_size,
        ));
    };
    for section in sections {
        add_listed_frame(&mut stream, section.name, section.extension, &section.body);
        for raw_frame in &section.unlisted_frames_after {
            stream.extend_from_slice(raw_frame);
        }
    }
    add_listed_frame(&mut stream, "1_2_3", ".apm", b"attachment");

    let mut trailer_payload = length_prefixed("Example Career");
    trailer_payload.extend_from_slice(&300_u32.to_le_bytes());
    for (name, extension, relative_offset, compressed_size, decompressed_size) in
        directory_rows.iter().rev()
    {
        trailer_payload.extend(length_prefixed(name));
        trailer_payload.extend(length_prefixed(extension));
        for value in [*relative_offset, *compressed_size, *decompressed_size, 0, 0] {
            trailer_payload.extend_from_slice(&value.to_le_bytes());
        }
    }
    trailer_payload.extend_from_slice(&[0; 16]);
    container_with_trailer(&stream, &compress(&trailer_payload))
}

/// A header pointing past `stream` at a trailer holding `trailer_frame`.
pub(super) fn container_with_trailer(stream: &[u8], trailer_frame: &[u8]) -> Vec<u8> {
    let trailer_offset = (HEADER_SIZE + stream.len()) as u64;
    let mut content = FILE_MAGIC.to_vec();
    content.extend_from_slice(&8_u16.to_le_bytes());
    content.push(0);
    content.extend_from_slice(&(trailer_offset - 9).to_le_bytes());
    content.extend_from_slice(&17_u64.to_le_bytes());
    content.push(3);
    content.extend_from_slice(stream);
    content.extend_from_slice(FILE_MAGIC);
    content.extend_from_slice(&8_u16.to_le_bytes());
    content.push(0);
    content.extend_from_slice(trailer_frame);
    content
}

pub(super) fn fm27_container(build: &str) -> Vec<u8> {
    build_container(&fm27_sections(build), &[])
}

/// `sections` with the named section's body replaced.
pub(super) fn with_body(
    mut sections: Vec<SectionFrame>,
    name: &str,
    body: Vec<u8>,
) -> Vec<SectionFrame> {
    sections
        .iter_mut()
        .find(|section| section.name == name)
        .expect("fixture section")
        .body = body;
    sections
}

pub(super) fn write_save(directory: &Path, content: &[u8]) -> PathBuf {
    let path = directory.join("Example Career.fm");
    std::fs::write(&path, content).expect("write fixture save");
    path
}

const NAME_POOL_SIGNATURE: [u32; 6] = [46421, 0, 1024, 256, 2048, 0];
const PLAYER_RECORD_MARKER: [u8; 4] = [0x01, 0x00, 0x6C, 0x07];
pub(super) const PLAYER_RECORD_BODY_BYTES: usize = 300;

/// The signature, then three pools: a u32 count, then a u32 id, u32 length, and the name for
/// each entry.
fn name_pools_bytes(pools: [&[&str]; 3]) -> Vec<u8> {
    let mut bytes: Vec<u8> = NAME_POOL_SIGNATURE
        .iter()
        .flat_map(|word| word.to_le_bytes())
        .collect();
    for names in pools {
        bytes.extend_from_slice(&(names.len() as u32).to_le_bytes());
        for (index, name) in names.iter().enumerate() {
            bytes.extend_from_slice(&(index as u32).to_le_bytes());
            bytes.extend(length_prefixed(name));
        }
    }
    bytes
}

/// A fictional player record. Its GK rating is the first of the 15 position ratings; the
/// others make every example a natural striker.
#[derive(Debug, Clone, Copy)]
pub(super) struct ExamplePlayer {
    pub pindex: u32,
    pub uid: u32,
    /// Written as the header closing the player's object, after the record body.
    pub unique_id: Option<u32>,
    pub gk_rating: u8,
}

impl ExamplePlayer {
    pub(super) fn outfield(pindex: u32, uid: u32, unique_id: Option<u32>) -> Self {
        Self {
            pindex,
            uid,
            unique_id,
            gk_rating: 1,
        }
    }

    pub(super) fn goalkeeper(pindex: u32, uid: u32, unique_id: Option<u32>) -> Self {
        Self {
            gk_rating: 20,
            ..Self::outfield(pindex, uid, unique_id)
        }
    }
}

/// A 26-byte header (pindex, the doubled uid, object kind, three reputations), then the
/// record body, then the closing header when the player has a Unique ID. The record starts
/// 26 bytes in; body offsets count from there.
fn player_record_bytes(player: &ExamplePlayer) -> Vec<u8> {
    let mut bytes = b"\x00\x40\x00\x00\x00\x00\x00".to_vec();
    for word in [player.pindex, player.uid, player.uid] {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes.push(0);
    for reputation in [5_000_u16, 5_000, 4_000] {
        bytes.extend_from_slice(&reputation.to_le_bytes());
    }
    let mut body = vec![0; PLAYER_RECORD_BODY_BYTES];
    body[0..2].copy_from_slice(&120_u16.to_le_bytes());
    body[2..4].copy_from_slice(&150_i16.to_le_bytes());
    body[8] = 50;
    body[16..20].copy_from_slice(&901_u32.to_le_bytes());
    let mut ratings = [5_u8; 15];
    ratings[0] = player.gk_rating;
    ratings[12] = 19;
    body[24..39].copy_from_slice(&ratings);
    body[39..93].fill(10);
    body[98..102].copy_from_slice(&packed_date(200, 2025, 0));
    body[102..106].copy_from_slice(&PLAYER_RECORD_MARKER);
    body[106..108].copy_from_slice(&5_000_u16.to_le_bytes());
    body[110..112].copy_from_slice(&9_000_u16.to_le_bytes());
    body[121] = 180;
    bytes.extend(body);
    if let Some(unique_id) = player.unique_id {
        for word in [player.pindex + 1, unique_id, unique_id] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
    }
    bytes
}

/// The name pools, 64 zero bytes, then each player's record, with no section header.
pub(super) fn game_db_payload(players: &[ExamplePlayer]) -> Vec<u8> {
    game_db_payload_with_staff(players, &[])
}

/// `game_db_payload`, then each of `staff` as its own object.
pub(super) fn game_db_payload_with_staff(
    players: &[ExamplePlayer],
    staff: &[ExampleStaff],
) -> Vec<u8> {
    let mut payload = name_pools_bytes([&["Alex"], &["Example"], &[]]);
    payload.extend_from_slice(&[0; 64]);
    for player in players {
        payload.extend(player_record_bytes(player));
    }
    for person in staff {
        payload.extend(person.to_bytes());
    }
    payload
}

/// A fictional non-player. A list stores it by `key`; the object before it in `game_db` has
/// the key one lower.
#[derive(Debug, Clone, Copy)]
pub(super) struct ExampleStaff {
    pub key: u32,
    pub unique_id: u32,
    /// The type of job the person holds: 58 is a recruitment analyst, 46 a doctor, and 0
    /// none.
    pub job_type: u8,
    /// The 15 job ratings in stored order, or `None` for a person stored without any.
    pub job_ratings: Option<[u8; 15]>,
    /// The stored coaching qualification: 0 is none and 3 a Continental B Licence.
    pub qualification: u8,
}

impl ExampleStaff {
    /// A person whose one highest job rating is the 1-based `preferred`.
    pub(super) fn preferring(key: u32, unique_id: u32, preferred: usize) -> Self {
        let mut job_ratings = [1; 15];
        job_ratings[preferred - 1] = 20;
        Self {
            key,
            unique_id,
            job_type: 0,
            job_ratings: Some(job_ratings),
            qualification: 0,
        }
    }

    /// The trailer of the object before this one, the person's object, and its trailer: the
    /// key, then the Unique ID twice.
    fn to_bytes(self) -> Vec<u8> {
        let unset_date = [1, 0, 0x6C, 0x07];
        // The earlier object's trailer, whose Unique ID the game allocated and stored once.
        let mut bytes = (self.key - 1).to_le_bytes().to_vec();
        bytes.extend_from_slice(&2_100_000_000_u32.to_le_bytes());
        bytes.extend_from_slice(&7_u32.to_le_bytes());
        // Kind, no leading entries, three reputations, both abilities, and the job type.
        bytes.push(1);
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        for word in [3_290_u16, 3_290, 2_887, 70, 97] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes.push(self.job_type);
        // Eight unread bytes, the attribute block, and fields of no fixed length.
        bytes.extend_from_slice(&[0; 8]);
        bytes.push(12);
        bytes.extend_from_slice(&[10; 52]);
        bytes.extend_from_slice(&[0xFF; 24]);
        bytes.extend_from_slice(&[0; 40]);
        // A first list of one entry, an empty second list, then the job ratings.
        bytes.extend_from_slice(&[0, 1]);
        bytes.extend_from_slice(&7_u64.to_le_bytes());
        bytes.push(10);
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        match self.job_ratings {
            Some(job_ratings) => {
                bytes.push(1);
                bytes.extend_from_slice(&job_ratings);
            }
            None => bytes.push(0),
        }
        // One four-byte entry, three unread bytes, and the dates before the qualification.
        bytes.push(1);
        bytes.extend_from_slice(&[0, 0xB7, 1, 0]);
        bytes.extend_from_slice(&[0; 3]);
        bytes.extend_from_slice(&packed_date(194, 2026, 0));
        bytes.push(0);
        bytes.extend_from_slice(&unset_date);
        bytes.extend_from_slice(&unset_date);
        bytes.extend_from_slice(&[0xFF; 8]);
        bytes.extend_from_slice(&unset_date);
        bytes.push(self.qualification);
        bytes.extend_from_slice(&[0; 40]);
        for word in [self.key, self.unique_id, self.unique_id] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes
    }
}

const SEASON_LINE_BYTES: usize = 155;
const SEASON_HEADER_BYTES: usize = 16;
const SEASON_BLOCK_BYTES: usize = 20;
const SEASON_FOOTER_BYTES: usize = 16;
const SEASON_RATING_ITEM_BYTES: usize = 8;

/// Each line field: (name, offset from the line start, width in bytes, signed, stored units
/// per public unit). The `word_*` fields are the words goalkeepers and outfield players read
/// differently.
const SEASON_LINE_FIELDS: &[(&str, usize, usize, bool, f64)] = &[
    ("rating_sum", 2, 2, false, 1.0),
    ("minutes", 4, 2, false, 1.0),
    ("starts", 6, 1, false, 1.0),
    ("substitute_appearances", 7, 1, false, 1.0),
    ("rated_appearances", 8, 1, false, 1.0),
    ("goals", 9, 1, false, 1.0),
    ("assists", 10, 1, false, 1.0),
    ("goals_allowed", 11, 1, false, 1.0),
    ("player_of_the_match", 14, 1, false, 1.0),
    ("clean_sheets", 17, 1, false, 1.0),
    ("team_goals", 19, 2, false, 1.0),
    ("games_won", 23, 2, false, 1.0),
    ("games_lost", 25, 2, false, 1.0),
    ("passes_attempted", 27, 2, false, 1.0),
    ("word_31", 31, 2, false, 1.0),
    ("word_33", 33, 2, false, 1.0),
    ("word_35", 35, 2, false, 1.0),
    ("word_37", 37, 2, false, 1.0),
    ("shots", 45, 2, false, 1.0),
    ("distance_km", 65, 2, false, 10.0),
    ("offsides", 67, 2, false, 1.0),
    ("word_69", 69, 2, false, 1.0),
    ("word_71", 71, 2, false, 1.0),
    ("word_83", 83, 2, false, 1.0),
    ("expected_goals", 91, 2, false, 100.0),
    ("expected_assists", 93, 2, false, 100.0),
    ("word_95", 95, 2, false, 1.0),
    ("goals_outside_box", 101, 2, false, 1.0),
    ("expected_goals_prevented", 113, 2, true, 100.0),
    ("pressures_attempted", 121, 2, false, 1.0),
    ("pressures_completed", 123, 2, false, 1.0),
    ("non_penalty_expected_goals", 127, 2, false, 100.0),
];

/// One FM27 line: `01 07`, then each named field at its offset, zero elsewhere. Values are in
/// public units (xG as 1.25, distance in km) and stored scaled.
pub(super) fn stat_line(values: &[(&str, f64)]) -> Vec<u8> {
    let mut line = vec![0; SEASON_LINE_BYTES];
    line[0..2].copy_from_slice(b"\x01\x07");
    for (name, value) in values {
        let &(_, offset, width, signed, scale) = SEASON_LINE_FIELDS
            .iter()
            .find(|field| field.0 == *name)
            .expect("fixture line field");
        let stored = (value * scale).round() as i64;
        let bytes = if signed {
            (stored as i16).to_le_bytes().to_vec()
        } else {
            (stored as u64).to_le_bytes()[..width].to_vec()
        };
        line[offset..offset + width].copy_from_slice(&bytes);
    }
    line
}

/// A slot holding nothing, or a line.
pub(super) type Slot = Option<Vec<u8>>;

/// One player's record: 8 own slots and, for each other team, its id and 6 slots.
#[derive(Debug, Clone)]
pub(super) struct ExampleRecord {
    pub key: u32,
    pub own_slots: Vec<Slot>,
    pub other_teams: Vec<(u32, Vec<Slot>)>,
}

impl ExampleRecord {
    /// A record whose own slots hold `lines` by slot number and nothing else.
    pub(super) fn with_lines(key: u32, lines: &[(usize, Vec<u8>)]) -> Self {
        let mut own_slots = vec![None; 8];
        for (slot, line) in lines {
            own_slots[*slot] = Some(line.clone());
        }
        Self {
            key,
            own_slots,
            other_teams: Vec::new(),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let slots = |slots: &[Slot]| -> Vec<u8> {
            slots
                .iter()
                .flat_map(|slot| slot.clone().unwrap_or_else(|| vec![0]))
                .collect()
        };
        let mut record = vec![1];
        record.extend_from_slice(&self.key.to_le_bytes());
        record.extend_from_slice(&[0; SEASON_HEADER_BYTES]);
        record.extend(slots(&self.own_slots));
        record.extend_from_slice(&[0; SEASON_BLOCK_BYTES]);
        record.extend_from_slice(&(self.other_teams.len() as u32).to_le_bytes());
        for (team_id, team_slots) in &self.other_teams {
            record.push(0);
            record.extend_from_slice(&team_id.to_le_bytes());
            record.extend(slots(team_slots));
        }
        record.extend_from_slice(&1_u32.to_le_bytes());
        record.push(2);
        record.extend_from_slice(&[0; SEASON_RATING_ITEM_BYTES - 1]);
        record.push(16);
        for _ in 0..16 {
            record.extend_from_slice(&[64, 5, 6]);
        }
        record
    }
}

/// The whole section body: header, records, the zero that ends the list, and the footer.
pub(super) fn season_stats_body(records: &[ExampleRecord], schema: u16) -> Vec<u8> {
    let mut payload: Vec<u8> = records.iter().flat_map(ExampleRecord::to_bytes).collect();
    payload.push(0);
    payload.extend_from_slice(&[0; SEASON_FOOTER_BYTES]);
    section_body(".dat", schema, &payload)
}

/// An FM27 save of `build` whose `game_db` holds `players` at `game_db_schema` and whose span
/// holds another section frame, then `season_stats`.
pub(super) fn season_stats_save(
    build: &str,
    game_db_schema: u16,
    players: &[ExamplePlayer],
    season_stats: &[u8],
) -> Vec<u8> {
    build_container(
        &season_stats_sections(build, game_db_schema, players, &[], season_stats),
        &[],
    )
}

/// `season_stats_save` with a `scout_man` section holding `scout_section`.
pub(super) fn save_with_scout_section(
    build: &str,
    players: &[ExamplePlayer],
    season_stats: &[u8],
    scout_section: Vec<u8>,
) -> Vec<u8> {
    save_with_staff(build, players, &[], season_stats, scout_section)
}

/// `save_with_scout_section` whose `game_db` also holds `staff`.
pub(super) fn save_with_staff(
    build: &str,
    players: &[ExamplePlayer],
    staff: &[ExampleStaff],
    season_stats: &[u8],
    scout_section: Vec<u8>,
) -> Vec<u8> {
    let mut sections = season_stats_sections(build, 4000, players, staff, season_stats);
    sections.push(SectionFrame::new("scout_man", scout_section));
    build_container(&sections, &[])
}

fn season_stats_sections(
    build: &str,
    game_db_schema: u16,
    players: &[ExamplePlayer],
    staff: &[ExampleStaff],
    season_stats: &[u8],
) -> Vec<SectionFrame> {
    let game_db = section_body(
        ".dat",
        game_db_schema,
        &game_db_payload_with_staff(players, staff),
    );
    let mut sections = with_body(fm27_sections(build), "game_db", game_db);
    let span = sections
        .iter_mut()
        .find(|section| section.name == "non_pl_hist_ls")
        .expect("span section");
    span.unlisted_frames_after = vec![
        compress(&section_body(".dat", 9, &[0; 40])),
        compress(season_stats),
    ];
    sections
}

/// One list record of a `scout_man` section. Kind 0 with the `frlp` filter tag is a player
/// shortlist and kind 8 with the `flpn` filter tag a staff shortlist; an empty name is the
/// game's default list of its kind.
#[derive(Debug, Clone)]
pub(super) struct ExampleList {
    pub kind: u32,
    pub filter_tag: [u8; 4],
    pub name: &'static str,
    /// Each member's key: a player's pindex plus one, or a staff member's own key.
    pub member_keys: Vec<u32>,
}

impl ExampleList {
    pub(super) fn shortlist(name: &'static str, player_keys: &[u32]) -> Self {
        Self {
            kind: 0,
            filter_tag: *b"frlp",
            name,
            member_keys: player_keys.to_vec(),
        }
    }

    /// The game's default staff shortlist, holding nobody.
    pub(super) fn staff() -> Self {
        Self::staff_shortlist("", &[])
    }

    pub(super) fn staff_shortlist(name: &'static str, staff_keys: &[u32]) -> Self {
        Self {
            kind: 8,
            filter_tag: *b"flpn",
            name,
            member_keys: staff_keys.to_vec(),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        // Owner, two flag bytes, kind, 14 unused bytes, an unset id, a flag, an unset date.
        let mut record = 38_573_830_u32.to_le_bytes().to_vec();
        record.extend_from_slice(&[0, 1]);
        record.extend_from_slice(&self.kind.to_le_bytes());
        record.extend_from_slice(&[0; 14]);
        record.extend_from_slice(&[0xFF; 4]);
        record.extend_from_slice(&[1, 1, 0, 0x6C, 0x07]);
        record.extend(length_prefixed(self.name));
        record.extend_from_slice(&[1, 0, 1]);
        record.extend_from_slice(&self.filter_tag);
        record.extend_from_slice(b"tlif");
        // A filter body of no fixed length, then the member tag of the filter's kind.
        record.extend_from_slice(&[0, 0x12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1]);
        if &self.filter_tag == b"flpn" {
            record.extend_from_slice(b"fpnp\0\x01roip\0\x01\x01\0");
            record.extend_from_slice(b"manP\x01");
        } else {
            record.extend_from_slice(b"roip\0rfsc\0");
            record.extend_from_slice(b"rSrP\0");
        }
        record.extend_from_slice(&(self.member_keys.len() as u32).to_le_bytes());
        for key in &self.member_keys {
            record.push(2);
            record.extend_from_slice(&key.to_le_bytes());
            record.extend_from_slice(&[1, 0, 0xEB, 0x07, 0, 0, 0, 0, 1, 0, 0x6C, 0x07]);
            record.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0]);
        }
        // List settings, two view columns, four unused bytes, and the record's end.
        record.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x5A, 0, 0]);
        record.extend_from_slice(&2_u32.to_le_bytes());
        record.extend_from_slice(&[1, 0, 0, 0, 2, 0, 0, 0]);
        record.extend_from_slice(&[0; 4]);
        record.extend_from_slice(&30_u16.to_le_bytes());
        record
    }
}

/// A whole `scout_man` section: its header, the list count, and `lists` back to back.
pub(super) fn scout_section(lists: &[ExampleList], schema: u16) -> Vec<u8> {
    let mut payload = 31_u16.to_le_bytes().to_vec();
    payload.extend_from_slice(&(lists.len() as u32).to_le_bytes());
    payload.extend(lists.iter().flat_map(ExampleList::to_bytes));
    section_body(".dat", schema, &payload)
}
