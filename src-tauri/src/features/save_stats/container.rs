//! The save container: file access, header, trailer directory, regions, and bounded frames.
//!
//! Ported from fmsave 0.10.0 (commit a71abe9b7b38e35affa05c49c3619ff05eb21795, MIT
//! License, Copyright (c) 2026 Randal S. Olson): `_container.py` and the integer and string
//! helpers of `_scan.py`. See `THIRD_PARTY_NOTICES.md`.
//!
//! A save is a 26-byte header, a run of Zstandard frames, and a trailer holding one frame
//! whose payload lists the named frames (the directory). Every size read from the file is
//! untrusted: frame bounds and declared output sizes are checked before anything is
//! decompressed, and no buffer is sized from a limit rather than from a declared size.

use std::borrow::Cow;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::time::SystemTime;

use super::SaveStatsError;

/// The largest save file the reader opens. The developer's reference save is 136 MiB.
pub(super) const MAX_SAVE_FILE_BYTES: u64 = 1024 * 1024 * 1024;

const FILE_MAGIC: &[u8] = b"\x02\x01fmf.";
const HEADER_SIZE: u64 = 26;
const TRAILER_POINTER_AT: usize = 9;
const TRAILER_HEADER_SIZE: u64 = 9;
const FRAME_BASE: u64 = 26;
const SECTION_EXTENSIONS: [&str; 2] = [".dat", ".cmt"];
const UNLISTED_REGION_PREFIX: &str = "unlisted_after_";

// Decompressed output is bounded relative to the file, as fmsave bounds it: one part may
// expand to 64 times its compressed size and the whole save to 32 times the file size, but
// each may always reach 64 MiB. fmsave measured no real part over 64 MiB expanding more than
// 11 times and no save expanding more than 4.7 times.
const EXPANSION_ALLOWANCE: u64 = 64 * 1024 * 1024;
const MAX_PART_EXPANSION: u64 = 64;
const MAX_SAVE_EXPANSION: u64 = 32;
const MAX_TRAILER_COMPRESSED_BYTES: u64 = 64 * 1024 * 1024;
const MAX_TRAILER_DECOMPRESSED_BYTES: u64 = 64 * 1024 * 1024;
/// Metadata sections are a few kilobytes in every save fmsave measured.
const MAX_METADATA_SECTION_BYTES: u64 = 1024 * 1024;
const MAX_SAVE_NAME_BYTES: usize = 4096;
const MAX_ENTRY_NAME_BYTES: u32 = 256;
const MAX_DIRECTORY_ENTRIES: usize = 100_000;
const ENTRY_FIXED_TAIL_BYTES: usize = 40;
const HEAD_READ_BYTES: u64 = 128 * 1024 + 32;
const MAX_FRAMES_PER_REGION: usize = 1_000_000;
const ZSTD_FRAME_MAGIC: u32 = 0xFD2F_B528;
const SKIPPABLE_MAGIC_FIRST: u32 = 0x184D_2A50;
const SKIPPABLE_MAGIC_LAST: u32 = 0x184D_2A5F;
const ZSTD_MAX_BLOCK_BYTES: u32 = 128 * 1024;
const BLOCK_TYPE_RLE: u32 = 1;
const BLOCK_TYPE_RESERVED: u32 = 3;

/// The metadata that must not change while a save is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct FileStamp {
    pub len: u64,
    pub modified: Option<SystemTime>,
}

impl FileStamp {
    pub(super) fn of(file: &File) -> Result<Self, SaveStatsError> {
        let metadata = file.metadata().map_err(|_| SaveStatsError::Unreadable)?;
        Ok(Self {
            len: metadata.len(),
            modified: metadata.modified().ok(),
        })
    }
}

/// Reads the whole save into memory once, failing if it changed during the read.
pub(super) fn read_save_bytes(path: &Path) -> Result<Vec<u8>, SaveStatsError> {
    read_save_bytes_with(path, FileStamp::of)
}

