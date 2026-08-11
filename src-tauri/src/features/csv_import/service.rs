use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::Serialize;
use serde_json::{Map, Number, Value};

use super::parser::{parse_csv_with_row_limit, ParsedCsv};
use super::{CsvImportError, MoneyballMetricValue, MoneyballPlayer, MoneyballTransferValue};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CsvImportSummary {
    pub format: CsvPreviewFormat,
    pub total_players: usize,
    pub stored_players: usize,
    pub skipped_players: usize,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CsvPersistenceError {
    Preview(CsvPreviewError),
    NumericValueOutOfRange,
    InvalidStatistics,
    Database,
}

impl std::fmt::Display for CsvPersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preview(error) => error.fmt(f),
            Self::NumericValueOutOfRange => {
                write!(f, "CSV contains a value that is too large to store")
            }
            Self::InvalidStatistics => write!(f, "CSV contains a statistic that cannot be stored"),
            Self::Database => write!(f, "CSV import is unavailable"),
        }
    }
}

impl std::error::Error for CsvPersistenceError {}

impl From<CsvPreviewError> for CsvPersistenceError {
    fn from(error: CsvPreviewError) -> Self {
        Self::Preview(error)
    }
}

pub(crate) enum PreparedCsvImport {
    YouthTracker(Vec<PreparedYouthCareerStats>),
    Moneyball(Vec<PreparedMoneyballStats>),
}

pub(crate) struct PreparedYouthCareerStats {
    player_uid: i64,
    career_appearances: Option<i64>,
    international_caps: Option<i64>,
    career_goals: Option<i64>,
    career_assists: Option<i64>,
}

