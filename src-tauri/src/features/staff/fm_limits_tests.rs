use std::cell::Cell;

use rusqlite::{params, Connection};

use super::assignment_targets::get_targets;
use super::fm_limits::{apply_staff_limits, follow_fm_staff_limits, StaffLimitsOutcome};
use crate::db::{migrations, Db};
use crate::features::memory_read::service::{
    StaffLimitsGroup, StaffLimitsJob, StaffLimitsResult, StaffLimitsTeam,
};
use crate::features::snapshot::service::SaveContext;

/// A save managing club 680 with three Planner teams and needs set by hand.
fn club() -> (tempfile::TempDir, Connection, SaveContext) {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let conn = Connection::open(temp_dir.path().join("fm-limits.db")).expect("open database");
    conn.pragma_update(None, "foreign_keys", true)
        .expect("enable foreign keys");
    migrations::apply(&conn).expect("apply migrations");
    conn.execute_batch(
        "INSERT INTO saves (id, name, is_active) VALUES (1, 'Save', 1);
         INSERT INTO managed_club_settings (save_id, club_name, club_uid) VALUES (1, 'Club A', 680);
         INSERT INTO planner_teams (save_id, team, display_name) VALUES
             (1, 'senior', 'First Team'),
             (1, 'reserves', 'U21'),
             (1, 'youth', 'U18');
         INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count) VALUES
             (1, 'senior', 'coaches', 10),
             (1, 'reserves', 'physio', 3),
             (1, 'youth', 'sports_scientist', 1),
             (1, 'club', 'scout', 20);",
    )
    .expect("configure club, teams and needs");
    let context_token = conn
        .query_row("SELECT context_token FROM saves WHERE id = 1", [], |row| {
            row.get(0)
        })
        .expect("read context token");
    (
        temp_dir,
        conn,
        SaveContext {
            id: 1,
            context_token,
        },
    )
}

fn job(job_type: i32, max: i32) -> StaffLimitsJob {
    StaffLimitsJob {
        job_type,
        name: None,
        count: None,
        max,
    }
}

fn team(level: &str, team_type: i32, jobs: Vec<StaffLimitsJob>) -> StaffLimitsTeam {
    StaffLimitsTeam {
        level: level.to_string(),
        team_type,
        groups: if jobs.is_empty() {
            Vec::new()
        } else {
            vec![StaffLimitsGroup {
                group: "coaching".to_string(),
                name: None,
                count: None,
                max: jobs.iter().map(|job| job.max).sum(),
                jobs,
            }]
        },
    }
}

/// FM's limits as the bridge read them on 2026-11-20: the first team's coaching, recruitment and
/// medical jobs, a few reserve jobs, and nothing for the youth team. FM's board counts every
/// coach in one row of job type 104. `coaches` is the first team's Coaches limit.
fn limits(coaches: i32, reserve_physios: i32) -> StaffLimitsResult {
    StaffLimitsResult {
        teams: vec![
            team(
                "senior",
                0,
                vec![
                    job(16, 1),
                    job(20, 1),
                    job(104, coaches),
                    job(60, 10),
                    job(22, 2),
                    job(64, 1),
                    job(14, 16),
                    job(58, 12),
                    job(12, 6),
                    job(46, 2),
                ],
            ),
            team(
                "reserve",
                10,
                vec![
                    job(16, 1),
                    job(104, 4),
                    job(12, reserve_physios),
                    job(14, 3),
                ],
            ),
            team("youth", 12, Vec::new()),
        ],
    }
}

fn needs(conn: &Connection) -> Vec<(String, String, i64)> {
    conn.prepare(
        "SELECT scope, job_id, slot_count FROM staff_assignment_targets
         WHERE save_id = 1 ORDER BY scope, job_id",
    )
    .expect("prepare needs")
    .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
    .expect("query needs")
    .collect::<Result<_, _>>()
    .expect("read needs")
}

fn need(scope: &str, job_id: &str, slot_count: i64) -> (String, String, i64) {
    (scope.to_string(), job_id.to_string(), slot_count)
}

