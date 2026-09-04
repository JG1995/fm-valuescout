#![cfg_attr(not(test), allow(dead_code))]

use std::collections::HashSet;
use std::path::Path;

use csv::{ReaderBuilder, StringRecord, Trim};
use rusqlite::{params, Connection};
use serde::Serialize;

use super::parser::detect_delimiter;
use super::service::{
    capture_active_import_context, read_csv_file, revalidate_active_import_context,
    ActiveImportContext, CsvImportServiceError,
};
use super::CsvImportError;

pub(crate) const MAX_PLAYER_SHORTLIST_ROWS: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedPlayerShortlist {
    pub total_rows: usize,
    pub player_uids: Vec<i64>,
    pub skipped_invalid: usize,
    pub skipped_duplicates: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlayerShortlistImportSummary {
    pub total_players: usize,
    pub stored_players: usize,
    pub skipped_players: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlayerShortlistImportContext {
    active: ActiveImportContext,
    player_uids: HashSet<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PlayerShortlistImportError {
    Import(CsvImportServiceError),
    InvalidCsv(CsvImportError),
    NoMatchingPlayers,
    Database,
}

impl std::fmt::Display for PlayerShortlistImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Import(error) => error.fmt(f),
            Self::InvalidCsv(error) => write!(f, "CSV file is invalid: {error}"),
            Self::NoMatchingPlayers => {
                write!(f, "CSV does not contain players in the current snapshot")
            }
            Self::Database => write!(f, "CSV import is unavailable"),
        }
    }
}

impl std::error::Error for PlayerShortlistImportError {}

impl From<CsvImportServiceError> for PlayerShortlistImportError {
    fn from(error: CsvImportServiceError) -> Self {
        Self::Import(error)
    }
}

pub(crate) fn capture_player_shortlist_import_context(
    conn: &Connection,
) -> Result<PlayerShortlistImportContext, PlayerShortlistImportError> {
    let active = capture_active_import_context(conn)?;
    let mut statement = conn
        .prepare("SELECT uid FROM players WHERE snapshot_id = ?1")
        .map_err(|_| PlayerShortlistImportError::Database)?;
    let player_uids = statement
        .query_map([active.snapshot_id], |row| row.get(0))
        .map_err(|_| PlayerShortlistImportError::Database)?
        .collect::<Result<HashSet<_>, _>>()
        .map_err(|_| PlayerShortlistImportError::Database)?;

    Ok(PlayerShortlistImportContext {
        active,
        player_uids,
    })
}

pub(crate) fn prepare_player_shortlist_import(
    path: &Path,
) -> Result<ParsedPlayerShortlist, PlayerShortlistImportError> {
    let input = read_csv_file(path)?;
    parse_player_shortlist(&input).map_err(PlayerShortlistImportError::InvalidCsv)
}

pub(crate) fn persist_player_shortlist_import(
    conn: &mut Connection,
    context: &PlayerShortlistImportContext,
    parsed: ParsedPlayerShortlist,
) -> Result<PlayerShortlistImportSummary, PlayerShortlistImportError> {
    let tx = conn
        .transaction()
        .map_err(|_| PlayerShortlistImportError::Database)?;
    revalidate_active_import_context(&tx, &context.active)?;

    let matching = parsed
        .player_uids
        .iter()
        .filter(|uid| context.player_uids.contains(uid))
        .collect::<Vec<_>>();
    if matching.is_empty() {
        return Err(PlayerShortlistImportError::NoMatchingPlayers);
    }

    tx.execute(
        "DELETE FROM player_shortlist_entries WHERE save_id = ?1",
        [context.active.save_id],
    )
    .map_err(|_| PlayerShortlistImportError::Database)?;
    for uid in &matching {
        tx.execute(
            "INSERT INTO player_shortlist_entries (save_id, player_uid) VALUES (?1, ?2)",
            params![context.active.save_id, uid],
        )
        .map_err(|_| PlayerShortlistImportError::Database)?;
    }

    tx.commit()
        .map_err(|_| PlayerShortlistImportError::Database)?;
    Ok(PlayerShortlistImportSummary {
        total_players: parsed.total_rows,
        stored_players: matching.len(),
        skipped_players: parsed.total_rows - matching.len(),
    })
}

