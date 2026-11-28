use std::collections::{BTreeMap, HashSet};

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;

use crate::features::managed_club::service::{
    managed_club_loaned_out_player_sql, managed_club_owned_player_sql,
};
use crate::features::memory_read::dump_validation::canonical_game_date;

/// FM gives the players it generates Unique IDs from here up; database players sit below.
const FIRST_GENERATED_UID: i64 = 2_000_000_000;
/// The oldest a generated player may be when first seen and still count as an intake player.
const MAX_INTAKE_AGE: i64 = 17;
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
    pub outcome: Option<AcademyMemberOutcome>,
    pub is_graduate: Option<bool>,
    /// The managed club has loaned the member out; the current club is the borrower.
    pub loaned_out: bool,
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
    in_managed_club: bool,
    loaned_out: bool,
}

/// What the linked FM save held of a member's season on `as_of`, its in-game date.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SeasonReading {
    as_of: String,
    minutes: i64,
    appearances: i64,
    goals: i64,
    assists: i64,
}

/// A member's competitive matches as the season statistics of the linked FM save have shown
/// them: the seasons FM has closed, and the running one as last read.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CareerStats {
    closed_appearances: i64,
    closed_goals: i64,
    closed_assists: i64,
    season: SeasonReading,
}

impl CareerStats {
    fn starting_with(season: SeasonReading) -> Self {
        Self {
            closed_appearances: 0,
            closed_goals: 0,
            closed_assists: 0,
            season,
        }
    }

    // ponytail: FM has closed a season when a later read shows fewer minutes, and the season
    // counts as it was last read. Totals are a lower bound, and a season first read after it
    // has passed the previous one's last reading merges with it.
    // Upgrade to the save's archived season history (fmsave `player_season_history`) if a
    // member's totals fall more than a few matches a season short of FM's career history.
    fn advance(self, reading: SeasonReading) -> Self {
        // A read dated before the last one comes from an earlier save of the same season.
        if reading.as_of < self.season.as_of || reading.minutes >= self.season.minutes {
            return Self {
                season: reading,
                ..self
            };
        }
        Self {
            closed_appearances: self.closed_appearances + self.season.appearances,
            closed_goals: self.closed_goals + self.season.goals,
            closed_assists: self.closed_assists + self.season.assists,
            season: reading,
        }
    }

    fn appearances(&self) -> i64 {
        self.closed_appearances + self.season.appearances
    }

    fn goals(&self) -> i64 {
        self.closed_goals + self.season.goals
    }

    fn assists(&self) -> i64 {
        self.closed_assists + self.season.assists
    }
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

/// Who FM had already generated when a new snapshot arrives, and the class its intake joins.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct IntakeBaseline {
    class_year: i64,
    generated_uids: HashSet<i64>,
}

