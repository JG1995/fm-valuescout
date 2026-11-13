//! Whether a player is a realistic signing for the save's managed club.
//!
//! This is a best guess, not Football Manager's own check. It copies the two largest terms of
//! the game's player-interest score and leaves out the rest, among them relationships,
//! rivalries, and league rules. The research is in `.wiki/notes/fm27-player-interest-filter.md`.
//!
//! A player is a realistic signing when the club already holds him, or when this score is at
//! least 5,000, the game's "Interested" level and its own default:
//!
//! - **Ability.** The club's level against the player's own, in points per ability point. The
//!   club's level is the mean current ability of its best players, raised as the game raises
//!   it for a player offered the best squad status. A young player counts part of his
//!   potential. A player who is too good and aged enough to know it is moved further away
//!   the more his reputation exceeds the club's.
//! - **Club reputation.** A step up from his current club adds to the score and a step down
//!   takes away several times as much.
//! - **Stature.** A world reputation above the club's takes away more.
//!
//! Every weight below is the game's, read from FM27 build 26577676.

/// A club is measured by this many of its best players.
const SQUAD_LEVEL_PLAYERS: u32 = 12;
/// What the club's level is multiplied by for a player offered its best squad status.
const BEST_STATUS_FACTOR: f64 = 1.1;
/// Score points per point of ability between the club's level and the player's.
const POINTS_PER_ABILITY_POINT: f64 = 880.0;
/// A player younger than this counts a share of his remaining potential as his level.
const POTENTIAL_AGE_LIMIT: u32 = 26;
const POTENTIAL_SHARE: f64 = 0.25;
/// The highest level a player counts for himself: the game's cap of 38,000 in ability points.
const PLAYER_LEVEL_CAP: f64 = 38_000.0 / 220.0;
/// From this age, a player too good for the club is moved away by his reputation.
const REPUTATION_AGE: u32 = 24;
/// Score points per point of team or world reputation.
const POINTS_PER_REPUTATION_POINT: u32 = 4;
/// Reputations count up to this value.
const REPUTATION_CAP: u32 = 9000;
/// The lowest score of a realistic signing: the game's "Interested" level.
const REALISTIC_SCORE: i32 = 5000;

/// SQL that is true for the realistic signings among `players` rows (`wanted`) or for the
/// rest, and null for every row when the managed club has no UID or no players.
///
/// The clause reads the snapshot from `?1`, which every search statement and the dashboard's
/// signings statement bind first, so the club's standing is worked out once for the statement
/// and not once for each player.
// ponytail: a club chosen before club UIDs were recorded matches nobody.
// Upgrade to matching that club by name if a save still holds one after its next Load Data.
pub(crate) fn clause(wanted: bool) -> String {
    let level = format!(
        "MIN(CASE WHEN players.age < {POTENTIAL_AGE_LIMIT} \
         THEN players.ca + {POTENTIAL_SHARE} * (players.pa - players.ca) \
         ELSE players.ca END, {PLAYER_LEVEL_CAP})"
    );
    let ability =
        format!("{POINTS_PER_ABILITY_POINT} * ({BEST_STATUS_FACTOR} * club.squad_level - {level})");
    let reputation_factor = "CASE \
         WHEN players.reputation_current > 1.4 * club.reputation THEN 20 \
         WHEN players.reputation_current > 1.3 * club.reputation THEN 10 \
         WHEN players.reputation_current > 1.2 * club.reputation THEN 5 \
         WHEN players.reputation_current > 0.9 * club.reputation THEN 2 \
         ELSE 1 END";
    let club_reputation = format!("MIN(club.reputation, {REPUTATION_CAP})");
    let step =
        format!("{POINTS_PER_REPUTATION_POINT} * ({club_reputation} - players.club_reputation)");
    let stature = format!(
        "{POINTS_PER_REPUTATION_POINT} * \
         ({club_reputation} - MIN(players.reputation_world, {REPUTATION_CAP}))"
    );
    // An unread reputation leaves its term out: every comparison with null is false, so each
    // CASE falls through to a null that COALESCE turns to zero, or to a factor of one.
    let score = format!(
        "{ability} * (CASE WHEN {ability} < 0 AND players.age >= {REPUTATION_AGE} \
                      THEN {reputation_factor} ELSE 1 END) \
         + COALESCE(CASE WHEN {step} <= -4001 THEN 6 * {step} \
                         WHEN {step} <= -3001 THEN 5 * {step} \
                         WHEN {step} <= -2001 THEN 4 * {step} \
                         WHEN {step} < -1000 THEN 2 * {step} \
                         ELSE {step} END, 0) \
         + COALESCE(CASE WHEN {stature} >= 0 THEN 0 \
                         WHEN {stature} >= -500 THEN {stature} \
                         ELSE 8 * {stature} END, 0)"
    );
    let realistic = format!(
        "(COALESCE(players.current_club_uid = club.uid, 0) \
          OR COALESCE(players.parent_club_uid = club.uid, 0) \
          OR {score} >= {REALISTIC_SCORE})"
    );
    let verdict = if wanted {
        realistic
    } else {
        format!("NOT {realistic}")
    };
    // MATERIALIZED keeps SQLite from merging `club` into the per-player test, which would
    // read the club's squad again for every player.
    format!(
        "EXISTS (WITH club AS MATERIALIZED (\
           SELECT managed.club_uid AS uid, \
             (SELECT AVG(best.ca) FROM (\
                SELECT squad.ca FROM players squad \
                WHERE squad.snapshot_id = ?1 AND squad.current_club_uid = managed.club_uid \
                ORDER BY squad.ca DESC LIMIT {SQUAD_LEVEL_PLAYERS}) best) AS squad_level, \
             (SELECT MAX(squad.club_reputation) FROM players squad \
              WHERE squad.snapshot_id = ?1 AND squad.current_club_uid = managed.club_uid) \
               AS reputation \
           FROM managed_club_settings managed \
           INNER JOIN snapshots ON snapshots.save_id = managed.save_id \
           WHERE snapshots.id = ?1 AND managed.club_uid IS NOT NULL) \
         SELECT 1 FROM club WHERE club.squad_level IS NOT NULL AND {verdict})"
    )
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::clause;
    use crate::db::migrations;

    #[test]
    fn the_managed_club_is_measured_once_for_a_statement() {
        let conn = Connection::open_in_memory().expect("open db");
        migrations::apply(&conn).expect("apply migrations");
        let sql = format!(
            "EXPLAIN QUERY PLAN SELECT uid FROM players WHERE players.snapshot_id = ?1 AND {}",
            clause(true)
        );

        let plan = conn
            .prepare(&sql)
            .expect("prepare plan")
            .query_map([1], |row| row.get::<_, String>(3))
            .expect("read plan")
            .collect::<Result<Vec<_>, _>>()
            .expect("plan rows");

        assert!(
            plan.iter().any(|step| step == "MATERIALIZE club"),
            "the club's standing is not materialized: {plan:?}"
        );
    }
}
