//! A save's player shortlists, read from its linked FM save file.
//!
//! Each read of that file replaces them. Search and the dashboard match the IDs of a selected
//! shortlist against the current snapshot.

use std::collections::{BTreeMap, BTreeSet};

use rusqlite::{params, Connection};

use crate::features::save_stats::SaveShortlist;

/// The name the game shows for its default shortlist, which the save stores unnamed.
const DEFAULT_SHORTLIST_NAME: &str = "(Default)";

/// Replaces save `save_id`'s shortlists with `shortlists`, and returns how many it now keeps.
/// Run it inside the caller's transaction.
///
/// A list without players is not kept, and lists with one name are stored as one. A stored
/// list keeps its ID while a list of its name is still read, so a selection survives a
/// refresh. Every listed ID is stored, so a later snapshot can match players the current one
/// lacks.
pub(crate) fn replace_save_file_shortlists(
    conn: &Connection,
    save_id: i64,
    shortlists: &[SaveShortlist],
) -> rusqlite::Result<usize> {
    let mut players_by_name: BTreeMap<&str, BTreeSet<i64>> = BTreeMap::new();
    for shortlist in shortlists {
        if shortlist.player_unique_ids.is_empty() {
            continue;
        }
        let name = if shortlist.name.is_empty() {
            DEFAULT_SHORTLIST_NAME
        } else {
            &shortlist.name
        };
        players_by_name
            .entry(name)
            .or_default()
            .extend(shortlist.player_unique_ids.iter().map(|&id| i64::from(id)));
    }

    let mut kept = Vec::with_capacity(players_by_name.len());
    let mut insert_player = conn.prepare(
        "INSERT INTO player_shortlist_entries (shortlist_id, player_uid) VALUES (?1, ?2)",
    )?;
    for (name, players) in &players_by_name {
        let id: i64 = conn.query_row(
            "INSERT INTO player_shortlists (save_id, name) VALUES (?1, ?2)
             ON CONFLICT (save_id, name) DO UPDATE SET name = excluded.name
             RETURNING id",
            params![save_id, name],
            |row| row.get(0),
        )?;
        conn.execute(
            "DELETE FROM player_shortlist_entries WHERE shortlist_id = ?1",
            [id],
        )?;
        for player in players {
            insert_player.execute(params![id, player])?;
        }
        kept.push(id);
    }
    let stored: Vec<i64> = conn
        .prepare("SELECT id FROM player_shortlists WHERE save_id = ?1")?
        .query_map([save_id], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    for id in stored.into_iter().filter(|id| !kept.contains(id)) {
        conn.execute("DELETE FROM player_shortlists WHERE id = ?1", [id])?;
    }
    Ok(kept.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::features::snapshot::service::{active_save_id, create_save};

    /// The ID, name, and players of each of the save's shortlists, by name.
    fn stored_lists(conn: &Connection, save_id: i64) -> Vec<(i64, String, Vec<i64>)> {
        let lists: Vec<(i64, String)> = conn
            .prepare("SELECT id, name FROM player_shortlists WHERE save_id = ?1 ORDER BY name")
            .expect("prepare shortlists")
            .query_map([save_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("query shortlists")
            .collect::<Result<_, _>>()
            .expect("read shortlists");
        let mut players = conn
            .prepare(
                "SELECT player_uid FROM player_shortlist_entries WHERE shortlist_id = ?1
                 ORDER BY player_uid",
            )
            .expect("prepare players");
        lists
            .into_iter()
            .map(|(id, name)| {
                let players = players
                    .query_map([id], |row| row.get(0))
                    .expect("query players")
                    .collect::<Result<_, _>>()
                    .expect("read players");
                (id, name, players)
            })
            .collect()
    }

    fn save_list(name: &str, players: &[u32]) -> SaveShortlist {
        SaveShortlist {
            name: name.to_owned(),
            player_unique_ids: players.to_vec(),
        }
    }

    #[test]
    fn a_read_replaces_the_saves_shortlists_and_keeps_their_ids_by_name() {
        let (_database_directory, db) = db::open_with_golden_snapshot();
        let conn = db.0.lock().expect("lock");
        let save_id = active_save_id(&conn).expect("active save");
        let other_save = create_save(&conn, "Other save").expect("create save").id;
        replace_save_file_shortlists(&conn, other_save, &[save_list("Targets", &[9])])
            .expect("store the other save's list");
        let others = stored_lists(&conn, other_save);

        // The unnamed list is the game's default; a list without players is not kept. A
        // listed player the snapshot lacks is stored too.
        let kept = replace_save_file_shortlists(
            &conn,
            save_id,
            &[
                save_list("Targets", &[12345, 77]),
                save_list("", &[77]),
                save_list("Empty", &[]),
            ],
        )
        .expect("first read");

        assert_eq!(kept, 2);
        let first = stored_lists(&conn, save_id);
        assert_eq!(
            first
                .iter()
                .map(|(_, name, players)| (name.as_str(), players.clone()))
                .collect::<Vec<_>>(),
            [("(Default)", vec![77]), ("Targets", vec![77, 12345])]
        );
        let targets_id = first[1].0;

        // Lists with one name are stored as one, and a list no longer read is removed.
        let kept = replace_save_file_shortlists(
            &conn,
            save_id,
            &[
                save_list("Targets", &[77]),
                save_list("Wingers", &[5, 6]),
                save_list("Wingers", &[6, 7]),
            ],
        )
        .expect("second read");

        assert_eq!(kept, 2);
        assert_eq!(
            stored_lists(&conn, save_id)
                .iter()
                .map(|(id, name, players)| (*id == targets_id, name.as_str(), players.clone()))
                .collect::<Vec<_>>(),
            [
                (true, "Targets", vec![77]),
                (false, "Wingers", vec![5, 6, 7])
            ]
        );

        assert_eq!(
            replace_save_file_shortlists(&conn, save_id, &[]).expect("empty read"),
            0
        );
        assert!(stored_lists(&conn, save_id).is_empty());
        // Another save's lists are never touched.
        assert_eq!(stored_lists(&conn, other_save), others);
    }
}
