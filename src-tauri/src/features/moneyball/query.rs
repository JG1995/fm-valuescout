use std::collections::{BTreeMap, BTreeSet};

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;

use super::{
    percentile::calculate_percentiles,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyballComparisonBasis {
    Available {
        natural_positions: Vec<String>,
        comparison_player_count: usize,
    },
    UnavailableNoNaturalPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoneyballProfile {
    pub state: MoneyballProfileState,
    pub asking_price_kind: Option<String>,
    pub asking_price_lower_eur: Option<i64>,
    pub asking_price_upper_eur: Option<i64>,
    pub starts: Option<i64>,
    pub substitute_appearances: Option<i64>,
    pub minutes: Option<i64>,
    pub statistics: Option<BTreeMap<String, Option<f64>>>,
    pub percentiles: Option<BTreeMap<String, Option<u8>>>,
    pub role_catalog_version: Option<u32>,
    pub role_scores: Option<Vec<MoneyballRoleScore>>,
    pub comparison_basis: Option<MoneyballComparisonBasis>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoneyballRoleScore {
    pub role_id: String,
    pub display_name: String,
    pub phase: String,
    pub position_family: String,
    pub position_tags: Vec<String>,
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
            asking_price_kind: None,
            asking_price_lower_eur: None,
            asking_price_upper_eur: None,
            starts: None,
            substitute_appearances: None,
            minutes: None,
            statistics: None,
            percentiles: None,
            role_catalog_version: None,
            role_scores: None,
            comparison_basis: None,
        }
    }

    fn needs_reimport() -> Self {
        Self {
            state: MoneyballProfileState::NeedsReimport,
            ..Self::no_data()
        }
    }
}

/// Reads only the active save's effective current snapshot for a known player UID.
/// A current player with no imported cohort is distinct from a pre-v30 unscored row.
pub fn get_player_moneyball(
    conn: &Connection,
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

    let row = conn
        .query_row(
            "SELECT moneyball.asking_price_kind, moneyball.asking_price_lower_eur,
                    moneyball.asking_price_upper_eur, moneyball.starts,
                    moneyball.substitute_appearances, moneyball.minutes,
                    moneyball.statistics_json, moneyball.percentiles_json, players.positions_json
             FROM player_moneyball_stats moneyball
             INNER JOIN players ON players.snapshot_id = moneyball.snapshot_id
                AND players.uid = moneyball.player_uid
             WHERE moneyball.snapshot_id = ?1 AND moneyball.player_uid = ?2",
            params![snapshot_id, uid],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, String>(8)?,
                ))
            },
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let Some((
        asking_price_kind,
        asking_price_lower_eur,
        asking_price_upper_eur,
        starts,
        substitute_appearances,
        minutes,
        statistics_json,
        percentiles_json,
        positions_json,
    )) = row
    else {
        return Ok(Some(MoneyballProfile::no_data()));
    };

    if percentiles_json.is_none() {
        return Ok(Some(MoneyballProfile::needs_reimport()));
    }

    let statistics = parse_statistics(&statistics_json)?;
    let natural_positions = natural_positions(&positions_json)?;
    if natural_positions.is_empty() {
        return Ok(Some(MoneyballProfile {
            state: MoneyballProfileState::Ready,
            asking_price_kind,
            asking_price_lower_eur,
            asking_price_upper_eur,
            starts,
            substitute_appearances,
            minutes,
            statistics: Some(statistics),
            percentiles: None,
            role_catalog_version: None,
            role_scores: None,
            comparison_basis: Some(MoneyballComparisonBasis::UnavailableNoNaturalPosition),
        }));
    }

    let cohort_statistics = load_natural_position_cohort(conn, snapshot_id, &natural_positions)?;
    let percentiles = calculate_percentiles(&cohort_statistics)
        .remove(&uid)
        .ok_or_else(|| {
            "subject Moneyball row is missing from its natural-position cohort".to_string()
        })?;
    let role_catalog = builtin_catalog()?;
    let role_scores = score_roles(&role_catalog.definitions, &percentiles)?;

    Ok(Some(MoneyballProfile {
        state: MoneyballProfileState::Ready,
        asking_price_kind,
        asking_price_lower_eur,
        asking_price_upper_eur,
        starts,
        substitute_appearances,
        minutes,
        statistics: Some(statistics),
        percentiles: Some(percentiles),
        role_catalog_version: Some(role_catalog.version),
        role_scores: Some(role_scores),
        comparison_basis: Some(MoneyballComparisonBasis::Available {
            natural_positions,
            comparison_player_count: cohort_statistics.len(),
        }),
    }))
}

