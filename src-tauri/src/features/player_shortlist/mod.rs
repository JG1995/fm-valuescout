//! Player shortlists that FM exports as `.fmf` files, linked one per save.
//!
//! `file` reads a shortlist file into its list name and player Unique IDs. Choosing a file
//! links it to the active save and stores every ID it lists, so a later snapshot can match
//! players the current one lacks. Search matches the IDs against the current snapshot.

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
    match revalidate_active_import_context(&tx, active) {
        Ok(()) => {}
        Err(CsvImportServiceError::StaleContext) => return Err(StaleContext),
        Err(_) => return Err(Database),
    }
    let matched_players = {
        let mut in_snapshot = tx
            .prepare("SELECT EXISTS(SELECT 1 FROM players WHERE snapshot_id = ?1 AND uid = ?2)")
            .map_err(|_| Database)?;
        let mut matched_players = 0;
        for uid in &shortlist.player_uids {
            if in_snapshot
                .query_row(params![active.snapshot_id, uid], |row| {
                    row.get::<_, bool>(0)
                })
                .map_err(|_| Database)?
            {
                matched_players += 1;
            }
        }
        matched_players
    };
    if matched_players == 0 {
        return Err(NoMatchingPlayers);
    }

    tx.execute(
        "UPDATE saves SET player_shortlist_file_path = ?1, player_shortlist_name = ?2 WHERE id = ?3",
        params![path, shortlist.name, active.save_id],
    )
    .map_err(|_| Database)?;
    tx.execute(
        "DELETE FROM player_shortlist_entries WHERE save_id = ?1",
        [active.save_id],
    )
    .map_err(|_| Database)?;
    {
        let mut insert = tx
            .prepare("INSERT INTO player_shortlist_entries (save_id, player_uid) VALUES (?1, ?2)")
            .map_err(|_| Database)?;
        for uid in &shortlist.player_uids {
            insert
                .execute(params![active.save_id, uid])
                .map_err(|_| Database)?;
        }
    }
    tx.commit().map_err(|_| Database)?;
    Ok(matched_players)
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
}
