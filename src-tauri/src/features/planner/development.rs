//! Development slots: the tactic slot each young player at the managed club is growing into.
//!
//! The Planner places a player where the squad needs him now, and his training follows the
//! slot. A development slot is where he should end up instead: the slot where his potential
//! fit score ranks highest against every player in the save who can play it, or the slot the
//! user chose for him. A save decides whether training follows it ([`TrainingBasis`]), whether
//! Optimize squads keeps young players in it ([`OptimizeFor`]), and up to what age a player
//! counts as developing.

use std::collections::HashMap;
use std::sync::Arc;

use rusqlite::{params, Connection};

use super::fit::PositionRule;
use super::optimizer::load_club_players;
use super::standout::{slot_rankings, SlotRankings};
use super::tactic::{base_position, get_tactic, PlannerTactic, TacticLane};
use crate::features::managed_club::service::managed_club_player_sql;

/// What a player's individual training follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrainingBasis {
    /// The slot the Planner has him in.
    Planner,
    /// His development slot while he is young enough, and his Planner slot after.
    Development,
}

impl TrainingBasis {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "planner" => Ok(Self::Planner),
            "development" => Ok(Self::Development),
            _ => Err(format!("Unknown training basis `{value}`")),
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Planner => "planner",
            Self::Development => "development",
        }
    }
}

/// What Optimize squads builds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OptimizeFor {
    /// The best squads today: every player goes where the squad is strongest.
    Strongest,
    /// A developing player takes only his development slot, or one that trains the same.
    Development,
}

impl OptimizeFor {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "strongest" => Ok(Self::Strongest),
            "development" => Ok(Self::Development),
            _ => Err(format!("Unknown Optimize goal `{value}`")),
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Strongest => "strongest",
            Self::Development => "development",
        }
    }
}

/// The youngest and oldest age limit a save can set for developing players. Potential scores
/// equal current scores from 29, so an older player has no slot to grow into.
pub(super) const MIN_DEVELOPMENT_MAX_AGE: i64 = 15;
pub(super) const MAX_DEVELOPMENT_MAX_AGE: i64 = 28;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DevelopmentSettings {
    pub(crate) training_basis: TrainingBasis,
    pub(crate) optimize_for: OptimizeFor,
    /// The oldest age at which a player still counts as developing.
    pub(crate) max_age: i64,
}

