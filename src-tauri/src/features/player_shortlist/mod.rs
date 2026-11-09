//! Player shortlists that FM exports as `.fmf` files, linked one per save.
//!
//! `file` reads a shortlist file into its list name and player Unique IDs. Choosing a file
//! links it to the active save and stores every ID it lists, so a later snapshot can match
//! players the current one lacks. Load Data re-reads the linked file after publication.
//! Search matches the IDs against the current snapshot.

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

/// A linked shortlist and how many of its players the current snapshot holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlayerShortlistChoice {
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

/// A rejected choice or removal, which changed nothing. It names the file, never its path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlayerShortlistError {
    pub kind: PlayerShortlistErrorKind,
    pub file_name: Option<String>,
}

/// What a Load Data re-read of the linked shortlist did. Only `Read` changed the stored
/// shortlist. Nothing here carries the linked path; `Failed` names the file only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub(crate) enum PlayerShortlistOutcome {
    /// The name and players were replaced, even when none of them is in the snapshot.
    Read {
        name: String,
        total_players: usize,
        matched_players: usize,
    },
    NotLinked,
    /// The active save, its current snapshot, its context token, or the link changed, so the
    /// read was discarded.
    StaleContext,
    /// `kind` is a reader kind, or `database` for a failed write.
    Failed {
        kind: PlayerShortlistErrorKind,
        file_name: Option<String>,
    },
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

/// Links the shortlist file at `path` to save `save_id`, identified by `context_token`, and
/// replaces its stored players. The file is read without holding the database lock.
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
    let matched_players = persist(&mut conn, &active, path, &shortlist).map_err(reject)?;
    Ok(PlayerShortlistChoice {
        name: shortlist.name,
        file_name,
        total_players: shortlist.player_uids.len(),
        matched_players,
    })
}

