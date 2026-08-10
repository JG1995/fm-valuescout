use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;

use super::parser::{parse_csv_with_row_limit, ParsedCsv};
use super::CsvImportError;
pub(crate) const MAX_CSV_BYTES: u64 = 1024 * 1024;
pub(crate) const MAX_CSV_ROWS: usize = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum CsvPreviewFormat {
    YouthTracker,
    Moneyball,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CsvMatchPreview {
    pub format: CsvPreviewFormat,
    pub total_players: usize,
    pub matched_players: usize,
    pub unmatched_players: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreviewContext {
    save_id: i64,
    snapshot_id: i64,
    player_uids: HashSet<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CsvPreviewError {
    NoCurrentSnapshot,
    StaleContext,
    InvalidFile,
    FileTooLarge,
    InvalidUtf8,
    UnsupportedFormat,
    TooManyRows,
    InvalidCsv(CsvImportError),
    Database,
}

impl std::fmt::Display for CsvPreviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoCurrentSnapshot => write!(f, "Load data before previewing a CSV export"),
            Self::StaleContext => write!(f, "The current save changed while the CSV was read"),
            Self::InvalidFile => write!(f, "Select a regular .csv file"),
            Self::FileTooLarge => write!(f, "CSV file exceeds the 1 MiB limit"),
            Self::InvalidUtf8 => write!(f, "CSV file must use UTF-8 encoding"),
            Self::UnsupportedFormat => write!(f, "CSV format is not supported"),
            Self::TooManyRows => write!(f, "CSV contains more than 1000 player rows"),
            Self::InvalidCsv(error) => write!(f, "CSV file is invalid: {error}"),
            Self::Database => write!(f, "CSV preview is unavailable"),
        }
    }
}

impl std::error::Error for CsvPreviewError {}

pub(crate) fn capture_preview_context(
    conn: &Connection,
) -> Result<PreviewContext, CsvPreviewError> {
    let context = conn
        .query_row(
            "SELECT sv.id, s.id
             FROM saves sv
             INNER JOIN snapshots s ON s.save_id = sv.id
             WHERE sv.is_active = 1 AND s.is_current = 1
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|_| CsvPreviewError::Database)?
        .ok_or(CsvPreviewError::NoCurrentSnapshot)?;

    let mut statement = conn
        .prepare("SELECT uid FROM players WHERE snapshot_id = ?1")
        .map_err(|_| CsvPreviewError::Database)?;
    let player_uids = statement
        .query_map([context.1], |row| row.get(0))
        .map_err(|_| CsvPreviewError::Database)?
        .collect::<Result<HashSet<_>, _>>()
        .map_err(|_| CsvPreviewError::Database)?;

    Ok(PreviewContext {
        save_id: context.0,
        snapshot_id: context.1,
        player_uids,
    })
}

pub(crate) fn preview_csv_file(
    path: &Path,
    context: &PreviewContext,
) -> Result<CsvMatchPreview, CsvPreviewError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| CsvPreviewError::InvalidFile)?;
    if !metadata.file_type().is_file() || !has_csv_extension(path) {
        return Err(CsvPreviewError::InvalidFile);
    }
    if metadata.len() > MAX_CSV_BYTES {
        return Err(CsvPreviewError::FileTooLarge);
    }

    let mut file = File::open(path).map_err(|_| CsvPreviewError::InvalidFile)?;
    let metadata = file.metadata().map_err(|_| CsvPreviewError::InvalidFile)?;
    if !metadata.file_type().is_file() {
        return Err(CsvPreviewError::InvalidFile);
    }
    let input = read_bounded_file(&mut file)?;
    let (format, uids) =
        match parse_csv_with_row_limit(&input, Some(MAX_CSV_ROWS)).map_err(map_csv_error)? {
            ParsedCsv::YouthTracker(players) => (
                CsvPreviewFormat::YouthTracker,
                players
                    .into_iter()
                    .map(|player| i64::from(player.uid))
                    .collect::<Vec<_>>(),
            ),
            ParsedCsv::Moneyball(players) => (
                CsvPreviewFormat::Moneyball,
                players
                    .into_iter()
                    .map(|player| i64::from(player.uid))
                    .collect::<Vec<_>>(),
            ),
        };
    let matched_players = uids
        .iter()
        .filter(|uid| context.player_uids.contains(uid))
        .count();
    let total_players = uids.len();

    Ok(CsvMatchPreview {
        format,
        total_players,
        matched_players,
        unmatched_players: total_players - matched_players,
    })
}

pub(crate) fn revalidate_preview_context(
    conn: &Connection,
    context: &PreviewContext,
) -> Result<(), CsvPreviewError> {
    let is_current: bool = conn
        .query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM snapshots
                WHERE id = ?1 AND save_id = ?2 AND is_current = 1
                  AND EXISTS (SELECT 1 FROM saves WHERE id = ?2 AND is_active = 1)
            )",
            [context.snapshot_id, context.save_id],
            |row| row.get(0),
        )
        .map_err(|_| CsvPreviewError::Database)?;

    is_current
        .then_some(())
        .ok_or(CsvPreviewError::StaleContext)
}

