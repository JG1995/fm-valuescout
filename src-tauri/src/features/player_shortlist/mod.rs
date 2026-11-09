//! Player shortlists that FM exports as `.fmf` files. A save links any number of them.
//!
//! `file` reads a shortlist file into its list name and player Unique IDs. Adding a file
//! links it to the active save, or re-reads it when the save already links that path, and
//! stores every ID it lists, so a later snapshot can match players the current one lacks.
//! Load Data re-reads every linked file after publication. Search matches the IDs of the
//! selected shortlist against the current snapshot.

pub(crate) mod commands;
mod file;
#[cfg(test)]
mod test_support;

use std::path::Path;

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::db::Db;
use crate::features::csv_import::service::{
    capture_active_import_context, revalidate_active_import_context, ActiveImportContext,
    CsvImportServiceError,
};

use file::{read_player_shortlist, PlayerShortlistFile, ShortlistFileError};

/// An added shortlist and how many of its players the current snapshot holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlayerShortlistChoice {
    pub id: i64,
    pub name: String,
    pub file_name: String,
    pub total_players: usize,
    pub matched_players: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PlayerShortlistErrorKind {
    Unreadable,
    TooLarge,
    NotFmShortlist,
    NotPlayerShortlist,
    UnsupportedFormat,
    NoSnapshot,
    NoMatchingPlayers,
    /// The save, its current snapshot, or its context token changed.
    StaleContext,
    Database,
}

/// A rejected add or removal, or a failed Load Data re-read, which changed nothing. It names
/// the file, never its path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlayerShortlistError {
    pub kind: PlayerShortlistErrorKind,
    pub file_name: Option<String>,
}

/// What Load Data's re-read of the save's shortlists did. A re-read discarded because the
/// save or the shortlist changed counts as neither read nor failed. Nothing here carries a
/// linked path.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlayerShortlistsOutcome {
    /// How many shortlists had their name and players replaced.
    pub read: usize,
    /// Each shortlist that kept its previous data. `kind` is a reader kind, or `database`
    /// for a failed write.
    pub failed: Vec<PlayerShortlistError>,
}

use PlayerShortlistErrorKind::{Database, NoMatchingPlayers, NoSnapshot, StaleContext};

impl From<ShortlistFileError> for PlayerShortlistErrorKind {
    fn from(error: ShortlistFileError) -> Self {
        match error {
            ShortlistFileError::Unreadable => Self::Unreadable,
            ShortlistFileError::TooLarge => Self::TooLarge,
            ShortlistFileError::NotFmShortlist => Self::NotFmShortlist,
            ShortlistFileError::NotPlayerShortlist => Self::NotPlayerShortlist,
            ShortlistFileError::UnsupportedFormat => Self::UnsupportedFormat,
        }
    }
}

/// Adds the shortlist file at `path` to save `save_id`, identified by `context_token`, or
/// replaces the name and players of the save's shortlist for that path. The file is read
/// without holding the database lock.
pub(crate) fn choose_player_shortlist(
    db: &Db,
    save_id: i64,
    context_token: &str,
    path: &str,
) -> Result<PlayerShortlistChoice, PlayerShortlistError> {
    let file_name = file_name(path);
    let reject = |kind| PlayerShortlistError {
        kind,
        file_name: Some(file_name.clone()),
    };
    // A relative path cannot name a picked file.
    if !Path::new(path).is_absolute() {
        return Err(reject(PlayerShortlistErrorKind::Unreadable));
    }
    let active = {
        let conn = db.0.lock().map_err(|_| reject(Database))?;
        capture(&conn, save_id, context_token).map_err(reject)?
    };
    let shortlist = read_player_shortlist(Path::new(path)).map_err(|error| reject(error.into()))?;
    let mut conn = db.0.lock().map_err(|_| reject(Database))?;
    let (id, matched_players) = persist(&mut conn, &active, path, &shortlist).map_err(reject)?;
    Ok(PlayerShortlistChoice {
        id,
        name: shortlist.name,
        file_name,
        total_players: shortlist.player_uids.len(),
        matched_players,
    })
}

