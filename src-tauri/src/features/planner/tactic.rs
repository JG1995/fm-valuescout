use std::collections::HashSet;

use rusqlite::{params, Connection, Transaction};

use crate::features::moneyball::{is_moneyball_statistic_key, is_save_unavailable_statistic_key};
use crate::features::scoring::catalog::{all_roles, RolePhase};
use crate::features::search::filter::{
    compile_slot_filters, filter_value_from_json, FilterRule, MAX_SLOT_FILTER_RULES,
};

pub const TACTIC_LANE_COUNT: usize = 11;
const MAX_SLOT_METRICS: usize = 5;
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
    /// Ordered Moneyball statistic keys for the slot's custom score.
    pub custom_metric_keys: Vec<String>,
    /// Rules a signing must pass to be recommended for the slot. They never apply to the
    /// club's own players.
    pub recruitment_filters: Vec<RecruitmentFilter>,
}

/// One recruitment filter rule: a Search filter field, its operator, and the value in Search's
/// IPC form. `search::filter::compile_slot_filters` owns which rules are valid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecruitmentFilter {
    pub field: String,
    pub op: String,
    pub value: serde_json::Value,
}

impl RecruitmentFilter {
    pub(crate) fn to_filter_rule(&self) -> Result<FilterRule, String> {
        Ok(FilterRule {
            field: self.field.clone(),
            op: self.op.clone(),
            value: filter_value_from_json(self.value.clone())?,
        })
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({ "field": self.field, "op": self.op, "value": self.value })
    }

    fn from_json(value: serde_json::Value) -> Option<Self> {
        let serde_json::Value::Object(mut object) = value else {
            return None;
        };
        Some(Self {
            field: object.remove("field")?.as_str()?.to_string(),
            op: object.remove("op")?.as_str()?.to_string(),
            value: object.remove("value")?,
        })
    }
}

/// The lane's recruitment filters as Search filter rules.
pub(crate) fn recruitment_filter_rules(lane: &TacticLane) -> Result<Vec<FilterRule>, String> {
    lane.recruitment_filters
        .iter()
        .map(RecruitmentFilter::to_filter_rule)
        .collect()
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
        ip_position: "DCR",
        ip_role_id: "centre_back_ip",
        oop_position: "DCR",
        oop_role_id: "covering_centre_back_oop",
    },
    DefaultLane {
        lane_id: "right_centre_back",
        ip_position: "DCL",
        ip_role_id: "centre_back_ip",
        oop_position: "DCL",
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
        ip_position: "MCR",
        ip_role_id: "central_midfielder_ip",
        oop_position: "MCR",
        oop_role_id: "pressing_central_midfielder_oop",
    },
    DefaultLane {
        lane_id: "right_central_midfielder",
        ip_position: "MCL",
        ip_role_id: "central_midfielder_ip",
        oop_position: "MCL",
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
        ip_position: "STC",
        ip_role_id: "centre_forward_ip",
        oop_position: "STC",
        oop_role_id: "central_outlet_centre_forward_oop",
    },
];

fn sided_placements(base_position: &str) -> Option<[&'static str; 3]> {
    match base_position {
        "DC" => Some(["DCR", "DC", "DCL"]),
        "DM" => Some(["DMCR", "DM", "DMCL"]),
        "MC" => Some(["MCR", "MC", "MCL"]),
        "AMC" => Some(["AMCR", "AMC", "AMCL"]),
        "ST" => Some(["STCR", "STC", "STCL"]),
        _ => None,
    }
}

pub(crate) fn base_position(placement: &str) -> &str {
    match placement {
        "DCR" | "DC" | "DCL" => "DC",
        "DMCR" | "DM" | "DMCL" => "DM",
        "MCR" | "MC" | "MCL" => "MC",
        "AMCR" | "AMC" | "AMCL" => "AMC",
        "STCR" | "ST" | "STC" | "STCL" => "ST",
        _ => placement,
    }
}

fn canonical_placement(placement: &str) -> &str {
    if placement == "ST" {
        "STC"
    } else {
        placement
    }
}

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
                custom_metric_keys: Vec::new(),
                recruitment_filters: Vec::new(),
            })
            .collect(),
    }
}

pub fn get_tactic(conn: &Connection, save_id: i64) -> Result<PlannerTactic, String> {
    ensure_save_exists(conn, save_id)?;

    if !tactic_exists(conn, save_id)? {
        let tactic = default_tactic();
        save_tactic(conn, save_id, &tactic)?;
    }

    load_tactic(conn, save_id)
}

