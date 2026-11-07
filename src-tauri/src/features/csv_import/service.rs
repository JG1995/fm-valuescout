use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};

use super::parser::parse_csv_with_row_limit;
use super::{CsvImportError as ParsedCsvError, YouthTrackerPlayer};
pub(crate) const MAX_CSV_BYTES: u64 = 8 * 1024 * 1024;
pub(crate) const MAX_CSV_ROWS: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum CsvImportFormat {
    YouthTracker,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CsvImportSummary {
    pub format: CsvImportFormat,
    pub total_players: usize,
    pub stored_players: usize,
    pub skipped_players: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImportContext {
    active: ActiveImportContext,
    player_uids: HashSet<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActiveImportContext {
    pub(crate) save_id: i64,
    save_context_token: String,
    pub(crate) snapshot_id: i64,
    snapshot_context_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CsvImportServiceError {
    NoCurrentSnapshot,
    StaleContext,
    InvalidFile,
    FileTooLarge,
    InvalidUtf8,
    UnsupportedFormat,
    MoneyballExport,
    TooManyRows,
    InvalidCsv(ParsedCsvError),
    Database,
}

impl std::fmt::Display for CsvImportServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoCurrentSnapshot => write!(f, "Load data before importing a CSV export"),
            Self::StaleContext => write!(
                f,
                "The current save or snapshot changed while the CSV was imported"
            ),
            Self::InvalidFile => write!(f, "Select a regular .csv file"),
            Self::FileTooLarge => write!(f, "CSV file exceeds the 8 MiB limit"),
            Self::InvalidUtf8 => write!(f, "CSV file must use UTF-8 encoding"),
            Self::UnsupportedFormat => write!(f, "CSV format is not supported"),
            Self::MoneyballExport => write!(
                f,
                "Moneyball stats now come from the linked FM save file, not a CSV export"
            ),
            Self::TooManyRows => write!(f, "CSV contains more than 10,000 rows"),
            Self::InvalidCsv(error) => write!(f, "CSV file is invalid: {error}"),
            Self::Database => write!(f, "CSV import is unavailable"),
        }
    }
}

impl std::error::Error for CsvImportServiceError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CsvPersistenceError {
    Import(CsvImportServiceError),
    Database,
}

impl std::fmt::Display for CsvPersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Import(error) => error.fmt(f),
            Self::Database => write!(f, "CSV import is unavailable"),
        }
    }
}

impl std::error::Error for CsvPersistenceError {}

impl From<CsvImportServiceError> for CsvPersistenceError {
    fn from(error: CsvImportServiceError) -> Self {
        Self::Import(error)
    }
}

pub(crate) struct PreparedYouthCareerStats {
    player_uid: i64,
    career_appearances: Option<i64>,
    international_caps: Option<i64>,
    career_goals: Option<i64>,
    career_assists: Option<i64>,
}

impl ImportContext {
    pub(crate) fn active(&self) -> &ActiveImportContext {
        &self.active
    }

    pub(crate) fn player_uids(&self) -> &HashSet<i64> {
        &self.player_uids
    }
}

impl ActiveImportContext {
    pub(crate) fn save_context_token(&self) -> &str {
        &self.save_context_token
    }
}

pub(crate) fn capture_import_context(
    conn: &Connection,
) -> Result<ImportContext, CsvImportServiceError> {
    let active = capture_active_import_context(conn)?;

    let mut statement = conn
        .prepare("SELECT uid FROM players WHERE snapshot_id = ?1")
        .map_err(|_| CsvImportServiceError::Database)?;
    let player_uids = statement
        .query_map([active.snapshot_id], |row| row.get(0))
        .map_err(|_| CsvImportServiceError::Database)?
        .collect::<Result<HashSet<_>, _>>()
        .map_err(|_| CsvImportServiceError::Database)?;

    Ok(ImportContext {
        active,
        player_uids,
    })
}

