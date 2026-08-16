use std::collections::HashSet;

use rusqlite::{params, Connection, OptionalExtension};

pub const MAX_CLUB_NAME_LEN: usize = 120;
const ATTACHED_TEAMS: [&str; 2] = ["reserves", "youth"];
const TEAM_LEVELS: [&str; 3] = ["senior", "reserve", "youth"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClubSourceInput {
    pub team: String,
    pub club_name: String,
    pub team_level: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClubSource {
    pub id: i64,
    pub team: String,
    pub club_name: String,
    pub team_level: Option<String>,
    pub is_primary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClubFamily {
    pub primary_club: Option<String>,
    pub sources: Vec<ClubSource>,
}

pub fn get_club_family(conn: &Connection, save_id: i64) -> Result<ClubFamily, String> {
    let primary_club = conn
        .query_row(
            "SELECT primary_club FROM planner_club_settings WHERE save_id = ?1",
            params![save_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    let mut statement = conn
        .prepare(
            "SELECT id, team, club_name, team_level, is_primary
             FROM planner_club_sources
             WHERE save_id = ?1
             ORDER BY is_primary DESC, team, id",
        )
        .map_err(|error| error.to_string())?;
    let sources = statement
        .query_map(params![save_id], |row| {
            Ok(ClubSource {
                id: row.get(0)?,
                team: row.get(1)?,
                club_name: row.get(2)?,
                team_level: row.get(3)?,
                is_primary: row.get::<_, i32>(4)? == 1,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    Ok(ClubFamily {
        primary_club,
        sources,
    })
}

pub fn list_clubs_for_snapshot(conn: &Connection, save_id: i64) -> Result<Vec<String>, String> {
    let mut statement = conn
        .prepare(
            "SELECT DISTINCT p.current_club
             FROM players p
             INNER JOIN snapshots s ON s.id = p.snapshot_id
             WHERE s.save_id = ?1
               AND s.is_current = 1
               AND p.current_club IS NOT NULL
               AND trim(p.current_club) <> ''
             ORDER BY p.current_club COLLATE NOCASE",
        )
        .map_err(|error| error.to_string())?;
    let clubs = statement
        .query_map(params![save_id], |row| row.get(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string());
    clubs
}

pub fn save_club_family(
    conn: &Connection,
    save_id: i64,
    primary_club: &str,
    attached_sources: &[ClubSourceInput],
) -> Result<ClubFamily, String> {
    let primary_club = validate_club_name(primary_club, "Primary club")?;
    let attached_sources = normalize_sources(attached_sources)?;

    let save_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM saves WHERE id = ?1",
            params![save_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if save_exists == 0 {
        return Err(format!("Save {save_id} not found"));
    }

    validate_club_names_against_snapshot(conn, save_id, &primary_club, &attached_sources)?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    tx.execute(
        "INSERT INTO planner_club_settings (save_id, primary_club)
         VALUES (?1, ?2)
         ON CONFLICT(save_id) DO UPDATE SET primary_club = excluded.primary_club",
        params![save_id, primary_club],
    )
    .map_err(|error| error.to_string())?;
    tx.execute(
        "DELETE FROM planner_club_sources WHERE save_id = ?1",
        params![save_id],
    )
    .map_err(|error| error.to_string())?;

    for (team, team_level) in [
        ("senior", Some("senior")),
        ("reserves", Some("reserve")),
        ("youth", Some("youth")),
    ] {
        insert_source(&tx, save_id, team, &primary_club, team_level, true)?;
    }
    for source in &attached_sources {
        insert_source(
            &tx,
            save_id,
            &source.team,
            &source.club_name,
            source.team_level.as_deref(),
            false,
        )?;
    }

    tx.commit().map_err(|error| error.to_string())?;
    get_club_family(conn, save_id)
}

fn validate_club_names_against_snapshot(
    conn: &Connection,
    save_id: i64,
    primary_club: &str,
    attached_sources: &[ClubSourceInput],
) -> Result<(), String> {
    let has_current_snapshot: bool = conn
        .query_row(
            "SELECT EXISTS(
                 SELECT 1 FROM snapshots WHERE save_id = ?1 AND is_current = 1
             )",
            params![save_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !has_current_snapshot {
        return Err("No current snapshot loaded for this save".to_string());
    }

    let available_clubs = list_clubs_for_snapshot(conn, save_id)?;
    let mut previously_configured = HashSet::new();
    if let Some(existing_primary) = conn
        .query_row(
            "SELECT primary_club FROM planner_club_settings WHERE save_id = ?1",
            params![save_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?
    {
        previously_configured.insert(existing_primary);
    }
    let mut statement = conn
        .prepare("SELECT club_name FROM planner_club_sources WHERE save_id = ?1")
        .map_err(|error| error.to_string())?;
    let source_names = statement
        .query_map(params![save_id], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    previously_configured.extend(source_names);

    let is_available = |club_name: &str| {
        available_clubs
            .iter()
            .any(|available| available == club_name)
            || previously_configured.contains(club_name)
    };
    if !is_available(primary_club) {
        return Err(format!(
            "Primary club `{primary_club}` is not in the current snapshot"
        ));
    }
    if let Some(source) = attached_sources
        .iter()
        .find(|source| !is_available(&source.club_name))
    {
        return Err(format!(
            "Club source `{}` is not in the current snapshot",
            source.club_name
        ));
    }
    Ok(())
}

fn insert_source(
    tx: &rusqlite::Transaction<'_>,
    save_id: i64,
    team: &str,
    club_name: &str,
    team_level: Option<&str>,
    is_primary: bool,
) -> Result<(), String> {
    tx.execute(
        "INSERT INTO planner_club_sources (save_id, team, club_name, team_level, is_primary)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![save_id, team, club_name, team_level, i32::from(is_primary)],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn normalize_sources(sources: &[ClubSourceInput]) -> Result<Vec<ClubSourceInput>, String> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::with_capacity(sources.len());
    for source in sources {
        if !ATTACHED_TEAMS.contains(&source.team.as_str()) {
            return Err(format!("Unknown planner team `{}`", source.team));
        }
        let club_name = validate_club_name(&source.club_name, "Club source")?;
        if let Some(team_level) = &source.team_level {
            if !TEAM_LEVELS.contains(&team_level.as_str()) {
                return Err(format!("Unknown team level `{team_level}`"));
            }
        }
        if !seen.insert((
            source.team.clone(),
            club_name.clone(),
            source.team_level.clone(),
        )) {
            return Err("Planner club sources must be unique".to_string());
        }
        normalized.push(ClubSourceInput {
            team: source.team.clone(),
            club_name,
            team_level: source.team_level.clone(),
        });
    }
    Ok(normalized)
}

fn validate_club_name(value: &str, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} must not be empty"));
    }
    if trimmed.chars().count() > MAX_CLUB_NAME_LEN {
        return Err(format!(
            "{label} must be at most {MAX_CLUB_NAME_LEN} characters"
        ));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::db::migrations;

    #[test]
    fn saves_primary_and_attached_sources_per_save() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = Connection::open(temp_dir.path().join("planner.db")).expect("open db");
        migrations::apply(&conn).expect("apply migrations");

        let default_save_id: i64 = conn
            .query_row(
                "INSERT INTO saves (name, is_active) VALUES ('First save', 1) RETURNING id",
                [],
                |row| row.get(0),
            )
            .expect("create first save");
        let second_save_id: i64 = conn
            .query_row(
                "INSERT INTO saves (name, is_active) VALUES ('Second save', 0) RETURNING id",
                [],
                |row| row.get(0),
            )
            .expect("create second save");

        let dump_path = temp_dir.path().join("dump.json");
        let mut dump: serde_json::Value =
            serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v7.json"))
                .expect("parse golden dump");
        let mut attached_player = dump["players"][0].clone();
        attached_player["uid"] = serde_json::Value::Number(78.into());
        attached_player["name"] = serde_json::Value::String("B Team Player".to_string());
        attached_player["currentClub"] = serde_json::Value::String("Loan B FC".to_string());
        dump["players"]
            .as_array_mut()
            .expect("players array")
            .push(attached_player);
        dump["playerCount"] = serde_json::Value::Number(2.into());
        std::fs::write(
            &dump_path,
            serde_json::to_string(&dump).expect("serialize dump"),
        )
        .expect("write dump");
        crate::features::snapshot::ingest::ingest_dump_file(&mut conn, &dump_path)
            .expect("ingest dump");

        super::save_club_family(
            &conn,
            default_save_id,
            "Loan FC",
            &[super::ClubSourceInput {
                team: "reserves".to_string(),
                club_name: "  Loan B FC  ".to_string(),
                team_level: None,
            }],
        )
        .expect("save first club family");

        let first = super::get_club_family(&conn, default_save_id).expect("load first family");
        assert_eq!(first.primary_club.as_deref(), Some("Loan FC"));
        assert!(first.sources.iter().any(|source| {
            source.club_name == "Loan B FC"
                && source.team == "reserves"
                && source.team_level.is_none()
                && !source.is_primary
        }));

        let second = super::get_club_family(&conn, second_save_id).expect("load second family");
        assert!(second.primary_club.is_none());
        assert!(second.sources.is_empty());
    }

    #[test]
    fn rejects_config_when_current_snapshot_is_missing() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn =
            Connection::open(temp_dir.path().join("planner-no-snapshot.db")).expect("open db");
        migrations::apply(&conn).expect("apply migrations");
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Test save', 1)",
            [],
        )
        .expect("create save");
        let save_id = conn.last_insert_rowid();

        let error = super::save_club_family(&conn, save_id, "Barcelona", &[])
            .expect_err("reject save without snapshot");
        assert!(error.contains("No current snapshot"));
        assert!(super::get_club_family(&conn, save_id)
            .expect("load family")
            .sources
            .is_empty());
    }

    #[test]
    fn rejects_invalid_or_duplicate_attached_sources() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn =
            Connection::open(temp_dir.path().join("planner-validation.db")).expect("open db");
        migrations::apply(&conn).expect("apply migrations");
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Test save', 1)",
            [],
        )
        .expect("create save");
        let save_id = conn.last_insert_rowid();

        let error = super::save_club_family(
            &conn,
            save_id,
            "Barcelona",
            &[super::ClubSourceInput {
                team: "reserve".to_string(),
                club_name: "Barca Athletic".to_string(),
                team_level: None,
            }],
        )
        .expect_err("reject unknown team");
        assert!(error.contains("Unknown planner team"));

        let source = super::ClubSourceInput {
            team: "reserves".to_string(),
            club_name: "Barca Athletic".to_string(),
            team_level: None,
        };
        let error = super::save_club_family(&conn, save_id, "Barcelona", &[source.clone(), source])
            .expect_err("reject duplicate source");
        assert!(error.contains("must be unique"));
    }

    #[test]
    fn lists_distinct_clubs_from_the_current_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = Connection::open(temp_dir.path().join("planner-clubs.db")).expect("open db");
        migrations::apply(&conn).expect("apply migrations");
        crate::features::snapshot::service::list_saves(&conn).expect("seed save");

        let dump_path = temp_dir.path().join("dump.json");
        std::fs::write(
            &dump_path,
            include_str!("../memory_read/fixtures/golden_dump_v7.json"),
        )
        .expect("write dump");
        crate::features::snapshot::ingest::ingest_dump_file(&mut conn, &dump_path)
            .expect("ingest dump");

        let save_id =
            crate::features::snapshot::service::active_save_id(&conn).expect("read active save");
        assert_eq!(
            super::list_clubs_for_snapshot(&conn, save_id).expect("list clubs"),
            ["Loan FC"]
        );
    }

    #[test]
    fn rejects_new_missing_club_mappings_but_preserves_existing_missing_names() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn =
            Connection::open(temp_dir.path().join("planner-club-validation.db")).expect("open db");
        migrations::apply(&conn).expect("apply migrations");
        crate::features::snapshot::service::list_saves(&conn).expect("seed save");
        let save_id =
            crate::features::snapshot::service::active_save_id(&conn).expect("read active save");

        let error = super::save_club_family(&conn, save_id, "Missing FC", &[])
            .expect_err("reject mapping before a snapshot exists");
        assert!(error.contains("No current snapshot"));

        let dump_path = temp_dir.path().join("dump.json");
        let dump = include_str!("../memory_read/fixtures/golden_dump_v7.json");
        std::fs::write(&dump_path, dump).expect("write dump");
        crate::features::snapshot::ingest::ingest_dump_file(&mut conn, &dump_path)
            .expect("ingest dump");

        super::save_club_family(&conn, save_id, "Loan FC", &[])
            .expect("save a club from the first snapshot");

        let mut moved_dump: serde_json::Value =
            serde_json::from_str(dump).expect("parse golden dump");
        for player in moved_dump
            .get_mut("players")
            .and_then(serde_json::Value::as_array_mut)
            .expect("players array")
        {
            player["currentClub"] = serde_json::Value::String("Other FC".to_string());
        }
        let moved_dump_path = temp_dir.path().join("moved-dump.json");
        std::fs::write(
            &moved_dump_path,
            serde_json::to_string(&moved_dump).expect("serialize moved dump"),
        )
        .expect("write moved dump");
        crate::features::snapshot::ingest::ingest_dump_file(&mut conn, &moved_dump_path)
            .expect("replace snapshot");

        super::save_club_family(&conn, save_id, "Loan FC", &[])
            .expect("preserve a previously configured missing mapping");
        let error = super::save_club_family(&conn, save_id, "Unknown FC", &[])
            .expect_err("reject a newly introduced missing primary");
        assert!(error.contains("not in the current snapshot"));
        let error = super::save_club_family(
            &conn,
            save_id,
            "Loan FC",
            &[super::ClubSourceInput {
                team: "reserves".to_string(),
                club_name: "Unknown FC".to_string(),
                team_level: None,
            }],
        )
        .expect_err("reject a newly introduced missing source");
        assert!(error.contains("not in the current snapshot"));
    }
}
