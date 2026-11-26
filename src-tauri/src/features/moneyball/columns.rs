//! The `player_moneyball_stats` columns that hold the catalog statistics.
//!
//! Each of the 138 statistics has a column of its own, named by its catalog key with `-`
//! written as `_`. Counts are INTEGER columns and every other statistic a REAL one, so SQLite
//! sorts and filters a statistic without parsing anything and a query reads only the
//! statistics it needs. Migration 66 checks in the column inventory. A catalog change needs a
//! migration of its own, and `the_table_has_one_typed_column_for_each_statistic` fails until
//! it has one.
//!
//! SQL identifiers come only from the catalog, never from a request.

use std::sync::OnceLock;

use rusqlite::Row;

use super::comparison::statistic_index;
use super::MONEYBALL_STATISTIC_KEYS;

/// The column that holds the catalog statistic at `index`.
pub(crate) fn statistic_column(index: usize) -> &'static str {
    static COLUMNS: OnceLock<Vec<String>> = OnceLock::new();
    &COLUMNS.get_or_init(|| {
        MONEYBALL_STATISTIC_KEYS
            .iter()
            .map(|key| key.replace('-', "_"))
            .collect()
    })[index]
}

/// The column that holds the statistic `key`. `None` for a key outside the catalog.
pub(crate) fn statistic_column_of(key: &str) -> Option<&'static str> {
    statistic_index(key).map(statistic_column)
}

/// The statistics one query reads from a statistics row, as catalog indexes in ascending
/// order.
pub(crate) struct StatisticColumns(Vec<usize>);

impl StatisticColumns {
    /// Every statistic.
    pub(crate) fn all() -> Self {
        Self((0..MONEYBALL_STATISTIC_KEYS.len()).collect())
    }

    /// The statistics named by `keys`, each once. An error for a key outside the catalog.
    pub(crate) fn of_keys<'a>(keys: impl IntoIterator<Item = &'a str>) -> Result<Self, String> {
        let mut indexes = keys
            .into_iter()
            .map(|key| {
                statistic_index(key).ok_or_else(|| format!("unknown Moneyball statistic: {key}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        indexes.sort_unstable();
        indexes.dedup();
        Ok(Self(indexes))
    }

    /// How many result columns the statistics take.
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    /// The statistics' columns of the row aliased `alias`, each after a comma, to end a
    /// select list with.
    pub(crate) fn appended_sql(&self, alias: &str) -> String {
        self.0
            .iter()
            .map(|index| format!(", {alias}.{}", statistic_column(*index)))
            .collect()
    }

    /// Reads the statistics from the result columns that start at `first`, into values in
    /// `MONEYBALL_STATISTIC_KEYS` order. A statistic that is NULL or that the query does not
    /// read is NaN.
    pub(crate) fn read(&self, row: &Row<'_>, first: usize) -> rusqlite::Result<Box<[f64]>> {
        let mut values = vec![f64::NAN; MONEYBALL_STATISTIC_KEYS.len()].into_boxed_slice();
        for (offset, index) in self.0.iter().enumerate() {
            if let Some(value) = row.get::<_, Option<f64>>(first + offset)? {
                values[*index] = value;
            }
        }
        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::*;
    use crate::db::migrations;
    use crate::features::moneyball::persistence::test_support::insert_statistics_row;

    fn database() -> Connection {
        let conn = Connection::open_in_memory().expect("open db");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    #[test]
    fn the_table_has_one_typed_column_for_each_statistic() {
        let conn = database();
        let columns = conn
            .prepare("SELECT name, type FROM pragma_table_info('player_moneyball_stats')")
            .expect("prepare table info")
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("read table info")
            .collect::<Result<Vec<(String, String)>, _>>()
            .expect("collect columns");

        let statistics = columns
            .iter()
            .skip_while(|(name, _)| name != statistic_column(0))
            .collect::<Vec<_>>();
        let expected = (0..MONEYBALL_STATISTIC_KEYS.len())
            .map(statistic_column)
            .collect::<Vec<_>>();
        assert_eq!(
            statistics
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        for (name, declared) in statistics {
            // Queries name these columns without quoting them.
            assert!(
                name.bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
                "{name} is not a plain identifier"
            );
            assert!(
                matches!(declared.as_str(), "INTEGER" | "REAL"),
                "{name} is declared {declared}"
            );
        }
    }

    #[test]
    fn a_query_reads_the_statistics_it_names_and_leaves_the_others_absent() {
        let conn = database();
        conn.pragma_update(None, "foreign_keys", false)
            .expect("skip the player rows");
        insert_statistics_row(
            &conn,
            1,
            77,
            Some(900),
            r#"{"goals": 7, "np-xg_per_90": 0.30000000000000004, "xg-op": -1.5}"#,
            Some("{}"),
        );
        let named = StatisticColumns::of_keys(["np-xg_per_90", "goals", "assists", "goals"])
            .expect("catalog keys");
        assert_eq!(named.len(), 3);

        for (columns, reads_xg_op) in [(named, false), (StatisticColumns::all(), true)] {
            let values = conn
                .query_row(
                    &format!(
                        "SELECT moneyball.player_uid{} FROM player_moneyball_stats moneyball",
                        columns.appended_sql("moneyball")
                    ),
                    [],
                    |row| columns.read(row, 1),
                )
                .expect("read statistics");

            let value = |key: &str| values[statistic_index(key).expect("catalog key")];
            assert_eq!(value("goals"), 7.0);
            assert_eq!(value("np-xg_per_90"), 0.30000000000000004);
            assert!(value("assists").is_nan(), "a NULL statistic is absent");
            assert_eq!(value("xg-op").is_nan(), !reads_xg_op);
            assert_eq!(
                values.iter().filter(|value| !value.is_nan()).count(),
                if reads_xg_op { 3 } else { 2 }
            );
        }
    }

    #[test]
    fn a_key_outside_the_catalog_has_no_column() {
        assert_eq!(statistic_column_of("np-xg"), Some("np_xg"));
        assert_eq!(statistic_column_of("goals; DROP TABLE players"), None);
        assert!(StatisticColumns::of_keys(["goals", "own_goals"]).is_err());
    }
}
