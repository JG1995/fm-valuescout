//! Reads FM player shortlist files (`.fmf`), written from the format recorded in the
//! feature design.
//!
//! A file is a 26-byte header, the stored members, and a compressed index listing each
//! member. Each member is an AES-128-CTR encrypted Zstandard frame; a player shortlist has a
//! `.slf` member holding the list name and Unique IDs. The file is untrusted: it is read
//! once into a capped buffer, and every declared size is checked before it is used.

use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use ctr::cipher::{KeyIvInit, StreamCipher};
use serde::Serialize;

const MAX_FILE_BYTES: u64 = 1024 * 1024;
const MAX_UNPACKED_BYTES: u64 = 4 * 1024 * 1024;
const MAX_MEMBERS: usize = 64;
const MAX_ROOT_NAME_BYTES: u32 = 4096;
const MAX_MEMBER_NAME_BYTES: u32 = 256;
const MAX_EXTENSION_BYTES: u32 = 16;

const CONTAINER_MAGIC: &[u8] = b"\x02\x01afe.";
const INDEX_POINTER_AT: usize = 9;
const MEMBER_BASE: usize = 26;
const INDEX_MAGIC: &[u8] = b"\x02\x01fmf.";
const INDEX_FRAME_AFTER_TAG: usize = 9;
/// A member record's offset, stored size, and unpacked size, then 16 unused bytes.
const MEMBER_RECORD_TAIL_BYTES: usize = 40;
/// A member's key and IV sizes, its 16-byte key, then its 16-byte IV.
const MEMBER_HEADER_BYTES: usize = 40;
const CIPHER_PART_BYTES: u32 = 16;

const PLAYER_LIST_EXTENSION: &str = ".slf";
const PLAYER_LIST_TAG: &[u8] = b"\x03\x01fls.";
const PLAYER_LIST_HEADER_BYTES: usize = 7;

/// A player shortlist read from an FM file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlayerShortlistFile {
    pub name: String,
    /// Each Unique ID once, in file order.
    pub player_uids: Vec<i64>,
}

/// Why a shortlist file was rejected. No variant carries the file's path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ShortlistFileError {
    Unreadable,
    TooLarge,
    /// The container, index, decryption, or decompression is invalid.
    NotFmShortlist,
    /// A valid FM file without a player list, such as a view or a staff shortlist.
    NotPlayerShortlist,
    UnsupportedFormat,
}

use ShortlistFileError::{NotFmShortlist, NotPlayerShortlist, TooLarge, UnsupportedFormat};

/// One member listed in the index. `offset` is relative to the member region.
#[derive(Debug)]
struct IndexedMember {
    extension: String,
    offset: u64,
    stored_size: u64,
    unpacked_size: u64,
}

/// Reads the player list of the FM shortlist file at `path`, which is never modified.
pub(crate) fn read_player_shortlist(
    path: &Path,
) -> Result<PlayerShortlistFile, ShortlistFileError> {
    let bytes = read_capped(path)?;
    let index_start = index_start(&bytes)?;
    let members = read_index(&bytes[index_start..])?;
    let list = members
        .iter()
        .find(|member| member.extension.eq_ignore_ascii_case(PLAYER_LIST_EXTENSION))
        .ok_or(NotPlayerShortlist)?;
    let stored = member_bytes(&bytes[MEMBER_BASE..index_start], list)?;
    parse_player_list(&decode_member(stored, list.unpacked_size)?)
}

/// The whole file, read through a cap one byte past the limit so a larger file is detected
/// without reading it.
fn read_capped(path: &Path) -> Result<Vec<u8>, ShortlistFileError> {
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|file| file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes))
        .map_err(|_| ShortlistFileError::Unreadable)?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(TooLarge);
    }
    Ok(bytes)
}

/// Where the index starts, checked to lie after the header and to carry the index tag.
fn index_start(bytes: &[u8]) -> Result<usize, ShortlistFileError> {
    if !bytes.starts_with(CONTAINER_MAGIC) {
        return Err(NotFmShortlist);
    }
    read_u64(bytes, INDEX_POINTER_AT)
        .and_then(|distance| usize::try_from(distance).ok())
        .and_then(|distance| distance.checked_add(INDEX_POINTER_AT))
        .filter(|&start| {
            start >= MEMBER_BASE
                && bytes
                    .get(start..)
                    .is_some_and(|index| index.starts_with(INDEX_MAGIC))
        })
        .ok_or(NotFmShortlist)
}