pub(crate) fn capture_active_import_context(
    conn: &Connection,
) -> Result<ActiveImportContext, CsvImportServiceError> {
    let context = conn
        .query_row(
            "SELECT sv.id, sv.context_token, s.id, s.context_token
             FROM saves sv
             INNER JOIN snapshots s ON s.save_id = sv.id
             WHERE sv.is_active = 1 AND s.is_current = 1
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(|_| CsvImportServiceError::Database)?
        .ok_or(CsvImportServiceError::NoCurrentSnapshot)?;

    Ok(ActiveImportContext {
        save_id: context.0,
        save_context_token: context.1,
        snapshot_id: context.2,
        snapshot_context_token: context.3,
    })
}

pub(crate) fn prepare_csv_import(
    path: &Path,
) -> Result<Vec<PreparedYouthCareerStats>, CsvPersistenceError> {
    Ok(parse_csv_file(path)?
        .into_iter()
        .map(|player| PreparedYouthCareerStats {
            player_uid: i64::from(player.uid),
            career_appearances: player.all_time_appearances.map(i64::from),
            international_caps: player.international_appearances.map(i64::from),
            career_goals: player.all_time_goals.map(i64::from),
            career_assists: player.assists.map(i64::from),
        })
        .collect())
}

pub(crate) fn persist_csv_import(
    conn: &mut Connection,
    context: &ImportContext,
    players: Vec<PreparedYouthCareerStats>,
) -> Result<CsvImportSummary, CsvPersistenceError> {
    let tx = conn
        .transaction()
        .map_err(|_| CsvPersistenceError::Database)?;
    revalidate_import_context(&tx, context)?;

    let total_players = players.len();
    let mut stored_players = 0;
    for player in players {
        if !context.player_uids.contains(&player.player_uid) {
            continue;
        }
        upsert_youth_career_stats(&tx, context.active.save_id, &player)?;
        stored_players += 1;
    }

    tx.commit().map_err(|_| CsvPersistenceError::Database)?;
    Ok(CsvImportSummary {
        format: CsvImportFormat::YouthTracker,
        total_players,
        stored_players,
        skipped_players: total_players - stored_players,
    })
}

fn parse_csv_file(path: &Path) -> Result<Vec<YouthTrackerPlayer>, CsvImportServiceError> {
    let input = read_csv_file(path)?;
    parse_csv_with_row_limit(&input, Some(MAX_CSV_ROWS)).map_err(map_csv_error)
}

pub(crate) fn read_csv_file(path: &Path) -> Result<String, CsvImportServiceError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| CsvImportServiceError::InvalidFile)?;
    if !metadata.file_type().is_file() || !has_csv_extension(path) {
        return Err(CsvImportServiceError::InvalidFile);
    }
    if metadata.len() > MAX_CSV_BYTES {
        return Err(CsvImportServiceError::FileTooLarge);
    }

    let mut file = File::open(path).map_err(|_| CsvImportServiceError::InvalidFile)?;
    let metadata = file
        .metadata()
        .map_err(|_| CsvImportServiceError::InvalidFile)?;
    if !metadata.file_type().is_file() {
        return Err(CsvImportServiceError::InvalidFile);
    }
    read_bounded_file(&mut file)
}

pub(crate) fn revalidate_import_context(
    conn: &Connection,
    context: &ImportContext,
) -> Result<(), CsvImportServiceError> {
    revalidate_active_import_context(conn, &context.active)
}

pub(crate) fn revalidate_active_import_context(
    conn: &Connection,
    context: &ActiveImportContext,
) -> Result<(), CsvImportServiceError> {
    let is_current: bool = conn
        .query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM snapshots
                WHERE id = ?1
                  AND context_token = ?2
                  AND save_id = ?3
                  AND is_current = 1
                  AND EXISTS (
                      SELECT 1 FROM saves
                      WHERE id = ?3 AND context_token = ?4 AND is_active = 1
                  )
            )",
            params![
                context.snapshot_id,
                context.snapshot_context_token,
                context.save_id,
                context.save_context_token,
            ],
            |row| row.get(0),
        )
        .map_err(|_| CsvImportServiceError::Database)?;

    is_current
        .then_some(())
        .ok_or(CsvImportServiceError::StaleContext)
}