fn read_save_bytes_with(
    path: &Path,
    stamp: impl Fn(&File) -> Result<FileStamp, SaveStatsError>,
) -> Result<Vec<u8>, SaveStatsError> {
    read_unchanged(path, stamp, |file, len| {
        let mut bytes = buffer_for(len)?;
        file.take(len + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| SaveStatsError::Unreadable)?;
        if bytes.len() as u64 != len {
            return Err(SaveStatsError::ChangedDuringRead);
        }
        Ok(bytes)
    })
}

/// Opens a regular file read-only, runs `read` on it, and fails with `ChangedDuringRead` if
/// the file's length or modification time differs afterwards, whatever `read` returned.
/// `read` receives the file length, already checked against `MAX_SAVE_FILE_BYTES`.
pub(super) fn read_unchanged<T>(
    path: &Path,
    stamp: impl Fn(&File) -> Result<FileStamp, SaveStatsError>,
    read: impl FnOnce(&mut File, u64) -> Result<T, SaveStatsError>,
) -> Result<T, SaveStatsError> {
    let mut file = open_regular_file(path)?;
    let before = stamp(&file)?;
    if before.len > MAX_SAVE_FILE_BYTES {
        return Err(SaveStatsError::OutOfBounds);
    }
    let result = read(&mut file, before.len);
    if stamp(&file)? != before {
        return Err(SaveStatsError::ChangedDuringRead);
    }
    result
}

/// Rejects anything but a regular file before opening it, so a directory, pipe, or device
/// is never read, and checks the opened handle again.
fn open_regular_file(path: &Path) -> Result<File, SaveStatsError> {
    let metadata = fs::symlink_metadata(path).map_err(open_error)?;
    if !metadata.file_type().is_file() {
        return Err(SaveStatsError::Unreadable);
    }
    let file = File::open(path).map_err(open_error)?;
    let metadata = file.metadata().map_err(open_error)?;
    if !metadata.file_type().is_file() {
        return Err(SaveStatsError::Unreadable);
    }
    Ok(file)
}

fn open_error(error: io::Error) -> SaveStatsError {
    if error.kind() == io::ErrorKind::NotFound {
        SaveStatsError::NotFound
    } else {
        SaveStatsError::Unreadable
    }
}

/// Where container bytes come from: the open file for bounded metadata reads, or the whole
/// file already in memory.
pub(super) trait SaveSource {
    /// Exactly `length` bytes at `offset`, or `Corrupt` when the file ends first.
    fn read_at(&mut self, offset: u64, length: u64) -> Result<Cow<'_, [u8]>, SaveStatsError>;
}

impl SaveSource for File {
    fn read_at(&mut self, offset: u64, length: u64) -> Result<Cow<'_, [u8]>, SaveStatsError> {
        self.seek(SeekFrom::Start(offset))
            .map_err(|_| SaveStatsError::Unreadable)?;
        let mut bytes = buffer_for(length)?;
        self.take(length)
            .read_to_end(&mut bytes)
            .map_err(|_| SaveStatsError::Unreadable)?;
        if bytes.len() as u64 != length {
            return Err(SaveStatsError::Corrupt);
        }
        Ok(Cow::Owned(bytes))
    }
}

impl SaveSource for &[u8] {
    fn read_at(&mut self, offset: u64, length: u64) -> Result<Cow<'_, [u8]>, SaveStatsError> {
        let bytes: &[u8] = self;
        let start = usize::try_from(offset).map_err(|_| SaveStatsError::Corrupt)?;
        let length = usize::try_from(length).map_err(|_| SaveStatsError::Corrupt)?;
        start
            .checked_add(length)
            .and_then(|end| bytes.get(start..end))
            .map(Cow::Borrowed)
            .ok_or(SaveStatsError::Corrupt)
    }
}