pub(crate) fn parse_player_shortlist(input: &str) -> Result<ParsedPlayerShortlist, CsvImportError> {
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    if input.trim().is_empty() {
        return Err(CsvImportError::EmptyInput);
    }

    // A UID-only export has a single column under either delimiter, so the
    // established detection cannot pick a winner; either delimiter then parses
    // the same single field per line.
    let delimiter = match detect_delimiter(input) {
        Ok(delimiter) => delimiter,
        Err(CsvImportError::UnsupportedDialect)
            if input
                .lines()
                .next()
                .is_some_and(|header| !header.contains(',') && !header.contains(';')) =>
        {
            b','
        }
        Err(error) => return Err(error),
    };
    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter)
        .trim(Trim::All)
        .flexible(false)
        .from_reader(input.as_bytes());
    let headers = reader
        .headers()
        .map_err(|_| CsvImportError::MalformedCsv { row: 1 })?
        .clone();
    let uid_column = required_player_uid_column(&headers)?;

    let mut seen_uids = HashSet::new();
    let mut player_uids = Vec::new();
    let mut skipped_invalid = 0;
    let mut skipped_duplicates = 0;
    let mut total_rows = 0;
    for (record_index, record) in reader.records().enumerate() {
        if record_index >= MAX_PLAYER_SHORTLIST_ROWS {
            return Err(CsvImportError::TooManyRows {
                limit: MAX_PLAYER_SHORTLIST_ROWS,
            });
        }
        total_rows += 1;
        let row = record_index + 2;
        let record = record.map_err(|_| CsvImportError::MalformedCsv { row })?;
        let Some(player_uid) = parse_player_uid(record.get(uid_column).unwrap_or_default()) else {
            skipped_invalid += 1;
            continue;
        };
        if !seen_uids.insert(player_uid) {
            skipped_duplicates += 1;
            continue;
        }
        player_uids.push(player_uid);
    }

    Ok(ParsedPlayerShortlist {
        total_rows,
        player_uids,
        skipped_invalid,
        skipped_duplicates,
    })
}

fn required_player_uid_column(headers: &StringRecord) -> Result<usize, CsvImportError> {
    let matches = headers
        .iter()
        .enumerate()
        .filter_map(|(index, header)| (header.trim() == "Player UID").then_some(index))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => Err(CsvImportError::MissingRequiredHeader("Player UID")),
        [index] => Ok(*index),
        _ => Err(CsvImportError::DuplicateHeader("Player UID")),
    }
}