/// The members the index lists, in index order.
///
/// Member records sit at the root or inside named folders. The reader does not decode
/// folder records: it recognizes a member record wherever one starts and steps over the
/// bytes between them.
fn read_index(index: &[u8]) -> Result<Vec<IndexedMember>, ShortlistFileError> {
    let frame = index.get(INDEX_FRAME_AFTER_TAG..).ok_or(NotFmShortlist)?;
    let mut payload = Vec::new();
    zstd::stream::read::Decoder::with_buffer(frame)
        .map_err(|_| NotFmShortlist)?
        .single_frame()
        .take(MAX_UNPACKED_BYTES + 1)
        .read_to_end(&mut payload)
        .map_err(|_| NotFmShortlist)?;
    if payload.len() as u64 > MAX_UNPACKED_BYTES {
        return Err(TooLarge);
    }
    let (_root_name, mut at) =
        length_prefixed(&payload, 0, MAX_ROOT_NAME_BYTES).ok_or(NotFmShortlist)?;
    let mut members = Vec::new();
    while at < payload.len() {
        match member_record(&payload, at) {
            Some((member, next)) => {
                if members.len() == MAX_MEMBERS {
                    return Err(TooLarge);
                }
                members.push(member);
                at = next;
            }
            None => at += 1,
        }
    }
    Ok(members)
}

/// The member record starting at `at`, and the position after it, or `None` when no record
/// starts there.
fn member_record(payload: &[u8], at: usize) -> Option<(IndexedMember, usize)> {
    let (name, at) = length_prefixed(payload, at, MAX_MEMBER_NAME_BYTES)?;
    if name.is_empty() || std::str::from_utf8(name).is_err() {
        return None;
    }
    let (extension, at) = length_prefixed(payload, at, MAX_EXTENSION_BYTES)?;
    let extension_is_valid = extension.len() >= 2
        && extension[0] == b'.'
        && extension[1..].iter().all(u8::is_ascii_alphanumeric);
    if !extension_is_valid {
        return None;
    }
    let end = at.checked_add(MEMBER_RECORD_TAIL_BYTES)?;
    let tail = payload.get(at..end)?;
    let member = IndexedMember {
        extension: String::from_utf8(extension.to_vec()).ok()?,
        offset: read_u64(tail, 0)?,
        stored_size: read_u64(tail, 8)?,
        unpacked_size: read_u64(tail, 16)?,
    };
    Some((member, end))
}

/// The member's stored bytes, which must lie inside the member region.
fn member_bytes<'a>(
    region: &'a [u8],
    member: &IndexedMember,
) -> Result<&'a [u8], ShortlistFileError> {
    let start = usize::try_from(member.offset).ok();
    let size = usize::try_from(member.stored_size).ok();
    start
        .zip(size)
        .and_then(|(start, size)| region.get(start..start.checked_add(size)?))
        .ok_or(NotFmShortlist)
}

/// Decrypts and decompresses a member's stored bytes into exactly `unpacked_size` bytes.
fn decode_member(stored: &[u8], unpacked_size: u64) -> Result<Vec<u8>, ShortlistFileError> {
    if unpacked_size > MAX_UNPACKED_BYTES {
        return Err(TooLarge);
    }
    let header = stored.get(..MEMBER_HEADER_BYTES).ok_or(NotFmShortlist)?;
    if read_u32(header, 0) != Some(CIPHER_PART_BYTES)
        || read_u32(header, 4) != Some(CIPHER_PART_BYTES)
    {
        return Err(NotFmShortlist);
    }
    let (key, iv) = (&header[8..24], &header[24..40]);
    let mut frame = stored[MEMBER_HEADER_BYTES..].to_vec();
    ctr::Ctr128BE::<aes::Aes128>::new_from_slices(key, iv)
        .map_err(|_| NotFmShortlist)?
        .apply_keystream(&mut frame);

    let expected_len = usize::try_from(unpacked_size).map_err(|_| TooLarge)?;
    let mut payload = Vec::with_capacity(expected_len);
    zstd::stream::read::Decoder::with_buffer(frame.as_slice())
        .map_err(|_| NotFmShortlist)?
        .single_frame()
        .take(unpacked_size + 1)
        .read_to_end(&mut payload)
        .map_err(|_| NotFmShortlist)?;
    if payload.len() != expected_len {
        return Err(NotFmShortlist);
    }
    Ok(payload)
}

