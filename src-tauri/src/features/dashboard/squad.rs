//! Squad upkeep for the managed club: contracts that run out soon and young players with
//! room to grow into the tactic.

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashSet};

use rusqlite::{params, Connection};

use crate::features::planner::fit::tactic_adjusted_score;
use crate::features::planner::optimizer::{allocate_lanes, OptimizerCandidate};
use crate::features::planner::tactic::{read_tactic, TacticLane};
use crate::features::player_metrics::compact::{
    assert_read_models_complete, player_current_column, player_metrics_join,
    player_potential_column, PLAYER_METRICS_ALIAS,
};

use super::{club_snapshot_id, MANAGED_CLUB_NAME_SQL};

pub(crate) const CONTRACT_WINDOW_MONTHS: u32 = 12;
pub(crate) const MAX_EXPIRING_CONTRACTS: usize = 25;
pub(crate) const PROSPECT_MAX_AGE: i64 = 21;
pub(crate) const MAX_PROSPECTS: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpiringContract {
    pub(crate) uid: i64,
    pub(crate) name: String,
    pub(crate) age: Option<i64>,
    pub(crate) ca: i64,
    pub(crate) market_value_gbp: Option<i64>,
    pub(crate) weekly_wage_gbp: Option<i64>,
    /// The contract's last day as `YYYY-MM-DD`.
    pub(crate) expiry_date: String,
    /// Days from the snapshot's in-game date to the expiry, when that date is known.
    pub(crate) days_remaining: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpiringContracts {
    /// The window after the in-game date that the list covers. Without a usable in-game
    /// date there is no window and the list holds the soonest expiries.
    pub(crate) window_months: Option<u32>,
    pub(crate) total: usize,
    pub(crate) players: Vec<ExpiringContract>,
}

/// The managed club's own players by contract expiry, soonest first. Players on loan at the
/// club are left out, because their contracts belong to their parent clubs.
pub(crate) fn get_expiring_contracts(
    conn: &Connection,
    save_id: i64,
) -> Result<Option<ExpiringContracts>, String> {
    let Some(snapshot_id) = club_snapshot_id(conn, save_id)? else {
        return Ok(None);
    };
    let game_date: Option<String> = conn
        .query_row(
            "SELECT CASE WHEN date(game_date) IS game_date THEN game_date END
             FROM snapshots
             WHERE id = ?1",
            [snapshot_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    let mut players = conn
        .prepare(&format!(
            "WITH contracts AS (
                 SELECT p.uid, p.name, p.age, p.ca, p.market_value_gbp, p.weekly_wage_gbp,
                        date(
                            printf('%04d-01-01', p.contract_expiry_year),
                            printf('+%d days', p.contract_expiry_day_of_year - 1)
                        ) AS expiry_date
                 FROM players p
                 WHERE p.snapshot_id = ?1
                   AND p.current_club = {MANAGED_CLUB_NAME_SQL}
                   AND p.on_loan IS NOT 1
                   AND p.contract_expiry_year IS NOT NULL
                   AND p.contract_expiry_day_of_year IS NOT NULL
             )
             SELECT uid, name, age, ca, market_value_gbp, weekly_wage_gbp, expiry_date,
                    CAST(julianday(expiry_date) - julianday(?3) AS INTEGER)
             FROM contracts
             WHERE expiry_date IS NOT NULL
               AND (?3 IS NULL OR expiry_date <= date(?3, '+{CONTRACT_WINDOW_MONTHS} months'))
             ORDER BY expiry_date, ca DESC, uid"
        ))
        .map_err(|error| error.to_string())?
        .query_map(params![snapshot_id, save_id, game_date], |row| {
            Ok(ExpiringContract {
                uid: row.get(0)?,
                name: row.get(1)?,
                age: row.get(2)?,
                ca: row.get(3)?,
                market_value_gbp: row.get(4)?,
                weekly_wage_gbp: row.get(5)?,
                expiry_date: row.get(6)?,
                days_remaining: row.get(7)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let total = players.len();
    players.truncate(MAX_EXPIRING_CONTRACTS);
    Ok(Some(ExpiringContracts {
        window_months: game_date.map(|_| CONTRACT_WINDOW_MONTHS),
        total,
        players,
    }))
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Prospect {
    pub(crate) uid: i64,
    pub(crate) name: String,
    pub(crate) age: i64,
    /// The tactic slot where the player's potential score is highest.
    pub(crate) lane: TacticLane,
    pub(crate) current_score: u8,
    pub(crate) potential_score: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct YouthXiPlayer {
    pub(crate) uid: i64,
    pub(crate) name: String,
    pub(crate) age: i64,
    pub(crate) current_score: u8,
    pub(crate) potential_score: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct YouthXiSlot {
    pub(crate) lane: TacticLane,
    /// Absent when no young player can play the slot.
    pub(crate) player: Option<YouthXiPlayer>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Prospects {
    /// Young players with room to grow in their best slot, highest potential first.
    pub(crate) players: Vec<Prospect>,
    /// Every tactic slot with the young player it takes when the slots are allocated by
    /// potential score, as the Planner's Optimize allocates them.
    pub(crate) youth_xi: Vec<YouthXiSlot>,
}

struct YoungPlayer {
    uid: i64,
    name: String,
    age: i64,
    /// The current and potential score in each tactic slot, in lane order.
    lane_scores: Vec<Option<(u8, u8)>>,
}

/// The managed club's players aged `PROSPECT_MAX_AGE` or younger: those whose potential score
/// in their best tactic slot exceeds their current one, and the XI they form by potential.
pub(crate) fn get_prospects(conn: &Connection, save_id: i64) -> Result<Option<Prospects>, String> {
    let Some(snapshot_id) = club_snapshot_id(conn, save_id)? else {
        return Ok(None);
    };
    assert_read_models_complete(conn, snapshot_id, true, true)?;
    let tactic = read_tactic(conn, save_id)?;
    let mut score_columns = String::new();
    for lane in &tactic.lanes {
        for role_id in [&lane.ip_role_id, &lane.oop_role_id] {
            score_columns.push_str(&format!(
                ", {PLAYER_METRICS_ALIAS}.{}, {PLAYER_METRICS_ALIAS}.{}",
                player_current_column(role_id)?,
                player_potential_column(role_id)?
            ));
        }
    }
    let sql = format!(
        "SELECT p.uid, p.name, p.age, p.preferred_foot, p.positions_json{score_columns}
         FROM players p{}
         WHERE p.snapshot_id = ?1
           AND p.current_club = {MANAGED_CLUB_NAME_SQL}
           AND p.age <= ?3
         ORDER BY p.uid",
        player_metrics_join("p", true, true)
    );
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let mut rows = statement
        .query(params![snapshot_id, save_id, PROSPECT_MAX_AGE])
        .map_err(|error| error.to_string())?;
    let mut young = Vec::new();
    while let Some(row) = rows.next().map_err(|error| error.to_string())? {
        let read = |error: rusqlite::Error| error.to_string();
        let foot: String = row.get(3).map_err(read)?;
        let positions = serde_json::from_str::<BTreeMap<String, Option<i64>>>(
            &row.get::<_, String>(4).map_err(read)?,
        )
        .map_err(|error| error.to_string())?;
        let mut lane_scores = Vec::with_capacity(tactic.lanes.len());
        for (lane_index, lane) in tactic.lanes.iter().enumerate() {
            // Per lane: IP current, IP potential, OOP current, OOP potential.
            let column = 5 + lane_index * 4;
            let score = |ip: usize, oop: usize| -> Result<Option<u8>, String> {
                Ok(tactic_adjusted_score(
                    row.get(ip).map_err(read)?,
                    row.get(oop).map_err(read)?,
                    lane.ip_weight,
                    &foot,
                    &positions,
                    lane,
                ))
            };
            lane_scores.push(score(column, column + 2)?.zip(score(column + 1, column + 3)?));
        }
        young.push(YoungPlayer {
            uid: row.get(0).map_err(read)?,
            name: row.get(1).map_err(read)?,
            age: row.get(2).map_err(read)?,
            lane_scores,
        });
    }

    let mut players = young
        .iter()
        .filter_map(|player| {
            // Between equal fits the earlier lane wins.
            let (lane_index, (current_score, potential_score)) = player
                .lane_scores
                .iter()
                .enumerate()
                .filter_map(|(lane_index, scores)| Some((lane_index, (*scores)?)))
                .max_by_key(|&(lane_index, (current, potential))| {
                    (potential, current, Reverse(lane_index))
                })?;
            (potential_score > current_score).then(|| Prospect {
                uid: player.uid,
                name: player.name.clone(),
                age: player.age,
                lane: tactic.lanes[lane_index].clone(),
                current_score,
                potential_score,
            })
        })
        .collect::<Vec<_>>();
    // Between equal potentials, the lower current score has more room to grow.
    players.sort_by(|left, right| {
        right
            .potential_score
            .cmp(&left.potential_score)
            .then_with(|| left.current_score.cmp(&right.current_score))
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
            .then_with(|| left.uid.cmp(&right.uid))
    });
    players.truncate(MAX_PROSPECTS);

    let candidates = young
        .iter()
        .map(|player| OptimizerCandidate {
            player_uid: player.uid,
            last_known_name: player.name.clone(),
            lane_scores: player
                .lane_scores
                .iter()
                .map(|scores| scores.map(|(_, potential)| u16::from(potential)))
                .collect(),
        })
        .collect::<Vec<_>>();
    let mut youth_xi = tactic
        .lanes
        .iter()
        .map(|lane| YouthXiSlot {
            lane: lane.clone(),
            player: None,
        })
        .collect::<Vec<_>>();
    let every_lane = (0..tactic.lanes.len()).collect::<Vec<_>>();
    for (lane_index, player_index) in
        allocate_lanes(&tactic, &every_lane, &candidates, &HashSet::new())
    {
        let player = &young[player_index];
        youth_xi[lane_index].player =
            player.lane_scores[lane_index].map(|(current_score, potential_score)| YouthXiPlayer {
                uid: player.uid,
                name: player.name.clone(),
                age: player.age,
                current_score,
                potential_score,
            });
    }

    Ok(Some(Prospects { players, youth_xi }))
}