pub(super) fn load_settings(
    conn: &Connection,
    save_id: i64,
) -> Result<DevelopmentSettings, String> {
    let (training_basis, optimize_for, max_age) = conn
        .query_row(
            "SELECT training_basis, optimize_for, development_max_age FROM saves WHERE id = ?1",
            [save_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .map_err(|error| error.to_string())?;
    Ok(DevelopmentSettings {
        training_basis: TrainingBasis::parse(&training_basis)?,
        optimize_for: OptimizeFor::parse(&optimize_for)?,
        max_age,
    })
}

pub(super) fn set_training_basis(
    conn: &Connection,
    save_id: i64,
    basis: TrainingBasis,
) -> Result<(), String> {
    conn.execute(
        "UPDATE saves SET training_basis = ?2 WHERE id = ?1",
        params![save_id, basis.as_str()],
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

pub(super) fn set_optimize_for(
    conn: &Connection,
    save_id: i64,
    optimize_for: OptimizeFor,
) -> Result<(), String> {
    conn.execute(
        "UPDATE saves SET optimize_for = ?2 WHERE id = ?1",
        params![save_id, optimize_for.as_str()],
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

pub(super) fn set_max_age(conn: &Connection, save_id: i64, max_age: i64) -> Result<(), String> {
    if !(MIN_DEVELOPMENT_MAX_AGE..=MAX_DEVELOPMENT_MAX_AGE).contains(&max_age) {
        return Err(format!(
            "Players develop up to an age from {MIN_DEVELOPMENT_MAX_AGE} to {MAX_DEVELOPMENT_MAX_AGE}"
        ));
    }
    conn.execute(
        "UPDATE saves SET development_max_age = ?2 WHERE id = ?1",
        params![save_id, max_age],
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

/// Keeps `lane_id` as the development slot of a player at the managed club, in place of the
/// suggested one; `None` returns him to the suggestion.
pub(super) fn choose_slot(
    conn: &Connection,
    save_id: i64,
    snapshot_id: i64,
    player_uid: i64,
    lane_id: Option<&str>,
) -> Result<(), String> {
    let Some(lane_id) = lane_id else {
        return conn
            .execute(
                "DELETE FROM planner_development_slots WHERE save_id = ?1 AND player_uid = ?2",
                params![save_id, player_uid],
            )
            .map(|_| ())
            .map_err(|error| error.to_string());
    };
    let tactic = get_tactic(conn, save_id)?;
    if !tactic.lanes.iter().any(|lane| lane.lane_id == lane_id) {
        return Err(format!("Unknown tactic lane `{lane_id}`"));
    }
    let at_club: bool = conn
        .query_row(
            &format!(
                "SELECT EXISTS(
                     SELECT 1 FROM players p
                     WHERE p.snapshot_id = ?1 AND p.uid = ?3 AND {}
                 )",
                managed_club_player_sql("p", "?2")
            ),
            params![snapshot_id, save_id, player_uid],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !at_club {
        return Err("The player is not at the managed club".to_string());
    }
    conn.execute(
        "INSERT INTO planner_development_slots (save_id, player_uid, lane_id)
         VALUES (?1, ?2, ?3)
         ON CONFLICT (save_id, player_uid) DO UPDATE SET lane_id = excluded.lane_id",
        params![save_id, player_uid, lane_id],
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

/// Whether two lanes train the same: the same roles at the same positions, as a tactic's two
/// centre-back or central-midfield slots often do. A player developing towards one may play
/// the other, and the two count as one slot to grow into.
pub(super) fn same_training(left: &TacticLane, right: &TacticLane) -> bool {
    left.ip_role_id == right.ip_role_id
        && left.oop_role_id == right.oop_role_id
        && base_position(&left.ip_position) == base_position(&right.ip_position)
        && base_position(&left.oop_position) == base_position(&right.oop_position)
}

/// A slot a developing player could grow into.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct DevelopmentOption {
    pub(super) lane_index: usize,
    /// His fit score in the slot now and at his potential. A slot he cannot play yet is scored
    /// on his roles and foot alone.
    pub(super) current: u8,
    pub(super) potential: u8,
    /// The share of the slot's players in the save whom his potential outranks; `None` for a
    /// slot he cannot play yet.
    pub(super) potential_rank: Option<f64>,
}

/// Where a player aged up to the save's limit is growing.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct PlayerDevelopment {
    /// One option for each set of lanes that train the same: the slots he can play, highest
    /// potential rank first, then the others in the tactic's order.
    pub(super) options: Vec<DevelopmentOption>,
    /// The option the app suggests; `None` when he can play no slot.
    pub(super) suggested: Option<usize>,
    /// His development slot: the option he was given, otherwise the suggested one.
    pub(super) chosen: Option<usize>,
    /// Whether the user chose `chosen`.
    pub(super) locked: bool,
}

impl PlayerDevelopment {
    /// His scores now and at potential in his development slot.
    pub(super) fn chosen_scores(&self) -> Option<(u8, u8)> {
        let chosen = self.chosen?;
        self.options
            .iter()
            .find(|option| option.lane_index == chosen)
            .map(|option| (option.current, option.potential))
    }
}

/// The development slot of every managed-club player young enough to have one, by player UID.
#[derive(Debug, Clone, Default)]
pub(super) struct DevelopmentSlots {
    players: HashMap<i64, PlayerDevelopment>,
}

impl DevelopmentSlots {
    pub(super) fn of(&self, player_uid: i64) -> Option<&PlayerDevelopment> {
        self.players.get(&player_uid)
    }

    /// The lane the player is developing towards; `None` for a player who is too old, of
    /// unknown age, or suits no slot.
    pub(super) fn lane_of(&self, player_uid: i64) -> Option<usize> {
        self.of(player_uid)?.chosen
    }

    /// Each player with a development slot, and its lane.
    pub(super) fn lanes(&self) -> impl Iterator<Item = (i64, usize)> + '_ {
        self.players
            .iter()
            .filter_map(|(player_uid, development)| Some((*player_uid, development.chosen?)))
    }
}

/// Works out where each managed-club player aged up to `max_age` is growing.
///
/// A player is suggested the lane where his potential ranks highest among the lanes he can
/// play ([`PositionRule::Familiarity`], as Best role fit ranks him). He can be given any lane
/// his foot and goalkeeping allow, for a position he is to learn. Of lanes that train the same
/// he is offered the one the Planner has him in, otherwise the one he scores highest in.
///
/// `familiarity_rankings` gives the save's rankings under [`PositionRule::Familiarity`]; it is
/// asked only when a player is young enough.
pub(super) fn load_development_slots(
    conn: &Connection,
    save_id: i64,
    snapshot_id: i64,
    tactic: &PlannerTactic,
    max_age: i64,
    familiarity_rankings: impl FnOnce() -> Result<Arc<SlotRankings>, String>,
) -> Result<DevelopmentSlots, String> {
    let playable = load_club_players(
        conn,
        save_id,
        snapshot_id,
        tactic,
        PositionRule::Familiarity,
    )?;
    let learnable = load_club_players(
        conn,
        save_id,
        snapshot_id,
        tactic,
        PositionRule::AnyPosition,
    )?;
    if !playable
        .iter()
        .any(|player| player.development.age.is_some_and(|age| age <= max_age))
    {
        return Ok(DevelopmentSlots::default());
    }
    let rankings = familiarity_rankings()?;
    let lane_index = |lane_id: &str| tactic.lanes.iter().position(|lane| lane.lane_id == lane_id);
    let lane_by_player = |sql: &str| -> Result<HashMap<i64, usize>, String> {
        let mut statement = conn.prepare(sql).map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([save_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        // A lane the tactic no longer has is skipped.
        Ok(rows
            .into_iter()
            .filter_map(|(player_uid, lane_id)| Some((player_uid, lane_index(&lane_id)?)))
            .collect())
    };
    let chosen_lanes = lane_by_player(
        "SELECT player_uid, lane_id FROM planner_development_slots WHERE save_id = ?1",
    )?;
    let planner_lanes =
        lane_by_player("SELECT player_uid, lane_id FROM planner_assignments WHERE save_id = ?1")?;

    let mut players = HashMap::new();
    // Both reads list the club's players by UID.
    for (player, learner) in playable.iter().zip(&learnable) {
        if !player.development.age.is_some_and(|age| age <= max_age) {
            continue;
        }
        let planner_lane = planner_lanes.get(&player.uid).copied();
        // The lane that stands for `lane_index` and the lanes that train the same.
        let representative = |lane_index: usize, scores: &[Option<(u8, u8)>]| {
            let twins = (0..tactic.lanes.len())
                .filter(|&other| {
                    scores[other].is_some()
                        && same_training(&tactic.lanes[other], &tactic.lanes[lane_index])
                })
                .collect::<Vec<_>>();
            planner_lane
                .filter(|lane| twins.contains(lane))
                .or_else(|| {
                    twins.iter().copied().max_by_key(|&twin| {
                        let (current, potential) = scores[twin].expect("a scored twin");
                        (potential, current, std::cmp::Reverse(twin))
                    })
                })
                .unwrap_or(lane_index)
        };

        let mut options = Vec::<DevelopmentOption>::new();
        for (scores, can_play) in [(&player.lane_scores, true), (&learner.lane_scores, false)] {
            for lane_index in 0..tactic.lanes.len() {
                if scores[lane_index].is_none() {
                    continue;
                }
                let lane_index = representative(lane_index, scores);
                if options.iter().any(|option| {
                    same_training(&tactic.lanes[option.lane_index], &tactic.lanes[lane_index])
                }) {
                    continue;
                }
                let (current, potential) = scores[lane_index].expect("a scored lane");
                options.push(DevelopmentOption {
                    lane_index,
                    current,
                    potential,
                    potential_rank: can_play
                        .then(|| rankings.lane(lane_index).potential.rank(potential)),
                });
            }
        }
        // The lanes he can play come first, best potential rank first; the sort is stable, so
        // the others keep the tactic's order.
        options.sort_by(
            |left, right| match (left.potential_rank, right.potential_rank) {
                (Some(left_rank), Some(right_rank)) => right_rank
                    .total_cmp(&left_rank)
                    .then(right.potential.cmp(&left.potential)),
                (left_rank, right_rank) => right_rank.is_some().cmp(&left_rank.is_some()),
            },
        );

        let option_for = |lane_index: usize| {
            options
                .iter()
                .find(|option| {
                    same_training(&tactic.lanes[option.lane_index], &tactic.lanes[lane_index])
                })
                .map(|option| option.lane_index)
        };
        let suggested = rankings
            .development_lane(&player.lane_scores)
            .and_then(option_for);
        let given = chosen_lanes.get(&player.uid).copied().and_then(option_for);
        players.insert(
            player.uid,
            PlayerDevelopment {
                options,
                suggested,
                chosen: given.or(suggested),
                locked: given.is_some(),
            },
        );
    }
    Ok(DevelopmentSlots { players })
}

/// [`load_development_slots`] with the rankings a reader keeps for its data generation.
pub(super) fn development_slots(
    conn: &Connection,
    save_id: i64,
    snapshot_id: i64,
    tactic: &PlannerTactic,
    max_age: i64,
) -> Result<DevelopmentSlots, String> {
    load_development_slots(conn, save_id, snapshot_id, tactic, max_age, || {
        slot_rankings(conn, snapshot_id, tactic, PositionRule::Familiarity)
    })
}

/// The lane each player trains in place of his Planner slot: his development slot, for the
/// players who have one, when the save trains by development. Empty otherwise.
pub(super) fn training_lanes(
    conn: &Connection,
    save_id: i64,
    snapshot_id: i64,
    tactic: &PlannerTactic,
) -> Result<HashMap<i64, usize>, String> {
    let settings = load_settings(conn, save_id)?;
    if settings.training_basis != TrainingBasis::Development {
        return Ok(HashMap::new());
    }
    let slots = development_slots(conn, save_id, snapshot_id, tactic, settings.max_age)?;
    Ok(slots.lanes().collect())
}
