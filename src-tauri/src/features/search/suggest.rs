//! Ctrl+K player-name suggestions, ranked from an in-memory list of the current snapshot.

use std::collections::HashMap;

use rusqlite::{params_from_iter, Connection, OptionalExtension};

use crate::db::result_cache::ResultCache;
use crate::db::Db;

pub const DEFAULT_SUGGEST_LIMIT: usize = 10;
pub const MAX_SUGGEST_LIMIT: usize = 20;

/// The suggestion list of the latest data generation; one fingerprint, so one entry.
pub type SuggestionCache = ResultCache<Option<SuggestionIndex>>;

const SUGGESTION_KEY: &str = "current-snapshot-names";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerSuggestHit {
    pub uid: i64,
    pub name: String,
    pub ca: i64,
}

/// The active save's current-snapshot players, sorted by CA descending then UID.
///
/// Names are ASCII-lowercased, which is the case folding SQLite's `NOCASE` and `LIKE` apply,
/// and stored back to back in one string to keep the list small at a million players.
pub struct SuggestionIndex {
    snapshot_id: i64,
    names: String,
    players: Vec<IndexedPlayer>,
}

struct IndexedPlayer {
    uid: i64,
    ca: i64,
    /// End of this player's name in `names`; it starts where the previous one ends.
    name_end: usize,
}

/// Ranked name matches for the active save's current snapshot: exact, then prefix, then
/// contains, each by CA descending then UID. A blank query matches nothing.
pub fn suggest_players(
    db: &Db,
    cache: &SuggestionCache,
    query: &str,
    limit: usize,
) -> Result<Vec<PlayerSuggestHit>, String> {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let index = cache.get_or_compute(SUGGESTION_KEY, db.generation(), || {
        let reader = db.read()?;
        Ok::<_, String>((reader.generation(), load_index(&reader)?))
    })?;
    let Some(index) = index.as_ref() else {
        return Ok(Vec::new());
    };

    let ranked = index.rank(&query, limit.clamp(1, MAX_SUGGEST_LIMIT));
    if ranked.is_empty() {
        return Ok(Vec::new());
    }
    let mut names = read_names(&*db.read()?, index.snapshot_id, &ranked)?;
    // A name is missing only when its snapshot was deleted after the list was built.
    Ok(ranked
        .into_iter()
        .filter_map(|player| {
            Some(PlayerSuggestHit {
                uid: player.uid,
                name: names.remove(&player.uid)?,
                ca: player.ca,
            })
        })
        .collect())
}

impl SuggestionIndex {
    /// One pass in list order keeps up to `limit` exact, prefix, and contains matches each.
    fn rank(&self, query: &str, limit: usize) -> Vec<&IndexedPlayer> {
        let [mut exact, mut prefix, mut contains] = [(); 3].map(|()| Vec::with_capacity(limit));
        let mut name_start = 0;
        for player in &self.players {
            let name = &self.names[name_start..player.name_end];
            name_start = player.name_end;
            let tier = if name == query {
                &mut exact
            } else if name.starts_with(query) {
                &mut prefix
            } else if name.contains(query) {
                &mut contains
            } else {
                continue;
            };
            if tier.len() < limit {
                tier.push(player);
            }
            if exact.len() == limit {
                break;
            }
        }
        exact.extend(prefix);
        exact.extend(contains);
        exact.truncate(limit);
        exact
    }
}

