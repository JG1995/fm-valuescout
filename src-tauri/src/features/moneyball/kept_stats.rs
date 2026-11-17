//! Season statistics borrowed from kept snapshots.
//!
//! A player with under `MIN_SEASON_MINUTES` minutes in the current snapshot is scored on his
//! statistics from the latest kept snapshot of the save in which he has at least that many,
//! provided they are less than a year older than the current snapshot's in-game date. Those
//! statistics rank within that snapshot's comparison groups, and its league strengths move
//! his scores, because both describe the season the statistics were played in.

use rusqlite::Connection;

use super::comparison::MIN_SEASON_MINUTES;

/// A kept snapshot whose season statistics can stand in for the current snapshot's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KeptStats {
    pub(crate) snapshot_id: i64,
    /// The in-game date of its statistics.
    pub(crate) stats_date: String,
}

/// The kept snapshots whose statistics can stand in for `snapshot_id`'s, latest first: the
/// save's other kept snapshots with statistics dated less than a year before its in-game date,
/// and not after it. Empty when it has no valid in-game date.
pub(crate) fn kept_stats_snapshots(
    conn: &Connection,
    snapshot_id: i64,
) -> Result<Vec<KeptStats>, String> {
    conn.prepare(
        "SELECT kept.id, kept.moneyball_stats_game_date
         FROM snapshots current_snapshot
         INNER JOIN snapshots kept
            ON kept.save_id = current_snapshot.save_id AND kept.id <> current_snapshot.id
         WHERE current_snapshot.id = ?1
           AND kept.is_kept = 1
           AND kept.moneyball_stats_game_date > date(current_snapshot.game_date, '-1 year')
           AND kept.moneyball_stats_game_date <= date(current_snapshot.game_date)
         ORDER BY kept.moneyball_stats_game_date DESC, kept.id DESC",
    )
    .map_err(|error| error.to_string())?
    .query_map([snapshot_id], |row| {
        Ok(KeptStats {
            snapshot_id: row.get(0)?,
            stats_date: row.get(1)?,
        })
    })
    .map_err(|error| error.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|error| error.to_string())
}

/// SQL for the snapshot whose statistics score the `players` row aliased `players`: its own
/// snapshot when he has `MIN_SEASON_MINUTES` minutes there, else the first of the kept
/// snapshots (`kept_stats_snapshots`, latest first) in which he has, else his own again.
pub(crate) fn stats_snapshot_sql(players: &str, kept_snapshot_ids: &[i64]) -> String {
    if kept_snapshot_ids.is_empty() {
        return format!("{players}.snapshot_id");
    }
    let borrowed = kept_snapshot_ids
        .iter()
        .map(|snapshot_id| {
            format!(
                "(SELECT kept.snapshot_id FROM player_moneyball_stats kept
                  WHERE kept.snapshot_id = {snapshot_id} AND kept.player_uid = {players}.uid
                    AND kept.minutes >= {MIN_SEASON_MINUTES}
                    AND kept.percentiles_json IS NOT NULL)"
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "CASE WHEN (SELECT own.minutes FROM player_moneyball_stats own
                    WHERE own.snapshot_id = {players}.snapshot_id
                      AND own.player_uid = {players}.uid) >= {MIN_SEASON_MINUTES}
              THEN {players}.snapshot_id
              ELSE COALESCE({borrowed}, {players}.snapshot_id) END"
    )
}

/// A `LEFT JOIN` of the statistics row, aliased `moneyball`, that scores the `players` row.
pub(crate) fn stats_join_sql(players: &str, kept_snapshot_ids: &[i64]) -> String {
    format!(
        " LEFT JOIN player_moneyball_stats moneyball
             ON moneyball.snapshot_id = {} AND moneyball.player_uid = {players}.uid",
        stats_snapshot_sql(players, kept_snapshot_ids)
    )
}

#[cfg(test)]
pub(crate) mod test_support {
    use rusqlite::{params, Connection};

