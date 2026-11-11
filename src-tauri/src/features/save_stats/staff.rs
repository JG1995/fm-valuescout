//! The people on the manager's staff shortlists: who they are, the job each prefers, and
//! their coaching qualifications, read from `game_db`.
//!
//! fmsave reads no staff, so this layout is this project's own. It was worked out on FM27
//! build `27.0.1+2494687`, where the 13 staff of a shortlist read from a save matched the
//! game's staff screen by Unique ID, Preferred Job, and Coaching Qualifications.
//!
//! Every person's object in `game_db` ends with a 12-byte trailer: the person's key, then
//! the Unique ID twice. A list in `scout_man` stores a member by that key. The object's own
//! bytes lie between the trailer of the key before it and its own trailer, so a member is
//! found by those two trailers. The game stores an allocated Unique ID, one from 2,000,000,000
//! up, once in some trailers, with another word after it.
//!
//! A non-player's object opens with its kind, a counted block, three reputations, both
//! abilities, and the type of job the person holds. Further in come two counted lists, a
//! flag saying whether 15 job ratings follow, the ratings, a counted block, four dates at
//! fixed places, and the coaching qualification. Nothing before the flag has a known length,
//! so the ratings are found as the one place in the object where all of that holds.
//!
//! Other bytes read as a trailer too, most often for a low key. So a trailer only opens an
//! object when a non-player's opening follows it, an object ends at the nearest trailer that
//! leaves it readable, a trailer followed by another object's kind is preferred to one that
//! is not, and the work spent on one key is bounded.
//!
//! The game shows the job with the highest rating as the Preferred Job. Which job it shows
//! when several share the highest rating is known only for the pairs listed at `TIE_ORDER`.

use std::collections::{HashMap, HashSet};

use super::container::read_u32;
use super::{decode_date, SaveShortlistedStaff, SaveStatsError};

const TRAILER_BYTES: usize = 12;
/// The value a trailer's Unique ID holds when it refers to nothing.
const MISSING_REFERENCE: u32 = 0xFFFF_FFFF;
/// Unique IDs from here up are allocated by the game, and a trailer may store one once.
const ALLOCATED_UNIQUE_ID_MINIMUM: u32 = 2_000_000_000;
/// The developer's reference save holds 316,826 people.
const MAX_PERSON_KEY: u32 = 16 * 1024 * 1024 - 1;
const MAX_TRAILER_CANDIDATES: usize = 1024 * 1024;
/// The largest object in the reference save is 2,026 bytes.
const MAX_OBJECT_BYTES: usize = 16 * 1024;
/// How many candidate objects are read for one key. The reference save's worst key, the
/// first person's, has about 400.
const MAX_OBJECT_READS_PER_KEY: usize = 1_024;

/// The kinds of object seen in the reference save are 1 to 9, 16, and 18. One of them
/// follows every trailer but the last person's.
const OBJECT_KINDS: std::ops::RangeInclusive<u8> = 1..=31;
/// An object of this kind is a non-player's, which opens with the fields below.
const NON_PLAYER_KIND: u8 = 1;
/// A player who also holds non-player data. Its object opens with the player's fields.
const PLAYER_AND_NON_PLAYER_KIND: u8 = 3;
/// The kind, then a u16 count of seven-byte entries, in a non-player's object only.
const LEADING_COUNT_AT: usize = 1;
const LEADING_ENTRIES_AT: usize = 3;
const LEADING_ENTRY_BYTES: usize = 7;
const MAX_LEADING_ENTRIES: usize = 64;
/// Three reputations, then the current and potential ability, two bytes each.
const REPUTATIONS: usize = 3;
const MAX_REPUTATION: u16 = 10_000;
const CURRENT_ABILITY_RANGE: std::ops::RangeInclusive<u16> = 1..=200;
const POTENTIAL_ABILITY_RANGE: std::ops::RangeInclusive<i16> = -10..=200;
/// The job type follows the abilities in a non-player's object.
const JOB_TYPE_AFTER_REPUTATIONS: usize = 10;
const RECRUITMENT_ANALYST_JOB_TYPE: u8 = 58;
/// A chief doctor and a doctor. Neither has job ratings.
const DOCTOR_JOB_TYPES: [u8; 2] = [38, 46];

