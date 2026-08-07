use std::collections::BTreeMap;

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;

const MAX_ACADEMY_CANDIDATES: usize = 100;
const MAX_CANDIDATE_SEARCH_LEN: usize = 120;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademyClass {
    pub id: i64,
    pub class_year: i64,
    pub member_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademyClassDetail {
    pub id: i64,
    pub class_year: i64,
    pub members: Vec<AcademyMember>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademyCandidate {
    pub player_uid: i64,
    pub name: String,
    pub age: Option<i64>,
    pub positions: BTreeMap<String, i64>,
    pub current_club: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcademyMemberState {
    Resolved,
    Departed,
    Unresolved,
}

impl AcademyMemberState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Departed => "departed",
            Self::Unresolved => "unresolved",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademyMember {
    pub player_uid: i64,
    pub last_known_name: String,
    pub current_name: Option<String>,
    pub state: AcademyMemberState,
    pub age: Option<i64>,
    pub nationalities: Vec<String>,
    pub positions: BTreeMap<String, i64>,
    pub current_club: Option<String>,
    pub parent_club: Option<String>,
    pub team_level: Option<String>,
    pub pa: Option<i64>,
    pub determination: Option<i64>,
    pub height_cm: Option<i64>,
    pub preferred_foot: Option<String>,
    pub senior_league_appearances: Option<i64>,
    pub goals: Option<i64>,
    pub assists: Option<i64>,
    pub international_caps: Option<i64>,
    pub sale_fee_gbp: Option<i64>,
    pub is_released: Option<bool>,
    pub is_graduate: Option<bool>,
}

struct CurrentAcademyPlayer {
    name: String,
    age: Option<i64>,
    nationalities: Vec<String>,
    positions: BTreeMap<String, i64>,
    current_club: Option<String>,
    parent_club: Option<String>,
    team_level: Option<String>,
    pa: i64,
    determination: Option<i64>,
    height_cm: Option<i64>,
    preferred_foot: String,
}

pub fn create_class(
    conn: &Connection,
    save_id: i64,
    class_year: i64,
) -> Result<AcademyClass, String> {
    validate_class_year(class_year)?;

    let already_exists: bool = conn
        .query_row(
            "SELECT EXISTS(
                 SELECT 1
                 FROM academy_classes
                 WHERE save_id = ?1 AND class_year = ?2
             )",
            params![save_id, class_year],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if already_exists {
        return Err(format!("Class of {class_year} already exists"));
    }

    conn.execute(
        "INSERT INTO academy_classes (save_id, class_year) VALUES (?1, ?2)",
        params![save_id, class_year],
    )
    .map_err(|error| error.to_string())?;

    Ok(AcademyClass {
        id: conn.last_insert_rowid(),
        class_year,
        member_count: 0,
    })
}

pub fn list_classes(conn: &Connection, save_id: i64) -> Result<Vec<AcademyClass>, String> {
    let mut statement = conn
        .prepare(
            "SELECT class.id, class.class_year, COUNT(member.player_uid)
             FROM academy_classes class
             LEFT JOIN academy_memberships member
               ON member.save_id = class.save_id
              AND member.class_id = class.id
             WHERE class.save_id = ?1
             GROUP BY class.id, class.class_year
             ORDER BY class.class_year DESC",
        )
        .map_err(|error| error.to_string())?;

    let classes = statement
        .query_map(params![save_id], |row| {
            Ok(AcademyClass {
                id: row.get(0)?,
                class_year: row.get(1)?,
                member_count: row.get(2)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    Ok(classes)
}

pub fn delete_class(
    conn: &Connection,
    save_id: i64,
    class_id: i64,
    confirmed: bool,
) -> Result<(), String> {
    if !confirmed {
        return Err("Deleting an academy class requires confirmation".to_string());
    }

    let deleted = conn
        .execute(
            "DELETE FROM academy_classes WHERE id = ?1 AND save_id = ?2",
            params![class_id, save_id],
        )
        .map_err(|error| error.to_string())?;
    if deleted == 0 {
        return Err(format!("Academy class {class_id} not found"));
    }

    Ok(())
}

pub fn get_class(
    conn: &Connection,
    save_id: i64,
    class_id: i64,
) -> Result<AcademyClassDetail, String> {
    let class_year = academy_class_year(conn, save_id, class_id)?;
    let snapshot_id = current_snapshot_id(conn, save_id)?;
    let mut statement = conn
        .prepare(
            "SELECT player_uid, last_known_name
             FROM academy_memberships
             WHERE save_id = ?1 AND class_id = ?2
             ORDER BY last_known_name COLLATE NOCASE, player_uid",
        )
        .map_err(|error| error.to_string())?;
    let memberships = statement
        .query_map(params![save_id, class_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    drop(statement);

    let members = memberships
        .into_iter()
        .map(|(player_uid, last_known_name)| {
            let player = match snapshot_id {
                Some(snapshot_id) => load_current_player(conn, snapshot_id, player_uid)?,
                None => None,
            };
            academy_member(conn, save_id, player_uid, last_known_name, player)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AcademyClassDetail {
        id: class_id,
        class_year,
        members,
    })
}

pub fn list_candidates(
    conn: &Connection,
    save_id: i64,
    search: &str,
) -> Result<Vec<AcademyCandidate>, String> {
    let search = normalize_candidate_search(search)?;
    let snapshot_id = current_snapshot_id(conn, save_id)?
        .ok_or_else(|| "No current snapshot loaded for this save".to_string())?;
    let pattern = format!("%{}%", escape_like(&search));
    let mut statement = conn
        .prepare(
            "SELECT p.uid, p.name, p.age, p.positions_json, p.current_club
             FROM players p
             WHERE p.snapshot_id = ?1
               AND p.name LIKE ?2 ESCAPE '\\' COLLATE NOCASE
               AND EXISTS(
                   SELECT 1
                   FROM planner_club_sources source
                   WHERE source.save_id = ?3
                     AND source.club_name = p.current_club
               )
               AND NOT EXISTS(
                   SELECT 1
                   FROM academy_memberships membership
                   WHERE membership.save_id = ?3
                     AND membership.player_uid = p.uid
               )
             ORDER BY p.name COLLATE NOCASE, p.uid
             LIMIT ?4",
        )
        .map_err(|error| error.to_string())?;
    let candidates = statement
        .query_map(
            params![snapshot_id, pattern, save_id, MAX_ACADEMY_CANDIDATES as i64,],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    candidates
        .into_iter()
        .map(|(player_uid, name, age, positions_json, current_club)| {
            Ok(AcademyCandidate {
                player_uid,
                name,
                age,
                positions: parse_positions(&positions_json)?,
                current_club,
            })
        })
        .collect()
}

pub fn assign_member(
    conn: &Connection,
    save_id: i64,
    class_id: i64,
    player_uid: i64,
) -> Result<(), String> {
    let snapshot_id = current_snapshot_id(conn, save_id)?
        .ok_or_else(|| "No current snapshot loaded for this save".to_string())?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    academy_class_year(&tx, save_id, class_id)?;

    let last_known_name: Option<String> = tx
        .query_row(
            "SELECT p.name
             FROM players p
             WHERE p.snapshot_id = ?1
               AND p.uid = ?2
               AND EXISTS(
                   SELECT 1
                   FROM planner_club_sources source
                   WHERE source.save_id = ?3
                     AND source.club_name = p.current_club
               )",
            params![snapshot_id, player_uid, save_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let Some(last_known_name) = last_known_name else {
        return Err(format!(
            "Player {player_uid} is not an eligible academy candidate"
        ));
    };

    let already_assigned: bool = tx
        .query_row(
            "SELECT EXISTS(
                 SELECT 1
                 FROM academy_memberships
                 WHERE save_id = ?1 AND player_uid = ?2
             )",
            params![save_id, player_uid],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if already_assigned {
        return Err(format!(
            "Player {player_uid} is already assigned to an academy class"
        ));
    }

    tx.execute(
        "INSERT INTO academy_memberships (save_id, class_id, player_uid, last_known_name)
         VALUES (?1, ?2, ?3, ?4)",
        params![save_id, class_id, player_uid, last_known_name],
    )
    .map_err(|error| error.to_string())?;
    tx.commit().map_err(|error| error.to_string())
}

pub fn remove_member(
    conn: &Connection,
    save_id: i64,
    class_id: i64,
    player_uid: i64,
) -> Result<(), String> {
    let removed = conn
        .execute(
            "DELETE FROM academy_memberships
             WHERE save_id = ?1 AND class_id = ?2 AND player_uid = ?3",
            params![save_id, class_id, player_uid],
        )
        .map_err(|error| error.to_string())?;
    if removed == 0 {
        return Err(format!(
            "Player {player_uid} is not assigned to academy class {class_id}"
        ));
    }
    Ok(())
}

fn academy_class_year(conn: &Connection, save_id: i64, class_id: i64) -> Result<i64, String> {
    conn.query_row(
        "SELECT class_year
         FROM academy_classes
         WHERE id = ?1 AND save_id = ?2",
        params![class_id, save_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| format!("Academy class {class_id} not found"))
}

fn current_snapshot_id(conn: &Connection, save_id: i64) -> Result<Option<i64>, String> {
    conn.query_row(
        "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
        params![save_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|error| error.to_string())
}

fn academy_member(
    conn: &Connection,
    save_id: i64,
    player_uid: i64,
    last_known_name: String,
    player: Option<CurrentAcademyPlayer>,
) -> Result<AcademyMember, String> {
    let Some(player) = player else {
        return Ok(AcademyMember {
            player_uid,
            last_known_name,
            current_name: None,
            state: AcademyMemberState::Unresolved,
            age: None,
            nationalities: Vec::new(),
            positions: BTreeMap::new(),
            current_club: None,
            parent_club: None,
            team_level: None,
            pa: None,
            determination: None,
            height_cm: None,
            preferred_foot: None,
            senior_league_appearances: None,
            goals: None,
            assists: None,
            international_caps: None,
            sale_fee_gbp: None,
            is_released: None,
            is_graduate: None,
        });
    };
    let state = if player_is_in_club_family(conn, save_id, player.current_club.as_deref())? {
        AcademyMemberState::Resolved
    } else {
        AcademyMemberState::Departed
    };

    Ok(AcademyMember {
        player_uid,
        last_known_name,
        current_name: Some(player.name),
        state,
        age: player.age,
        nationalities: player.nationalities,
        positions: player.positions,
        current_club: player.current_club,
        parent_club: player.parent_club,
        team_level: player.team_level,
        pa: Some(player.pa),
        determination: player.determination,
        height_cm: player.height_cm,
        preferred_foot: Some(player.preferred_foot),
        senior_league_appearances: None,
        goals: None,
        assists: None,
        international_caps: None,
        sale_fee_gbp: None,
        is_released: None,
        is_graduate: None,
    })
}

fn load_current_player(
    conn: &Connection,
    snapshot_id: i64,
    player_uid: i64,
) -> Result<Option<CurrentAcademyPlayer>, String> {
    let player = conn
        .query_row(
            "SELECT
                name,
                age,
                nationalities_json,
                positions_json,
                current_club,
                parent_club,
                team_level,
                pa,
                attributes_json,
                height_cm,
                preferred_foot
             FROM players
             WHERE snapshot_id = ?1 AND uid = ?2",
            params![snapshot_id, player_uid],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, Option<i64>>(9)?,
                    row.get::<_, String>(10)?,
                ))
            },
        )
        .optional()
        .map_err(|error| error.to_string())?;

    player
        .map(
            |(
                name,
                age,
                nationalities_json,
                positions_json,
                current_club,
                parent_club,
                team_level,
                pa,
                attributes_json,
                height_cm,
                preferred_foot,
            )| {
                Ok(CurrentAcademyPlayer {
                    name,
                    age,
                    nationalities: parse_string_array(&nationalities_json)?,
                    positions: parse_positions(&positions_json)?,
                    current_club,
                    parent_club,
                    team_level,
                    pa,
                    determination: parse_determination(&attributes_json)?,
                    height_cm,
                    preferred_foot,
                })
            },
        )
        .transpose()
}

fn player_is_in_club_family(
    conn: &Connection,
    save_id: i64,
    current_club: Option<&str>,
) -> Result<bool, String> {
    let Some(current_club) = current_club else {
        return Ok(false);
    };
    conn.query_row(
        "SELECT EXISTS(
             SELECT 1
             FROM planner_club_sources
             WHERE save_id = ?1 AND club_name = ?2
         )",
        params![save_id, current_club],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

fn normalize_candidate_search(search: &str) -> Result<String, String> {
    let search = search.trim();
    if search.chars().count() > MAX_CANDIDATE_SEARCH_LEN {
        return Err(format!(
            "Candidate search must be at most {MAX_CANDIDATE_SEARCH_LEN} characters"
        ));
    }
    Ok(search.to_string())
}

fn escape_like(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        if matches!(ch, '\\' | '%' | '_') {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

fn parse_string_array(json: &str) -> Result<Vec<String>, String> {
    let value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let array = value
        .as_array()
        .ok_or_else(|| "nationalities_json must be an array".to_string())?;
    array
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| "nationality must be a string".to_string())
        })
        .collect()
}

fn parse_positions(json: &str) -> Result<BTreeMap<String, i64>, String> {
    let value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let object = value
        .as_object()
        .ok_or_else(|| "positions_json must be an object".to_string())?;
    object
        .iter()
        .map(|(key, value)| {
            value
                .as_i64()
                .map(|value| (key.clone(), value))
                .ok_or_else(|| format!("position `{key}` must be an integer"))
        })
        .collect()
}

fn parse_determination(json: &str) -> Result<Option<i64>, String> {
    let value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let object = value
        .as_object()
        .ok_or_else(|| "attributes_json must be an object".to_string())?;
    match object.get("Determination") {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_i64()
            .map(Some)
            .ok_or_else(|| "Determination must be an integer or null".to_string()),
    }
}

fn validate_class_year(class_year: i64) -> Result<(), String> {
    if class_year <= 0 {
        return Err("Class year must be positive".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use rusqlite::{params, Connection};
    use serde_json::{json, Value};

    use crate::db::migrations;
    use crate::features::planner::service::{self as planner_service, ClubSourceInput};
    use crate::features::snapshot::ingest;

    fn open_migrated(path: &Path) -> Connection {
        let conn = Connection::open(path).expect("open db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    fn insert_save(conn: &Connection, name: &str) -> i64 {
        conn.execute("INSERT INTO saves (name) VALUES (?1)", [name])
            .expect("insert save");
        conn.last_insert_rowid()
    }

    fn fixture_player(uid: i64, name: &str, current_club: &str, team_level: &str) -> Value {
        let mut player: Value =
            serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v5.json"))
                .expect("parse fixture");
        let player = player["players"].get_mut(0).expect("fixture player");
        player["uid"] = json!(uid);
        player["name"] = json!(name);
        player["currentClub"] = json!(current_club);
        player["teamLevel"] = json!(team_level);
        player.clone()
    }

    fn ingest_players(
        temp_dir: &tempfile::TempDir,
        conn: &mut Connection,
        save_id: i64,
        filename: &str,
        players: Vec<Value>,
    ) {
        let mut dump: Value =
            serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v5.json"))
                .expect("parse fixture");
        dump["players"] = Value::Array(players);
        dump["playerCount"] = json!(dump["players"].as_array().expect("players").len());
        let dump_path = temp_dir.path().join(filename);
        fs::write(
            &dump_path,
            serde_json::to_string(&dump).expect("serialize dump"),
        )
        .expect("write dump");
        ingest::ingest_dump_file_for_save(conn, save_id, &dump_path).expect("ingest dump");
    }

    fn configure_club_family(conn: &Connection, save_id: i64) {
        planner_service::save_club_family(
            conn,
            save_id,
            "Loan FC",
            &[ClubSourceInput {
                team: "youth".to_string(),
                club_name: "Loan B FC".to_string(),
                team_level: Some("youth".to_string()),
            }],
        )
        .expect("configure club family");
    }

    #[test]
    fn creates_and_lists_classes_for_one_save() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("academy-service.db")).expect("open db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Academy save', 1)",
            [],
        )
        .expect("insert save");
        let save_id = conn.last_insert_rowid();

        let created = super::create_class(&conn, save_id, 2030).expect("create academy class");
        let invalid_year = super::create_class(&conn, save_id, 0)
            .expect_err("reject a non-positive academy class year");

        assert_eq!(created.class_year, 2030);
        assert_eq!(created.member_count, 0);
        assert_eq!(invalid_year, "Class year must be positive");
        assert_eq!(
            super::list_classes(&conn, save_id).expect("list academy classes"),
            [created]
        );
    }

    #[test]
    fn class_years_are_unique_within_each_save() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn =
            Connection::open(temp_dir.path().join("academy-class-years.db")).expect("open db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn.execute("INSERT INTO saves (name) VALUES ('First save')", [])
            .expect("insert first save");
        let first_save_id = conn.last_insert_rowid();
        conn.execute("INSERT INTO saves (name) VALUES ('Second save')", [])
            .expect("insert second save");
        let second_save_id = conn.last_insert_rowid();

        super::create_class(&conn, first_save_id, 2030).expect("create first class");
        let duplicate = super::create_class(&conn, first_save_id, 2030)
            .expect_err("reject duplicate class year");
        let second_class =
            super::create_class(&conn, second_save_id, 2030).expect("create second class");

        assert_eq!(duplicate, "Class of 2030 already exists");
        assert_eq!(second_class.class_year, 2030);
    }

    #[test]
    fn deleting_a_class_requires_confirmation_and_cascades_memberships() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("academy-delete.db")).expect("open db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn.execute("INSERT INTO saves (name) VALUES ('Academy save')", [])
            .expect("insert save");
        let save_id = conn.last_insert_rowid();
        let academy_class =
            super::create_class(&conn, save_id, 2030).expect("create academy class");
        conn.execute(
            "INSERT INTO academy_memberships (save_id, class_id, player_uid, last_known_name)
             VALUES (?1, ?2, 77, 'Academy Player')",
            params![save_id, academy_class.id],
        )
        .expect("insert academy membership");

        let unconfirmed = super::delete_class(&conn, save_id, academy_class.id, false)
            .expect_err("require class deletion confirmation");
        super::delete_class(&conn, save_id, academy_class.id, true)
            .expect("delete confirmed academy class");

        assert_eq!(
            unconfirmed,
            "Deleting an academy class requires confirmation"
        );
        assert!(super::list_classes(&conn, save_id)
            .expect("list academy classes")
            .is_empty());
        let membership_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM academy_memberships", [], |row| {
                row.get(0)
            })
            .expect("count academy memberships");
        assert_eq!(membership_count, 0);
    }

    #[test]
    fn candidates_follow_the_club_family_and_members_keep_nullable_career_stats() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("academy-candidates.db"));
        let save_id = insert_save(&conn, "Academy save");
        ingest_players(
            &temp_dir,
            &mut conn,
            save_id,
            "academy-candidates.json",
            vec![
                fixture_player(77, "Golden Fixture Player", "Loan FC", "senior"),
                fixture_player(78, "Attached Player", "Loan B FC", "senior"),
                fixture_player(79, "Other Player", "Other FC", "youth"),
            ],
        );
        configure_club_family(&conn, save_id);
        let academy_class =
            super::create_class(&conn, save_id, 2030).expect("create academy class");

        let candidates = super::list_candidates(&conn, save_id, "").expect("list candidates");

        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.player_uid)
                .collect::<Vec<_>>(),
            [78, 77]
        );
        super::assign_member(&conn, save_id, academy_class.id, 77).expect("assign eligible player");
        assert_eq!(
            super::assign_member(&conn, save_id, academy_class.id, 77)
                .expect_err("reject duplicate academy member"),
            "Player 77 is already assigned to an academy class"
        );
        assert_eq!(
            super::assign_member(&conn, save_id, academy_class.id, 79)
                .expect_err("reject unrelated player"),
            "Player 79 is not an eligible academy candidate"
        );

        let detail =
            super::get_class(&conn, save_id, academy_class.id).expect("load academy class");
        let member = &detail.members[0];
        assert_eq!(member.last_known_name, "Golden Fixture Player");
        assert_eq!(
            member.current_name.as_deref(),
            Some("Golden Fixture Player")
        );
        assert_eq!(member.state, super::AcademyMemberState::Resolved);
        assert_eq!(member.nationalities, ["ENG"]);
        assert_eq!(member.determination, None);
        assert_eq!(member.senior_league_appearances, None);
        assert_eq!(member.goals, None);
        assert_eq!(member.assists, None);
        assert_eq!(member.international_caps, None);
        assert_eq!(member.sale_fee_gbp, None);
        assert_eq!(member.is_released, None);
        assert_eq!(member.is_graduate, None);
        assert_eq!(
            super::list_candidates(&conn, save_id, "")
                .expect("list unassigned candidates")
                .iter()
                .map(|candidate| candidate.player_uid)
                .collect::<Vec<_>>(),
            [78]
        );
    }

    #[test]
    fn members_survive_departures_and_snapshot_replacement() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("academy-retention.db"));
        let save_id = insert_save(&conn, "Academy save");
        ingest_players(
            &temp_dir,
            &mut conn,
            save_id,
            "academy-first.json",
            vec![
                fixture_player(77, "Golden Fixture Player", "Loan FC", "senior"),
                fixture_player(78, "Attached Player", "Loan B FC", "senior"),
            ],
        );
        configure_club_family(&conn, save_id);
        let first_class =
            super::create_class(&conn, save_id, 2030).expect("create first academy class");
        let second_class =
            super::create_class(&conn, save_id, 2031).expect("create second academy class");
        super::assign_member(&conn, save_id, first_class.id, 77).expect("assign academy member");
        super::remove_member(&conn, save_id, first_class.id, 77).expect("remove academy member");
        super::assign_member(&conn, save_id, second_class.id, 77).expect("reassign academy member");

        ingest_players(
            &temp_dir,
            &mut conn,
            save_id,
            "academy-departed.json",
            vec![
                fixture_player(77, "Golden Fixture Player", "Other FC", "senior"),
                fixture_player(78, "Attached Player", "Loan B FC", "senior"),
            ],
        );
        let departed =
            super::get_class(&conn, save_id, second_class.id).expect("load departed member");
        assert_eq!(
            departed.members[0].state,
            super::AcademyMemberState::Departed
        );
        assert_eq!(
            departed.members[0].current_name.as_deref(),
            Some("Golden Fixture Player")
        );

        ingest_players(
            &temp_dir,
            &mut conn,
            save_id,
            "academy-replaced.json",
            vec![fixture_player(78, "Attached Player", "Loan B FC", "senior")],
        );
        let unresolved =
            super::get_class(&conn, save_id, second_class.id).expect("load unresolved member");
        assert_eq!(
            unresolved.members[0].state,
            super::AcademyMemberState::Unresolved
        );
        assert_eq!(unresolved.members[0].current_name, None);
        assert_eq!(
            unresolved.members[0].last_known_name,
            "Golden Fixture Player"
        );
    }
}