/// The baseline for the intake of the snapshot `snapshot_id` dated `game_date`: the generated
/// players of the save's latest other snapshot dated on or before it. Load Data reads it before
/// it deletes that snapshot. `None` when the date is not trusted or the save has no earlier
/// snapshot; no intake can be told then.
pub(crate) fn intake_baseline(
    conn: &Connection,
    save_id: i64,
    snapshot_id: i64,
    game_date: Option<&str>,
    game_date_source: &str,
) -> Result<Option<IntakeBaseline>, String> {
    let Some(class_year) = observed_class_year(game_date, game_date_source) else {
        return Ok(None);
    };
    let earlier_snapshot_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM snapshots
             WHERE save_id = ?1 AND id <> ?2 AND game_date IS NOT NULL AND game_date <= ?3
             ORDER BY game_date DESC, loaded_at_utc DESC, id DESC
             LIMIT 1",
            params![save_id, snapshot_id, game_date],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let Some(earlier_snapshot_id) = earlier_snapshot_id else {
        return Ok(None);
    };
    let generated_uids = conn
        .prepare("SELECT uid FROM players WHERE snapshot_id = ?1 AND uid >= ?2")
        .map_err(|error| error.to_string())?
        .query_map(params![earlier_snapshot_id, FIRST_GENERATED_UID], |row| {
            row.get(0)
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|error| error.to_string())?;

    Ok(Some(IntakeBaseline {
        class_year,
        generated_uids,
    }))
}

/// Adds the youth intake of the snapshot `snapshot_id` to the class of its year, creating the
/// class when the intake is its first member. An intake player is a generated player the
/// managed club owns who was not in the game at the baseline and is at most `MAX_INTAKE_AGE`.
/// A player bought in existed before, so he is left out.
///
/// The year is the snapshot's, so an intake first loaded after the following New Year joins the
/// later class.
pub(crate) fn record_intake(
    conn: &Connection,
    save_id: i64,
    snapshot_id: i64,
    baseline: &IntakeBaseline,
) -> Result<(), String> {
    let intake = conn
        .prepare(&format!(
            "SELECT p.uid, p.name
             FROM players p
             WHERE p.uid >= ?3
               AND p.age <= ?4
               AND trim(p.name) <> ''
               AND {}
               AND NOT EXISTS(
                   SELECT 1
                   FROM academy_memberships membership
                   WHERE membership.save_id = ?2
                     AND membership.player_uid = p.uid
               )",
            managed_club_owned_player_sql("p", "?1", "?2")
        ))
        .map_err(|error| error.to_string())?
        .query_map(
            params![snapshot_id, save_id, FIRST_GENERATED_UID, MAX_INTAKE_AGE],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|error| error.to_string())?
        .filter(|player| {
            player
                .as_ref()
                .map_or(true, |(uid, _)| !baseline.generated_uids.contains(uid))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    if intake.is_empty() {
        return Ok(());
    }

    conn.execute(
        "INSERT INTO academy_classes (save_id, class_year) VALUES (?1, ?2)
         ON CONFLICT(save_id, class_year) DO NOTHING",
        params![save_id, baseline.class_year],
    )
    .map_err(|error| error.to_string())?;
    let mut insert = conn
        .prepare(
            "INSERT INTO academy_memberships (save_id, class_id, player_uid, last_known_name)
             SELECT ?1, id, ?3, ?4
             FROM academy_classes
             WHERE save_id = ?1 AND class_year = ?2",
        )
        .map_err(|error| error.to_string())?;
    for (player_uid, name) in intake {
        insert
            .execute(params![save_id, baseline.class_year, player_uid, name])
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

/// Carries every member's career forward to the season statistics just attached to the
/// snapshot `snapshot_id`. A member without a stored career starts from the snapshots the save
/// still holds.
pub(crate) fn record_season_readings(
    conn: &Connection,
    save_id: i64,
    snapshot_id: i64,
) -> Result<(), String> {
    let member_uids = conn
        .prepare("SELECT player_uid FROM academy_memberships WHERE save_id = ?1")
        .map_err(|error| error.to_string())?
        .query_map([save_id], |row| row.get::<_, i64>(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let mut upsert = conn
        .prepare(
            "INSERT INTO academy_member_career_stats (
                save_id, player_uid, closed_appearances, closed_goals, closed_assists,
                season_as_of, season_minutes, season_appearances, season_goals, season_assists
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(save_id, player_uid) DO UPDATE SET
                closed_appearances = excluded.closed_appearances,
                closed_goals = excluded.closed_goals,
                closed_assists = excluded.closed_assists,
                season_as_of = excluded.season_as_of,
                season_minutes = excluded.season_minutes,
                season_appearances = excluded.season_appearances,
                season_goals = excluded.season_goals,
                season_assists = excluded.season_assists",
        )
        .map_err(|error| error.to_string())?;
    for player_uid in member_uids {
        let career = match stored_career(conn, save_id, player_uid)? {
            Some(career) => season_readings(conn, save_id, player_uid, Some(snapshot_id))?
                .into_iter()
                .fold(career, CareerStats::advance),
            None => match retained_career(conn, save_id, player_uid)? {
                Some(career) => career,
                None => continue,
            },
        };
        upsert
            .execute(params![
                save_id,
                player_uid,
                career.closed_appearances,
                career.closed_goals,
                career.closed_assists,
                career.season.as_of,
                career.season.minutes,
                career.season.appearances,
                career.season.goals,
                career.season.assists,
            ])
            .map_err(|error| error.to_string())?;
    }
    Ok(())
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
             ORDER BY class.class_year ASC",
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
    let class_year = academy_class(conn, save_id, class_id)?;
    let snapshot_id = current_snapshot_id(conn, save_id)?;
    let mut statement = conn
        .prepare(
            "SELECT membership.player_uid,
                    membership.last_known_name,
                    outcome.status,
                    outcome.buying_club,
                    outcome.sale_fee_eur
             FROM academy_memberships membership
             LEFT JOIN academy_member_outcomes outcome
               ON outcome.save_id = membership.save_id
              AND outcome.player_uid = membership.player_uid
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
            ))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    drop(statement);

    let members = memberships
        .into_iter()
        .map(
            |(player_uid, last_known_name, status, buying_club, sale_fee_eur)| {
                let player = match snapshot_id {
                    Some(snapshot_id) => {
                        load_current_player(conn, save_id, snapshot_id, player_uid)?
                    }
                    None => None,
                };
                // A member added since the last read of the linked save has no stored career.
                let career = match stored_career(conn, save_id, player_uid)? {
                    Some(career) => Some(career),
                    None => retained_career(conn, save_id, player_uid)?,
                };
                let outcome = stored_member_outcome(status, buying_club, sale_fee_eur)?;
                Ok(academy_member(
                    player_uid,
                    last_known_name,
                    player,
                    career,
                    outcome,
                ))
            },
        )
        .collect::<Result<Vec<_>, String>>()?;

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
        .prepare(&format!(
            "SELECT p.uid, p.name, p.age, p.positions_json, p.current_club
             FROM players p
             WHERE p.snapshot_id = ?1
               AND p.name LIKE ?2 ESCAPE '\\' COLLATE NOCASE
               AND {}
               AND NOT EXISTS(
                   SELECT 1
                   FROM academy_memberships membership
                   WHERE membership.save_id = ?3
                     AND membership.player_uid = p.uid
               )
             ORDER BY p.name COLLATE NOCASE, p.uid
             LIMIT ?4",
            managed_club_owned_player_sql("p", "?1", "?3")
        ))
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
            &format!(
                "SELECT p.name
                 FROM players p
                 WHERE p.snapshot_id = ?1 AND p.uid = ?2 AND {}",
                managed_club_owned_player_sql("p", "?1", "?3")
            ),
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

/// The year of the class `class_id` of the save, or an error when the save has no such class.
fn academy_class(conn: &Connection, save_id: i64, class_id: i64) -> Result<i64, String> {
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

/// The year of a valid `yyyy-MM-dd` game date the bridge read or derived.
fn observed_class_year(game_date: Option<&str>, game_date_source: &str) -> Option<i64> {
    let game_date = game_date.filter(|date| {
        matches!(game_date_source, "memory" | "derived") && canonical_game_date(date)
    })?;
    game_date[0..4].parse().ok()
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
    player_uid: i64,
    last_known_name: String,
    player: Option<CurrentAcademyPlayer>,
    career: Option<CareerStats>,
    outcome: Option<AcademyMemberOutcome>,
) -> AcademyMember {
    let reported_career_appearances = career.as_ref().map(CareerStats::appearances);
    let goals = career.as_ref().map(CareerStats::goals);
    let assists = career.as_ref().map(CareerStats::assists);
    let is_graduate = reported_career_appearances.map(|appearances| appearances >= 1);
    let Some(player) = player else {
        return AcademyMember {
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
            reported_career_appearances,
            goals,
            assists,
            outcome,
            is_graduate,
            loaned_out: false,
        };
    };
    let state = if player.in_managed_club {
        AcademyMemberState::Resolved
    } else {
        AcademyMemberState::Departed
    };

    AcademyMember {
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
        reported_career_appearances,
        goals,
        assists,
        outcome,
        is_graduate,
        loaned_out: player.loaned_out,
    }
}

fn stored_career(
    conn: &Connection,
    save_id: i64,
    player_uid: i64,
) -> Result<Option<CareerStats>, String> {
    conn.query_row(
        "SELECT closed_appearances, closed_goals, closed_assists,
                season_as_of, season_minutes, season_appearances, season_goals, season_assists
         FROM academy_member_career_stats
         WHERE save_id = ?1 AND player_uid = ?2",
        params![save_id, player_uid],
        |row| {
            Ok(CareerStats {
                closed_appearances: row.get(0)?,
                closed_goals: row.get(1)?,
                closed_assists: row.get(2)?,
                season: SeasonReading {
                    as_of: row.get(3)?,
                    minutes: row.get(4)?,
                    appearances: row.get(5)?,
                    goals: row.get(6)?,
                    assists: row.get(7)?,
                },
            })
        },
    )
    .optional()
    .map_err(|error| error.to_string())
}

/// The career the save's remaining snapshots show, or `None` when none of them has season
/// statistics.
fn retained_career(
    conn: &Connection,
    save_id: i64,
    player_uid: i64,
) -> Result<Option<CareerStats>, String> {
    Ok(season_readings(conn, save_id, player_uid, None)?
        .into_iter()
        .fold(None, |career: Option<CareerStats>, reading| {
            Some(match career {
                Some(career) => career.advance(reading),
                None => CareerStats::starting_with(reading),
            })
        }))
}

/// The player's season in each of the save's snapshots that has season statistics, or in
/// `only_snapshot_id` alone, oldest first. Statistics that leave him out read as no match
/// played.
fn season_readings(
    conn: &Connection,
    save_id: i64,
    player_uid: i64,
    only_snapshot_id: Option<i64>,
) -> Result<Vec<SeasonReading>, String> {
    conn.prepare(
        "SELECT snapshot.moneyball_stats_game_date,
                COALESCE(stats.minutes, 0),
                COALESCE(stats.starts, 0) + COALESCE(stats.substitute_appearances, 0),
                COALESCE(stats.goals, 0),
                COALESCE(stats.assists, 0)
         FROM snapshots snapshot
         LEFT JOIN player_moneyball_stats stats
           ON stats.snapshot_id = snapshot.id AND stats.player_uid = ?2
         WHERE snapshot.save_id = ?1
           AND snapshot.moneyball_stats_game_date IS NOT NULL
           AND (?3 IS NULL OR snapshot.id = ?3)
         ORDER BY snapshot.moneyball_stats_game_date, snapshot.id",
    )
    .map_err(|error| error.to_string())?
    .query_map(params![save_id, player_uid, only_snapshot_id], |row| {
        Ok(SeasonReading {
            as_of: row.get(0)?,
            minutes: row.get(1)?,
            appearances: row.get(2)?,
            goals: row.get(3)?,
            assists: row.get(4)?,
        })
    })
    .map_err(|error| error.to_string())?
    .collect::<Result<_, _>>()
    .map_err(|error| error.to_string())
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
    save_id: i64,
    snapshot_id: i64,
    player_uid: i64,
) -> Result<Option<CurrentAcademyPlayer>, String> {
    let player = conn
        .query_row(
            &format!(
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
                    preferred_foot,
                    COALESCE({}, 0),
                    COALESCE({}, 0)
                 FROM players
                 WHERE snapshot_id = ?1 AND uid = ?2",
                managed_club_owned_player_sql("players", "?1", "?3"),
                managed_club_loaned_out_player_sql("players", "?3")
            ),
            params![snapshot_id, player_uid, save_id],
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
                    row.get::<_, bool>(11)?,
                    row.get::<_, bool>(12)?,
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
                in_managed_club,
                loaned_out,
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
                    in_managed_club,
                    loaned_out,
                })
            },
        )
        .transpose()
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
            serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v11.json"))
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
            serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v11.json"))
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

    /// Loads `players` as a snapshot dated `game_date`, or with no known date for `None`.
    fn ingest_dated(
        temp_dir: &tempfile::TempDir,
        conn: &mut Connection,
        save_id: i64,
        filename: &str,
        game_date: Option<&str>,
        players: Vec<Value>,
    ) {
        let mut dump: Value =
            serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v11.json"))
                .expect("parse fixture");
        dump["gameDate"] = json!(game_date);
        dump["gameDateSource"] = json!(if game_date.is_some() {
            "derived"
        } else {
            "unknown"
        });
        dump["playerCount"] = json!(players.len());
        dump["players"] = Value::Array(players);
        let dump_path = temp_dir.path().join(filename);
        fs::write(
            &dump_path,
            serde_json::to_string(&dump).expect("serialize dump"),
        )
        .expect("write dump");
        ingest::ingest_dump_file_for_save(conn, save_id, &dump_path).expect("ingest dump");
    }

    /// FM's first Unique ID for a player it generates.
    const FIRST_GENERATED: i64 = super::FIRST_GENERATED_UID;

    /// A youth player aged `age` whom Loan FC, the club the tests manage, owns and has at home.
    fn club_youngster(uid: i64, name: &str, age: i64) -> Value {
        let mut player = fixture_player(uid, name, "Loan FC", "youth");
        player["age"] = json!(age);
        player["onLoan"] = json!(false);
        player["parentClub"] = json!("Loan FC");
        player["parentClubUid"] = json!(1000);
        player
    }

    fn member_names(conn: &Connection, save_id: i64, class_id: i64) -> Vec<String> {
        super::get_class(conn, save_id, class_id)
            .expect("read class")
            .members
            .into_iter()
            .map(|member| member.last_known_name)
            .collect()
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
        managed_club_service::set_managed_club(conn, save_id, "Loan FC", Some(1000))
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

    /// Stores `player_uid`'s season in the snapshot as a read of the linked save on `as_of`
    /// leaves it: `[minutes, starts, substitute appearances, goals, assists]`, and no row for a
    /// player without a minute.
    fn seed_season(
        conn: &Connection,
        snapshot_id: i64,
        as_of: &str,
        player_uid: i64,
        season: [i64; 5],
    ) {
        conn.execute(
            "DELETE FROM player_moneyball_stats WHERE snapshot_id = ?1 AND player_uid = ?2",
            params![snapshot_id, player_uid],
        )
        .expect("clear season");
        if season[0] > 0 {
            conn.execute(
                "INSERT INTO player_moneyball_stats (
                    snapshot_id, player_uid, minutes, starts, substitute_appearances, goals, assists
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    snapshot_id,
                    player_uid,
                    season[0],
                    season[1],
                    season[2],
                    season[3],
                    season[4]
                ],
            )
            .expect("insert season");
        }
        conn.execute(
            "UPDATE snapshots SET moneyball_stats_game_date = ?1 WHERE id = ?2",
            params![as_of, snapshot_id],
        )
        .expect("set as-of date");
    }

    /// A read of the linked save on `as_of`: the season lands on the current snapshot and
    /// every member's career is carried forward.
    fn read_season(
        conn: &Connection,
        save_id: i64,
        as_of: &str,
        player_uid: i64,
        season: [i64; 5],
    ) {
        let snapshot_id = super::current_snapshot_id(conn, save_id)
            .expect("read current snapshot")
            .expect("current snapshot");
        seed_season(conn, snapshot_id, as_of, player_uid, season);
        super::record_season_readings(conn, save_id, snapshot_id).expect("record season readings");
    }

    /// The member's appearances, goals, assists, and graduate status.
    fn career(
        conn: &Connection,
        save_id: i64,
        class_id: i64,
        player_uid: i64,
    ) -> (Option<i64>, Option<i64>, Option<i64>, Option<bool>) {
        let member = super::get_class(conn, save_id, class_id)
            .expect("read class")
            .members
            .into_iter()
            .find(|member| member.player_uid == player_uid)
            .expect("member");
        (
            member.reported_career_appearances,
            member.goals,
            member.assists,
            member.is_graduate,
        )
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
    fn a_load_adds_the_managed_clubs_new_generated_youngsters_to_the_class_of_its_year() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("academy-intake.db"));
        let save_id = insert_save(&conn, "Academy save");
        ingest_dated(
            &temp_dir,
            &mut conn,
            save_id,
            "before-intake.json",
            Some("2027-01-23"),
            vec![club_youngster(FIRST_GENERATED + 1, "Earlier Intake", 16)],
        );
        configure_managed_club(&conn, save_id);
        assert_eq!(
            super::list_classes(&conn, save_id),
            Ok(vec![]),
            "the first load has no earlier snapshot to compare with"
        );

        let mut loaned_out = club_youngster(FIRST_GENERATED + 11, "Loaned Out", 16);
        loaned_out["currentClub"] = json!("Borrower FC");
        loaned_out["currentClubUid"] = json!(4000);
        loaned_out["onLoan"] = json!(true);
        let mut other_club = club_youngster(FIRST_GENERATED + 13, "Other Club", 15);
        for (key, value) in [("currentClub", "Other FC"), ("parentClub", "Other FC")] {
            other_club[key] = json!(value);
        }
        for key in ["currentClubUid", "parentClubUid"] {
            other_club[key] = json!(5000);
        }
        let mut namesake_club = club_youngster(FIRST_GENERATED + 14, "Namesake Club", 15);
        for key in ["currentClubUid", "parentClubUid"] {
            namesake_club[key] = json!(9999);
        }
        ingest_dated(
            &temp_dir,
            &mut conn,
            save_id,
            "after-intake.json",
            Some("2027-05-22"),
            vec![
                club_youngster(FIRST_GENERATED + 1, "Earlier Intake", 16),
                club_youngster(FIRST_GENERATED + 10, "New Intake", 15),
                loaned_out,
                club_youngster(FIRST_GENERATED + 12, "Too Old", 18),
                club_youngster(90, "Database Player", 16),
                other_club,
                namesake_club,
            ],
        );

        let classes = super::list_classes(&conn, save_id).expect("list classes");
        assert_eq!(
            classes
                .iter()
                .map(|academy_class| (academy_class.class_year, academy_class.member_count))
                .collect::<Vec<_>>(),
            [(2027, 2)]
        );
        assert_eq!(
            member_names(&conn, save_id, classes[0].id),
            ["Loaned Out", "New Intake"]
        );
    }

    #[test]
    fn a_later_load_adds_neither_a_member_nor_a_removed_player_again() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("academy-intake-repeat.db"));
        let save_id = insert_save(&conn, "Academy save");
        let anchor = || fixture_player(77, "Senior Player", "Loan FC", "senior");
        let removed = || club_youngster(FIRST_GENERATED + 10, "Removed Later", 15);
        let placed = || club_youngster(FIRST_GENERATED + 11, "Placed By Hand", 16);
        ingest_dated(
            &temp_dir,
            &mut conn,
            save_id,
            "before-intake.json",
            Some("2027-01-23"),
            vec![anchor()],
        );
        configure_managed_club(&conn, save_id);
        let manual = super::create_class(&conn, save_id, 2020).expect("create manual class");
        insert_membership(&conn, save_id, manual.id, FIRST_GENERATED + 11);

        ingest_dated(
            &temp_dir,
            &mut conn,
            save_id,
            "intake.json",
            Some("2027-05-22"),
            vec![anchor(), removed(), placed()],
        );
        let intake_class = super::list_classes(&conn, save_id)
            .expect("list classes")
            .into_iter()
            .find(|academy_class| academy_class.class_year == 2027)
            .expect("intake class");
        assert_eq!(
            member_names(&conn, save_id, intake_class.id),
            ["Removed Later"]
        );
        assert_eq!(member_names(&conn, save_id, manual.id), ["Academy Player"]);

        super::remove_member(&conn, save_id, intake_class.id, FIRST_GENERATED + 10)
            .expect("remove intake player");
        ingest_dated(
            &temp_dir,
            &mut conn,
            save_id,
            "later.json",
            Some("2027-09-04"),
            vec![
                anchor(),
                removed(),
                placed(),
                club_youngster(FIRST_GENERATED + 20, "Late Arrival", 15),
            ],
        );

        assert_eq!(
            member_names(&conn, save_id, intake_class.id),
            ["Late Arrival"]
        );
        assert_eq!(member_names(&conn, save_id, manual.id), ["Academy Player"]);
    }

    #[test]
    fn no_intake_is_recorded_without_a_managed_club_or_a_trusted_date() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("academy-intake-unknown.db"));
        let save_id = insert_save(&conn, "Academy save");
        let anchor = || fixture_player(77, "Senior Player", "Loan FC", "senior");
        ingest_dated(
            &temp_dir,
            &mut conn,
            save_id,
            "first.json",
            Some("2027-01-23"),
            vec![anchor()],
        );
        ingest_dated(
            &temp_dir,
            &mut conn,
            save_id,
            "no-club.json",
            Some("2027-05-22"),
            vec![
                anchor(),
                club_youngster(FIRST_GENERATED + 10, "No Managed Club", 15),
            ],
        );
        configure_managed_club(&conn, save_id);
        ingest_dated(
            &temp_dir,
            &mut conn,
            save_id,
            "no-date.json",
            None,
            vec![
                anchor(),
                club_youngster(FIRST_GENERATED + 10, "No Managed Club", 15),
                club_youngster(FIRST_GENERATED + 11, "No Game Date", 15),
            ],
        );

        assert_eq!(super::list_classes(&conn, save_id), Ok(vec![]));
    }

    #[test]
    fn a_load_dated_before_the_current_snapshot_records_no_intake() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("academy-intake-earlier.db"));
        let save_id = insert_save(&conn, "Academy save");
        let anchor = || fixture_player(77, "Senior Player", "Loan FC", "senior");
        ingest_dated(
            &temp_dir,
            &mut conn,
            save_id,
            "first.json",
            Some("2027-01-23"),
            vec![anchor()],
        );
        configure_managed_club(&conn, save_id);
        ingest_dated(
            &temp_dir,
            &mut conn,
            save_id,
            "current.json",
            Some("2027-05-22"),
            vec![anchor()],
        );
        conn.execute(
            "UPDATE snapshots SET is_kept = 1 WHERE save_id = ?1",
            [save_id],
        )
        .expect("keep the current snapshot");

        // Another timeline of the same save: its intake is not in the current snapshot.
        ingest_dated(
            &temp_dir,
            &mut conn,
            save_id,
            "earlier.json",
            Some("2027-03-01"),
            vec![
                anchor(),
                club_youngster(FIRST_GENERATED + 10, "Other Timeline", 15),
            ],
        );

        assert_eq!(super::list_classes(&conn, save_id), Ok(vec![]));
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
    fn a_member_the_club_has_loaned_out_stays_a_member_and_is_marked() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("academy-loans.db"));
        let save_id = insert_save(&conn, "Academy save");
        ingest_players(
            &temp_dir,
            &mut conn,
            save_id,
            "academy-loans.json",
            vec![
                fixture_player(77, "Club Player", "Loan FC", "senior"),
                fixture_player(78, "Loaned Prospect", "Borrower FC", "senior"),
            ],
        );
        conn.execute(
            "UPDATE players SET current_club_uid = 4000, parent_club = 'Loan FC',
                 parent_club_uid = 1000, on_loan = 1
             WHERE uid = 78",
            [],
        )
        .expect("loan the prospect out");
        configure_managed_club(&conn, save_id);
        let academy_class =
            super::create_class(&conn, save_id, 2030).expect("create academy class");

        super::assign_member(&conn, save_id, academy_class.id, 78)
            .expect("assign the loaned-out prospect");
        let detail =
            super::get_class(&conn, save_id, academy_class.id).expect("load academy class");

        assert_eq!(detail.members[0].state, super::AcademyMemberState::Resolved);
        assert!(detail.members[0].loaned_out);
        assert_eq!(
            detail.members[0].current_club.as_deref(),
            Some("Borrower FC")
        );
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
        assert_eq!(member.outcome, None);
        assert_eq!(member.is_graduate, None);
        assert!(super::list_candidates(&conn, save_id, "")
            .expect("list unassigned candidates")
            .is_empty());
    }

    #[test]
    fn a_namesake_clubs_player_is_neither_a_candidate_nor_a_current_member() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("academy-namesake.db"));
        let save_id = insert_save(&conn, "Academy save");
        let at_namesake = |uid, name| {
            let mut player = fixture_player(uid, name, "Loan FC", "youth");
            player["currentClubUid"] = json!(3000);
            player
        };
        ingest_players(
            &temp_dir,
            &mut conn,
            save_id,
            "academy-namesake-first.json",
            vec![
                fixture_player(77, "Golden Fixture Player", "Loan FC", "youth"),
                at_namesake(78, "Namesake Player"),
            ],
        );
        configure_managed_club(&conn, save_id);
        let academy_class =
            super::create_class(&conn, save_id, 2030).expect("create academy class");
        let candidate_uids = |conn: &Connection| {
            super::list_candidates(conn, save_id, "")
                .expect("list candidates")
                .iter()
                .map(|candidate| candidate.player_uid)
                .collect::<Vec<_>>()
        };

        assert_eq!(candidate_uids(&conn), [77]);
        assert_eq!(
            super::assign_member(&conn, save_id, academy_class.id, 78)
                .expect_err("reject namesake club player"),
            "Player 78 is not an eligible academy candidate"
        );
        super::assign_member(&conn, save_id, academy_class.id, 77).expect("assign club player");

        ingest_players(
            &temp_dir,
            &mut conn,
            save_id,
            "academy-namesake-moved.json",
            vec![
                at_namesake(77, "Golden Fixture Player"),
                at_namesake(78, "Namesake Player"),
            ],
        );
        let member_state = |conn: &Connection| {
            super::get_class(conn, save_id, academy_class.id)
                .expect("load academy class")
                .members[0]
                .state
                .clone()
        };
        assert_eq!(member_state(&conn), super::AcademyMemberState::Departed);

        // A club chosen before UIDs were recorded matches by name.
        conn.execute("UPDATE managed_club_settings SET club_uid = NULL", [])
            .expect("forget club UID");
        assert_eq!(member_state(&conn), super::AcademyMemberState::Resolved);
        assert_eq!(candidate_uids(&conn), [78]);
    }

    #[test]
    fn a_career_adds_the_seasons_fm_closed_to_the_running_one() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("academy-career.db"));
        let save_id = insert_save(&conn, "Academy save");
        ingest_players(
            &temp_dir,
            &mut conn,
            save_id,
            "academy-career.json",
            vec![fixture_player(77, "Academy Graduate", "Loan FC", "senior")],
        );
        configure_managed_club(&conn, save_id);
        let class_id = super::create_class(&conn, save_id, 2030)
            .expect("create class")
            .id;
        super::assign_member(&conn, save_id, class_id, 77).expect("assign member");
        let career = |conn: &Connection| career(conn, save_id, class_id, 77);
        assert_eq!(
            career(&conn),
            (None, None, None, None),
            "nothing is known before the linked save is read"
        );

        read_season(&conn, save_id, "2027-03-01", 77, [900, 8, 2, 3, 1]);
        assert_eq!(career(&conn), (Some(10), Some(3), Some(1), Some(true)));
        read_season(&conn, save_id, "2027-05-20", 77, [1800, 18, 4, 6, 2]);
        assert_eq!(
            career(&conn),
            (Some(22), Some(6), Some(2), Some(true)),
            "a later read of the same season replaces the earlier one"
        );

        read_season(&conn, save_id, "2027-07-05", 77, [0; 5]);
        assert_eq!(career(&conn), (Some(22), Some(6), Some(2), Some(true)));
        read_season(&conn, save_id, "2027-09-01", 77, [270, 3, 0, 1, 0]);
        assert_eq!(career(&conn), (Some(25), Some(7), Some(2), Some(true)));

        read_season(&conn, save_id, "2027-08-20", 77, [180, 2, 0, 1, 0]);
        assert_eq!(
            career(&conn),
            (Some(24), Some(7), Some(2), Some(true)),
            "an earlier save of the same season closes nothing"
        );

        ingest_players(
            &temp_dir,
            &mut conn,
            save_id,
            "academy-career-left.json",
            vec![fixture_player(
                78,
                "Replacement Player",
                "Loan FC",
                "senior",
            )],
        );
        assert_eq!(
            career(&conn),
            (Some(24), Some(7), Some(2), Some(true)),
            "a member the snapshot no longer holds keeps his career"
        );
    }

    #[test]
    fn a_member_without_a_stored_career_reads_it_from_the_snapshots_the_save_kept() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("academy-career-kept.db"));
        let save_id = insert_save(&conn, "Academy save");
        let players = || {
            vec![
                fixture_player(77, "Played Before", "Loan FC", "senior"),
                fixture_player(78, "Never Played", "Loan FC", "youth"),
            ]
        };
        ingest_dated(
            &temp_dir,
            &mut conn,
            save_id,
            "last-season.json",
            Some("2027-05-22"),
            players(),
        );
        configure_managed_club(&conn, save_id);
        let last_season = super::current_snapshot_id(&conn, save_id)
            .expect("read snapshot")
            .expect("snapshot");
        seed_season(&conn, last_season, "2027-05-22", 77, [1800, 20, 2, 5, 4]);
        conn.execute(
            "UPDATE snapshots SET is_kept = 1 WHERE id = ?1",
            [last_season],
        )
        .expect("keep last season");
        ingest_dated(
            &temp_dir,
            &mut conn,
            save_id,
            "this-season.json",
            Some("2027-09-04"),
            players(),
        );
        let this_season = super::current_snapshot_id(&conn, save_id)
            .expect("read snapshot")
            .expect("snapshot");
        seed_season(&conn, this_season, "2027-09-04", 77, [270, 3, 0, 1, 0]);

        let class_id = super::create_class(&conn, save_id, 2025)
            .expect("create class")
            .id;
        super::assign_member(&conn, save_id, class_id, 77).expect("assign member");
        super::assign_member(&conn, save_id, class_id, 78).expect("assign member");
        let careers = |conn: &Connection| {
            (
                career(conn, save_id, class_id, 77),
                career(conn, save_id, class_id, 78),
            )
        };
        let expected = (
            (Some(25), Some(6), Some(4), Some(true)),
            (Some(0), Some(0), Some(0), Some(false)),
        );
        assert_eq!(careers(&conn), expected);

        // The next read stores the same careers, and reading again adds nothing.
        for _ in 0..2 {
            super::record_season_readings(&conn, save_id, this_season).expect("record readings");
            assert_eq!(careers(&conn), expected);
        }
        let stored: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM academy_member_career_stats WHERE save_id = ?1",
                [save_id],
                |row| row.get(0),
            )
            .expect("count stored careers");
        assert_eq!(stored, 2);
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