/// An empty buffer that can hold `capacity` bytes, or `OutOfBounds` when that much memory is
/// unavailable. Every capacity passed here has already been checked against the read bounds.
fn buffer_for(capacity: u64) -> Result<Vec<u8>, SaveStatsError> {
    let capacity = usize::try_from(capacity).map_err(|_| SaveStatsError::OutOfBounds)?;
    let mut buffer = Vec::new();
    buffer
        .try_reserve_exact(capacity)
        .map_err(|_| SaveStatsError::OutOfBounds)?;
    Ok(buffer)
}

fn bytes_at<const N: usize>(buffer: &[u8], offset: usize) -> Result<[u8; N], SaveStatsError> {
    offset
        .checked_add(N)
        .and_then(|end| buffer.get(offset..end))
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or(SaveStatsError::Corrupt)
}

pub(super) fn read_u16(buffer: &[u8], offset: usize) -> Result<u16, SaveStatsError> {
    bytes_at(buffer, offset).map(u16::from_le_bytes)
}

pub(super) fn read_u32(buffer: &[u8], offset: usize) -> Result<u32, SaveStatsError> {
    bytes_at(buffer, offset).map(u32::from_le_bytes)
}

pub(super) fn read_u64(buffer: &[u8], offset: usize) -> Result<u64, SaveStatsError> {
    bytes_at(buffer, offset).map(u64::from_le_bytes)
}

/// A u32 length followed by that many UTF-8 bytes, and the offset after them.
pub(super) fn read_length_prefixed_string(
    buffer: &[u8],
    offset: usize,
    max_length: usize,
) -> Result<(&str, usize), SaveStatsError> {
    let length = read_u32(buffer, offset)? as usize;
    if length > max_length {
        return Err(SaveStatsError::Corrupt);
    }
    let text_start = offset + 4;
    let text_end = text_start
        .checked_add(length)
        .ok_or(SaveStatsError::Corrupt)?;
    let text = buffer
        .get(text_start..text_end)
        .ok_or(SaveStatsError::Corrupt)?;
    let text = std::str::from_utf8(text).map_err(|_| SaveStatsError::Corrupt)?;
    Ok((text, text_end))
}

/// The first `marker` lying wholly inside `[start, end)`.
pub(super) fn find_marker(
    buffer: &[u8],
    marker: &[u8],
    start: usize,
    end: usize,
) -> Result<Option<usize>, SaveStatsError> {
    if start > end || end > buffer.len() {
        return Err(SaveStatsError::Corrupt);
    }
    Ok(buffer[start..end]
        .windows(marker.len())
        .position(|window| window == marker)
        .map(|position| start + position))
}

/// One named frame listed in the trailer directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DirectoryEntry {
    pub name: String,
    pub extension: String,
    pub frame_offset: u64,
    pub compressed_size: u64,
    pub decompressed_size: u64,
}

impl DirectoryEntry {
    pub(super) fn is_section(&self) -> bool {
        SECTION_EXTENSIONS.contains(&self.extension.as_str())
    }

    /// Saturates instead of wrapping; a saturated end lies past the trailer and is rejected.
    fn frame_end(&self) -> u64 {
        self.frame_offset.saturating_add(self.compressed_size)
    }
}

/// A byte range of the file: one section frame, or an unlisted run of frames.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Region {
    pub name: String,
    pub start: u64,
    pub end: u64,
}

#[derive(Debug)]
pub(super) struct ContainerIndex {
    pub file_size: u64,
    pub entries: Vec<DirectoryEntry>,
    pub regions: Vec<Region>,
}

impl ContainerIndex {
    pub(super) fn section(&self, name: &str) -> Result<&DirectoryEntry, SaveStatsError> {
        self.entries
            .iter()
            .find(|entry| entry.is_section() && entry.name == name)
            .ok_or(SaveStatsError::Corrupt)
    }

    pub(super) fn region(&self, name: &str) -> Result<&Region, SaveStatsError> {
        self.regions
            .iter()
            .find(|region| region.name == name)
            .ok_or(SaveStatsError::Corrupt)
    }

