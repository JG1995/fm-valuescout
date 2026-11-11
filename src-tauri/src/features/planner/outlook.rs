//! What a tactic slot is worth to each Planner team: a player's score now, plus a share of
//! where his development is heading.
//!
//! Every weight here is a judgment call, not a number from Football Manager. Tune them here.

use rusqlite::Row;

use super::depth::PlannerTeam;

/// A missing personality attribute counts as this average value.
const AVERAGE_ATTRIBUTE: i64 = 10;

/// The age and personality that decide how much of his potential a player reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Development {
    pub(super) age: Option<i64>,
    pub(super) professionalism: Option<i64>,
    pub(super) ambition: Option<i64>,
    pub(super) determination: Option<i64>,
}

impl Development {
    /// The SQL columns [`Development::from_row`] reads, for `players` rows aliased `alias`.
    pub(super) fn columns(alias: &str) -> String {
        format!(
            "{alias}.age,
             json_extract({alias}.personality_json, '$.Professionalism'),
             json_extract({alias}.personality_json, '$.Ambition'),
             json_extract({alias}.attributes_json, '$.Determination')"
        )
    }

    /// Reads the four [`Development::columns`], the first of them at `first_column`.
    pub(super) fn from_row(row: &Row<'_>, first_column: usize) -> rusqlite::Result<Self> {
        Ok(Self {
            age: row.get(first_column)?,
            professionalism: row.get(first_column + 1)?,
            ambition: row.get(first_column + 2)?,
            determination: row.get(first_column + 3)?,
        })
    }

    /// How hard the player develops, on the 1–20 attribute scale.
    fn drive(&self) -> f64 {
        let value = |attribute: Option<i64>| attribute.unwrap_or(AVERAGE_ATTRIBUTE) as f64;
        0.60 * value(self.professionalism)
            + 0.25 * value(self.ambition)
            + 0.15 * value(self.determination)
    }

    /// The share of the gap between his current and potential scores the player is expected
    /// to close: the time his age leaves him, scaled by his drive.
    pub(super) fn reach(&self) -> f64 {
        (runway(self.age) * (0.40 + 0.04 * self.drive())).min(1.0)
    }
}

/// The share of the gap a player of this age still has time to close. Potential scores equal
/// current scores from 29, and an unknown age promises nothing.
fn runway(age: Option<i64>) -> f64 {
    match age {
        None => 0.0,
        Some(..=18) => 1.0,
        Some(19) => 0.95,
        Some(20) => 0.85,
        Some(21) => 0.75,
        Some(22) => 0.60,
        Some(23) => 0.45,
        Some(24) => 0.30,
        Some(25) => 0.20,
        Some(26) => 0.10,
        Some(27 | 28) => 0.05,
        Some(_) => 0.0,
    }
}

/// The points a player past his peak is expected to lose: two for each year past 30, ten at
/// most. Goalkeepers start three years later.
fn decline(age: Option<i64>, goalkeeper: bool) -> f64 {
    let last_peak_age = if goalkeeper { 33 } else { 30 };
    age.map_or(0.0, |age| (2 * (age - last_peak_age)).clamp(0, 10) as f64)
}

/// How much of a player's outlook the team's selection weighs against his score now.
fn focus(team: PlannerTeam) -> f64 {
    match team {
        PlannerTeam::Senior => 0.25,
        PlannerTeam::Reserves => 0.60,
        PlannerTeam::Youth => 0.85,
    }
}

/// The score the player is expected to reach in a slot.
pub(super) fn expected_score(current: u8, potential: u8, development: &Development) -> f64 {
    let current = f64::from(current);
    current + (f64::from(potential) - current).max(0.0) * development.reach()
}

/// What the player is worth to `team` in a slot, in tenths of a score point: his current score,
/// moved towards the score he is heading for by the team's focus.
pub(super) fn slot_value(
    team: PlannerTeam,
    current: u8,
    potential: u8,
    development: &Development,
    goalkeeper: bool,
) -> u16 {
    let outlook =
        expected_score(current, potential, development) - decline(development.age, goalkeeper);
    let current = f64::from(current);
    let value = current + focus(team) * (outlook - current);
    (value * 10.0).round().max(0.0) as u16
}
