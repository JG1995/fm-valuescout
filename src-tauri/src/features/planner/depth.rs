use rusqlite::{params, Connection, OptionalExtension};

use crate::features::scoring::combine::combine_role_scores;

use super::tactic::{self, PlannerTactic, TacticLane};

pub(super) const PLANNER_TEAMS: [PlannerTeam; 3] = [
    PlannerTeam::Senior,
    PlannerTeam::Reserves,
    PlannerTeam::Youth,
];
const MAX_SLOT_CANDIDATES: usize = 100;
const MAX_CANDIDATE_SEARCH_LEN: usize = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlannerTeam {
    Senior,
    Reserves,
    Youth,
}

impl PlannerTeam {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "senior" => Ok(Self::Senior),
            "reserves" => Ok(Self::Reserves),
            "youth" => Ok(Self::Youth),
            _ => Err(format!("Unknown planner team `{value}`")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Senior => "senior",
            Self::Reserves => "reserves",
            Self::Youth => "youth",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignmentState {
    Resolved,
    OutsidePool,
    Unresolved,
}

impl AssignmentState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::OutsidePool => "outside_pool",
            Self::Unresolved => "unresolved",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannerAssignment {
    pub id: i64,
    pub lane_id: String,
    pub player_uid: i64,
    pub last_known_name: String,
    pub current_name: Option<String>,
    pub state: AssignmentState,
    pub combined_score: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannerAssignmentLocation {
    pub team: PlannerTeam,
    pub string_id: i64,
    pub string_order: i64,
    pub lane_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannerSlotCandidate {
    pub player_uid: i64,
    pub name: String,
    pub current_club: String,
    pub ip_score: Option<u8>,
    pub oop_score: Option<u8>,
    pub combined_score: Option<u8>,
    pub assignment_location: Option<PlannerAssignmentLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannerString {
    pub id: i64,
    pub string_order: i64,
    pub assignments: Vec<PlannerAssignment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannerDepthTeam {
    pub team: PlannerTeam,
    pub strings: Vec<PlannerString>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlannerDepth {
    pub tactic: PlannerTactic,
    pub teams: Vec<PlannerDepthTeam>,
}

#[derive(Clone, Copy)]
pub(super) enum AssignmentProvenance {
    Manual,
    Optimizer,
}

impl AssignmentProvenance {
    fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Optimizer => "optimizer",
        }
    }
}

pub fn get_depth(conn: &Connection, save_id: i64) -> Result<PlannerDepth, String> {
    let tactic = ensure_depth(conn, save_id)?;
    let snapshot_id = current_snapshot_id(conn, save_id)?;
    let mut teams = PLANNER_TEAMS
        .into_iter()
        .map(|team| PlannerDepthTeam {
            team,
            strings: Vec::new(),
        })
        .collect::<Vec<_>>();

    let mut statement = conn
        .prepare(
            "SELECT id, team, string_order
             FROM planner_strings
             WHERE save_id = ?1
             ORDER BY CASE team
                 WHEN 'senior' THEN 0
                 WHEN 'reserves' THEN 1
                 WHEN 'youth' THEN 2
             END, string_order",
        )
        .map_err(|error| error.to_string())?;
    let strings = statement
        .query_map(params![save_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    for (id, team, string_order) in strings {
        let team = PlannerTeam::parse(&team)?;
        let assignments = load_assignments(conn, save_id, id, team, snapshot_id, &tactic)?;
        let planner_string = PlannerString {
            id,
            string_order,
            assignments,
        };
        let team_depth = teams
            .iter_mut()
            .find(|team_depth| team_depth.team == team)
            .ok_or_else(|| format!("Unknown planner team `{}`", team.as_str()))?;
        team_depth.strings.push(planner_string);
    }

    Ok(PlannerDepth { tactic, teams })
}

pub fn get_slot_candidates(
    conn: &Connection,
    save_id: i64,
    team: PlannerTeam,
    lane_id: &str,
    search: &str,
) -> Result<Vec<PlannerSlotCandidate>, String> {
    let search = normalize_candidate_search(search)?;
    let tactic = ensure_depth(conn, save_id)?;
    let lane = find_lane(&tactic, lane_id)?;
    let snapshot_id = current_snapshot_id(conn, save_id)?
        .ok_or_else(|| "No current snapshot loaded for this save".to_string())?;

    let mut statement = conn
        .prepare(
            "SELECT
                p.uid,
                p.name,
                p.current_club,
                ip.score,
                oop.score,
                assignment_string.team,
                assignment_string.id,
                assignment_string.string_order,
                assignment.lane_id
             FROM players p
             LEFT JOIN player_role_scores ip
               ON ip.snapshot_id = p.snapshot_id
              AND ip.uid = p.uid
              AND ip.role_id = ?4
             LEFT JOIN player_role_scores oop
               ON oop.snapshot_id = p.snapshot_id
              AND oop.uid = p.uid
              AND oop.role_id = ?5
             LEFT JOIN planner_assignments assignment
               ON assignment.save_id = ?2
              AND assignment.player_uid = p.uid
             LEFT JOIN planner_strings assignment_string
               ON assignment_string.id = assignment.string_id
             WHERE p.snapshot_id = ?1
               AND EXISTS(
                   SELECT 1
                   FROM planner_club_sources source
                   WHERE source.save_id = ?2
                     AND source.team = ?3
                     AND source.club_name = p.current_club
               )",
        )
        .map_err(|error| error.to_string())?;
    let candidates = statement
        .query_map(
            params![
                snapshot_id,
                save_id,
                team.as_str(),
                lane.ip_role_id,
                lane.oop_role_id,
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<u8>>(3)?,
                    row.get::<_, Option<u8>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                ))
            },
        )
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    let mut candidates = candidates
        .into_iter()
        .filter(|(_, name, _, _, _, _, _, _, _)| name.to_lowercase().contains(&search))
        .map(
            |(
                player_uid,
                name,
                current_club,
                ip_score,
                oop_score,
                assignment_team,
                assignment_string_id,
                assignment_string_order,
                assignment_lane_id,
            )| {
                let assignment_location = match (
                    assignment_team,
                    assignment_string_id,
                    assignment_string_order,
                    assignment_lane_id,
                ) {
                    (Some(team), Some(string_id), Some(string_order), Some(lane_id)) => {
                        Some(PlannerAssignmentLocation {
                            team: PlannerTeam::parse(&team)?,
                            string_id,
                            string_order,
                            lane_id,
                        })
                    }
                    _ => None,
                };
                Ok(PlannerSlotCandidate {
                    player_uid,
                    name,
                    current_club,
                    ip_score,
                    oop_score,
                    combined_score: combine_role_scores(ip_score, oop_score, lane.ip_weight),
                    assignment_location,
                })
            },
        )
        .collect::<Result<Vec<_>, String>>()?;
    candidates.sort_by(|left, right| {
        right
            .combined_score
            .cmp(&left.combined_score)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
            .then_with(|| left.player_uid.cmp(&right.player_uid))
    });
    candidates.truncate(MAX_SLOT_CANDIDATES);
    Ok(candidates)
}

pub fn add_string(
    conn: &Connection,
    save_id: i64,
    team: PlannerTeam,
) -> Result<PlannerString, String> {
    ensure_depth(conn, save_id)?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let string_order: i64 = tx
        .query_row(
            "SELECT COALESCE(MAX(string_order), -1) + 1
             FROM planner_strings
             WHERE save_id = ?1 AND team = ?2",
            params![save_id, team.as_str()],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    tx.execute(
        "INSERT INTO planner_strings (save_id, team, string_order)
         VALUES (?1, ?2, ?3)",
        params![save_id, team.as_str(), string_order],
    )
    .map_err(|error| error.to_string())?;
    let id = tx.last_insert_rowid();
    tx.commit().map_err(|error| error.to_string())?;

    Ok(PlannerString {
        id,
        string_order,
        assignments: Vec::new(),
    })
}

pub fn remove_string(
    conn: &Connection,
    save_id: i64,
    string_id: i64,
    confirm_populated: bool,
) -> Result<(), String> {
    ensure_depth(conn, save_id)?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let team = string_team(&tx, save_id, string_id)?;
    let string_count: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM planner_strings WHERE save_id = ?1 AND team = ?2",
            params![save_id, team.as_str()],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if string_count <= 1 {
        return Err(format!(
            "The {} team must keep at least one string",
            team.as_str()
        ));
    }

    let assignment_count: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM planner_assignments WHERE string_id = ?1",
            params![string_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if assignment_count > 0 && !confirm_populated {
        return Err("Removing a populated string requires confirmation".to_string());
    }

    tx.execute(
        "DELETE FROM planner_assignments WHERE save_id = ?1 AND string_id = ?2",
        params![save_id, string_id],
    )
    .map_err(|error| error.to_string())?;
    tx.execute(
        "DELETE FROM planner_strings WHERE id = ?1 AND save_id = ?2",
        params![string_id, save_id],
    )
    .map_err(|error| error.to_string())?;
    let mut statement = tx
        .prepare(
            "SELECT id
             FROM planner_strings
             WHERE save_id = ?1 AND team = ?2
             ORDER BY string_order",
        )
        .map_err(|error| error.to_string())?;
    let string_ids = statement
        .query_map(params![save_id, team.as_str()], |row| row.get::<_, i64>(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    drop(statement);
    for (string_order, remaining_string_id) in string_ids.into_iter().enumerate() {
        tx.execute(
            "UPDATE planner_strings SET string_order = ?1 WHERE id = ?2",
            params![string_order as i64, remaining_string_id],
        )
        .map_err(|error| error.to_string())?;
    }
    tx.commit().map_err(|error| error.to_string())
}

pub fn clear_team(
    conn: &Connection,
    save_id: i64,
    team: PlannerTeam,
    confirmed: bool,
) -> Result<(), String> {
    if !confirmed {
        return Err("Clearing a squad requires confirmation".to_string());
    }
    ensure_depth(conn, save_id)?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    tx.execute(
        "DELETE FROM planner_assignments
         WHERE save_id = ?1
           AND string_id IN (
             SELECT id FROM planner_strings WHERE save_id = ?1 AND team = ?2
           )",
        params![save_id, team.as_str()],
    )
    .map_err(|error| error.to_string())?;
    tx.commit().map_err(|error| error.to_string())
}

pub fn clear_assignment(
    conn: &Connection,
    save_id: i64,
    string_id: i64,
    lane_id: &str,
) -> Result<(), String> {
    let tactic = ensure_depth(conn, save_id)?;
    find_lane(&tactic, lane_id)?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    string_team(&tx, save_id, string_id)?;
    tx.execute(
        "DELETE FROM planner_assignments
         WHERE save_id = ?1 AND string_id = ?2 AND lane_id = ?3",
        params![save_id, string_id, lane_id],
    )
    .map_err(|error| error.to_string())?;
    tx.commit().map_err(|error| error.to_string())
}

pub fn assign_player(
    conn: &Connection,
    save_id: i64,
    string_id: i64,
    lane_id: &str,
    player_uid: i64,
) -> Result<(), String> {
    let tactic = ensure_depth(conn, save_id)?;
    find_lane(&tactic, lane_id)?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let team = string_team(&tx, save_id, string_id)?;
    ensure_cell_is_empty(&tx, string_id, lane_id)?;
    let last_known_name = assignable_player_name(&tx, save_id, team, player_uid)?;
    ensure_player_is_unassigned(&tx, save_id, player_uid)?;
    insert_assignment(
        &tx,
        save_id,
        string_id,
        lane_id,
        player_uid,
        &last_known_name,
        AssignmentProvenance::Manual,
    )?;
    tx.commit().map_err(|error| error.to_string())
}

pub fn move_player(
    conn: &Connection,
    save_id: i64,
    string_id: i64,
    lane_id: &str,
    player_uid: i64,
) -> Result<(), String> {
    let tactic = ensure_depth(conn, save_id)?;
    find_lane(&tactic, lane_id)?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let team = string_team(&tx, save_id, string_id)?;
    ensure_cell_is_empty(&tx, string_id, lane_id)?;
    let existing_assignment_id: i64 = tx
        .query_row(
            "SELECT id FROM planner_assignments WHERE save_id = ?1 AND player_uid = ?2",
            params![save_id, player_uid],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Player {player_uid} is not assigned"))?;
    let last_known_name = assignable_player_name(&tx, save_id, team, player_uid)?;
    tx.execute(
        "DELETE FROM planner_assignments WHERE id = ?1",
        params![existing_assignment_id],
    )
    .map_err(|error| error.to_string())?;
    insert_assignment(
        &tx,
        save_id,
        string_id,
        lane_id,
        player_uid,
        &last_known_name,
        AssignmentProvenance::Manual,
    )?;
    tx.commit().map_err(|error| error.to_string())
}

fn normalize_candidate_search(search: &str) -> Result<String, String> {
    let search = search.trim();
    if search.chars().count() > MAX_CANDIDATE_SEARCH_LEN {
        return Err(format!(
            "Candidate search must be at most {MAX_CANDIDATE_SEARCH_LEN} characters"
        ));
    }
    Ok(search.to_lowercase())
}

pub(super) fn ensure_depth(conn: &Connection, save_id: i64) -> Result<PlannerTactic, String> {
    let tactic = tactic::get_tactic(conn, save_id)?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    for team in PLANNER_TEAMS {
        tx.execute(
            "INSERT INTO planner_strings (save_id, team, string_order)
             SELECT ?1, ?2, 0
             WHERE NOT EXISTS (
                 SELECT 1
                 FROM planner_strings
                 WHERE save_id = ?1 AND team = ?2
             )",
            params![save_id, team.as_str()],
        )
        .map_err(|error| error.to_string())?;
    }
    tx.commit().map_err(|error| error.to_string())?;
    Ok(tactic)
}

pub(super) fn current_snapshot_id(conn: &Connection, save_id: i64) -> Result<Option<i64>, String> {
    conn.query_row(
        "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
        params![save_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|error| error.to_string())
}

fn load_assignments(
    conn: &Connection,
    save_id: i64,
    string_id: i64,
    team: PlannerTeam,
    snapshot_id: Option<i64>,
    tactic: &PlannerTactic,
) -> Result<Vec<PlannerAssignment>, String> {
    let mut statement = conn
        .prepare(
            "SELECT id, lane_id, player_uid, last_known_name
             FROM planner_assignments
             WHERE string_id = ?1
             ORDER BY id",
        )
        .map_err(|error| error.to_string())?;
    let assignments = statement
        .query_map(params![string_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    assignments
        .into_iter()
        .map(|(id, lane_id, player_uid, last_known_name)| {
            let lane = find_lane(tactic, &lane_id)?;
            let resolved = resolve_assignment(conn, save_id, team, snapshot_id, player_uid, lane)?;
            Ok(PlannerAssignment {
                id,
                lane_id,
                player_uid,
                last_known_name,
                current_name: resolved.current_name,
                state: resolved.state,
                combined_score: resolved.combined_score,
            })
        })
        .collect()
}

struct ResolvedAssignment {
    current_name: Option<String>,
    state: AssignmentState,
    combined_score: Option<u8>,
}

fn resolve_assignment(
    conn: &Connection,
    save_id: i64,
    team: PlannerTeam,
    snapshot_id: Option<i64>,
    player_uid: i64,
    lane: &TacticLane,
) -> Result<ResolvedAssignment, String> {
    let Some(snapshot_id) = snapshot_id else {
        return Ok(ResolvedAssignment {
            current_name: None,
            state: AssignmentState::Unresolved,
            combined_score: None,
        });
    };

    let player = conn
        .query_row(
            "SELECT
                p.name,
                EXISTS(
                    SELECT 1
                    FROM planner_club_sources source
                    WHERE source.save_id = ?1
                      AND source.team = ?2
                      AND source.club_name = p.current_club
                ),
                ip.score,
                oop.score
             FROM players p
             LEFT JOIN player_role_scores ip
               ON ip.snapshot_id = p.snapshot_id
              AND ip.uid = p.uid
              AND ip.role_id = ?5
             LEFT JOIN player_role_scores oop
               ON oop.snapshot_id = p.snapshot_id
              AND oop.uid = p.uid
              AND oop.role_id = ?6
             WHERE p.snapshot_id = ?3 AND p.uid = ?4",
            params![
                save_id,
                team.as_str(),
                snapshot_id,
                player_uid,
                lane.ip_role_id,
                lane.oop_role_id,
            ],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i32>(1)? == 1,
                    row.get::<_, Option<u8>>(2)?,
                    row.get::<_, Option<u8>>(3)?,
                ))
            },
        )
        .optional()
        .map_err(|error| error.to_string())?;

    let Some((current_name, is_in_pool, ip_score, oop_score)) = player else {
        return Ok(ResolvedAssignment {
            current_name: None,
            state: AssignmentState::Unresolved,
            combined_score: None,
        });
    };
    let state = if is_in_pool {
        AssignmentState::Resolved
    } else {
        AssignmentState::OutsidePool
    };

    Ok(ResolvedAssignment {
        current_name: Some(current_name),
        state,
        combined_score: combine_role_scores(ip_score, oop_score, lane.ip_weight),
    })
}

fn find_lane<'a>(tactic: &'a PlannerTactic, lane_id: &str) -> Result<&'a TacticLane, String> {
    tactic
        .lanes
        .iter()
        .find(|lane| lane.lane_id == lane_id)
        .ok_or_else(|| format!("Unknown tactic lane `{lane_id}`"))
}

fn string_team(conn: &Connection, save_id: i64, string_id: i64) -> Result<PlannerTeam, String> {
    let team = conn
        .query_row(
            "SELECT team FROM planner_strings WHERE id = ?1 AND save_id = ?2",
            params![string_id, save_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Planner string {string_id} not found"))?;
    PlannerTeam::parse(&team)
}

fn ensure_cell_is_empty(conn: &Connection, string_id: i64, lane_id: &str) -> Result<(), String> {
    let occupied: bool = conn
        .query_row(
            "SELECT EXISTS(
                 SELECT 1 FROM planner_assignments WHERE string_id = ?1 AND lane_id = ?2
             )",
            params![string_id, lane_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if occupied {
        return Err("Planner cell is already occupied".to_string());
    }
    Ok(())
}

fn ensure_player_is_unassigned(
    conn: &Connection,
    save_id: i64,
    player_uid: i64,
) -> Result<(), String> {
    let assigned: bool = conn
        .query_row(
            "SELECT EXISTS(
                 SELECT 1 FROM planner_assignments WHERE save_id = ?1 AND player_uid = ?2
             )",
            params![save_id, player_uid],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if assigned {
        return Err(format!("Player {player_uid} is already assigned"));
    }
    Ok(())
}

fn assignable_player_name(
    conn: &Connection,
    save_id: i64,
    team: PlannerTeam,
    player_uid: i64,
) -> Result<String, String> {
    let snapshot_id = current_snapshot_id(conn, save_id)?
        .ok_or_else(|| "No current snapshot loaded for this save".to_string())?;
    conn.query_row(
        "SELECT p.name
         FROM players p
         WHERE p.snapshot_id = ?1
           AND p.uid = ?2
           AND EXISTS(
               SELECT 1
               FROM planner_club_sources source
               WHERE source.save_id = ?3
                 AND source.team = ?4
                 AND source.club_name = p.current_club
           )",
        params![snapshot_id, player_uid, save_id, team.as_str()],
        |row| row.get(0),
    )
    .optional()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| {
        format!(
            "Player {player_uid} is not available to the {} team",
            team.as_str()
        )
    })
}

pub(super) fn insert_assignment(
    tx: &rusqlite::Transaction<'_>,
    save_id: i64,
    string_id: i64,
    lane_id: &str,
    player_uid: i64,
    last_known_name: &str,
    provenance: AssignmentProvenance,
) -> Result<(), String> {
    tx.execute(
        "INSERT INTO planner_assignments (
             save_id, string_id, lane_id, player_uid, last_known_name, provenance
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            save_id,
            string_id,
            lane_id,
            player_uid,
            last_known_name,
            provenance.as_str(),
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}