    /// The directory's declared output, which validation has bounded by the save bound.
    fn declared_total(&self) -> u64 {
        self.entries
            .iter()
            .map(|entry| entry.decompressed_size)
            .sum()
    }
}

/// Reads the header and trailer directory and checks every entry's frame bounds and declared
/// output size before anything else is decompressed.
pub(super) fn read_index(
    source: &mut impl SaveSource,
    file_size: u64,
) -> Result<ContainerIndex, SaveStatsError> {
    let header_length = file_size.min(HEADER_SIZE);
    let header = source.read_at(0, header_length)?;
    if !header.starts_with(FILE_MAGIC) {
        return Err(SaveStatsError::NotFmSave);
    }
    if header_length < HEADER_SIZE {
        return Err(SaveStatsError::Corrupt);
    }
    let trailer_offset = (TRAILER_POINTER_AT as u64)
        .checked_add(read_u64(&header, TRAILER_POINTER_AT)?)
        .ok_or(SaveStatsError::Corrupt)?;
    if trailer_offset < HEADER_SIZE
        || trailer_offset.saturating_add(TRAILER_HEADER_SIZE) >= file_size
    {
        return Err(SaveStatsError::Corrupt);
    }
    if file_size - trailer_offset - TRAILER_HEADER_SIZE > MAX_TRAILER_COMPRESSED_BYTES {
        return Err(SaveStatsError::OutOfBounds);
    }
    let trailer = source.read_at(trailer_offset, file_size - trailer_offset)?;
    if !trailer.starts_with(FILE_MAGIC) {
        return Err(SaveStatsError::Corrupt);
    }
    let payload = decompress_frame(
        &trailer[TRAILER_HEADER_SIZE as usize..],
        None,
        MAX_TRAILER_DECOMPRESSED_BYTES,
    )?;
    let entries = parse_directory(&payload)?;
    validate_entries(&entries, trailer_offset, file_size)?;
    let mut section_names = HashSet::new();
    for entry in entries.iter().filter(|entry| entry.is_section()) {
        if !section_names.insert(entry.name.as_str()) {
            return Err(SaveStatsError::Corrupt);
        }
    }
    let regions = build_regions(&entries, trailer_offset)?;
    Ok(ContainerIndex {
        file_size,
        entries,
        regions,
    })
}

fn parse_directory(payload: &[u8]) -> Result<Vec<DirectoryEntry>, SaveStatsError> {
    let (_save_name, mut cursor) = read_length_prefixed_string(payload, 0, MAX_SAVE_NAME_BYTES)?;
    // A u32 whose meaning is not identified.
    cursor += 4;
    let mut entries = Vec::new();
    while let Some((entry, next)) = parse_directory_entry(payload, cursor) {
        entries.push(entry);
        if entries.len() > MAX_DIRECTORY_ENTRIES {
            return Err(SaveStatsError::OutOfBounds);
        }
        cursor = next;
    }
    if !entries.iter().any(DirectoryEntry::is_section) {
        return Err(SaveStatsError::Corrupt);
    }
    entries.sort_by_key(|entry| entry.frame_offset);
    Ok(entries)
}

