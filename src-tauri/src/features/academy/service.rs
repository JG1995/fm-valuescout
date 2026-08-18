use std::collections::BTreeMap;

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;

const BASELINE_CLASS_YEAR: i64 = 2025;
const MAX_ACADEMY_CANDIDATES: usize = 100;
const MAX_CANDIDATE_SEARCH_LEN: usize = 120;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademyClass {
    pub id: i64,
    pub class_year: i64,
    pub is_automatic: bool,
    pub member_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademyClassDetail {
    pub id: i64,
    pub class_year: i64,
    pub is_automatic: bool,
    pub members: Vec<AcademyMember>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademyCandidate {
    pub player_uid: i64,
    pub name: String,
    pub age: Option<i64>,
    pub positions: BTreeMap<String, Option<i64>>,
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
pub enum AcademyMemberOutcomeStatus {
    Sold,
    Released,
}

impl AcademyMemberOutcomeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sold => "sold",
            Self::Released => "released",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademyMemberOutcome {
    pub status: AcademyMemberOutcomeStatus,
    pub buying_club: Option<String>,
    pub sale_fee_eur: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademyMemberOutcomeInput {
    pub status: String,
    pub buying_club: Option<String>,
    pub sale_fee_eur: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcademyMember {
    pub player_uid: i64,
    pub last_known_name: String,
    pub current_name: Option<String>,
    pub state: AcademyMemberState,
    pub age: Option<i64>,
    pub nationalities: Vec<String>,
    pub positions: BTreeMap<String, Option<i64>>,
    pub current_club: Option<String>,
    pub parent_club: Option<String>,
    pub team_level: Option<String>,
    pub pa: Option<i64>,
    pub determination: Option<i64>,
    pub height_cm: Option<i64>,
    pub preferred_foot: Option<String>,
    pub reported_career_appearances: Option<i64>,
    pub goals: Option<i64>,
    pub assists: Option<i64>,
    pub international_caps: Option<i64>,
    pub outcome: Option<AcademyMemberOutcome>,
    pub is_graduate: Option<bool>,
}

struct CurrentAcademyPlayer {
    name: String,
    age: Option<i64>,
    nationalities: Vec<String>,
    positions: BTreeMap<String, Option<i64>>,
    current_club: Option<String>,
    parent_club: Option<String>,
    team_level: Option<String>,
    pa: i64,
    determination: Option<i64>,
    height_cm: Option<i64>,
    preferred_foot: String,
}

struct AcademyCareerStats {
    reported_career_appearances: Option<i64>,
    goals: Option<i64>,
    assists: Option<i64>,
    international_caps: Option<i64>,
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
        is_automatic: false,
        member_count: 0,
    })
}

pub fn ensure_baseline_class(conn: &Connection, save_id: i64) -> Result<(), String> {
    ensure_automatic_class(conn, save_id, BASELINE_CLASS_YEAR)
}

pub fn ensure_class_for_game_date(
    conn: &Connection,
    save_id: i64,
    game_date: Option<&str>,
    game_date_source: &str,
) -> Result<(), String> {
    let Some(class_year) = observed_class_year(game_date, game_date_source) else {
        return Ok(());
    };

    ensure_automatic_class(conn, save_id, class_year)
}

pub fn list_classes(conn: &Connection, save_id: i64) -> Result<Vec<AcademyClass>, String> {
    let mut statement = conn
        .prepare(
            "SELECT class.id, class.class_year, class.is_automatic, COUNT(member.player_uid)
             FROM academy_classes class
             LEFT JOIN academy_memberships member
               ON member.save_id = class.save_id
              AND member.class_id = class.id
             WHERE class.save_id = ?1
             GROUP BY class.id, class.class_year, class.is_automatic
             ORDER BY class.class_year ASC",
        )
        .map_err(|error| error.to_string())?;

    let classes = statement
        .query_map(params![save_id], |row| {
            Ok(AcademyClass {
                id: row.get(0)?,
                class_year: row.get(1)?,
                is_automatic: row.get::<_, i32>(2)? == 1,
                member_count: row.get(3)?,
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

    let is_automatic: Option<bool> = conn
        .query_row(
            "SELECT is_automatic FROM academy_classes WHERE id = ?1 AND save_id = ?2",
            params![class_id, save_id],
            |row| Ok(row.get::<_, i32>(0)? == 1),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    match is_automatic {
        Some(true) => {
            return Err("Automatically managed academy classes cannot be deleted".to_string())
        }
        Some(false) => {}
        None => return Err(format!("Academy class {class_id} not found")),
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
    let (class_year, is_automatic) = academy_class(conn, save_id, class_id)?;
    let snapshot_id = current_snapshot_id(conn, save_id)?;
    let mut statement = conn
        .prepare(
            "SELECT membership.player_uid,
                    membership.last_known_name,
                    outcome.status,
                    outcome.buying_club,
                    outcome.sale_fee_eur,
                    career.career_appearances,
                    career.career_goals,
                    career.career_assists,
                    career.international_caps
             FROM academy_memberships membership
             LEFT JOIN academy_member_outcomes outcome
               ON outcome.save_id = membership.save_id
              AND outcome.player_uid = membership.player_uid
             LEFT JOIN player_youth_career_stats career
               ON career.save_id = membership.save_id
              AND career.player_uid = membership.player_uid
             WHERE membership.save_id = ?1 AND membership.class_id = ?2
             ORDER BY membership.last_known_name COLLATE NOCASE, membership.player_uid",
        )
        .map_err(|error| error.to_string())?;
    let memberships = statement
        .query_map(params![save_id, class_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<i64>>(4)?,
                AcademyCareerStats {
                    reported_career_appearances: row.get(5)?,
                    goals: row.get(6)?,
                    assists: row.get(7)?,
                    international_caps: row.get(8)?,
                },
            ))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    drop(statement);

    let members = memberships
        .into_iter()
        .map(
            |(player_uid, last_known_name, status, buying_club, sale_fee_eur, career_stats)| {
                let player = match snapshot_id {
                    Some(snapshot_id) => load_current_player(conn, snapshot_id, player_uid)?,
                    None => None,
                };
                let outcome = stored_member_outcome(status, buying_club, sale_fee_eur)?;
                academy_member(
                    conn,
                    save_id,
                    player_uid,
                    last_known_name,
                    player,
                    career_stats,
                    outcome,
                )
            },
        )
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AcademyClassDetail {
        id: class_id,
        class_year,
        is_automatic,
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
               AND p.current_club = (
                   SELECT club_name FROM managed_club_settings WHERE save_id = ?3
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
    academy_class(&tx, save_id, class_id)?;

    let last_known_name: Option<String> = tx
        .query_row(
            "SELECT p.name
             FROM players p
             WHERE p.snapshot_id = ?1
               AND p.uid = ?2
               AND p.current_club = (
                   SELECT club_name FROM managed_club_settings WHERE save_id = ?3
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

pub fn set_member_outcome(
    conn: &Connection,
    save_id: i64,
    class_id: i64,
    player_uid: i64,
    outcome: Option<AcademyMemberOutcomeInput>,
) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let is_member: bool = tx
        .query_row(
            "SELECT EXISTS(
                 SELECT 1
                 FROM academy_memberships
                 WHERE save_id = ?1 AND class_id = ?2 AND player_uid = ?3
             )",
            params![save_id, class_id, player_uid],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !is_member {
        return Err(format!(
            "Player {player_uid} is not assigned to academy class {class_id}"
        ));
    }

    match outcome {
        Some(outcome) => {
            let outcome = validate_member_outcome(outcome)?;
            tx.execute(
                "INSERT INTO academy_member_outcomes (
                    save_id,
                    player_uid,
                    status,
                    buying_club,
                    sale_fee_eur
                 ) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(save_id, player_uid) DO UPDATE SET
                    status = excluded.status,
                    buying_club = excluded.buying_club,
                    sale_fee_eur = excluded.sale_fee_eur",
                params![
                    save_id,
                    player_uid,
                    outcome.status.as_str(),
                    outcome.buying_club,
                    outcome.sale_fee_eur,
                ],
            )
            .map_err(|error| error.to_string())?;
        }
        None => {
            tx.execute(
                "DELETE FROM academy_member_outcomes WHERE save_id = ?1 AND player_uid = ?2",
                params![save_id, player_uid],
            )
            .map_err(|error| error.to_string())?;
        }
    }

    tx.commit().map_err(|error| error.to_string())
}

fn academy_class(conn: &Connection, save_id: i64, class_id: i64) -> Result<(i64, bool), String> {
    conn.query_row(
        "SELECT class_year, is_automatic
         FROM academy_classes
         WHERE id = ?1 AND save_id = ?2",
        params![class_id, save_id],
        |row| Ok((row.get(0)?, row.get::<_, i32>(1)? == 1)),
    )
    .optional()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| format!("Academy class {class_id} not found"))
}

fn ensure_automatic_class(conn: &Connection, save_id: i64, class_year: i64) -> Result<(), String> {
    conn.execute(
        "INSERT INTO academy_classes (save_id, class_year, is_automatic)
         VALUES (?1, ?2, 1)
         ON CONFLICT(save_id, class_year) DO UPDATE SET is_automatic = 1",
        params![save_id, class_year],
    )
    .map_err(|error| error.to_string())?;

    Ok(())
}

fn observed_class_year(game_date: Option<&str>, game_date_source: &str) -> Option<i64> {
    if !matches!(game_date_source, "memory" | "derived") {
        return None;
    }

    let game_date = game_date?;
    let bytes = game_date.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }

    let year = game_date[0..4].parse::<i64>().ok()?;
    let month = game_date[5..7].parse::<u8>().ok()?;
    let day = game_date[8..10].parse::<u8>().ok()?;
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => return None,
    };
    if !(1..=days_in_month).contains(&day) || year < BASELINE_CLASS_YEAR {
        return None;
    }

    Some(year)
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
    career_stats: AcademyCareerStats,
    outcome: Option<AcademyMemberOutcome>,
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
            reported_career_appearances: career_stats.reported_career_appearances,
            goals: career_stats.goals,
            assists: career_stats.assists,
            international_caps: career_stats.international_caps,
            outcome,
            is_graduate: career_stats
                .reported_career_appearances
                .map(|appearances| appearances >= 1),
        });
    };
    let state = if player_is_in_managed_club(conn, save_id, player.current_club.as_deref())? {
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
        reported_career_appearances: career_stats.reported_career_appearances,
        goals: career_stats.goals,
        assists: career_stats.assists,
        international_caps: career_stats.international_caps,
        outcome,
        is_graduate: career_stats
            .reported_career_appearances
            .map(|appearances| appearances >= 1),
    })
}

fn validate_member_outcome(
    input: AcademyMemberOutcomeInput,
) -> Result<AcademyMemberOutcome, String> {
    match input.status.as_str() {
        "sold" => {
            let buying_club = input
                .buying_club
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "Sale outcomes require a buying club".to_string())?;
            let sale_fee_eur = input
                .sale_fee_eur
                .filter(|value| *value >= 0)
                .ok_or_else(|| "Sale outcomes require a non-negative whole-euro fee".to_string())?;
            Ok(AcademyMemberOutcome {
                status: AcademyMemberOutcomeStatus::Sold,
                buying_club: Some(buying_club),
                sale_fee_eur: Some(sale_fee_eur),
            })
        }
        "released" => {
            if input.buying_club.is_some() || input.sale_fee_eur.is_some() {
                return Err("Released outcomes cannot include sale details".to_string());
            }
            Ok(AcademyMemberOutcome {
                status: AcademyMemberOutcomeStatus::Released,
                buying_club: None,
                sale_fee_eur: None,
            })
        }
        _ => Err(format!("Unknown academy outcome status `{}`", input.status)),
    }
}

fn stored_member_outcome(
    status: Option<String>,
    buying_club: Option<String>,
    sale_fee_eur: Option<i64>,
) -> Result<Option<AcademyMemberOutcome>, String> {
    let Some(status) = status else {
        return Ok(None);
    };
    validate_member_outcome(AcademyMemberOutcomeInput {
        status,
        buying_club,
        sale_fee_eur,
    })
    .map(Some)
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

fn player_is_in_managed_club(
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
             FROM managed_club_settings
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

fn parse_positions(json: &str) -> Result<BTreeMap<String, Option<i64>>, String> {
    let value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let object = value
        .as_object()
        .ok_or_else(|| "positions_json must be an object".to_string())?;
    object
        .iter()
        .map(|(key, value)| {
            let familiarity = match value {
                Value::Null => None,
                Value::Number(number) => Some(
                    number
                        .as_i64()
                        .ok_or_else(|| format!("position `{key}` must be an integer or null"))?,
                ),
                _ => return Err(format!("position `{key}` must be an integer or null")),
            };
            Ok((key.clone(), familiarity))
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
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::Path;

    use rusqlite::{params, Connection};
    use serde_json::{json, Value};

    use crate::db::migrations;
    use crate::features::managed_club::service as managed_club_service;
    use crate::features::snapshot::ingest;
    use crate::features::snapshot::service as snapshot_service;

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
            serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v8.json"))
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
            serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v8.json"))
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

    fn ingest_with_game_date(
        temp_dir: &tempfile::TempDir,
        conn: &mut Connection,
        save_id: i64,
        filename: &str,
        game_date: Value,
        game_date_source: &str,
    ) -> Result<(), String> {
        let mut dump: Value =
            serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v8.json"))
                .expect("parse fixture");
        dump["gameDate"] = game_date;
        dump["gameDateSource"] = json!(game_date_source);
        let dump_path = temp_dir.path().join(filename);
        fs::write(
            &dump_path,
            serde_json::to_string(&dump).expect("serialize dump"),
        )
        .expect("write dump");
        ingest::ingest_dump_file_for_save(conn, save_id, &dump_path).map(|_| ())
    }

    #[test]
    fn position_parser_preserves_zero_and_unread_values() {
        assert_eq!(
            super::parse_positions(r#"{"AMR":20,"GK":0,"SW":null}"#).expect("positions"),
            BTreeMap::from([
                ("AMR".to_string(), Some(20)),
                ("GK".to_string(), Some(0)),
                ("SW".to_string(), None),
            ])
        );
    }

    fn configure_managed_club(conn: &Connection, save_id: i64) {
        managed_club_service::set_managed_club(conn, save_id, "Loan FC")
            .expect("configure managed club");
    }

    fn insert_membership(conn: &Connection, save_id: i64, class_id: i64, player_uid: i64) {
        conn.execute(
            "INSERT INTO academy_memberships (save_id, class_id, player_uid, last_known_name)
             VALUES (?1, ?2, ?3, 'Academy Player')",
            params![save_id, class_id, player_uid],
        )
        .expect("insert academy membership");
    }

    fn insert_youth_career_stats(
        conn: &Connection,
        save_id: i64,
        player_uid: i64,
        career_appearances: i64,
        international_caps: i64,
        career_goals: i64,
        career_assists: i64,
    ) {
        conn.execute(
            "INSERT INTO player_youth_career_stats (
                save_id,
                player_uid,
                career_appearances,
                international_caps,
                career_goals,
                career_assists
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                save_id,
                player_uid,
                career_appearances,
                international_caps,
                career_goals,
                career_assists,
            ],
        )
        .expect("insert youth career stats");
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
    fn generates_automatic_classes_without_replacing_matching_manual_memberships() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("academy-automatic-classes.db"));
        let save = snapshot_service::create_save(&conn, "Academy save").expect("create save");
        let default_save = snapshot_service::list_saves(&conn)
            .expect("list saves")
            .into_iter()
            .find(|candidate| candidate.is_active)
            .expect("default save");

        assert_eq!(
            super::list_classes(&conn, save.id)
                .expect("list baseline classes")
                .into_iter()
                .map(|academy_class| academy_class.class_year)
                .collect::<Vec<_>>(),
            vec![2025]
        );

        let manual = super::create_class(&conn, save.id, 2026).expect("create manual class");
        conn.execute(
            "INSERT INTO academy_memberships (save_id, class_id, player_uid, last_known_name)
             VALUES (?1, ?2, 77, 'Manual class player')",
            params![save.id, manual.id],
        )
        .expect("assign manual class player");

        ingest_players(
            &temp_dir,
            &mut conn,
            save.id,
            "automatic-2026.json",
            vec![fixture_player(78, "Academy prospect", "Loan FC", "youth")],
        );
        ingest_players(
            &temp_dir,
            &mut conn,
            save.id,
            "automatic-2026-retry.json",
            vec![fixture_player(79, "Academy retry", "Loan FC", "youth")],
        );

        let classes = super::list_classes(&conn, save.id).expect("list automatic classes");
        assert_eq!(
            classes
                .iter()
                .map(|academy_class| academy_class.class_year)
                .collect::<Vec<_>>(),
            vec![2025, 2026]
        );
        let matching_class = classes
            .iter()
            .find(|academy_class| academy_class.class_year == 2026)
            .expect("matching 2026 class");
        assert_eq!(matching_class.id, manual.id);
        assert!(matching_class.is_automatic);
        assert_eq!(matching_class.member_count, 1);
        let is_automatic: i32 = conn
            .query_row(
                "SELECT is_automatic FROM academy_classes WHERE id = ?1",
                [manual.id],
                |row| row.get(0),
            )
            .expect("read matching class marker");
        assert_eq!(is_automatic, 1);
        assert_eq!(
            super::list_classes(&conn, default_save.id)
                .expect("list default-save classes")
                .into_iter()
                .map(|academy_class| academy_class.class_year)
                .collect::<Vec<_>>(),
            vec![2025]
        );
    }

    #[test]
    fn derived_game_date_creates_the_same_observed_year_class_at_a_year_boundary() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("academy-derived-date.db"));
        let save = snapshot_service::create_save(&conn, "Academy save").expect("create save");

        ingest_with_game_date(
            &temp_dir,
            &mut conn,
            save.id,
            "derived-date.json",
            json!("2026-01-01"),
            "derived",
        )
        .expect("ingest derived date");

        assert_eq!(
            super::list_classes(&conn, save.id)
                .expect("list classes")
                .into_iter()
                .map(|academy_class| academy_class.class_year)
                .collect::<Vec<_>>(),
            vec![2025, 2026]
        );
    }

    #[test]
    fn automatic_classes_reject_deletion_while_manual_classes_remain_deletable() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("academy-automatic-delete.db"));
        let save = snapshot_service::create_save(&conn, "Academy save").expect("create save");
        let baseline = super::list_classes(&conn, save.id)
            .expect("list baseline")
            .into_iter()
            .find(|academy_class| academy_class.class_year == 2025)
            .expect("baseline class");

        let automatic_error = super::delete_class(&conn, save.id, baseline.id, true)
            .expect_err("reject automatic class deletion");
        assert_eq!(
            automatic_error,
            "Automatically managed academy classes cannot be deleted"
        );

        let custom = super::create_class(&conn, save.id, 2027).expect("create custom class");
        super::delete_class(&conn, save.id, custom.id, true).expect("delete custom class");
    }

    #[test]
    fn unknown_early_invalid_and_failed_snapshots_do_not_create_observed_classes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("academy-unavailable-date.db"));
        let save = snapshot_service::create_save(&conn, "Academy save").expect("create save");

        ingest_with_game_date(
            &temp_dir,
            &mut conn,
            save.id,
            "unknown-date.json",
            Value::Null,
            "unknown",
        )
        .expect("ingest unknown date");
        let malformed = ingest_with_game_date(
            &temp_dir,
            &mut conn,
            save.id,
            "malformed-date.json",
            json!("not-a-date"),
            "memory",
        );
        assert!(malformed.is_err());
        ingest_with_game_date(
            &temp_dir,
            &mut conn,
            save.id,
            "early-date.json",
            json!("2024-12-31"),
            "memory",
        )
        .expect("ingest early date");
        let failed_json = include_str!("../memory_read/fixtures/golden_dump_v8.json")
            .replace("\"schemaVersion\": 8", "\"schemaVersion\": 4")
            .replace(
                "\"gameDate\": \"2026-08-14\"",
                "\"gameDate\": \"2027-01-01\"",
            );
        let failed_path = temp_dir.path().join("failed-date.json");
        fs::write(&failed_path, failed_json).expect("write failed dump");
        let failed = ingest::ingest_dump_file_for_save(&mut conn, save.id, &failed_path);

        assert!(failed.is_err());
        assert_eq!(
            super::list_classes(&conn, save.id)
                .expect("list classes after unavailable dates")
                .into_iter()
                .map(|academy_class| academy_class.class_year)
                .collect::<Vec<_>>(),
            vec![2025]
        );
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
    fn candidates_follow_the_managed_club_and_members_keep_nullable_career_stats() {
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
        configure_managed_club(&conn, save_id);
        let academy_class =
            super::create_class(&conn, save_id, 2030).expect("create academy class");

        let candidates = super::list_candidates(&conn, save_id, "").expect("list candidates");

        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.player_uid)
                .collect::<Vec<_>>(),
            [77]
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
        assert_eq!(member.reported_career_appearances, None);
        assert_eq!(member.goals, None);
        assert_eq!(member.assists, None);
        assert_eq!(member.international_caps, None);
        assert_eq!(member.outcome, None);
        assert_eq!(member.is_graduate, None);
        assert!(super::list_candidates(&conn, save_id, "")
            .expect("list unassigned candidates")
            .is_empty());
    }

    #[test]
    fn members_keep_save_scoped_youth_career_stats_after_snapshot_replacement() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("academy-career-stats.db"));
        let first_save_id = insert_save(&conn, "First academy save");
        let second_save_id = insert_save(&conn, "Second academy save");
        ingest_players(
            &temp_dir,
            &mut conn,
            first_save_id,
            "academy-career-stats-first.json",
            vec![
                fixture_player(77, "Golden Fixture Player", "Loan FC", "senior"),
                fixture_player(78, "Attached Player", "Loan B FC", "senior"),
            ],
        );
        configure_managed_club(&conn, first_save_id);
        let first_class =
            super::create_class(&conn, first_save_id, 2030).expect("create first academy class");
        super::assign_member(&conn, first_save_id, first_class.id, 77)
            .expect("assign first academy member");
        let second_class =
            super::create_class(&conn, second_save_id, 2030).expect("create second academy class");
        insert_membership(&conn, second_save_id, second_class.id, 77);
        insert_youth_career_stats(&conn, first_save_id, 77, 4, 3, 2, 1);
        insert_youth_career_stats(&conn, second_save_id, 77, 0, 98, 97, 96);

        let first_member = &super::get_class(&conn, first_save_id, first_class.id)
            .expect("load first academy class")
            .members[0];
        assert_eq!(first_member.state, super::AcademyMemberState::Resolved);
        assert_eq!(first_member.reported_career_appearances, Some(4));
        assert_eq!(first_member.goals, Some(2));
        assert_eq!(first_member.assists, Some(1));
        assert_eq!(first_member.international_caps, Some(3));
        assert_eq!(first_member.is_graduate, Some(true));

        let second_member = &super::get_class(&conn, second_save_id, second_class.id)
            .expect("load second academy class")
            .members[0];
        assert_eq!(second_member.state, super::AcademyMemberState::Unresolved);
        assert_eq!(second_member.reported_career_appearances, Some(0));
        assert_eq!(second_member.goals, Some(97));
        assert_eq!(second_member.assists, Some(96));
        assert_eq!(second_member.international_caps, Some(98));
        assert_eq!(second_member.is_graduate, Some(false));

        ingest_players(
            &temp_dir,
            &mut conn,
            first_save_id,
            "academy-career-stats-replaced.json",
            vec![fixture_player(
                78,
                "Replacement Player",
                "Loan FC",
                "senior",
            )],
        );
        let unresolved_member = &super::get_class(&conn, first_save_id, first_class.id)
            .expect("load unresolved academy member")
            .members[0];
        assert_eq!(
            unresolved_member.state,
            super::AcademyMemberState::Unresolved
        );
        assert_eq!(unresolved_member.reported_career_appearances, Some(4));
        assert_eq!(unresolved_member.goals, Some(2));
        assert_eq!(unresolved_member.assists, Some(1));
        assert_eq!(unresolved_member.international_caps, Some(3));
        assert_eq!(unresolved_member.is_graduate, Some(true));
    }

    #[test]
    fn records_replaces_and_clears_member_outcomes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("academy-outcomes.db"));
        let save_id = insert_save(&conn, "Academy save");
        let academy_class =
            super::create_class(&conn, save_id, 2030).expect("create academy class");
        insert_membership(&conn, save_id, academy_class.id, 77);

        super::set_member_outcome(
            &conn,
            save_id,
            academy_class.id,
            77,
            Some(super::AcademyMemberOutcomeInput {
                status: "sold".to_string(),
                buying_club: Some("  Rovers FC  ".to_string()),
                sale_fee_eur: Some(1_250_000),
            }),
        )
        .expect("record sale");
        assert_eq!(
            super::get_class(&conn, save_id, academy_class.id)
                .expect("load sold member")
                .members[0]
                .outcome,
            Some(super::AcademyMemberOutcome {
                status: super::AcademyMemberOutcomeStatus::Sold,
                buying_club: Some("Rovers FC".to_string()),
                sale_fee_eur: Some(1_250_000),
            })
        );

        super::set_member_outcome(
            &conn,
            save_id,
            academy_class.id,
            77,
            Some(super::AcademyMemberOutcomeInput {
                status: "released".to_string(),
                buying_club: None,
                sale_fee_eur: None,
            }),
        )
        .expect("replace sale with release");
        assert_eq!(
            super::get_class(&conn, save_id, academy_class.id)
                .expect("load released member")
                .members[0]
                .outcome,
            Some(super::AcademyMemberOutcome {
                status: super::AcademyMemberOutcomeStatus::Released,
                buying_club: None,
                sale_fee_eur: None,
            })
        );

        super::set_member_outcome(&conn, save_id, academy_class.id, 77, None)
            .expect("restore member to club");
        assert_eq!(
            super::get_class(&conn, save_id, academy_class.id)
                .expect("load restored member")
                .members[0]
                .outcome,
            None
        );
    }

    #[test]
    fn outcome_validation_and_save_scoping_preserve_existing_outcomes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("academy-outcome-validation.db"));
        let first_save_id = insert_save(&conn, "First save");
        let second_save_id = insert_save(&conn, "Second save");
        let academy_class =
            super::create_class(&conn, first_save_id, 2030).expect("create academy class");
        insert_membership(&conn, first_save_id, academy_class.id, 77);

        super::set_member_outcome(
            &conn,
            first_save_id,
            academy_class.id,
            77,
            Some(super::AcademyMemberOutcomeInput {
                status: "sold".to_string(),
                buying_club: Some("Existing FC".to_string()),
                sale_fee_eur: Some(250_000),
            }),
        )
        .expect("record existing sale");

        let invalid_sale = super::set_member_outcome(
            &conn,
            first_save_id,
            academy_class.id,
            77,
            Some(super::AcademyMemberOutcomeInput {
                status: "sold".to_string(),
                buying_club: Some(" ".to_string()),
                sale_fee_eur: Some(1),
            }),
        )
        .expect_err("reject blank buying club");
        let negative_fee = super::set_member_outcome(
            &conn,
            first_save_id,
            academy_class.id,
            77,
            Some(super::AcademyMemberOutcomeInput {
                status: "sold".to_string(),
                buying_club: Some("Rovers FC".to_string()),
                sale_fee_eur: Some(-1),
            }),
        )
        .expect_err("reject negative sale fee");
        let released_with_sale_data = super::set_member_outcome(
            &conn,
            first_save_id,
            academy_class.id,
            77,
            Some(super::AcademyMemberOutcomeInput {
                status: "released".to_string(),
                buying_club: Some("Rovers FC".to_string()),
                sale_fee_eur: Some(1),
            }),
        )
        .expect_err("reject release sale data");
        let unknown_status = super::set_member_outcome(
            &conn,
            first_save_id,
            academy_class.id,
            77,
            Some(super::AcademyMemberOutcomeInput {
                status: "loaned".to_string(),
                buying_club: None,
                sale_fee_eur: None,
            }),
        )
        .expect_err("reject unknown outcome status");
        let cross_save = super::set_member_outcome(
            &conn,
            second_save_id,
            academy_class.id,
            77,
            Some(super::AcademyMemberOutcomeInput {
                status: "sold".to_string(),
                buying_club: Some("Rovers FC".to_string()),
                sale_fee_eur: Some(1),
            }),
        )
        .expect_err("reject another save's member");

        assert_eq!(invalid_sale, "Sale outcomes require a buying club");
        assert_eq!(
            negative_fee,
            "Sale outcomes require a non-negative whole-euro fee"
        );
        assert_eq!(
            released_with_sale_data,
            "Released outcomes cannot include sale details"
        );
        assert_eq!(unknown_status, "Unknown academy outcome status `loaned`");
        assert_eq!(
            cross_save,
            format!(
                "Player 77 is not assigned to academy class {}",
                academy_class.id
            )
        );
        assert_eq!(
            super::get_class(&conn, first_save_id, academy_class.id)
                .expect("load unchanged member")
                .members[0]
                .outcome,
            Some(super::AcademyMemberOutcome {
                status: super::AcademyMemberOutcomeStatus::Sold,
                buying_club: Some("Existing FC".to_string()),
                sale_fee_eur: Some(250_000),
            })
        );
    }

    #[test]
    fn outcome_storage_rejects_incomplete_or_mixed_values() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("academy-outcome-schema.db"));
        let save_id = insert_save(&conn, "Academy save");
        let academy_class =
            super::create_class(&conn, save_id, 2030).expect("create academy class");
        insert_membership(&conn, save_id, academy_class.id, 77);

        for (status, buying_club, sale_fee_eur) in [
            ("sold", None, Some(1)),
            ("sold", Some("Rovers FC"), None),
            ("released", Some("Rovers FC"), None),
            ("released", None, Some(1)),
        ] {
            let error = conn
                .execute(
                    "INSERT INTO academy_member_outcomes (
                        save_id,
                        player_uid,
                        status,
                        buying_club,
                        sale_fee_eur
                     ) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![save_id, 77, status, buying_club, sale_fee_eur],
                )
                .expect_err("reject incomplete or mixed outcome values");
            assert!(error.to_string().contains("CHECK constraint failed"));
        }
    }

    #[test]
    fn removing_membership_cascades_its_outcome() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("academy-outcome-cascade.db"));
        let save_id = insert_save(&conn, "Academy save");
        let academy_class =
            super::create_class(&conn, save_id, 2030).expect("create academy class");
        insert_membership(&conn, save_id, academy_class.id, 77);
        super::set_member_outcome(
            &conn,
            save_id,
            academy_class.id,
            77,
            Some(super::AcademyMemberOutcomeInput {
                status: "sold".to_string(),
                buying_club: Some("Rovers FC".to_string()),
                sale_fee_eur: Some(500_000),
            }),
        )
        .expect("record sale");

        super::remove_member(&conn, save_id, academy_class.id, 77).expect("remove academy member");
        let outcome_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM academy_member_outcomes", [], |row| {
                row.get(0)
            })
            .expect("count outcomes");
        assert_eq!(outcome_count, 0);
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
        configure_managed_club(&conn, save_id);
        let first_class =
            super::create_class(&conn, save_id, 2030).expect("create first academy class");
        let second_class =
            super::create_class(&conn, save_id, 2031).expect("create second academy class");
        super::assign_member(&conn, save_id, first_class.id, 77).expect("assign academy member");
        super::remove_member(&conn, save_id, first_class.id, 77).expect("remove academy member");
        super::assign_member(&conn, save_id, second_class.id, 77).expect("reassign academy member");
        super::set_member_outcome(
            &conn,
            save_id,
            second_class.id,
            77,
            Some(super::AcademyMemberOutcomeInput {
                status: "sold".to_string(),
                buying_club: Some("Rovers FC".to_string()),
                sale_fee_eur: Some(750_000),
            }),
        )
        .expect("record sale before snapshot replacement");

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
        assert_eq!(
            departed.members[0].outcome,
            Some(super::AcademyMemberOutcome {
                status: super::AcademyMemberOutcomeStatus::Sold,
                buying_club: Some("Rovers FC".to_string()),
                sale_fee_eur: Some(750_000),
            })
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
        assert_eq!(
            unresolved.members[0].outcome,
            Some(super::AcademyMemberOutcome {
                status: super::AcademyMemberOutcomeStatus::Sold,
                buying_club: Some("Rovers FC".to_string()),
                sale_fee_eur: Some(750_000),
            })
        );
    }
}
