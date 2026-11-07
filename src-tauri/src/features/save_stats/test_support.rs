//! Synthetic save containers for tests, written from observed format facts.
//!
//! Ported from fmsave 0.10.0 (commit a71abe9b7b38e35affa05c49c3619ff05eb21795, MIT
//! License, Copyright (c) 2026 Randal S. Olson): `tests/fixtures/container.py` and the FM27
//! `game_info` and structured summary fixtures in `tests/test_version.py`. See
//! `THIRD_PARTY_NOTICES.md`.
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