/// The list name and Unique IDs of a decoded `.slf` member.
fn parse_player_list(payload: &[u8]) -> Result<PlayerShortlistFile, ShortlistFileError> {
    let body = payload
        .strip_prefix(PLAYER_LIST_TAG)
        .and_then(|rest| rest.get(PLAYER_LIST_HEADER_BYTES..))
        .ok_or(UnsupportedFormat)?;
    let (name, at) = length_prefixed(body, 0, u32::MAX).ok_or(UnsupportedFormat)?;
    let name = std::str::from_utf8(name).map_err(|_| UnsupportedFormat)?;
    let count = read_u32(body, at).ok_or(UnsupportedFormat)?;
    let uids = usize::try_from(count)
        .ok()
        .and_then(|count| count.checked_mul(4))
        .and_then(|length| {
            let start = at.checked_add(4)?;
            body.get(start..start.checked_add(length)?)
        })
        .ok_or(UnsupportedFormat)?;

    let mut seen = HashSet::new();
    let player_uids = uids
        .chunks_exact(4)
        .map(|uid| i64::from(u32::from_le_bytes([uid[0], uid[1], uid[2], uid[3]])))
        .filter(|uid| seen.insert(*uid))
        .collect();
    Ok(PlayerShortlistFile {
        name: name.to_owned(),
        player_uids,
    })
}

fn read_u32(bytes: &[u8], at: usize) -> Option<u32> {
    let field = bytes.get(at..at.checked_add(4)?)?;
    Some(u32::from_le_bytes(field.try_into().ok()?))
}

fn read_u64(bytes: &[u8], at: usize) -> Option<u64> {
    let field = bytes.get(at..at.checked_add(8)?)?;
    Some(u64::from_le_bytes(field.try_into().ok()?))
}