fn load_index(conn: &Connection) -> Result<Option<SuggestionIndex>, String> {
    let snapshot_id: Option<i64> = conn
        .query_row(
            "SELECT s.id
             FROM snapshots s
             INNER JOIN saves sv ON sv.id = s.save_id AND sv.is_active = 1
             WHERE s.is_current = 1
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let Some(snapshot_id) = snapshot_id else {
        return Ok(None);
    };

    let mut statement = conn
        .prepare(
            "SELECT uid, name, ca FROM players
             WHERE snapshot_id = ?1
             ORDER BY ca DESC, uid ASC",
        )
        .map_err(|error| error.to_string())?;
    let mut rows = statement
        .query([snapshot_id])
        .map_err(|error| error.to_string())?;
    let mut names = String::new();
    let mut players = Vec::new();
    while let Some(row) = rows.next().map_err(|error| error.to_string())? {
        let name = row
            .get_ref(1)
            .and_then(|value| Ok(value.as_str()?))
            .map_err(|error| error.to_string())?;
        names.extend(name.chars().map(|ch| ch.to_ascii_lowercase()));
        players.push(IndexedPlayer {
            uid: row.get(0).map_err(|error| error.to_string())?,
            ca: row.get(2).map_err(|error| error.to_string())?,
            name_end: names.len(),
        });
    }
    names.shrink_to_fit();
    players.shrink_to_fit();
    Ok(Some(SuggestionIndex {
        snapshot_id,
        names,
        players,
    }))
}

fn read_names(
    conn: &Connection,
    snapshot_id: i64,
    players: &[&IndexedPlayer],
) -> Result<HashMap<i64, String>, String> {
    let placeholders = vec!["?"; players.len()].join(", ");
    let sql =
        format!("SELECT uid, name FROM players WHERE snapshot_id = ? AND uid IN ({placeholders})");
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let parameters = std::iter::once(snapshot_id).chain(players.iter().map(|player| player.uid));
    let rows = statement
        .query_map(params_from_iter(parameters), |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<_, _>>()
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::snapshot::ingest::ingest_dump_file;

    /// Adds a copy of the golden player with another UID, name, and CA to the current snapshot.
    fn add_player(db: &Db, uid: i64, name: &str, ca: i64) {
        let writer = db.0.lock().expect("lock writer");
        writer
            .execute_batch(
                "CREATE TEMP TABLE IF NOT EXISTS template AS
                 SELECT * FROM players WHERE uid = 77",
            )
            .expect("copy golden player");
        writer
            .execute(
                "UPDATE template SET uid = ?1, name = ?2, ca = ?3",
                rusqlite::params![uid, name, ca],
            )
            .expect("set player identity");
        writer
            .execute("INSERT INTO players SELECT * FROM template", [])
            .expect("add player");
    }

    fn suggest(
        db: &Db,
        cache: &SuggestionCache,
        query: &str,
        limit: usize,
    ) -> Vec<PlayerSuggestHit> {
        suggest_players(db, cache, query, limit).expect("suggest")
    }

    fn uids(hits: &[PlayerSuggestHit]) -> Vec<i64> {
        hits.iter().map(|hit| hit.uid).collect()
    }

    #[test]
    fn ranks_exact_then_prefix_then_contains_by_ca_then_uid_with_ascii_case_folding() {
        let (_temp_dir, db) = crate::db::open_with_golden_snapshot();
        for (uid, name, ca) in [
            (1, "Alex", 100),
            (3, "ALEX", 160),
            (2, "alex", 160),
            (5, "alexis", 120),
            (4, "Alexander", 120),
            (6, "Sam Alex", 180),
            (7, "Unrelated", 200),
            (8, "ÉLODIE", 150),
            (9, "élodie", 140),
        ] {
            add_player(&db, uid, name, ca);
        }
        let cache = SuggestionCache::new(1);

        let hits = suggest(&db, &cache, "  aLeX ", DEFAULT_SUGGEST_LIMIT);
        assert_eq!(uids(&hits), vec![2, 3, 1, 4, 5, 6]);
        assert_eq!(hits[3].name, "Alexander");
        assert_eq!(
            suggest(&db, &cache, "alex", 1),
            vec![PlayerSuggestHit {
                uid: 2,
                name: "alex".to_string(),
                ca: 160,
            }]
        );

        // Only ASCII letters fold, as in SQLite's NOCASE and LIKE.
        assert_eq!(uids(&suggest(&db, &cache, "élodie", 10)), vec![9]);
        assert_eq!(uids(&suggest(&db, &cache, "ÉLODIE", 10)), vec![8]);
    }

    #[test]
    fn keeps_like_wildcards_literal_and_clamps_the_limit() {
        let (_temp_dir, db) = crate::db::open_with_golden_snapshot();
        for uid in 1..=22 {
            add_player(&db, uid, &format!("Pat {uid}"), 100 + uid);
        }
        add_player(&db, 30, "Pat 100%", 50);
        add_player(&db, 31, "Pat_31", 40);
        let cache = SuggestionCache::new(1);

        let capped = suggest(&db, &cache, "Pat", 2);
        assert_eq!(uids(&capped), vec![22, 21]);
        assert_eq!(capped[0].name, "Pat 22");
        assert_eq!(suggest(&db, &cache, "Pat", 100).len(), MAX_SUGGEST_LIMIT);
        assert_eq!(suggest(&db, &cache, "Pat", 0).len(), 1);

        assert_eq!(uids(&suggest(&db, &cache, "%", 10)), vec![30]);
        assert_eq!(uids(&suggest(&db, &cache, "_", 10)), vec![31]);
        assert!(suggest(&db, &cache, "P%t", 10).is_empty());
        assert!(suggest(&db, &cache, "P_t", 10).is_empty());
    }

    #[test]
    fn a_blank_query_or_no_current_snapshot_suggests_nothing() {
        let (_temp_dir, db) = crate::db::open_with_golden_snapshot();
        let cache = SuggestionCache::new(1);
        assert!(suggest(&db, &cache, "   ", 10).is_empty());

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let empty = crate::db::open(&temp_dir.path().join(crate::db::APP_DB_FILE)).expect("open");
        assert!(suggest(&empty, &SuggestionCache::new(1), "golden", 10).is_empty());
    }

    #[test]
    fn suggestions_follow_the_newest_current_snapshot_after_a_data_change() {
        let (temp_dir, db) = crate::db::open_with_golden_snapshot();
        let cache = SuggestionCache::new(1);
        assert_eq!(uids(&suggest(&db, &cache, "golden", 10)), vec![77]);

        let mut dump: serde_json::Value =
            serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v9.json"))
                .expect("parse golden dump");
        dump["gameDate"] = "2027-08-14".into();
        dump["players"][0]["name"] = "Renamed Fixture Player".into();
        let path = temp_dir.path().join("renamed.json");
        std::fs::write(&path, dump.to_string()).expect("write dump");
        ingest_dump_file(&mut db.0.lock().expect("lock writer"), &path).expect("ingest");

        // The earlier snapshot still holds the old name, but only the current one is listed.
        assert!(suggest(&db, &cache, "golden", 10).is_empty());
        assert_eq!(
            suggest(&db, &cache, "renamed", 10),
            vec![PlayerSuggestHit {
                uid: 77,
                name: "Renamed Fixture Player".to_string(),
                ca: 150,
            }]
        );
    }
}
