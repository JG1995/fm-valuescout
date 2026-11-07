//! Attaches the season statistics of a save's linked FM save file to that save's current
//! snapshot. Load Data runs it after publication; it never holds the database lock while
//! the file is read.

use std::collections::HashSet;
use std::path::Path;

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::db::Db;
use crate::features::csv_import::service::{
    capture_import_context, revalidate_active_import_context, CsvImportServiceError, ImportContext,
};
use crate::features::save_stats::{self, SaveSeasonStats};

use super::persistence::{replace_snapshot_cohort, CohortRow};
use super::save_statistics::cohort_row;

/// What one stats read did, and how long it took.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SeasonStatsOutcome {
    #[serde(flatten)]
    pub result: SeasonStatsResult,
    pub elapsed_ms: u64,
}

/// Every variant except `Attached` leaves the stored cohort unchanged. Nothing here carries
/// the linked file's path; `Failed` names the file only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub(crate) enum SeasonStatsResult {
    /// The cohort was replaced. Unmatched players have a Unique ID and competitive minutes
    /// but are not in the snapshot.
    Attached {
        matched_players: usize,
        unmatched_players: usize,
        game_date: String,
    },
    NoMatches {
        unmatched_players: usize,
        game_date: String,
    },
    NotLinked,
    NoSnapshot,
    /// The active save, its current snapshot, or the save's link changed, so the result was
    /// discarded.
    StaleContext,
    /// `kind` is a save-file error kind, or `database` for a failed write.
    Failed {
        kind: &'static str,
        file_name: Option<String>,
    },
}

/// The context a read is bound to, captured under one brief lock.
struct CapturedContext {
    import: ImportContext,
    linked_path: String,
}

/// Reads the linked file of save `save_id` and replaces its current snapshot's cohort when
/// that save, identified by `context_token`, is still the active one.
pub(crate) fn attach_season_stats(
    db: &Db,
    save_id: i64,
    context_token: &str,
    now_ms: &mut dyn FnMut() -> u64,
) -> SeasonStatsOutcome {
    let started = now_ms();
    let result = attach(db, save_id, context_token);
    SeasonStatsOutcome {
        result,
        elapsed_ms: now_ms().saturating_sub(started),
    }
}

fn attach(db: &Db, save_id: i64, context_token: &str) -> SeasonStatsResult {
    let captured = {
        let Ok(conn) = db.0.lock() else {
            return database_failure();
        };
        match capture(&conn, save_id, context_token) {
            Ok(captured) => captured,
            Err(result) => return result,
        }
    };

    let stats = match save_stats::read_season_stats(Path::new(&captured.linked_path)) {
        Ok(stats) => stats,
        Err(error) => {
            return SeasonStatsResult::Failed {
                kind: error.kind(),
                file_name: file_name(&captured.linked_path),
            }
        }
    };
    let (rows, unmatched_players) = match_cohort(&stats, captured.import.player_uids());
    let game_date = stats.info.game_date;
    if rows.is_empty() {
        return SeasonStatsResult::NoMatches {
            unmatched_players,
            game_date,
        };
    }

    let Ok(mut conn) = db.0.lock() else {
        return database_failure();
    };
    match persist(&mut conn, &captured, &rows, &game_date) {
        Ok(()) => SeasonStatsResult::Attached {
            matched_players: rows.len(),
            unmatched_players,
            game_date,
        },
        Err(result) => result,
    }
}

fn capture(
    conn: &Connection,
    save_id: i64,
    context_token: &str,
) -> Result<CapturedContext, SeasonStatsResult> {
    let import = match capture_import_context(conn) {
        Ok(import) => import,
        Err(CsvImportServiceError::NoCurrentSnapshot) => {
            return Err(if is_active_save(conn, save_id, context_token)? {
                SeasonStatsResult::NoSnapshot
            } else {
                SeasonStatsResult::StaleContext
            });
        }
        Err(_) => return Err(database_failure()),
    };
    let active = import.active();
    if active.save_id != save_id || active.save_context_token() != context_token {
        return Err(SeasonStatsResult::StaleContext);
    }
    let linked_path: Option<String> = conn
        .query_row(
            "SELECT linked_save_file_path FROM saves WHERE id = ?1",
            [save_id],
            |row| row.get(0),
        )
        .map_err(|_| database_failure())?;
    let linked_path = linked_path.ok_or(SeasonStatsResult::NotLinked)?;
    Ok(CapturedContext {
        import,
        linked_path,
    })
}

