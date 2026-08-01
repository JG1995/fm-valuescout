use rusqlite::{params, Connection, OptionalExtension};

use crate::features::scoring::combine::combine_role_scores;

use super::tactic::{self, PlannerTactic, TacticLane};

const PLANNER_TEAMS: [PlannerTeam; 3] = [
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
                     AND (source.team_level IS NULL OR source.team_level = p.team_level)
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
                    combined_score: combine_role_scores(ip_score, oop_score, tactic.ip_weight),
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

fn ensure_depth(conn: &Connection, save_id: i64) -> Result<PlannerTactic, String> {
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

fn current_snapshot_id(conn: &Connection, save_id: i64) -> Result<Option<i64>, String> {
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
            let resolved = resolve_assignment(
                conn,
                save_id,
                team,
                snapshot_id,
                player_uid,
                lane,
                tactic.ip_weight,
            )?;
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
    ip_weight: f64,
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
                      AND (source.team_level IS NULL OR source.team_level = p.team_level)
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
        combined_score: combine_role_scores(ip_score, oop_score, ip_weight),
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
                 AND (source.team_level IS NULL OR source.team_level = p.team_level)
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

fn insert_assignment(
    tx: &rusqlite::Transaction<'_>,
    save_id: i64,
    string_id: i64,
    lane_id: &str,
    player_uid: i64,
    last_known_name: &str,
) -> Result<(), String> {
    tx.execute(
        "INSERT INTO planner_assignments (
             save_id, string_id, lane_id, player_uid, last_known_name
         ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![save_id, string_id, lane_id, player_uid, last_known_name],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use crate::db::migrations;
    use crate::features::planner::service::{self, ClubSourceInput};
    use crate::features::planner::tactic;
    use crate::features::snapshot;

    use super::{
        add_string, assign_player, clear_assignment, get_depth, get_slot_candidates, move_player,
        remove_string, AssignmentState, PlannerTeam,
    };

    fn open_with_snapshot() -> (tempfile::TempDir, Connection, i64) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = Connection::open(temp_dir.path().join("planner-depth.db")).expect("open db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        let save = snapshot::service::list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        let dump_path = temp_dir.path().join("dump.json");
        std::fs::write(
            &dump_path,
            include_str!("../memory_read/fixtures/golden_dump_v5.json"),
        )
        .expect("write dump");
        snapshot::ingest::ingest_dump_file(&mut conn, &dump_path).expect("ingest dump");
        service::save_club_family(&conn, save.id, "Loan FC", &[]).expect("configure club family");
        (temp_dir, conn, save.id)
    }

    fn team_strings(depth: &super::PlannerDepth, team: PlannerTeam) -> &[super::PlannerString] {
        &depth
            .teams
            .iter()
            .find(|team_depth| team_depth.team == team)
            .expect("team depth")
            .strings
    }

    fn add_picker_candidates(temp_dir: &tempfile::TempDir, conn: &mut Connection, save_id: i64) {
        let dump_path = temp_dir.path().join("picker-candidates.json");
        let mut dump: serde_json::Value =
            serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v5.json"))
                .expect("parse golden dump");
        let original = dump["players"][0].clone();
        let mut reserve = original.clone();
        reserve["uid"] = serde_json::Value::Number(78.into());
        reserve["name"] = serde_json::Value::String("Reserve Player".to_string());
        reserve["teamLevel"] = serde_json::Value::String("reserve".to_string());
        let mut b_team = original.clone();
        b_team["uid"] = serde_json::Value::Number(79.into());
        b_team["name"] = serde_json::Value::String("B Team Player".to_string());
        b_team["currentClub"] = serde_json::Value::String("Loan B FC".to_string());
        let mut unknown = b_team.clone();
        unknown["uid"] = serde_json::Value::Number(80.into());
        unknown["name"] = serde_json::Value::String("Unknown Score Player".to_string());
        dump["players"] = serde_json::Value::Array(vec![original, reserve, b_team, unknown]);
        dump["playerCount"] = serde_json::Value::Number(4.into());
        std::fs::write(
            &dump_path,
            serde_json::to_string(&dump).expect("serialize picker candidates"),
        )
        .expect("write picker candidates");
        snapshot::ingest::ingest_dump_file_for_save(conn, save_id, &dump_path)
            .expect("ingest picker candidates");
        service::save_club_family(
            conn,
            save_id,
            "Loan FC",
            &[ClubSourceInput {
                team: "reserves".to_string(),
                club_name: "Loan B FC".to_string(),
                team_level: None,
            }],
        )
        .expect("configure B-team source");
    }

    #[test]
    fn returns_ranked_candidates_from_the_target_team_club_family() {
        let (temp_dir, mut conn, save_id) = open_with_snapshot();
        add_picker_candidates(&temp_dir, &mut conn, save_id);
        let snapshot_id: i64 = conn
            .query_row(
                "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
                params![save_id],
                |row| row.get(0),
            )
            .expect("current snapshot");
        conn.execute(
            "UPDATE player_role_scores
             SET score = CASE uid
                 WHEN 78 THEN 50
                 WHEN 79 THEN 80
                 WHEN 80 THEN NULL
                 ELSE score
             END
             WHERE snapshot_id = ?1
               AND role_id IN ('goalkeeper_ip', 'line_holding_keeper_oop')",
            params![snapshot_id],
        )
        .expect("set candidate scores");
        let depth = get_depth(&conn, save_id).expect("create planner depth");
        let reserve_string_id = team_strings(&depth, PlannerTeam::Reserves)[0].id;
        assign_player(&conn, save_id, reserve_string_id, "goalkeeper", 78)
            .expect("assign reserve player");

        let candidates =
            get_slot_candidates(&conn, save_id, PlannerTeam::Reserves, "goalkeeper", "")
                .expect("load reserve candidates");

        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.player_uid)
                .collect::<Vec<_>>(),
            [79, 78, 80]
        );
        assert_eq!(candidates[0].combined_score, Some(80));
        assert_eq!(candidates[1].combined_score, Some(50));
        assert_eq!(candidates[2].combined_score, None);
        assert_eq!(candidates[0].current_club, "Loan B FC");
        assert_eq!(
            candidates[1].assignment_location.as_ref().map(|location| (
                location.team.as_str(),
                location.string_id,
                location.lane_id.as_str()
            )),
            Some(("reserves", reserve_string_id, "goalkeeper"))
        );

        let searched = get_slot_candidates(
            &conn,
            save_id,
            PlannerTeam::Reserves,
            "goalkeeper",
            "b team",
        )
        .expect("search reserve candidates");
        assert_eq!(
            searched
                .iter()
                .map(|candidate| candidate.player_uid)
                .collect::<Vec<_>>(),
            [79]
        );

        let error = get_slot_candidates(
            &conn,
            save_id,
            PlannerTeam::Reserves,
            "goalkeeper",
            &"x".repeat(121),
        )
        .expect_err("reject an unbounded search");
        assert_eq!(error, "Candidate search must be at most 120 characters");
    }

    #[test]
    fn creates_one_default_string_for_each_team() {
        let (_temp_dir, conn, save_id) = open_with_snapshot();

        let depth = get_depth(&conn, save_id).expect("create planner depth");

        assert_eq!(
            depth
                .teams
                .iter()
                .map(|team| (team.team.as_str(), team.strings.len()))
                .collect::<Vec<_>>(),
            [("senior", 1), ("reserves", 1), ("youth", 1)]
        );
        assert!(depth
            .teams
            .iter()
            .all(|team| team.strings[0].string_order == 0));
    }

    #[test]
    fn adds_ordered_strings_and_rejects_removing_the_final_string() {
        let (_temp_dir, conn, save_id) = open_with_snapshot();
        let first = get_depth(&conn, save_id).expect("create planner depth");
        let first_senior_id = team_strings(&first, PlannerTeam::Senior)[0].id;

        let added = add_string(&conn, save_id, PlannerTeam::Senior).expect("add string");
        assert_eq!(added.string_order, 1);
        remove_string(&conn, save_id, first_senior_id, false).expect("remove empty string");

        let error =
            remove_string(&conn, save_id, added.id, false).expect_err("keep the final string");
        assert!(error.contains("at least one string"));
        let reloaded = get_depth(&conn, save_id).expect("reload depth");
        let strings = team_strings(&reloaded, PlannerTeam::Senior);
        assert_eq!(
            strings
                .iter()
                .map(|string| string.string_order)
                .collect::<Vec<_>>(),
            [0]
        );

        let next = add_string(&conn, save_id, PlannerTeam::Senior).expect("add next string");
        assert_eq!(next.string_order, 1);
    }

    #[test]
    fn populated_string_requires_confirmation_and_deletes_only_its_assignments() {
        let (_temp_dir, conn, save_id) = open_with_snapshot();
        let depth = get_depth(&conn, save_id).expect("create planner depth");
        let populated_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
        let remaining = add_string(&conn, save_id, PlannerTeam::Senior).expect("add string");
        assign_player(&conn, save_id, populated_id, "goalkeeper", 77).expect("assign player");

        let error = remove_string(&conn, save_id, populated_id, false)
            .expect_err("require populated confirmation");
        assert!(error.contains("requires confirmation"));
        assert_eq!(
            team_strings(
                &get_depth(&conn, save_id).expect("reload depth"),
                PlannerTeam::Senior
            )[0]
            .assignments
            .len(),
            1
        );

        remove_string(&conn, save_id, populated_id, true).expect("remove confirmed string");
        let reloaded = get_depth(&conn, save_id).expect("reload depth");
        let strings = team_strings(&reloaded, PlannerTeam::Senior);
        assert_eq!(strings.len(), 1);
        assert_eq!(strings[0].id, remaining.id);
        assert!(strings[0].assignments.is_empty());
    }

    #[test]
    fn enforces_player_uniqueness_and_moves_in_one_save() {
        let (_temp_dir, conn, save_id) = open_with_snapshot();
        service::save_club_family(
            &conn,
            save_id,
            "Loan FC",
            &[ClubSourceInput {
                team: "reserves".to_string(),
                club_name: "Loan FC".to_string(),
                team_level: None,
            }],
        )
        .expect("add reserve source");
        let depth = get_depth(&conn, save_id).expect("create planner depth");
        let first_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
        let reserve_id = team_strings(&depth, PlannerTeam::Reserves)[0].id;
        let second = add_string(&conn, save_id, PlannerTeam::Senior).expect("add string");
        assign_player(&conn, save_id, first_id, "goalkeeper", 77).expect("assign player");

        let error = assign_player(&conn, save_id, second.id, "goalkeeper", 77)
            .expect_err("reject duplicate player");
        assert!(error.contains("already assigned"));
        let error = assign_player(&conn, save_id, reserve_id, "goalkeeper", 77)
            .expect_err("reject duplicate player across teams");
        assert!(error.contains("already assigned"));

        move_player(&conn, save_id, second.id, "goalkeeper", 77).expect("move player");
        let reloaded = get_depth(&conn, save_id).expect("reload depth");
        let strings = team_strings(&reloaded, PlannerTeam::Senior);
        assert!(strings
            .iter()
            .any(|string| string.id == first_id && string.assignments.is_empty()));
        assert!(strings.iter().any(|string| {
            string.id == second.id
                && string
                    .assignments
                    .iter()
                    .any(|assignment| assignment.player_uid == 77)
        }));

        clear_assignment(&conn, save_id, second.id, "goalkeeper").expect("clear assignment");
        assert!(team_strings(
            &get_depth(&conn, save_id).expect("reload after clear"),
            PlannerTeam::Senior,
        )
        .iter()
        .all(|string| string.assignments.is_empty()));
    }

    #[test]
    fn preserves_assignment_as_unresolved_when_snapshot_replaces_player() {
        let (temp_dir, mut conn, save_id) = open_with_snapshot();
        let depth = get_depth(&conn, save_id).expect("create planner depth");
        let string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
        assign_player(&conn, save_id, string_id, "goalkeeper", 77).expect("assign player");

        let replacement_path = temp_dir.path().join("replacement.json");
        let replacement = include_str!("../memory_read/fixtures/golden_dump_v5.json")
            .replace("\"uid\": 77", "\"uid\": 78")
            .replace("\"name\": \"Loan Player\"", "\"name\": \"Replacement\"");
        std::fs::write(&replacement_path, replacement).expect("write replacement dump");
        snapshot::ingest::ingest_dump_file(&mut conn, &replacement_path).expect("replace snapshot");

        let reloaded = get_depth(&conn, save_id).expect("reload depth");
        let assignment = &team_strings(&reloaded, PlannerTeam::Senior)[0].assignments[0];
        assert_eq!(assignment.last_known_name, "Golden Fixture Player");
        assert_eq!(assignment.current_name, None);
        assert_eq!(assignment.state, AssignmentState::Unresolved);
        assert_eq!(assignment.combined_score, None);
    }

    #[test]
    fn resolves_combined_scores_and_marks_current_players_outside_the_pool() {
        let (temp_dir, mut conn, save_id) = open_with_snapshot();
        let depth = get_depth(&conn, save_id).expect("create planner depth");
        let string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
        assign_player(&conn, save_id, string_id, "goalkeeper", 77).expect("assign player");
        let snapshot_id: i64 = conn
            .query_row(
                "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
                params![save_id],
                |row| row.get(0),
            )
            .expect("current snapshot");
        conn.execute(
            "UPDATE player_role_scores
             SET score = CASE role_id
                 WHEN 'goalkeeper_ip' THEN 80
                 WHEN 'line_holding_keeper_oop' THEN 60
                 ELSE score
             END
             WHERE snapshot_id = ?1 AND uid = 77",
            params![snapshot_id],
        )
        .expect("set role scores");
        let scored = get_depth(&conn, save_id).expect("load score");
        let assignment = &team_strings(&scored, PlannerTeam::Senior)[0].assignments[0];
        assert_eq!(assignment.state, AssignmentState::Resolved);
        assert_eq!(assignment.combined_score, Some(70));

        let moved_path = temp_dir.path().join("moved.json");
        let moved = include_str!("../memory_read/fixtures/golden_dump_v5.json").replace(
            "\"currentClub\": \"Loan FC\"",
            "\"currentClub\": \"Other FC\"",
        );
        std::fs::write(&moved_path, moved).expect("write moved dump");
        snapshot::ingest::ingest_dump_file(&mut conn, &moved_path).expect("replace snapshot");

        let reloaded = get_depth(&conn, save_id).expect("reload depth");
        let assignment = &team_strings(&reloaded, PlannerTeam::Senior)[0].assignments[0];
        assert_eq!(assignment.state, AssignmentState::OutsidePool);
        assert_eq!(assignment.combined_score, None);
    }

    #[test]
    fn source_and_tactic_updates_preserve_existing_assignments_and_saves_are_isolated() {
        let (temp_dir, mut conn, save_id) = open_with_snapshot();
        let depth = get_depth(&conn, save_id).expect("create planner depth");
        let string_id = team_strings(&depth, PlannerTeam::Senior)[0].id;
        assign_player(&conn, save_id, string_id, "goalkeeper", 77).expect("assign player");

        service::save_club_family(
            &conn,
            save_id,
            "Loan FC",
            &[ClubSourceInput {
                team: "reserves".to_string(),
                club_name: "Loan FC".to_string(),
                team_level: None,
            }],
        )
        .expect("replace sources");
        let mut tactic = tactic::get_tactic(&conn, save_id).expect("load tactic");
        tactic.lanes[0].ip_role_id = "ball_playing_goalkeeper_ip".to_string();
        tactic::save_tactic(&conn, save_id, &tactic).expect("change tactic role");
        assert_eq!(
            team_strings(
                &get_depth(&conn, save_id).expect("reload depth"),
                PlannerTeam::Senior
            )[0]
            .assignments
            .len(),
            1
        );

        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Second save', 0)",
            [],
        )
        .expect("create second save");
        let second_save_id = conn.last_insert_rowid();
        let second_dump_path = temp_dir.path().join("second-save.json");
        std::fs::write(
            &second_dump_path,
            include_str!("../memory_read/fixtures/golden_dump_v5.json"),
        )
        .expect("write second save dump");
        snapshot::ingest::ingest_dump_file_for_save(&mut conn, second_save_id, &second_dump_path)
            .expect("ingest second save");
        service::save_club_family(&conn, second_save_id, "Loan FC", &[])
            .expect("configure second save");
        let second_depth = get_depth(&conn, second_save_id).expect("create isolated depth");
        let second_string_id = team_strings(&second_depth, PlannerTeam::Senior)[0].id;
        assign_player(&conn, second_save_id, second_string_id, "goalkeeper", 77)
            .expect("assign same player uid in second save");
        assert!(second_depth
            .teams
            .iter()
            .all(|team| team.strings[0].assignments.is_empty()));
    }
}
