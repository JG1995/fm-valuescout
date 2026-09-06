use std::collections::HashSet;

use rusqlite::{params, Connection, OptionalExtension, Transaction};

use crate::features::staff::assignment_targets;

use super::depth::{ordinal_label, preflight_depth_snapshot, PlannerTeam};

pub(super) const MAX_DISPLAY_NAME_LEN: usize = 40;

const REORDER_TEMP_BASE: i64 = 1_000_000;

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
pub(super) struct PlannerStringInput {
    pub(super) id: Option<i64>,
    pub(super) display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PlannerTeamInput {
    pub(super) team: String,
    pub(super) display_name: String,
    pub(super) strings: Vec<PlannerStringInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedStringSetting {
    id: Option<i64>,
    display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedTeamSetting {
    team: PlannerTeam,
    display_name: String,
    strings: Vec<NormalizedStringSetting>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PlannerStringRemovalImpact {
    pub(super) string_id: i64,
    pub(super) display_name: String,
    pub(super) assignment_count: i64,
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
    pub(super) strings: Vec<PlannerStringRemovalImpact>,
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
    let current_strings = load_string_rows(conn, save_id)?;
    let mut impacts = Vec::new();
    for setting in &current {
        match desired.iter().find(|item| item.team == setting.team) {
            None => {
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
                let strings = string_removal_impacts(
                    conn,
                    &current_strings
                        .iter()
                        .filter(|row| row.team == setting.team)
                        .map(|row| row.id)
                        .collect::<Vec<_>>(),
                    &current_strings,
                )?;
                impacts.push(PlannerTeamRemovalImpact {
                    team: setting.team,
                    display_name: setting.display_name.clone(),
                    assignment_count,
                    staffing_targets,
                    strings,
                });
            }
            Some(item) => {
                let desired_ids = item
                    .strings
                    .iter()
                    .filter_map(|string| string.id)
                    .collect::<HashSet<_>>();
                let removed_ids = current_strings
                    .iter()
                    .filter(|row| row.team == setting.team && !desired_ids.contains(&row.id))
                    .map(|row| row.id)
                    .collect::<Vec<_>>();
                if !removed_ids.is_empty() {
                    let strings = string_removal_impacts(conn, &removed_ids, &current_strings)?;
                    let assignment_count =
                        strings.iter().map(|impact| impact.assignment_count).sum();
                    impacts.push(PlannerTeamRemovalImpact {
                        team: setting.team,
                        display_name: setting.display_name.clone(),
                        assignment_count,
                        staffing_targets: Vec::new(),
                        strings,
                    });
                }
            }
        }
    }
    Ok(impacts)
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
    let current_strings = load_string_rows_from_tx(&tx, save_id)?;
    validate_string_ids(&tx, save_id, &desired, &current_strings)?;
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
    let removed_string_impacts =
        removed_string_impacts_in_tx(&tx, &desired, &current, &current_strings)?;
    let populated_teams = impacts
        .iter()
        .filter(|(_, assignment_count, staffing_targets)| {
            *assignment_count > 0 || !staffing_targets.is_empty()
        })
        .collect::<Vec<_>>();
    let populated_strings = removed_string_impacts
        .iter()
        .filter(|(_, _, assignment_count)| *assignment_count > 0)
        .collect::<Vec<_>>();
    if (!populated_teams.is_empty() || !populated_strings.is_empty()) && !confirm_populated_removal
    {
        let mut confirmed = populated_teams
            .iter()
            .map(|(setting, assignment_count, _)| {
                format_assignment_count(&setting.display_name, *assignment_count)
            })
            .collect::<Vec<_>>();
        confirmed.extend(
            populated_strings
                .iter()
                .map(|(_, display_name, assignment_count)| {
                    format_assignment_count(display_name, *assignment_count)
                }),
        );
        return Err(format!(
            "Removing populated planner squads or strings requires confirmation: {}",
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

    for item in &desired {
        if current.iter().any(|setting| setting.team == item.team) {
            apply_retained_team_strings(&tx, save_id, item, &current_strings)?;
        }
    }

    tx.execute("DELETE FROM planner_teams WHERE save_id = ?1", [save_id])
        .map_err(|error| error.to_string())?;

    for item in &desired {
        let was_available = current.iter().any(|setting| setting.team == item.team);
        tx.execute(
            "INSERT INTO planner_teams (save_id, team, display_name)
             VALUES (?1, ?2, ?3)",
            params![save_id, item.team.as_str(), item.display_name],
        )
        .map_err(|error| error.to_string())?;
        if !was_available {
            for (string_order, string) in item.strings.iter().enumerate() {
                tx.execute(
                    "INSERT INTO planner_strings (save_id, team, string_order, display_name)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![
                        save_id,
                        item.team.as_str(),
                        string_order as i64,
                        string.display_name
                    ],
                )
                .map_err(|error| error.to_string())?;
            }
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
        normalized.push(NormalizedTeamSetting {
            team,
            display_name: display_name.to_string(),
            strings: normalize_string_inputs(team, &input.strings)?,
        });
    }
    normalized.sort_by_key(|setting| team_order(setting.team));
    Ok(normalized)
}

fn normalize_string_inputs(
    team: PlannerTeam,
    inputs: &[PlannerStringInput],
) -> Result<Vec<NormalizedStringSetting>, String> {
    if inputs.is_empty() {
        return Err(format!(
            "The {} team must keep at least one string",
            team.as_str()
        ));
    }
    let mut seen_names = HashSet::new();
    let mut normalized = Vec::with_capacity(inputs.len());
    for (order, input) in inputs.iter().enumerate() {
        let trimmed = input.display_name.trim();
        let display_name = if input.id.is_none() && trimmed.is_empty() {
            ordinal_label(order as i64)
        } else {
            if trimmed.is_empty() {
                return Err("Planner string display name must not be empty".to_string());
            }
            trimmed.to_string()
        };
        if display_name.chars().count() > MAX_DISPLAY_NAME_LEN {
            return Err(format!(
                "Planner string display name must be at most {MAX_DISPLAY_NAME_LEN} characters"
            ));
        }
        if !seen_names.insert(display_name.to_lowercase()) {
            return Err("Planner string display names must be unique".to_string());
        }
        normalized.push(NormalizedStringSetting {
            id: input.id,
            display_name,
        });
    }
    Ok(normalized)
}

struct StringRow {
    id: i64,
    team: PlannerTeam,
    display_name: String,
}

fn load_string_rows(conn: &Connection, save_id: i64) -> Result<Vec<StringRow>, String> {
    let mut statement = conn
        .prepare(
            "SELECT id, team, display_name
             FROM planner_strings
             WHERE save_id = ?1",
        )
        .map_err(|error| error.to_string())?;
    let mapped = statement
        .query_map(params![save_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|error| error.to_string())?;
    mapped
        .map(|row| {
            let (id, team, display_name) = row.map_err(|error| error.to_string())?;
            Ok(StringRow {
                id,
                team: PlannerTeam::parse(&team)?,
                display_name,
            })
        })
        .collect::<Result<Vec<_>, String>>()
}

fn load_string_rows_from_tx(tx: &Transaction<'_>, save_id: i64) -> Result<Vec<StringRow>, String> {
    let mut statement = tx
        .prepare(
            "SELECT id, team, display_name
             FROM planner_strings
             WHERE save_id = ?1",
        )
        .map_err(|error| error.to_string())?;
    let mapped = statement
        .query_map(params![save_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|error| error.to_string())?;
    mapped
        .map(|row| {
            let (id, team, display_name) = row.map_err(|error| error.to_string())?;
            Ok(StringRow {
                id,
                team: PlannerTeam::parse(&team)?,
                display_name,
            })
        })
        .collect::<Result<Vec<_>, String>>()
}

fn validate_string_ids(
    tx: &Transaction<'_>,
    save_id: i64,
    desired: &[NormalizedTeamSetting],
    current_strings: &[StringRow],
) -> Result<(), String> {
    let mut seen_ids = HashSet::new();
    for item in desired {
        for string in &item.strings {
            let Some(id) = string.id else {
                continue;
            };
            if !seen_ids.insert(id) {
                return Err(format!("Planner string {id} must be unique"));
            }
            if let Some(row) = current_strings.iter().find(|row| row.id == id) {
                if row.team != item.team {
                    return Err(format!(
                        "Planner string {id} does not belong to the `{}` team",
                        item.team.as_str()
                    ));
                }
                continue;
            }
            let stored: Option<(i64, String)> = tx
                .query_row(
                    "SELECT save_id, team FROM planner_strings WHERE id = ?1",
                    params![id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
                .map_err(|error| error.to_string())?;
            match stored {
                None => return Err(format!("Planner string {id} not found")),
                Some((owner_save_id, _)) if owner_save_id != save_id => {
                    return Err(format!("Planner string {id} does not belong to this save"));
                }
                Some(_) => {
                    return Err(format!(
                        "Planner string {id} does not belong to the `{}` team",
                        item.team.as_str()
                    ));
                }
            }
        }
    }
    Ok(())
}

fn string_assignment_count(tx: &Transaction<'_>, string_id: i64) -> Result<i64, String> {
    tx.query_row(
        "SELECT COUNT(*) FROM planner_assignments WHERE string_id = ?1",
        params![string_id],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

fn removed_string_impacts_in_tx(
    tx: &Transaction<'_>,
    desired: &[NormalizedTeamSetting],
    current: &[PlannerTeamSetting],
    current_strings: &[StringRow],
) -> Result<Vec<(i64, String, i64)>, String> {
    let mut impacts = Vec::new();
    for setting in current {
        let Some(item) = desired.iter().find(|item| item.team == setting.team) else {
            continue;
        };
        let desired_ids = item
            .strings
            .iter()
            .filter_map(|string| string.id)
            .collect::<HashSet<_>>();
        for row in current_strings
            .iter()
            .filter(|row| row.team == setting.team && !desired_ids.contains(&row.id))
        {
            let assignment_count = string_assignment_count(tx, row.id)?;
            impacts.push((row.id, row.display_name.clone(), assignment_count));
        }
    }
    Ok(impacts)
}

fn string_removal_impacts(
    conn: &Connection,
    removed_ids: &[i64],
    current_strings: &[StringRow],
) -> Result<Vec<PlannerStringRemovalImpact>, String> {
    let mut impacts = Vec::new();
    for id in removed_ids {
        let row = current_strings
            .iter()
            .find(|row| row.id == *id)
            .ok_or_else(|| format!("Planner string {id} not found"))?;
        let assignment_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM planner_assignments WHERE string_id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        impacts.push(PlannerStringRemovalImpact {
            string_id: *id,
            display_name: row.display_name.clone(),
            assignment_count,
        });
    }
    Ok(impacts)
}

fn apply_retained_team_strings(
    tx: &Transaction<'_>,
    save_id: i64,
    item: &NormalizedTeamSetting,
    current_strings: &[StringRow],
) -> Result<(), String> {
    let desired_ids = item
        .strings
        .iter()
        .filter_map(|string| string.id)
        .collect::<HashSet<_>>();
    for row in current_strings
        .iter()
        .filter(|row| row.team == item.team && !desired_ids.contains(&row.id))
    {
        tx.execute(
            "DELETE FROM planner_assignments WHERE save_id = ?1 AND string_id = ?2",
            params![save_id, row.id],
        )
        .map_err(|error| error.to_string())?;
        tx.execute("DELETE FROM planner_strings WHERE id = ?1", params![row.id])
            .map_err(|error| error.to_string())?;
    }

    let retained_ids = item
        .strings
        .iter()
        .filter_map(|string| string.id)
        .collect::<Vec<_>>();
    for (index, id) in retained_ids.iter().enumerate() {
        tx.execute(
            "UPDATE planner_strings SET string_order = ?1 WHERE id = ?2",
            params![REORDER_TEMP_BASE + index as i64, id],
        )
        .map_err(|error| error.to_string())?;
    }
    for (string_order, string) in item.strings.iter().enumerate() {
        if let Some(id) = string.id {
            tx.execute(
                "UPDATE planner_strings
                 SET string_order = ?1, display_name = ?2
                 WHERE id = ?3",
                params![string_order as i64, string.display_name, id],
            )
            .map_err(|error| error.to_string())?;
        }
    }
    for (string_order, string) in item.strings.iter().enumerate() {
        if string.id.is_none() {
            tx.execute(
                "INSERT INTO planner_strings (save_id, team, string_order, display_name)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    save_id,
                    item.team.as_str(),
                    string_order as i64,
                    string.display_name
                ],
            )
            .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
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
