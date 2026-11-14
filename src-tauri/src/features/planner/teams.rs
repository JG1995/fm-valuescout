use std::collections::HashSet;

use rusqlite::{params, Connection, Transaction};

use crate::features::staff::assignment_targets;

use super::depth::{insert_default_strings, preflight_depth_snapshot, PlannerTeam};

pub(super) const MAX_DISPLAY_NAME_LEN: usize = 40;

/// The lowest and highest age limit a team may set.
const MIN_MAX_AGE: i64 = 14;
const MAX_MAX_AGE: i64 = 45;

const DEFAULT_TEAM_NAMES: [(PlannerTeam, &str); 3] = [
    (PlannerTeam::Senior, "Senior"),
    (PlannerTeam::Reserves, "Reserves"),
    (PlannerTeam::Youth, "Youth"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlannerTeamSetting {
    pub(super) team: PlannerTeam,
    pub(super) display_name: String,
    /// The oldest age the team's squads take, or `None` for no limit.
    pub(super) max_age: Option<i64>,
}

impl PlannerTeamSetting {
    /// Whether a player of this age may play for the team. An unknown age fits only a team
    /// without a limit.
    pub(super) fn takes_age(&self, age: Option<i64>) -> bool {
        match self.max_age {
            None => true,
            Some(max_age) => age.is_some_and(|age| age <= max_age),
        }
    }
}

/// The age limit a team starts with: the usual under-23 and under-18 squads.
pub(super) fn default_max_age(team: PlannerTeam) -> Option<i64> {
    match team {
        PlannerTeam::Senior => None,
        PlannerTeam::Reserves => Some(23),
        PlannerTeam::Youth => Some(18),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PlannerTeamInput {
    pub(super) team: String,
    pub(super) display_name: String,
    pub(super) max_age: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedTeamSetting {
    team: PlannerTeam,
    display_name: String,
    max_age: Option<i64>,
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
    if !team_settings_exist(conn, save_id)? {
        let tx = conn
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;
        ensure_team_rows(&tx, save_id)?;
        tx.commit().map_err(|error| error.to_string())?;
    }
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

/// What removing the teams missing from `inputs` would delete: their assignments and their
/// staffing targets.
pub(super) fn planner_team_removal_impacts(
    conn: &Connection,
    save_id: i64,
    inputs: &[PlannerTeamInput],
) -> Result<Vec<PlannerTeamRemovalImpact>, String> {
    let desired = normalize_inputs(inputs)?;
    ensure_team_settings(conn, save_id)?;
    load_team_settings(conn, save_id)?
        .into_iter()
        .filter(|setting| !desired.iter().any(|item| item.team == setting.team))
        .map(|setting| {
            Ok(PlannerTeamRemovalImpact {
                team: setting.team,
                assignment_count: team_assignment_count(conn, save_id, setting.team)?,
                staffing_targets: assignment_targets::nonzero_targets_for_scope(
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
                .collect(),
                display_name: setting.display_name,
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
    let populated_teams = impacts
        .iter()
        .filter(|(_, assignment_count, staffing_targets)| {
            *assignment_count > 0 || !staffing_targets.is_empty()
        })
        .collect::<Vec<_>>();
    if !populated_teams.is_empty() && !confirm_populated_removal {
        let confirmed = populated_teams
            .iter()
            .map(|(setting, assignment_count, _)| {
                format_assignment_count(&setting.display_name, *assignment_count)
            })
            .collect::<Vec<_>>();
        return Err(format!(
            "Removing populated planner squads requires confirmation: {}",
            confirmed.join(", ")
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

    for item in &desired {
        tx.execute(
            "INSERT INTO planner_teams (save_id, team, display_name, max_age)
             VALUES (?1, ?2, ?3, ?4)",
            params![save_id, item.team.as_str(), item.display_name, item.max_age],
        )
        .map_err(|error| error.to_string())?;
        // A kept team has its strings already; a restored one starts with the defaults.
        insert_default_strings(&tx, save_id, item.team)?;
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
            "SELECT team, display_name, max_age
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
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<i64>>(2)?,
            ))
        })
        .map_err(|error| error.to_string())?
        .map(|row| {
            let (team, display_name, max_age) = row.map_err(|error| error.to_string())?;
            Ok(PlannerTeamSetting {
                team: PlannerTeam::parse(&team)?,
                display_name,
                max_age,
            })
        })
        .collect::<Result<Vec<_>, String>>();
    settings
}

/// Whether the save has team rows. Without any, `ensure_team_settings` creates the defaults;
/// removed teams are never recreated while another remains.
pub(crate) fn team_settings_exist(conn: &Connection, save_id: i64) -> Result<bool, String> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM planner_teams WHERE save_id = ?1)",
        params![save_id],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

fn ensure_team_rows(tx: &Transaction<'_>, save_id: i64) -> Result<(), String> {
    if team_settings_exist(tx, save_id)? {
        return Ok(());
    }
    for (team, display_name) in DEFAULT_TEAM_NAMES {
        tx.execute(
            "INSERT INTO planner_teams (save_id, team, display_name, max_age)
             VALUES (?1, ?2, ?3, ?4)",
            params![save_id, team.as_str(), display_name, default_max_age(team)],
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn normalize_inputs(inputs: &[PlannerTeamInput]) -> Result<Vec<NormalizedTeamSetting>, String> {
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
        if team == PlannerTeam::Senior && input.max_age.is_some() {
            return Err("The senior team has no age limit".to_string());
        }
        if input
            .max_age
            .is_some_and(|max_age| !(MIN_MAX_AGE..=MAX_MAX_AGE).contains(&max_age))
        {
            return Err(format!(
                "Planner team age limit must be between {MIN_MAX_AGE} and {MAX_MAX_AGE}"
            ));
        }
        normalized.push(NormalizedTeamSetting {
            team,
            display_name: display_name.to_string(),
            max_age: input.max_age,
        });
    }
    normalized.sort_by_key(|setting| team_order(setting.team));
    Ok(normalized)
}

fn format_assignment_count(display_name: &str, assignment_count: i64) -> String {
    format!(
        "{display_name} ({assignment_count} assignment{})",
        if assignment_count == 1 { "" } else { "s" }
    )
}

fn team_order(team: PlannerTeam) -> usize {
    match team {
        PlannerTeam::Senior => 0,
        PlannerTeam::Reserves => 1,
        PlannerTeam::Youth => 2,
    }
}

fn team_assignment_count(
    conn: &Connection,
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
