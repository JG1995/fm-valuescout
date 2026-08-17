#![cfg_attr(not(test), allow(dead_code))]

use std::collections::{HashMap, HashSet};
use std::path::Path;

use csv::{ReaderBuilder, StringRecord, Trim};
use rusqlite::{params, Connection};
use serde::Serialize;

use super::service::{
    capture_active_import_context, read_csv_file, revalidate_active_import_context,
    ActiveImportContext, CsvImportServiceError,
};
use super::CsvImportError;

pub(crate) const MAX_STAFF_SHORTLIST_ROWS: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StaffShortlistEntry {
    pub staff_uid: i64,
    pub preferred_job: String,
    pub club_job: String,
    pub coaching_qualifications: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StaffShortlistImportSummary {
    pub total_staff: usize,
    pub stored_staff: usize,
    pub skipped_staff: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StaffShortlistImportContext {
    active: ActiveImportContext,
    staff_uids: HashSet<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StaffShortlistImportError {
    Import(CsvImportServiceError),
    InvalidCsv(CsvImportError),
    NoMatchingStaff,
    Database,
}

impl std::fmt::Display for StaffShortlistImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Import(error) => error.fmt(f),
            Self::InvalidCsv(error) => write!(f, "CSV file is invalid: {error}"),
            Self::NoMatchingStaff => {
                write!(f, "CSV does not contain staff in the current snapshot")
            }
            Self::Database => write!(f, "CSV import is unavailable"),
        }
    }
}

impl std::error::Error for StaffShortlistImportError {}

impl From<CsvImportServiceError> for StaffShortlistImportError {
    fn from(error: CsvImportServiceError) -> Self {
        Self::Import(error)
    }
}

pub(crate) fn capture_staff_shortlist_import_context(
    conn: &Connection,
) -> Result<StaffShortlistImportContext, StaffShortlistImportError> {
    let active = capture_active_import_context(conn)?;
    let mut statement = conn
        .prepare("SELECT uid FROM staff WHERE snapshot_id = ?1")
        .map_err(|_| StaffShortlistImportError::Database)?;
    let staff_uids = statement
        .query_map([active.snapshot_id], |row| row.get(0))
        .map_err(|_| StaffShortlistImportError::Database)?
        .collect::<Result<HashSet<_>, _>>()
        .map_err(|_| StaffShortlistImportError::Database)?;

    Ok(StaffShortlistImportContext { active, staff_uids })
}

pub(crate) fn prepare_staff_shortlist_import(
    path: &Path,
) -> Result<Vec<StaffShortlistEntry>, StaffShortlistImportError> {
    let input = read_csv_file(path)?;
    parse_staff_shortlist(&input).map_err(StaffShortlistImportError::InvalidCsv)
}

pub(crate) fn persist_staff_shortlist_import(
    conn: &mut Connection,
    context: &StaffShortlistImportContext,
    entries: Vec<StaffShortlistEntry>,
) -> Result<StaffShortlistImportSummary, StaffShortlistImportError> {
    let tx = conn
        .transaction()
        .map_err(|_| StaffShortlistImportError::Database)?;
    revalidate_active_import_context(&tx, &context.active)?;

    let total_staff = entries.len();
    let matching_entries = entries
        .into_iter()
        .filter(|entry| context.staff_uids.contains(&entry.staff_uid))
        .collect::<Vec<_>>();
    if matching_entries.is_empty() {
        return Err(StaffShortlistImportError::NoMatchingStaff);
    }

    tx.execute(
        "DELETE FROM staff_shortlist_entries WHERE save_id = ?1",
        [context.active.save_id],
    )
    .map_err(|_| StaffShortlistImportError::Database)?;
    for entry in &matching_entries {
        tx.execute(
            "INSERT INTO staff_shortlist_entries (
                save_id, staff_uid, preferred_job, club_job, coaching_qualifications
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                context.active.save_id,
                entry.staff_uid,
                entry.preferred_job,
                entry.club_job,
                entry.coaching_qualifications,
            ],
        )
        .map_err(|_| StaffShortlistImportError::Database)?;
    }

    tx.commit()
        .map_err(|_| StaffShortlistImportError::Database)?;
    Ok(StaffShortlistImportSummary {
        total_staff,
        stored_staff: matching_entries.len(),
        skipped_staff: total_staff - matching_entries.len(),
    })
}

