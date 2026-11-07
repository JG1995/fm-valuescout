//! Writes snapshot-owned Moneyball cohorts: the 138-key statistics and percentile JSON, whole
//! cohort replacement with its as-of date, and current-only retention.

use rusqlite::{params, Transaction};
use serde_json::{Map, Number, Value};

use super::percentile::{calculate_percentiles, numeric_statistics, MoneyballPercentiles};
use super::{MoneyballMetricValue, MoneyballStatistics};

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

/// Replaces the snapshot's whole cohort with `rows`, scores full-cohort percentiles, and
/// records the save's in-game date as the cohort's as-of date. The caller owns the
/// transaction, so any failure leaves the prior cohort and date in place.
pub(crate) fn replace_snapshot_cohort(
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