const JOB_RATINGS: usize = 15;
const JOB_RATING_RANGE: std::ops::RangeInclusive<u8> = 1..=20;
/// The first list: a zero byte, a one-byte count, then a u64 id and a level for each entry.
const MAX_FIRST_LIST_ENTRIES: usize = 64;
const MAX_FIRST_LIST_LEVEL: u8 = 20;
/// The second list: a u32 count, then entries as long as the first list's.
const MAX_SECOND_LIST_ENTRIES: usize = 255;
const LIST_ENTRY_BYTES: usize = 9;
/// The block after the ratings: a one-byte count of four-byte entries, then three bytes.
const MAX_BLOCK_ENTRIES: usize = 64;
const BLOCK_ENTRY_BYTES: usize = 4;
const AFTER_BLOCK_BYTES: usize = 3;
/// Where a date starts after the block, and where the qualification is.
const DATES_AFTER_BLOCK: [usize; 4] = [0, 5, 9, 21];
const QUALIFICATION_AFTER_BLOCK: usize = 25;
/// A date field the game has not set: day 1 of 1900.
const UNSET_DATE: [u8; 4] = [1, 0, 0x6C, 0x07];

/// The game's name for each rated job, in stored order.
const RATED_JOBS: [&str; JOB_RATINGS] = [
    "Manager",
    "Assistant Manager",
    "Coach",
    "Physio",
    "Scout",
    "Goalkeeping Coach",
    "Fitness Coach",
    "Chairman",
    "Director of Football",
    "Head of Youth Development",
    "Loan Manager",
    "Technical Director",
    "Performance Analyst",
    "Sports Scientist",
    "Set Piece Coach",
];
const PERFORMANCE_ANALYST: usize = 12;
/// Which rated job is the Preferred Job when several share the highest rating: the earliest
/// here. The game's screen showed Manager and Assistant Manager over Coach, and Coach over
/// Performance Analyst. An earlier export of another save, matched by Unique ID, showed Head
/// of Youth Development over Manager, Assistant Manager, Coach, and Scout; Assistant Manager
/// over Director of Football and Fitness Coach; Scout over Coach and Performance Analyst;
/// Sports Scientist over Physio; Physio over Fitness Coach; and Fitness Coach over
/// Performance Analyst. Every other pair is placed by judgment.
const TIE_ORDER: [usize; JOB_RATINGS] = [9, 0, 1, 8, 11, 4, 13, 3, 5, 6, 14, 10, 2, 12, 7];
/// The game's name for each stored coaching qualification.
const QUALIFICATIONS: [&str; 8] = [
    "No Qualifications",
    "Continental Pro Licence",
    "Continental A Licence",
    "Continental B Licence",
    "Continental C Licence",
    "National A Licence",
    "National B Licence",
    "National C Licence",
];
/// The highest stored qualification. Its name in the game is not known.
const MAX_QUALIFICATION: u8 = 8;
const UNKNOWN: &str = "-";

/// The person behind each of `keys`, in that order, each Unique ID once.
///
/// A key that gives nobody, or whose candidate objects give different people, fails the read,
/// so no person is attributed by a guess.
pub(super) fn read_shortlisted_staff(
    game_db: &[u8],
    keys: &[u32],
) -> Result<Vec<SaveShortlistedStaff>, SaveStatsError> {
    if keys.is_empty() {
        return Ok(Vec::new());
    }
    if keys.contains(&0) || keys.iter().any(|key| *key > MAX_PERSON_KEY) {
        return Err(SaveStatsError::Corrupt);
    }
    let trailers = find_trailers(game_db, keys)?;
    let mut staff = Vec::with_capacity(keys.len());
    let mut listed = HashSet::new();
    for key in keys {
        let person = person_of_key(game_db, *key, &trailers)?;
        if listed.insert(person.unique_id) {
            staff.push(person);
        }
    }
    Ok(staff)
}