/// Returns `get_tactic`'s values without writing: a save without stored lanes gets
/// `default_tactic()` in memory, so a read-only connection can load it.
pub fn read_tactic(conn: &Connection, save_id: i64) -> Result<PlannerTactic, String> {
    ensure_save_exists(conn, save_id)?;

    if tactic_exists(conn, save_id)? {
        load_tactic(conn, save_id)
    } else {
        Ok(default_tactic())
    }
}

/// Whether the save has stored lanes. Without them, `get_tactic` and
/// `load_or_initialize_tactic_in_tx` store the default tactic.
pub(crate) fn tactic_exists(conn: &Connection, save_id: i64) -> Result<bool, String> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM planner_tactic_lanes WHERE save_id = ?1)",
        params![save_id],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

pub fn save_tactic(conn: &Connection, save_id: i64, tactic: &PlannerTactic) -> Result<(), String> {
    ensure_save_exists(conn, save_id)?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    save_tactic_in_tx(&tx, save_id, tactic)?;
    tx.commit().map_err(|error| error.to_string())
}

pub(crate) fn load_or_initialize_tactic_in_tx(
    tx: &Transaction<'_>,
    save_id: i64,
) -> Result<PlannerTactic, String> {
    if !tactic_exists(tx, save_id)? {
        let tactic = default_tactic();
        save_tactic_in_tx(tx, save_id, &tactic)?;
    }
    load_tactic(tx, save_id)
}

