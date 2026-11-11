//! Read models for the dashboard: what the managed club's current snapshot says about its
//! starting XI, transfer targets, contracts, and prospects.

use rusqlite::{Connection, OptionalExtension};

pub mod commands;
pub(crate) mod signings;
pub(crate) mod squad;

#[cfg(test)]
mod signings_tests;
#[cfg(test)]
mod squad_tests;
#[cfg(test)]
mod test_support;

/// The save's managed club name as a scalar subquery. The save ID is parameter `?2`.
const MANAGED_CLUB_NAME_SQL: &str =
    "(SELECT club_name FROM managed_club_settings WHERE save_id = ?2)";

/// The save's current snapshot, or `None` without one or without a configured managed club.
fn club_snapshot_id(conn: &Connection, save_id: i64) -> Result<Option<i64>, String> {
    conn.query_row(
        "SELECT id
         FROM snapshots
         WHERE save_id = ?1
           AND is_current = 1
           AND EXISTS (SELECT 1 FROM managed_club_settings WHERE save_id = ?1)",
        [save_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|error| error.to_string())
}