/// A set of person keys, tested once for every byte of `game_db`.
struct KeySet {
    bits: Vec<u64>,
}

impl KeySet {
    fn of(keys: impl Iterator<Item = u32> + Clone) -> Self {
        let highest = keys.clone().max().unwrap_or(0);
        let mut bits = vec![0_u64; highest as usize / 64 + 1];
        for key in keys {
            bits[key as usize / 64] |= 1 << (key % 64);
        }
        Self { bits }
    }

    fn contains(&self, key: u32) -> bool {
        self.bits
            .get(key as usize / 64)
            .is_some_and(|word| word & (1 << (key % 64)) != 0)
    }
}

/// The places in `game_db` that read as a trailer bounding a listed person's object.
#[derive(Default)]
struct Trailers {
    /// By key: each trailer that stores its Unique ID twice, ascending.
    closing: HashMap<u32, Vec<ClosingTrailer>>,
    /// By key: where a trailer starts that a non-player's opening follows, ascending.
    opening: HashMap<u32, Vec<usize>>,
}

struct ClosingTrailer {
    at: usize,
    unique_id: u32,
    /// Whether another object's kind follows, as it does after every person but the last.
    object_follows: bool,
}

/// Every place that reads as the trailer closing a listed key's object, or as the trailer
/// before one. Other bytes can read as a trailer too, so each is only a candidate until the
/// object it bounds is read.
fn find_trailers(game_db: &[u8], keys: &[u32]) -> Result<Trailers, SaveStatsError> {
    let closing = KeySet::of(keys.iter().copied());
    let opening = KeySet::of(keys.iter().map(|key| key - 1));
    let word = |window: &[u8], at: usize| {
        u32::from_le_bytes([window[at], window[at + 1], window[at + 2], window[at + 3]])
    };
    let mut trailers = Trailers::default();
    let mut found = 0;
    for (at, window) in game_db.windows(TRAILER_BYTES).enumerate() {
        let key = word(window, 0);
        let (closes, opens) = (closing.contains(key), opening.contains(key));
        if !closes && !opens {
            continue;
        }
        let (unique_id, copy) = (word(window, 4), word(window, 8));
        if unique_id == 0 || unique_id == MISSING_REFERENCE {
            continue;
        }
        let stored_twice = unique_id == copy;
        if closes && stored_twice {
            trailers
                .closing
                .entry(key)
                .or_default()
                .push(ClosingTrailer {
                    at,
                    unique_id,
                    object_follows: game_db
                        .get(at + TRAILER_BYTES)
                        .is_some_and(|kind| OBJECT_KINDS.contains(kind)),
                });
            found += 1;
        }
        if opens
            && (stored_twice || unique_id >= ALLOCATED_UNIQUE_ID_MINIMUM)
            && reputations_at(&game_db[at + TRAILER_BYTES..]).is_some()
        {
            trailers.opening.entry(key).or_default().push(at);
            found += 1;
        }
        if found > MAX_TRAILER_CANDIDATES {
            return Err(SaveStatsError::OutOfBounds);
        }
    }
    Ok(trailers)
}

/// The person whose object closes with `key`. Trailers that another object follows are
/// tried first, and the rest only when those give nobody.
fn person_of_key(
    game_db: &[u8],
    key: u32,
    trailers: &Trailers,
) -> Result<SaveShortlistedStaff, SaveStatsError> {
    let closing = trailers.closing.get(&key).map_or(&[][..], Vec::as_slice);
    let opening = trailers
        .opening
        .get(&(key - 1))
        .map_or(&[][..], Vec::as_slice);
    let mut reads = 0;
    for object_must_follow in [true, false] {
        let person = person_between(game_db, opening, closing, object_must_follow, &mut reads)?;
        if let Some(person) = person {
            return Ok(person);
        }
    }
    Err(SaveStatsError::Corrupt)
}

