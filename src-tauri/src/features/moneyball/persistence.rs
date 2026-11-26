//! Writes snapshot-owned Moneyball cohorts: one row per player with a column for each of the
//! 138 catalog statistics (see `columns`), and whole cohort replacement with its as-of date. A
//! cohort stays with its snapshot until that is deleted.
//!
//! Percentiles are not stored: `moneyball::comparison` ranks the stored statistics when they
//! are read. The `percentiles_json` column only marks a row written by an import that could be
//! scored: rows from before v30 have none, and readers ask for a refresh instead.

use std::sync::OnceLock;

use rusqlite::{params, types::Null, Statement, Transaction};

use super::columns::statistic_column;
use super::{MoneyballMetricValue, MoneyballStatistics, MONEYBALL_STATISTIC_KEYS};

/// What `percentiles_json` holds for a scored row: an empty object.
const SCORED_ROW_MARKER: &str = "{}";

/// The insert's first statistic parameter, after the row's six other values.
const FIRST_STATISTIC_PARAMETER: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MoneyballPersistenceError {
    InvalidStatistics,
    Database,
}

/// One player's row in a replacement cohort. The save provides no asking price.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CohortRow {
    pub player_uid: i64,
    pub starts: i64,
    pub substitute_appearances: i64,
    pub minutes: i64,
    pub statistics: MoneyballStatistics,
}

/// A row's statistics in `MONEYBALL_STATISTIC_KEYS` order. They must be the whole catalog and
/// nothing else, with finite decimals.
fn catalog_values(
    statistics: &MoneyballStatistics,
) -> Result<Box<[Option<MoneyballMetricValue>]>, MoneyballPersistenceError> {
    if statistics.len() != MONEYBALL_STATISTIC_KEYS.len() {
        return Err(MoneyballPersistenceError::InvalidStatistics);
    }
    MONEYBALL_STATISTIC_KEYS
        .iter()
        .map(|key| match statistics.get(*key) {
            Some(Some(MoneyballMetricValue::Decimal(value))) if !value.is_finite() => {
                Err(MoneyballPersistenceError::InvalidStatistics)
            }
            Some(value) => Ok(*value),
            None => Err(MoneyballPersistenceError::InvalidStatistics),
        })
        .collect()
}

/// A replacement cohort whose statistics are already checked and in column order, so writing
/// it only replaces rows.
#[derive(Debug)]
pub(crate) struct PreparedCohort {
    rows: Vec<PreparedCohortRow>,
}

#[derive(Debug)]
struct PreparedCohortRow {
    player_uid: i64,
    starts: i64,
    substitute_appearances: i64,
    minutes: i64,
    /// In `MONEYBALL_STATISTIC_KEYS` order.
    statistics: Box<[Option<MoneyballMetricValue>]>,
}

#[cfg(test)]
thread_local! {
    /// Runs once a preparation has finished, so a test can observe what was held meanwhile.
    pub(crate) static PREPARED_HOOK: std::cell::RefCell<Option<Box<dyn FnMut()>>> =
        std::cell::RefCell::new(None);
}

/// Checks each row's statistics and puts them in column order. It needs no database, so
/// callers run it before taking the writer.
pub(crate) fn prepare_snapshot_cohort(
    rows: &[CohortRow],
) -> Result<PreparedCohort, MoneyballPersistenceError> {
    let rows = rows
        .iter()
        .map(|row| {
            Ok(PreparedCohortRow {
                player_uid: row.player_uid,
                starts: row.starts,
                substitute_appearances: row.substitute_appearances,
                minutes: row.minutes,
                statistics: catalog_values(&row.statistics)?,
            })
        })
        .collect::<Result<_, _>>()?;

    #[cfg(test)]
    PREPARED_HOOK.with(|hook| {
        if let Some(hook) = hook.borrow_mut().as_mut() {
            hook();
        }
    });
    Ok(PreparedCohort { rows })
}