fn is_active_save(
    conn: &Connection,
    save_id: i64,
    context_token: &str,
) -> Result<bool, SeasonStatsResult> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM saves WHERE id = ?1 AND context_token = ?2 AND is_active = 1)",
        params![save_id, context_token],
        |row| row.get(0),
    )
    .map_err(|_| database_failure())
}

/// The cohort rows of the save players found in the snapshot, and how many were not found.
fn match_cohort(stats: &SaveSeasonStats, player_uids: &HashSet<i64>) -> (Vec<CohortRow>, usize) {
    let rows: Vec<CohortRow> = stats
        .players
        .iter()
        .filter(|player| player_uids.contains(&i64::from(player.unique_id)))
        .map(cohort_row)
        .collect();
    let unmatched_players = stats.players.len() - rows.len();
    (rows, unmatched_players)
}

/// Writes the cohort only if the captured save, snapshot, and link are all still current.
fn persist(
    conn: &mut Connection,
    captured: &CapturedContext,
    rows: &[CohortRow],
    game_date: &str,
) -> Result<(), SeasonStatsResult> {
    let tx = conn.transaction().map_err(|_| database_failure())?;
    let active = captured.import.active();
    match revalidate_active_import_context(&tx, active) {
        Ok(()) => {}
        Err(CsvImportServiceError::StaleContext) => return Err(SeasonStatsResult::StaleContext),
        Err(_) => return Err(database_failure()),
    }
    // Changing a link keeps the save's context token, so the link is checked on its own.
    let same_link: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM saves WHERE id = ?1 AND linked_save_file_path = ?2)",
            params![active.save_id, captured.linked_path],
            |row| row.get(0),
        )
        .map_err(|_| database_failure())?;
    if !same_link {
        return Err(SeasonStatsResult::StaleContext);
    }
    replace_snapshot_cohort(&tx, active.snapshot_id, rows, game_date)
        .map_err(|_| database_failure())?;
    tx.commit().map_err(|_| database_failure())
}

fn database_failure() -> SeasonStatsResult {
    SeasonStatsResult::Failed {
        kind: "database",
        file_name: None,
    }
}