/// The one person the objects between `opening` and `closing` trailers give. Each opening
/// trailer's object ends at the nearest closing trailer that leaves it readable, because a
/// later one would take in the next object. Objects that give different people fail the read.
fn person_between(
    game_db: &[u8],
    opening: &[usize],
    closing: &[ClosingTrailer],
    object_must_follow: bool,
    reads: &mut usize,
) -> Result<Option<SaveShortlistedStaff>, SaveStatsError> {
    let mut person: Option<SaveShortlistedStaff> = None;
    for object_at in opening.iter().map(|at| at + TRAILER_BYTES) {
        let first = closing.partition_point(|trailer| trailer.at <= object_at);
        let ends = closing[first..]
            .iter()
            .take_while(|trailer| trailer.at - object_at <= MAX_OBJECT_BYTES)
            .filter(|trailer| trailer.object_follows || !object_must_follow);
        for end in ends {
            *reads += 1;
            if *reads > MAX_OBJECT_READS_PER_KEY {
                return Err(SaveStatsError::OutOfBounds);
            }
            let Some(facts) = read_object(&game_db[object_at..end.at])? else {
                continue;
            };
            let found = SaveShortlistedStaff {
                unique_id: end.unique_id,
                preferred_job: facts.preferred_job,
                coaching_qualifications: facts.coaching_qualifications,
            };
            if person.as_ref().is_some_and(|person| *person != found) {
                return Err(SaveStatsError::Corrupt);
            }
            person = Some(found);
            break;
        }
    }
    Ok(person)
}

struct StaffFacts {
    preferred_job: &'static str,
    coaching_qualifications: &'static str,
}

