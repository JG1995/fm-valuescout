//! Moving the managed club's players into the FM teams their Planner squads stand for.
//!
//! Senior stands for the first team, Reserves for the club's reserve team, and Youth for its
//! youth team. The bridge picks the actual FM team at each level and makes each move through
//! FM's own squad action; this module plans the moves and records the ones FM confirms.

use rusqlite::{params, Connection, OptionalExtension};

use super::depth::{current_snapshot_id, PlannerTeam};
use crate::db::Db;
use crate::features::managed_club::service::{managed_club_player_sql, selected_club_identity};
use crate::features::memory_read::service::{SquadMoveItem, SquadMoveResponse, MAX_SQUAD_MOVES};
use crate::features::player::boost_gate;
use crate::features::snapshot::service;

/// The FM team level a Planner squad stands for.
pub(super) fn level_for(team: PlannerTeam) -> &'static str {
    match team {
        PlannerTeam::Senior => "senior",
        PlannerTeam::Reserves => "reserve",
        PlannerTeam::Youth => "youth",
    }
}

/// A Planner-placed player whose FM team is at another level than his Planner squad.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FmSquadMove {
    pub(super) player_uid: i64,
    pub(super) name: String,
    pub(super) age: Option<i64>,
    /// The level of his FM team now: `senior`, `reserve`, or `youth`.
    pub(super) from_level: String,
    pub(super) to_team: PlannerTeam,
    pub(super) to_team_name: String,
    /// Older than the target squad's age limit. FM does not check age, so he moves anyway.
    pub(super) over_age: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct FmSquadMovePlan {
    pub(super) moves: Vec<FmSquadMove>,
    /// Planner-placed players whose FM team the snapshot does not know, so no move is planned.
    pub(super) unknown_team: Vec<(i64, String)>,
}

/// The moves that would put every Planner-placed player at the managed club in the FM team
/// his squad stands for. Players outside the Planner, and players no longer at the club, stay
/// out of the plan.
pub(super) fn plan_fm_squad_moves(
    conn: &Connection,
    save_id: i64,
) -> Result<FmSquadMovePlan, String> {
    let Some(snapshot_id) = current_snapshot_id(conn, save_id)? else {
        return Ok(FmSquadMovePlan::default());
    };
    let sql = format!(
        "SELECT assignment.player_uid, p.name, p.age, p.team_level, planner_string.team,
                team.display_name, team.max_age
         FROM planner_assignments assignment
         INNER JOIN planner_strings planner_string ON planner_string.id = assignment.string_id
         INNER JOIN planner_teams team
           ON team.save_id = assignment.save_id AND team.team = planner_string.team
         INNER JOIN players p ON p.snapshot_id = ?2 AND p.uid = assignment.player_uid
         WHERE assignment.save_id = ?1 AND {}",
        managed_club_player_sql("p", "?1")
    );
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![save_id, snapshot_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<i64>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<i64>>(6)?,
            ))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    let mut plan = FmSquadMovePlan::default();
    for (player_uid, name, age, from_level, team, to_team_name, max_age) in rows {
        let to_team = PlannerTeam::parse(&team)?;
        match from_level {
            None => plan.unknown_team.push((player_uid, name)),
            Some(level) if level == level_for(to_team) => {}
            Some(from_level) => plan.moves.push(FmSquadMove {
                player_uid,
                name,
                age,
                from_level,
                to_team,
                to_team_name,
                over_age: matches!((age, max_age), (Some(age), Some(max)) if age > max),
            }),
        }
    }
    plan.moves.sort_by(|left, right| {
        team_order(left.to_team)
            .cmp(&team_order(right.to_team))
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
            .then_with(|| left.player_uid.cmp(&right.player_uid))
    });
    plan.unknown_team
        .sort_by_key(|(_, name)| name.to_lowercase());
    Ok(plan)
}

fn team_order(team: PlannerTeam) -> u8 {
    match team {
        PlannerTeam::Senior => 0,
        PlannerTeam::Reserves => 1,
        PlannerTeam::Youth => 2,
    }
}

/// What a squad move needs from the app: the live scan behind the current snapshot and the
/// managed club's FM identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FmSquadMoveContext {
    pub(super) save_id: i64,
    pub(super) snapshot_id: i64,
    pub(super) source_request_id: String,
    pub(super) club_uid: u32,
}

pub(super) const LOAD_DATA_FIRST: &str = "Load Data with FM open on this save before moving squads";

