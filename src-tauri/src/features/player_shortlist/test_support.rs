//! Synthetic FM shortlist files for tests, written from the format recorded in the feature
//! design.
//!
//! Nothing here uses the reader's constants or parsers, so a wrong offset in the reader fails
//! tests built here. The files hold fictional content.

use std::path::{Path, PathBuf};

use ctr::cipher::{KeyIvInit, StreamCipher};

const CONTAINER_MAGIC: &[u8] = b"\x02\x01afe.";
const INDEX_MAGIC: &[u8] = b"\x02\x01fmf.";
const FORMAT_VERSION: u16 = 8;
const MEMBER_BASE: usize = 26;

const TEST_KEY: [u8; 16] = *b"0123456789abcdef";
const TEST_IV: [u8; 16] = *b"fedcba9876543210";

/// A player list payload: the `.slf` tag, a 7-byte header, the name, the IDs, and one
/// trailing byte, as FM writes it.
pub(super) fn player_list_payload(name: &str, uids: &[u32]) -> Vec<u8> {
    let mut payload = b"\x03\x01fls.".to_vec();
    payload.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0]);
    payload.extend_from_slice(&(name.len() as u32).to_le_bytes());
    payload.extend_from_slice(name.as_bytes());
    payload.extend_from_slice(&(uids.len() as u32).to_le_bytes());
    for uid in uids {
        payload.extend_from_slice(&uid.to_le_bytes());
    }
    payload.push(0);
    payload
}

fn compress(bytes: &[u8]) -> Vec<u8> {
    zstd::bulk::compress(bytes, 3).expect("compress fixture frame")
}

/// Stored member bytes: the key and IV sizes, the key, the IV, then the compressed payload
/// encrypted with AES-128-CTR.
fn encrypted_member(payload: &[u8], key: [u8; 16], iv: [u8; 16]) -> Vec<u8> {
    let mut data = compress(payload);
    ctr::Ctr128BE::<aes::Aes128>::new(&key.into(), &iv.into()).apply_keystream(&mut data);
    let mut stored = 16_u32.to_le_bytes().to_vec();
    stored.extend_from_slice(&16_u32.to_le_bytes());
    stored.extend_from_slice(&key);
    stored.extend_from_slice(&iv);
    stored.extend(data);
    stored
}

/// One member of a synthetic file. The index declares `declared_offset` and
/// `declared_unpacked_size` when set, and the real values otherwise.
pub(super) struct ShortlistMember {
    pub folder: Option<&'static str>,
    pub name: &'static str,
    pub extension: &'static str,
    pub stored: Vec<u8>,
    pub unpacked_size: u64,
    pub declared_offset: Option<u64>,
    pub declared_unpacked_size: Option<u64>,
}

impl ShortlistMember {
    /// A member holding `payload`, encrypted with the test key and IV.
    pub(super) fn new(name: &'static str, extension: &'static str, payload: &[u8]) -> Self {
        Self {
            folder: None,
            name,
            extension,
            stored: encrypted_member(payload, TEST_KEY, TEST_IV),
            unpacked_size: payload.len() as u64,
            declared_offset: None,
            declared_unpacked_size: None,
        }
    }

    pub(super) fn in_folder(self, folder: &'static str) -> Self {
        Self {
            folder: Some(folder),
            ..self
        }
    }
}

fn length_prefixed(text: &str) -> Vec<u8> {
    let mut bytes = (text.len() as u32).to_le_bytes().to_vec();
    bytes.extend_from_slice(text.as_bytes());
    bytes
}

fn member_record(member: &ShortlistMember, offset: u64) -> Vec<u8> {
    let mut record = length_prefixed(member.name);
    record.extend(length_prefixed(member.extension));
    let declared_offset = member.declared_offset.unwrap_or(offset);
    let unpacked_size = member
        .declared_unpacked_size
        .unwrap_or(member.unpacked_size);
    for value in [declared_offset, member.stored.len() as u64, unpacked_size] {
        record.extend_from_slice(&value.to_le_bytes());
    }
    record.extend_from_slice(&[0xA5; 16]);
    record
}

/// The decoded index. The reader does not decode folder records, so this folder layout is
/// invented: a named folder is its length-prefixed name and member count, then its members.
fn index_payload(root_name: &str, members: &[ShortlistMember], offsets: &[u64]) -> Vec<u8> {
    let records: Vec<(Option<&str>, Vec<u8>)> = members
        .iter()
        .zip(offsets)
        .map(|(member, offset)| (member.folder, member_record(member, *offset)))
        .collect();
    let mut payload = length_prefixed(root_name);
    let root: Vec<&Vec<u8>> = records
        .iter()
        .filter(|(folder, _)| folder.is_none())
        .map(|(_, record)| record)
        .collect();
    payload.extend_from_slice(&(root.len() as u32).to_le_bytes());
    payload.extend(root.into_iter().flatten());
    let mut folders: Vec<&str> = records.iter().filter_map(|(folder, _)| *folder).collect();
    folders.dedup();
    payload.extend_from_slice(&(folders.len() as u32).to_le_bytes());
    for folder in folders {
        let inside: Vec<&Vec<u8>> = records
            .iter()
            .filter(|(member_folder, _)| *member_folder == Some(folder))
            .map(|(_, record)| record)
            .collect();
        payload.extend(length_prefixed(folder));
        payload.extend_from_slice(&(inside.len() as u32).to_le_bytes());
        payload.extend(inside.into_iter().flatten());
    }
    payload
}

/// A complete `.fmf` file holding `members` in order.
pub(super) fn build_shortlist_file(root_name: &str, members: &[ShortlistMember]) -> Vec<u8> {
    let mut stored = Vec::new();
    let mut offsets = Vec::new();
    for member in members {
        offsets.push(stored.len() as u64);
        stored.extend_from_slice(&member.stored);
    }
    let index_start = (MEMBER_BASE + stored.len()) as u64;
    let mut file = CONTAINER_MAGIC.to_vec();
    file.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    file.push(0);
    file.extend_from_slice(&(index_start - 9).to_le_bytes());
    file.resize(MEMBER_BASE, 0);
    file.extend(stored);
    file.extend_from_slice(INDEX_MAGIC);
    file.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    file.push(0);
    file.extend(compress(&index_payload(root_name, members, &offsets)));
    file
}

/// A player shortlist file as FM exports it: a details member and the player list.
pub(super) fn player_shortlist_file(name: &str, uids: &[u32]) -> Vec<u8> {
    build_shortlist_file(
        name,
        &[
            ShortlistMember::new("details", ".aom", b"fictional details"),
            ShortlistMember::new("shortlist", ".slf", &player_list_payload(name, uids)),
        ],
    )
}

pub(super) fn write_shortlist_file(directory: &Path, content: &[u8]) -> PathBuf {
    let path = directory.join("Example Shortlist.fmf");
    std::fs::write(&path, content).expect("write shortlist file");
    path
}