fn natural_positions(json: &str) -> Result<Vec<String>, String> {
    let value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let object = value
        .as_object()
        .ok_or_else(|| "positions_json must be an object".to_string())?;

    object
        .iter()
        .filter_map(|(position, familiarity)| match familiarity {
            Value::Number(value) if value.as_i64() == Some(20) => Some(Ok(position.clone())),
            Value::Null | Value::Number(_) => None,
            _ => Some(Err(format!(
                "position `{position}` must be an integer or null"
            ))),
        })
        .collect()
}

fn load_natural_position_cohort(
    conn: &Connection,
    snapshot_id: i64,
    subject_natural_positions: &[String],
) -> Result<BTreeMap<i64, BTreeMap<String, Option<f64>>>, String> {
    let mut statement = conn
        .prepare(
            "SELECT moneyball.player_uid, moneyball.statistics_json, players.positions_json
             FROM player_moneyball_stats moneyball
             INNER JOIN players ON players.snapshot_id = moneyball.snapshot_id
                AND players.uid = moneyball.player_uid
             WHERE moneyball.snapshot_id = ?1 AND moneyball.percentiles_json IS NOT NULL",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([snapshot_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|error| error.to_string())?;

    let mut cohort = BTreeMap::new();
    for row in rows {
        let (player_uid, statistics_json, positions_json) =
            row.map_err(|error| error.to_string())?;
        let peer_natural_positions = natural_positions(&positions_json)?;
        if peer_natural_positions
            .iter()
            .any(|position| subject_natural_positions.contains(position))
        {
            cohort.insert(player_uid, parse_statistics(&statistics_json)?);
        }
    }
    Ok(cohort)
}

fn score_roles(
    definitions: &[RoleDefinition],
    percentiles: &BTreeMap<String, Option<u8>>,
) -> Result<Vec<MoneyballRoleScore>, String> {
    definitions
        .iter()
        .map(|definition| {
            let details = explain_role(definition, percentiles).ok_or_else(|| {
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
                score: details.score,
                contributions,
            })
        })
        .collect()
}

fn parse_statistics(json: &str) -> Result<BTreeMap<String, Option<f64>>, String> {
    parse_map(json, "statistics", |key, value| match value {
        Value::Null => Ok(None),
        Value::Number(number) => number
            .as_f64()
            .filter(|number| number.is_finite())
            .map(Some)
            .ok_or_else(|| format!("Moneyball statistic `{key}` must be a finite number or null")),
        _ => Err(format!(
            "Moneyball statistic `{key}` must be a finite number or null"
        )),
    })
}

fn parse_map<T>(
    json: &str,
    name: &str,
    parse_value: impl Fn(&str, &Value) -> Result<Option<T>, String>,
) -> Result<BTreeMap<String, Option<T>>, String> {
    let value: Value = serde_json::from_str(json)
        .map_err(|error| format!("invalid Moneyball {name} JSON: {error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| format!("Moneyball {name} must be a JSON object"))?;
    let expected = MONEYBALL_STATISTIC_KEYS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(format!(
            "Moneyball {name} must contain the exact metric catalogue"
        ));
    }

    MONEYBALL_STATISTIC_KEYS
        .iter()
        .map(|key| {
            let value = object
                .get(*key)
                .expect("catalogue equality proves every metric key exists");
            parse_value(key, value).map(|parsed| ((*key).to_string(), parsed))
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
            .join("src/features/memory_read/fixtures/golden_dump_v8.json");
        ingest_dump_file(conn, &path).expect("ingest fixture").id
    }

    fn complete_statistics(value: f64) -> String {
        let values = MONEYBALL_STATISTIC_KEYS
            .iter()
            .map(|key| ((*key).to_string(), json!(value)))
            .collect::<Map<_, _>>();
        Value::Object(values).to_string()
    }

    fn complete_percentiles(value: u8) -> String {
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
        conn.execute(
            "INSERT INTO player_moneyball_stats (
                snapshot_id, player_uid, asking_price_kind, asking_price_lower_eur,
                asking_price_upper_eur, starts, substitute_appearances, minutes,
                statistics_json, percentiles_json
             ) VALUES (?1, ?2, 'single', 12000000, NULL, 18, 3, 1500, ?3, ?4)",
            params![snapshot_id, player_uid, statistics_json, percentiles_json],
        )
        .expect("insert Moneyball row");
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

    #[test]
    fn recomputes_exact_natural_position_percentiles_without_partial_peers() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-profile-cohort.db"));
        let snapshot_id = ingest_current_player(&mut conn);
        insert_moneyball_row_for_player(
            &conn,
            snapshot_id,
            77,
            &statistics_with("goals", Some(10.0)),
            Some(&complete_percentiles(99)),
        );
        insert_player(&conn, snapshot_id, 78, r#"{"AMR":20}"#);
        insert_moneyball_row_for_player(
            &conn,
            snapshot_id,
            78,
            &statistics_with("goals", Some(20.0)),
            Some(&complete_percentiles(1)),
        );
        insert_player(&conn, snapshot_id, 79, r#"{"AMR":19}"#);
        insert_moneyball_row_for_player(
            &conn,
            snapshot_id,
            79,
            &statistics_with("goals", Some(100.0)),
            Some(&complete_percentiles(1)),
        );

        let profile = get_player_moneyball(&conn, 77)
            .expect("query player")
            .expect("known current player");

        assert_eq!(
            profile.comparison_basis,
            Some(super::MoneyballComparisonBasis::Available {
                natural_positions: vec!["AMR".to_owned()],
                comparison_player_count: 2,
            })
        );
        assert_eq!(profile.percentiles.expect("percentiles")["goals"], Some(0));
    }

    #[test]
    fn deduplicates_overlapping_natural_position_peers() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-profile-union.db"));
        let snapshot_id = ingest_current_player(&mut conn);
        set_positions(&conn, snapshot_id, 77, r#"{"AMR":20,"ST":20}"#);
        insert_moneyball_row_for_player(
            &conn,
            snapshot_id,
            77,
            &statistics_with("goals", Some(10.0)),
            Some(&complete_percentiles(99)),
        );
        for (uid, positions, goals) in [
            (78, r#"{"AMR":20}"#, 20.0),
            (79, r#"{"ST":20}"#, 30.0),
            (80, r#"{"AMR":20,"ST":20}"#, 40.0),
        ] {
            insert_player(&conn, snapshot_id, uid, positions);
            insert_moneyball_row_for_player(
                &conn,
                snapshot_id,
                uid,
                &statistics_with("goals", Some(goals)),
                Some(&complete_percentiles(1)),
            );
        }

        let profile = get_player_moneyball(&conn, 77)
            .expect("query player")
            .expect("known current player");

        assert_eq!(
            profile.comparison_basis,
            Some(super::MoneyballComparisonBasis::Available {
                natural_positions: vec!["AMR".to_owned(), "ST".to_owned()],
                comparison_player_count: 4,
            })
        );
        assert_eq!(profile.percentiles.expect("percentiles")["goals"], Some(0));
    }

    #[test]
    fn preserves_ties_nulls_inversion_and_singleton_neutrality_in_the_cohort() {
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
            Some(&complete_percentiles(99)),
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
                Some(&complete_percentiles(1)),
            );
        }

        let percentiles = get_player_moneyball(&conn, 77)
            .expect("query player")
            .expect("known current player")
            .percentiles
            .expect("percentiles");

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
            Some(&complete_percentiles(99)),
        );
        insert_player(&conn, first_snapshot_id, 78, r#"{"AMR":20}"#);
        insert_moneyball_row_for_player(
            &conn,
            first_snapshot_id,
            78,
            &statistics_with("goals", Some(1.0)),
            Some(&complete_percentiles(1)),
        );

        let second_snapshot_id = ingest_current_player(&mut conn);
        insert_moneyball_row_for_player(
            &conn,
            second_snapshot_id,
            77,
            &statistics_with("goals", Some(10.0)),
            Some(&complete_percentiles(99)),
        );
        insert_player(&conn, second_snapshot_id, 78, r#"{"AMR":20}"#);
        insert_moneyball_row_for_player(
            &conn,
            second_snapshot_id,
            78,
            &statistics_with("goals", Some(20.0)),
            Some(&complete_percentiles(1)),
        );
        conn.execute(
            "UPDATE snapshots SET is_current = id = ?1",
            [second_snapshot_id],
        )
        .expect("select current snapshot");

        let profile = get_player_moneyball(&conn, 77)
            .expect("query player")
            .expect("known current player");

        assert_eq!(profile.percentiles.expect("percentiles")["goals"], Some(0));
        assert_eq!(
            profile.comparison_basis,
            Some(super::MoneyballComparisonBasis::Available {
                natural_positions: vec!["AMR".to_owned()],
                comparison_player_count: 2,
            })
        );
    }

    #[test]
    fn returns_raw_metrics_without_scores_when_no_natural_position_exists() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-profile-no-natural.db"));
        let snapshot_id = ingest_current_player(&mut conn);
        set_positions(&conn, snapshot_id, 77, r#"{"AMR":19,"ST":null}"#);
        insert_moneyball_row_for_player(
            &conn,
            snapshot_id,
            77,
            &statistics_with("goals", Some(10.0)),
            Some(&complete_percentiles(99)),
        );

        let profile = get_player_moneyball(&conn, 77)
            .expect("query player")
            .expect("known current player");

        assert_eq!(
            profile.comparison_basis,
            Some(super::MoneyballComparisonBasis::UnavailableNoNaturalPosition)
        );
        assert_eq!(
            profile.statistics.expect("raw metrics")["goals"],
            Some(10.0)
        );
        assert!(profile.percentiles.is_none());
        assert!(profile.role_catalog_version.is_none());
        assert!(profile.role_scores.is_none());
    }

    #[test]
    fn distinguishes_absent_legacy_and_scored_current_rows() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-profile.db"));
        let snapshot_id = ingest_current_player(&mut conn);

        let absent = get_player_moneyball(&conn, 77)
            .expect("query absent player")
            .expect("known current player");
        assert_eq!(absent.state, super::MoneyballProfileState::NoData);
        assert!(absent.role_scores.is_none());

        insert_moneyball_row(&conn, snapshot_id, None);
        let legacy = get_player_moneyball(&conn, 77)
            .expect("query legacy player")
            .expect("known current player");
        assert_eq!(legacy.state, super::MoneyballProfileState::NeedsReimport);
        assert!(legacy.statistics.is_none());
        assert!(legacy.percentiles.is_none());
        assert!(legacy.role_scores.is_none());

        conn.execute(
            "UPDATE player_moneyball_stats SET percentiles_json = ?1 WHERE snapshot_id = ?2 AND player_uid = 77",
            params![complete_percentiles(83), snapshot_id],
        )
        .expect("score imported row");
        let scored = get_player_moneyball(&conn, 77)
            .expect("query scored player")
            .expect("known current player");
        assert_eq!(scored.state, super::MoneyballProfileState::Ready);
        assert_eq!(scored.starts, Some(18));
        assert_eq!(scored.statistics.as_ref().expect("raw metrics").len(), 138);
        assert_eq!(scored.percentiles.as_ref().expect("scores").len(), 138);
        assert_eq!(
            scored.statistics.as_ref().expect("raw metrics")["goals"],
            Some(2.25)
        );
        assert_eq!(
            scored.percentiles.as_ref().expect("scores")["goals"],
            Some(50)
        );
        assert_eq!(scored.role_catalog_version, Some(1));
        assert_eq!(scored.role_scores.as_ref().expect("role scores").len(), 88);
        assert!(scored
            .role_scores
            .as_ref()
            .expect("role scores")
            .iter()
            .all(|role| role.score == Some(50)));
    }

    #[test]
    fn identifies_a_null_metric_in_the_role_explanation() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-role-explanation.db"));
        let snapshot_id = ingest_current_player(&mut conn);
        let catalog = builtin_catalog().expect("built-in catalog");
        let first_metric = catalog.definitions[0].metrics[0].key.clone();
        insert_moneyball_row_for_player(
            &conn,
            snapshot_id,
            77,
            &statistics_with(&first_metric, None),
            Some(&complete_percentiles(83)),
        );

        let profile = get_player_moneyball(&conn, 77)
            .expect("query player")
            .expect("known current player");
        let role = profile
            .role_scores
            .as_ref()
            .expect("role scores")
            .iter()
            .find(|role| role.role_id == catalog.definitions[0].id)
            .expect("first role");

        assert_eq!(role.score, None);
        let missing = role
            .contributions
            .iter()
            .find(|contribution| contribution.metric_key == first_metric)
            .expect("missing metric contribution");
        assert_eq!(missing.percentile, None);
        assert_eq!(missing.weighted_contribution, None);
        assert_eq!(missing.source_label, "xG per 90");
    }

    #[test]
    fn rejects_unknown_uids_and_older_snapshot_rows() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-profile-snapshots.db"));
        let first_snapshot_id = ingest_current_player(&mut conn);
        insert_moneyball_row(&conn, first_snapshot_id, Some(&complete_percentiles(61)));
        let second_snapshot_id = ingest_current_player(&mut conn);
        conn.execute(
            "UPDATE snapshots SET is_current = id = ?1",
            [second_snapshot_id],
        )
        .expect("select latest snapshot");

        assert!(get_player_moneyball(&conn, 999_999)
            .expect("query unknown uid")
            .is_none());
        let current = get_player_moneyball(&conn, 77)
            .expect("query current player")
            .expect("current player");
        assert_eq!(current.state, super::MoneyballProfileState::NoData);
    }

    #[test]
    fn rejects_scored_rows_without_the_exact_metric_catalogue() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-profile-invalid-json.db"));
        let snapshot_id = ingest_current_player(&mut conn);
        insert_moneyball_row(&conn, snapshot_id, Some(&complete_percentiles(61)));
        conn.execute(
            "UPDATE player_moneyball_stats SET statistics_json = '{}' WHERE snapshot_id = ?1 AND player_uid = 77",
            [snapshot_id],
        )
        .expect("corrupt statistics fixture");

        let error = get_player_moneyball(&conn, 77).expect_err("invalid catalogue must fail");
        assert!(error.contains("exact metric catalogue"));
    }
}
