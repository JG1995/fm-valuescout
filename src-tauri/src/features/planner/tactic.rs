use std::collections::HashSet;

use rusqlite::{params, Connection};

use crate::features::scoring::catalog::{all_roles, RolePhase};

pub const TACTIC_LANE_COUNT: usize = 11;
pub const DEFAULT_LANE_IDS: [&str; TACTIC_LANE_COUNT] = [
    "goalkeeper",
    "left_back",
    "left_centre_back",
    "right_centre_back",
    "right_back",
    "defensive_midfielder",
    "left_central_midfielder",
    "right_central_midfielder",
    "left_winger",
    "right_winger",
    "centre_forward",
];

#[derive(Debug, Clone, PartialEq)]
pub struct TacticLane {
    pub lane_id: String,
    pub ip_weight: f64,
    pub importance_rank: Option<u8>,
    pub preferred_foot: String,
    pub foot_preference: String,
    pub ip_position: String,
    pub ip_role_id: String,
    pub oop_position: String,
    pub oop_role_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlannerTactic {
    pub lanes: Vec<TacticLane>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TacticRoleOption {
    pub role_id: String,
    pub display_name: String,
    pub phase: String,
    pub position_tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TacticOptions {
    pub placements: Vec<String>,
    pub roles: Vec<TacticRoleOption>,
}

struct DefaultLane {
    lane_id: &'static str,
    ip_position: &'static str,
    ip_role_id: &'static str,
    oop_position: &'static str,
    oop_role_id: &'static str,
}

const DEFAULT_LANES: [DefaultLane; TACTIC_LANE_COUNT] = [
    DefaultLane {
        lane_id: "goalkeeper",
        ip_position: "GK",
        ip_role_id: "goalkeeper_ip",
        oop_position: "GK",
        oop_role_id: "line_holding_keeper_oop",
    },
    DefaultLane {
        lane_id: "left_back",
        ip_position: "DL",
        ip_role_id: "full_back_ip",
        oop_position: "DL",
        oop_role_id: "holding_full_back_oop",
    },
    DefaultLane {
        lane_id: "left_centre_back",
        ip_position: "DC",
        ip_role_id: "centre_back_ip",
        oop_position: "DC",
        oop_role_id: "covering_centre_back_oop",
    },
    DefaultLane {
        lane_id: "right_centre_back",
        ip_position: "DC",
        ip_role_id: "centre_back_ip",
        oop_position: "DC",
        oop_role_id: "covering_centre_back_oop",
    },
    DefaultLane {
        lane_id: "right_back",
        ip_position: "DR",
        ip_role_id: "full_back_ip",
        oop_position: "DR",
        oop_role_id: "holding_full_back_oop",
    },
    DefaultLane {
        lane_id: "defensive_midfielder",
        ip_position: "DM",
        ip_role_id: "defensive_midfielder_ip",
        oop_position: "DM",
        oop_role_id: "screening_defensive_midfielder_oop",
    },
    DefaultLane {
        lane_id: "left_central_midfielder",
        ip_position: "MC",
        ip_role_id: "central_midfielder_ip",
        oop_position: "MC",
        oop_role_id: "pressing_central_midfielder_oop",
    },
    DefaultLane {
        lane_id: "right_central_midfielder",
        ip_position: "MC",
        ip_role_id: "central_midfielder_ip",
        oop_position: "MC",
        oop_role_id: "pressing_central_midfielder_oop",
    },
    DefaultLane {
        lane_id: "left_winger",
        ip_position: "AML",
        ip_role_id: "winger_ip",
        oop_position: "ML",
        oop_role_id: "tracking_wide_midfielder_oop",
    },
    DefaultLane {
        lane_id: "right_winger",
        ip_position: "AMR",
        ip_role_id: "winger_ip",
        oop_position: "MR",
        oop_role_id: "tracking_wide_midfielder_oop",
    },
    DefaultLane {
        lane_id: "centre_forward",
        ip_position: "ST",
        ip_role_id: "centre_forward_ip",
        oop_position: "ST",
        oop_role_id: "central_outlet_centre_forward_oop",
    },
];

pub fn default_tactic() -> PlannerTactic {
    PlannerTactic {
        lanes: DEFAULT_LANES
            .iter()
            .map(|lane| TacticLane {
                lane_id: lane.lane_id.to_string(),
                ip_weight: 0.5,
                importance_rank: None,
                preferred_foot: "any".to_string(),
                foot_preference: "preferred".to_string(),
                ip_position: lane.ip_position.to_string(),
                ip_role_id: lane.ip_role_id.to_string(),
                oop_position: lane.oop_position.to_string(),
                oop_role_id: lane.oop_role_id.to_string(),
            })
            .collect(),
    }
}

pub fn get_tactic(conn: &Connection, save_id: i64) -> Result<PlannerTactic, String> {
    ensure_save_exists(conn, save_id)?;

    let tactic_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM planner_tactic_lanes WHERE save_id = ?1)",
            params![save_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !tactic_exists {
        let tactic = default_tactic();
        save_tactic(conn, save_id, &tactic)?;
    }

    load_tactic(conn, save_id)
}

pub fn save_tactic(conn: &Connection, save_id: i64, tactic: &PlannerTactic) -> Result<(), String> {
    ensure_save_exists(conn, save_id)?;
    validate_tactic(tactic)?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    tx.execute(
        "DELETE FROM planner_tactic_lanes WHERE save_id = ?1",
        params![save_id],
    )
    .map_err(|error| error.to_string())?;

    for (lane_order, lane) in tactic.lanes.iter().enumerate() {
        tx.execute(
            "INSERT INTO planner_tactic_lanes (
                 save_id, lane_order, lane_id, ip_weight, importance_rank, preferred_foot, foot_preference, ip_position, ip_role_id, oop_position, oop_role_id
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                save_id,
                lane_order as i64,
                lane.lane_id,
                lane.ip_weight,
                lane.importance_rank,
                lane.preferred_foot,
                if lane.preferred_foot == "any" {
                    "preferred"
                } else {
                    &lane.foot_preference
                },
                lane.ip_position,
                lane.ip_role_id,
                lane.oop_position,
                lane.oop_role_id,
            ],
        )
        .map_err(|error| error.to_string())?;
    }

    tx.commit().map_err(|error| error.to_string())
}

pub fn get_tactic_options() -> TacticOptions {
    let mut placements = Vec::new();
    let mut seen_placements = HashSet::new();
    let roles = all_roles()
        .iter()
        .map(|role| {
            for position_tag in role.position_tags {
                if seen_placements.insert(*position_tag) {
                    placements.push((*position_tag).to_string());
                }
            }

            TacticRoleOption {
                role_id: role.role_id.to_string(),
                display_name: role.display_name.to_string(),
                phase: role.phase.as_db_str().to_string(),
                position_tags: role
                    .position_tags
                    .iter()
                    .map(|tag| (*tag).to_string())
                    .collect(),
            }
        })
        .collect();

    TacticOptions { placements, roles }
}

fn load_tactic(conn: &Connection, save_id: i64) -> Result<PlannerTactic, String> {
    let mut statement = conn
        .prepare(
            "SELECT lane_id, ip_weight, importance_rank, preferred_foot, foot_preference, ip_position, ip_role_id, oop_position, oop_role_id
             FROM planner_tactic_lanes
             WHERE save_id = ?1
             ORDER BY lane_order",
        )
        .map_err(|error| error.to_string())?;
    let lanes = statement
        .query_map(params![save_id], |row| {
            Ok(TacticLane {
                lane_id: row.get(0)?,
                ip_weight: row.get(1)?,
                importance_rank: row.get(2)?,
                preferred_foot: row.get(3)?,
                foot_preference: row.get(4)?,
                ip_position: row.get(5)?,
                ip_role_id: row.get(6)?,
                oop_position: row.get(7)?,
                oop_role_id: row.get(8)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    let tactic = PlannerTactic { lanes };
    validate_tactic(&tactic)?;
    Ok(tactic)
}

fn ensure_save_exists(conn: &Connection, save_id: i64) -> Result<(), String> {
    let save_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM saves WHERE id = ?1)",
            params![save_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !save_exists {
        return Err(format!("Save {save_id} not found"));
    }
    Ok(())
}

fn validate_tactic(tactic: &PlannerTactic) -> Result<(), String> {
    if tactic.lanes.len() != TACTIC_LANE_COUNT {
        return Err(format!(
            "Tactic must contain exactly {TACTIC_LANE_COUNT} lanes"
        ));
    }

    let mut importance_ranks = HashSet::new();
    for (index, lane) in tactic.lanes.iter().enumerate() {
        let expected_lane_id = DEFAULT_LANE_IDS[index];
        if lane.lane_id != expected_lane_id {
            return Err(format!(
                "Lane {index} must be `{expected_lane_id}`, got `{}`",
                lane.lane_id
            ));
        }
        if !lane.ip_weight.is_finite() || !(0.0..=1.0).contains(&lane.ip_weight) {
            return Err(format!(
                "Lane `{}` IP weight must be between 0 and 1",
                lane.lane_id
            ));
        }
        if let Some(rank) = lane.importance_rank {
            if !(1..=TACTIC_LANE_COUNT as u8).contains(&rank) {
                return Err(format!(
                    "Lane `{}` importance rank must be between 1 and {TACTIC_LANE_COUNT}",
                    lane.lane_id
                ));
            }
            if !importance_ranks.insert(rank) {
                return Err("Importance ranks must be unique".to_string());
            }
        }
        if !matches!(
            lane.preferred_foot.as_str(),
            "any" | "left" | "right" | "both"
        ) {
            return Err(format!(
                "Lane `{}` preferred foot must be Either, Left, Right, or Both",
                lane.lane_id
            ));
        }
        if !matches!(lane.foot_preference.as_str(), "preferred" | "strict") {
            return Err(format!(
                "Lane `{}` foot preference must be Preferred or Strict",
                lane.lane_id
            ));
        }
        validate_role(
            &lane.lane_id,
            "in-possession",
            &lane.ip_position,
            &lane.ip_role_id,
            RolePhase::InPossession,
        )?;
        validate_role(
            &lane.lane_id,
            "out-of-possession",
            &lane.oop_position,
            &lane.oop_role_id,
            RolePhase::OutOfPossession,
        )?;
    }

    Ok(())
}

fn validate_role(
    lane_id: &str,
    phase_label: &str,
    position: &str,
    role_id: &str,
    expected_phase: RolePhase,
) -> Result<(), String> {
    let role = all_roles()
        .iter()
        .find(|role| role.role_id == role_id)
        .ok_or_else(|| format!("Lane `{lane_id}` references unknown role `{role_id}`"))?;
    if role.phase != expected_phase {
        return Err(format!(
            "Lane `{lane_id}` uses role `{role_id}` in the wrong {phase_label} phase"
        ));
    }
    if !role.position_tags.contains(&position) {
        return Err(format!(
            "Role `{role_id}` does not support position {position}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::db::migrations;

    use super::*;

    fn open_with_save() -> (tempfile::TempDir, Connection, i64) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("planner-tactic.db")).expect("open db");
        migrations::apply(&conn).expect("apply migrations");
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Test save', 1)",
            [],
        )
        .expect("create save");
        let save_id = conn.last_insert_rowid();
        (temp_dir, conn, save_id)
    }

    #[test]
    fn default_tactic_has_eleven_linked_lanes_and_expected_shapes() {
        let (_temp_dir, conn, save_id) = open_with_save();

        let tactic = get_tactic(&conn, save_id).expect("load default tactic");

        assert_eq!(tactic.lanes.len(), 11);
        assert_eq!(
            tactic
                .lanes
                .iter()
                .map(|lane| lane.lane_id.as_str())
                .collect::<Vec<_>>(),
            DEFAULT_LANE_IDS
        );
        assert_eq!(tactic.lanes[0].ip_position, "GK");
        assert_eq!(tactic.lanes[0].ip_weight, 0.5);
        assert_eq!(tactic.lanes[0].oop_position, "GK");
        assert_eq!(tactic.lanes[5].ip_position, "DM");
        assert_eq!(tactic.lanes[5].oop_position, "DM");
        assert_eq!(tactic.lanes[8].ip_position, "AML");
        assert_eq!(tactic.lanes[8].oop_position, "ML");
    }

    #[test]
    fn tactic_is_save_scoped_and_reloads_after_update() {
        let (_temp_dir, conn, first_save_id) = open_with_save();
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Second save', 0)",
            [],
        )
        .expect("create second save");
        let second_save_id = conn.last_insert_rowid();

        let mut first = get_tactic(&conn, first_save_id).expect("load first tactic");
        let second = get_tactic(&conn, second_save_id).expect("load second tactic");
        first.lanes[0].ip_weight = 0.7;
        first.lanes[0].ip_role_id = "ball_playing_goalkeeper_ip".to_string();

        save_tactic(&conn, first_save_id, &first).expect("save first tactic");

        let reloaded = get_tactic(&conn, first_save_id).expect("reload first tactic");
        assert_eq!(reloaded.lanes[0].ip_weight, 0.7);
        assert_eq!(reloaded.lanes[1].ip_weight, 0.5);
        assert_eq!(reloaded.lanes[0].ip_role_id, "ball_playing_goalkeeper_ip");
        assert_eq!(
            get_tactic(&conn, second_save_id).expect("reload second tactic"),
            second
        );
    }

    #[test]
    fn persists_gapped_importance_ranks_and_rejects_invalid_values() {
        let (_temp_dir, conn, save_id) = open_with_save();
        let mut tactic = get_tactic(&conn, save_id).expect("load default tactic");
        tactic.lanes[0].importance_rank = Some(3);
        tactic.lanes[1].importance_rank = Some(11);

        save_tactic(&conn, save_id, &tactic).expect("save gapped ranks");

        let reloaded = get_tactic(&conn, save_id).expect("reload tactic");
        assert_eq!(reloaded.lanes[0].importance_rank, Some(3));
        assert_eq!(reloaded.lanes[1].importance_rank, Some(11));

        tactic.lanes[2].importance_rank = Some(3);
        let duplicate_error =
            save_tactic(&conn, save_id, &tactic).expect_err("reject duplicate rank");
        assert!(duplicate_error.contains("Importance ranks must be unique"));

        tactic.lanes[2].importance_rank = Some(12);
        let range_error =
            save_tactic(&conn, save_id, &tactic).expect_err("reject out-of-range rank");
        assert!(range_error.contains("between 1 and 11"));
    }

    #[test]
    fn persists_lane_foot_preferences_and_rejects_unknown_values() {
        let (_temp_dir, conn, save_id) = open_with_save();
        let mut tactic = get_tactic(&conn, save_id).expect("load default tactic");
        tactic.lanes[0].preferred_foot = "both".to_string();
        tactic.lanes[0].foot_preference = "strict".to_string();

        save_tactic(&conn, save_id, &tactic).expect("save foot preference");

        let reloaded = get_tactic(&conn, save_id).expect("reload tactic");
        assert_eq!(reloaded.lanes[0].preferred_foot, "both");
        assert_eq!(reloaded.lanes[0].foot_preference, "strict");

        tactic.lanes[0].preferred_foot = "unknown".to_string();
        let error = save_tactic(&conn, save_id, &tactic).expect_err("reject unknown foot");
        assert!(error.contains("preferred foot"));
    }

    #[test]
    fn rejects_incomplete_tactic() {
        let (_temp_dir, conn, save_id) = open_with_save();
        let mut tactic = get_tactic(&conn, save_id).expect("load default tactic");
        tactic.lanes.pop();

        let error = save_tactic(&conn, save_id, &tactic).expect_err("reject incomplete tactic");
        assert!(error.contains("exactly 11 lanes"));
    }

    #[test]
    fn rejects_phase_incompatible_roles() {
        let (_temp_dir, conn, save_id) = open_with_save();
        let mut tactic = get_tactic(&conn, save_id).expect("load default tactic");
        tactic.lanes[0].oop_role_id = "goalkeeper_ip".to_string();

        let error =
            save_tactic(&conn, save_id, &tactic).expect_err("reject an IP role in OOP phase");
        assert!(error.contains("out-of-possession"));
    }

    #[test]
    fn rejects_unknown_roles() {
        let (_temp_dir, conn, save_id) = open_with_save();
        let mut tactic = get_tactic(&conn, save_id).expect("load default tactic");
        tactic.lanes[0].ip_role_id = "not_a_catalog_role".to_string();

        let error = save_tactic(&conn, save_id, &tactic).expect_err("reject an unknown role");
        assert!(error.contains("unknown role"));
    }

    #[test]
    fn rejects_invalid_role_position_pairs() {
        let (_temp_dir, conn, save_id) = open_with_save();
        let mut tactic = get_tactic(&conn, save_id).expect("load default tactic");
        tactic.lanes[0].ip_position = "ST".to_string();

        let error = save_tactic(&conn, save_id, &tactic)
            .expect_err("reject a goalkeeper role at the striker placement");
        assert!(error.contains("does not support position ST"));
    }

    #[test]
    fn rejects_ip_weights_outside_zero_to_one() {
        let (_temp_dir, conn, save_id) = open_with_save();
        let mut tactic = get_tactic(&conn, save_id).expect("load default tactic");
        tactic.lanes[0].ip_weight = 1.1;

        let error = save_tactic(&conn, save_id, &tactic).expect_err("reject an invalid weight");
        assert!(error.contains("IP weight must be between 0 and 1"));
    }

    #[test]
    fn rejects_ip_weights_below_zero() {
        let (_temp_dir, conn, save_id) = open_with_save();
        let mut tactic = get_tactic(&conn, save_id).expect("load default tactic");
        tactic.lanes[0].ip_weight = -0.1;

        let error = save_tactic(&conn, save_id, &tactic).expect_err("reject a negative weight");
        assert!(error.contains("IP weight must be between 0 and 1"));
    }

    #[test]
    fn exposes_phase_compatible_roles_and_placements_from_catalog() {
        let options = get_tactic_options();

        assert!(options.placements.contains(&"GK".to_string()));
        assert!(options.placements.contains(&"AML".to_string()));
        let goalkeeper = options
            .roles
            .iter()
            .find(|role| role.role_id == "goalkeeper_ip")
            .expect("goalkeeper role option");
        assert_eq!(goalkeeper.phase, "in_possession");
        assert_eq!(goalkeeper.position_tags, vec!["GK"]);
    }
}