/// One entry at `cursor`, or `None` where the entry list ends.
fn parse_directory_entry(payload: &[u8], cursor: usize) -> Option<(DirectoryEntry, usize)> {
    let name_length = read_u32(payload, cursor).ok()?;
    if !(1..=MAX_ENTRY_NAME_BYTES).contains(&name_length) {
        return None;
    }
    let extension_length_at = cursor + 4 + name_length as usize;
    let tail_at = extension_length_at + 8;
    if tail_at + ENTRY_FIXED_TAIL_BYTES > payload.len() {
        return None;
    }
    let name = &payload[cursor + 4..extension_length_at];
    let extension = &payload[extension_length_at + 4..tail_at];
    let name_is_valid = name
        .iter()
        .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_');
    let extension_is_valid =
        extension[0] == b'.' && extension[1..].iter().all(u8::is_ascii_alphanumeric);
    if !name_is_valid || read_u32(payload, extension_length_at).ok()? != 4 || !extension_is_valid {
        return None;
    }
    let relative_offset = read_u64(payload, tail_at).ok()?;
    let entry = DirectoryEntry {
        name: String::from_utf8(name.to_vec()).ok()?,
        extension: String::from_utf8(extension.to_vec()).ok()?,
        frame_offset: FRAME_BASE.saturating_add(relative_offset),
        compressed_size: read_u64(payload, tail_at + 8).ok()?,
        decompressed_size: read_u64(payload, tail_at + 16).ok()?,
    };
    Some((entry, tail_at + ENTRY_FIXED_TAIL_BYTES))
}

fn validate_entries(
    entries: &[DirectoryEntry],
    trailer_offset: u64,
    file_size: u64,
) -> Result<(), SaveStatsError> {
    let mut previous_end = HEADER_SIZE;
    let mut total_decompressed: u64 = 0;
    for entry in entries {
        if entry.compressed_size == 0
            || entry.frame_offset < previous_end
            || entry.frame_end() > trailer_offset
        {
            return Err(SaveStatsError::Corrupt);
        }
        if entry.decompressed_size > part_bound(entry.compressed_size) {
            return Err(SaveStatsError::OutOfBounds);
        }
        total_decompressed = total_decompressed.saturating_add(entry.decompressed_size);
        previous_end = entry.frame_end();
    }
    if total_decompressed > save_bound(file_size) {
        return Err(SaveStatsError::OutOfBounds);
    }
    Ok(())
}

fn build_regions(
    entries: &[DirectoryEntry],
    trailer_offset: u64,
) -> Result<Vec<Region>, SaveStatsError> {
    let mut regions: Vec<Region> = Vec::new();
    let mut add_region = |region: Region| {
        if regions.iter().any(|existing| existing.name == region.name) {
            return Err(SaveStatsError::Corrupt);
        }
        regions.push(region);
        Ok(())
    };
    let mut cursor = HEADER_SIZE;
    let mut previous_name = String::from("header");
    for entry in entries {
        if entry.frame_offset > cursor {
            add_region(Region {
                name: format!("{UNLISTED_REGION_PREFIX}{previous_name}"),
                start: cursor,
                end: entry.frame_offset,
            })?;
        }
        if entry.is_section() {
            add_region(Region {
                name: entry.name.clone(),
                start: entry.frame_offset,
                end: entry.frame_end(),
            })?;
            previous_name = entry.name.clone();
        } else {
            previous_name = format!("{}{}", entry.name, entry.extension);
        }
        cursor = entry.frame_end();
    }
    if trailer_offset > cursor {
        add_region(Region {
            name: format!("{UNLISTED_REGION_PREFIX}{previous_name}"),
            start: cursor,
            end: trailer_offset,
        })?;
    }
    Ok(regions)
}

fn part_bound(compressed_size: u64) -> u64 {
    EXPANSION_ALLOWANCE.max(compressed_size.saturating_mul(MAX_PART_EXPANSION))
}

fn save_bound(file_size: u64) -> u64 {
    EXPANSION_ALLOWANCE.max(file_size.saturating_mul(MAX_SAVE_EXPANSION))
}

/// A section's decompressed bytes, never past its declared size or its part bound.
pub(super) fn read_section(
    source: &mut impl SaveSource,
    index: &ContainerIndex,
    name: &str,
) -> Result<Vec<u8>, SaveStatsError> {
    let entry = index.section(name)?;
    let compressed = source.read_at(entry.frame_offset, entry.compressed_size)?;
    decompress_frame(
        &compressed,
        Some(entry.decompressed_size),
        part_bound(entry.compressed_size),
    )
}

