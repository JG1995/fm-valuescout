//! Writes snapshot-owned Moneyball cohorts: the 138-key statistics JSON and whole cohort
//! replacement with its as-of date. A cohort stays with its snapshot until that is deleted.
//!
//! Percentiles are not stored: `moneyball::comparison` ranks the stored statistics when they
//! are read. The `percentiles_json` column only marks a row written by an import that could be
//! scored: rows from before v30 have none, and readers ask for a refresh instead.

use rusqlite::{params, Transaction};
use serde_json::{Map, Number, Value};

use super::{MoneyballMetricValue, MoneyballStatistics};

/// What `percentiles_json` holds for a scored row: an empty object.
const SCORED_ROW_MARKER: &str = "{}";

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

pub(crate) fn serialize_moneyball_statistics(
    statistics: &MoneyballStatistics,
) -> Result<String, MoneyballPersistenceError> {
    let mut serialized = Map::new();
    for (key, value) in statistics {
        let value = match value {
            None => Value::Null,
            Some(MoneyballMetricValue::Count(value)) => Value::Number(Number::from(*value)),
            Some(MoneyballMetricValue::Decimal(value)) => Value::Number(
                Number::from_f64(*value).ok_or(MoneyballPersistenceError::InvalidStatistics)?,
            ),
        };
        serialized.insert(key.clone(), value);
    }

    serde_json::to_string(&Value::Object(serialized))
        .map_err(|_| MoneyballPersistenceError::InvalidStatistics)
}

/// A replacement cohort whose statistics JSON is already built, so writing it only replaces
/// rows.
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
    statistics_json: String,
}

#[cfg(test)]
thread_local! {
    /// Runs once a preparation has finished, so a test can observe what was held meanwhile.
    pub(crate) static PREPARED_HOOK: std::cell::RefCell<Option<Box<dyn FnMut()>>> =
        std::cell::RefCell::new(None);
}

/// Serializes each row's statistics. It needs no database, so callers run it before taking
/// the writer.
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
                statistics_json: serialize_moneyball_statistics(&row.statistics)?,
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
        .prepare(
            "INSERT INTO player_moneyball_stats (
                snapshot_id, player_uid, starts, substitute_appearances, minutes,
                statistics_json, percentiles_json
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .map_err(|_| MoneyballPersistenceError::Database)?;
    for row in &cohort.rows {
        insert
            .execute(params![
                snapshot_id,
                row.player_uid,
                row.starts,
                row.substitute_appearances,
                row.minutes,
                row.statistics_json,
                SCORED_ROW_MARKER,
            ])
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
    use rusqlite::{params, Connection};

    /// Gives `snapshot_id` a one-player cohort with every statistic null and `as_of` as its
    /// as-of date. The player must belong to the snapshot.
    pub(crate) fn seed_cohort(conn: &Connection, snapshot_id: i64, player_uid: i64, as_of: &str) {
        let statistics = crate::features::moneyball::MONEYBALL_STATISTIC_KEYS
            .iter()
            .map(|key| ((*key).to_string(), serde_json::Value::Null))
            .collect::<serde_json::Map<_, _>>();
        conn.execute(
            "INSERT INTO player_moneyball_stats (snapshot_id, player_uid, statistics_json)
             VALUES (?1, ?2, ?3)",
            params![
                snapshot_id,
                player_uid,
                serde_json::to_string(&statistics).expect("serialize statistics")
            ],
        )
        .expect("seed Moneyball row");
        conn.execute(
            "UPDATE snapshots SET moneyball_stats_game_date = ?1 WHERE id = ?2",
            params![as_of, snapshot_id],
        )
        .expect("seed as-of date");
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
    use crate::features::moneyball::MONEYBALL_STATISTIC_KEYS;
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

    type StoredRow = (i64, i64, i64, i64, String, String);

    /// The snapshot's stored cohort rows by player, and its as-of date.
    fn stored(conn: &Connection, snapshot_id: i64) -> (Vec<StoredRow>, Option<String>) {
        let rows = conn
            .prepare(
                "SELECT player_uid, starts, substitute_appearances, minutes, statistics_json,
                    percentiles_json
                 FROM player_moneyball_stats WHERE snapshot_id = ?1 ORDER BY player_uid",
            )
            .expect("prepare stored rows")
            .query_map([snapshot_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
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
                (
                    row.player_uid,
                    row.starts,
                    row.substitute_appearances,
                    row.minutes,
                    serialize_moneyball_statistics(&row.statistics).expect("statistics"),
                    "{}".to_string(),
                )
            })
            .collect::<Vec<_>>();
        expected.sort();
        assert_eq!(
            stored(&conn, snapshot_id),
            (expected, Some("2031-05-01".to_string()))
        );
    }
}