fn parse_player_uid(value: &str) -> Option<i64> {
    value
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|uid| *uid > 0)
        .map(i64::from)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use rusqlite::{params, Connection};

    use crate::db::migrations;

    use super::*;

    #[test]
    fn parses_bom_either_delimiter_and_any_header_order_while_ignoring_other_columns() {
        let parsed = parse_player_shortlist(
            "\u{feff}Name,Player UID,Club\nAlice,77,Club A\nBob,88,Club B\n",
        )
        .expect("parse comma-delimited shortlist with BOM");

        assert_eq!(MAX_PLAYER_SHORTLIST_ROWS, 10_000);
        assert_eq!(parsed.total_rows, 2);
        assert_eq!(parsed.player_uids, vec![77, 88]);
        assert_eq!(parsed.skipped_invalid, 0);
        assert_eq!(parsed.skipped_duplicates, 0);

        let parsed = parse_player_shortlist("Player UID;Name\n77;Alice\n")
            .expect("parse semicolon-delimited shortlist");
        assert_eq!(parsed.player_uids, vec![77]);
    }

    #[test]
    fn rejects_missing_header_and_malformed_csv() {
        assert_eq!(
            parse_player_shortlist("Unique ID\n77\n"),
            Err(CsvImportError::MissingRequiredHeader("Player UID"))
        );
        assert_eq!(
            parse_player_shortlist("Player UID,Player UID\n77,77\n"),
            Err(CsvImportError::DuplicateHeader("Player UID"))
        );
        assert_eq!(
            parse_player_shortlist("Player UID,Name\n77\n"),
            Err(CsvImportError::MalformedCsv { row: 2 })
        );
        assert_eq!(
            parse_player_shortlist("Player UID\n77\n"),
            Ok(ParsedPlayerShortlist {
                total_rows: 1,
                player_uids: vec![77],
                skipped_invalid: 0,
                skipped_duplicates: 0,
            })
        );
        assert_eq!(
            parse_player_shortlist("Player UID\tName\n77\tAlice\n"),
            Err(CsvImportError::MissingRequiredHeader("Player UID"))
        );
    }

    #[test]
    fn counts_every_data_row_against_the_limit_before_validity_or_deduplication() {
        let mut input = "Player UID\n".to_string();
        for _ in 0..MAX_PLAYER_SHORTLIST_ROWS {
            input.push_str("not-a-number\n");
        }
        let parsed = parse_player_shortlist(&input).expect("accept 10,000 skippable data rows");
        assert_eq!(parsed.total_rows, MAX_PLAYER_SHORTLIST_ROWS);
        assert!(parsed.player_uids.is_empty());

        input.push_str("77\n");
        assert_eq!(
            parse_player_shortlist(&input),
            Err(CsvImportError::TooManyRows {
                limit: MAX_PLAYER_SHORTLIST_ROWS,
            })
        );
    }

    #[test]
    fn zero_match_import_preserves_the_prior_list() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("player-shortlist-zero-match.db"));
        let (save_id, _) = seed_current_player_snapshot(&mut conn, &[77]);
        seed_shortlist(&conn, save_id, &[9]);
        let context = capture_player_shortlist_import_context(&conn).expect("capture context");

        assert_eq!(
            persist_player_shortlist_import(
                &mut conn,
                &context,
                ParsedPlayerShortlist {
                    total_rows: 1,
                    player_uids: vec![88],
                    skipped_invalid: 0,
                    skipped_duplicates: 0,
                },
            ),
            Err(PlayerShortlistImportError::NoMatchingPlayers)
        );
        assert_eq!(shortlist_uids(&conn, save_id), vec![9]);
    }

    #[test]
    fn matching_import_replaces_with_exact_total_stored_and_skipped_counts() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("player-shortlist-replace.db"));
        let (save_id, _) = seed_current_player_snapshot(&mut conn, &[77]);
        seed_shortlist(&conn, save_id, &[9]);
        let context = capture_player_shortlist_import_context(&conn).expect("capture context");
        let parsed = parse_player_shortlist("Player UID\n77\n77\n   \n0\n4294967296\n88\n")
            .expect("parse rows with duplicates, blanks, and unmatched UIDs");

        assert_eq!(parsed.total_rows, 6);
        assert_eq!(parsed.player_uids, vec![77, 88]);
        assert_eq!(parsed.skipped_invalid, 3);
        assert_eq!(parsed.skipped_duplicates, 1);

        let summary = persist_player_shortlist_import(&mut conn, &context, parsed)
            .expect("replace with one current match");

        assert_eq!(
            summary,
            PlayerShortlistImportSummary {
                total_players: 6,
                stored_players: 1,
                skipped_players: 5,
            }
        );
        assert_eq!(shortlist_uids(&conn, save_id), vec![77]);
    }

    #[test]
    fn replacement_touches_only_the_captured_save_and_survives_snapshot_replacement() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("player-shortlist-scope.db"));
        let (save_id, snapshot_id) = seed_current_player_snapshot(&mut conn, &[77]);
        let other_save_id = insert_inactive_save(&conn);
        seed_shortlist(&conn, save_id, &[9]);
        seed_shortlist(&conn, other_save_id, &[10]);
        let context = capture_player_shortlist_import_context(&conn).expect("capture context");

        persist_player_shortlist_import(
            &mut conn,
            &context,
            ParsedPlayerShortlist {
                total_rows: 1,
                player_uids: vec![77],
                skipped_invalid: 0,
                skipped_duplicates: 0,
            },
        )
        .expect("replace active save shortlist");

        assert_eq!(shortlist_uids(&conn, save_id), vec![77]);
        assert_eq!(shortlist_uids(&conn, other_save_id), vec![10]);

        conn.execute("DELETE FROM snapshots WHERE id = ?1", [snapshot_id])
            .expect("replace the current snapshot");
        assert_eq!(shortlist_uids(&conn, save_id), vec![77]);
    }

    #[test]
    fn rejects_stale_context_without_replacing_the_existing_shortlist() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("player-shortlist-stale.db"));
        let (save_id, snapshot_id) = seed_current_player_snapshot(&mut conn, &[77]);
        seed_shortlist(&conn, save_id, &[9]);
        let context = capture_player_shortlist_import_context(&conn).expect("capture context");
        conn.execute(
            "UPDATE snapshots SET is_current = 0 WHERE id = ?1",
            [snapshot_id],
        )
        .expect("make captured context stale");

        assert_eq!(
            persist_player_shortlist_import(
                &mut conn,
                &context,
                ParsedPlayerShortlist {
                    total_rows: 1,
                    player_uids: vec![77],
                    skipped_invalid: 0,
                    skipped_duplicates: 0,
                },
            ),
            Err(PlayerShortlistImportError::Import(
                CsvImportServiceError::StaleContext
            ))
        );
        assert_eq!(shortlist_uids(&conn, save_id), vec![9]);
    }

    #[test]
    fn rolls_back_replacement_if_a_database_write_fails() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("player-shortlist-rollback.db"));
        let (save_id, _) = seed_current_player_snapshot(&mut conn, &[77]);
        seed_shortlist(&conn, save_id, &[9]);
        conn.execute_batch(
            "CREATE TRIGGER reject_player_shortlist_insert
             BEFORE INSERT ON player_shortlist_entries
             BEGIN SELECT RAISE(ABORT, 'test rollback'); END;",
        )
        .expect("create rejection trigger");
        let context = capture_player_shortlist_import_context(&conn).expect("capture context");

        assert_eq!(
            persist_player_shortlist_import(
                &mut conn,
                &context,
                ParsedPlayerShortlist {
                    total_rows: 1,
                    player_uids: vec![77],
                    skipped_invalid: 0,
                    skipped_duplicates: 0,
                },
            ),
            Err(PlayerShortlistImportError::Database)
        );
        assert_eq!(shortlist_uids(&conn, save_id), vec![9]);
    }

    #[test]
    fn moneyball_import_leaves_the_player_shortlist_unchanged() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = crate::db::open(&temp_dir.path().join("player-shortlist-moneyball.db"))
            .expect("open database");
        let moneyball_uid: i64 = 2_002_402_173;
        {
            let mut conn = db.0.lock().expect("lock database");
            let (save_id, snapshot_id) = seed_current_player_snapshot(&mut conn, &[moneyball_uid]);
            seed_shortlist(&conn, save_id, &[moneyball_uid]);
            conn.execute(
                "INSERT INTO players (
                    snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                    nationalities_json, preferred_foot, positions_json, attributes_json,
                    hidden_attributes_json, personality_json
                 ) VALUES (?1, 2002188319, 100, 100, 'Moneyball other', 2000, 1, '[]',
                    'Right', '{}', '{}', '{}', '{}')",
                [snapshot_id],
            )
            .expect("insert second Moneyball player");
        }
        let csv_path = temp_dir.path().join("moneyball.csv");
        std::fs::write(&csv_path, include_str!("fixtures/moneyball_stats.csv"))
            .expect("write Moneyball CSV");

        let summary =
            crate::features::csv_import::commands::import_csv_for_path_with_expected_format(
                &csv_path,
                &db,
                Some(crate::features::csv_import::service::CsvImportFormat::Moneyball),
            )
            .expect("import Moneyball enrichment");
        assert!(summary.stored_players > 0);

        let conn = db.0.lock().expect("lock database");
        let save_id: i64 = conn
            .query_row("SELECT id FROM saves WHERE is_active = 1", [], |row| {
                row.get(0)
            })
            .expect("read active save");
        assert_eq!(shortlist_uids(&conn, save_id), vec![moneyball_uid]);
    }

    fn open_migrated(path: &Path) -> Connection {
        let conn = Connection::open(path).expect("open database");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    fn seed_current_player_snapshot(conn: &mut Connection, uids: &[i64]) -> (i64, i64) {
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Player shortlist save', 1)",
            [],
        )
        .expect("create active save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO snapshots (
                save_id, is_current, schema_version, generated_at_utc, game_version,
                supported_game_version, bridge_version, protocol_version, game_date_source,
                game_date_basis, player_database_scope, scan_truncated, max_accepted,
                player_count, staff_count
             ) VALUES (?1, 1, 6, '2026-01-01T00:00:00Z', '26.0', '26.0', 'test', 1,
                'memory', 'memory', 'men', 0, NULL, ?2, 0)",
            params![save_id, uids.len() as i64],
        )
        .expect("create current snapshot");
        let snapshot_id = conn.last_insert_rowid();
        for uid in uids {
            conn.execute(
                "INSERT INTO players (
                    snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                    nationalities_json, preferred_foot, positions_json, attributes_json,
                    hidden_attributes_json, personality_json
                 ) VALUES (?1, ?2, 100, 100, 'Shortlist player', 2000, 1, '[]',
                    'Right', '{}', '{}', '{}', '{}')",
                params![snapshot_id, uid],
            )
            .expect("insert player");
        }
        (save_id, snapshot_id)
    }

    fn insert_inactive_save(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Other player shortlist save', 0)",
            [],
        )
        .expect("create inactive save");
        conn.last_insert_rowid()
    }

    fn seed_shortlist(conn: &Connection, save_id: i64, uids: &[i64]) {
        for uid in uids {
            conn.execute(
                "INSERT INTO player_shortlist_entries (save_id, player_uid) VALUES (?1, ?2)",
                params![save_id, uid],
            )
            .expect("seed shortlist entry");
        }
    }

    fn shortlist_uids(conn: &Connection, save_id: i64) -> Vec<i64> {
        conn.prepare(
            "SELECT player_uid FROM player_shortlist_entries WHERE save_id = ?1 ORDER BY player_uid",
        )
        .expect("prepare shortlist query")
        .query_map([save_id], |row| row.get(0))
        .expect("query shortlist")
        .collect::<Result<_, _>>()
        .expect("read shortlist")
    }
}
