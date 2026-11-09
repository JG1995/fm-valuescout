//! Writes snapshot-owned Moneyball cohorts: the 138-key statistics and percentile JSON, whole
//! cohort replacement with its as-of date, and current-only retention.

use rusqlite::{params, Transaction};
use serde_json::{Map, Number, Value};

use super::percentile::{numeric_statistics, rank_column, MoneyballPercentiles};
use super::{MoneyballMetricValue, MoneyballStatistics, MONEYBALL_STATISTIC_KEYS};

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

pub(crate) fn serialize_moneyball_percentiles(
    percentiles: &MoneyballPercentiles,
) -> Result<String, MoneyballPersistenceError> {
    let serialized = percentiles
        .iter()
        .map(|(key, value)| {
            (
                key.clone(),
                value.map_or(Value::Null, |value| Value::Number(Number::from(value))),
            )
        })
        .collect();

    serde_json::to_string(&Value::Object(serialized))
        .map_err(|_| MoneyballPersistenceError::InvalidStatistics)
}

/// A replacement cohort whose statistics and percentile JSON are already built, so writing it
/// only replaces rows.
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
    percentiles_json: String,
}

#[cfg(test)]
thread_local! {
    /// Runs once a preparation has finished, so a test can observe what was held meanwhile.
    pub(crate) static PREPARED_HOOK: std::cell::RefCell<Option<Box<dyn FnMut()>>> =
        std::cell::RefCell::new(None);
}

/// Scores `rows`' full-cohort percentiles and serializes each row's statistics and
/// percentiles. It needs no database, so callers run it before taking the writer.
pub(crate) fn prepare_snapshot_cohort(
    rows: &[CohortRow],
) -> Result<PreparedCohort, MoneyballPersistenceError> {
    let mut values: Vec<Vec<Option<f64>>> = MONEYBALL_STATISTIC_KEYS
        .iter()
        .map(|_| Vec::with_capacity(rows.len()))
        .collect();
    for row in rows {
        let statistics = numeric_statistics(&row.statistics);
        for (column, key) in values.iter_mut().zip(MONEYBALL_STATISTIC_KEYS) {
            column.push(statistics.get(key).copied().flatten());
        }
    }
    let columns = MONEYBALL_STATISTIC_KEYS
        .iter()
        .zip(values)
        .map(|(key, values)| rank_column(&values, key))
        .collect::<Vec<_>>();

    let rows = rows
        .iter()
        .enumerate()
        .map(|(position, row)| {
            let percentiles: MoneyballPercentiles = MONEYBALL_STATISTIC_KEYS
                .iter()
                .zip(&columns)
                .map(|(key, column)| ((*key).to_string(), column[position]))
                .collect();
            Ok(PreparedCohortRow {
                player_uid: row.player_uid,
                starts: row.starts,
                substitute_appearances: row.substitute_appearances,
                minutes: row.minutes,
                statistics_json: serialize_moneyball_statistics(&row.statistics)?,
                percentiles_json: serialize_moneyball_percentiles(&percentiles)?,
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
                row.percentiles_json,
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

/// Deletes Moneyball rows and the as-of date from every non-current snapshot of one save,
/// so only the current snapshot keeps a cohort. Call it in the transaction that changes the
/// current snapshot.
pub(crate) fn clear_non_current_snapshots(
    tx: &Transaction<'_>,
    save_id: i64,
) -> Result<(), String> {
    tx.execute(
        "DELETE FROM player_moneyball_stats
         WHERE snapshot_id IN (
             SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 0
         )",
        [save_id],
    )
    .map_err(|error| error.to_string())?;
    tx.execute(
        "UPDATE snapshots SET moneyball_stats_game_date = NULL
         WHERE save_id = ?1 AND is_current = 0 AND moneyball_stats_game_date IS NOT NULL",
        [save_id],
    )
    .map_err(|error| error.to_string())?;
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
    use crate::features::moneyball::percentile::calculate_percentiles;
    use crate::features::snapshot::service as snapshot_service;

    /// The cohort replacement as it was before preparation moved out of the transaction,
    /// kept as the oracle for the stored JSON.
    fn previous_replace_snapshot_cohort(
        tx: &Transaction<'_>,
        snapshot_id: i64,
        rows: &[CohortRow],
        stats_game_date: &str,
    ) -> Result<(), MoneyballPersistenceError> {
        tx.execute(
            "DELETE FROM player_moneyball_stats WHERE snapshot_id = ?1",
            [snapshot_id],
        )
        .map_err(|_| MoneyballPersistenceError::Database)?;

        let percentiles = calculate_percentiles(
            &rows
                .iter()
                .map(|row| (row.player_uid, numeric_statistics(&row.statistics)))
                .collect(),
        );
        let mut insert = tx
            .prepare(
                "INSERT INTO player_moneyball_stats (
                    snapshot_id, player_uid, starts, substitute_appearances, minutes,
                    statistics_json, percentiles_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )
            .map_err(|_| MoneyballPersistenceError::Database)?;
        for row in rows {
            let player_percentiles = percentiles
                .get(&row.player_uid)
                .ok_or(MoneyballPersistenceError::InvalidStatistics)?;
            insert
                .execute(params![
                    snapshot_id,
                    row.player_uid,
                    row.starts,
                    row.substitute_appearances,
                    row.minutes,
                    serialize_moneyball_statistics(&row.statistics)?,
                    serialize_moneyball_percentiles(player_percentiles)?,
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
    fn a_prepared_cohort_stores_the_same_json_as_the_previous_replacement() {
        let rows = fixed_cohort();
        let (mut previous, previous_snapshot) = database(&rows);
        let tx = previous.transaction().expect("begin previous write");
        previous_replace_snapshot_cohort(&tx, previous_snapshot, &rows, "2031-05-01")
            .expect("previous replacement");
        tx.commit().expect("commit previous write");
        let (mut prepared, prepared_snapshot) = database(&rows);
        let cohort = prepare_snapshot_cohort(&rows).expect("prepare cohort");
        let tx = prepared.transaction().expect("begin prepared write");
        replace_snapshot_cohort(&tx, prepared_snapshot, &cohort, "2031-05-01")
            .expect("prepared replacement");
        tx.commit().expect("commit prepared write");

        let expected = stored(&previous, previous_snapshot);
        assert_eq!(expected.0.len(), rows.len());
        assert_eq!(stored(&prepared, prepared_snapshot), expected);
    }
}
