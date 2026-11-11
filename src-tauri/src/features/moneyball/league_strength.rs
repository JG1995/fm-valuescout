//! How strong each league is, and how far that moves a Moneyball score.
//!
//! A Moneyball score ranks a player against everyone with statistics, in whatever league they
//! play, so the best of a weak league score like the best of a strong one. Each score is
//! therefore moved by the gap between the player's league and the managed club's.
//!
//! Every weight here is a judgment call, not a number from Football Manager. Tune them here.

use std::collections::{BTreeMap, HashMap};

use rusqlite::{params, Connection, OptionalExtension};

use crate::features::managed_club::service::selected_club_identity;

/// A club is measured by this many of its best players, and only when it has that many.
const CLUB_MEASURE_PLAYERS: usize = 11;
/// A league with this many measured clubs is taken at its measured strength. With fewer it
/// leans towards the thin-league strength, and with none it takes it.
const FULL_CONFIDENCE_CLUBS: i64 = 6;
/// The thin-league strength is the one this share of the fully measured leagues fall below:
/// a league the save holds too few players of counts as weak.
const THIN_LEAGUE_QUANTILE: f64 = 0.10;
/// Score points per point of league strength. At one point per point, a strong league's
/// players crowd out everyone else; at half, the standouts of the next leagues down stay
/// in view.
const POINTS_PER_STRENGTH_POINT: f64 = 0.5;

/// One league as measured from a snapshot's players.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LeagueMeasure {
    pub(crate) division: String,
    /// The mean strength of the league's measured clubs. A club's strength is the mean
    /// current ability of its best players.
    pub(crate) club_strength: f64,
    pub(crate) clubs: i64,
}

/// Measures every league from each player's division, club UID, and current ability. A club
/// counts under the division its players carry, and players without either are left out.
pub(crate) fn measure<'a>(
    players: impl IntoIterator<Item = (Option<&'a str>, Option<i64>, i64)>,
) -> Vec<LeagueMeasure> {
    // Ordered maps keep the sums, and so the stored strengths, the same on every run.
    let mut clubs: BTreeMap<(&str, i64), Vec<i64>> = BTreeMap::new();
    for (division, club_uid, current_ability) in players {
        if let (Some(division), Some(club_uid)) = (division, club_uid) {
            clubs
                .entry((division, club_uid))
                .or_default()
                .push(current_ability);
        }
    }
    let mut leagues: BTreeMap<&str, (f64, i64)> = BTreeMap::new();
    for ((division, _), mut abilities) in clubs {
        if abilities.len() < CLUB_MEASURE_PLAYERS {
            continue;
        }
        abilities.sort_unstable_by(|left, right| right.cmp(left));
        let best = &abilities[..CLUB_MEASURE_PLAYERS];
        let league = leagues.entry(division).or_default();
        league.0 += best.iter().sum::<i64>() as f64 / CLUB_MEASURE_PLAYERS as f64;
        league.1 += 1;
    }
    leagues
        .into_iter()
        .map(|(division, (strength_sum, clubs))| LeagueMeasure {
            division: division.to_owned(),
            club_strength: strength_sum / clubs as f64,
            clubs,
        })
        .collect()
}