pub(crate) fn save_tactic_in_tx(
    tx: &Transaction<'_>,
    save_id: i64,
    tactic: &PlannerTactic,
) -> Result<(), String> {
    validate_tactic(tactic)?;
    tx.execute(
        "DELETE FROM planner_tactic_lanes WHERE save_id = ?1",
        params![save_id],
    )
    .map_err(|error| error.to_string())?;
    for (lane_order, lane) in tactic.lanes.iter().enumerate() {
        let custom_metric_keys_json =
            serde_json::to_string(&lane.custom_metric_keys).map_err(|error| error.to_string())?;
        let recruitment_filters_json = serde_json::Value::Array(
            lane.recruitment_filters
                .iter()
                .map(RecruitmentFilter::to_json)
                .collect(),
        )
        .to_string();
        tx.execute(
            "INSERT INTO planner_tactic_lanes (
                 save_id, lane_order, lane_id, ip_weight, importance_rank, preferred_foot, foot_preference, ip_position, ip_role_id, oop_position, oop_role_id, custom_metric_keys_json, recruitment_filters_json
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
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
                custom_metric_keys_json,
                recruitment_filters_json,
            ],
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn get_tactic_options() -> TacticOptions {
    let mut placements = Vec::new();
    let mut seen_base_positions = HashSet::new();
    let roles = all_roles()
        .iter()
        .map(|role| {
            for position_tag in role.position_tags {
                if seen_base_positions.insert(*position_tag) {
                    if let Some(sided) = sided_placements(position_tag) {
                        placements.extend(sided.map(str::to_string));
                    } else {
                        placements.push((*position_tag).to_string());
                    }
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

pub(super) fn load_tactic(conn: &Connection, save_id: i64) -> Result<PlannerTactic, String> {
    let mut statement = conn
        .prepare(
            "SELECT lane_id, ip_weight, importance_rank, preferred_foot, foot_preference, ip_position, ip_role_id, oop_position, oop_role_id, custom_metric_keys_json, recruitment_filters_json
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
                custom_metric_keys: parse_custom_metric_keys(&row.get::<_, String>(9)?).map_err(
                    |error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            9,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    },
                )?,
                recruitment_filters: parse_recruitment_filters(&row.get::<_, String>(10)?)
                    .map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            10,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    let tactic = PlannerTactic { lanes };
    validate_tactic_contents(&tactic, false)?;
    Ok(tactic)
}

fn is_offered_slot_metric(key: &str) -> bool {
    is_moneyball_statistic_key(key) && !is_save_unavailable_statistic_key(key)
}

/// Keeps only the first occurrence of each stored key the catalog still offers. Moneyball
/// Search interpolates these keys into SQL text, so this filter must stay on the load path.
fn parse_custom_metric_keys(json: &str) -> Result<Vec<String>, serde_json::Error> {
    let stored: Vec<serde_json::Value> = serde_json::from_str(json)?;
    let mut seen = HashSet::new();
    Ok(stored
        .into_iter()
        .filter_map(|value| value.as_str().map(str::to_string))
        .filter(|key| is_offered_slot_metric(key) && seen.insert(key.clone()))
        .collect())
}

/// Keeps the stored rules that still compile, such as after a catalog change. The dashboard
/// interpolates their statistic keys into SQL, so this filter must stay on the load path.
fn parse_recruitment_filters(json: &str) -> Result<Vec<RecruitmentFilter>, serde_json::Error> {
    let stored: Vec<serde_json::Value> = serde_json::from_str(json)?;
    Ok(stored
        .into_iter()
        .filter_map(RecruitmentFilter::from_json)
        .filter(|filter| {
            filter
                .to_filter_rule()
                .and_then(|rule| compile_slot_filters(&[rule], 1))
                .is_ok()
        })
        .take(MAX_SLOT_FILTER_RULES)
        .collect())
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
    validate_tactic_contents(tactic, true)
}

fn validate_tactic_contents(
    tactic: &PlannerTactic,
    require_unique_placements: bool,
) -> Result<(), String> {
    if tactic.lanes.len() != TACTIC_LANE_COUNT {
        return Err(format!(
            "Tactic must contain exactly {TACTIC_LANE_COUNT} lanes"
        ));
    }

    let mut importance_ranks = HashSet::new();
    let mut ip_placements = HashSet::new();
    let mut oop_placements = HashSet::new();
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
        let ip_placement = canonical_placement(&lane.ip_position);
        if require_unique_placements && !ip_placements.insert(ip_placement) {
            return Err(format!(
                "{} is already used in the in-possession phase",
                ip_placement
            ));
        }
        let oop_placement = canonical_placement(&lane.oop_position);
        if require_unique_placements && !oop_placements.insert(oop_placement) {
            return Err(format!(
                "{} is already used in the out-of-possession phase",
                oop_placement
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
        validate_custom_metric_keys(lane)?;
        recruitment_filter_rules(lane)
            .and_then(|rules| compile_slot_filters(&rules, 1))
            .map_err(|error| format!("Lane `{}` recruitment filters: {error}", lane.lane_id))?;
    }

    Ok(())
}

fn validate_custom_metric_keys(lane: &TacticLane) -> Result<(), String> {
    let lane_id = &lane.lane_id;
    if lane.custom_metric_keys.len() > MAX_SLOT_METRICS {
        return Err(format!(
            "Lane `{lane_id}` can have at most {MAX_SLOT_METRICS} Moneyball metrics"
        ));
    }
    let mut seen = HashSet::new();
    for key in &lane.custom_metric_keys {
        if !is_moneyball_statistic_key(key) {
            return Err(format!(
                "Lane `{lane_id}` references unknown Moneyball metric `{key}`"
            ));
        }
        if is_save_unavailable_statistic_key(key) {
            return Err(format!(
                "Lane `{lane_id}` uses Moneyball metric `{key}`, which save files do not provide"
            ));
        }
        if !seen.insert(key.as_str()) {
            return Err(format!("Lane `{lane_id}` repeats Moneyball metric `{key}`"));
        }
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
    if !role.position_tags.contains(&base_position(position)) {
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
        assert_eq!(tactic.lanes[2].ip_position, "DCR");
        assert_eq!(tactic.lanes[3].ip_position, "DCL");
        assert_eq!(tactic.lanes[5].ip_position, "DM");
        assert_eq!(tactic.lanes[5].oop_position, "DM");
        assert_eq!(tactic.lanes[6].ip_position, "MCR");
        assert_eq!(tactic.lanes[7].ip_position, "MCL");
        assert_eq!(tactic.lanes[8].ip_position, "AML");
        assert_eq!(tactic.lanes[8].oop_position, "ML");
        assert_eq!(tactic.lanes[10].ip_position, "STC");
    }

    #[test]
    fn read_tactic_returns_get_tactic_values_without_storing_lanes() {
        let (_temp_dir, conn, save_id) = open_with_save();
        let stored_lanes = |conn: &Connection| -> i64 {
            conn.query_row("SELECT COUNT(*) FROM planner_tactic_lanes", [], |row| {
                row.get(0)
            })
            .expect("count lanes")
        };

        let read_default = read_tactic(&conn, save_id).expect("read default tactic");
        assert_eq!(stored_lanes(&conn), 0);
        assert_eq!(
            read_default,
            get_tactic(&conn, save_id).expect("get tactic")
        );

        let mut edited = read_default;
        edited.lanes[0].ip_weight = 0.7;
        save_tactic(&conn, save_id, &edited).expect("save tactic");
        assert_eq!(
            read_tactic(&conn, save_id).expect("read stored tactic"),
            get_tactic(&conn, save_id).expect("get stored tactic")
        );
        assert_eq!(
            read_tactic(&conn, save_id + 1).unwrap_err(),
            get_tactic(&conn, save_id + 1).unwrap_err()
        );
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
    fn accepts_sided_placements_for_roles_that_support_the_base_position() {
        let (_temp_dir, conn, save_id) = open_with_save();
        let mut tactic = get_tactic(&conn, save_id).expect("load default tactic");
        tactic.lanes[6].ip_position = "MCR".to_string();
        tactic.lanes[7].ip_position = "MCL".to_string();

        save_tactic(&conn, save_id, &tactic).expect("save sided midfield placements");

        let reloaded = get_tactic(&conn, save_id).expect("reload tactic");
        assert_eq!(reloaded.lanes[6].ip_position, "MCR");
        assert_eq!(reloaded.lanes[7].ip_position, "MCL");
    }

    #[test]
    fn rejects_duplicate_sided_placements_in_one_phase() {
        let (_temp_dir, conn, save_id) = open_with_save();
        let mut tactic = get_tactic(&conn, save_id).expect("load default tactic");
        tactic.lanes[6].ip_position = "MCR".to_string();
        tactic.lanes[7].ip_position = "MCR".to_string();

        let error = save_tactic(&conn, save_id, &tactic)
            .expect_err("reject two lanes in the same sided placement");

        assert!(error.contains("MCR is already used in the in-possession phase"));
    }

    #[test]
    fn rejects_legacy_striker_alias_in_the_canonical_centre_placement() {
        let (_temp_dir, conn, save_id) = open_with_save();
        let mut tactic = get_tactic(&conn, save_id).expect("load default tactic");
        tactic.lanes[0].ip_position = "ST".to_string();
        tactic.lanes[0].ip_role_id = "centre_forward_ip".to_string();

        let error = save_tactic(&conn, save_id, &tactic)
            .expect_err("reject legacy and canonical centre striker aliases");

        assert!(error.contains("STC is already used in the in-possession phase"));
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

    fn keys(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn persists_ordered_slot_metrics_per_lane() {
        let (_temp_dir, conn, save_id) = open_with_save();
        let mut tactic = get_tactic(&conn, save_id).expect("load default tactic");
        tactic.lanes[0].custom_metric_keys = keys(&["save_ratio", "goals", "clean_sheets_ratio"]);

        save_tactic(&conn, save_id, &tactic).expect("save slot metrics");

        let reloaded = get_tactic(&conn, save_id).expect("reload tactic");
        assert_eq!(
            reloaded.lanes[0].custom_metric_keys,
            ["save_ratio", "goals", "clean_sheets_ratio"]
        );
        assert!(reloaded.lanes[1..]
            .iter()
            .all(|lane| lane.custom_metric_keys.is_empty()));
    }

    #[test]
    fn rejects_invalid_slot_metrics_without_changing_the_stored_tactic() {
        let (_temp_dir, conn, save_id) = open_with_save();
        let mut stored = get_tactic(&conn, save_id).expect("load default tactic");
        stored.lanes[1].custom_metric_keys = keys(&["assists"]);
        save_tactic(&conn, save_id, &stored).expect("save valid slot metrics");

        for (metrics, expected_error) in [
            (
                keys(&["goals", "assists", "xg", "xa", "shots", "save_ratio"]),
                "Lane `goalkeeper` can have at most 5 Moneyball metrics",
            ),
            (
                keys(&["goals", "assists", "goals"]),
                "Lane `goalkeeper` repeats Moneyball metric `goals`",
            ),
            (
                keys(&["not_a_metric"]),
                "Lane `goalkeeper` references unknown Moneyball metric `not_a_metric`",
            ),
            (
                keys(&["goals_from_outside_the_box"]),
                "Lane `goalkeeper` uses Moneyball metric `goals_from_outside_the_box`, which save files do not provide",
            ),
        ] {
            let mut attempted = stored.clone();
            attempted.lanes[0].custom_metric_keys = metrics;
            attempted.lanes[1].custom_metric_keys = keys(&["goals"]);

            let error = save_tactic(&conn, save_id, &attempted).expect_err("reject slot metrics");

            assert_eq!(error, expected_error);
            assert_eq!(get_tactic(&conn, save_id).expect("reload tactic"), stored);
        }
    }

    #[test]
    fn loading_drops_stored_slot_metrics_the_catalog_no_longer_offers() {
        let (_temp_dir, conn, save_id) = open_with_save();
        get_tactic(&conn, save_id).expect("initialize tactic");
        conn.execute(
            "UPDATE planner_tactic_lanes
             SET custom_metric_keys_json = '[\"save_ratio\",\"retired_metric\",\"goals_from_outside_the_box\",\"clean_sheets_ratio\",\"save_ratio\"]'
             WHERE save_id = ?1 AND lane_id = 'goalkeeper'",
            params![save_id],
        )
        .expect("store drifted slot metrics");

        let tactic = get_tactic(&conn, save_id).expect("load drifted tactic");

        assert_eq!(
            tactic.lanes[0].custom_metric_keys,
            ["save_ratio", "clean_sheets_ratio"]
        );
    }

    fn filter(field: &str, op: &str, value: serde_json::Value) -> RecruitmentFilter {
        RecruitmentFilter {
            field: field.to_string(),
            op: op.to_string(),
            value,
        }
    }

    #[test]
    fn persists_recruitment_filters_per_lane_and_rejects_invalid_ones() {
        let (_temp_dir, conn, save_id) = open_with_save();
        let mut stored = get_tactic(&conn, save_id).expect("load default tactic");
        stored.lanes[3].recruitment_filters = vec![
            filter("preferred_foot", "in", serde_json::json!(["right"])),
            filter("height", "gte", serde_json::json!(183)),
            filter(
                "moneyball.progressive_passes",
                "pct_gte",
                serde_json::json!(25),
            ),
        ];
        save_tactic(&conn, save_id, &stored).expect("save recruitment filters");

        let reloaded = get_tactic(&conn, save_id).expect("reload tactic");
        assert_eq!(reloaded, stored);

        let mut attempted = stored.clone();
        attempted.lanes[0].recruitment_filters =
            vec![filter("club_dna", "gte", serde_json::json!(50))];
        let error = save_tactic(&conn, save_id, &attempted).expect_err("reject Club DNA");
        assert_eq!(
            error,
            "Lane `goalkeeper` recruitment filters: club_dna cannot filter a tactic slot"
        );
        attempted.lanes[0].recruitment_filters =
            vec![filter("height", "gte", serde_json::json!(170)); MAX_SLOT_FILTER_RULES + 1];
        save_tactic(&conn, save_id, &attempted).expect_err("reject too many filters");
        assert_eq!(get_tactic(&conn, save_id).expect("reload tactic"), stored);
    }

    #[test]
    fn loading_drops_stored_recruitment_filters_that_no_longer_compile() {
        let (_temp_dir, conn, save_id) = open_with_save();
        get_tactic(&conn, save_id).expect("initialize tactic");
        conn.execute(
            r#"UPDATE planner_tactic_lanes
               SET recruitment_filters_json = '[
                   {"field":"moneyball.retired_metric","op":"pct_gte","value":50},
                   {"field":"height","op":"gte","value":183},
                   {"field":"height"},
                   "not an object"
               ]'
               WHERE save_id = ?1 AND lane_id = 'goalkeeper'"#,
            params![save_id],
        )
        .expect("store drifted filters");

        let tactic = get_tactic(&conn, save_id).expect("load drifted tactic");

        assert_eq!(
            tactic.lanes[0].recruitment_filters,
            [filter("height", "gte", serde_json::json!(183))]
        );
    }

    #[test]
    fn exposes_phase_compatible_roles_and_placements_from_catalog() {
        let options = get_tactic_options();

        assert!(options.placements.contains(&"GK".to_string()));
        assert!(options.placements.contains(&"AML".to_string()));
        assert!(options.placements.contains(&"WBR".to_string()));
        assert!(options.placements.contains(&"WBL".to_string()));
        assert!(options.placements.contains(&"MCR".to_string()));
        assert!(options.placements.contains(&"MCL".to_string()));
        assert!(options.placements.contains(&"STC".to_string()));
        let goalkeeper = options
            .roles
            .iter()
            .find(|role| role.role_id == "goalkeeper_ip")
            .expect("goalkeeper role option");
        assert_eq!(goalkeeper.phase, "in_possession");
        assert_eq!(goalkeeper.position_tags, vec!["GK"]);
        let generic_keeper = options
            .roles
            .iter()
            .find(|role| role.role_id == "goalkeeper_oop")
            .expect("generic OOP goalkeeper role option");
        assert_eq!(generic_keeper.phase, "out_of_possession");
        assert_eq!(generic_keeper.position_tags, vec!["GK"]);
        let channel_midfielder = options
            .roles
            .iter()
            .find(|role| role.role_id == "channel_midfielder_ip")
            .expect("channel midfielder role option");
        assert_eq!(channel_midfielder.phase, "in_possession");
        assert_eq!(channel_midfielder.position_tags, vec!["AMC", "MC"]);
    }
}