/// Unlinks the shortlist file of save `save_id`, identified by `context_token`, and clears
/// its stored players.
pub(crate) fn remove_player_shortlist(
    db: &Db,
    save_id: i64,
    context_token: &str,
) -> Result<(), PlayerShortlistError> {
    let reject = |kind| PlayerShortlistError {
        kind,
        file_name: None,
    };
    let mut conn = db.0.lock().map_err(|_| reject(Database))?;
    let tx = conn.transaction().map_err(|_| reject(Database))?;
    let unlinked = tx
        .execute(
            "UPDATE saves
             SET player_shortlist_file_path = NULL, player_shortlist_name = NULL
             WHERE id = ?1 AND context_token = ?2",
            params![save_id, context_token],
        )
        .map_err(|_| reject(Database))?;
    if unlinked == 0 {
        return Err(reject(StaleContext));
    }
    tx.execute(
        "DELETE FROM player_shortlist_entries WHERE save_id = ?1",
        [save_id],
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

/// Links the file and replaces the save's players in one transaction, but only while the
/// captured save and snapshot are still current and at least one listed player is in that
/// snapshot. Returns how many listed players the snapshot holds.
fn persist(
    conn: &mut Connection,
    active: &ActiveImportContext,
    path: &str,
    shortlist: &PlayerShortlistFile,
) -> Result<usize, PlayerShortlistErrorKind> {
    let tx = conn.transaction().map_err(|_| Database)?;
    revalidate(&tx, active)?;
    let matched_players = count_in_snapshot(&tx, active.snapshot_id, &shortlist.player_uids)?;
    if matched_players == 0 {
        return Err(NoMatchingPlayers);
    }
    store(&tx, active.save_id, path, shortlist)?;
    tx.commit().map_err(|_| Database)?;
    Ok(matched_players)
}

/// Re-reads the shortlist file linked to save `save_id`, identified by `context_token`, and
/// replaces its stored name and players while that save is active and still links the same
/// file. Load Data runs it after publication; the file is read without the database lock.
pub(crate) fn refresh_linked_player_shortlist(
    db: &Db,
    save_id: i64,
    context_token: &str,
) -> PlayerShortlistOutcome {
    let failed = |kind, file_name| PlayerShortlistOutcome::Failed { kind, file_name };
    let (active, linked_path) = {
        let Ok(conn) = db.0.lock() else {
            return failed(Database, None);
        };
        match capture_link(&conn, save_id, context_token) {
            Ok(captured) => captured,
            Err(outcome) => return outcome,
        }
    };
    let shortlist = match read_player_shortlist(Path::new(&linked_path)) {
        Ok(shortlist) => shortlist,
        Err(error) => return failed(error.into(), Some(file_name(&linked_path))),
    };
    let Ok(mut conn) = db.0.lock() else {
        return failed(Database, None);
    };
    match persist_refresh(&mut conn, &active, &linked_path, &shortlist) {
        Ok(matched_players) => PlayerShortlistOutcome::Read {
            total_players: shortlist.player_uids.len(),
            name: shortlist.name,
            matched_players,
        },
        Err(StaleContext) => PlayerShortlistOutcome::StaleContext,
        Err(kind) => failed(kind, None),
    }
}

/// The active save's current snapshot and its linked shortlist path, captured before the
/// file is read.
fn capture_link(
    conn: &Connection,
    save_id: i64,
    context_token: &str,
) -> Result<(ActiveImportContext, String), PlayerShortlistOutcome> {
    let database_failure = || PlayerShortlistOutcome::Failed {
        kind: Database,
        file_name: None,
    };
    let active = match capture(conn, save_id, context_token) {
        Ok(active) => active,
        // Load Data has just published a current snapshot for this save, so its absence
        // means the context changed.
        Err(StaleContext | NoSnapshot) => return Err(PlayerShortlistOutcome::StaleContext),
        Err(_) => return Err(database_failure()),
    };
    let linked_path: Option<String> = conn
        .query_row(
            "SELECT player_shortlist_file_path FROM saves WHERE id = ?1",
            [save_id],
            |row| row.get(0),
        )
        .map_err(|_| database_failure())?;
    let linked_path = linked_path.ok_or(PlayerShortlistOutcome::NotLinked)?;
    Ok((active, linked_path))
}

/// Replaces the save's name and players in one transaction, but only while the captured
/// save and snapshot are still current and the save still links `linked_path`. Returns how
/// many listed players the snapshot holds.
fn persist_refresh(
    conn: &mut Connection,
    active: &ActiveImportContext,
    linked_path: &str,
    shortlist: &PlayerShortlistFile,
) -> Result<usize, PlayerShortlistErrorKind> {
    let tx = conn.transaction().map_err(|_| Database)?;
    revalidate(&tx, active)?;
    // Choosing or removing a file keeps the save's context token, so the link is checked on
    // its own.
    let same_link: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM saves WHERE id = ?1 AND player_shortlist_file_path = ?2)",
            params![active.save_id, linked_path],
            |row| row.get(0),
        )
        .map_err(|_| Database)?;
    if !same_link {
        return Err(StaleContext);
    }
    let matched_players = count_in_snapshot(&tx, active.snapshot_id, &shortlist.player_uids)?;
    store(&tx, active.save_id, linked_path, shortlist)?;
    tx.commit().map_err(|_| Database)?;
    Ok(matched_players)
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

/// Sets the save's link and name and replaces its players with every listed ID.
fn store(
    conn: &Connection,
    save_id: i64,
    path: &str,
    shortlist: &PlayerShortlistFile,
) -> Result<(), PlayerShortlistErrorKind> {
    conn.execute(
        "UPDATE saves SET player_shortlist_file_path = ?1, player_shortlist_name = ?2 WHERE id = ?3",
        params![path, shortlist.name, save_id],
    )
    .map_err(|_| Database)?;
    conn.execute(
        "DELETE FROM player_shortlist_entries WHERE save_id = ?1",
        [save_id],
    )
    .map_err(|_| Database)?;
    let mut insert = conn
        .prepare("INSERT INTO player_shortlist_entries (save_id, player_uid) VALUES (?1, ?2)")
        .map_err(|_| Database)?;
    for uid in &shortlist.player_uids {
        insert
            .execute(params![save_id, uid])
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
    use crate::features::snapshot::service::{list_saves, SaveSummary};

    /// Golden-dump player 77 is in the current snapshot; 12345 is not.
    const IN_SNAPSHOT: u32 = 77;
    const NOT_IN_SNAPSHOT: u32 = 12345;

    type ShortlistState = (Option<String>, Option<String>, Vec<i64>);

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

    impl Fixture {
        fn write(&self, file_name: &str, content: &[u8]) -> String {
            let path: PathBuf = self.files.path().join("Private Folder").join(file_name);
            std::fs::write(&path, content).expect("write shortlist file");
            path.to_str().expect("UTF-8 path").to_string()
        }

        fn choose(&self, path: &str) -> Result<PlayerShortlistChoice, PlayerShortlistError> {
            choose_player_shortlist(&self.db, self.save.id, &self.save.context_token, path)
        }

        fn refresh(&self) -> PlayerShortlistOutcome {
            refresh_linked_player_shortlist(&self.db, self.save.id, &self.save.context_token)
        }

        /// Links `Targets.fmf` holding the in-snapshot player and returns its path.
        fn link_targets(&self) -> String {
            let path = self.write(
                "Targets.fmf",
                &player_shortlist_file("Targets", &[IN_SNAPSHOT]),
            );
            self.choose(&path).expect("link shortlist");
            path
        }

        fn state(&self) -> ShortlistState {
            let conn = self.db.0.lock().expect("lock");
            let (path, name) = conn
                .query_row(
                    "SELECT player_shortlist_file_path, player_shortlist_name FROM saves WHERE id = ?1",
                    [self.save.id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .expect("read link");
            let entries = conn
                .prepare(
                    "SELECT player_uid FROM player_shortlist_entries WHERE save_id = ?1
                     ORDER BY player_uid",
                )
                .expect("prepare entries")
                .query_map([self.save.id], |row| row.get(0))
                .expect("query entries")
                .collect::<Result<_, _>>()
                .expect("read entries");
            (path, name, entries)
        }
    }

    #[test]
    fn choosing_a_file_links_it_and_stores_every_listed_player() {
        let fixture = fixture();
        let path = fixture.write(
            "Targets.fmf",
            &player_shortlist_file("Targets", &[NOT_IN_SNAPSHOT, IN_SNAPSHOT]),
        );

        let choice = fixture.choose(&path).expect("choose shortlist");

        assert_eq!(
            serde_json::to_value(&choice).expect("serialize choice"),
            json!({
                "name": "Targets",
                "fileName": "Targets.fmf",
                "totalPlayers": 2,
                "matchedPlayers": 1,
            })
        );
        assert_eq!(
            fixture.state(),
            (
                Some(path),
                Some("Targets".to_string()),
                vec![i64::from(IN_SNAPSHOT), i64::from(NOT_IN_SNAPSHOT)],
            )
        );
        let save = active_save(&fixture.db);
        assert_eq!(save.player_shortlist_name.as_deref(), Some("Targets"));
        assert_eq!(
            save.player_shortlist_file_name.as_deref(),
            Some("Targets.fmf")
        );
    }

    #[test]
    fn a_rejected_choice_keeps_the_previous_shortlist_and_names_only_the_file() {
        let fixture = fixture();
        let linked = fixture.write(
            "Linked.fmf",
            &player_shortlist_file("Linked", &[IN_SNAPSHOT]),
        );
        fixture
            .choose(&linked)
            .expect("link the previous shortlist");
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

        let rejections = [
            (fixture.choose("Linked.fmf"), "unreadable", "Linked.fmf"),
            (fixture.choose(&not_fm), "not_fm_shortlist", "Notes.fmf"),
            (fixture.choose(&view), "not_player_shortlist", "View.fmf"),
            (
                fixture.choose(&unmatched),
                "no_matching_players",
                "Other Save.fmf",
            ),
            (
                choose_player_shortlist(&fixture.db, fixture.save.id, "stale-token", &linked),
                "stale_context",
                "Linked.fmf",
            ),
        ];

        for (result, kind, file_name) in rejections {
            let error = result.expect_err("choice is rejected");
            assert_eq!(
                serde_json::to_value(&error).expect("serialize error"),
                json!({ "kind": kind, "fileName": file_name })
            );
        }
        assert_eq!(fixture.state(), before);
    }

    #[test]
    fn a_save_without_a_current_snapshot_cannot_choose_a_shortlist() {
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
        assert_eq!(active_save(&db).player_shortlist_file_name, None);
    }

    #[test]
    fn a_read_whose_snapshot_changed_writes_nothing() {
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
        assert_eq!(fixture.state(), (None, None, Vec::new()));
    }

    #[test]
    fn removing_unlinks_the_file_and_clears_the_players_only_for_the_current_context() {
        let fixture = fixture();
        let path = fixture.write(
            "Targets.fmf",
            &player_shortlist_file("Targets", &[IN_SNAPSHOT]),
        );
        fixture.choose(&path).expect("choose shortlist");
        let linked = fixture.state();

        assert_eq!(
            remove_player_shortlist(&fixture.db, fixture.save.id, "stale-token")
                .map_err(|error| error.kind),
            Err(StaleContext)
        );
        assert_eq!(fixture.state(), linked);

        remove_player_shortlist(&fixture.db, fixture.save.id, &fixture.save.context_token)
            .expect("remove shortlist");
        assert_eq!(fixture.state(), (None, None, Vec::new()));
        assert_eq!(active_save(&fixture.db).player_shortlist_name, None);
    }

    #[test]
    fn a_reread_replaces_the_name_and_players_even_when_none_is_in_the_snapshot() {
        let fixture = fixture();
        let path = fixture.link_targets();
        fixture.write(
            "Targets.fmf",
            &player_shortlist_file("Renamed", &[NOT_IN_SNAPSHOT]),
        );

        assert_eq!(
            serde_json::to_value(fixture.refresh()).expect("serialize outcome"),
            json!({
                "status": "read",
                "name": "Renamed",
                "totalPlayers": 1,
                "matchedPlayers": 0,
            })
        );
        assert_eq!(
            fixture.state(),
            (
                Some(path.clone()),
                Some("Renamed".to_string()),
                vec![i64::from(NOT_IN_SNAPSHOT)],
            )
        );

        fixture.write("Targets.fmf", &player_shortlist_file("Emptied", &[]));
        assert_eq!(
            fixture.refresh(),
            PlayerShortlistOutcome::Read {
                name: "Emptied".to_string(),
                total_players: 0,
                matched_players: 0,
            }
        );
        assert_eq!(
            fixture.state(),
            (Some(path), Some("Emptied".to_string()), Vec::new())
        );
    }

    #[test]
    fn a_failed_reread_keeps_the_previous_shortlist_and_names_only_the_file() {
        let fixture = fixture();
        let path = fixture.link_targets();
        let linked = fixture.state();

        fixture.write("Targets.fmf", b"not an FM shortlist");
        let invalid = fixture.refresh();
        std::fs::remove_file(&path).expect("remove linked file");
        let missing = fixture.refresh();

        for (outcome, kind) in [(invalid, "not_fm_shortlist"), (missing, "unreadable")] {
            let serialized = serde_json::to_value(&outcome).expect("serialize outcome");
            assert_eq!(
                serialized,
                json!({ "status": "failed", "kind": kind, "fileName": "Targets.fmf" })
            );
            assert!(!serialized.to_string().contains("Private Folder"));
        }
        assert_eq!(fixture.state(), linked);
    }

    #[test]
    fn a_reread_whose_link_changed_writes_nothing() {
        let fixture = fixture();
        let path = fixture.link_targets();
        let (active, linked_path) = capture_link(
            &fixture.db.0.lock().expect("lock"),
            fixture.save.id,
            &fixture.save.context_token,
        )
        .expect("capture link");
        fixture.write(
            "Targets.fmf",
            &player_shortlist_file("Stale", &[NOT_IN_SNAPSHOT]),
        );
        let shortlist = read_player_shortlist(Path::new(&linked_path)).expect("read shortlist");
        let other = fixture.write("Other.fmf", &player_shortlist_file("Other", &[IN_SNAPSHOT]));
        fixture.choose(&other).expect("choose another file");
        let relinked = fixture.state();

        let mut conn = fixture.db.0.lock().expect("lock");
        assert_eq!(
            persist_refresh(&mut conn, &active, &path, &shortlist),
            Err(StaleContext)
        );
        drop(conn);
        assert_eq!(fixture.state(), relinked);
    }

    #[test]
    fn an_unlinked_or_inactive_save_reads_no_shortlist() {
        let fixture = fixture();

        assert_eq!(fixture.refresh(), PlayerShortlistOutcome::NotLinked);
        fixture.link_targets();
        assert_eq!(
            refresh_linked_player_shortlist(&fixture.db, fixture.save.id, "stale-token"),
            PlayerShortlistOutcome::StaleContext
        );
    }
}