/// Stores a snapshot's measured leagues.
pub(crate) fn persist(
    conn: &Connection,
    snapshot_id: i64,
    measures: &[LeagueMeasure],
) -> Result<(), String> {
    let mut statement = conn
        .prepare(
            "INSERT INTO league_strengths (snapshot_id, division, club_strength, clubs)
             VALUES (?1, ?2, ?3, ?4)",
        )
        .map_err(|error| error.to_string())?;
    for measure in measures {
        statement
            .execute(params![
                snapshot_id,
                measure.division,
                measure.club_strength,
                measure.clubs
            ])
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

/// Whether the save's Moneyball scores are adjusted for league strength.
pub(crate) fn is_enabled(conn: &Connection, save_id: i64) -> Result<bool, String> {
    conn.query_row(
        "SELECT moneyball_league_adjusted FROM saves WHERE id = ?1",
        [save_id],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

pub(crate) fn set_enabled(conn: &Connection, save_id: i64, enabled: bool) -> Result<(), String> {
    conn.execute(
        "UPDATE saves SET moneyball_league_adjusted = ?2 WHERE id = ?1",
        params![save_id, enabled],
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

/// What moved the Moneyball scores of one player.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LeagueBreakdown {
    /// The player's league, or none when the snapshot records none for him.
    pub(crate) division: Option<String>,
    pub(crate) strength: f64,
    /// The league has too few measured clubs to be taken at its measured strength.
    pub(crate) estimated: bool,
    pub(crate) reference_division: String,
    pub(crate) reference_strength: f64,
    /// The score points added to each of his scores: negative in a weaker league.
    pub(crate) shift: i16,
}

/// The league adjustment of one snapshot's Moneyball scores, or none.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct LeagueAdjustment(Option<Strengths>);

#[derive(Debug, Clone, PartialEq)]
struct Strengths {
    /// Each measured league's strength and whether it is an estimate.
    leagues: HashMap<String, (f64, bool)>,
    /// The strength of a league with no measured club.
    thin: f64,
    reference_division: String,
    reference_strength: f64,
}

impl LeagueAdjustment {
    /// The adjustment for `snapshot_id`: none when its save has turned it off or the snapshot
    /// has no fully measured league to judge the others by.
    pub(crate) fn load(conn: &Connection, snapshot_id: i64) -> Result<Self, String> {
        let save_id = snapshot_save_id(conn, snapshot_id)?;
        if !is_enabled(conn, save_id)? {
            return Ok(Self(None));
        }
        Ok(Self(Strengths::read(conn, save_id, snapshot_id)?))
    }

    /// The league the save's current snapshot adjusts scores to, whether or not the
    /// adjustment is on. None when that snapshot has no fully measured league.
    pub(crate) fn reference_division(
        conn: &Connection,
        save_id: i64,
    ) -> Result<Option<String>, String> {
        let snapshot_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
                [save_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        let Some(snapshot_id) = snapshot_id else {
            return Ok(None);
        };
        Ok(Strengths::read(conn, save_id, snapshot_id)?
            .map(|strengths| strengths.reference_division))
    }

    /// Whether any score is moved.
    pub(crate) fn applies(&self) -> bool {
        self.0.is_some()
    }

    /// The score points a player in `division` gains: negative in a league weaker than the
    /// reference, zero without an adjustment.
    pub(crate) fn shift(&self, division: Option<&str>) -> i16 {
        self.0
            .as_ref()
            .map_or(0, |strengths| strengths.shift(strengths.league(division).0))
    }

    /// What moves the scores of a player in `division`, or none without an adjustment.
    pub(crate) fn breakdown(&self, division: Option<&str>) -> Option<LeagueBreakdown> {
        let strengths = self.0.as_ref()?;
        let (strength, estimated) = strengths.league(division);
        Some(LeagueBreakdown {
            division: division.map(str::to_owned),
            strength,
            estimated,
            reference_division: strengths.reference_division.clone(),
            reference_strength: strengths.reference_strength,
            shift: strengths.shift(strength),
        })
    }
}

impl Strengths {
    fn read(conn: &Connection, save_id: i64, snapshot_id: i64) -> Result<Option<Self>, String> {
        let measures = conn
            .prepare(
                "SELECT division, club_strength, clubs FROM league_strengths
                 WHERE snapshot_id = ?1 ORDER BY division",
            )
            .map_err(|error| error.to_string())?
            .query_map([snapshot_id], |row| {
                Ok(LeagueMeasure {
                    division: row.get(0)?,
                    club_strength: row.get(1)?,
                    clubs: row.get(2)?,
                })
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;

        let mut full = measures
            .iter()
            .filter(|measure| measure.clubs >= FULL_CONFIDENCE_CLUBS)
            .collect::<Vec<_>>();
        full.sort_by(|left, right| left.club_strength.total_cmp(&right.club_strength));
        let Some(strongest) = full.last() else {
            return Ok(None);
        };
        let thin =
            full[((full.len() - 1) as f64 * THIN_LEAGUE_QUANTILE).round() as usize].club_strength;
        let leagues = measures
            .iter()
            .map(|measure| {
                let confidence =
                    (measure.clubs as f64 / FULL_CONFIDENCE_CLUBS as f64).clamp(0.0, 1.0);
                let strength = confidence * measure.club_strength + (1.0 - confidence) * thin;
                (measure.division.clone(), (strength, confidence < 1.0))
            })
            .collect::<HashMap<_, _>>();

        let mut strengths = Self {
            leagues,
            thin,
            reference_division: strongest.division.clone(),
            reference_strength: strongest.club_strength,
        };
        if let Some(division) = managed_club_division(conn, save_id, snapshot_id)? {
            strengths.reference_strength = strengths.league(Some(&division)).0;
            strengths.reference_division = division;
        }
        Ok(Some(strengths))
    }

    /// The score points a league of `strength` gains against the reference.
    fn shift(&self, strength: f64) -> i16 {
        ((strength - self.reference_strength) * POINTS_PER_STRENGTH_POINT).round() as i16
    }

    /// A league's strength and whether it is an estimate. An unknown league is a thin one.
    fn league(&self, division: Option<&str>) -> (f64, bool) {
        division
            .and_then(|division| self.leagues.get(division).copied())
            .unwrap_or((self.thin, true))
    }
}

/// `score` moved by a league `shift`, kept on the 0–100 scale.
pub(crate) fn shifted(score: u8, shift: i16) -> u8 {
    (i16::from(score) + shift).clamp(0, 100) as u8
}

fn snapshot_save_id(conn: &Connection, snapshot_id: i64) -> Result<i64, String> {
    conn.query_row(
        "SELECT save_id FROM snapshots WHERE id = ?1",
        [snapshot_id],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

/// The division most of the managed club's players carry in the snapshot.
fn managed_club_division(
    conn: &Connection,
    save_id: i64,
    snapshot_id: i64,
) -> Result<Option<String>, String> {
    let Some((club_name, club_uid)) = selected_club_identity(conn, save_id)? else {
        return Ok(None);
    };
    // The UID reads an index. A club chosen before UIDs were recorded is matched by name.
    let (club_sql, club) = match club_uid {
        Some(uid) => (
            "current_club_uid = ?2",
            rusqlite::types::Value::Integer(uid),
        ),
        None => ("current_club = ?2", rusqlite::types::Value::Text(club_name)),
    };
    conn.query_row(
        &format!(
            "SELECT division FROM players
             WHERE snapshot_id = ?1 AND {club_sql} AND division IS NOT NULL
             GROUP BY division ORDER BY COUNT(*) DESC, division LIMIT 1"
        ),
        params![snapshot_id, club],
        |row| row.get(0),
    )
    .optional()
    .map_err(|error| error.to_string())
}

#[cfg(test)]
pub(crate) mod test_support {
    use rusqlite::Connection;

    use super::{persist, LeagueMeasure, FULL_CONFIDENCE_CLUBS};

    /// Records each league's strength for the snapshot, over enough clubs to be taken as
    /// measured.
    pub(crate) fn measure_leagues(conn: &Connection, snapshot_id: i64, leagues: &[(&str, f64)]) {
        let measures = leagues
            .iter()
            .map(|&(division, club_strength)| LeagueMeasure {
                division: division.to_owned(),
                club_strength,
                clubs: FULL_CONFIDENCE_CLUBS,
            })
            .collect::<Vec<_>>();
        persist(conn, snapshot_id, &measures).expect("persist league strengths");
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use super::{measure, persist, set_enabled, shifted, LeagueAdjustment, LeagueMeasure};
    use crate::db::migrations;

    /// A club's players as `measure` reads them: `count` players of falling ability from `best`.
    fn club(
        division: &'static str,
        club_uid: i64,
        best: i64,
        count: i64,
    ) -> Vec<(Option<&'static str>, Option<i64>, i64)> {
        (0..count)
            .map(|rank| (Some(division), Some(club_uid), best - rank))
            .collect()
    }

    fn league(division: &str, club_strength: f64, clubs: i64) -> LeagueMeasure {
        LeagueMeasure {
            division: division.to_owned(),
            club_strength,
            clubs,
        }
    }

    /// A save whose current snapshot measures a strong, a middle, and a weak full league,
    /// and a two-club league.
    fn fixture() -> (Connection, i64, i64) {
        let conn = Connection::open_in_memory().expect("open database");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn.execute("INSERT INTO saves (name, is_active) VALUES ('Save', 1)", [])
            .expect("insert save");
        let save_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO snapshots (
                 save_id, is_current, schema_version, generated_at_utc, game_version,
                 supported_game_version, bridge_version, protocol_version, game_date_source,
                 scan_truncated, player_count
             ) VALUES (?1, 1, 9, '2026-11-11T10:00:00Z', '27.0', '27.0', '0.24.0', 1, 'memory',
                       0, 0)",
            [save_id],
        )
        .expect("insert snapshot");
        let snapshot_id = conn.last_insert_rowid();
        persist(
            &conn,
            snapshot_id,
            &[
                league("Top", 150.0, 20),
                league("Middle", 120.0, 12),
                league("Weak", 90.0, 6),
                league("Thin", 150.0, 2),
            ],
        )
        .expect("persist leagues");
        (conn, save_id, snapshot_id)
    }

    fn insert_player(conn: &Connection, snapshot_id: i64, uid: i64, club_uid: i64, division: &str) {
        conn.execute(
            "INSERT INTO players (
                 snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                 nationalities_json, preferred_foot, positions_json, attributes_json,
                 hidden_attributes_json, personality_json, current_club, current_club_uid,
                 division
             ) VALUES (?1, ?2, 100, 100, 'Player', 2000, 1, '[]', 'Right', '{}', '{}', '{}',
                       '{}', 'Club', ?3, ?4)",
            params![snapshot_id, uid, club_uid, division],
        )
        .expect("insert player");
    }

    fn manage_club(conn: &Connection, save_id: i64, club_uid: i64) {
        conn.execute(
            "INSERT INTO managed_club_settings (save_id, club_name, club_uid)
             VALUES (?1, 'Club', ?2)",
            params![save_id, club_uid],
        )
        .expect("set managed club");
    }

    #[test]
    fn a_league_is_the_mean_of_its_clubs_best_eleven() {
        let mut players = club("Top", 1, 160, 30);
        players.extend(club("Top", 2, 140, 11));
        // Ten players do not measure a club, and neither do players without a club or league.
        players.extend(club("Top", 3, 200, 10));
        players.extend(club("Other", 4, 100, 11));
        players.push((None, Some(1), 200));
        players.push((Some("Top"), None, 200));

        assert_eq!(
            measure(players),
            [league("Other", 95.0, 1), league("Top", 145.0, 2)]
        );
    }

    #[test]
    fn load_data_measures_the_snapshots_leagues() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = Connection::open(temp_dir.path().join("load.db")).expect("open database");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        let mut dump: serde_json::Value =
            serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v9.json"))
                .expect("parse golden dump");
        let template = dump["players"][0].clone();
        let players = (1..=11)
            .map(|uid| {
                let mut player = template.clone();
                player["uid"] = uid.into();
                player["ca"] = (100 + uid).into();
                player["pa"] = 150.into();
                player["currentClubUid"] = 500.into();
                player["division"] = "Top".into();
                player
            })
            .collect::<Vec<_>>();
        dump["playerCount"] = players.len().into();
        dump["players"] = serde_json::Value::Array(players);
        let dump_path = temp_dir.path().join("dump.json");
        std::fs::write(&dump_path, dump.to_string()).expect("write dump");

        let snapshot_id =
            crate::features::snapshot::ingest::ingest_dump_file(&mut conn, &dump_path)
                .expect("ingest dump")
                .id;

        let leagues: Vec<(String, f64, i64)> = conn
            .prepare("SELECT division, club_strength, clubs FROM league_strengths WHERE snapshot_id = ?1")
            .expect("prepare leagues")
            .query_map([snapshot_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .expect("query leagues")
            .collect::<Result<_, _>>()
            .expect("collect leagues");
        assert_eq!(leagues, [("Top".to_owned(), 106.0, 1)]);
    }

    #[test]
    fn a_score_moves_by_the_gap_to_the_strongest_league_without_a_managed_club() {
        let (conn, _, snapshot_id) = fixture();
        let league = LeagueAdjustment::load(&conn, snapshot_id).expect("load");

        // Half a score point for each of the 30 points between the leagues.
        assert_eq!(league.shift(Some("Top")), 0);
        assert_eq!(league.shift(Some("Middle")), -15);
        let breakdown = league.breakdown(Some("Middle")).expect("breakdown");
        assert_eq!(breakdown.reference_division, "Top");
        assert_eq!(
            (breakdown.strength, breakdown.reference_strength),
            (120.0, 150.0)
        );
        assert!(!breakdown.estimated);
    }

    #[test]
    fn a_score_moves_by_the_gap_to_the_managed_clubs_league() {
        let (conn, save_id, snapshot_id) = fixture();
        insert_player(&conn, snapshot_id, 1, 500, "Middle");
        insert_player(&conn, snapshot_id, 2, 500, "Middle");
        insert_player(&conn, snapshot_id, 3, 500, "Youth league");
        manage_club(&conn, save_id, 500);
        let league = LeagueAdjustment::load(&conn, snapshot_id).expect("load");

        assert_eq!(
            [
                league.shift(Some("Top")),
                league.shift(Some("Middle")),
                league.shift(Some("Weak"))
            ],
            [15, 0, -15]
        );
        assert_eq!(
            league
                .breakdown(Some("Weak"))
                .expect("breakdown")
                .reference_division,
            "Middle"
        );
    }

    #[test]
    fn a_thin_league_leans_towards_the_weakest_measured_leagues() {
        let (conn, _, snapshot_id) = fixture();
        let league = LeagueAdjustment::load(&conn, snapshot_id).expect("load");

        // Two of six clubs: a third of its measured 150 and two thirds of the thin 90.
        let thin = league.breakdown(Some("Thin")).expect("breakdown");
        assert_eq!(
            (thin.strength, thin.estimated, thin.shift),
            (110.0, true, -20)
        );
        // A league with no measured club, and a player with no league, take the thin strength.
        for division in [Some("Unmeasured"), None] {
            let unknown = league.breakdown(division).expect("breakdown");
            assert_eq!(
                (unknown.strength, unknown.estimated, unknown.shift),
                (90.0, true, -30)
            );
        }
    }

    #[test]
    fn nothing_moves_when_the_save_turns_the_adjustment_off() {
        let (conn, save_id, snapshot_id) = fixture();
        set_enabled(&conn, save_id, false).expect("turn off");
        let league = LeagueAdjustment::load(&conn, snapshot_id).expect("load");

        assert_eq!(league.shift(Some("Weak")), 0);
        assert_eq!(league.breakdown(Some("Weak")), None);
        assert_eq!(
            LeagueAdjustment::reference_division(&conn, save_id).expect("reference"),
            Some("Top".to_owned())
        );
    }

    #[test]
    fn nothing_moves_without_a_fully_measured_league() {
        let (conn, save_id, snapshot_id) = fixture();
        conn.execute("DELETE FROM league_strengths WHERE clubs >= 6", [])
            .expect("delete full leagues");
        let league = LeagueAdjustment::load(&conn, snapshot_id).expect("load");

        assert_eq!(league.shift(Some("Thin")), 0);
        assert_eq!(
            LeagueAdjustment::reference_division(&conn, save_id).expect("reference"),
            None
        );
    }

    #[test]
    fn a_shifted_score_stays_between_0_and_100() {
        assert_eq!(
            [shifted(90, -24), shifted(10, -24), shifted(90, 24)],
            [66, 0, 100]
        );
    }
}