/// A section that opening a save decodes, refusing one far larger than any save writes.
pub(super) fn read_metadata_section(
    source: &mut impl SaveSource,
    index: &ContainerIndex,
    name: &str,
) -> Result<Vec<u8>, SaveStatsError> {
    let entry = index.section(name)?;
    if entry.compressed_size > MAX_METADATA_SECTION_BYTES
        || entry.decompressed_size > MAX_METADATA_SECTION_BYTES
    {
        return Err(SaveStatsError::OutOfBounds);
    }
    read_section(source, index, name)
}

/// One frame of a region, located from frame and block headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct FrameSpan {
    pub offset: u64,
    pub size: u64,
    pub skippable: bool,
}

/// Every frame in a region, located without decompressing.
pub(super) fn walk_frames(
    source: &mut impl SaveSource,
    index: &ContainerIndex,
    region_name: &str,
) -> Result<Vec<FrameSpan>, SaveStatsError> {
    let region = index.region(region_name)?;
    let (start, end) = (region.start, region.end);
    let mut spans = Vec::new();
    let mut position = start;
    while position < end {
        if spans.len() >= MAX_FRAMES_PER_REGION {
            return Err(SaveStatsError::OutOfBounds);
        }
        if position + 4 > end {
            return Err(SaveStatsError::Corrupt);
        }
        let magic = read_u32(&source.read_at(position, 4)?, 0)?;
        if (SKIPPABLE_MAGIC_FIRST..=SKIPPABLE_MAGIC_LAST).contains(&magic) {
            if position + 8 > end {
                return Err(SaveStatsError::Corrupt);
            }
            let size = 8 + u64::from(read_u32(&source.read_at(position + 4, 4)?, 0)?);
            if position + size > end {
                return Err(SaveStatsError::Corrupt);
            }
            spans.push(FrameSpan {
                offset: position,
                size,
                skippable: true,
            });
            position += size;
            continue;
        }
        if magic != ZSTD_FRAME_MAGIC {
            return Err(SaveStatsError::Corrupt);
        }
        let frame_end = compressed_frame_end(source, position, end)?;
        spans.push(FrameSpan {
            offset: position,
            size: frame_end - position,
            skippable: false,
        });
        position = frame_end;
    }
    Ok(spans)
}

/// The end of the compressed frame at `frame_start`, found from its frame and block headers.
fn compressed_frame_end(
    source: &mut impl SaveSource,
    frame_start: u64,
    region_end: u64,
) -> Result<u64, SaveStatsError> {
    if frame_start + 5 > region_end {
        return Err(SaveStatsError::Corrupt);
    }
    let descriptor = source.read_at(frame_start + 4, 1)?[0];
    if descriptor & 0x08 != 0 {
        return Err(SaveStatsError::Corrupt);
    }
    let single_segment = descriptor & 0x20 != 0;
    let dictionary_id_bytes = [0, 1, 2, 4][usize::from(descriptor & 0x03)];
    let content_size_bytes = [u64::from(single_segment), 2, 4, 8][usize::from(descriptor >> 6)];
    let header_size = 5 + u64::from(!single_segment) + dictionary_id_bytes + content_size_bytes;
    let mut block_position = frame_start + header_size;
    loop {
        if block_position + 3 > region_end {
            return Err(SaveStatsError::Corrupt);
        }
        let header_bytes = source.read_at(block_position, 3)?;
        let block_header =
            u32::from_le_bytes([header_bytes[0], header_bytes[1], header_bytes[2], 0]);
        let block_type = (block_header >> 1) & 0x03;
        let block_size = block_header >> 3;
        if block_type == BLOCK_TYPE_RESERVED || block_size > ZSTD_MAX_BLOCK_BYTES {
            return Err(SaveStatsError::Corrupt);
        }
        let body_bytes = if block_type == BLOCK_TYPE_RLE {
            1
        } else {
            block_size
        };
        block_position += 3 + u64::from(body_bytes);
        if block_header & 0x01 != 0 {
            break;
        }
    }
    if descriptor & 0x04 != 0 {
        block_position += 4;
    }
    if block_position > region_end {
        return Err(SaveStatsError::Corrupt);
    }
    Ok(block_position)
}

