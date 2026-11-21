//! The staff shortlist a save reads from its linked FM save file.

use rusqlite::{params, Connection};

use crate::features::save_stats::SaveShortlistedStaff;

/// Replaces save `save_id`'s staff shortlist with `staff`, whose Unique IDs are distinct, and
/// returns how many people it now holds. Run it inside the caller's transaction.
///
/// Staff the current snapshot lacks are stored too: the shortlist belongs to the save and is
/// joined to whichever snapshot is current when it is read.
pub(crate) fn replace_save_file_shortlist(
    conn: &Connection,
    save_id: i64,
    staff: &[SaveShortlistedStaff],
) -> rusqlite::Result<usize> {
    conn.execute(
        "DELETE FROM staff_shortlist_entries WHERE save_id = ?1",
        [save_id],
    )?;
    let mut insert = conn.prepare(
        "INSERT INTO staff_shortlist_entries (
            save_id, staff_uid, preferred_job, coaching_qualifications,
            on_senior_shortlist, on_reserves_shortlist, on_youth_shortlist
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;
    for person in staff {
        let on_team_shortlist = |word| names_team(&person.shortlists, word);
        insert.execute(params![
            save_id,
            i64::from(person.unique_id),
            person.preferred_job,
            person.coaching_qualifications,
            on_team_shortlist("senior"),
            on_team_shortlist("reserve"),
            on_team_shortlist("youth"),
        ])?;
    }
    Ok(staff.len())
}

/// Whether one of `shortlists` is a Planner team's staff shortlist: its name holds the team's
/// word in any case, as "Staff - Senior" or "Reserve Staff" do.
fn names_team(shortlists: &[String], word: &str) -> bool {
    shortlists
        .iter()
        .any(|name| name.to_lowercase().contains(word))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::features::snapshot::service::active_save_id;

    #[test]
    fn a_shortlist_naming_a_team_in_any_case_marks_its_people_for_that_team() {
        let (_database_directory, db) = db::open_with_golden_snapshot();
        let conn = db.0.lock().expect("lock");
        let save_id = active_save_id(&conn).expect("active save");
        let person = |unique_id, shortlists: &[&str]| SaveShortlistedStaff {
            unique_id,
            preferred_job: "Coach",
            coaching_qualifications: "-",
            shortlists: shortlists.iter().map(|name| name.to_string()).collect(),
        };

        replace_save_file_shortlist(
            &conn,
            save_id,
            &[
                person(1, &["Staff - SENIOR"]),
                person(2, &["Reserves Staff"]),
                person(3, &["staff for the youth teams", "Reserve coaches"]),
                person(4, &["", "Staff targets"]),
            ],
        )
        .expect("store the shortlist");

        let stored: Vec<(i64, bool, bool, bool)> = conn
            .prepare(
                "SELECT staff_uid, on_senior_shortlist, on_reserves_shortlist,
                        on_youth_shortlist
                 FROM staff_shortlist_entries WHERE save_id = ?1 ORDER BY staff_uid",
            )
            .expect("prepare entries")
            .query_map([save_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .expect("query entries")
            .collect::<Result<_, _>>()
            .expect("read entries");
        assert_eq!(
            stored,
            [
                (1, true, false, false),
                (2, false, true, false),
                (3, false, true, true),
                (4, false, false, false),
            ]
        );
    }
}