/// A u32 length of at most `max_length`, that many bytes, and the position after them.
fn length_prefixed(bytes: &[u8], at: usize, max_length: u32) -> Option<(&[u8], usize)> {
    let length = read_u32(bytes, at)?;
    if length > max_length {
        return None;
    }
    let start = at.checked_add(4)?;
    let end = start.checked_add(usize::try_from(length).ok()?)?;
    Some((bytes.get(start..end)?, end))
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{
        build_shortlist_file, player_list_payload, player_shortlist_file, write_shortlist_file,
        ShortlistMember,
    };
    use super::*;

    /// The `.slf` member of a real FM27 shortlist export, as stored in the file.
    const REAL_SLF_MEMBER_HEX: &str = "1000000010000000fdf3426d9856ca2682aa7821b96105bf3654aa6e99551a1b64206477aabc8750db9a59e12857be1c30f95b860f859307b6bfbd4962d3c83a03b07ee9e08786b5e72e6df7a9b0c4005d3e070ee3a89b46102ebf42bae24e8206c03d64f611b925f206079e286fab13fabf88d0cbf280";
    const REAL_SLF_UNPACKED_SIZE: u64 = 66;

    fn from_hex(hex: &str) -> Vec<u8> {
        (0..hex.len())
            .step_by(2)
            .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).expect("hex byte"))
            .collect()
    }

    fn read(content: &[u8]) -> Result<PlayerShortlistFile, ShortlistFileError> {
        let directory = tempfile::tempdir().expect("temp dir");
        read_player_shortlist(&write_shortlist_file(directory.path(), content))
    }

    #[test]
    fn decodes_the_player_list_of_a_real_fm27_export() {
        let payload =
            decode_member(&from_hex(REAL_SLF_MEMBER_HEX), REAL_SLF_UNPACKED_SIZE).expect("decode");
        let list = parse_player_list(&payload).expect("parse");

        assert_eq!(list.name, "002 - DR");
        assert_eq!(
            list.player_uids,
            [
                27100968, 2000396938, 2000305100, 2000273434, 70116131, 37056062, 37046274,
                28100123, 27119236,
            ]
        );
    }

    #[test]
    fn reads_the_player_list_from_the_root_or_a_folder_without_changing_the_file() {
        let payload = player_list_payload("Targets", &[7, 4_000_000_000, 7, 3]);
        for list_folder in [None, Some("lists")] {
            let mut list = ShortlistMember::new("Targets", ".slf", &payload);
            list.folder = list_folder;
            let content = build_shortlist_file(
                "Targets",
                &[
                    ShortlistMember::new("details", ".aom", b"fictional details"),
                    ShortlistMember::new("picture", ".png", b"fictional picture")
                        .in_folder("images"),
                    list,
                ],
            );
            let directory = tempfile::tempdir().expect("temp dir");
            let path = write_shortlist_file(directory.path(), &content);

            let shortlist = read_player_shortlist(&path).expect("read shortlist");

            assert_eq!(
                shortlist,
                PlayerShortlistFile {
                    name: "Targets".to_owned(),
                    player_uids: vec![7, 4_000_000_000, 3],
                },
                "list folder {list_folder:?}"
            );
            assert_eq!(std::fs::read(&path).expect("reread file"), content);
        }
    }

    #[test]
    fn rejects_files_that_are_not_readable_player_shortlists() {
        let list = || player_list_payload("Targets", &[1, 2]);
        let valid = player_shortlist_file("Targets", &[1, 2]);
        let mut bad_magic = valid.clone();
        bad_magic[2] = b'x';
        let index_start = valid
            .windows(6)
            .rposition(|window| window == b"\x02\x01fmf.")
            .expect("index tag");
        let mut wrong_index_tag = valid.clone();
        wrong_index_tag[index_start + 2] = b'x';
        let mut truncated_count = list();
        let count_at = truncated_count.len() - 4 * 2 - 1 - 4;
        truncated_count[count_at..count_at + 4].copy_from_slice(&3_u32.to_le_bytes());
        let mut staff_style_tag = list();
        staff_style_tag[0] = 0x02;
        let slf = |payload: &[u8]| ShortlistMember::new("Targets", ".slf", payload);

        let cases: Vec<(&str, Vec<u8>, ShortlistFileError)> = vec![
            ("oversized file", vec![0; 1024 * 1024 + 1], TooLarge),
            ("bad magic", bad_magic, NotFmShortlist),
            ("wrong index tag", wrong_index_tag, NotFmShortlist),
            (
                "member offset past the member region",
                build_shortlist_file(
                    "Targets",
                    &[ShortlistMember {
                        declared_offset: Some(10_000),
                        ..slf(&list())
                    }],
                ),
                NotFmShortlist,
            ),
            (
                "unpacked size mismatch",
                build_shortlist_file(
                    "Targets",
                    &[ShortlistMember {
                        declared_unpacked_size: Some(list().len() as u64 + 1),
                        ..slf(&list())
                    }],
                ),
                NotFmShortlist,
            ),
            (
                "unpacked size over the cap",
                build_shortlist_file(
                    "Targets",
                    &[ShortlistMember {
                        declared_unpacked_size: Some(4 * 1024 * 1024 + 1),
                        ..slf(&list())
                    }],
                ),
                TooLarge,
            ),
            (
                "count larger than the payload",
                build_shortlist_file("Targets", &[slf(&truncated_count)]),
                UnsupportedFormat,
            ),
            (
                "only a view member",
                build_shortlist_file(
                    "Targets",
                    &[ShortlistMember::new("Scouting", ".view", b"fictional view")],
                ),
                NotPlayerShortlist,
            ),
            (
                "02 01 fls. list",
                build_shortlist_file("Targets", &[slf(&staff_style_tag)]),
                UnsupportedFormat,
            ),
        ];

        for (case, content, expected) in cases {
            assert_eq!(read(&content), Err(expected), "{case}");
        }
    }

    #[test]
    fn rejects_a_missing_file_as_unreadable() {
        let directory = tempfile::tempdir().expect("temp dir");

        assert_eq!(
            read_player_shortlist(&directory.path().join("missing.fmf")),
            Err(ShortlistFileError::Unreadable)
        );
    }

    /// Reads the private shortlist file named by `FM_VALUESCOUT_SHORTLIST_FILE` and prints
    /// only its name and IDs, never its path.
    #[test]
    #[ignore = "reads a private FM shortlist file"]
    fn reads_the_private_shortlist_file() {
        let path = std::env::var_os("FM_VALUESCOUT_SHORTLIST_FILE")
            .expect("set FM_VALUESCOUT_SHORTLIST_FILE to an FM player shortlist file");

        let shortlist = read_player_shortlist(Path::new(&path))
            .unwrap_or_else(|error| panic!("the shortlist file was rejected: {error:?}"));

        println!("name: {}", shortlist.name);
        println!("count: {}", shortlist.player_uids.len());
        println!("uids: {:?}", shortlist.player_uids);
    }
}