/// Each compressed frame of a region with at most the first `length` bytes of its body, so a
/// reader can pick its frame without decompressing the rest of the region.
pub(super) fn region_frame_heads(
    source: &mut impl SaveSource,
    index: &ContainerIndex,
    region_name: &str,
    length: usize,
) -> Result<Vec<(FrameSpan, Vec<u8>)>, SaveStatsError> {
    let spans = walk_frames(source, index, region_name)?;
    let mut heads = Vec::new();
    for span in spans.into_iter().filter(|span| !span.skippable) {
        let prefix = source.read_at(span.offset, span.size.min(HEAD_READ_BYTES))?;
        heads.push((span, decompress_head(&prefix, length)?));
    }
    Ok(heads)
}

/// Decompresses the chosen frames of a region. They count against the whole-save bound after
/// the directory's declared output, so no run of frames can decompress past it.
pub(super) fn read_chosen_region_frames(
    source: &mut impl SaveSource,
    index: &ContainerIndex,
    spans: &[FrameSpan],
) -> Result<Vec<Vec<u8>>, SaveStatsError> {
    let mut counted = index.declared_total();
    let mut frames = Vec::with_capacity(spans.len());
    for span in spans {
        let compressed = source.read_at(span.offset, span.size)?;
        let save_left = save_bound(index.file_size).saturating_sub(counted);
        let cap = part_bound(span.size).min(save_left);
        let frame = decompress_frame(&compressed, None, cap)?;
        counted += frame.len() as u64;
        frames.push(frame);
    }
    Ok(frames)
}

/// Decompresses exactly one Zstandard frame without producing more than `cap` bytes.
///
/// `expected` is a size the directory declares. A size the frame itself declares is checked
/// against `expected` and `cap` before decompression and sizes the output buffer; without
/// one, the buffer grows with the output and the decoder stops one byte past the limit.
fn decompress_frame(
    compressed: &[u8],
    expected: Option<u64>,
    cap: u64,
) -> Result<Vec<u8>, SaveStatsError> {
    if read_u32(compressed, 0)? != ZSTD_FRAME_MAGIC
        || zstd::zstd_safe::find_frame_compressed_size(compressed) != Ok(compressed.len())
    {
        return Err(SaveStatsError::Corrupt);
    }
    let declared =
        zstd::zstd_safe::get_frame_content_size(compressed).map_err(|_| SaveStatsError::Corrupt)?;
    if let Some(declared) = declared {
        if expected.is_some_and(|expected| expected != declared) {
            return Err(SaveStatsError::Corrupt);
        }
        if declared > cap {
            return Err(SaveStatsError::OutOfBounds);
        }
    }
    let limit = expected.map_or(cap, |expected| expected.min(cap));
    let decoder = zstd::stream::read::Decoder::with_buffer(compressed)
        .map_err(|_| SaveStatsError::Corrupt)?
        .single_frame();
    let mut output = buffer_for(declared.unwrap_or(0))?;
    decoder
        .take(limit.saturating_add(1))
        .read_to_end(&mut output)
        .map_err(|_| SaveStatsError::Corrupt)?;
    let produced = output.len() as u64;
    if produced > limit {
        return Err(if expected.map_or(true, |expected| expected > cap) {
            SaveStatsError::OutOfBounds
        } else {
            SaveStatsError::Corrupt
        });
    }
    if expected.is_some_and(|expected| expected != produced) {
        return Err(SaveStatsError::Corrupt);
    }
    Ok(output)
}