fn file_name(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use rusqlite::params;

    use super::*;
    use crate::db::migrations;
    use crate::features::moneyball::persistence::test_support::{cohort_state, seed_cohort};
    use crate::features::save_stats::synthetic::{write_season_stats_save, GAME_DATE};
    use crate::features::snapshot::service::{self as snapshot_service, LinkedSaveFile};

    const PRIOR_AS_OF: &str = "2030-01-01";

    /// A change's name and a write that makes a captured read stale.
    type DatabaseChange = (&'static str, fn(&Connection, &Fixture));

    struct Fixture {
        db: Db,
        save_id: i64,
        save_token: String,
        snapshot_id: i64,
        directory: tempfile::TempDir,
    }

    /// An active save whose current snapshot holds players 10, 20, and 30, with a prior
    /// one-player cohort on player 30.
    fn fixture() -> Fixture {
        let directory = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(directory.path().join("attach.db")).expect("open db");
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
                0, NULL, 3)",
            [save.id],
        )
        .expect("insert snapshot");
        let snapshot_id = conn.last_insert_rowid();
        for uid in [10, 20, 30] {
            conn.execute(
                "INSERT INTO players (
                    snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                    nationalities_json, preferred_foot, positions_json, attributes_json,
                    hidden_attributes_json, personality_json
                 ) VALUES (?1, ?2, 100, 100, 'Attach player', 2000, 1, '[]', 'Right', '{}',
                    '{}', '{}', '{}')",
                params![snapshot_id, uid],
            )
            .expect("insert player");
        }
        seed_cohort(&conn, snapshot_id, 30, PRIOR_AS_OF);
        Fixture {
            db: Db(Mutex::new(conn)),
            save_id: save.id,
            save_token: save.context_token,
            snapshot_id,
            directory,
        }
    }

    impl Fixture {
        /// Writes a save in which players 10 and 20 and the unknown player 99 played, links
        /// it through the real link validation, and returns its path.
        fn link_matching_save(&self) -> String {
            let path = write_season_stats_save(
                self.directory.path(),
                &[
                    (10, &[("minutes", 90.0), ("starts", 1.0), ("goals", 1.0)]),
                    (20, &[("minutes", 90.0), ("starts", 1.0), ("goals", 3.0)]),
                    (99, &[("minutes", 45.0), ("substitute_appearances", 1.0)]),
                ],
            );
            let path = path.to_str().expect("UTF-8 path").to_string();
            self.link(&path);
            path
        }

        fn link(&self, path: &str) {
            let file = LinkedSaveFile::validate(path).expect("valid FM27 save");
            let conn = self.db.0.lock().expect("lock");
            snapshot_service::set_save_file_link(
                &conn,
                self.save_id,
                &self.save_token,
                Some(&file),
            )
            .expect("link save file");
        }

        fn link_without_validation(&self, path: &str) {
            let conn = self.db.0.lock().expect("lock");
            conn.execute(
                "UPDATE saves SET linked_save_file_path = ?1 WHERE id = ?2",
                params![path, self.save_id],
            )
            .expect("store link");
        }

        fn attach(&self) -> SeasonStatsResult {
            attach_season_stats(&self.db, self.save_id, &self.save_token, &mut || 0).result
        }

        fn cohort(&self) -> Vec<(i64, Option<i64>, Option<i64>)> {
            let conn = self.db.0.lock().expect("lock");
            let rows = conn
                .prepare(
                    "SELECT player_uid, json_extract(statistics_json, '$.goals'),
                        json_extract(percentiles_json, '$.goals')
                 FROM player_moneyball_stats WHERE snapshot_id = ?1 ORDER BY player_uid",
                )
                .expect("prepare cohort query")
                .query_map([self.snapshot_id], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                })
                .expect("query cohort")
                .collect::<Result<_, _>>()
                .expect("read cohort");
            rows
        }

        fn state(&self) -> (i64, Option<String>) {
            cohort_state(&self.db.0.lock().expect("lock"), self.snapshot_id)
        }

        /// Captures and reads as the attach does, so a test can change the database before
        /// the write.
        fn capture_and_read(&self) -> (CapturedContext, Vec<CohortRow>) {
            let captured = capture(
                &self.db.0.lock().expect("lock"),
                self.save_id,
                &self.save_token,
            )
            .unwrap_or_else(|result| panic!("capture failed: {result:?}"));
            let stats =
                save_stats::read_season_stats(Path::new(&captured.linked_path)).expect("read save");
            let (rows, _) = match_cohort(&stats, captured.import.player_uids());
            (captured, rows)
        }

        fn persist(&self, captured: &CapturedContext, rows: &[CohortRow]) -> SeasonStatsResult {
            let mut conn = self.db.0.lock().expect("lock");
            persist(&mut conn, captured, rows, GAME_DATE).expect_err("write is rejected")
        }
    }

    #[test]
    fn a_linked_save_replaces_the_cohort_scores_percentiles_and_sets_the_as_of_date() {
        let fixture = fixture();
        fixture.link_matching_save();

        assert_eq!(
            fixture.attach(),
            SeasonStatsResult::Attached {
                matched_players: 2,
                unmatched_players: 1,
                game_date: GAME_DATE.to_string(),
            }
        );
        assert_eq!(
            fixture.cohort(),
            vec![(10, Some(1), Some(0)), (20, Some(3), Some(100))]
        );
        assert_eq!(fixture.state(), (2, Some(GAME_DATE.to_string())));
    }

    #[test]
    fn a_save_with_no_matching_player_writes_nothing() {
        let fixture = fixture();
        let path = write_season_stats_save(
            fixture.directory.path(),
            &[(99, &[("minutes", 90.0), ("starts", 1.0)])],
        );
        fixture.link(path.to_str().expect("UTF-8 path"));

        assert_eq!(
            fixture.attach(),
            SeasonStatsResult::NoMatches {
                unmatched_players: 1,
                game_date: GAME_DATE.to_string(),
            }
        );
        assert_eq!(fixture.state(), (1, Some(PRIOR_AS_OF.to_string())));
    }

    #[test]
    fn an_unlinked_save_reads_nothing() {
        let fixture = fixture();

        assert_eq!(fixture.attach(), SeasonStatsResult::NotLinked);
        assert_eq!(fixture.state(), (1, Some(PRIOR_AS_OF.to_string())));
    }

    #[test]
    fn a_requested_save_that_is_not_active_is_stale_before_any_read() {
        let fixture = fixture();
        // A read of this link would fail as not found, not as stale.
        fixture.link_without_validation("/missing/Career.fm");
        let other = {
            let conn = fixture.db.0.lock().expect("lock");
            snapshot_service::create_save(&conn, "Other save").expect("create other save")
        };

        let stale_token =
            attach_season_stats(&fixture.db, fixture.save_id, "stale-token", &mut || 0).result;
        let inactive =
            attach_season_stats(&fixture.db, other.id, &other.context_token, &mut || 0).result;

        assert_eq!(stale_token, SeasonStatsResult::StaleContext);
        assert_eq!(inactive, SeasonStatsResult::StaleContext);
        assert_eq!(fixture.state(), (1, Some(PRIOR_AS_OF.to_string())));
    }

    #[test]
    fn a_read_whose_snapshot_save_or_link_changed_writes_nothing() {
        let changes: [DatabaseChange; 3] = [
            ("snapshot demoted", |conn, fixture| {
                conn.execute(
                    "UPDATE snapshots SET is_current = 0 WHERE id = ?1",
                    [fixture.snapshot_id],
                )
                .expect("demote snapshot");
            }),
            ("another save activated", |conn, fixture| {
                let other =
                    snapshot_service::create_save(conn, "Other save").expect("create other save");
                conn.execute(
                    "UPDATE saves SET is_active = 0 WHERE id = ?1",
                    [fixture.save_id],
                )
                .expect("deactivate save");
                conn.execute("UPDATE saves SET is_active = 1 WHERE id = ?1", [other.id])
                    .expect("activate other save");
            }),
            ("link changed", |conn, fixture| {
                conn.execute(
                    "UPDATE saves SET linked_save_file_path = '/saves/Other.fm' WHERE id = ?1",
                    [fixture.save_id],
                )
                .expect("change link");
            }),
        ];

        for (case, change) in changes {
            let fixture = fixture();
            fixture.link_matching_save();
            let (captured, rows) = fixture.capture_and_read();
            change(&fixture.db.0.lock().expect("lock"), &fixture);

            assert_eq!(
                fixture.persist(&captured, &rows),
                SeasonStatsResult::StaleContext,
                "{case}"
            );
            assert_eq!(fixture.cohort(), vec![(30, None, None)], "{case}");
            assert_eq!(
                fixture.state(),
                (1, Some(PRIOR_AS_OF.to_string())),
                "{case}"
            );
        }
    }

    #[test]
    fn a_failed_cohort_write_rolls_back_to_the_prior_cohort_and_date() {
        let fixture = fixture();
        fixture.link_matching_save();
        fixture
            .db
            .0
            .lock()
            .expect("lock")
            .execute_batch(
                "CREATE TRIGGER deny_second_cohort_row
                 BEFORE INSERT ON player_moneyball_stats
                 WHEN NEW.player_uid = 20
                 BEGIN SELECT RAISE(ABORT, 'test write failure'); END;",
            )
            .expect("create deny-write trigger");

        assert_eq!(
            fixture.attach(),
            SeasonStatsResult::Failed {
                kind: "database",
                file_name: None,
            }
        );
        assert_eq!(fixture.cohort(), vec![(30, None, None)]);
        assert_eq!(fixture.state(), (1, Some(PRIOR_AS_OF.to_string())));
    }

    #[test]
    fn a_read_failure_names_only_the_linked_file() {
        let fixture = fixture();
        fixture.link_without_validation("/missing/directory/Career Two.fm");

        assert_eq!(
            fixture.attach(),
            SeasonStatsResult::Failed {
                kind: "not_found",
                file_name: Some("Career Two.fm".to_string()),
            }
        );
        assert_eq!(fixture.state(), (1, Some(PRIOR_AS_OF.to_string())));
    }

    #[test]
    fn the_outcome_serializes_its_status_fields_and_elapsed_time() {
        let outcome = SeasonStatsOutcome {
            result: SeasonStatsResult::Attached {
                matched_players: 2,
                unmatched_players: 1,
                game_date: GAME_DATE.to_string(),
            },
            elapsed_ms: 12,
        };

        assert_eq!(
            serde_json::to_value(outcome).expect("serialize outcome"),
            serde_json::json!({
                "status": "attached",
                "matchedPlayers": 2,
                "unmatchedPlayers": 1,
                "gameDate": GAME_DATE,
                "elapsedMs": 12,
            })
        );
    }
}