pub(crate) fn parse_staff_shortlist(
    input: &str,
) -> Result<Vec<StaffShortlistEntry>, CsvImportError> {
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    if input.trim().is_empty() {
        return Err(CsvImportError::EmptyInput);
    }

    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .trim(Trim::All)
        .flexible(false)
        .from_reader(input.as_bytes());
    let headers = reader
        .headers()
        .map_err(|_| CsvImportError::MalformedCsv { row: 1 })?
        .clone();
    if headers.len() == 1 && headers.get(0).is_some_and(|header| header.contains(',')) {
        return Err(CsvImportError::UnsupportedDialect);
    }
    let columns = StaffShortlistColumns::from_headers(&headers)?;
    let mut seen_uids = HashMap::new();
    let mut entries = Vec::new();

    for (record_index, record) in reader.records().enumerate() {
        if entries.len() >= MAX_STAFF_SHORTLIST_ROWS {
            return Err(CsvImportError::TooManyRows {
                limit: MAX_STAFF_SHORTLIST_ROWS,
            });
        }
        let row = record_index + 2;
        let record = record.map_err(|_| CsvImportError::MalformedCsv { row })?;
        let staff_uid = parse_staff_uid(value(&record, columns.uid), row)?;
        if let Some(first_row) = seen_uids.insert(staff_uid, row) {
            return Err(CsvImportError::DuplicateUid { first_row, row });
        }

        let preferred_job = value(&record, columns.preferred_job).trim();
        if preferred_job.is_empty() {
            return Err(CsvImportError::InvalidValue {
                row,
                field: "Preferred Job",
                expected: "non-empty text",
            });
        }

        entries.push(StaffShortlistEntry {
            staff_uid,
            preferred_job: preferred_job.to_string(),
            club_job: value(&record, columns.club_job).trim().to_string(),
            coaching_qualifications: value(&record, columns.coaching_qualifications)
                .trim()
                .to_string(),
        });
    }

    Ok(entries)
}

struct StaffShortlistColumns {
    uid: usize,
    preferred_job: usize,
    club_job: usize,
    coaching_qualifications: usize,
}

impl StaffShortlistColumns {
    fn from_headers(headers: &StringRecord) -> Result<Self, CsvImportError> {
        Ok(Self {
            uid: required_column(headers, "Unique ID")?,
            preferred_job: required_column(headers, "Preferred Job")?,
            club_job: required_column(headers, "Club Job")?,
            coaching_qualifications: required_column(headers, "Coaching Qualifications")?,
        })
    }
}

fn required_column(headers: &StringRecord, name: &'static str) -> Result<usize, CsvImportError> {
    let matches = headers
        .iter()
        .enumerate()
        .filter_map(|(index, header)| (header.trim() == name).then_some(index))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => Err(CsvImportError::MissingRequiredHeader(name)),
        [index] => Ok(*index),
        _ => Err(CsvImportError::DuplicateHeader(name)),
    }
}

fn value(record: &StringRecord, column: usize) -> &str {
    record.get(column).unwrap_or_default()
}