pub(super) fn capture_move_context(conn: &Connection) -> Result<FmSquadMoveContext, String> {
    let save_id = service::active_save_id(conn)?;
    let snapshot: Option<(i64, Option<String>)> = conn
        .query_row(
            "SELECT id, bridge_source_request_id FROM snapshots
             WHERE save_id = ?1 AND is_current = 1",
            params![save_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let Some((snapshot_id, Some(source_request_id))) = snapshot else {
        return Err(LOAD_DATA_FIRST.to_string());
    };
    if source_request_id.trim().is_empty() {
        return Err(LOAD_DATA_FIRST.to_string());
    }
    let club_uid =
        match selected_club_identity(conn, save_id)? {
            Some((_, Some(uid))) => u32::try_from(uid)
                .map_err(|_| "The managed club's FM identity is invalid".to_string())?,
            Some((_, None)) => return Err(
                "Choose the managed club again in My Club so FM ValueScout records its FM identity"
                    .to_string(),
            ),
            None => return Err("Choose your managed club in My Club first".to_string()),
        };
    Ok(FmSquadMoveContext {
        save_id,
        snapshot_id,
        source_request_id,
        club_uid,
    })
}

/// A move FM did not make, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FmSquadMoveFailure {
    pub(super) player_uid: i64,
    pub(super) name: String,
    pub(super) reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct FmSquadMoveReport {
    /// Players FM moved.
    pub(super) moved: Vec<i64>,
    /// Players already in the right FM team, which the snapshot had not caught up with.
    pub(super) unchanged: Vec<i64>,
    pub(super) failed: Vec<FmSquadMoveFailure>,
    /// The error that stopped the request early, when one did.
    pub(super) error: Option<String>,
}

/// Moves the chosen players, those of the current plan whose UIDs `player_uids` lists, and
/// records each confirmed move's FM team in the current snapshot.
pub(super) fn apply_fm_squad_moves<F>(
    db: &Db,
    player_uids: &[i64],
    request: F,
) -> Result<FmSquadMoveReport, String>
where
    F: FnOnce(&str, u32, &[SquadMoveItem]) -> Result<SquadMoveResponse, String>,
{
    let _guard = boost_gate::acquire_boost_gate()?;
    let (context, moves) = {
        let conn =
            db.0.lock()
                .map_err(|_| "database lock poisoned".to_string())?;
        let context = capture_move_context(&conn)?;
        let moves = plan_fm_squad_moves(&conn, context.save_id)?
            .moves
            .into_iter()
            .filter(|planned| player_uids.contains(&planned.player_uid))
            .collect::<Vec<_>>();
        (context, moves)
    };
    if moves.is_empty() {
        return Err("None of the chosen players needs a move".to_string());
    }
    if moves.len() > MAX_SQUAD_MOVES {
        return Err(format!("Move at most {MAX_SQUAD_MOVES} players at a time"));
    }
    let items = moves
        .iter()
        .map(|planned| {
            Ok(SquadMoveItem {
                player_uid: u32::try_from(planned.player_uid)
                    .map_err(|_| format!("{} has an invalid player identity", planned.name))?,
                target_level: level_for(planned.to_team),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let response = request(&context.source_request_id, context.club_uid, &items)?;
    if response.moves.len() != moves.len() {
        return Err("The bridge reported a different number of moves than were sent".to_string());
    }

    let mut report = FmSquadMoveReport {
        error: response.error,
        ..FmSquadMoveReport::default()
    };
    let mut conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    for (planned, outcome) in moves.iter().zip(response.moves) {
        match outcome.outcome.as_str() {
            "moved" | "already-there" => {
                tx.execute(
                    "UPDATE players SET team_type = ?3, team_level = ?4
                     WHERE snapshot_id = ?1 AND uid = ?2",
                    params![
                        context.snapshot_id,
                        planned.player_uid,
                        outcome.team_type,
                        outcome.team_level
                    ],
                )
                .map_err(|error| error.to_string())?;
                if outcome.outcome == "moved" {
                    report.moved.push(planned.player_uid);
                } else {
                    report.unchanged.push(planned.player_uid);
                }
            }
            other => report.failed.push(FmSquadMoveFailure {
                player_uid: planned.player_uid,
                name: planned.name.clone(),
                reason: failure_reason(other, planned.to_team),
            }),
        }
    }
    tx.commit().map_err(|error| error.to_string())?;
    Ok(report)
}

fn failure_reason(outcome: &str, to_team: PlannerTeam) -> String {
    match outcome {
        "not-found" => "FM could not find him; Load Data and try again".to_string(),
        "not-in-club-teams" => {
            "He is not in one of the club's own squads in FM, for example a B team".to_string()
        }
        "no-target-team" => format!("The club has no {} team in FM", level_for(to_team)),
        "not-confirmed" => "FM did not confirm the move".to_string(),
        _ => "FM did not report an outcome".to_string(),
    }
}