/// Deletes shortlist `shortlist_id` of save `save_id`, identified by `context_token`, with its
/// players. A shortlist the save does not have is already removed.
pub(crate) fn remove_player_shortlist(
    db: &Db,
    save_id: i64,
    context_token: &str,
    shortlist_id: i64,
) -> Result<(), PlayerShortlistError> {
    let reject = |kind| PlayerShortlistError {
        kind,
        file_name: None,
    };
    let mut conn = db.0.lock().map_err(|_| reject(Database))?;
    let tx = conn.transaction().map_err(|_| reject(Database))?;
    let is_current: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM saves WHERE id = ?1 AND context_token = ?2)",
            params![save_id, context_token],
            |row| row.get(0),
        )
        .map_err(|_| reject(Database))?;
    if !is_current {
        return Err(reject(StaleContext));
    }
    tx.execute(
        "DELETE FROM player_shortlists WHERE id = ?1 AND save_id = ?2",
        params![shortlist_id, save_id],
    )
    .map_err(|_| reject(Database))?;
    tx.commit().map_err(|_| reject(Database))
}

/// The active save's current snapshot, captured before the file is read.
fn capture(
    conn: &Connection,
    save_id: i64,
    context_token: &str,
) -> Result<ActiveImportContext, PlayerShortlistErrorKind> {
    let is_active: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM saves WHERE id = ?1 AND context_token = ?2 AND is_active = 1)",
            params![save_id, context_token],
            |row| row.get(0),
        )
        .map_err(|_| Database)?;
    if !is_active {
        return Err(StaleContext);
    }
    match capture_active_import_context(conn) {
        Ok(active) => Ok(active),
        Err(CsvImportServiceError::NoCurrentSnapshot) => Err(NoSnapshot),
        Err(_) => Err(Database),
    }
}

/// Adds or updates the save's shortlist for `path` in one transaction, but only while the
/// captured save and snapshot are still current and at least one listed player is in that
/// snapshot. Returns the shortlist's ID and how many listed players the snapshot holds.
fn persist(
    conn: &mut Connection,
    active: &ActiveImportContext,
    path: &str,
    shortlist: &PlayerShortlistFile,
) -> Result<(i64, usize), PlayerShortlistErrorKind> {
    let tx = conn.transaction().map_err(|_| Database)?;
    revalidate(&tx, active)?;
    let matched_players = count_in_snapshot(&tx, active.snapshot_id, &shortlist.player_uids)?;
    if matched_players == 0 {
        return Err(NoMatchingPlayers);
    }
    let id = tx
        .query_row(
            "INSERT INTO player_shortlists (save_id, file_path, name) VALUES (?1, ?2, ?3)
             ON CONFLICT (save_id, file_path) DO UPDATE SET name = excluded.name
             RETURNING id",
            params![active.save_id, path, shortlist.name],
            |row| row.get(0),
        )
        .map_err(|_| Database)?;
    store_players(&tx, id, &shortlist.player_uids)?;
    tx.commit().map_err(|_| Database)?;
    Ok((id, matched_players))
}

/// A linked shortlist captured before Load Data re-reads its file.
struct LinkedShortlist {
    id: i64,
    path: String,
}

/// Re-reads every shortlist file of save `save_id`, identified by `context_token`, and
/// replaces each shortlist's name and players in its own transaction while that save is
/// active and still has that shortlist. Load Data runs it after publication; the files are
/// read without the database lock.
pub(crate) fn refresh_player_shortlists(
    db: &Db,
    save_id: i64,
    context_token: &str,
) -> PlayerShortlistsOutcome {
    let mut outcome = PlayerShortlistsOutcome::default();
    let captured = match db.0.lock() {
        Ok(conn) => capture_links(&conn, save_id, context_token),
        Err(_) => Err(Database),
    };
    let (active, linked) = match captured {
        Ok(Some(captured)) => captured,
        Ok(None) => return outcome,
        Err(kind) => {
            outcome.failed.push(PlayerShortlistError {
                kind,
                file_name: None,
            });
            return outcome;
        }
    };
    for LinkedShortlist { id, path } in linked {
        let file_name = file_name(&path);
        let written = match read_player_shortlist(Path::new(&path)) {
            Ok(shortlist) => match db.0.lock() {
                Ok(mut conn) => persist_refresh(&mut conn, &active, id, &path, &shortlist),
                Err(_) => Err(Database),
            },
            Err(error) => Err(error.into()),
        };
        match written {
            Ok(()) => outcome.read += 1,
            Err(StaleContext) => {}
            Err(kind) => outcome.failed.push(PlayerShortlistError {
                kind,
                file_name: Some(file_name),
            }),
        }
    }
    outcome
}