fn upsert_youth_career_stats(
    tx: &Transaction<'_>,
    save_id: i64,
    player: &PreparedYouthCareerStats,
) -> Result<(), CsvPersistenceError> {
    tx.execute(
        "INSERT INTO player_youth_career_stats (
            save_id, player_uid, career_appearances, international_caps, career_goals, career_assists
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(save_id, player_uid) DO UPDATE SET
            career_appearances = excluded.career_appearances,
            international_caps = excluded.international_caps,
            career_goals = excluded.career_goals,
            career_assists = excluded.career_assists,
            imported_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
        params![
            save_id,
            player.player_uid,
            player.career_appearances,
            player.international_caps,
            player.career_goals,
            player.career_assists,
        ],
    )
    .map_err(|_| CsvPersistenceError::Database)?;

    Ok(())
}

fn has_csv_extension(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("csv"))
}

fn read_bounded_file(file: &mut File) -> Result<String, CsvImportServiceError> {
    let metadata = file
        .metadata()
        .map_err(|_| CsvImportServiceError::InvalidFile)?;
    if metadata.len() > MAX_CSV_BYTES {
        return Err(CsvImportServiceError::FileTooLarge);
    }

    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.by_ref()
        .take(MAX_CSV_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| CsvImportServiceError::InvalidFile)?;
    if bytes.len() as u64 > MAX_CSV_BYTES {
        return Err(CsvImportServiceError::FileTooLarge);
    }

    String::from_utf8(bytes).map_err(|_| CsvImportServiceError::InvalidUtf8)
}