/// The insert of one cohort row: its six other values, then every statistic in catalog order.
fn insert_sql() -> &'static str {
    static SQL: OnceLock<String> = OnceLock::new();
    SQL.get_or_init(|| {
        let statistics = 0..MONEYBALL_STATISTIC_KEYS.len();
        let columns = statistics
            .clone()
            .map(|index| format!(", {}", statistic_column(index)))
            .collect::<String>();
        let parameters = statistics
            .map(|index| format!(", ?{}", FIRST_STATISTIC_PARAMETER + index))
            .collect::<String>();
        format!(
            "INSERT INTO player_moneyball_stats (
                snapshot_id, player_uid, starts, substitute_appearances, minutes,
                percentiles_json{columns}
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6{parameters})"
        )
    })
}

fn insert_row(
    insert: &mut Statement<'_>,
    snapshot_id: i64,
    row: &PreparedCohortRow,
) -> rusqlite::Result<()> {
    insert.raw_bind_parameter(1, snapshot_id)?;
    insert.raw_bind_parameter(2, row.player_uid)?;
    insert.raw_bind_parameter(3, row.starts)?;
    insert.raw_bind_parameter(4, row.substitute_appearances)?;
    insert.raw_bind_parameter(5, row.minutes)?;
    insert.raw_bind_parameter(6, SCORED_ROW_MARKER)?;
    for (index, value) in row.statistics.iter().enumerate() {
        let parameter = FIRST_STATISTIC_PARAMETER + index;
        match value {
            None => insert.raw_bind_parameter(parameter, Null)?,
            Some(MoneyballMetricValue::Count(value)) => {
                insert.raw_bind_parameter(parameter, value)?;
            }
            Some(MoneyballMetricValue::Decimal(value)) => {
                insert.raw_bind_parameter(parameter, value)?;
            }
        }
    }
    insert.raw_execute().map(|_| ())
}

