use std::collections::{BTreeMap, BTreeSet};

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;

use super::{
    role_catalog::{builtin_catalog, RolePhase},
    role_score::explain_role,
    MONEYBALL_STATISTIC_KEYS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoneyballProfileState {
    NoData,
    NeedsReimport,
    Ready,
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
            "SELECT asking_price_kind, asking_price_lower_eur, asking_price_upper_eur,
                    starts, substitute_appearances, minutes, statistics_json, percentiles_json
             FROM player_moneyball_stats
             WHERE snapshot_id = ?1 AND player_uid = ?2",
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
    )) = row
    else {
        return Ok(Some(MoneyballProfile::no_data()));
    };
    let Some(percentiles_json) = percentiles_json else {
        return Ok(Some(MoneyballProfile::needs_reimport()));
    };

    let percentiles = parse_percentiles(&percentiles_json)?;
    let role_catalog = builtin_catalog()?;
    let role_scores = role_catalog
        .definitions
        .iter()
        .map(|definition| {
            let details = explain_role(definition, &percentiles).ok_or_else(|| {
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
        .collect::<Result<Vec<_>, String>>()?;

    Ok(Some(MoneyballProfile {
        state: MoneyballProfileState::Ready,
        asking_price_kind,
        asking_price_lower_eur,
        asking_price_upper_eur,
        starts,
        substitute_appearances,
        minutes,
        statistics: Some(parse_statistics(&statistics_json)?),
        percentiles: Some(percentiles),
        role_catalog_version: Some(role_catalog.version),
        role_scores: Some(role_scores),
    }))
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

fn parse_percentiles(json: &str) -> Result<BTreeMap<String, Option<u8>>, String> {
    parse_map(json, "percentiles", |key, value| match value {
        Value::Null => Ok(None),
        Value::Number(number) => number
            .as_u64()
            .filter(|number| *number <= 100)
            .map(|number| Some(number as u8))
            .ok_or_else(|| {
                format!(
                    "Moneyball percentile `{key}` must be an integer from 0 through 100 or null"
                )
            }),
        _ => Err(format!(
            "Moneyball percentile `{key}` must be an integer from 0 through 100 or null"
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

    fn percentiles_with_null(key: &str, value: u8) -> String {
        let mut values = MONEYBALL_STATISTIC_KEYS
            .iter()
            .map(|metric_key| ((*metric_key).to_string(), json!(value)))
            .collect::<Map<_, _>>();
        values.insert(key.to_string(), Value::Null);
        Value::Object(values).to_string()
    }

    fn insert_moneyball_row(conn: &Connection, snapshot_id: i64, percentiles_json: Option<&str>) {
        conn.execute(
            "INSERT INTO player_moneyball_stats (
                snapshot_id, player_uid, asking_price_kind, asking_price_lower_eur,
                asking_price_upper_eur, starts, substitute_appearances, minutes,
                statistics_json, percentiles_json
             ) VALUES (?1, 77, 'single', 12000000, NULL, 18, 3, 1500, ?2, ?3)",
            params![snapshot_id, complete_statistics(2.25), percentiles_json],
        )
        .expect("insert Moneyball row");
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
            Some(83)
        );
        assert_eq!(scored.role_catalog_version, Some(1));
        assert_eq!(scored.role_scores.as_ref().expect("role scores").len(), 88);
        assert!(scored
            .role_scores
            .as_ref()
            .expect("role scores")
            .iter()
            .all(|role| role.score == Some(83)));
    }

    #[test]
    fn identifies_a_null_metric_in_the_role_explanation() {
        let temp_dir = tempfile::tempdir().expect("temp directory");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-role-explanation.db"));
        let snapshot_id = ingest_current_player(&mut conn);
        let catalog = builtin_catalog().expect("built-in catalog");
        let first_metric = catalog.definitions[0].metrics[0].key.clone();
        insert_moneyball_row(
            &conn,
            snapshot_id,
            Some(&percentiles_with_null(&first_metric, 83)),
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