#[test]
fn the_first_read_sets_each_teams_needs_and_the_club_needs_from_the_first_team() {
    let (_temp_dir, conn, context) = club();

    let outcome = apply_staff_limits(&conn, &context, &limits(13, 2)).expect("apply limits");

    assert_eq!(
        outcome,
        StaffLimitsOutcome::Followed {
            changed: 11,
            unread_teams: vec!["U18".to_string()],
        }
    );
    // FM's Coaches row sets Coaches, the Set Piece Coach is capped at one, the first team's
    // Manager and the Doctor set nothing, and the reserves' Scout row is not club-wide.
    assert_eq!(
        needs(&conn),
        vec![
            need("club", "head_of_youth_development", 1),
            need("club", "recruitment_analyst", 12),
            need("club", "scout", 16),
            need("club", "set_piece_coach", 1),
            need("reserves", "coaches", 4),
            need("reserves", "manager", 1),
            need("reserves", "physio", 2),
            need("senior", "assistant_manager", 1),
            need("senior", "coaches", 13),
            need("senior", "performance_analyst", 10),
            need("senior", "physio", 6),
            need("youth", "sports_scientist", 1),
        ]
    );
    let targets = get_targets(&conn, &context.context_token).expect("read targets");
    assert!(targets.fm_limits.iter().any(|limit| limit.scope == "senior"
        && limit.job_id == "coaches"
        && limit.slot_limit == 13));
    assert!(!targets.fm_limits.iter().any(|limit| limit.scope == "youth"));
}

#[test]
fn an_edited_need_stays_until_fms_limit_for_it_changes() {
    let (_temp_dir, conn, context) = club();
    apply_staff_limits(&conn, &context, &limits(13, 2)).expect("apply limits");
    conn.execute(
        "UPDATE staff_assignment_targets SET slot_count = 12
         WHERE save_id = 1 AND scope = 'senior' AND job_id = 'coaches'",
        [],
    )
    .expect("edit a need");

    let unchanged = apply_staff_limits(&conn, &context, &limits(13, 2)).expect("apply again");
    assert_eq!(
        unchanged,
        StaffLimitsOutcome::Followed {
            changed: 0,
            unread_teams: vec!["U18".to_string()],
        }
    );
    assert!(needs(&conn).contains(&need("senior", "coaches", 12)));

    let followed = apply_staff_limits(&conn, &context, &limits(14, 0)).expect("apply new limits");
    assert_eq!(
        followed,
        StaffLimitsOutcome::Followed {
            changed: 2,
            unread_teams: vec!["U18".to_string()],
        }
    );
    let needs = needs(&conn);
    assert!(needs.contains(&need("senior", "coaches", 14)));
    assert!(!needs
        .iter()
        .any(|(scope, job_id, _)| scope == "reserves" && job_id == "physio"));
}

#[test]
fn needs_stay_when_the_limits_are_not_read_or_the_save_changed() {
    let (_temp_dir, conn, context) = club();
    let before = needs(&conn);

    let stale = SaveContext {
        id: 1,
        context_token: "another-load".to_string(),
    };
    assert_eq!(
        apply_staff_limits(&conn, &stale, &limits(13, 2)).expect("apply to a stale save"),
        StaffLimitsOutcome::StaleContext
    );

    let db = Db::new(conn);
    let not_read = follow_fm_staff_limits(&db, &context, "scan-1", |source, club_uid| {
        assert_eq!((source, club_uid), ("scan-1", 680));
        Err("the bridge plugin cannot read FM's staff limits".to_string())
    });
    assert_eq!(
        not_read,
        StaffLimitsOutcome::NotRead {
            reason: "the bridge plugin cannot read FM's staff limits".to_string(),
        }
    );

    let conn = db.0.lock().expect("lock database");
    assert_eq!(needs(&conn), before);
    conn.execute(
        "UPDATE managed_club_settings SET club_uid = NULL WHERE save_id = ?1",
        params![1],
    )
    .expect("forget the club's identity");
    drop(conn);
    let read = Cell::new(false);
    assert_eq!(
        follow_fm_staff_limits(&db, &context, "scan-1", |_, _| {
            read.set(true);
            Ok(limits(13, 2))
        }),
        StaffLimitsOutcome::NoManagedClub
    );
    assert!(!read.get());
}