/// Replaces the snapshot's whole cohort with the prepared rows and records the save's in-game
/// date as the cohort's as-of date. The caller owns the transaction, so any failure leaves the
/// prior cohort and date in place.
pub(crate) fn replace_snapshot_cohort(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    cohort: &PreparedCohort,
    stats_game_date: &str,
) -> Result<(), MoneyballPersistenceError> {
    tx.execute(
        "DELETE FROM player_moneyball_stats WHERE snapshot_id = ?1",
        [snapshot_id],
    )
    .map_err(|_| MoneyballPersistenceError::Database)?;

    let mut insert = tx
        .prepare(insert_sql())
        .map_err(|_| MoneyballPersistenceError::Database)?;
    for row in &cohort.rows {
        insert_row(&mut insert, snapshot_id, row)
            .map_err(|_| MoneyballPersistenceError::Database)?;
    }

    let changed = tx
        .execute(
            "UPDATE snapshots SET moneyball_stats_game_date = ?1 WHERE id = ?2",
            params![stats_game_date, snapshot_id],
        )
        .map_err(|_| MoneyballPersistenceError::Database)?;
    if changed != 1 {
        return Err(MoneyballPersistenceError::Database);
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod test_support {
    use rusqlite::{params, params_from_iter, types::Value, Connection};

    use crate::features::moneyball::columns::statistic_column_of;
    use crate::features::moneyball::json_scan::scan_json_object;

    /// Gives `snapshot_id` a one-player cohort with every statistic null and `as_of` as its
    /// as-of date. The player must belong to the snapshot.
    pub(crate) fn seed_cohort(conn: &Connection, snapshot_id: i64, player_uid: i64, as_of: &str) {
        conn.execute(
            "INSERT INTO player_moneyball_stats (snapshot_id, player_uid) VALUES (?1, ?2)",
            params![snapshot_id, player_uid],
        )
        .expect("seed Moneyball row");
        conn.execute(
            "UPDATE snapshots SET moneyball_stats_game_date = ?1 WHERE id = ?2",
            params![as_of, snapshot_id],
        )
        .expect("seed as-of date");
    }

    /// Inserts a player's statistics row from a JSON object of catalog keys and numbers. A
    /// statistic the object lacks or holds as `null` is NULL.
    pub(crate) fn insert_statistics_row(
        conn: &Connection,
        snapshot_id: i64,
        player_uid: i64,
        minutes: Option<i64>,
        statistics_json: &str,
        percentiles_json: Option<&str>,
    ) {
        let mut columns = String::new();
        let mut values = vec![
            Value::from(snapshot_id),
            Value::from(player_uid),
            Value::from(minutes),
            Value::from(percentiles_json.map(str::to_string)),
        ];
        scan_json_object(statistics_json, |key, text| {
            let column = statistic_column_of(key)
                .unwrap_or_else(|| panic!("{key} is not a catalog statistic"));
            columns.push_str(", ");
            columns.push_str(column);
            values.push(match text.parse::<i64>() {
                Ok(count) => Value::Integer(count),
                Err(_) if text == "null" => Value::Null,
                Err(_) => Value::Real(text.parse().expect("a statistic is a number or null")),
            });
            Ok(())
        })
        .expect("statistics object");
        let parameters = (1..=values.len())
            .map(|parameter| format!("?{parameter}"))
            .collect::<Vec<_>>()
            .join(", ");
        conn.execute(
            &format!(
                "INSERT INTO player_moneyball_stats (
                    snapshot_id, player_uid, minutes, percentiles_json{columns}
                 ) VALUES ({parameters})"
            ),
            params_from_iter(values),
        )
        .expect("insert statistics row");
    }

    /// The snapshot's Moneyball row count and as-of date.
    pub(crate) fn cohort_state(conn: &Connection, snapshot_id: i64) -> (i64, Option<String>) {
        conn.query_row(
            "SELECT
                 (SELECT COUNT(*) FROM player_moneyball_stats WHERE snapshot_id = ?1),
                 moneyball_stats_game_date
             FROM snapshots WHERE id = ?1",
            [snapshot_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("read cohort state")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use rusqlite::{params, Connection};

    use super::*;
    use crate::db::migrations;
    use crate::features::moneyball::columns::StatisticColumns;
    use crate::features::snapshot::service as snapshot_service;

    /// Players in an unsorted order, with counts, decimals, ties, negatives, and nulls in
    /// every statistic, including the lower-is-better ones.
    fn fixed_cohort() -> Vec<CohortRow> {
        [41_i64, 7, 23, 3, 88, 15, 60, 2, 34, 19, 72, 5]
            .into_iter()
            .enumerate()
            .map(|(index, player_uid)| {
                let statistics = MONEYBALL_STATISTIC_KEYS
                    .iter()
                    .enumerate()
                    .map(|(key_index, key)| {
                        let value = match (index + key_index) % 6 {
                            0 => None,
                            1 => Some(MoneyballMetricValue::Count(
                                ((index * key_index) % 4) as u32,
                            )),
                            2 => Some(MoneyballMetricValue::Decimal(
                                index as f64 * 0.25 + key_index as f64,
                            )),
                            3 => Some(MoneyballMetricValue::Count(7)),
                            4 => Some(MoneyballMetricValue::Decimal(index as f64 / 3.0)),
                            _ => Some(MoneyballMetricValue::Decimal(-(index as f64) * 1.5)),
                        };
                        ((*key).to_string(), value)
                    })
                    .collect::<BTreeMap<_, _>>();
                CohortRow {
                    player_uid,
                    starts: index as i64,
                    substitute_appearances: (index % 3) as i64,
                    minutes: index as i64 * 90,
                    statistics,
                }
            })
            .collect()
    }

    /// A database whose current snapshot holds the players of `rows`.
    fn database(rows: &[CohortRow]) -> (Connection, i64) {
        let conn = Connection::open_in_memory().expect("open db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        let save = snapshot_service::list_saves(&conn)
            .expect("seed default save")
            .remove(0);
        conn.execute(
            "INSERT INTO snapshots (
                save_id, is_current, schema_version, generated_at_utc, game_version,
                supported_game_version, bridge_version, protocol_version, game_date_source,
                scan_truncated, max_accepted, player_count
             ) VALUES (?1, 1, 6, '2031-03-01T00:00:00Z', '27.0', '27.0', 'test', 1, 'memory',
                0, NULL, ?2)",
            params![save.id, rows.len() as i64],
        )
        .expect("insert snapshot");
        let snapshot_id = conn.last_insert_rowid();
        for row in rows {
            conn.execute(
                "INSERT INTO players (
                    snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                    nationalities_json, preferred_foot, positions_json, attributes_json,
                    hidden_attributes_json, personality_json
                 ) VALUES (?1, ?2, 100, 100, 'Cohort player', 2000, 1, '[]', 'Right', '{}',
                    '{}', '{}', '{}')",
                params![snapshot_id, row.player_uid],
            )
            .expect("insert player");
        }
        (conn, snapshot_id)
    }

    /// A stored row: its player, playing time, scored-row marker, and each statistic.
    type StoredRow = (i64, i64, i64, i64, String, Vec<Option<f64>>);

    /// The snapshot's stored cohort rows by player, and its as-of date.
    fn stored(conn: &Connection, snapshot_id: i64) -> (Vec<StoredRow>, Option<String>) {
        let statistics = StatisticColumns::all();
        let rows = conn
            .prepare(&format!(
                "SELECT stats.player_uid, stats.starts, stats.substitute_appearances,
                        stats.minutes, stats.percentiles_json{}
                 FROM player_moneyball_stats stats
                 WHERE stats.snapshot_id = ?1 ORDER BY stats.player_uid",
                statistics.appended_sql("stats")
            ))
            .expect("prepare stored rows")
            .query_map([snapshot_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    statistics
                        .read(row, 5)?
                        .iter()
                        .map(|value| Some(*value).filter(|value| !value.is_nan()))
                        .collect(),
                ))
            })
            .expect("query stored rows")
            .collect::<Result<_, _>>()
            .expect("read stored rows");
        let as_of = conn
            .query_row(
                "SELECT moneyball_stats_game_date FROM snapshots WHERE id = ?1",
                [snapshot_id],
                |row| row.get(0),
            )
            .expect("read as-of date");
        (rows, as_of)
    }

    #[test]
    fn a_replacement_stores_each_rows_statistics_and_marks_it_scored() {
        let rows = fixed_cohort();
        let (mut conn, snapshot_id) = database(&rows);
        let cohort = prepare_snapshot_cohort(&rows).expect("prepare cohort");
        let tx = conn.transaction().expect("begin write");
        replace_snapshot_cohort(&tx, snapshot_id, &cohort, "2031-05-01").expect("replacement");
        tx.commit().expect("commit write");

        let mut expected = rows
            .iter()
            .map(|row| {
                let statistics = MONEYBALL_STATISTIC_KEYS
                    .iter()
                    .map(|key| match row.statistics[*key] {
                        None => None,
                        Some(MoneyballMetricValue::Count(value)) => Some(f64::from(value)),
                        Some(MoneyballMetricValue::Decimal(value)) => Some(value),
                    })
                    .collect::<Vec<_>>();
                (
                    row.player_uid,
                    row.starts,
                    row.substitute_appearances,
                    row.minutes,
                    "{}".to_string(),
                    statistics,
                )
            })
            .collect::<Vec<_>>();
        expected.sort_by_key(|row| row.0);
        assert_eq!(
            stored(&conn, snapshot_id),
            (expected, Some("2031-05-01".to_string()))
        );
    }

    #[test]
    fn a_cohort_must_hold_the_whole_catalog_in_finite_values() {
        let whole = fixed_cohort().remove(0);
        let with = |change: &dyn Fn(&mut MoneyballStatistics)| {
            let mut row = whole.clone();
            change(&mut row.statistics);
            prepare_snapshot_cohort(&[row]).map(|_| ())
        };

        assert_eq!(with(&|_| {}), Ok(()));
        for (case, result) in [
            (
                "a missing statistic",
                with(&|statistics| {
                    statistics.remove("goals");
                }),
            ),
            (
                "a statistic outside the catalog",
                with(&|statistics| {
                    statistics.insert("own_goals".to_string(), None);
                }),
            ),
            (
                "another statistic in place of a catalog one",
                with(&|statistics| {
                    statistics.remove("goals");
                    statistics.insert("own_goals".to_string(), None);
                }),
            ),
            (
                "a decimal that is not a number",
                with(&|statistics| {
                    statistics.insert(
                        "xg".to_string(),
                        Some(MoneyballMetricValue::Decimal(f64::NAN)),
                    );
                }),
            ),
            (
                "an infinite decimal",
                with(&|statistics| {
                    statistics.insert(
                        "xg".to_string(),
                        Some(MoneyballMetricValue::Decimal(f64::INFINITY)),
                    );
                }),
            ),
        ] {
            assert_eq!(
                result,
                Err(MoneyballPersistenceError::InvalidStatistics),
                "{case}"
            );
        }
    }
}
