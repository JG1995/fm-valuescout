//! The manager's player and staff shortlists, read from the `scout_man` section.
//!
//! fmsave reads no shortlists, so this layout is this project's own. It was worked out on
//! FM27 builds `27.0.0+2493426` and `27.0.1+2494687`, where five lists of 1 to 42 players
//! read from a save matched the game's shortlist screen, and a staff shortlist of 13 matched
//! the game's staff screen in the same order.
//!
//! The section is its header, a list count, and that many list records, newest first within
//! each kind. The game keeps a default list of each kind with an empty name, and the manager
//! may add named ones. The kinds are not grouped: a named staff shortlist can come before
//! the player shortlists.
//!
//! A list record is a fixed header holding the list's kind, a length-prefixed name, a
//! filter, the members, and the list's settings and view. The filter's length is not known,
//! so the member block is found by its tag inside a bounded window that must hold no other
//! list's filter. Everything after the tag is walked by its stored lengths, and the record
//! must end on its end marker, so a byte out of place fails the read instead of shifting
//! what later records read.
//!
//! A player filter is followed by player members and a non-player filter by staff members,
//! each under its own tag. Lists of other kinds, such as the scouting lists, are walked and
//! left out.

use std::collections::{HashMap, HashSet};

use super::container::{find_marker, read_u16, read_u32};
use super::players::PlayerRecord;
use super::{SaveShortlist, SaveStatsError};

pub(super) const SECTION: &str = "scout_man";

const SECTION_MAGIC: &[u8] = b"\x03\x01tad.";
const SECTION_SCHEMA: u16 = 31;
/// The section repeats its schema before the list count.
const LIST_VERSION_AT: usize = 8;
const LIST_COUNT_AT: usize = 10;
const RECORDS_AT: usize = 14;
const MAX_LISTS: u32 = 4_096;

const KIND_OFFSET: usize = 6;
const PLAYER_SHORTLIST_KIND: u32 = 0;
const STAFF_SHORTLIST_KIND: u32 = 8;
const NAME_LENGTH_OFFSET: usize = 33;
const MAX_NAME_BYTES: u32 = 256;
/// Three fixed bytes, the four-byte tag of what the filter selects, then the filter tag.
const FILTER_LEAD: &[u8] = &[1, 0, 1];
const PLAYER_FILTER_TAG: &[u8] = b"frlp";
const STAFF_FILTER_TAG: &[u8] = b"flpn";
const SUBJECT_TAG_BYTES: usize = 4;
const FILTER_TAG: &[u8] = b"tlif";
const MAX_FILTER_BYTES: usize = 64 * 1024;

const PLAYER_MEMBERS_TAG: &[u8] = b"rSrP\0";
const STAFF_MEMBERS_TAG: &[u8] = b"manP\x01";
const MAX_MEMBERS: u32 = 100_000;
/// A marker byte, the member's key, two dates, and nine more bytes.
const ENTRY_BYTES: usize = 22;
const ENTRY_MARKER: u8 = 2;

const SETTINGS_BYTES: usize = 14;
const MAX_VIEW_COLUMNS: u32 = 4_096;
const VIEW_COLUMN_BYTES: usize = 4;
const AFTER_VIEW_BYTES: usize = 4;
const RECORD_END: u16 = 30;

/// The lists of a `scout_man` section that the app reads.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct SaveLists {
    /// The player shortlists in the save's order.
    pub player_shortlists: Vec<SaveShortlist>,
    /// The key of each person on any staff shortlist, once, in the save's order. A key is
    /// the one that closes the person's object in `game_db`.
    pub staff_keys: Vec<u32>,
}