    /// Adds a kept snapshot of `snapshot_id`'s save, dated `game_date`, holding one player
    /// row for each of `uids` copied from `snapshot_id`. Returns its ID.
    pub(crate) fn add_kept_snapshot(
        conn: &Connection,
        snapshot_id: i64,
        game_date: &str,
        uids: &[i64],
    ) -> i64 {
        let columns = |table: &str| {
            conn.prepare(&format!("SELECT name FROM pragma_table_info('{table}')"))
                .expect("prepare table info")
                .query_map([], |row| row.get::<_, String>(0))
                .expect("read table info")
                .collect::<Result<Vec<_>, _>>()
                .expect("collect columns")
        };
        let copied = |table: &str, skip: &[&str]| {
            columns(table)
                .into_iter()
                .filter(|column| !skip.contains(&column.as_str()))
                .collect::<Vec<_>>()
                .join(", ")
        };
        // The new snapshot gets its own token from a trigger and no season stats yet.
        let snapshot_columns = copied(
            "snapshots",
            &[
                "id",
                "is_current",
                "is_kept",
                "game_date",
                "context_token",
                "moneyball_stats_game_date",
            ],
        );
        conn.execute(
            &format!(
                "INSERT INTO snapshots ({snapshot_columns}, is_current, is_kept, game_date)
                 SELECT {snapshot_columns}, 0, 1, ?2
                 FROM snapshots WHERE id = ?1"
            ),
            params![snapshot_id, game_date],
        )
        .expect("insert kept snapshot");
        let kept_id = conn.last_insert_rowid();
        let player_columns = copied("players", &["snapshot_id"]);
        for uid in uids {
            conn.execute(
                &format!(
                    "INSERT INTO players (snapshot_id, {player_columns})
                     SELECT ?1, {player_columns} FROM players WHERE snapshot_id = ?2 AND uid = ?3"
                ),
                params![kept_id, snapshot_id, uid],
            )
            .expect("copy player row");
        }
        kept_id
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use super::test_support::add_kept_snapshot;
    use super::*;
    use crate::db::migrations;
    use crate::features::snapshot::service as snapshot_service;

    /// A database whose current snapshot is dated 2031-08-14 and holds players 1 to 4.
    fn database() -> (Connection, i64) {
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
                supported_game_version, bridge_version, protocol_version, game_date,
                game_date_source, scan_truncated, max_accepted, player_count
             ) VALUES (?1, 1, 6, '2031-08-14T00:00:00Z', '27.0', '27.0', 'test', 1,
                '2031-08-14', 'memory', 0, NULL, 4)",
            [save.id],
        )
        .expect("insert current snapshot");
        let snapshot_id = conn.last_insert_rowid();
        for uid in 1..=4 {
            conn.execute(
                "INSERT INTO players (
                    snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                    nationalities_json, preferred_foot, positions_json, attributes_json,
                    hidden_attributes_json, personality_json
                 ) VALUES (?1, ?2, 100, 100, 'Player', 2000, 1, '[]', 'Right', '{}',
                    '{}', '{}', '{}')",
                params![snapshot_id, uid],
            )
            .expect("insert player");
        }
        (conn, snapshot_id)
    }

    fn set_stats_date(conn: &Connection, snapshot_id: i64, stats_date: Option<&str>) {
        conn.execute(
            "UPDATE snapshots SET moneyball_stats_game_date = ?1 WHERE id = ?2",
            params![stats_date, snapshot_id],
        )
        .expect("set stats date");
    }

    fn add_stats(conn: &Connection, snapshot_id: i64, uid: i64, minutes: i64) {
        conn.execute(
            "INSERT INTO player_moneyball_stats (
                 snapshot_id, player_uid, minutes, statistics_json, percentiles_json
             ) VALUES (?1, ?2, ?3, '{}', '{}')",
            params![snapshot_id, uid, minutes],
        )
        .expect("insert stats row");
    }

    fn kept_ids(conn: &Connection, snapshot_id: i64) -> Vec<i64> {
        kept_stats_snapshots(conn, snapshot_id)
            .expect("kept stats snapshots")
            .into_iter()
            .map(|kept| kept.snapshot_id)
            .collect()
    }

    #[test]
    fn kept_snapshots_with_stats_under_a_year_old_stand_in_latest_first() {
        let (conn, current) = database();
        let older = add_kept_snapshot(&conn, current, "2031-01-10", &[1]);
        set_stats_date(&conn, older, Some("2031-01-10"));
        let latest = add_kept_snapshot(&conn, current, "2031-05-20", &[1]);
        set_stats_date(&conn, latest, Some("2031-05-20"));
        let year_old = add_kept_snapshot(&conn, current, "2030-08-14", &[1]);
        set_stats_date(&conn, year_old, Some("2030-08-14"));
        add_kept_snapshot(&conn, current, "2031-06-01", &[1]);
        let not_kept = add_kept_snapshot(&conn, current, "2031-06-02", &[1]);
        set_stats_date(&conn, not_kept, Some("2031-06-02"));
        conn.execute("UPDATE snapshots SET is_kept = 0 WHERE id = ?1", [not_kept])
            .expect("unkeep snapshot");

        assert_eq!(kept_ids(&conn, current), [latest, older]);
        assert_eq!(
            kept_stats_snapshots(&conn, current).expect("kept stats")[0].stats_date,
            "2031-05-20"
        );
    }

    #[test]
    fn a_current_snapshot_without_a_valid_date_borrows_nothing() {
        let (conn, current) = database();
        let kept = add_kept_snapshot(&conn, current, "2031-05-20", &[1]);
        set_stats_date(&conn, kept, Some("2031-05-20"));
        conn.execute(
            "UPDATE snapshots SET game_date = NULL WHERE id = ?1",
            [current],
        )
        .expect("clear current date");

        assert_eq!(kept_ids(&conn, current), Vec::<i64>::new());
    }

    #[test]
    fn a_short_sample_borrows_from_the_latest_kept_snapshot_where_he_has_the_minutes() {
        let (conn, current) = database();
        let older = add_kept_snapshot(&conn, current, "2031-01-10", &[1, 2, 3, 4]);
        set_stats_date(&conn, older, Some("2031-01-10"));
        let latest = add_kept_snapshot(&conn, current, "2031-05-20", &[1, 2, 3, 4]);
        set_stats_date(&conn, latest, Some("2031-05-20"));
        // 1 has enough minutes now; 2 borrows the latest; 3 is short there too, so borrows
        // the older; 4 has nothing to borrow and keeps his short sample.
        add_stats(&conn, current, 1, 900);
        add_stats(&conn, current, 2, 100);
        add_stats(&conn, current, 4, 100);
        for (snapshot_id, uid, minutes) in [
            (latest, 1, 2000),
            (latest, 2, 1500),
            (latest, 3, 200),
            (older, 3, 800),
        ] {
            add_stats(&conn, snapshot_id, uid, minutes);
        }
        let kept = kept_ids(&conn, current);
        let sources = conn
            .prepare(&format!(
                "SELECT players.uid, moneyball.snapshot_id, moneyball.minutes
                 FROM players{}
                 WHERE players.snapshot_id = ?1
                 ORDER BY players.uid",
                stats_join_sql("players", &kept)
            ))
            .expect("prepare sources")
            .query_map([current], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .expect("query sources")
            .collect::<Result<Vec<_>, _>>()
            .expect("read sources");

        assert_eq!(
            sources,
            [
                (1, current, 900),
                (2, latest, 1500),
                (3, older, 800),
                (4, current, 100),
            ]
        );
    }
}
