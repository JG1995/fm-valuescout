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
            save_id, staff_uid, preferred_job, coaching_qualifications
         ) VALUES (?1, ?2, ?3, ?4)",
    )?;
    for person in staff {
        insert.execute(params![
            save_id,
            i64::from(person.unique_id),
            person.preferred_job,
            person.coaching_qualifications,
        ])?;
    }
    Ok(staff.len())
}
