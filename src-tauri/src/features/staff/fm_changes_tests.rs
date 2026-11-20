use std::cell::RefCell;

use rusqlite::{params, Connection};

use super::commands::read_staff_fm_changes;
use super::fm_changes::{
    apply_fm_staff_changes, plan_fm_staff_changes, FmStaffChangeKind, LOAD_DATA_FIRST,
};
use crate::db::{migrations, Db};
use crate::features::memory_read::service::{
    StaffChangeItem, StaffChangeOutcome, StaffChangeResponse,
};
use crate::features::player::boost_gate::BOOST_TEST_GATE;

const CLUB_UID: i64 = 101;

/// A managed club whose optimizer result needs every kind of change: the old assistant (1) is
/// sacked for a better one from another club (4), the second assistant (2) moves to the U21s,
/// and a scout (3) becomes Chief Scout.
fn club_with_changes() -> (tempfile::TempDir, Connection, String, String) {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let conn = Connection::open(temp_dir.path().join("fm-changes.db")).expect("open database");
    conn.pragma_update(None, "foreign_keys", true)
        .expect("enable foreign keys");
    migrations::apply(&conn).expect("apply migrations");
    conn.execute_batch(
        "INSERT INTO saves (id, name, is_active) VALUES (1, 'Save', 1);
         INSERT INTO snapshots (
             id, save_id, is_current, schema_version, generated_at_utc, game_version,
             supported_game_version, bridge_version, protocol_version, game_date_source,
             scan_truncated, player_count
         ) VALUES (1, 1, 1, 10, 'now', '27.1', '27', '0.31', 1, 'unknown', 0, 0);
         INSERT INTO managed_club_settings (save_id, club_name, club_uid) VALUES (1, 'Club A', 101);
         INSERT INTO planner_teams (save_id, team, display_name) VALUES
             (1, 'senior', 'First Team'),
             (1, 'reserves', 'U21');
         INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count) VALUES
             (1, 'senior', 'assistant_manager', 1),
             (1, 'reserves', 'assistant_manager', 1),
             (1, 'club', 'chief_scout', 1);",
    )
    .expect("configure club, teams and targets");
    for (uid, name, club, club_uid, job, wage) in [
        (
            1,
            "Old Assistant",
            Some("Club A"),
            Some(CLUB_UID),
            Some(20),
            Some(4000),
        ),
        (
            2,
            "Second Assistant",
            Some("Club A"),
            Some(CLUB_UID),
            Some(20),
            Some(3000),
        ),
        (
            3,
            "Rising Scout",
            Some("Club A"),
            Some(CLUB_UID),
            Some(14),
            Some(1500),
        ),
        (
            4,
            "Better Assistant",
            Some("Other FC"),
            Some(202),
            Some(20),
            Some(9000),
        ),
    ] {
        conn.execute(
            "INSERT INTO staff (
                 snapshot_id, uid, name, age, nationalities_json, gender, ca, pa,
                 staff_attributes_json, club, club_uid, job_id, weekly_wage_gbp,
                 team_type, team_level
             ) VALUES (1, ?1, ?2, 40, '[]', 'unknown', 100, 120, '{}', ?3, ?4, ?5, ?6, 0, 'senior')",
            params![uid, name, club, club_uid, job, wage],
        )
        .expect("insert staff");
    }
    conn.execute_batch(
        "INSERT INTO staff_shortlist_entries (
             save_id, staff_uid, preferred_job, coaching_qualifications
         ) VALUES (1, 4, 'Assistant Manager', 'Continental Pro');
         INSERT INTO staff_role_metrics (snapshot_id, uid, score_model_version, assistant_manager, scout)
         VALUES (1, 1, 1, 60, NULL), (1, 2, 1, 70, NULL), (1, 3, 1, NULL, 80), (1, 4, 1, 90, NULL);",
    )
    .expect("shortlist and score");
    let save_token = conn
        .query_row("SELECT context_token FROM saves WHERE id = 1", [], |row| {
            row.get(0)
        })
        .expect("save token");
    let snapshot_token = conn
        .query_row(
            "SELECT context_token FROM snapshots WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .expect("snapshot token");
    (temp_dir, conn, save_token, snapshot_token)
}