/// The active save's current snapshot and its shortlists, captured before their files are
/// read, or `None` when the save is no longer active.
fn capture_links(
    conn: &Connection,
    save_id: i64,
    context_token: &str,
) -> Result<Option<(ActiveImportContext, Vec<LinkedShortlist>)>, PlayerShortlistErrorKind> {
    let active = match capture(conn, save_id, context_token) {
        Ok(active) => active,
        // Load Data has just published a current snapshot for this save, so its absence
        // means the context changed.
        Err(StaleContext | NoSnapshot) => return Ok(None),
        Err(kind) => return Err(kind),
    };
    let linked = conn
        .prepare("SELECT id, file_path FROM player_shortlists WHERE save_id = ?1 ORDER BY id")
        .and_then(|mut statement| {
            statement
                .query_map([save_id], |row| {
                    Ok(LinkedShortlist {
                        id: row.get(0)?,
                        path: row.get(1)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()
        })
        .map_err(|_| Database)?;
    Ok(Some((active, linked)))
}

/// Replaces shortlist `shortlist_id`'s name and players in one transaction, even when none of
/// them is in the snapshot, but only while the captured save and snapshot are still current
/// and the save still has that shortlist for `path`.
fn persist_refresh(
    conn: &mut Connection,
    active: &ActiveImportContext,
    shortlist_id: i64,
    path: &str,
    shortlist: &PlayerShortlistFile,
) -> Result<(), PlayerShortlistErrorKind> {
    let tx = conn.transaction().map_err(|_| Database)?;
    revalidate(&tx, active)?;
    // Adding or removing a shortlist keeps the save's context token, so the shortlist is
    // checked on its own.
    let renamed = tx
        .execute(
            "UPDATE player_shortlists SET name = ?1
             WHERE id = ?2 AND save_id = ?3 AND file_path = ?4",
            params![shortlist.name, shortlist_id, active.save_id, path],
        )
        .map_err(|_| Database)?;
    if renamed == 0 {
        return Err(StaleContext);
    }
    store_players(&tx, shortlist_id, &shortlist.player_uids)?;
    tx.commit().map_err(|_| Database)
}

fn revalidate(
    conn: &Connection,
    active: &ActiveImportContext,
) -> Result<(), PlayerShortlistErrorKind> {
    match revalidate_active_import_context(conn, active) {
        Ok(()) => Ok(()),
        Err(CsvImportServiceError::StaleContext) => Err(StaleContext),
        Err(_) => Err(Database),
    }
}

fn count_in_snapshot(
    conn: &Connection,
    snapshot_id: i64,
    player_uids: &[i64],
) -> Result<usize, PlayerShortlistErrorKind> {
    let mut in_snapshot = conn
        .prepare("SELECT EXISTS(SELECT 1 FROM players WHERE snapshot_id = ?1 AND uid = ?2)")
        .map_err(|_| Database)?;
    let mut matched_players = 0;
    for uid in player_uids {
        if in_snapshot
            .query_row(params![snapshot_id, uid], |row| row.get::<_, bool>(0))
            .map_err(|_| Database)?
        {
            matched_players += 1;
        }
    }
    Ok(matched_players)
}

/// Replaces the shortlist's players with every listed ID.
fn store_players(
    conn: &Connection,
    shortlist_id: i64,
    player_uids: &[i64],
) -> Result<(), PlayerShortlistErrorKind> {
    conn.execute(
        "DELETE FROM player_shortlist_entries WHERE shortlist_id = ?1",
        [shortlist_id],
    )
    .map_err(|_| Database)?;
    let mut insert = conn
        .prepare("INSERT INTO player_shortlist_entries (shortlist_id, player_uid) VALUES (?1, ?2)")
        .map_err(|_| Database)?;
    for uid in player_uids {
        insert
            .execute(params![shortlist_id, uid])
            .map_err(|_| Database)?;
    }
    Ok(())
}

fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::json;

    use super::test_support::{build_shortlist_file, player_shortlist_file, ShortlistMember};
    use super::*;
    use crate::db;
    use crate::features::snapshot::service::{
        create_save, list_saves, PlayerShortlistSummary, SaveSummary,
    };

    /// Golden-dump player 77 is in the current snapshot; 12345 is not.
    const IN_SNAPSHOT: u32 = 77;
    const NOT_IN_SNAPSHOT: u32 = 12345;

    /// A stored shortlist: its ID, path, name, and players.
    type StoredShortlist = (i64, String, String, Vec<i64>);

    struct Fixture {
        _database_directory: tempfile::TempDir,
        db: Db,
        save: SaveSummary,
        files: tempfile::TempDir,
    }

    fn fixture() -> Fixture {
        let (database_directory, db) = db::open_with_golden_snapshot();
        let save = active_save(&db);
        let files = tempfile::tempdir().expect("temp dir");
        std::fs::create_dir(files.path().join("Private Folder")).expect("create folder");
        Fixture {
            _database_directory: database_directory,
            db,
            save,
            files,
        }
    }

    fn active_save(db: &Db) -> SaveSummary {
        list_saves(&db.0.lock().expect("lock"))
            .expect("list saves")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save")
    }

    fn uids(players: &[u32]) -> Vec<i64> {
        players.iter().map(|&uid| i64::from(uid)).collect()
    }

    impl Fixture {
        fn write(&self, file_name: &str, content: &[u8]) -> String {
            let path: PathBuf = self.files.path().join("Private Folder").join(file_name);
            std::fs::write(&path, content).expect("write shortlist file");
            path.to_str().expect("UTF-8 path").to_string()
        }

        fn choose(&self, path: &str) -> Result<PlayerShortlistChoice, PlayerShortlistError> {
            choose_player_shortlist(&self.db, self.save.id, &self.save.context_token, path)
        }

        fn remove(&self, shortlist_id: i64) -> Result<(), PlayerShortlistErrorKind> {
            remove_player_shortlist(
                &self.db,
                self.save.id,
                &self.save.context_token,
                shortlist_id,
            )
            .map_err(|error| error.kind)
        }

        fn refresh(&self) -> PlayerShortlistsOutcome {
            refresh_player_shortlists(&self.db, self.save.id, &self.save.context_token)
        }

        /// Writes and adds a file holding `players` and returns its path and shortlist ID.
        fn add(&self, file_name: &str, name: &str, players: &[u32]) -> (String, i64) {
            let path = self.write(file_name, &player_shortlist_file(name, players));
            let id = self.choose(&path).expect("add shortlist").id;
            (path, id)
        }

        /// Every stored shortlist of every save, in ID order.
        fn state(&self) -> Vec<StoredShortlist> {
            let conn = self.db.0.lock().expect("lock");
            let shortlists: Vec<(i64, String, String)> = conn
                .prepare("SELECT id, file_path, name FROM player_shortlists ORDER BY id")
                .expect("prepare shortlists")
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .expect("query shortlists")
                .collect::<Result<_, _>>()
                .expect("read shortlists");
            let mut entries = conn
                .prepare(
                    "SELECT player_uid FROM player_shortlist_entries WHERE shortlist_id = ?1
                     ORDER BY player_uid",
                )
                .expect("prepare entries");
            shortlists
                .into_iter()
                .map(|(id, path, name)| {
                    let players = entries
                        .query_map([id], |row| row.get(0))
                        .expect("query entries")
                        .collect::<Result<_, _>>()
                        .expect("read entries");
                    (id, path, name, players)
                })
                .collect()
        }
    }

    #[test]
    fn adding_a_file_adds_a_shortlist_and_stores_every_listed_player() {
        let fixture = fixture();
        let path = fixture.write(
            "Targets.fmf",
            &player_shortlist_file("Targets", &[NOT_IN_SNAPSHOT, IN_SNAPSHOT]),
        );

        let choice = fixture.choose(&path).expect("add shortlist");

        assert_eq!(
            serde_json::to_value(&choice).expect("serialize choice"),
            json!({
                "id": choice.id,
                "name": "Targets",
                "fileName": "Targets.fmf",
                "totalPlayers": 2,
                "matchedPlayers": 1,
            })
        );
        assert_eq!(
            fixture.state(),
            [(
                choice.id,
                path,
                "Targets".to_string(),
                uids(&[IN_SNAPSHOT, NOT_IN_SNAPSHOT]),
            )]
        );
        assert_eq!(
            active_save(&fixture.db).player_shortlists,
            [PlayerShortlistSummary {
                id: choice.id,
                name: "Targets".to_string(),
                file_name: "Targets.fmf".to_string(),
            }]
        );
    }

    #[test]
    fn adding_another_path_adds_a_shortlist_and_adding_a_linked_path_updates_it() {
        let fixture = fixture();
        let (targets, targets_id) = fixture.add("Targets.fmf", "Targets", &[IN_SNAPSHOT]);
        let (loans, loans_id) = fixture.add("Loans.fmf", "Loans", &[IN_SNAPSHOT]);
        assert_ne!(targets_id, loans_id);

        fixture.write(
            "Targets.fmf",
            &player_shortlist_file("Renamed", &[IN_SNAPSHOT, NOT_IN_SNAPSHOT]),
        );
        let readded = fixture.choose(&targets).expect("add the file again");

        assert_eq!(readded.id, targets_id);
        assert_eq!(readded.name, "Renamed");
        assert_eq!(
            fixture.state(),
            [
                (
                    targets_id,
                    targets,
                    "Renamed".to_string(),
                    uids(&[IN_SNAPSHOT, NOT_IN_SNAPSHOT]),
                ),
                (loans_id, loans, "Loans".to_string(), uids(&[IN_SNAPSHOT])),
            ]
        );
    }

    #[test]
    fn a_rejected_add_keeps_every_shortlist_and_names_only_the_file() {
        let fixture = fixture();
        let (linked, _) = fixture.add("Linked.fmf", "Linked", &[IN_SNAPSHOT]);
        fixture.add("Other.fmf", "Other", &[IN_SNAPSHOT]);
        let before = fixture.state();
        let not_fm = fixture.write("Notes.fmf", b"not an FM shortlist");
        let view = fixture.write(
            "View.fmf",
            &build_shortlist_file("View", &[ShortlistMember::new("view", ".vw", b"columns")]),
        );
        let unmatched = fixture.write(
            "Other Save.fmf",
            &player_shortlist_file("Other", &[NOT_IN_SNAPSHOT]),
        );
        // Re-adding a linked path whose file no longer matches the snapshot is rejected too.
        fixture.write(
            "Linked.fmf",
            &player_shortlist_file("Changed", &[NOT_IN_SNAPSHOT]),
        );

        let rejections = [
            (fixture.choose("Linked.fmf"), "unreadable", "Linked.fmf"),
            (fixture.choose(&not_fm), "not_fm_shortlist", "Notes.fmf"),
            (fixture.choose(&view), "not_player_shortlist", "View.fmf"),
            (
                fixture.choose(&unmatched),
                "no_matching_players",
                "Other Save.fmf",
            ),
            (fixture.choose(&linked), "no_matching_players", "Linked.fmf"),
            (
                choose_player_shortlist(&fixture.db, fixture.save.id, "stale-token", &not_fm),
                "stale_context",
                "Notes.fmf",
            ),
        ];

        for (result, kind, file_name) in rejections {
            let error = result.expect_err("add is rejected");
            assert_eq!(
                serde_json::to_value(&error).expect("serialize error"),
                json!({ "kind": kind, "fileName": file_name })
            );
        }
        assert_eq!(fixture.state(), before);
    }

    #[test]
    fn a_save_without_a_current_snapshot_cannot_add_a_shortlist() {
        let directory = tempfile::tempdir().expect("temp dir");
        let db = db::open(&directory.path().join(db::APP_DB_FILE)).expect("open db");
        let save = active_save(&db);
        let path = directory.path().join("Targets.fmf");
        std::fs::write(&path, player_shortlist_file("Targets", &[IN_SNAPSHOT]))
            .expect("write shortlist file");

        let error = choose_player_shortlist(
            &db,
            save.id,
            &save.context_token,
            path.to_str().expect("UTF-8 path"),
        )
        .expect_err("no snapshot");

        assert_eq!(error.kind, NoSnapshot);
        assert!(active_save(&db).player_shortlists.is_empty());
    }

    #[test]
    fn an_add_whose_snapshot_changed_writes_nothing() {
        let fixture = fixture();
        let path = fixture.write(
            "Targets.fmf",
            &player_shortlist_file("Targets", &[IN_SNAPSHOT]),
        );
        let active = capture(
            &fixture.db.0.lock().expect("lock"),
            fixture.save.id,
            &fixture.save.context_token,
        )
        .expect("capture context");
        let shortlist = read_player_shortlist(Path::new(&path)).expect("read shortlist");
        let mut conn = fixture.db.0.lock().expect("lock");
        conn.execute(
            "UPDATE snapshots SET is_current = 0 WHERE id = ?1",
            [active.snapshot_id],
        )
        .expect("demote the captured snapshot");

        assert_eq!(
            persist(&mut conn, &active, &path, &shortlist),
            Err(StaleContext)
        );
        drop(conn);
        assert_eq!(fixture.state(), []);
    }

    #[test]
    fn removing_deletes_only_the_given_shortlist_of_the_current_context() {
        let fixture = fixture();
        let (_, targets_id) = fixture.add("Targets.fmf", "Targets", &[IN_SNAPSHOT]);
        let (loans, loans_id) = fixture.add("Loans.fmf", "Loans", &[IN_SNAPSHOT]);
        let other_save = {
            let conn = fixture.db.0.lock().expect("lock");
            let other = create_save(&conn, "Other save").expect("create save");
            conn.execute(
                "INSERT INTO player_shortlists (save_id, file_path, name)
                 VALUES (?1, '/lists/Elsewhere.fmf', 'Elsewhere')",
                [other.id],
            )
            .expect("add another save's shortlist");
            let id = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO player_shortlist_entries (shortlist_id, player_uid) VALUES (?1, 77)",
                [id],
            )
            .expect("add another save's player");
            id
        };
        let before = fixture.state();

        assert_eq!(
            remove_player_shortlist(&fixture.db, fixture.save.id, "stale-token", targets_id)
                .map_err(|error| error.kind),
            Err(StaleContext)
        );
        assert_eq!(fixture.remove(other_save), Ok(()));
        assert_eq!(fixture.remove(i64::MAX), Ok(()));
        assert_eq!(fixture.state(), before);

        fixture.remove(targets_id).expect("remove shortlist");
        assert_eq!(
            fixture.state(),
            [
                (loans_id, loans, "Loans".to_string(), uids(&[IN_SNAPSHOT])),
                (
                    other_save,
                    "/lists/Elsewhere.fmf".to_string(),
                    "Elsewhere".to_string(),
                    uids(&[IN_SNAPSHOT]),
                ),
            ]
        );
        let entries: i64 = fixture
            .db
            .0
            .lock()
            .expect("lock")
            .query_row(
                "SELECT COUNT(*) FROM player_shortlist_entries WHERE shortlist_id = ?1",
                [targets_id],
                |row| row.get(0),
            )
            .expect("count removed entries");
        assert_eq!(entries, 0);
    }

    #[test]
    fn a_reread_replaces_every_shortlist_even_when_none_of_its_players_is_in_the_snapshot() {
        let fixture = fixture();
        let (targets, targets_id) = fixture.add("Targets.fmf", "Targets", &[IN_SNAPSHOT]);
        let (loans, loans_id) = fixture.add("Loans.fmf", "Loans", &[IN_SNAPSHOT]);
        fixture.write(
            "Targets.fmf",
            &player_shortlist_file("Renamed", &[NOT_IN_SNAPSHOT]),
        );
        fixture.write("Loans.fmf", &player_shortlist_file("Emptied", &[]));

        assert_eq!(
            serde_json::to_value(fixture.refresh()).expect("serialize outcome"),
            json!({ "read": 2, "failed": [] })
        );
        assert_eq!(
            fixture.state(),
            [
                (
                    targets_id,
                    targets,
                    "Renamed".to_string(),
                    uids(&[NOT_IN_SNAPSHOT]),
                ),
                (loans_id, loans, "Emptied".to_string(), Vec::new()),
            ]
        );
    }

    #[test]
    fn a_failed_reread_keeps_that_shortlist_and_still_updates_the_others() {
        let fixture = fixture();
        let (targets, _) = fixture.add("Targets.fmf", "Targets", &[IN_SNAPSHOT]);
        let (_, loans_id) = fixture.add("Loans.fmf", "Loans", &[IN_SNAPSHOT]);
        let (missing, _) = fixture.add("Missing.fmf", "Missing", &[IN_SNAPSHOT]);
        let before = fixture.state();
        fixture.write("Targets.fmf", b"not an FM shortlist");
        std::fs::remove_file(&missing).expect("remove linked file");
        fixture.write(
            "Loans.fmf",
            &player_shortlist_file("Loans", &[NOT_IN_SNAPSHOT]),
        );

        let serialized = serde_json::to_value(fixture.refresh()).expect("serialize outcome");

        assert_eq!(
            serialized,
            json!({
                "read": 1,
                "failed": [
                    { "kind": "not_fm_shortlist", "fileName": "Targets.fmf" },
                    { "kind": "unreadable", "fileName": "Missing.fmf" },
                ],
            })
        );
        assert!(!serialized.to_string().contains("Private Folder"));
        let after = fixture.state();
        assert_eq!(after[0], before[0], "{targets} keeps its players");
        assert_eq!(after[1].0, loans_id);
        assert_eq!(after[1].3, uids(&[NOT_IN_SNAPSHOT]));
        assert_eq!(after[2], before[2]);
    }

    #[test]
    fn a_reread_of_a_shortlist_removed_meanwhile_writes_nothing() {
        let fixture = fixture();
        fixture.add("Targets.fmf", "Targets", &[IN_SNAPSHOT]);
        let (active, linked) = capture_links(
            &fixture.db.0.lock().expect("lock"),
            fixture.save.id,
            &fixture.save.context_token,
        )
        .expect("capture shortlists")
        .expect("active save");
        let LinkedShortlist { id, path } = linked.into_iter().next().expect("one shortlist");
        let shortlist = read_player_shortlist(Path::new(&path)).expect("read shortlist");
        fixture.remove(id).expect("remove shortlist");

        let mut conn = fixture.db.0.lock().expect("lock");
        assert_eq!(
            persist_refresh(&mut conn, &active, id, &path, &shortlist),
            Err(StaleContext)
        );
        drop(conn);
        assert_eq!(fixture.state(), []);
    }

    #[test]
    fn a_save_without_shortlists_or_an_inactive_save_reads_nothing() {
        let fixture = fixture();

        assert_eq!(fixture.refresh(), PlayerShortlistsOutcome::default());
        fixture.add("Targets.fmf", "Targets", &[IN_SNAPSHOT]);
        assert_eq!(
            refresh_player_shortlists(&fixture.db, fixture.save.id, "stale-token"),
            PlayerShortlistsOutcome::default()
        );
    }
}