fn map_csv_error(error: ParsedCsvError) -> CsvImportServiceError {
    match error {
        ParsedCsvError::UnsupportedDialect | ParsedCsvError::MissingRequiredHeader(_) => {
            CsvImportServiceError::UnsupportedFormat
        }
        ParsedCsvError::MoneyballExport => CsvImportServiceError::MoneyballExport,
        ParsedCsvError::TooManyRows { .. } => CsvImportServiceError::TooManyRows,
        error => CsvImportServiceError::InvalidCsv(error),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use rusqlite::{params, Connection, OptionalExtension};

    use super::*;
    use crate::db::migrations;
    use crate::features::snapshot::ingest::{ingest_dump_file, ingest_dump_file_for_save};
    use crate::features::snapshot::service as snapshot_service;

    type CareerValues = (Option<i64>, Option<i64>, Option<i64>, Option<i64>);

    #[test]
    fn imports_only_matched_youth_rows_replaces_included_rows_and_preserves_omitted_rows() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("youth-import.db"));
        seed_current_snapshot(&mut conn, &[1, 2]);
        let first_path = write_csv(
            &temp_dir,
            "first.csv",
            "Unique ID;Player;AT Apps;Int Apps;AT Gls;Assists\n1;One;10;1;2;3\n2;Two;20;2;3;4\n",
        );

        assert_eq!(
            import_csv_file(&mut conn, &first_path).expect("import matching players"),
            CsvImportSummary {
                format: CsvImportFormat::YouthTracker,
                total_players: 2,
                stored_players: 2,
                skipped_players: 0,
            }
        );

        let save_id = active_save_id(&conn);
        conn.execute(
            "INSERT INTO player_youth_career_stats (save_id, player_uid, career_appearances)
             VALUES (?1, 99, 7)",
            [save_id],
        )
        .expect("seed departed player enrichment");

        let replacement_path = write_csv(
            &temp_dir,
            "replacement.csv",
            "Unique ID;Player;AT Apps;Int Apps;AT Gls;Assists\n1;One;99;;;\n99;Unknown;8;7;6;5\n",
        );
        assert_eq!(
            import_csv_file(&mut conn, &replacement_path).expect("replace matching player"),
            CsvImportSummary {
                format: CsvImportFormat::YouthTracker,
                total_players: 2,
                stored_players: 1,
                skipped_players: 1,
            }
        );

        assert_eq!(
            career_values(&conn, save_id, 1),
            Some((Some(99), None, None, None))
        );
        assert_eq!(
            career_values(&conn, save_id, 2),
            Some((Some(20), Some(2), Some(3), Some(4)))
        );
        assert_eq!(
            career_values(&conn, save_id, 99),
            Some((Some(7), None, None, None))
        );
    }

    #[test]
    fn rejects_a_stale_import_context_without_writing_rows() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("stale-import.db"));
        seed_current_snapshot(&mut conn, &[1]);
        let path = write_csv(
            &temp_dir,
            "player.csv",
            "Unique ID;Player;AT Apps\n1;One;12\n",
        );
        let context = capture_import_context(&conn).expect("capture context");
        let import = prepare_csv_import(&path).expect("prepare import outside transaction");

        conn.execute(
            "UPDATE snapshots SET is_current = 0 WHERE id = ?1",
            [context.active.snapshot_id],
        )
        .expect("replace current snapshot");

        assert_eq!(
            persist_csv_import(&mut conn, &context, import).expect_err("reject stale context"),
            CsvPersistenceError::Import(CsvImportServiceError::StaleContext)
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM player_youth_career_stats",
                [],
                |row| row.get::<_, i64>(0)
            )
            .expect("count enrichment rows"),
            0
        );
    }

    #[test]
    fn rejects_an_import_when_the_active_save_changes_after_parsing() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("stale-save-import.db"));
        seed_current_snapshot(&mut conn, &[1]);
        let path = write_csv(
            &temp_dir,
            "player.csv",
            "Unique ID;Player;AT Apps\n1;One;12\n",
        );
        let context = capture_import_context(&conn).expect("capture context");
        let import = prepare_csv_import(&path).expect("prepare import outside transaction");
        conn.execute(
            "UPDATE saves SET is_active = 0 WHERE id = ?1",
            [context.active.save_id],
        )
        .expect("deactivate captured save");
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('New active save', 1)",
            [],
        )
        .expect("activate another save");

        assert_eq!(
            persist_csv_import(&mut conn, &context, import).expect_err("reject stale save"),
            CsvPersistenceError::Import(CsvImportServiceError::StaleContext)
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM player_youth_career_stats",
                [],
                |row| row.get::<_, i64>(0)
            )
            .expect("count enrichment rows"),
            0
        );
    }

    #[test]
    fn rolls_back_every_matching_row_when_a_database_write_fails() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("rollback-import.db"));
        seed_current_snapshot(&mut conn, &[1, 2]);
        let path = write_csv(
            &temp_dir,
            "players.csv",
            "Unique ID;Player;AT Apps\n1;One;12\n2;Two;13\n",
        );
        let context = capture_import_context(&conn).expect("capture context");
        let import = prepare_csv_import(&path).expect("prepare import");
        conn.execute_batch(
            "CREATE TRIGGER reject_second_youth_import
             BEFORE INSERT ON player_youth_career_stats
             WHEN NEW.player_uid = 2
             BEGIN
                 SELECT RAISE(ABORT, 'test write failure');
             END;",
        )
        .expect("create test trigger");

        assert_eq!(
            persist_csv_import(&mut conn, &context, import).expect_err("reject failed write"),
            CsvPersistenceError::Database
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM player_youth_career_stats",
                [],
                |row| row.get::<_, i64>(0)
            )
            .expect("count rolled-back rows"),
            0
        );
    }

    #[test]
    fn enrichment_is_save_scoped_survives_snapshot_replacement_and_cascades_with_save_deletion() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("save-lifecycle-import.db"));
        let dump_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/features/memory_read/fixtures/golden_dump_v9.json");
        let first_save_id = snapshot_service::active_save_id(&conn).expect("create default save");
        ingest_dump_file_for_save(&mut conn, first_save_id, &dump_path)
            .expect("ingest first snapshot");
        let first_path = write_csv(
            &temp_dir,
            "first-save.csv",
            "Unique ID;Player;AT Apps\n77;Player;12\n",
        );
        import_csv_file(&mut conn, &first_path).expect("import first save");

        let second_save =
            snapshot_service::create_save(&conn, "Second save").expect("create second save");
        snapshot_service::set_active_save(&mut conn, second_save.id).expect("activate second save");
        ingest_dump_file_for_save(&mut conn, second_save.id, &dump_path)
            .expect("ingest second snapshot");
        let second_path = write_csv(
            &temp_dir,
            "second-save.csv",
            "Unique ID;Player;AT Apps\n77;Player;24\n",
        );
        import_csv_file(&mut conn, &second_path).expect("import second save");

        assert_eq!(
            career_values(&conn, first_save_id, 77),
            Some((Some(12), None, None, None))
        );
        assert_eq!(
            career_values(&conn, second_save.id, 77),
            Some((Some(24), None, None, None))
        );

        ingest_dump_file(&mut conn, &dump_path).expect("replace active snapshot");
        assert_eq!(
            career_values(&conn, second_save.id, 77),
            Some((Some(24), None, None, None))
        );

        conn.execute("DELETE FROM saves WHERE id = ?1", [second_save.id])
            .expect("delete second save");
        assert_eq!(career_values(&conn, second_save.id, 77), None);
        assert_eq!(
            career_values(&conn, first_save_id, 77),
            Some((Some(12), None, None, None))
        );
    }

    #[test]
    fn rejects_invalid_files_without_leaking_the_local_path() {
        let temp_dir = tempfile::tempdir().expect("temp dir");

        let text_path = write_csv(&temp_dir, "players.txt", "Unique ID;Player\n1;Player\n");
        let text_error = parse_csv_file(&text_path).expect_err("reject text file");
        assert_eq!(text_error, CsvImportServiceError::InvalidFile);
        assert!(!text_error
            .to_string()
            .contains(temp_dir.path().to_str().expect("temp path")));

        let utf8_path = temp_dir.path().join("invalid.csv");
        fs::write(&utf8_path, [0xff, 0xfe]).expect("write invalid UTF-8");
        assert_eq!(
            parse_csv_file(&utf8_path).expect_err("reject invalid UTF-8"),
            CsvImportServiceError::InvalidUtf8
        );
    }

    #[test]
    fn accepts_eight_mib_csv_and_rejects_a_larger_file() {
        const EIGHT_MIB: usize = 8 * 1024 * 1024;
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let path = temp_dir.path().join("bounded.csv");

        fs::write(&path, vec![b'x'; EIGHT_MIB]).expect("write CSV at byte limit");
        assert_eq!(
            read_csv_file(&path)
                .expect("accept CSV at byte limit")
                .len(),
            EIGHT_MIB
        );

        fs::write(&path, vec![b'x'; EIGHT_MIB + 1]).expect("write CSV above byte limit");
        assert_eq!(
            read_csv_file(&path).expect_err("reject CSV above byte limit"),
            CsvImportServiceError::FileTooLarge
        );
    }

    #[test]
    fn accepts_ten_thousand_player_rows_and_rejects_the_next_row() {
        const TEN_THOUSAND_ROWS: usize = 10_000;
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let player_csv = |rows| {
            std::iter::once("Unique ID;Player".to_string())
                .chain((1..=rows).map(|uid| format!("{uid};Player {uid}")))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let accepted_path = write_csv(
            &temp_dir,
            "ten-thousand.csv",
            &player_csv(TEN_THOUSAND_ROWS),
        );
        assert_eq!(
            parse_csv_file(&accepted_path)
                .expect("accept CSV at row limit")
                .len(),
            TEN_THOUSAND_ROWS
        );

        let rejected_path = write_csv(
            &temp_dir,
            "ten-thousand-and-one.csv",
            &player_csv(TEN_THOUSAND_ROWS + 1),
        );
        assert_eq!(
            parse_csv_file(&rejected_path).expect_err("reject CSV above row limit"),
            CsvImportServiceError::TooManyRows
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
            CsvImportServiceError::FileTooLarge
        );
    }

    #[test]
    fn propagates_invalid_csv_data_without_returning_raw_rows() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let path = write_csv(
            &temp_dir,
            "duplicate.csv",
            "Unique ID;Player\n1;First player\n1;Raw row that must not cross IPC\n",
        );

        let error = parse_csv_file(&path).expect_err("duplicate UID propagates");

        assert_eq!(
            error,
            CsvImportServiceError::InvalidCsv(ParsedCsvError::DuplicateUid {
                first_row: 2,
                row: 3,
            })
        );
        assert!(!error
            .to_string()
            .contains("Raw row that must not cross IPC"));
    }

    #[test]
    fn rejects_an_import_when_the_current_save_changes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("stale.db"));
        seed_current_snapshot(&mut conn, &[1]);
        let context = capture_import_context(&conn).expect("capture current snapshot");

        conn.execute("UPDATE saves SET is_active = 0 WHERE is_active = 1", [])
            .expect("deactivate prior save");
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES (?1, 1)",
            ["New active save"],
        )
        .expect("switch active save");

        assert_eq!(
            revalidate_import_context(&conn, &context).expect_err("reject stale context"),
            CsvImportServiceError::StaleContext
        );
    }

    #[test]
    fn rejects_an_import_when_the_current_snapshot_changes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("stale-snapshot.db"));
        seed_current_snapshot(&mut conn, &[1]);
        let context = capture_import_context(&conn).expect("capture current snapshot");

        conn.execute(
            "UPDATE snapshots SET is_current = 0 WHERE is_current = 1",
            [],
        )
        .expect("replace current snapshot");

        assert_eq!(
            revalidate_import_context(&conn, &context).expect_err("reject stale context"),
            CsvImportServiceError::StaleContext
        );
    }

    #[test]
    fn rejects_a_deleted_and_reused_current_snapshot_id() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("reused-snapshot-context.db"));
        seed_current_snapshot(&mut conn, &[1]);
        let context = capture_import_context(&conn).expect("capture current snapshot");

        conn.execute(
            "DELETE FROM snapshots WHERE id = ?1",
            [context.active.snapshot_id],
        )
        .expect("delete captured snapshot");
        let replacement_snapshot_id =
            insert_snapshot(&mut conn, context.active.save_id, true, &[1]);
        assert_eq!(replacement_snapshot_id, context.active.snapshot_id);

        assert_eq!(
            revalidate_import_context(&conn, &context).expect_err("reject reused snapshot id"),
            CsvImportServiceError::StaleContext
        );
    }

    #[test]
    fn requires_a_current_snapshot_without_creating_database_rows() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("no-snapshot.db"));
        let before = database_fingerprint(&conn);

        assert_eq!(
            capture_import_context(&conn).expect_err("no current snapshot"),
            CsvImportServiceError::NoCurrentSnapshot
        );

        assert_eq!(database_fingerprint(&conn), before);
    }

    fn import_csv_file(
        conn: &mut Connection,
        path: &Path,
    ) -> Result<CsvImportSummary, CsvPersistenceError> {
        let context = capture_import_context(conn)?;
        let import = prepare_csv_import(path)?;
        persist_csv_import(conn, &context, import)
    }

    fn active_save_id(conn: &Connection) -> i64 {
        conn.query_row("SELECT id FROM saves WHERE is_active = 1", [], |row| {
            row.get(0)
        })
        .expect("read active save")
    }

    fn career_values(conn: &Connection, save_id: i64, player_uid: i64) -> Option<CareerValues> {
        conn.query_row(
            "SELECT career_appearances, international_caps, career_goals, career_assists
             FROM player_youth_career_stats
             WHERE save_id = ?1 AND player_uid = ?2",
            params![save_id, player_uid],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .expect("read career enrichment")
    }

    fn open_migrated(path: &Path) -> Connection {
        let conn = Connection::open(path).expect("open database");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    fn seed_current_snapshot(conn: &mut Connection, uids: &[u32]) {
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES (?1, 1)",
            ["Import save"],
        )
        .expect("create active save");
        let save_id = conn.last_insert_rowid();
        insert_snapshot(conn, save_id, true, uids);
    }

    fn insert_snapshot(conn: &mut Connection, save_id: i64, is_current: bool, uids: &[u32]) -> i64 {
        conn.execute(
            "INSERT INTO snapshots (
                save_id, is_current, schema_version, generated_at_utc, game_version,
                supported_game_version, bridge_version, protocol_version, game_date_source,
                game_date_basis, player_database_scope, scan_truncated, max_accepted,
                player_count, staff_count
            ) VALUES (?1, ?2, 6, '2026-01-01T00:00:00Z', '26.0', '26.0', 'test', 1,
                'memory', 'memory', 'men', 0, NULL, ?3, 0)",
            params![save_id, is_current, uids.len() as i64],
        )
        .expect("create snapshot");
        let snapshot_id = conn.last_insert_rowid();
        let tx = conn.transaction().expect("begin player transaction");
        let mut statement = tx
            .prepare(
                "INSERT INTO players (
                    snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                    nationalities_json, preferred_foot, positions_json, attributes_json,
                    hidden_attributes_json, personality_json
                ) VALUES (?1, ?2, 100, 100, 'Import player', 2000, 1, '[]', 'Right',
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
        snapshot_id
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
