use std::collections::BTreeMap;
use std::time::Instant;

use serde::Serialize;
use tauri::State;

use crate::blocking;
use crate::db::Db;
use crate::features::player::boost_gate;
use crate::features::snapshot::service::active_save_id;

use super::catalog::is_save_unavailable_statistic_key;
use super::comparison::{ComparisonGroupsCache, GroupsSource, MIN_SEASON_MINUTES};
use super::league_strength::{self, LeagueAdjustment, LeagueBreakdown};
use super::query::{
    self, BorrowedStats, MoneyballComparison, MoneyballProfile, MoneyballProfileState,
    MoneyballRoleContribution, MoneyballRoleScore,
};
use super::save_stats_import::{attach_season_stats, SeasonStatsOutcome};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyballProfileDto {
    pub state: &'static str,
    pub starts: Option<i64>,
    pub substitute_appearances: Option<i64>,
    pub minutes: Option<i64>,
    pub statistics: Option<BTreeMap<String, Option<f64>>>,
    /// The player's percentiles within each family he can be compared in, his best
    /// position's first.
    pub comparisons: Vec<MoneyballComparisonDto>,
    /// The fewest minutes that make a player one of a family's peers.
    pub minimum_minutes: i64,
    pub role_catalog_version: Option<u32>,
    pub role_scores: Option<Vec<MoneyballRoleScoreDto>>,
    pub league_adjustment: Option<MoneyballLeagueBreakdownDto>,
    /// Set when the player is under `minimum_minutes` this season and the statistics are a
    /// kept snapshot's.
    pub borrowed_stats: Option<BorrowedStatsDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BorrowedStatsDto {
    pub stats_date: String,
    pub current_minutes: i64,
}

impl From<BorrowedStats> for BorrowedStatsDto {
    fn from(borrowed: BorrowedStats) -> Self {
        Self {
            stats_date: borrowed.stats_date,
            current_minutes: borrowed.current_minutes,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyballComparisonDto {
    /// The role catalog's position family ID.
    pub family: &'static str,
    pub peers: usize,
    pub percentiles: BTreeMap<String, Option<u8>>,
}

impl From<MoneyballComparison> for MoneyballComparisonDto {
    fn from(comparison: MoneyballComparison) -> Self {
        Self {
            family: comparison.family.id(),
            peers: comparison.peers,
            percentiles: comparison.percentiles,
        }
    }
}

/// What moved a player's role scores for his league. Strengths are mean current abilities.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyballLeagueBreakdownDto {
    pub division: Option<String>,
    pub strength: f64,
    pub estimated: bool,
    pub reference_division: String,
    pub reference_strength: f64,
    pub shift: i16,
}

impl From<LeagueBreakdown> for MoneyballLeagueBreakdownDto {
    fn from(breakdown: LeagueBreakdown) -> Self {
        Self {
            division: breakdown.division,
            strength: breakdown.strength,
            estimated: breakdown.estimated,
            reference_division: breakdown.reference_division,
            reference_strength: breakdown.reference_strength,
            shift: breakdown.shift,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyballRoleScoreDto {
    pub role_id: String,
    pub display_name: String,
    pub phase: String,
    pub position_family: String,
    pub position_tags: Vec<String>,
    pub score: Option<u8>,
    pub contributions: Vec<MoneyballRoleContributionDto>,
}

impl From<MoneyballRoleScore> for MoneyballRoleScoreDto {
    fn from(role: MoneyballRoleScore) -> Self {
        Self {
            role_id: role.role_id,
            display_name: role.display_name,
            phase: role.phase,
            position_family: role.position_family,
            position_tags: role.position_tags,
            score: role.score,
            contributions: role
                .contributions
                .into_iter()
                .map(MoneyballRoleContributionDto::from)
                .collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyballRoleContributionDto {
    pub metric_key: String,
    pub source_label: String,
    pub weight: f64,
    pub direction: String,
    pub percentile: Option<u8>,
    pub weighted_contribution: Option<f64>,
    pub available_from_source: bool,
}

impl From<MoneyballRoleContribution> for MoneyballRoleContributionDto {
    fn from(contribution: MoneyballRoleContribution) -> Self {
        Self {
            available_from_source: !is_save_unavailable_statistic_key(&contribution.metric_key),
            metric_key: contribution.metric_key,
            source_label: contribution.source_label,
            weight: contribution.weight,
            direction: contribution.direction,
            percentile: contribution.percentile,
            weighted_contribution: contribution.weighted_contribution,
        }
    }
}

impl From<MoneyballProfile> for MoneyballProfileDto {
    fn from(profile: MoneyballProfile) -> Self {
        Self {
            state: match profile.state {
                MoneyballProfileState::NoData => "noData",
                MoneyballProfileState::NeedsReimport => "needsReimport",
                MoneyballProfileState::Ready => "ready",
            },
            starts: profile.starts,
            substitute_appearances: profile.substitute_appearances,
            minutes: profile.minutes,
            statistics: profile.statistics,
            comparisons: profile
                .comparisons
                .into_iter()
                .map(MoneyballComparisonDto::from)
                .collect(),
            minimum_minutes: MIN_SEASON_MINUTES,
            role_catalog_version: profile.role_catalog_version,
            role_scores: profile
                .role_scores
                .map(|roles| roles.into_iter().map(MoneyballRoleScoreDto::from).collect()),
            league_adjustment: profile
                .league_adjustment
                .map(MoneyballLeagueBreakdownDto::from),
            borrowed_stats: profile.borrowed_stats.map(BorrowedStatsDto::from),
        }
    }
}

/// Reads the active save's current Moneyball cohort for one current player UID.
#[tauri::command]
pub async fn get_player_moneyball(
    uid: i64,
    db: State<'_, Db>,
    groups: State<'_, ComparisonGroupsCache>,
) -> Result<Option<MoneyballProfileDto>, String> {
    let db = db.inner().clone();
    let groups = groups.inner().clone();
    blocking::run(move || read_player_moneyball(&db, &groups, uid)).await
}

fn read_player_moneyball(
    db: &Db,
    groups: &ComparisonGroupsCache,
    uid: i64,
) -> Result<Option<MoneyballProfileDto>, String> {
    let conn = db.read()?;
    let groups = GroupsSource::Cached(groups, conn.generation());
    query::get_player_moneyball(&conn, &groups, uid)
        .map(|profile| profile.map(MoneyballProfileDto::from))
}

/// Whether the active save adjusts Moneyball scores for league strength, and the league it
/// adjusts them to. No reference means its current snapshot has no measured leagues yet.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyballLeagueAdjustmentDto {
    pub enabled: bool,
    pub reference_division: Option<String>,
}

fn league_adjustment(
    conn: &rusqlite::Connection,
    save_id: i64,
) -> Result<MoneyballLeagueAdjustmentDto, String> {
    Ok(MoneyballLeagueAdjustmentDto {
        enabled: league_strength::is_enabled(conn, save_id)?,
        reference_division: LeagueAdjustment::reference_division(conn, save_id)?,
    })
}

#[tauri::command]
pub async fn get_moneyball_league_adjustment(
    db: State<'_, Db>,
) -> Result<MoneyballLeagueAdjustmentDto, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let conn = db.read()?;
        league_adjustment(&conn, active_save_id(&conn)?)
    })
    .await
}

#[tauri::command]
pub async fn set_moneyball_league_adjustment(
    enabled: bool,
    db: State<'_, Db>,
) -> Result<MoneyballLeagueAdjustmentDto, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let conn =
            db.0.lock()
                .map_err(|_| "database lock poisoned".to_string())?;
        let save_id = active_save_id(&conn)?;
        league_strength::set_enabled(&conn, save_id, enabled)?;
        league_adjustment(&conn, save_id)
    })
    .await
}

/// Why Refresh stats did not read the save. The frontend matches on `kind`.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct RefreshSeasonStatsError {
    kind: &'static str,
    message: &'static str,
}

const LOAD_DATA_RUNNING: RefreshSeasonStatsError = RefreshSeasonStatsError {
    kind: "loadDataRunning",
    message: "Load Data is running. Refresh stats after it finishes.",
};

/// Re-reads the linked FM save file of save `save_id`, identified by `context_token`, into
/// its current snapshot, unless Load Data is running.
#[tauri::command]
pub async fn refresh_season_stats(
    save_id: i64,
    context_token: String,
    db: State<'_, Db>,
) -> Result<SeasonStatsOutcome, RefreshSeasonStatsError> {
    let start = Instant::now();
    let db = db.inner().clone();
    blocking::run(move || {
        refresh_season_stats_with(&db, save_id, &context_token, &mut || {
            start.elapsed().as_millis() as u64
        })
    })
    .await
}

/// The check does not hold the load gate during the read, so a Load Data that starts
/// meanwhile is not blocked; if it publishes first, the attach's token revalidation
/// discards this read. A running boost holds the load gate too but never changes the
/// current snapshot, so a refresh proceeds while one runs.
fn refresh_season_stats_with(
    db: &Db,
    save_id: i64,
    context_token: &str,
    now_ms: &mut dyn FnMut() -> u64,
) -> Result<SeasonStatsOutcome, RefreshSeasonStatsError> {
    if boost_gate::is_load_data_running() {
        return Err(LOAD_DATA_RUNNING);
    }
    Ok(attach_season_stats(db, save_id, context_token, now_ms))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use rusqlite::{params, Connection};
    use serde_json::json;

    use super::LOAD_DATA_RUNNING;
    use super::{refresh_season_stats_with, MoneyballProfileDto, RefreshSeasonStatsError};
    use crate::db::{migrations, Db};
    use crate::features::moneyball::comparison::{ComparisonGroupsCache, PositionFamily};
    use crate::features::moneyball::persistence::test_support::{cohort_state, seed_cohort};
    use crate::features::moneyball::query::{
        MoneyballComparison, MoneyballProfile, MoneyballProfileState, MoneyballRoleContribution,
        MoneyballRoleScore,
    };
    use crate::features::moneyball::save_stats_import::SeasonStatsResult;
    use crate::features::player::boost_gate::{
        acquire_boost_gate, acquire_load_gate, BOOST_TEST_GATE,
    };
    use crate::features::save_stats::synthetic::{write_season_stats_save, GAME_DATE};
    use crate::features::snapshot::service::{self as snapshot_service, LinkedSaveFile};

    fn ready_profile(comparisons: Vec<MoneyballComparison>) -> MoneyballProfile {
        MoneyballProfile {
            state: MoneyballProfileState::Ready,
            starts: Some(18),
            substitute_appearances: Some(3),
            minutes: Some(1_500),
            statistics: Some(BTreeMap::new()),
            comparisons,
            role_catalog_version: Some(1),
            role_scores: Some(Vec::new()),
            league_adjustment: None,
            borrowed_stats: None,
        }
    }

    #[test]
    fn moneyball_profile_answers_on_a_reader() {
        let (_temp_dir, db) = crate::db::open_with_golden_snapshot();
        {
            let conn = db.0.lock().expect("lock writer");
            let snapshot_id: i64 = conn
                .query_row("SELECT id FROM snapshots WHERE is_current = 1", [], |row| {
                    row.get(0)
                })
                .expect("current snapshot");
            seed_cohort(&conn, snapshot_id, 77, PRIOR_AS_OF);
        }

        let profile = crate::db::read_while_writer_is_held(&db, |db| {
            super::read_player_moneyball(db, &ComparisonGroupsCache::default(), 77)
        })
        .expect("Moneyball profile on a reader")
        .expect("cohort player");

        // The seeded cohort row has no statistics, which the profile reports for reimport.
        assert_eq!(profile.state, "needsReimport");
    }

    #[test]
    fn serializes_each_comparison_with_its_family_id_and_the_minimum_minutes() {
        let dto = MoneyballProfileDto::from(ready_profile(vec![MoneyballComparison {
            family: PositionFamily::Winger,
            peers: 12,
            percentiles: BTreeMap::from([("goals".to_owned(), Some(40))]),
        }]));

        let value = serde_json::to_value(dto).expect("serialize profile");

        assert_eq!(
            value["comparisons"],
            json!([{ "family": "winger", "peers": 12, "percentiles": { "goals": 40 } }])
        );
        assert_eq!(value["minimumMinutes"], json!(450));
    }

    fn contribution(metric_key: &str, percentile: Option<u8>) -> MoneyballRoleContribution {
        MoneyballRoleContribution {
            metric_key: metric_key.to_owned(),
            source_label: metric_key.to_owned(),
            weight: 0.5,
            direction: "higher".to_owned(),
            percentile,
            weighted_contribution: percentile.map(f64::from),
        }
    }

    #[test]
    fn marks_contributions_that_save_files_cannot_provide() {
        let mut profile = ready_profile(Vec::new());
        profile.role_scores = Some(vec![MoneyballRoleScore {
            role_id: "mc_pressing_central_midfielder_oop".to_owned(),
            display_name: "Pressing Central Midfielder".to_owned(),
            phase: "out_of_possession".to_owned(),
            position_family: "central_midfielder".to_owned(),
            position_tags: vec!["MC".to_owned()],
            score: Some(60),
            contributions: vec![
                contribution("goals_from_outside_the_box_per_90", None),
                contribution("tackles_completed_per_90", Some(60)),
            ],
        }]);

        let value = serde_json::to_value(MoneyballProfileDto::from(profile)).expect("serialize");
        let contributions = &value["roleScores"][0]["contributions"];

        assert_eq!(contributions[0]["availableFromSource"], json!(false));
        assert_eq!(contributions[1]["availableFromSource"], json!(true));
    }

    const PRIOR_AS_OF: &str = "2030-01-01";

    struct RefreshFixture {
        db: Db,
        save_id: i64,
        save_token: String,
        snapshot_id: i64,
        directory: tempfile::TempDir,
    }

    /// The active save's current snapshot holds players 10 and 20, with a prior one-player
    /// cohort on player 20.
    fn refresh_fixture() -> RefreshFixture {
        let directory = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(directory.path().join("refresh.db")).expect("open db");
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
                0, NULL, 2)",
            [save.id],
        )
        .expect("insert snapshot");
        let snapshot_id = conn.last_insert_rowid();
        for uid in [10, 20] {
            conn.execute(
                "INSERT INTO players (
                    snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                    nationalities_json, preferred_foot, positions_json, attributes_json,
                    hidden_attributes_json, personality_json
                 ) VALUES (?1, ?2, 100, 100, 'Refresh player', 2000, 1, '[]', 'Right', '{}',
                    '{}', '{}', '{}')",
                params![snapshot_id, uid],
            )
            .expect("insert player");
        }
        seed_cohort(&conn, snapshot_id, 20, PRIOR_AS_OF);
        RefreshFixture {
            db: Db::new(conn),
            save_id: save.id,
            save_token: save.context_token,
            snapshot_id,
            directory,
        }
    }

    impl RefreshFixture {
        fn link_matching_save(&self) {
            let path = write_season_stats_save(
                self.directory.path(),
                &[
                    (10, &[("minutes", 90.0), ("goals", 1.0)]),
                    (20, &[("minutes", 90.0), ("goals", 2.0)]),
                ],
            );
            let file = LinkedSaveFile::validate(path.to_str().expect("UTF-8 path"))
                .expect("valid FM27 save");
            let conn = self.db.0.lock().expect("lock");
            snapshot_service::set_save_file_link(
                &conn,
                self.save_id,
                &self.save_token,
                Some(&file),
            )
            .expect("link save file");
        }

        fn refresh(
            &self,
            context_token: &str,
        ) -> Result<SeasonStatsResult, RefreshSeasonStatsError> {
            refresh_season_stats_with(&self.db, self.save_id, context_token, &mut || 0)
                .map(|outcome| outcome.result)
        }

        fn state(&self) -> (i64, Option<String>) {
            cohort_state(&self.db.0.lock().expect("lock"), self.snapshot_id)
        }
    }

    #[test]
    fn refresh_attaches_the_linked_save_to_the_current_snapshot() {
        let _gate = BOOST_TEST_GATE.lock().unwrap_or_else(|e| e.into_inner());
        let fixture = refresh_fixture();
        fixture.link_matching_save();

        assert_eq!(
            fixture.refresh(&fixture.save_token),
            Ok(SeasonStatsResult::Attached {
                matched_players: 2,
                unmatched_players: 0,
                game_date: GAME_DATE.to_string(),
            })
        );
        assert_eq!(fixture.state(), (2, Some(GAME_DATE.to_string())));
    }

    #[test]
    fn refresh_still_attaches_while_a_boost_runs() {
        let _gate = BOOST_TEST_GATE.lock().unwrap_or_else(|e| e.into_inner());
        let fixture = refresh_fixture();
        fixture.link_matching_save();
        let _boost = acquire_boost_gate().expect("acquire boost");

        assert!(matches!(
            fixture.refresh(&fixture.save_token),
            Ok(SeasonStatsResult::Attached { .. })
        ));
    }

    #[test]
    fn refresh_is_rejected_while_load_data_runs_and_writes_nothing() {
        let _gate = BOOST_TEST_GATE.lock().unwrap_or_else(|e| e.into_inner());
        let fixture = refresh_fixture();
        fixture.link_matching_save();
        let _load = acquire_load_gate().expect("acquire load");

        assert_eq!(fixture.refresh(&fixture.save_token), Err(LOAD_DATA_RUNNING));
        assert_eq!(fixture.state(), (1, Some(PRIOR_AS_OF.to_string())));
    }

    #[test]
    fn refresh_with_a_stale_save_token_writes_nothing() {
        let _gate = BOOST_TEST_GATE.lock().unwrap_or_else(|e| e.into_inner());
        let fixture = refresh_fixture();
        fixture.link_matching_save();

        assert_eq!(
            fixture.refresh("stale-token"),
            Ok(SeasonStatsResult::StaleContext)
        );
        assert_eq!(fixture.state(), (1, Some(PRIOR_AS_OF.to_string())));
    }

    #[test]
    fn refresh_without_a_linked_save_reports_not_linked() {
        let _gate = BOOST_TEST_GATE.lock().unwrap_or_else(|e| e.into_inner());
        let fixture = refresh_fixture();

        assert_eq!(
            fixture.refresh(&fixture.save_token),
            Ok(SeasonStatsResult::NotLinked)
        );
        assert_eq!(fixture.state(), (1, Some(PRIOR_AS_OF.to_string())));
    }

    #[test]
    fn the_load_data_running_error_serializes_its_kind() {
        assert_eq!(
            serde_json::to_value(LOAD_DATA_RUNNING).expect("serialize error"),
            json!({
                "kind": "loadDataRunning",
                "message": "Load Data is running. Refresh stats after it finishes.",
            })
        );
    }
}