fn parse_staff_uid(value: &str, row: usize) -> Result<i64, CsvImportError> {
    value
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|uid| *uid > 0)
        .map(i64::from)
        .ok_or(CsvImportError::InvalidValue {
            row,
            field: "Unique ID",
            expected: "a positive u32",
        })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use rusqlite::{params, Connection};

    use crate::db::migrations;

    use super::*;

    #[test]
    fn parses_required_staff_shortlist_columns_and_preserves_trimmed_csv_strings() {
        assert_eq!(MAX_STAFF_SHORTLIST_ROWS, 10_000);
        let entries = parse_staff_shortlist(include_str!("fixtures/staff_shortlist.csv"))
            .expect("parse valid staff shortlist export");

        assert_eq!(
            entries,
            vec![
                StaffShortlistEntry {
                    staff_uid: 77,
                    preferred_job: "Physio".to_string(),
                    club_job: "-".to_string(),
                    coaching_qualifications: "Continental Pro".to_string(),
                },
                StaffShortlistEntry {
                    staff_uid: 88,
                    preferred_job: "Scout".to_string(),
                    club_job: "".to_string(),
                    coaching_qualifications: "National C".to_string(),
                },
                StaffShortlistEntry {
                    staff_uid: 99,
                    preferred_job: "Coach".to_string(),
                    club_job: "Coach".to_string(),
                    coaching_qualifications: "National B".to_string(),
                },
            ]
        );
    }

    #[test]
    fn rejects_invalid_staff_shortlist_shapes_before_persisting() {
        assert_eq!(
            parse_staff_shortlist(
                "Unique ID;Preferred Job;Club Job;Coaching Qualifications\n77;Physio;-;A\n77;Scout;-;B\n",
            ),
            Err(CsvImportError::DuplicateUid {
                first_row: 2,
                row: 3,
            })
        );
        assert_eq!(
            parse_staff_shortlist("Unique ID;Preferred Job;Club Job\n77;Physio;-\n"),
            Err(CsvImportError::MissingRequiredHeader(
                "Coaching Qualifications"
            ))
        );
        assert_eq!(
            parse_staff_shortlist(
                "Unique ID;Preferred Job;Club Job;Coaching Qualifications\n77; ; - ; A\n",
            ),
            Err(CsvImportError::InvalidValue {
                row: 2,
                field: "Preferred Job",
                expected: "non-empty text",
            })
        );
        assert_eq!(
            parse_staff_shortlist(
                "Unique ID,Preferred Job,Club Job,Coaching Qualifications\n77,Physio,-,A\n",
            ),
            Err(CsvImportError::UnsupportedDialect)
        );
        assert_eq!(
            parse_staff_shortlist(
                "Unique ID;Preferred Job;Club Job;Coaching Qualifications\n0;Physio;-;A\n",
            ),
            Err(CsvImportError::InvalidValue {
                row: 2,
                field: "Unique ID",
                expected: "a positive u32",
            })
        );
        assert_eq!(
            parse_staff_shortlist(
                "Unique ID;Preferred Job;Club Job;Coaching Qualifications\n77;Physio;-\n",
            ),
            Err(CsvImportError::MalformedCsv { row: 2 })
        );
    }

    #[test]
    fn accepts_the_exact_row_limit_and_rejects_one_more() {
        let mut input = "Unique ID;Preferred Job;Club Job;Coaching Qualifications\n".to_string();
        for uid in 1..=MAX_STAFF_SHORTLIST_ROWS {
            input.push_str(&format!("{uid};Physio;-;A\n"));
        }

        assert_eq!(
            parse_staff_shortlist(&input)
                .expect("accept the exact staff shortlist row limit")
                .len(),
            MAX_STAFF_SHORTLIST_ROWS
        );
        input.push_str("10001;Physio;-;A\n");

        assert_eq!(
            parse_staff_shortlist(&input),
            Err(CsvImportError::TooManyRows {
                limit: MAX_STAFF_SHORTLIST_ROWS,
            })
        );
    }

    #[test]
    fn accepts_bom_crlf_and_required_headers_in_any_order() {
        assert_eq!(
            parse_staff_shortlist(
                "\u{feff}Coaching Qualifications;Club Job;Unique ID;Preferred Job\r\nA; - ;77;Physio\r\n",
            ),
            Ok(vec![entry(77, "Physio", "-", "A")])
        );
    }

    #[test]
    fn replaces_only_the_captured_save_shortlist_and_skips_unknown_staff() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("staff-shortlist.db"));
        let (save_id, _) = seed_current_staff_snapshot(&mut conn, &[77]);
        let other_save_id = insert_inactive_save(&conn);
        conn.execute(
            "INSERT INTO staff_shortlist_entries (
                save_id, staff_uid, preferred_job, club_job, coaching_qualifications
             ) VALUES (?1, 9, 'Old', '-', 'Old qualification')",
            [save_id],
        )
        .expect("seed active shortlist");
        conn.execute(
            "INSERT INTO staff_shortlist_entries (
                save_id, staff_uid, preferred_job, club_job, coaching_qualifications
             ) VALUES (?1, 10, 'Other', '-', 'Other qualification')",
            [other_save_id],
        )
        .expect("seed another save shortlist");

        let context = capture_staff_shortlist_import_context(&conn).expect("capture context");
        let summary = persist_staff_shortlist_import(
            &mut conn,
            &context,
            vec![
                entry(77, "Physio", "-", "Continental Pro"),
                entry(88, "Scout", "", "National C"),
            ],
        )
        .expect("replace current shortlist");

        assert_eq!(
            summary,
            StaffShortlistImportSummary {
                total_staff: 2,
                stored_staff: 1,
                skipped_staff: 1,
            }
        );
        assert_eq!(
            shortlist_values(&conn, save_id),
            vec![(
                77,
                "Physio".to_string(),
                "-".to_string(),
                "Continental Pro".to_string(),
            )]
        );
        assert_eq!(
            shortlist_values(&conn, other_save_id),
            vec![(
                10,
                "Other".to_string(),
                "-".to_string(),
                "Other qualification".to_string(),
            )]
        );
    }

    #[test]
    fn rejects_zero_matches_or_stale_context_without_replacing_existing_shortlist() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("staff-shortlist-context.db"));
        let (save_id, snapshot_id) = seed_current_staff_snapshot(&mut conn, &[77]);
        conn.execute(
            "INSERT INTO staff_shortlist_entries (
                save_id, staff_uid, preferred_job, club_job, coaching_qualifications
             ) VALUES (?1, 9, 'Old', '-', 'Old qualification')",
            [save_id],
        )
        .expect("seed shortlist");
        let context = capture_staff_shortlist_import_context(&conn).expect("capture context");

        assert_eq!(
            persist_staff_shortlist_import(&mut conn, &context, vec![entry(88, "Scout", "", "")]),
            Err(StaffShortlistImportError::NoMatchingStaff)
        );
        assert_eq!(shortlist_values(&conn, save_id)[0].0, 9);

        conn.execute(
            "UPDATE snapshots SET is_current = 0 WHERE id = ?1",
            [snapshot_id],
        )
        .expect("make captured context stale");
        assert_eq!(
            persist_staff_shortlist_import(
                &mut conn,
                &context,
                vec![entry(77, "Physio", "-", "A")]
            ),
            Err(StaffShortlistImportError::Import(
                CsvImportServiceError::StaleContext
            ))
        );
        assert_eq!(shortlist_values(&conn, save_id)[0].0, 9);
    }

    #[test]
    fn rolls_back_replacement_if_a_database_write_fails() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("staff-shortlist-rollback.db"));
        let (save_id, _) = seed_current_staff_snapshot(&mut conn, &[77]);
        conn.execute(
            "INSERT INTO staff_shortlist_entries (
                save_id, staff_uid, preferred_job, club_job, coaching_qualifications
             ) VALUES (?1, 9, 'Old', '-', 'Old qualification')",
            [save_id],
        )
        .expect("seed shortlist");
        conn.execute_batch(
            "CREATE TRIGGER reject_staff_shortlist_insert
             BEFORE INSERT ON staff_shortlist_entries
             BEGIN SELECT RAISE(ABORT, 'test rollback'); END;",
        )
        .expect("create rejection trigger");
        let context = capture_staff_shortlist_import_context(&conn).expect("capture context");

        assert_eq!(
            persist_staff_shortlist_import(
                &mut conn,
                &context,
                vec![entry(77, "Physio", "-", "A")]
            ),
            Err(StaffShortlistImportError::Database)
        );
        assert_eq!(shortlist_values(&conn, save_id)[0].0, 9);
    }

    fn entry(
        uid: i64,
        preferred_job: &str,
        club_job: &str,
        qualifications: &str,
    ) -> StaffShortlistEntry {
        StaffShortlistEntry {
            staff_uid: uid,
            preferred_job: preferred_job.to_string(),
            club_job: club_job.to_string(),
            coaching_qualifications: qualifications.to_string(),
        }
    }

    fn open_migrated(path: &Path) -> Connection {
        let conn = Connection::open(path).expect("open database");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    fn seed_current_staff_snapshot(conn: &mut Connection, uids: &[i64]) -> (i64, i64) {
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Staff shortlist save', 1)",
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
                'memory', 'memory', 'men', 0, NULL, 0, ?2)",
            params![save_id, uids.len() as i64],
        )
        .expect("create current snapshot");
        let snapshot_id = conn.last_insert_rowid();
        for uid in uids {
            conn.execute(
                "INSERT INTO staff (
                    snapshot_id, uid, nationalities_json, gender, ca, pa, staff_attributes_json
                 ) VALUES (?1, ?2, '[]', 'unknown', 100, 100, '{}')",
                params![snapshot_id, uid],
            )
            .expect("insert staff");
        }
        (save_id, snapshot_id)
    }

    fn insert_inactive_save(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Other staff shortlist save', 0)",
            [],
        )
        .expect("create inactive save");
        conn.last_insert_rowid()
    }

    fn shortlist_values(conn: &Connection, save_id: i64) -> Vec<(i64, String, String, String)> {
        conn.prepare(
            "SELECT staff_uid, preferred_job, club_job, coaching_qualifications
             FROM staff_shortlist_entries
             WHERE save_id = ?1
             ORDER BY staff_uid",
        )
        .expect("prepare shortlist query")
        .query_map([save_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .expect("query shortlist")
        .collect::<Result<_, _>>()
        .expect("read shortlist")
    }
}