/// What a non-player's object says, or `None` when `object` does not read as one.
fn read_object(object: &[u8]) -> Result<Option<StaffFacts>, SaveStatsError> {
    let Some(reputations_at) = reputations_at(object) else {
        return Ok(None);
    };
    // Only a non-player's own object is known to hold the job type there.
    let job_type = (object[0] == NON_PLAYER_KIND)
        .then(|| {
            object
                .get(reputations_at + JOB_TYPE_AFTER_REPUTATIONS)
                .copied()
        })
        .flatten();
    let mut rated = Vec::new();
    let mut unrated = Vec::new();
    for flag_at in 0..object.len() {
        let found = match object[flag_at] {
            1 => &mut rated,
            0 => &mut unrated,
            _ => continue,
        };
        let ratings = usize::from(object[flag_at] == 1) * JOB_RATINGS;
        let Some(qualification) = qualification_after(object, flag_at + 1 + ratings)? else {
            continue;
        };
        let ratings = &object[flag_at + 1..flag_at + 1 + ratings];
        if ratings
            .iter()
            .all(|rating| JOB_RATING_RANGE.contains(rating))
            && lists_end_at(object, flag_at)
        {
            found.push((ratings, qualification));
        }
    }
    let facts = match (rated.as_slice(), unrated.as_slice()) {
        ([(ratings, qualification)], _) => StaffFacts {
            preferred_job: preferred_job(ratings, job_type),
            coaching_qualifications: qualification_name(*qualification),
        },
        ([], [(_, qualification), others @ ..]) => {
            let doctor = job_type.is_some_and(|job_type| DOCTOR_JOB_TYPES.contains(&job_type));
            let agreed = others.iter().all(|(_, other)| other == qualification);
            StaffFacts {
                preferred_job: if doctor { "Doctor" } else { UNKNOWN },
                coaching_qualifications: if agreed {
                    qualification_name(*qualification)
                } else {
                    UNKNOWN
                },
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(facts))
}

/// Where the reputations start, when `object` opens as a non-player's does: a kind that holds
/// non-player data, reputations the game can hold, and both abilities in range.
fn reputations_at(object: &[u8]) -> Option<usize> {
    let word = |at: usize| Some(u16::from_le_bytes([*object.get(at)?, *object.get(at + 1)?]));
    let at = match *object.first()? {
        NON_PLAYER_KIND => {
            let leading_entries = usize::from(word(LEADING_COUNT_AT)?);
            if leading_entries > MAX_LEADING_ENTRIES {
                return None;
            }
            LEADING_ENTRIES_AT + leading_entries * LEADING_ENTRY_BYTES
        }
        PLAYER_AND_NON_PLAYER_KIND => 1,
        _ => return None,
    };
    for reputation in 0..REPUTATIONS {
        if word(at + 2 * reputation)? > MAX_REPUTATION {
            return None;
        }
    }
    let current_ability = word(at + 2 * REPUTATIONS)?;
    let potential_ability = word(at + 2 * REPUTATIONS + 2)? as i16;
    (CURRENT_ABILITY_RANGE.contains(&current_ability)
        && POTENTIAL_ABILITY_RANGE.contains(&potential_ability))
    .then_some(at)
}

/// The coaching qualification, when the counted block at `block_at` is followed by its three
/// bytes, a date at each fixed place, and a qualification the game stores.
fn qualification_after(object: &[u8], block_at: usize) -> Result<Option<u8>, SaveStatsError> {
    let Some(&entries) = object.get(block_at) else {
        return Ok(None);
    };
    if usize::from(entries) > MAX_BLOCK_ENTRIES {
        return Ok(None);
    }
    let after_block = block_at + 1 + usize::from(entries) * BLOCK_ENTRY_BYTES + AFTER_BLOCK_BYTES;
    let Some(&qualification) = object.get(after_block + QUALIFICATION_AFTER_BLOCK) else {
        return Ok(None);
    };
    if qualification > MAX_QUALIFICATION {
        return Ok(None);
    }
    for date_at in DATES_AFTER_BLOCK {
        let date_at = after_block + date_at;
        if object[date_at..date_at + 4] != UNSET_DATE && decode_date(object, date_at)?.is_none() {
            return Ok(None);
        }
    }
    Ok(Some(qualification))
}

/// Whether the two counted lists end where the ratings flag is at `flag_at`.
fn lists_end_at(object: &[u8], flag_at: usize) -> bool {
    (0..=MAX_SECOND_LIST_ENTRIES).any(|second_entries| {
        let Some(second_at) = flag_at.checked_sub(4 + second_entries * LIST_ENTRY_BYTES) else {
            return false;
        };
        read_u32(object, second_at) == Ok(second_entries as u32)
            && (0..=MAX_FIRST_LIST_ENTRIES)
                .any(|first_entries| first_list_ends_at(object, second_at, first_entries))
    })
}

/// Whether a first list of `entries` entries, with its zero byte and count, ends at `end`.
fn first_list_ends_at(object: &[u8], end: usize, entries: usize) -> bool {
    let Some(count_at) = end.checked_sub(1 + entries * LIST_ENTRY_BYTES) else {
        return false;
    };
    let Some(zero_at) = count_at.checked_sub(1) else {
        return false;
    };
    object[zero_at] == 0
        && usize::from(object[count_at]) == entries
        && object[count_at + 1..end]
            .chunks_exact(LIST_ENTRY_BYTES)
            .all(|entry| entry[4..8] == [0; 4] && entry[8] <= MAX_FIRST_LIST_LEVEL)
}

/// The game's name for the job with the highest rating. Recruitment and performance analysts
/// share one rating, so a person holding a recruitment analyst's job prefers that one.
fn preferred_job(ratings: &[u8], job_type: Option<u8>) -> &'static str {
    let highest = ratings.iter().copied().max();
    let job = TIE_ORDER
        .into_iter()
        .find(|job| ratings.get(*job).copied() == highest)
        .unwrap_or(PERFORMANCE_ANALYST);
    if job == PERFORMANCE_ANALYST && job_type == Some(RECRUITMENT_ANALYST_JOB_TYPE) {
        "Recruitment Analyst"
    } else {
        RATED_JOBS[job]
    }
}

fn qualification_name(qualification: u8) -> &'static str {
    QUALIFICATIONS
        .get(usize::from(qualification))
        .copied()
        .unwrap_or(UNKNOWN)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{
        game_db_payload_with_staff, save_with_staff, scout_section, season_stats_body, stat_line,
        write_save, ExampleList, ExamplePlayer, ExampleRecord, ExampleStaff, FM27_BUILD,
    };
    use super::super::{read_season_stats, SaveShortlist};
    use super::*;

    fn person(
        unique_id: u32,
        preferred_job: &'static str,
        qualifications: &'static str,
    ) -> SaveShortlistedStaff {
        SaveShortlistedStaff {
            unique_id,
            preferred_job,
            coaching_qualifications: qualifications,
        }
    }

    fn game_db(staff: &[ExampleStaff]) -> Vec<u8> {
        game_db_payload_with_staff(&[ExamplePlayer::outfield(10, 1_000, Some(1_500))], staff)
    }

    #[test]
    fn each_key_gives_its_person_with_the_preferred_job_and_the_qualification() {
        let physio = ExampleStaff {
            qualification: 3,
            ..ExampleStaff::preferring(510, 9_001, 4)
        };
        // A Unique ID the game allocated.
        let recruitment_analyst = ExampleStaff {
            job_type: 58,
            qualification: 7,
            ..ExampleStaff::preferring(520, 2_002_118_740, 13)
        };
        let performance_analyst = ExampleStaff::preferring(530, 9_003, 13);
        let doctor = ExampleStaff {
            job_type: 46,
            job_ratings: None,
            ..ExampleStaff::preferring(540, 9_004, 1)
        };
        let unrated = ExampleStaff {
            job_ratings: None,
            qualification: 8,
            ..ExampleStaff::preferring(550, 9_005, 1)
        };
        let game_db = game_db(&[
            physio,
            recruitment_analyst,
            performance_analyst,
            doctor,
            unrated,
        ]);

        // A key listed twice gives its person once.
        let staff = read_shortlisted_staff(&game_db, &[520, 510, 530, 520, 540, 550]);

        assert_eq!(
            staff,
            Ok(vec![
                person(2_002_118_740, "Recruitment Analyst", "National C Licence"),
                person(9_001, "Physio", "Continental B Licence"),
                person(9_003, "Performance Analyst", "No Qualifications"),
                person(9_004, "Doctor", "No Qualifications"),
                person(9_005, "-", "-"),
            ])
        );
        assert_eq!(read_shortlisted_staff(&game_db, &[]), Ok(Vec::new()));
    }

    #[test]
    fn every_rated_job_and_qualification_has_the_games_name() {
        let jobs = [
            "Manager",
            "Assistant Manager",
            "Coach",
            "Physio",
            "Scout",
            "Goalkeeping Coach",
            "Fitness Coach",
            "Chairman",
            "Director of Football",
            "Head of Youth Development",
            "Loan Manager",
            "Technical Director",
            "Performance Analyst",
            "Sports Scientist",
            "Set Piece Coach",
        ];
        let qualifications = [
            "No Qualifications",
            "Continental Pro Licence",
            "Continental A Licence",
            "Continental B Licence",
            "Continental C Licence",
            "National A Licence",
            "National B Licence",
            "National C Licence",
        ];
        let staff: Vec<ExampleStaff> = (0..jobs.len())
            .map(|slot| ExampleStaff {
                qualification: (slot % qualifications.len()) as u8,
                ..ExampleStaff::preferring(600 + 10 * slot as u32, 9_100 + slot as u32, slot + 1)
            })
            .collect();
        let keys: Vec<u32> = staff.iter().map(|person| person.key).collect();

        let read = read_shortlisted_staff(&game_db(&staff), &keys).expect("staff");

        let expected: Vec<SaveShortlistedStaff> = (0..jobs.len())
            .map(|slot| {
                person(
                    9_100 + slot as u32,
                    jobs[slot],
                    qualifications[slot % qualifications.len()],
                )
            })
            .collect();
        assert_eq!(read, expected);
    }

    #[test]
    fn jobs_sharing_the_highest_rating_follow_the_order_the_game_showed() {
        // Each case: the 1-based rated jobs sharing the highest rating, and the one preferred.
        for (tied, preferred) in [
            (&[1, 3][..], "Manager"),
            (&[2, 3], "Assistant Manager"),
            (&[3, 13], "Coach"),
            (&[1, 10], "Head of Youth Development"),
            (&[3, 5], "Scout"),
            (&[5, 13], "Scout"),
            (&[2, 9], "Assistant Manager"),
            (&[4, 14], "Sports Scientist"),
            (&[4, 7], "Physio"),
            (&[7, 13], "Fitness Coach"),
            (&[1, 2, 3], "Manager"),
        ] {
            let mut ratings = [5; JOB_RATINGS];
            for job in tied {
                ratings[job - 1] = 18;
            }

            assert_eq!(preferred_job(&ratings, None), preferred, "{tied:?}");
        }
    }

    #[test]
    fn a_key_whose_object_is_not_found_exactly_once_fails_the_read() {
        let listed = ExampleStaff::preferring(510, 9_001, 4);
        let sound = game_db(&[listed]);
        let trailer_at = sound.len() - TRAILER_BYTES;
        let change = |at: usize, byte: u8| {
            let mut damaged = sound.clone();
            damaged[at] = byte;
            damaged
        };
        let ratings_at = sound
            .windows(JOB_RATINGS)
            .rposition(|window| window == listed.job_ratings.expect("ratings"))
            .expect("job ratings");
        // Two objects closing with one key give it two people.
        let two_people = game_db(&[listed, ExampleStaff::preferring(510, 9_002, 4)]);
        // The object starts after the earlier object's trailer: its allocated Unique ID,
        // stored once, then another word.
        let earlier_unique_id_at = sound
            .windows(4)
            .position(|window| window == 2_100_000_000_u32.to_le_bytes())
            .expect("the earlier object's trailer");
        let player_object = change(earlier_unique_id_at + 8, 2);
        // The kind, an empty counted block, and three reputations lie before the abilities.
        let ability_at = earlier_unique_id_at + 8 + 9;

        for (case, game_db, keys) in [
            ("unknown key", sound.clone(), &[511][..]),
            ("key zero", sound.clone(), &[0]),
            ("key past the bound", sound.clone(), &[MAX_PERSON_KEY + 1]),
            ("no ability", change(ability_at, 0), &[510]),
            ("Unique ID stored once", change(trailer_at + 8, 1), &[510]),
            ("rating out of range", change(ratings_at, 21), &[510]),
            (
                "no qualification",
                change(ratings_at + JOB_RATINGS + 33, 9),
                &[510],
            ),
            ("a player's object", player_object, &[510]),
            ("two people with one key", two_people, &[510]),
        ] {
            assert_eq!(
                read_shortlisted_staff(&game_db, keys),
                Err(SaveStatsError::Corrupt),
                "{case}"
            );
        }
        assert!(read_shortlisted_staff(&sound, &[510]).is_ok());
    }

    #[test]
    fn bytes_that_only_read_as_trailers_do_not_change_who_a_key_gives() {
        let listed = ExampleStaff::preferring(510, 9_001, 4);
        let sound = game_db(&[listed]);
        // Bytes that read as the trailer before the listed key followed by a non-player's
        // opening, then bytes that read as the listed key's own trailer with another
        // Unique ID.
        let mut opening = 509_u32.to_le_bytes().to_vec();
        opening.extend_from_slice(&[7, 0, 0, 0, 7, 0, 0, 0]);
        opening.extend_from_slice(&[1, 0, 0, 1, 0, 1, 0, 1, 0, 50, 0, 60, 0, 0]);
        let mut closing = 510_u32.to_le_bytes().to_vec();
        closing.extend_from_slice(&[8, 0, 0, 0, 8, 0, 0, 0]);
        let crowded = |pairs: usize| {
            let mut game_db = sound.clone();
            game_db.extend(opening.repeat(pairs));
            game_db.extend(closing.repeat(pairs));
            game_db
        };

        // The person's object ends at its own trailer, not at a later one that would also
        // leave it readable.
        assert_eq!(
            read_shortlisted_staff(&crowded(8), &[510]),
            Ok(vec![person(9_001, "Physio", "No Qualifications")])
        );
        // The search for one key is bounded.
        assert_eq!(
            read_shortlisted_staff(&crowded(40), &[510]),
            Err(SaveStatsError::OutOfBounds)
        );
    }

    #[test]
    fn a_trailer_that_another_object_follows_is_preferred() {
        let listed = ExampleStaff::preferring(510, 9_001, 4);
        let mut game_db = game_db(&[listed]);
        // A player's object follows the listed person's.
        game_db.extend_from_slice(&[2, 0, 0, 0]);
        // Inside the listed person's object, after the job ratings, bytes read as the key's
        // trailer with another Unique ID. No object's kind follows them.
        let own_trailer_at = game_db
            .windows(8)
            .position(|window| {
                window == [9_001_u32.to_le_bytes(), 9_001_u32.to_le_bytes()].concat()
            })
            .expect("the listed person's trailer")
            - 4;
        let inside_at = own_trailer_at - 30;
        game_db[inside_at..inside_at + 4].copy_from_slice(&510_u32.to_le_bytes());
        game_db[inside_at + 4..inside_at + 12].copy_from_slice(&[9, 0, 0, 0, 9, 0, 0, 0]);
        game_db[inside_at + 12] = 0xFF;

        assert_eq!(
            read_shortlisted_staff(&game_db, &[510]),
            Ok(vec![person(9_001, "Physio", "No Qualifications")])
        );
    }

    #[test]
    fn a_save_gives_the_staff_of_every_staff_shortlist_beside_its_other_facts() {
        let players = [ExamplePlayer::outfield(10, 1_000, Some(1_500))];
        let staff = [
            ExampleStaff::preferring(510, 9_001, 5),
            ExampleStaff::preferring(520, 9_002, 2),
        ];
        let played = stat_line(&[("minutes", 90.0), ("starts", 1.0)]);
        let season_stats = season_stats_body(&[ExampleRecord::with_lines(11, &[(1, played)])], 15);
        let read = |lists: &[ExampleList]| {
            let save = save_with_staff(
                FM27_BUILD,
                &players,
                &staff,
                &season_stats,
                scout_section(lists, 31),
            );
            let directory = tempfile::tempdir().expect("temp dir");
            read_season_stats(&write_save(directory.path(), &save)).expect("statistics")
        };
        let targets = SaveShortlist {
            name: "Targets".to_owned(),
            player_unique_ids: vec![1_500],
        };

        let stats = read(&[
            ExampleList::staff_shortlist("Backroom", &[520]),
            ExampleList::shortlist("Targets", &[11]),
            ExampleList::staff_shortlist("", &[510, 520]),
        ]);

        assert_eq!(
            stats.staff_shortlist,
            Ok(vec![
                person(9_002, "Assistant Manager", "No Qualifications"),
                person(9_001, "Scout", "No Qualifications"),
            ])
        );
        assert_eq!(stats.shortlists, Ok(vec![targets.clone()]));

        // A listed person who cannot be read fails the staff alone.
        let stats = read(&[
            ExampleList::shortlist("Targets", &[11]),
            ExampleList::staff_shortlist("", &[999]),
        ]);

        assert_eq!(stats.staff_shortlist, Err(SaveStatsError::Corrupt));
        assert_eq!(stats.shortlists, Ok(vec![targets]));
        assert_eq!(stats.players.len(), 1);
    }
}