/// The player shortlists and staff shortlist members of a `scout_man` section. A player is
/// stored by the key the season statistics use: the player's pindex plus one.
///
/// A listed player with no Unique ID is left out. A listed player that is no player in
/// `players` fails the read, as does any record that does not walk. A record whose filter is
/// of neither kind ends the read, keeping the lists before it.
pub(super) fn read_lists(
    section: &[u8],
    players: &[PlayerRecord],
) -> Result<SaveLists, SaveStatsError> {
    if !section.starts_with(SECTION_MAGIC) {
        return Err(SaveStatsError::Corrupt);
    }
    if read_u16(section, SECTION_MAGIC.len())? != SECTION_SCHEMA {
        return Err(SaveStatsError::UnsupportedBuild);
    }
    if read_u16(section, LIST_VERSION_AT)? != SECTION_SCHEMA {
        return Err(SaveStatsError::Corrupt);
    }
    let list_count = read_u32(section, LIST_COUNT_AT)?;
    if list_count > MAX_LISTS {
        return Err(SaveStatsError::Corrupt);
    }
    let unique_ids: HashMap<u32, Option<u32>> = players
        .iter()
        .filter_map(|player| Some((player.pindex.checked_add(1)?, player.unique_id)))
        .collect();

    let mut lists = SaveLists {
        player_shortlists: Vec::new(),
        staff_keys: Vec::new(),
    };
    let mut listed_staff = HashSet::new();
    let mut position = RECORDS_AT;
    for _ in 0..list_count {
        let Some(record) = read_record(section, position)? else {
            break;
        };
        match record.kind {
            ListKind::PlayerShortlist => lists.player_shortlists.push(SaveShortlist {
                name: record.name.to_owned(),
                player_unique_ids: listed_players(&record.member_keys, &unique_ids)?,
            }),
            ListKind::StaffShortlist => lists.staff_keys.extend(
                record
                    .member_keys
                    .iter()
                    .filter(|key| listed_staff.insert(**key)),
            ),
            ListKind::Other => {}
        }
        position = record.end;
    }
    Ok(lists)
}

enum ListKind {
    PlayerShortlist,
    StaffShortlist,
    Other,
}

/// One walked list record.
struct ListRecord<'a> {
    kind: ListKind,
    name: &'a str,
    member_keys: Vec<u32>,
    /// The position after the record.
    end: usize,
}

/// The list record that starts at `start`. `None` means its filter selects neither players
/// nor staff, so its members cannot be found.
fn read_record(section: &[u8], start: usize) -> Result<Option<ListRecord<'_>>, SaveStatsError> {
    let kind = read_u32(section, start + KIND_OFFSET)?;
    let name_length = read_u32(section, start + NAME_LENGTH_OFFSET)?;
    if name_length > MAX_NAME_BYTES {
        return Err(SaveStatsError::Corrupt);
    }
    let name_at = start + NAME_LENGTH_OFFSET + 4;
    let lead_at = name_at + name_length as usize;
    let subject_at = lead_at + FILTER_LEAD.len();
    let filter_at = subject_at + SUBJECT_TAG_BYTES + FILTER_TAG.len();
    let lead = section
        .get(lead_at..filter_at)
        .ok_or(SaveStatsError::Corrupt)?;
    if !lead.starts_with(FILTER_LEAD) || !lead.ends_with(FILTER_TAG) {
        return Err(SaveStatsError::Corrupt);
    }
    let subject = &section[subject_at..subject_at + SUBJECT_TAG_BYTES];
    let (members_tag, kind) = if subject == PLAYER_FILTER_TAG {
        let shortlist = kind == PLAYER_SHORTLIST_KIND;
        let kind = if shortlist {
            ListKind::PlayerShortlist
        } else {
            ListKind::Other
        };
        (PLAYER_MEMBERS_TAG, kind)
    } else if subject == STAFF_FILTER_TAG {
        let shortlist = kind == STAFF_SHORTLIST_KIND;
        let kind = if shortlist {
            ListKind::StaffShortlist
        } else {
            ListKind::Other
        };
        (STAFF_MEMBERS_TAG, kind)
    } else {
        return Ok(None);
    };
    let name =
        std::str::from_utf8(&section[name_at..lead_at]).map_err(|_| SaveStatsError::Corrupt)?;

    let window_end = section.len().min(filter_at + MAX_FILTER_BYTES);
    let members_tag_at =
        find_marker(section, members_tag, filter_at, window_end)?.ok_or(SaveStatsError::Corrupt)?;
    // Another filter tag before the member tag means the tag found is a later list's.
    if find_marker(section, FILTER_TAG, filter_at, members_tag_at)?.is_some() {
        return Err(SaveStatsError::Corrupt);
    }
    let member_count_at = members_tag_at + members_tag.len();
    let member_count = read_u32(section, member_count_at)?;
    if member_count > MAX_MEMBERS {
        return Err(SaveStatsError::Corrupt);
    }
    let entries_at = member_count_at + 4;
    let entries_end = entries_at + member_count as usize * ENTRY_BYTES;
    let entries = section
        .get(entries_at..entries_end)
        .ok_or(SaveStatsError::Corrupt)?;
    let mut member_keys = Vec::with_capacity(member_count as usize);
    for entry in entries.chunks_exact(ENTRY_BYTES) {
        if entry[0] != ENTRY_MARKER {
            return Err(SaveStatsError::Corrupt);
        }
        member_keys.push(u32::from_le_bytes([entry[1], entry[2], entry[3], entry[4]]));
    }

    let view_columns = read_u32(section, entries_end + SETTINGS_BYTES)?;
    if view_columns > MAX_VIEW_COLUMNS {
        return Err(SaveStatsError::Corrupt);
    }
    let end_at = entries_end
        + SETTINGS_BYTES
        + 4
        + view_columns as usize * VIEW_COLUMN_BYTES
        + AFTER_VIEW_BYTES;
    if read_u16(section, end_at)? != RECORD_END {
        return Err(SaveStatsError::Corrupt);
    }
    Ok(Some(ListRecord {
        kind,
        name,
        member_keys,
        end: end_at + 2,
    }))
}