fn record_live_scan(conn: &Connection) {
    conn.execute(
        "UPDATE snapshots SET bridge_source_request_id = 'scan-1' WHERE id = 1",
        [],
    )
    .expect("record live scan");
}

type Contract = (
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<String>,
    Option<String>,
);

/// The person's club UID, job, wage, team level and club name in the snapshot.
fn contract(conn: &Connection, uid: i64) -> Contract {
    conn.query_row(
        "SELECT club_uid, job_id, weekly_wage_gbp, team_level, club FROM staff
         WHERE snapshot_id = 1 AND uid = ?1",
        [uid],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        },
    )
    .expect("read contract")
}

fn employed(outcome: &str, job: i32, wage: i64, team_type: i32, level: &str) -> StaffChangeOutcome {
    StaffChangeOutcome {
        outcome: outcome.to_string(),
        club_uid: Some(101),
        club: Some("Club A".to_string()),
        job_id: Some(job),
        weekly_wage_gbp: Some(wage),
        contract_expiry_year: Some(2031),
        contract_expiry_day_of_year: Some(181),
        team_type: Some(team_type),
        team_level: Some(level.to_string()),
        ..StaffChangeOutcome::default()
    }
}

fn unemployed(outcome: &str) -> StaffChangeOutcome {
    StaffChangeOutcome {
        outcome: outcome.to_string(),
        ..StaffChangeOutcome::default()
    }
}

