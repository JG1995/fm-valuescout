use std::collections::BTreeMap;

use rusqlite::{params, Connection, OptionalExtension};

use super::{
    columns::StatisticColumns,
    comparison::{player_families, GroupsSource, PositionFamily},
    kept_stats::{kept_stats_snapshots, stats_join_sql},
    league_strength::{shifted, LeagueAdjustment, LeagueBreakdown},
    role_catalog::{builtin_catalog, RoleDefinition, RolePhase},
    role_score::explain_role,
    MONEYBALL_STATISTIC_KEYS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoneyballProfileState {
    NoData,
    NeedsReimport,
    Ready,
}

/// The player's percentiles within one family's peers.
#[derive(Debug, Clone, PartialEq)]
pub struct MoneyballComparison {
    pub family: PositionFamily,
    /// The family's peers, whether or not they have a given statistic.
    pub peers: usize,
    pub percentiles: BTreeMap<String, Option<u8>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoneyballProfile {
    pub state: MoneyballProfileState,
    pub starts: Option<i64>,
    pub substitute_appearances: Option<i64>,
    pub minutes: Option<i64>,
    pub statistics: Option<BTreeMap<String, Option<f64>>>,
    /// The player's percentiles within each family he can be compared in, his best
    /// position's first. Empty when he is rated at no position.
    pub comparisons: Vec<MoneyballComparison>,
    pub role_catalog_version: Option<u32>,
    /// Each role ranked within its own family.
    pub role_scores: Option<Vec<MoneyballRoleScore>>,
    /// What moved every role score for the player's league, or none when nothing did.
    pub league_adjustment: Option<LeagueBreakdown>,
    /// Set when the player has under `MIN_SEASON_MINUTES` minutes this season and the profile
    /// shows a kept snapshot's statistics instead (see `kept_stats`).
    pub borrowed_stats: Option<BorrowedStats>,
}

/// Where a profile's borrowed statistics come from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BorrowedStats {
    /// The in-game date of the kept snapshot's statistics.
    pub stats_date: String,
    /// The player's minutes this season.
    pub current_minutes: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoneyballRoleScore {
    pub role_id: String,
    pub display_name: String,
    pub phase: String,
    pub position_family: String,
    pub position_tags: Vec<String>,
    /// The weighted percentile score, moved by the profile's league adjustment.
    pub score: Option<u8>,
    pub contributions: Vec<MoneyballRoleContribution>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoneyballRoleContribution {
    pub metric_key: String,
    pub source_label: String,
    pub weight: f64,
    pub direction: String,
    pub percentile: Option<u8>,
    pub weighted_contribution: Option<f64>,
}

impl MoneyballProfile {
    fn no_data() -> Self {
        Self {
            state: MoneyballProfileState::NoData,
            starts: None,
            substitute_appearances: None,
            minutes: None,
            statistics: None,
            comparisons: Vec::new(),
            role_catalog_version: None,
            role_scores: None,
            league_adjustment: None,
            borrowed_stats: None,
        }
    }

    fn needs_reimport() -> Self {
        Self {
            state: MoneyballProfileState::NeedsReimport,
            ..Self::no_data()
        }
    }
}

/// Reads only the active save's effective current snapshot for a known player UID, or the
/// kept snapshot whose statistics he borrows. A current player with no imported cohort is
/// distinct from a pre-v30 unscored row.
pub(crate) fn get_player_moneyball(
    conn: &Connection,
    groups: &GroupsSource<'_>,
    uid: i64,
) -> Result<Option<MoneyballProfile>, String> {
    let snapshot_id: Option<i64> = conn
        .query_row(
            "SELECT snapshots.id
             FROM snapshots
             INNER JOIN saves ON saves.id = snapshots.save_id AND saves.is_active = 1
             WHERE snapshots.is_current = 1
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let Some(snapshot_id) = snapshot_id else {
        return Ok(None);
    };

    let player_exists: bool = conn
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM players WHERE snapshot_id = ?1 AND uid = ?2
             )",
            params![snapshot_id, uid],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !player_exists {
        return Ok(None);
    }

    // His positions are today's, while his league is the one he played the statistics in.
    let kept = kept_stats_snapshots(conn, snapshot_id)?;
    let kept_ids = kept.iter().map(|kept| kept.snapshot_id).collect::<Vec<_>>();
    let columns = StatisticColumns::all();
    let (
        source_id,
        starts,
        substitute_appearances,
        minutes,
        percentiles_json,
        positions_json,
        division,
        current_minutes,
        statistic_values,
    ) = conn
        .query_row(
            &format!(
                "SELECT moneyball.snapshot_id, moneyball.starts, moneyball.substitute_appearances,
                        moneyball.minutes, moneyball.percentiles_json,
                        players.positions_json,
                        (SELECT source.division FROM players source
                         WHERE source.snapshot_id = moneyball.snapshot_id
                           AND source.uid = players.uid),
                        (SELECT own.minutes FROM player_moneyball_stats own
                         WHERE own.snapshot_id = players.snapshot_id
                           AND own.player_uid = players.uid){}
                 FROM players{}
                 WHERE players.snapshot_id = ?1 AND players.uid = ?2",
                columns.appended_sql("moneyball"),
                stats_join_sql("players", &kept_ids)
            ),
            params![snapshot_id, uid],
            |row| {
                Ok((
                    row.get::<_, Option<i64>>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                    columns.read(row, 8)?,
                ))
            },
        )
        .map_err(|error| error.to_string())?;
    let Some(source_id) = source_id else {
        return Ok(Some(MoneyballProfile::no_data()));
    };
    let borrowed = kept.into_iter().find(|kept| kept.snapshot_id == source_id);

    if percentiles_json.is_none() {
        return Ok(Some(MoneyballProfile::needs_reimport()));
    }

    let statistic_values = statistic_values
        .iter()
        .map(|value| Some(*value).filter(|value| !value.is_nan()))
        .collect::<Vec<_>>();
    let statistics = MONEYBALL_STATISTIC_KEYS
        .iter()
        .zip(&statistic_values)
        .map(|(key, value)| ((*key).to_string(), *value))
        .collect::<BTreeMap<_, _>>();
    let groups = match &borrowed {
        None => groups.load(conn, snapshot_id)?,
        Some(kept) => groups.load_kept(conn, kept.snapshot_id, &kept.stats_date)?,
    };
    let comparisons = player_families(&positions_json)?
        .into_iter()
        .map(|family| {
            let rankings = groups.family(family);
            MoneyballComparison {
                family,
                peers: rankings.peers(),
                percentiles: MONEYBALL_STATISTIC_KEYS
                    .iter()
                    .zip(&statistic_values)
                    .enumerate()
                    .map(|(index, (key, value))| {
                        ((*key).to_string(), rankings.percentile(index, *value))
                    })
                    .collect(),
            }
        })
        .collect();
    let role_catalog = builtin_catalog()?;
    let league_adjustment = LeagueAdjustment::load(conn, source_id)?.breakdown(division.as_deref());
    let league_shift = league_adjustment
        .as_ref()
        .map_or(0, |adjustment| adjustment.shift);
    let role_scores = score_roles(
        &role_catalog.definitions,
        |family, key| {
            groups
                .family(family)
                .percentile_of(key, statistics.get(key).copied().flatten())
        },
        league_shift,
    )?;

    Ok(Some(MoneyballProfile {
        state: MoneyballProfileState::Ready,
        starts,
        substitute_appearances,
        minutes,
        statistics: Some(statistics),
        comparisons,
        role_catalog_version: Some(role_catalog.version),
        role_scores: Some(role_scores),
        league_adjustment,
        borrowed_stats: borrowed.map(|kept| BorrowedStats {
            stats_date: kept.stats_date,
            current_minutes: current_minutes.unwrap_or(0),
        }),
    }))
}

/// Each role's score, its metrics ranked within the role's family by `percentile`.
fn score_roles(
    definitions: &[RoleDefinition],
    percentile: impl Fn(PositionFamily, &str) -> Option<u8>,
    league_shift: i16,
) -> Result<Vec<MoneyballRoleScore>, String> {
    definitions
        .iter()
        .map(|definition| {
            let family = PositionFamily::from_id(&definition.position_family).ok_or_else(|| {
                format!(
                    "built-in Moneyball role definition `{}` has an unknown family",
                    definition.id
                )
            })?;
            let details = explain_role(definition, &|key: &str| percentile(family, key))
                .ok_or_else(|| {
                    format!(
                        "built-in Moneyball role definition `{}` cannot be scored",
                        definition.id
                    )
                })?;
            let contributions = definition
                .metrics
                .iter()
                .zip(details.contributions)
                .map(|(metric, contribution)| {
                    let source_label = metric.source_label.clone().ok_or_else(|| {
                        format!(
                            "built-in Moneyball metric `{}` has no source label",
                            metric.key
                        )
                    })?;
                    Ok(MoneyballRoleContribution {
                        metric_key: contribution.key,
                        source_label,
                        weight: contribution.weight,
                        direction: if contribution.lower_is_better {
                            "lower".to_owned()
                        } else {
                            "higher".to_owned()
                        },
                        percentile: contribution.percentile,
                        weighted_contribution: contribution.weighted_contribution,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;

            Ok(MoneyballRoleScore {
                role_id: definition.id.clone(),
                display_name: definition.display_name.clone(),
                phase: match definition.phase {
                    RolePhase::InPossession => "in_possession".to_owned(),
                    RolePhase::OutOfPossession => "out_of_possession".to_owned(),
                },
                position_family: definition.position_family.clone(),
                position_tags: definition.position_tags.clone(),
                score: details.score.map(|score| shifted(score, league_shift)),
                contributions,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use rusqlite::{params, Connection};
    use serde_json::{json, Map, Value};

    use super::get_player_moneyball;
    use crate::db::migrations;
    use crate::features::moneyball::comparison::{GroupsSource, PositionFamily};
    use crate::features::moneyball::kept_stats::test_support::add_kept_snapshot;
    use crate::features::moneyball::league_strength::test_support::measure_leagues;
    use crate::features::moneyball::persistence::test_support::insert_statistics_row;
    use crate::features::moneyball::role_catalog::builtin_catalog;
    use crate::features::moneyball::MONEYBALL_STATISTIC_KEYS;
    use crate::features::snapshot::ingest::ingest_dump_file;

    fn open_migrated(path: &Path) -> Connection {
        let conn = Connection::open(path).expect("open test database");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    fn ingest_current_player(conn: &mut Connection) -> i64 {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/features/memory_read/fixtures/golden_dump_v11.json");
        ingest_dump_file(conn, &path).expect("ingest fixture").id
    }

    fn complete_statistics(value: f64) -> String {
        let values = MONEYBALL_STATISTIC_KEYS
            .iter()
            .map(|key| ((*key).to_string(), json!(value)))
            .collect::<Map<_, _>>();
        Value::Object(values).to_string()
    }

    fn insert_moneyball_row(conn: &Connection, snapshot_id: i64, percentiles_json: Option<&str>) {
        insert_moneyball_row_for_player(
            conn,
            snapshot_id,
            77,
            &complete_statistics(2.25),
            percentiles_json,
        );
    }

    fn insert_moneyball_row_for_player(
        conn: &Connection,
        snapshot_id: i64,
        player_uid: i64,
        statistics_json: &str,
        percentiles_json: Option<&str>,
    ) {
        insert_statistics_row(
            conn,
            snapshot_id,
            player_uid,
            Some(1500),
            statistics_json,
            percentiles_json,
        );
        conn.execute(
            "UPDATE player_moneyball_stats SET starts = 18, substitute_appearances = 3
             WHERE snapshot_id = ?1 AND player_uid = ?2",
            params![snapshot_id, player_uid],
        )
        .expect("set appearances");
    }

    fn insert_player(conn: &Connection, snapshot_id: i64, uid: i64, positions_json: &str) {
        conn.execute(
            "INSERT INTO players (
                snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                nationalities_json, preferred_foot, positions_json, attributes_json,
                hidden_attributes_json, personality_json
             ) VALUES (?1, ?2, 100, 100, 'Moneyball peer', 2000, 1, '[]', 'Right', ?3, '{}', '{}', '{}')",
            params![snapshot_id, uid, positions_json],
        )
        .expect("insert Moneyball peer");
    }

    fn statistics_with_values(values: &[(&str, Option<f64>)]) -> String {
        let mut statistics = MONEYBALL_STATISTIC_KEYS
            .iter()
            .map(|metric_key| ((*metric_key).to_string(), json!(1.0)))
            .collect::<Map<_, _>>();
        for (key, value) in values {
            statistics.insert(
                (*key).to_string(),
                value.map_or(Value::Null, |number| json!(number)),
            );
        }
        Value::Object(statistics).to_string()
    }

    fn statistics_with(key: &str, value: Option<f64>) -> String {
        statistics_with_values(&[(key, value)])
    }

    fn set_positions(conn: &Connection, snapshot_id: i64, uid: i64, positions_json: &str) {
        conn.execute(
            "UPDATE players SET positions_json = ?1 WHERE snapshot_id = ?2 AND uid = ?3",
            params![positions_json, snapshot_id, uid],
        )
        .expect("set player positions");
    }

    fn set_minutes(conn: &Connection, snapshot_id: i64, uid: i64, minutes: i64) {
        conn.execute(
            "UPDATE player_moneyball_stats SET minutes = ?1 WHERE snapshot_id = ?2 AND player_uid = ?3",
            params![minutes, snapshot_id, uid],
        )
        .expect("set season minutes");
    }

    fn profile(conn: &Connection, uid: i64) -> super::MoneyballProfile {
        get_player_moneyball(conn, &GroupsSource::Uncached, uid)
            .expect("query player")
            .expect("known current player")
    }

    /// The player's comparisons as family, peers, and one statistic's percentile.
    fn comparisons(
        profile: &super::MoneyballProfile,
        key: &str,
    ) -> Vec<(PositionFamily, usize, Option<u8>)> {
        profile
            .comparisons
            .iter()
            .map(|comparison| {
                (
                    comparison.family,
                    comparison.peers,
                    comparison.percentiles[key],
                )
            })
            .collect()
    }

    #[test]
    fn ranks_him_among_his_best_familys_peers_with_enough_minutes() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-profile-peers.db"));
        let snapshot_id = ingest_current_player(&mut conn);
        insert_moneyball_row_for_player(
            &conn,
            snapshot_id,
            77,
            &statistics_with("goals", Some(10.0)),
            Some("{}"),
        );
        // Peers of the winger family are rated 16 or more at AML or AMR with 450+ minutes.
        for (uid, positions, goals, minutes) in [
            (78, r#"{"AMR":20}"#, 20.0, 1_500),
            (79, r#"{"AML":16}"#, 100.0, 450),
            (80, r#"{"AMR":15}"#, 0.0, 1_500),
            (81, r#"{"AMR":20}"#, 0.0, 449),
        ] {
            insert_player(&conn, snapshot_id, uid, positions);
            insert_moneyball_row_for_player(
                &conn,
                snapshot_id,
                uid,
                &statistics_with("goals", Some(goals)),
                Some("{}"),
            );
            set_minutes(&conn, snapshot_id, uid, minutes);
        }

        // Rated 20 at AMR and 17 at MR, he is compared with wingers first, then wide
        // midfielders, where he is the only peer.
        assert_eq!(
            comparisons(&profile(&conn, 77), "goals"),
            vec![
                (PositionFamily::Winger, 3, Some(0)),
                (PositionFamily::WideMidfielder, 1, Some(50)),
            ]
        );
        // A short sample is ranked against the group without joining it.
        assert_eq!(
            comparisons(&profile(&conn, 81), "goals"),
            vec![(PositionFamily::Winger, 3, Some(0))]
        );
        set_minutes(&conn, snapshot_id, 81, 450);
        assert_eq!(
            comparisons(&profile(&conn, 77), "goals")[0],
            (PositionFamily::Winger, 4, Some(33))
        );
    }

    #[test]
    fn preserves_ties_nulls_inversion_and_singleton_neutrality_in_the_group() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-profile-rules.db"));
        let snapshot_id = ingest_current_player(&mut conn);
        insert_moneyball_row_for_player(
            &conn,
            snapshot_id,
            77,
            &statistics_with_values(&[
                ("goals", Some(10.0)),
                ("minutes_per_goal", Some(90.0)),
                ("assists", None),
                ("shots", Some(4.0)),
            ]),
            Some("{}"),
        );
        for (uid, goals, minutes_per_goal, assists) in
            [(78, 10.0, 60.0, 5.0), (79, 20.0, 30.0, 5.0)]
        {
            insert_player(&conn, snapshot_id, uid, r#"{"AMR":20}"#);
            insert_moneyball_row_for_player(
                &conn,
                snapshot_id,
                uid,
                &statistics_with_values(&[
                    ("goals", Some(goals)),
                    ("minutes_per_goal", Some(minutes_per_goal)),
                    ("assists", Some(assists)),
                    ("shots", Some(4.0)),
                ]),
                Some("{}"),
            );
        }

        let percentiles = profile(&conn, 77).comparisons.remove(0).percentiles;

        assert_eq!(percentiles["goals"], Some(0));
        assert_eq!(percentiles["minutes_per_goal"], Some(0));
        assert_eq!(percentiles["assists"], None);
        assert_eq!(percentiles["shots"], Some(50));
    }

    #[test]
    fn ignores_moneyball_rows_from_an_older_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-profile-scope.db"));
        let first_snapshot_id = ingest_current_player(&mut conn);
        insert_moneyball_row_for_player(
            &conn,
            first_snapshot_id,
            77,
            &statistics_with("goals", Some(100.0)),
            Some("{}"),
        );
        insert_player(&conn, first_snapshot_id, 78, r#"{"AMR":20}"#);
        insert_moneyball_row_for_player(
            &conn,
            first_snapshot_id,
            78,
            &statistics_with("goals", Some(1.0)),
            Some("{}"),
        );

        let second_snapshot_id = ingest_current_player(&mut conn);
        insert_moneyball_row_for_player(
            &conn,
            second_snapshot_id,
            77,
            &statistics_with("goals", Some(10.0)),
            Some("{}"),
        );
        insert_player(&conn, second_snapshot_id, 78, r#"{"AMR":20}"#);
        insert_moneyball_row_for_player(
            &conn,
            second_snapshot_id,
            78,
            &statistics_with("goals", Some(20.0)),
            Some("{}"),
        );
        conn.execute(
            "UPDATE snapshots SET is_current = id = ?1",
            [second_snapshot_id],
        )
        .expect("select current snapshot");

        assert_eq!(
            comparisons(&profile(&conn, 77), "goals")[0],
            (PositionFamily::Winger, 2, Some(0))
        );
    }

    #[test]
    fn a_player_rated_at_no_position_has_statistics_and_no_comparison() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-profile-no-position.db"));
        let snapshot_id = ingest_current_player(&mut conn);
        set_positions(&conn, snapshot_id, 77, r#"{"AMR":null,"ST":null}"#);
        insert_moneyball_row_for_player(
            &conn,
            snapshot_id,
            77,
            &statistics_with("goals", Some(10.0)),
            Some("{}"),
        );

        let profile = profile(&conn, 77);

        assert_eq!(
            profile.statistics.expect("raw metrics")["goals"],
            Some(10.0)
        );
        assert!(profile.comparisons.is_empty());
        // Roles still rank him within their families, which have no peers here.
        let roles = profile.role_scores.expect("role scores");
        assert_eq!(roles.len(), 88);
        assert!(roles.iter().all(|role| role.score.is_none()));
    }

    #[test]
    fn distinguishes_absent_legacy_and_scored_current_rows() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-profile.db"));
        let snapshot_id = ingest_current_player(&mut conn);

        let absent = profile(&conn, 77);
        assert_eq!(absent.state, super::MoneyballProfileState::NoData);
        assert!(absent.role_scores.is_none());

        insert_moneyball_row(&conn, snapshot_id, None);
        let legacy = profile(&conn, 77);
        assert_eq!(legacy.state, super::MoneyballProfileState::NeedsReimport);
        assert!(legacy.statistics.is_none());
        assert!(legacy.comparisons.is_empty());
        assert!(legacy.role_scores.is_none());

        conn.execute(
            "UPDATE player_moneyball_stats SET percentiles_json = '{}' WHERE snapshot_id = ?1 AND player_uid = 77",
            [snapshot_id],
        )
        .expect("score imported row");
        let scored = profile(&conn, 77);
        assert_eq!(scored.state, super::MoneyballProfileState::Ready);
        assert_eq!(scored.starts, Some(18));
        assert_eq!(scored.statistics.as_ref().expect("raw metrics").len(), 138);
        assert_eq!(scored.comparisons[0].percentiles.len(), 138);
        assert_eq!(
            scored.statistics.as_ref().expect("raw metrics")["goals"],
            Some(2.25)
        );
        assert_eq!(scored.comparisons[0].percentiles["goals"], Some(50));
        assert_eq!(scored.role_catalog_version, Some(1));
        // Alone in his families, he scores 50 in their roles; the others have no peers.
        let roles = scored.role_scores.as_ref().expect("role scores");
        assert_eq!(roles.len(), 88);
        for role in roles {
            let his_family = ["winger", "wide_midfielder"].contains(&role.position_family.as_str());
            assert_eq!(role.score, his_family.then_some(50), "{}", role.role_id);
        }
    }

    #[test]
    fn each_role_ranks_within_its_own_family() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-profile-role-family.db"));
        let snapshot_id = ingest_current_player(&mut conn);
        insert_moneyball_row_for_player(
            &conn,
            snapshot_id,
            77,
            &complete_statistics(5.0),
            Some("{}"),
        );
        // Strikers have every statistic lower than his, wingers every one higher.
        for (uid, positions, value) in [
            (78, r#"{"ST":20}"#, 1.0),
            (79, r#"{"ST":20}"#, 2.0),
            (80, r#"{"AMR":20}"#, 9.0),
        ] {
            insert_player(&conn, snapshot_id, uid, positions);
            insert_moneyball_row_for_player(
                &conn,
                snapshot_id,
                uid,
                &complete_statistics(value),
                Some("{}"),
            );
        }

        let profile = profile(&conn, 77);
        let catalog = builtin_catalog().expect("built-in catalog");
        let score = |family: &str| {
            let definition = catalog
                .definitions
                .iter()
                .find(|definition| {
                    definition.position_family == family
                        && definition.metrics.iter().all(|metric| !metric.inverted)
                })
                .expect("a role without inverted metrics");
            profile
                .role_scores
                .as_ref()
                .expect("role scores")
                .iter()
                .find(|role| role.role_id == definition.id)
                .expect("role score")
                .score
        };

        assert_eq!(score("striker"), Some(100));
        assert_eq!(score("winger"), Some(0));
    }

    #[test]
    fn role_scores_move_with_the_players_league() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-league.db"));
        let snapshot_id = ingest_current_player(&mut conn);
        insert_moneyball_row(&conn, snapshot_id, Some("{}"));
        conn.execute(
            "UPDATE players SET division = 'Middle' WHERE snapshot_id = ?1 AND uid = 77",
            [snapshot_id],
        )
        .expect("set division");
        let scores = |conn: &Connection| {
            profile(conn, 77)
                .role_scores
                .expect("role scores")
                .into_iter()
                .filter_map(|role| role.score)
                .collect::<Vec<_>>()
        };

        // Alone in his families, the player scores 50 in their roles.
        assert!(profile(&conn, 77).league_adjustment.is_none());
        assert!(scores(&conn).iter().all(|score| *score == 50));

        measure_leagues(&conn, snapshot_id, &[("Top", 150.0), ("Middle", 138.0)]);
        let adjustment = profile(&conn, 77)
            .league_adjustment
            .expect("league adjustment");
        assert_eq!(
            (
                adjustment.division.as_deref(),
                adjustment.strength,
                adjustment.reference_division.as_str(),
                adjustment.reference_strength,
                adjustment.shift
            ),
            (Some("Middle"), 138.0, "Top", 150.0, -6)
        );
        let adjusted = scores(&conn);
        assert!(!adjusted.is_empty());
        assert!(adjusted.iter().all(|score| *score == 44));
    }

    #[test]
    fn identifies_a_null_metric_in_the_role_explanation() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-role-explanation.db"));
        let snapshot_id = ingest_current_player(&mut conn);
        let catalog = builtin_catalog().expect("built-in catalog");
        let definition = catalog
            .definitions
            .iter()
            .find(|definition| definition.position_family == "winger")
            .expect("a winger role");
        let first_metric = &definition.metrics[0];
        insert_moneyball_row_for_player(
            &conn,
            snapshot_id,
            77,
            &statistics_with(&first_metric.key, None),
            Some("{}"),
        );

        let profile = profile(&conn, 77);
        let role = profile
            .role_scores
            .as_ref()
            .expect("role scores")
            .iter()
            .find(|role| role.role_id == definition.id)
            .expect("winger role");

        assert_eq!(role.score, None);
        let missing = role
            .contributions
            .iter()
            .find(|contribution| contribution.metric_key == first_metric.key)
            .expect("missing metric contribution");
        assert_eq!(missing.percentile, None);
        assert_eq!(missing.weighted_contribution, None);
        assert_eq!(
            Some(&missing.source_label),
            first_metric.source_label.as_ref()
        );
        // The role's other metrics still rank him alone among wingers.
        assert!(role
            .contributions
            .iter()
            .filter(|contribution| contribution.metric_key != first_metric.key)
            .all(|contribution| contribution.percentile == Some(50)));
    }

    #[test]
    fn a_short_sample_shows_the_kept_stats_he_borrows_ranked_in_their_season() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-profile-borrowed.db"));
        let snapshot_id = ingest_current_player(&mut conn);
        insert_moneyball_row_for_player(
            &conn,
            snapshot_id,
            77,
            &statistics_with("goals", Some(50.0)),
            Some("{}"),
        );
        set_minutes(&conn, snapshot_id, 77, 200);
        insert_player(&conn, snapshot_id, 78, r#"{"AMR":20}"#);
        insert_moneyball_row_for_player(
            &conn,
            snapshot_id,
            78,
            &statistics_with("goals", Some(100.0)),
            Some("{}"),
        );
        conn.execute(
            "UPDATE players SET division = 'Top' WHERE snapshot_id = ?1",
            [snapshot_id],
        )
        .expect("set current divisions");
        let kept = add_kept_snapshot(&conn, snapshot_id, "2026-05-20", &[77, 78]);
        conn.execute(
            "UPDATE snapshots SET moneyball_stats_game_date = '2026-05-20' WHERE id = ?1",
            [kept],
        )
        .expect("date kept stats");
        for (uid, goals) in [(77, 10.0), (78, 5.0)] {
            insert_moneyball_row_for_player(
                &conn,
                kept,
                uid,
                &statistics_with("goals", Some(goals)),
                Some("{}"),
            );
        }
        conn.execute(
            "UPDATE players SET division = 'Second' WHERE snapshot_id = ?1 AND uid = 77",
            [kept],
        )
        .expect("set kept division");
        measure_leagues(&conn, kept, &[("Top", 150.0), ("Second", 140.0)]);

        let profile = profile(&conn, 77);

        assert_eq!(
            profile.borrowed_stats,
            Some(super::BorrowedStats {
                stats_date: "2026-05-20".to_string(),
                current_minutes: 200,
            })
        );
        assert_eq!(profile.minutes, Some(1500));
        assert_eq!(
            profile.statistics.as_ref().expect("statistics")["goals"],
            Some(10.0)
        );
        // Among that season's wingers, 5 and 10, his 10 ranks at 100. Today he would be
        // ranked against the one 100.
        assert_eq!(
            comparisons(&profile, "goals")[0],
            (PositionFamily::Winger, 2, Some(100))
        );
        let league = profile.league_adjustment.expect("league adjustment");
        assert_eq!(
            (league.division.as_deref(), league.shift),
            (Some("Second"), -5)
        );
    }

    #[test]
    fn rejects_unknown_uids_and_older_snapshot_rows() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-profile-snapshots.db"));
        let first_snapshot_id = ingest_current_player(&mut conn);
        insert_moneyball_row(&conn, first_snapshot_id, Some("{}"));
        let second_snapshot_id = ingest_current_player(&mut conn);
        conn.execute(
            "UPDATE snapshots SET is_current = id = ?1",
            [second_snapshot_id],
        )
        .expect("select latest snapshot");

        assert!(
            get_player_moneyball(&conn, &GroupsSource::Uncached, 999_999)
                .expect("query unknown uid")
                .is_none()
        );
        assert_eq!(
            profile(&conn, 77).state,
            super::MoneyballProfileState::NoData
        );
    }
}