/// Each listed player's Unique ID once, in the list's order.
fn listed_players(
    member_keys: &[u32],
    unique_ids: &HashMap<u32, Option<u32>>,
) -> Result<Vec<u32>, SaveStatsError> {
    let mut player_unique_ids = Vec::new();
    let mut listed = HashSet::new();
    for key in member_keys {
        let unique_id = unique_ids.get(key).ok_or(SaveStatsError::Corrupt)?;
        if let Some(unique_id) = unique_id.filter(|unique_id| listed.insert(*unique_id)) {
            player_unique_ids.push(unique_id);
        }
    }
    Ok(player_unique_ids)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{
        save_with_scout_section, scout_section, season_stats_body, season_stats_save, stat_line,
        write_save, ExampleList, ExamplePlayer, ExampleRecord, FM27_BUILD, UNREGISTERED_FM27_BUILD,
    };
    use super::super::{read_season_stats, SaveSeasonStats};
    use super::*;

    const LEAGUE: usize = 1;
    const SCHEMA: u16 = 31;

    /// Three players: the first two have Unique IDs 1,500 and 2,500, the third has none.
    /// Their shortlist keys are 11, 21, and 31.
    fn players() -> [ExamplePlayer; 3] {
        [
            ExamplePlayer::outfield(10, 1_000, Some(1_500)),
            ExamplePlayer::outfield(20, 2_000, Some(2_500)),
            ExamplePlayer::outfield(30, 3_000, None),
        ]
    }

    fn records() -> Vec<PlayerRecord> {
        players()
            .iter()
            .map(|player| PlayerRecord {
                pindex: player.pindex,
                unique_id: player.unique_id,
                goalkeeper: false,
            })
            .collect()
    }

    fn read_save(build: &str, scout: Option<Vec<u8>>) -> SaveSeasonStats {
        let played = stat_line(&[("minutes", 90.0), ("starts", 1.0)]);
        let season_stats =
            season_stats_body(&[ExampleRecord::with_lines(11, &[(LEAGUE, played)])], 15);
        let save = match scout {
            Some(scout) => save_with_scout_section(build, &players(), &season_stats, scout),
            None => season_stats_save(build, 4000, &players(), &season_stats),
        };
        let directory = tempfile::tempdir().expect("temp dir");
        read_season_stats(&write_save(directory.path(), &save)).expect("statistics")
    }

    fn shortlist(name: &str, player_unique_ids: &[u32]) -> SaveShortlist {
        SaveShortlist {
            name: name.to_owned(),
            player_unique_ids: player_unique_ids.to_vec(),
        }
    }

    #[test]
    fn the_player_shortlists_are_read_in_save_order_wherever_other_lists_sit() {
        let mut scouting_list = ExampleList::shortlist("", &[21]);
        scouting_list.kind = 6;
        let scout = scout_section(
            &[
                // A named staff shortlist is newer than every player shortlist here.
                ExampleList::staff_shortlist("Staff targets", &[]),
                // A repeated member counts once, and one with no Unique ID is left out.
                ExampleList::shortlist("002 - DR", &[21, 11, 21, 31]),
                scouting_list,
                ExampleList::shortlist("001 - GK", &[]),
                ExampleList::shortlist("", &[11]),
                ExampleList::staff(),
            ],
            SCHEMA,
        );

        let stats = read_save(FM27_BUILD, Some(scout));

        assert_eq!(
            stats.shortlists,
            Ok(vec![
                shortlist("002 - DR", &[2_500, 1_500]),
                shortlist("001 - GK", &[]),
                shortlist("", &[1_500]),
            ])
        );
        assert_eq!(stats.players.len(), 1);
    }

    #[test]
    fn every_staff_shortlist_gives_its_members_keys_once_in_save_order() {
        let mut staff_search = ExampleList::staff_shortlist("", &[900]);
        staff_search.kind = 9;
        let section = scout_section(
            &[
                ExampleList::staff_shortlist("Staff targets", &[503, 501]),
                ExampleList::shortlist("Targets", &[11]),
                ExampleList::staff_shortlist("", &[501, 502]),
                staff_search,
            ],
            SCHEMA,
        );

        let lists = read_lists(&section, &records()).expect("lists");

        assert_eq!(lists.staff_keys, [503, 501, 502]);
        assert_eq!(lists.player_shortlists, [shortlist("Targets", &[1_500])]);
    }

    #[test]
    fn a_list_of_an_unknown_subject_ends_the_read_and_keeps_the_lists_before_it() {
        let mut unknown = ExampleList::shortlist("", &[11]);
        unknown.filter_tag = *b"fbul";
        let section = scout_section(
            &[
                ExampleList::shortlist("Targets", &[11]),
                unknown,
                ExampleList::shortlist("Later", &[21]),
            ],
            SCHEMA,
        );

        let lists = read_lists(&section, &records()).expect("lists");

        assert_eq!(lists.player_shortlists, [shortlist("Targets", &[1_500])]);
    }

    #[test]
    fn a_save_whose_shortlists_cannot_be_read_still_gives_its_statistics() {
        let other_schema = scout_section(&[ExampleList::shortlist("Targets", &[11])], 32);

        for (case, scout, expected) in [
            ("no section", None, SaveStatsError::Corrupt),
            (
                "another schema",
                Some(other_schema),
                SaveStatsError::UnsupportedBuild,
            ),
        ] {
            let stats = read_save(FM27_BUILD, scout);

            assert_eq!(stats.shortlists, Err(expected), "{case}");
            assert_eq!(stats.players.len(), 1, "{case}");
        }
    }

    #[test]
    fn a_section_that_does_not_walk_returns_no_shortlists() {
        let lists = [
            ExampleList::shortlist("Targets", &[11, 21]),
            ExampleList::staff_shortlist("", &[501]),
        ];
        let section = scout_section(&lists, SCHEMA);
        let tag_at = |tag: &[u8]| {
            section
                .windows(tag.len())
                .position(|window| window == tag)
                .expect("member tag")
        };
        let member_tag_at = tag_at(b"rSrP");
        let staff_tag_at = tag_at(b"manP");
        let change = |at: usize, byte: u8| {
            let mut damaged = section.clone();
            damaged[at] = byte;
            damaged
        };

        let unknown_player = scout_section(&[ExampleList::shortlist("Targets", &[99])], SCHEMA);
        // Two lists are counted where one is stored.
        let mut record_count_past_the_end = scout_section(&lists[..1], SCHEMA);
        record_count_past_the_end[10..14].copy_from_slice(&2_u32.to_le_bytes());
        let mut ends_inside_the_members = section.clone();
        ends_inside_the_members.truncate(member_tag_at + 9 + 22 + 5);
        // A section of one list ends with that list's end marker.
        let mut no_record_end = scout_section(&lists[..1], SCHEMA);
        let end_marker_at = no_record_end.len() - 2;
        no_record_end[end_marker_at] = 31;

        for (case, damaged) in [
            ("inner version", change(8, 30)),
            ("no member tag", change(member_tag_at, b'x')),
            // The member count's low byte, then the first entry's marker.
            ("member count past the end", change(member_tag_at + 5, 200)),
            ("entry marker", change(member_tag_at + 9, 3)),
            ("staff member tag", change(staff_tag_at, b'x')),
            ("staff entry marker", change(staff_tag_at + 9, 3)),
            ("record end", no_record_end),
            ("unknown player", unknown_player),
            ("record count past the end", record_count_past_the_end),
            ("ends inside the members", ends_inside_the_members),
        ] {
            assert_eq!(
                read_lists(&damaged, &records()),
                Err(SaveStatsError::Corrupt),
                "{case}"
            );
        }
        assert!(read_lists(&section, &records()).is_ok());
    }

    #[test]
    fn an_unregistered_build_reads_shortlists_and_reports_a_failed_walk_as_unsupported() {
        let section = scout_section(&[ExampleList::shortlist("Targets", &[11])], SCHEMA);
        let mut damaged = section.clone();
        damaged[8] = 30;

        assert_eq!(
            read_save(UNREGISTERED_FM27_BUILD, Some(section)).shortlists,
            Ok(vec![shortlist("Targets", &[1_500])])
        );
        assert_eq!(
            read_save(UNREGISTERED_FM27_BUILD, Some(damaged)).shortlists,
            Err(SaveStatsError::UnsupportedBuild)
        );
    }
}