/// At most the first `wanted` bytes of the frame starting `compressed`, which may be a
/// truncated prefix of that frame.
fn decompress_head(compressed: &[u8], wanted: usize) -> Result<Vec<u8>, SaveStatsError> {
    let mut decoder = zstd::stream::read::Decoder::with_buffer(compressed)
        .map_err(|_| SaveStatsError::Corrupt)?
        .single_frame();
    let mut head = vec![0; wanted];
    let mut filled = 0;
    while filled < wanted {
        match decoder.read(&mut head[filled..]) {
            Ok(0) => break,
            Ok(read) => filled += read,
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(_) => return Err(SaveStatsError::Corrupt),
        }
    }
    head.truncate(filled);
    Ok(head)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::super::test_support::{
        build_container, fm27_container, fm27_sections, frame_declaring_content_size, write_save,
        FM27_BUILD,
    };
    use super::*;

    #[test]
    fn whole_file_read_returns_the_file_bytes() {
        let directory = tempfile::tempdir().expect("temp dir");
        let content = fm27_container(FM27_BUILD);
        let path = write_save(directory.path(), &content);

        assert_eq!(read_save_bytes(&path), Ok(content));
    }

    #[test]
    fn metadata_change_during_a_read_is_reported() {
        let directory = tempfile::tempdir().expect("temp dir");
        let path = write_save(directory.path(), &fm27_container(FM27_BUILD));
        let calls = Cell::new(0);
        let stamp_changing_after_the_read = |file: &File| {
            calls.set(calls.get() + 1);
            let stamp = FileStamp::of(file)?;
            Ok(if calls.get() == 1 {
                stamp
            } else {
                FileStamp {
                    modified: Some(SystemTime::UNIX_EPOCH),
                    ..stamp
                }
            })
        };

        assert_eq!(
            read_save_bytes_with(&path, stamp_changing_after_the_read),
            Err(SaveStatsError::ChangedDuringRead)
        );
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn file_over_the_size_cap_is_rejected_before_it_is_read() {
        let directory = tempfile::tempdir().expect("temp dir");
        let path = write_save(directory.path(), b"");
        File::options()
            .write(true)
            .open(&path)
            .and_then(|file| file.set_len(MAX_SAVE_FILE_BYTES + 1))
            .expect("extend sparse fixture");

        assert_eq!(read_save_bytes(&path), Err(SaveStatsError::OutOfBounds));
    }

    #[test]
    fn unlisted_region_frames_are_located_and_decompressed() {
        let content = fm27_container(FM27_BUILD);
        let mut source: &[u8] = &content;
        let index = read_index(&mut source, content.len() as u64).expect("index");
        let region = "unlisted_after_non_pl_hist_ls";

        let spans = walk_frames(&mut source, &index, region).expect("spans");
        let heads = region_frame_heads(&mut source, &index, region, 4).expect("heads");
        let frames = read_chosen_region_frames(
            &mut source,
            &index,
            &heads.iter().map(|(span, _)| *span).collect::<Vec<_>>(),
        )
        .expect("frames");

        assert_eq!(
            spans.iter().map(|span| span.skippable).collect::<Vec<_>>(),
            [false, false, true]
        );
        assert_eq!(
            heads
                .iter()
                .map(|(_, head)| head.as_slice())
                .collect::<Vec<_>>(),
            [b"AAAA", b"BBBB"]
        );
        assert_eq!(frames, [vec![b'A'; 32], vec![b'B'; 48]]);
    }

    #[test]
    fn region_frame_declaring_more_than_the_save_bound_is_rejected_before_decompression() {
        let mut sections = fm27_sections(FM27_BUILD);
        sections[4].unlisted_frames_after = vec![frame_declaring_content_size(1 << 40)];
        let content = build_container(&sections, &[]);
        let mut source: &[u8] = &content;
        let index = read_index(&mut source, content.len() as u64).expect("index");
        let spans =
            walk_frames(&mut source, &index, "unlisted_after_non_pl_hist_ls").expect("spans");

        assert_eq!(
            read_chosen_region_frames(&mut source, &index, &spans),
            Err(SaveStatsError::OutOfBounds)
        );
    }
}