#[test]
fn plans_sacks_then_changes_then_hires_with_fm_jobs_and_team_levels() {
    let (_temp_dir, conn, save_token, snapshot_token) = club_with_changes();

    let plan = plan_fm_staff_changes(&conn, &save_token, &snapshot_token).expect("plan changes");

    let summary = plan
        .iter()
        .map(|change| {
            (
                change.uid,
                change.kind,
                change
                    .target
                    .as_ref()
                    .map(|target| (target.job_type, target.level)),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        summary,
        [
            (1, FmStaffChangeKind::Sack, None),
            (3, FmStaffChangeKind::Change, Some((44, "senior"))),
            (2, FmStaffChangeKind::Change, Some((20, "reserve"))),
            (4, FmStaffChangeKind::Hire, Some((20, "senior"))),
        ]
    );
    let person = |uid| {
        plan.iter()
            .find(|change| change.uid == uid)
            .expect("planned")
    };
    assert_eq!(person(1).current_job.as_deref(), Some("Assistant Manager"));
    assert_eq!(person(1).current_team_name.as_deref(), Some("First Team"));
    assert_eq!(person(1).weekly_wage_gbp, Some(4000));
    assert_eq!(person(2).current_team_name.as_deref(), Some("First Team"));
    assert_eq!(
        person(2)
            .target
            .as_ref()
            .map(|target| target.scope_display_name.as_str()),
        Some("U21")
    );
    assert_eq!(person(3).current_job.as_deref(), Some("Scout"));
    assert_eq!(person(4).current_club.as_deref(), Some("Other FC"));
}

#[test]
fn applies_the_chosen_changes_and_records_each_contract_fm_reports() {
    let _test_guard = BOOST_TEST_GATE
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let (_temp_dir, conn, save_token, snapshot_token) = club_with_changes();
    record_live_scan(&conn);
    let db = Db::new(conn);
    let sent = RefCell::new(Vec::<StaffChangeItem>::new());

    let report = apply_fm_staff_changes(
        &db,
        &save_token,
        &snapshot_token,
        &[1, 2, 4],
        |source, club, items| {
            assert_eq!((source, club), ("scan-1", 101));
            sent.borrow_mut().extend_from_slice(items);
            Ok(StaffChangeResponse {
                changes: vec![
                    unemployed("sacked"),
                    employed("changed", 20, 2800, 10, "reserve"),
                    employed("hired", 20, 8500, 0, "senior"),
                ],
                error: None,
            })
        },
    )
    .expect("apply changes");

    assert_eq!(
        *sent.borrow(),
        [
            StaffChangeItem {
                staff_uid: 1,
                kind: "sack",
                job_type: None,
                target_level: None
            },
            StaffChangeItem {
                staff_uid: 2,
                kind: "change",
                job_type: Some(20),
                target_level: Some("reserve")
            },
            StaffChangeItem {
                staff_uid: 4,
                kind: "hire",
                job_type: Some(20),
                target_level: Some("senior")
            },
        ]
    );
    assert_eq!(report.done.len(), 3);
    assert!(report.failed.is_empty());
    assert_eq!(report.done[2].job_label, Some("Assistant Manager"));
    assert_eq!(report.done[2].weekly_wage_gbp, Some(8500));

    let conn = db.0.lock().expect("lock db");
    assert_eq!(contract(&conn, 1), (None, None, None, None, None));
    assert_eq!(
        contract(&conn, 2),
        (
            Some(101),
            Some(20),
            Some(2800),
            Some("reserve".to_string()),
            Some("Club A".to_string())
        )
    );
    assert_eq!(
        contract(&conn, 4),
        (
            Some(101),
            Some(20),
            Some(8500),
            Some("senior".to_string()),
            Some("Club A".to_string())
        )
    );
    assert_eq!(
        contract(&conn, 3).1,
        Some(14),
        "an unticked change stays as it was"
    );
}

#[test]
fn reports_a_person_left_unemployed_and_a_hire_fm_did_not_confirm() {
    let _test_guard = BOOST_TEST_GATE
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let (_temp_dir, conn, save_token, snapshot_token) = club_with_changes();
    record_live_scan(&conn);
    let db = Db::new(conn);
    let mut still_elsewhere = employed("not-confirmed", 20, 9000, 0, "senior");
    still_elsewhere.club_uid = Some(202);
    still_elsewhere.club = Some("Other FC".to_string());

    let report = apply_fm_staff_changes(&db, &save_token, &snapshot_token, &[3, 4], |_, _, _| {
        Ok(StaffChangeResponse {
            changes: vec![unemployed("left-unemployed"), still_elsewhere],
            error: Some("FM rejected an action: boom".to_string()),
        })
    })
    .expect("apply changes");

    assert!(report.done.is_empty());
    assert_eq!(report.error.as_deref(), Some("FM rejected an action: boom"));
    assert_eq!(
        report
            .failed
            .iter()
            .map(|failure| (failure.uid, failure.reason.as_str()))
            .collect::<Vec<_>>(),
        [
            (
                3,
                "FM ended the contract but did not hire them again; re-hire them with FM's editor"
            ),
            (
                4,
                "FM did not confirm the new job; they hold Assistant Manager now"
            ),
        ]
    );
    let conn = db.0.lock().expect("lock db");
    assert_eq!(contract(&conn, 3), (None, None, None, None, None));
    assert_eq!(contract(&conn, 4).0, Some(202));
}

#[test]
fn says_why_changes_cannot_be_applied_and_refuses_before_asking_the_bridge() {
    let _test_guard = BOOST_TEST_GATE
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let (_temp_dir, conn, save_token, snapshot_token) = club_with_changes();
    let db = Db::new(conn);
    let never = |_: &str, _: u32, _: &[StaffChangeItem]| -> Result<StaffChangeResponse, String> {
        panic!("nothing may reach the bridge")
    };

    let preview = read_staff_fm_changes(&db, &save_token, &snapshot_token, || {
        panic!("no bridge read without a live scan")
    })
    .expect("preview without a live scan");
    assert_eq!(preview.unavailable_reason.as_deref(), Some(LOAD_DATA_FIRST));
    assert_eq!(preview.changes.len(), 4);
    assert_eq!(
        apply_fm_staff_changes(&db, &save_token, &snapshot_token, &[1], never),
        Err(LOAD_DATA_FIRST.to_string())
    );

    record_live_scan(&db.0.lock().expect("lock db"));
    let ready = read_staff_fm_changes(&db, &save_token, &snapshot_token, || None)
        .expect("preview ready to apply");
    assert_eq!(ready.unavailable_reason, None);
    assert_eq!(
        apply_fm_staff_changes(&db, &save_token, &snapshot_token, &[99], never),
        Err("None of the chosen staff needs a change".to_string())
    );
    assert_eq!(
        apply_fm_staff_changes(&db, "stale", &snapshot_token, &[1], never),
        Err("The data changed; optimize your staff again".to_string())
    );
}
