use rusqlite::{params, Connection, OptionalExtension};

pub const MAX_CLUB_NAME_LEN: usize = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedClubAvailability {
    Unconfigured,
    Available,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedClubStatus {
    pub club_name: Option<String>,
    pub availability: ManagedClubAvailability,
    pub unclassified_player_count: i64,
}

pub fn get_managed_club(conn: &Connection, save_id: i64) -> Result<ManagedClubStatus, String> {
    let club_name = selected_club(conn, save_id)?;
    let Some(club_name) = club_name else {
        return Ok(ManagedClubStatus {
            club_name: None,
            availability: ManagedClubAvailability::Unconfigured,
            unclassified_player_count: 0,
        });
    };

    let (available, unclassified_player_count): (bool, i64) = conn
        .query_row(
            "SELECT
                 EXISTS(
                     SELECT 1
                     FROM players p
                     INNER JOIN snapshots s ON s.id = p.snapshot_id
                     WHERE s.save_id = ?1
                       AND s.is_current = 1
                       AND p.current_club = ?2
                 ),
                 (
                     SELECT COUNT(*)
                     FROM players p
                     INNER JOIN snapshots s ON s.id = p.snapshot_id
                     WHERE s.save_id = ?1
                       AND s.is_current = 1
                       AND p.current_club = ?2
                       AND (p.team_level IS NULL OR p.team_level NOT IN ('senior', 'reserve', 'youth'))
                 )",
            params![save_id, club_name],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|error| error.to_string())?;

    Ok(ManagedClubStatus {
        club_name: Some(club_name),
        availability: if available {
            ManagedClubAvailability::Available
        } else {
            ManagedClubAvailability::Missing
        },
        unclassified_player_count,
    })
}

pub fn list_managed_club_options(conn: &Connection, save_id: i64) -> Result<Vec<String>, String> {
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
    let options = statement
        .query_map([save_id], |row| row.get(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(options)
}

pub fn set_managed_club(
    conn: &Connection,
    save_id: i64,
    club_name: &str,
) -> Result<ManagedClubStatus, String> {
    let club_name = validate_club_name(club_name)?;
    let save_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM saves WHERE id = ?1)",
            [save_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !save_exists {
        return Err(format!("Save {save_id} not found"));
    }

    let existing = selected_club(conn, save_id)?;
    if existing.as_deref() == Some(club_name.as_str()) {
        return get_managed_club(conn, save_id);
    }

    if !list_managed_club_options(conn, save_id)?
        .iter()
        .any(|option| option == &club_name)
    {
        return Err(format!(
            "Managed club `{club_name}` is not in the current snapshot"
        ));
    }

    conn.execute(
        "INSERT INTO managed_club_settings (save_id, club_name)
         VALUES (?1, ?2)
         ON CONFLICT(save_id) DO UPDATE SET club_name = excluded.club_name",
        params![save_id, club_name],
    )
    .map_err(|error| error.to_string())?;

    get_managed_club(conn, save_id)
}

pub fn selected_club(conn: &Connection, save_id: i64) -> Result<Option<String>, String> {
    conn.query_row(
        "SELECT club_name FROM managed_club_settings WHERE save_id = ?1",
        [save_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|error| error.to_string())
}

fn validate_club_name(club_name: &str) -> Result<String, String> {
    let club_name = club_name.trim();
    if club_name.is_empty() {
        return Err("Managed club must not be empty".to_string());
    }
    if club_name.chars().count() > MAX_CLUB_NAME_LEN {
        return Err(format!(
            "Managed club must be at most {MAX_CLUB_NAME_LEN} characters"
        ));
    }
    Ok(club_name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn connection() -> Connection {
        let conn = Connection::open_in_memory().expect("open database");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    fn insert_save(conn: &Connection, name: &str) -> i64 {
        conn.query_row(
            "INSERT INTO saves (name, is_active) VALUES (?1, 0) RETURNING id",
            [name],
            |row| row.get(0),
        )
        .expect("insert save")
    }

    fn insert_snapshot(conn: &Connection, save_id: i64, is_current: bool) -> i64 {
        conn.query_row(
            "INSERT INTO snapshots (
                 save_id, is_current, schema_version, generated_at_utc,
                 game_version, supported_game_version, bridge_version,
                 protocol_version, game_date_source, scan_truncated,
                 player_count
             ) VALUES (?1, ?2, 8, '2026-08-18T00:00:00Z', '26.3.2',
                       '26.3', '0.4.0', 1, 'inGame', 0, 0)
             RETURNING id",
            params![save_id, i32::from(is_current)],
            |row| row.get(0),
        )
        .expect("insert snapshot")
    }

    fn insert_player(
        conn: &Connection,
        snapshot_id: i64,
        uid: i64,
        club: &str,
        team_level: Option<&str>,
    ) {
        conn.execute(
            "INSERT INTO players (
                 snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                 nationalities_json, preferred_foot, positions_json,
                 attributes_json, hidden_attributes_json, personality_json,
                 current_club, team_level
             ) VALUES (?1, ?2, 100, 120, ?3, 2000, 1, '[]', 'Right', '{}',
                       '{}', '{}', '{}', ?4, ?5)",
            params![snapshot_id, uid, format!("Player {uid}"), club, team_level],
        )
        .expect("insert player");
    }

    #[test]
    fn options_and_status_use_only_the_effective_current_snapshot() {
        let conn = connection();
        let save_id = insert_save(&conn, "Save");
        let retained = insert_snapshot(&conn, save_id, false);
        let current = insert_snapshot(&conn, save_id, true);
        insert_player(&conn, retained, 1, "Earlier FC", Some("senior"));
        insert_player(&conn, current, 2, "Managed FC", Some("senior"));
        insert_player(&conn, current, 3, "Managed FC", None);
        insert_player(&conn, current, 4, "Managed FC", Some("academy"));
        insert_player(&conn, current, 5, "Other FC", Some("reserve"));

        assert_eq!(
            list_managed_club_options(&conn, save_id).expect("list options"),
            ["Managed FC", "Other FC"]
        );
        let status = set_managed_club(&conn, save_id, "Managed FC").expect("set club");
        assert_eq!(status.availability, ManagedClubAvailability::Available);
        assert_eq!(status.unclassified_player_count, 2);
    }

    #[test]
    fn selections_are_save_scoped_and_missing_values_are_retained() {
        let conn = connection();
        let first_save = insert_save(&conn, "First");
        let second_save = insert_save(&conn, "Second");
        let first_snapshot = insert_snapshot(&conn, first_save, true);
        let second_snapshot = insert_snapshot(&conn, second_save, true);
        insert_player(&conn, first_snapshot, 1, "First FC", Some("senior"));
        insert_player(&conn, second_snapshot, 2, "Second FC", Some("senior"));

        set_managed_club(&conn, first_save, "First FC").expect("set first club");
        set_managed_club(&conn, second_save, "Second FC").expect("set second club");
        conn.execute(
            "UPDATE snapshots SET is_current = 0 WHERE id = ?1",
            [first_snapshot],
        )
        .expect("remove current snapshot");

        let missing = get_managed_club(&conn, first_save).expect("get missing club");
        assert_eq!(missing.club_name.as_deref(), Some("First FC"));
        assert_eq!(missing.availability, ManagedClubAvailability::Missing);
        assert_eq!(
            set_managed_club(&conn, first_save, "First FC").expect("keep missing club"),
            missing
        );
        assert!(set_managed_club(&conn, first_save, "Second FC").is_err());
        assert_eq!(
            get_managed_club(&conn, second_save)
                .expect("get second club")
                .club_name
                .as_deref(),
            Some("Second FC")
        );
    }

    #[test]
    fn managed_club_cascades_with_its_save() {
        let conn = connection();
        let save_id = insert_save(&conn, "Save");
        let snapshot_id = insert_snapshot(&conn, save_id, true);
        insert_player(&conn, snapshot_id, 1, "Managed FC", Some("senior"));
        set_managed_club(&conn, save_id, "Managed FC").expect("set club");

        conn.execute("DELETE FROM saves WHERE id = ?1", [save_id])
            .expect("delete save");
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM managed_club_settings", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("count settings"),
            0
        );
    }
}