fn has_csv_extension(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("csv"))
}

fn read_bounded_file(file: &mut File) -> Result<String, CsvPreviewError> {
    let metadata = file.metadata().map_err(|_| CsvPreviewError::InvalidFile)?;
    if metadata.len() > MAX_CSV_BYTES {
        return Err(CsvPreviewError::FileTooLarge);
    }

    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.by_ref()
        .take(MAX_CSV_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| CsvPreviewError::InvalidFile)?;
    if bytes.len() as u64 > MAX_CSV_BYTES {
        return Err(CsvPreviewError::FileTooLarge);
    }

    String::from_utf8(bytes).map_err(|_| CsvPreviewError::InvalidUtf8)
}

fn map_csv_error(error: CsvImportError) -> CsvPreviewError {
    match error {
        CsvImportError::UnsupportedDialect | CsvImportError::MissingRequiredHeader(_) => {
            CsvPreviewError::UnsupportedFormat
        }
        CsvImportError::TooManyRows { .. } => CsvPreviewError::TooManyRows,
        error => CsvPreviewError::InvalidCsv(error),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use rusqlite::{params, Connection};

    use super::*;
    use crate::db::migrations;
    use crate::features::csv_import::parser::{parse_csv, ParsedCsv};

    const YOUTH_EXPORT: &str = include_str!("fixtures/2030_07_01_Full_Squad_CA_PA_Monza.csv");
    const MONEYBALL_EXPORT: &str = include_str!("fixtures/moneyball_stats.csv");

    #[test]
    fn previews_moneyball_uids_against_the_pinned_youth_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("preview.db"));
        let youth_uids = parsed_uids(YOUTH_EXPORT);
        seed_current_snapshot(&mut conn, &youth_uids);
        let path = write_csv(&temp_dir, "moneyball.csv", MONEYBALL_EXPORT);
        let before = database_fingerprint(&conn);

        let context = capture_preview_context(&conn).expect("capture current snapshot");
        let preview = preview_csv_file(&path, &context).expect("preview Moneyball export");

        assert_eq!(
            preview,
            CsvMatchPreview {
                format: CsvPreviewFormat::Moneyball,
                total_players: 75,
                matched_players: 74,
                unmatched_players: 1,
            }
        );
        revalidate_preview_context(&conn, &context).expect("context remains current");
        assert_eq!(database_fingerprint(&conn), before);
    }

    #[test]
    fn previews_youth_uids_with_exact_matches_and_unmatched_rows() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("preview-youth.db"));
        let youth_uids = parsed_uids(YOUTH_EXPORT);
        seed_current_snapshot(&mut conn, &youth_uids[..73]);
        let path = write_csv(&temp_dir, "youth.csv", YOUTH_EXPORT);

        let context = capture_preview_context(&conn).expect("capture current snapshot");
        let preview = preview_csv_file(&path, &context).expect("preview Youth export");

        assert_eq!(
            preview,
            CsvMatchPreview {
                format: CsvPreviewFormat::YouthTracker,
                total_players: 74,
                matched_players: 73,
                unmatched_players: 1,
            }
        );
    }

    #[test]
    fn rejects_invalid_files_and_bounds_without_leaking_the_local_path() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("bounds.db"));
        seed_current_snapshot(&mut conn, &[1]);
        let context = capture_preview_context(&conn).expect("capture current snapshot");

        let text_path = write_csv(&temp_dir, "players.txt", "Unique ID;Player\n1;Player\n");
        let text_error = preview_csv_file(&text_path, &context).expect_err("reject text file");
        assert_eq!(text_error, CsvPreviewError::InvalidFile);
        assert!(!text_error
            .to_string()
            .contains(temp_dir.path().to_str().expect("temp path")));

        let utf8_path = temp_dir.path().join("invalid.csv");
        fs::write(&utf8_path, [0xff, 0xfe]).expect("write invalid UTF-8");
        assert_eq!(
            preview_csv_file(&utf8_path, &context).expect_err("reject invalid UTF-8"),
            CsvPreviewError::InvalidUtf8
        );

        let oversized_path = temp_dir.path().join("oversized.csv");
        fs::write(&oversized_path, vec![b'x'; MAX_CSV_BYTES as usize + 1])
            .expect("write oversized CSV");
        assert_eq!(
            preview_csv_file(&oversized_path, &context).expect_err("reject oversized CSV"),
            CsvPreviewError::FileTooLarge
        );

        let too_many_rows = std::iter::once("Unique ID;Player".to_string())
            .chain((1..=MAX_CSV_ROWS + 1).map(|uid| format!("{uid};Player {uid}")))
            .collect::<Vec<_>>()
            .join("\n");
        let rows_path = write_csv(&temp_dir, "too-many.csv", &too_many_rows);
        assert_eq!(
            preview_csv_file(&rows_path, &context).expect_err("reject too many rows"),
            CsvPreviewError::TooManyRows
        );
    }

    #[test]
    fn opened_reader_rejects_growth_past_the_byte_limit() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let path = write_csv(&temp_dir, "growing.csv", "Unique ID;Player\n1;Player\n");
        let mut file = fs::File::open(&path).expect("open CSV");
        fs::write(&path, vec![b'x'; MAX_CSV_BYTES as usize + 1]).expect("grow CSV");

        assert_eq!(
            read_bounded_file(&mut file).expect_err("reject grown CSV"),
            CsvPreviewError::FileTooLarge
        );
    }

    #[test]
    fn propagates_invalid_csv_data_without_returning_raw_rows() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("invalid-data.db"));
        seed_current_snapshot(&mut conn, &[1]);
        let context = capture_preview_context(&conn).expect("capture current snapshot");
        let path = write_csv(
            &temp_dir,
            "duplicate.csv",
            "Unique ID;Player\n1;First player\n1;Raw row that must not cross IPC\n",
        );

        let error = preview_csv_file(&path, &context).expect_err("duplicate UID propagates");

        assert_eq!(
            error,
            CsvPreviewError::InvalidCsv(CsvImportError::DuplicateUid {
                first_row: 2,
                row: 3,
            })
        );
        assert!(!error
            .to_string()
            .contains("Raw row that must not cross IPC"));
    }

    #[test]
    fn rejects_a_preview_when_the_current_save_changes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("stale.db"));
        seed_current_snapshot(&mut conn, &[1]);
        let context = capture_preview_context(&conn).expect("capture current snapshot");

        conn.execute("UPDATE saves SET is_active = 0 WHERE is_active = 1", [])
            .expect("deactivate prior save");
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES (?1, 1)",
            ["New active save"],
        )
        .expect("switch active save");

        assert_eq!(
            revalidate_preview_context(&conn, &context).expect_err("reject stale context"),
            CsvPreviewError::StaleContext
        );
    }

    #[test]
    fn rejects_a_preview_when_the_current_snapshot_changes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("stale-snapshot.db"));
        seed_current_snapshot(&mut conn, &[1]);
        let context = capture_preview_context(&conn).expect("capture current snapshot");

        conn.execute(
            "UPDATE snapshots SET is_current = 0 WHERE is_current = 1",
            [],
        )
        .expect("replace current snapshot");

        assert_eq!(
            revalidate_preview_context(&conn, &context).expect_err("reject stale context"),
            CsvPreviewError::StaleContext
        );
    }

    #[test]
    fn requires_a_current_snapshot_without_creating_database_rows() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("no-snapshot.db"));
        let before = database_fingerprint(&conn);

        assert_eq!(
            capture_preview_context(&conn).expect_err("no current snapshot"),
            CsvPreviewError::NoCurrentSnapshot
        );

        assert_eq!(database_fingerprint(&conn), before);
    }

    fn open_migrated(path: &Path) -> Connection {
        let conn = Connection::open(path).expect("open database");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    fn parsed_uids(input: &str) -> Vec<u32> {
        match parse_csv(input).expect("parse pinned Youth export") {
            ParsedCsv::YouthTracker(players) => {
                players.into_iter().map(|player| player.uid).collect()
            }
            ParsedCsv::Moneyball(_) => panic!("expected Youth export"),
        }
    }

    fn seed_current_snapshot(conn: &mut Connection, uids: &[u32]) {
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES (?1, 1)",
            ["Preview save"],
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
        let tx = conn.transaction().expect("begin player transaction");
        let mut statement = tx
            .prepare(
                "INSERT INTO players (
                    snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                    nationalities_json, preferred_foot, positions_json, attributes_json,
                    hidden_attributes_json, personality_json
                ) VALUES (?1, ?2, 100, 100, 'Preview player', 2000, 1, '[]', 'Right',
                    '{}', '{}', '{}', '{}')",
            )
            .expect("prepare player insert");
        for uid in uids {
            statement
                .execute(params![snapshot_id, uid])
                .expect("insert player");
        }
        drop(statement);
        tx.commit().expect("commit players");
    }

    fn write_csv(temp_dir: &tempfile::TempDir, name: &str, contents: &str) -> std::path::PathBuf {
        let path = temp_dir.path().join(name);
        std::fs::write(&path, contents).expect("write CSV fixture");
        path
    }

    fn database_fingerprint(conn: &Connection) -> (i32, i64, i64, i64, u64) {
        let version = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read migration version");
        let saves = conn
            .query_row("SELECT COUNT(*) FROM saves", [], |row| row.get(0))
            .expect("count saves");
        let snapshots = conn
            .query_row("SELECT COUNT(*) FROM snapshots", [], |row| row.get(0))
            .expect("count snapshots");
        let players = conn
            .query_row("SELECT COUNT(*) FROM players", [], |row| row.get(0))
            .expect("count players");
        (version, saves, snapshots, players, conn.total_changes())
    }
}
