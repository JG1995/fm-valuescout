use std::collections::HashSet;

use rusqlite::{params, Connection, Transaction};

use crate::features::staff::assignment_targets;

use super::depth::{preflight_depth_snapshot, PlannerTeam};

pub(super) const MAX_DISPLAY_NAME_LEN: usize = 40;

const DEFAULT_TEAM_NAMES: [(PlannerTeam, &str); 3] = [
    (PlannerTeam::Senior, "Senior"),
    (PlannerTeam::Reserves, "Reserves"),
    (PlannerTeam::Youth, "Youth"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlannerTeamSetting {
    pub(super) team: PlannerTeam,
    pub(super) display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PlannerTeamInput {
    pub(super) team: String,
    pub(super) display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PlannerStaffingTargetRemovalImpact {
    pub(super) job_id: String,
    pub(super) job_label: String,
    pub(super) slot_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PlannerTeamRemovalImpact {
    pub(super) team: PlannerTeam,
    pub(super) display_name: String,
    pub(super) assignment_count: i64,
    pub(super) staffing_targets: Vec<PlannerStaffingTargetRemovalImpact>,
}

pub(super) fn get_team_settings(
    conn: &Connection,
    save_id: i64,
) -> Result<Vec<PlannerTeamSetting>, String> {
    load_team_settings(conn, save_id)
}

pub(crate) fn ensure_team_settings(
    conn: &Connection,
    save_id: i64,
) -> Result<Vec<PlannerTeamSetting>, String> {
    ensure_save_exists(conn, save_id)?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    ensure_team_rows(&tx, save_id)?;
    tx.commit().map_err(|error| error.to_string())?;
    load_team_settings(conn, save_id)
}

pub(super) fn ensure_available(
    conn: &Connection,
    save_id: i64,
    team: PlannerTeam,
) -> Result<(), String> {
    let available: bool = conn
        .query_row(
            "SELECT EXISTS(
                 SELECT 1 FROM planner_teams WHERE save_id = ?1 AND team = ?2
             )",
            params![save_id, team.as_str()],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !available {
        return Err(format!(
            "The {} team is not available in this save",
            team.as_str()
        ));
    }
    Ok(())
}

pub(super) fn planner_team_removal_impacts(
    conn: &Connection,
    save_id: i64,
    inputs: &[PlannerTeamInput],
) -> Result<Vec<PlannerTeamRemovalImpact>, String> {
    let desired = normalize_inputs(inputs)?;
    ensure_team_settings(conn, save_id)?;
    let current = load_team_settings(conn, save_id)?;
    current
        .into_iter()
        .filter(|setting| !desired.iter().any(|item| item.team == setting.team))
        .map(|setting| {
            let assignment_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM planner_assignments
                     WHERE save_id = ?1 AND string_id IN (
                         SELECT id FROM planner_strings WHERE save_id = ?1 AND team = ?2
                     )",
                    params![save_id, setting.team.as_str()],
                    |row| row.get(0),
                )
                .map_err(|error| error.to_string())?;
            let staffing_targets = assignment_targets::nonzero_targets_for_scope(
                conn,
                save_id,
                setting.team.as_str(),
            )?
            .into_iter()
            .map(|target| PlannerStaffingTargetRemovalImpact {
                job_id: target.job_id,
                job_label: target.job_label,
                slot_count: target.slot_count,
            })
            .collect();
            Ok(PlannerTeamRemovalImpact {
                team: setting.team,
                display_name: setting.display_name,
                assignment_count,
                staffing_targets,
            })
        })
        .collect()
}

pub(super) fn save_team_settings(
    conn: &Connection,
    save_id: i64,
    inputs: &[PlannerTeamInput],
    confirm_populated_removal: bool,
) -> Result<(Vec<PlannerTeamSetting>, Option<i64>), String> {
    let desired = normalize_inputs(inputs)?;
    let snapshot_id = preflight_depth_snapshot(conn, save_id)?;
    ensure_save_exists(conn, save_id)?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    ensure_team_rows(&tx, save_id)?;
    let current = load_team_settings_from_tx(&tx, save_id)?;
    let removed = current
        .iter()
        .filter(|setting| !desired.iter().any(|item| item.team == setting.team))
        .collect::<Vec<_>>();
    let impacts = removed
        .iter()
        .map(|setting| {
            let assignment_count = team_assignment_count(&tx, save_id, setting.team)?;
            let staffing_targets = assignment_targets::nonzero_targets_for_scope_tx(
                &tx,
                save_id,
                setting.team.as_str(),
            )?
            .into_iter()
            .map(|target| PlannerStaffingTargetRemovalImpact {
                job_id: target.job_id,
                job_label: target.job_label,
                slot_count: target.slot_count,
            })
            .collect::<Vec<_>>();
            Ok((setting, assignment_count, staffing_targets))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let populated = impacts
        .iter()
        .filter(|(_, assignment_count, staffing_targets)| {
            *assignment_count > 0 || !staffing_targets.is_empty()
        })
        .collect::<Vec<_>>();
    if !populated.is_empty() && !confirm_populated_removal {
        let teams = populated
            .iter()
            .map(|(setting, _, _)| setting.display_name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "Removing populated planner teams requires confirmation: {teams}"
        ));
    }

    for setting in removed {
        assignment_targets::delete_scope_targets(&tx, save_id, setting.team.as_str())?;
        tx.execute(
            "DELETE FROM planner_assignments
             WHERE save_id = ?1
               AND string_id IN (
                   SELECT id FROM planner_strings WHERE save_id = ?1 AND team = ?2
               )",
            params![save_id, setting.team.as_str()],
        )
        .map_err(|error| error.to_string())?;
        tx.execute(
            "DELETE FROM planner_strings WHERE save_id = ?1 AND team = ?2",
            params![save_id, setting.team.as_str()],
        )
        .map_err(|error| error.to_string())?;
        tx.execute(
            "DELETE FROM planner_teams WHERE save_id = ?1 AND team = ?2",
            params![save_id, setting.team.as_str()],
        )
        .map_err(|error| error.to_string())?;
    }

    tx.execute("DELETE FROM planner_teams WHERE save_id = ?1", [save_id])
        .map_err(|error| error.to_string())?;

    for setting in &desired {
        let was_available = current.iter().any(|item| item.team == setting.team);
        tx.execute(
            "INSERT INTO planner_teams (save_id, team, display_name)
             VALUES (?1, ?2, ?3)",
            params![save_id, setting.team.as_str(), setting.display_name],
        )
        .map_err(|error| error.to_string())?;
        if !was_available {
            tx.execute(
                "INSERT INTO planner_strings (save_id, team, string_order)
                 VALUES (?1, ?2, 0)",
                params![save_id, setting.team.as_str()],
            )
            .map_err(|error| error.to_string())?;
        }
    }

    tx.commit().map_err(|error| error.to_string())?;
    Ok((load_team_settings(conn, save_id)?, snapshot_id))
}

pub(super) fn load_team_settings_from_tx(
    tx: &Transaction<'_>,
    save_id: i64,
) -> Result<Vec<PlannerTeamSetting>, String> {
    load_team_settings_from_query(tx, save_id)
}

fn load_team_settings(conn: &Connection, save_id: i64) -> Result<Vec<PlannerTeamSetting>, String> {
    load_team_settings_from_query(conn, save_id)
}

fn load_team_settings_from_query(
    conn: &Connection,
    save_id: i64,
) -> Result<Vec<PlannerTeamSetting>, String> {
    let mut statement = conn
        .prepare(
            "SELECT team, display_name
             FROM planner_teams
             WHERE save_id = ?1
             ORDER BY CASE team
                 WHEN 'senior' THEN 0
                 WHEN 'reserves' THEN 1
                 WHEN 'youth' THEN 2
             END",
        )
        .map_err(|error| error.to_string())?;
    let settings = statement
        .query_map(params![save_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| error.to_string())?
        .map(|row| {
            let (team, display_name) = row.map_err(|error| error.to_string())?;
            Ok(PlannerTeamSetting {
                team: PlannerTeam::parse(&team)?,
                display_name,
            })
        })
        .collect::<Result<Vec<_>, String>>();
    settings
}

fn ensure_team_rows(tx: &Transaction<'_>, save_id: i64) -> Result<(), String> {
    let has_settings: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM planner_teams WHERE save_id = ?1)",
            params![save_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if has_settings {
        return Ok(());
    }
    for (team, display_name) in DEFAULT_TEAM_NAMES {
        tx.execute(
            "INSERT INTO planner_teams (save_id, team, display_name)
             VALUES (?1, ?2, ?3)",
            params![save_id, team.as_str(), display_name],
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn normalize_inputs(inputs: &[PlannerTeamInput]) -> Result<Vec<PlannerTeamSetting>, String> {
    if inputs.is_empty() {
        return Err("Planner configuration must include at least one team".to_string());
    }
    if inputs.len() > DEFAULT_TEAM_NAMES.len() {
        return Err(format!(
            "Planner configuration supports at most {} teams",
            DEFAULT_TEAM_NAMES.len()
        ));
    }

    let mut seen_teams = HashSet::new();
    let mut seen_names = HashSet::new();
    let mut normalized = Vec::with_capacity(inputs.len());
    for input in inputs {
        let team = PlannerTeam::parse(&input.team)?;
        if !seen_teams.insert(team.as_str()) {
            return Err(format!("Planner team `{}` must be unique", team.as_str()));
        }
        let display_name = input.display_name.trim();
        if display_name.is_empty() {
            return Err("Planner team display name must not be empty".to_string());
        }
        if display_name.chars().count() > MAX_DISPLAY_NAME_LEN {
            return Err(format!(
                "Planner team display name must be at most {MAX_DISPLAY_NAME_LEN} characters"
            ));
        }
        if !seen_names.insert(display_name.to_lowercase()) {
            return Err("Planner team display names must be unique".to_string());
        }
        normalized.push(PlannerTeamSetting {
            team,
            display_name: display_name.to_string(),
        });
    }
    normalized.sort_by_key(|setting| team_order(setting.team));
    Ok(normalized)
}

fn team_order(team: PlannerTeam) -> usize {
    match team {
        PlannerTeam::Senior => 0,
        PlannerTeam::Reserves => 1,
        PlannerTeam::Youth => 2,
    }
}

fn team_assignment_count(
    conn: &Transaction<'_>,
    save_id: i64,
    team: PlannerTeam,
) -> Result<i64, String> {
    conn.query_row(
        "SELECT COUNT(*)
         FROM planner_assignments assignment
         INNER JOIN planner_strings planner_string ON planner_string.id = assignment.string_id
         WHERE assignment.save_id = ?1 AND planner_string.team = ?2",
        params![save_id, team.as_str()],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

fn ensure_save_exists(conn: &Connection, save_id: i64) -> Result<(), String> {
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM saves WHERE id = ?1)",
            params![save_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !exists {
        return Err(format!("Save {save_id} not found"));
    }
    Ok(())
}