pub(crate) struct PreparedMoneyballStats {
    player_uid: i64,
    asking_price_kind: Option<&'static str>,
    asking_price_lower_eur: Option<i64>,
    asking_price_upper_eur: Option<i64>,
    starts: Option<i64>,
    substitute_appearances: Option<i64>,
    minutes: Option<i64>,
    statistics_json: String,
}

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
    let parsed = parse_csv_file(path)?;
    let (format, uids) = match parsed {
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

pub(crate) fn prepare_csv_import(path: &Path) -> Result<PreparedCsvImport, CsvPersistenceError> {
    match parse_csv_file(path)? {
        ParsedCsv::YouthTracker(players) => Ok(PreparedCsvImport::YouthTracker(
            players
                .into_iter()
                .map(|player| PreparedYouthCareerStats {
                    player_uid: i64::from(player.uid),
                    career_appearances: player.all_time_appearances.map(i64::from),
                    international_caps: player.international_appearances.map(i64::from),
                    career_goals: player.all_time_goals.map(i64::from),
                    career_assists: player.assists.map(i64::from),
                })
                .collect(),
        )),
        ParsedCsv::Moneyball(players) => players
            .into_iter()
            .map(prepare_moneyball_stats)
            .collect::<Result<Vec<_>, _>>()
            .map(PreparedCsvImport::Moneyball),
    }
}

pub(crate) fn persist_csv_import(
    conn: &mut Connection,
    context: &PreviewContext,
    import: PreparedCsvImport,
) -> Result<CsvImportSummary, CsvPersistenceError> {
    let tx = conn
        .transaction()
        .map_err(|_| CsvPersistenceError::Database)?;
    revalidate_preview_context(&tx, context)?;

    let summary = match import {
        PreparedCsvImport::YouthTracker(players) => {
            let total_players = players.len();
            let mut stored_players = 0;
            for player in players {
                if !context.player_uids.contains(&player.player_uid) {
                    continue;
                }
                upsert_youth_career_stats(&tx, context.save_id, &player)?;
                stored_players += 1;
            }
            CsvImportSummary {
                format: CsvPreviewFormat::YouthTracker,
                total_players,
                stored_players,
                skipped_players: total_players - stored_players,
            }
        }
        PreparedCsvImport::Moneyball(players) => {
            let total_players = players.len();
            let mut stored_players = 0;
            for player in players {
                if !context.player_uids.contains(&player.player_uid) {
                    continue;
                }
                upsert_moneyball_stats(&tx, context.save_id, &player)?;
                stored_players += 1;
            }
            CsvImportSummary {
                format: CsvPreviewFormat::Moneyball,
                total_players,
                stored_players,
                skipped_players: total_players - stored_players,
            }
        }
    };

    tx.commit().map_err(|_| CsvPersistenceError::Database)?;
    Ok(summary)
}

fn parse_csv_file(path: &Path) -> Result<ParsedCsv, CsvPreviewError> {
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
    parse_csv_with_row_limit(&input, Some(MAX_CSV_ROWS)).map_err(map_csv_error)
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

fn prepare_moneyball_stats(
    player: MoneyballPlayer,
) -> Result<PreparedMoneyballStats, CsvPersistenceError> {
    let (asking_price_kind, asking_price_lower_eur, asking_price_upper_eur) =
        match player.asking_price {
            None => (None, None, None),
            Some(MoneyballTransferValue::Single { euros }) => {
                (Some("single"), Some(sqlite_integer(euros)?), None)
            }
            Some(MoneyballTransferValue::Range {
                lower_euros,
                upper_euros,
            }) => (
                Some("range"),
                Some(sqlite_integer(lower_euros)?),
                Some(sqlite_integer(upper_euros)?),
            ),
            Some(MoneyballTransferValue::NotForSale) => (Some("not_for_sale"), None, None),
        };

    Ok(PreparedMoneyballStats {
        player_uid: i64::from(player.uid),
        asking_price_kind,
        asking_price_lower_eur,
        asking_price_upper_eur,
        starts: player
            .appearances
            .as_ref()
            .map(|appearances| i64::from(appearances.starts)),
        substitute_appearances: player
            .appearances
            .as_ref()
            .map(|appearances| i64::from(appearances.substitutes)),
        minutes: player.minutes.map(i64::from),
        statistics_json: serialize_moneyball_statistics(&player)?,
    })
}

fn sqlite_integer(value: u64) -> Result<i64, CsvPersistenceError> {
    i64::try_from(value).map_err(|_| CsvPersistenceError::NumericValueOutOfRange)
}

fn serialize_moneyball_statistics(player: &MoneyballPlayer) -> Result<String, CsvPersistenceError> {
    let mut statistics = Map::new();
    for (key, value) in player.canonical_statistics() {
        let value = match value {
            None => Value::Null,
            Some(MoneyballMetricValue::Count(value)) => Value::Number(Number::from(value)),
            Some(MoneyballMetricValue::Decimal(value)) => Value::Number(
                Number::from_f64(value).ok_or(CsvPersistenceError::InvalidStatistics)?,
            ),
        };
        statistics.insert(key, value);
    }

    serde_json::to_string(&Value::Object(statistics))
        .map_err(|_| CsvPersistenceError::InvalidStatistics)
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

fn upsert_moneyball_stats(
    tx: &Transaction<'_>,
    save_id: i64,
    player: &PreparedMoneyballStats,
) -> Result<(), CsvPersistenceError> {
    tx.execute(
        "INSERT INTO player_moneyball_stats (
            save_id, player_uid, asking_price_kind, asking_price_lower_eur, asking_price_upper_eur,
            starts, substitute_appearances, minutes, statistics_json
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(save_id, player_uid) DO UPDATE SET
            asking_price_kind = excluded.asking_price_kind,
            asking_price_lower_eur = excluded.asking_price_lower_eur,
            asking_price_upper_eur = excluded.asking_price_upper_eur,
            starts = excluded.starts,
            substitute_appearances = excluded.substitute_appearances,
            minutes = excluded.minutes,
            statistics_json = excluded.statistics_json,
            imported_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
        params![
            save_id,
            player.player_uid,
            player.asking_price_kind,
            player.asking_price_lower_eur,
            player.asking_price_upper_eur,
            player.starts,
            player.substitute_appearances,
            player.minutes,
            player.statistics_json,
        ],
    )
    .map_err(|_| CsvPersistenceError::Database)?;

    Ok(())
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

    use rusqlite::{params, Connection, OptionalExtension};
    use serde_json::{json, Value};

    use super::*;
    use crate::db::migrations;
    use crate::features::csv_import::parser::{parse_csv, ParsedCsv};
    use crate::features::snapshot::ingest::{ingest_dump_file, ingest_dump_file_for_save};
    use crate::features::snapshot::service as snapshot_service;

    const YOUTH_EXPORT: &str = include_str!("fixtures/2030_07_01_Full_Squad_CA_PA_Monza.csv");
    const MONEYBALL_EXPORT: &str = include_str!("fixtures/moneyball_stats.csv");
    type CareerValues = (Option<i64>, Option<i64>, Option<i64>, Option<i64>);
    type MoneyballStoredValues = (
        Option<String>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        String,
    );

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
                format: CsvPreviewFormat::YouthTracker,
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
                format: CsvPreviewFormat::YouthTracker,
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
    fn imports_moneyball_statistics_without_replacing_youth_enrichment() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-import.db"));
        let uid = 2_002_402_173;
        seed_current_snapshot(&mut conn, &[uid]);
        let youth_path = write_csv(
            &temp_dir,
            "youth.csv",
            "Unique ID;Player;AT Apps\n2002402173;Andrea Bisceglia;12\n",
        );
        import_csv_file(&mut conn, &youth_path).expect("import Youth enrichment");
        let moneyball_path = write_csv(
            &temp_dir,
            "moneyball.csv",
            &append_moneyball_column(MONEYBALL_EXPORT, "Asking Price", "€1M - €2M"),
        );

        assert_eq!(
            import_csv_file(&mut conn, &moneyball_path).expect("import Moneyball enrichment"),
            CsvImportSummary {
                format: CsvPreviewFormat::Moneyball,
                total_players: 75,
                stored_players: 1,
                skipped_players: 74,
            }
        );

        let (
            asking_price_kind,
            asking_price_lower,
            asking_price_upper,
            starts,
            substitutes,
            minutes,
            statistics_json,
        ): MoneyballStoredValues = conn
            .query_row(
                "SELECT asking_price_kind, asking_price_lower_eur, asking_price_upper_eur,
                        starts, substitute_appearances, minutes, statistics_json
                 FROM player_moneyball_stats
                 WHERE player_uid = ?1",
                params![uid],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )
            .expect("read Moneyball enrichment");
        assert_eq!(
            (
                asking_price_kind,
                asking_price_lower,
                asking_price_upper,
                starts,
                substitutes,
                minutes,
            ),
            (
                Some("range".to_string()),
                Some(1_000_000),
                Some(2_000_000),
                Some(0),
                Some(0),
                Some(0),
            )
        );
        let statistics: Value = serde_json::from_str(&statistics_json).expect("parse statistics");
        assert_eq!(
            statistics.as_object().expect("statistics object").len(),
            138
        );
        assert_eq!(statistics["goals"], json!(0));
        assert_eq!(statistics["goals_per_90"], Value::Null);
        assert_eq!(
            career_values(&conn, active_save_id(&conn), i64::from(uid)),
            Some((Some(12), None, None, None))
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
        let context = capture_preview_context(&conn).expect("capture context");
        let import = prepare_csv_import(&path).expect("prepare import outside transaction");

        conn.execute(
            "UPDATE snapshots SET is_current = 0 WHERE id = ?1",
            [context.snapshot_id],
        )
        .expect("replace current snapshot");

        assert_eq!(
            persist_csv_import(&mut conn, &context, import).expect_err("reject stale context"),
            CsvPersistenceError::Preview(CsvPreviewError::StaleContext)
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
        let context = capture_preview_context(&conn).expect("capture context");
        let import = prepare_csv_import(&path).expect("prepare import outside transaction");
        conn.execute(
            "UPDATE saves SET is_active = 0 WHERE id = ?1",
            [context.save_id],
        )
        .expect("deactivate captured save");
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('New active save', 1)",
            [],
        )
        .expect("activate another save");

        assert_eq!(
            persist_csv_import(&mut conn, &context, import).expect_err("reject stale save"),
            CsvPersistenceError::Preview(CsvPreviewError::StaleContext)
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
        let context = capture_preview_context(&conn).expect("capture context");
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
    fn invalid_and_overflow_imports_leave_prior_enrichment_unchanged() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("invalid-import.db"));
        let uid = 2_002_402_173;
        seed_current_snapshot(&mut conn, &[uid]);
        let moneyball_path = write_csv(&temp_dir, "moneyball.csv", MONEYBALL_EXPORT);
        import_csv_file(&mut conn, &moneyball_path).expect("import baseline Moneyball data");
        let save_id = active_save_id(&conn);
        let before: String = conn
            .query_row(
                "SELECT statistics_json FROM player_moneyball_stats WHERE save_id = ?1 AND player_uid = ?2",
                params![save_id, uid],
                |row| row.get(0),
            )
            .expect("read baseline statistics");

        let malformed_path = write_csv(
            &temp_dir,
            "duplicate.csv",
            "Unique ID;Player\n2002402173;First\n2002402173;Raw row that must not cross IPC\n",
        );
        let malformed =
            import_csv_file(&mut conn, &malformed_path).expect_err("reject duplicate UID");
        assert_eq!(
            malformed,
            CsvPersistenceError::Preview(CsvPreviewError::InvalidCsv(
                CsvImportError::DuplicateUid {
                    first_row: 2,
                    row: 3,
                }
            ))
        );
        assert!(!malformed
            .to_string()
            .contains("Raw row that must not cross IPC"));

        let overflow_path = write_csv(
            &temp_dir,
            "overflow.csv",
            &append_moneyball_column(MONEYBALL_EXPORT, "Asking Price", "€9223372036854775808"),
        );
        assert_eq!(
            import_csv_file(&mut conn, &overflow_path).expect_err("reject SQLite integer overflow"),
            CsvPersistenceError::NumericValueOutOfRange
        );
        let after: String = conn
            .query_row(
                "SELECT statistics_json FROM player_moneyball_stats WHERE save_id = ?1 AND player_uid = ?2",
                params![save_id, uid],
                |row| row.get(0),
            )
            .expect("read preserved statistics");
        assert_eq!(after, before);
    }

    #[test]
    fn enrichment_is_save_scoped_survives_snapshot_replacement_and_cascades_with_save_deletion() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("save-lifecycle-import.db"));
        let dump_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/features/memory_read/fixtures/golden_dump_v6.json");
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

    fn import_csv_file(
        conn: &mut Connection,
        path: &Path,
    ) -> Result<CsvImportSummary, CsvPersistenceError> {
        let context = capture_preview_context(conn)?;
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

    fn append_moneyball_column(input: &str, header: &str, first_row_value: &str) -> String {
        input
            .lines()
            .enumerate()
            .map(|(index, line)| match index {
                0 => format!("{line};{header}"),
                1 => format!("{line};{first_row_value}"),
                _ => format!("{line};-"),
            })
            .collect::<Vec<_>>()
            .join("\n")
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
