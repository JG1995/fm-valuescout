use rusqlite::{params, Connection};

use super::assignment_optimizer::{
    CoachDiscipline, StaffAssignmentClassification, StaffAssignmentSlot,
};
use super::assignment_optimizer_query::{
    optimize_staff_assignments, StaffAssignmentOptimizationState,
};
use crate::db::migrations;

fn open() -> (tempfile::TempDir, Connection, String) {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let conn = Connection::open(temp_dir.path().join("optimizer.db")).expect("open database");
    conn.pragma_update(None, "foreign_keys", true)
        .expect("enable foreign keys");
    migrations::apply(&conn).expect("apply migrations");
    conn.execute(
        "INSERT INTO saves (id, name, is_active) VALUES (1, 'Save', 1)",
        [],
    )
    .expect("insert save");
    let save_token = conn
        .query_row("SELECT context_token FROM saves WHERE id = 1", [], |row| {
            row.get(0)
        })
        .expect("read save token");
    (temp_dir, conn, save_token)
}

fn insert_current_snapshot(conn: &Connection, id: i64) -> String {
    conn.execute(
        "INSERT INTO snapshots (
             id, save_id, is_current, schema_version, generated_at_utc,
             game_version, supported_game_version, bridge_version,
             protocol_version, game_date_source, scan_truncated, player_count
         ) VALUES (?1, 1, 1, 8, 'now', '26.3', '26.3', '0.4', 1, 'unknown', 0, 0)",
        [id],
    )
    .expect("insert current snapshot");
    conn.query_row(
        "SELECT context_token FROM snapshots WHERE id = ?1",
        [id],
        |row| row.get(0),
    )
    .expect("read snapshot token")
}

fn configure_managed_club(conn: &Connection) {
    conn.execute(
        "INSERT INTO managed_club_settings (save_id, club_name) VALUES (1, 'Club A')",
        [],
    )
    .expect("configure managed club");
}

fn insert_staff(conn: &Connection, uid: i64, name: &str, club: Option<&str>) {
    insert_staff_with_name(conn, uid, Some(name), club);
}

fn insert_staff_with_name(conn: &Connection, uid: i64, name: Option<&str>, club: Option<&str>) {
    conn.execute(
        "INSERT INTO staff (
             snapshot_id, uid, name, age, nationalities_json, gender, ca, pa,
             staff_attributes_json, club
         ) VALUES (1, ?1, ?2, 40, '[]', 'unknown', 100, 120, '{}', ?3)",
        params![uid, name, club],
    )
    .expect("insert staff");
}

fn shortlist(conn: &Connection, uid: i64, preferred_job: &str, club_job: &str) {
    conn.execute(
        "INSERT INTO staff_shortlist_entries (
             save_id, staff_uid, preferred_job, club_job, coaching_qualifications
         ) VALUES (1, ?1, ?2, ?3, 'Continental Pro')",
        params![uid, preferred_job, club_job],
    )
    .expect("insert shortlist entry");
}

#[test]
fn distinguishes_context_and_setup_states_without_writing() {
    let (_temp_dir, conn, save_token) = open();

    let no_snapshot = optimize_staff_assignments(&conn, &save_token, "expected-snapshot")
        .expect("read no-snapshot state");
    assert_eq!(
        no_snapshot.state,
        StaffAssignmentOptimizationState::NoCurrentSnapshot
    );
    assert!(no_snapshot.snapshot_id.is_none());

    let snapshot_token = insert_current_snapshot(&conn, 1);
    let no_managed_club = optimize_staff_assignments(&conn, &save_token, &snapshot_token)
        .expect("read no-managed-club state");
    assert_eq!(
        no_managed_club.state,
        StaffAssignmentOptimizationState::NoManagedClub
    );

    configure_managed_club(&conn);
    let no_shortlist = optimize_staff_assignments(&conn, &save_token, &snapshot_token)
        .expect("read no-shortlist state");
    assert_eq!(
        no_shortlist.state,
        StaffAssignmentOptimizationState::NoShortlist
    );

    shortlist(&conn, 999, "Assistant Manager", "-");
    let ready =
        optimize_staff_assignments(&conn, &save_token, &snapshot_token).expect("read ready state");
    assert_eq!(ready.state, StaffAssignmentOptimizationState::Ready);
    assert_eq!(ready.joined_candidate_count, 0);
    assert_eq!(ready.configured_slot_count, 0);
    assert!(ready.slots.is_empty());

    let saved_targets: i64 = conn
        .query_row("SELECT COUNT(*) FROM staff_assignment_targets", [], |row| {
            row.get(0)
        })
        .expect("count target writes");
    assert_eq!(saved_targets, 0);
    let planner_teams: i64 = conn
        .query_row("SELECT COUNT(*) FROM planner_teams", [], |row| row.get(0))
        .expect("count planner-team writes");
    assert_eq!(planner_teams, 0);
}

#[test]
fn rejects_stale_tokens_including_a_reused_numeric_snapshot_id() {
    let (_temp_dir, conn, save_token) = open();
    let original_snapshot_token = insert_current_snapshot(&conn, 1);
    configure_managed_club(&conn);
    shortlist(&conn, 1, "Assistant Manager", "-");
    insert_staff(&conn, 1, "First", Some("Club A"));

    let stale_save = optimize_staff_assignments(&conn, "stale-save", &original_snapshot_token)
        .expect("read stale-save state");
    assert_eq!(
        stale_save.state,
        StaffAssignmentOptimizationState::StaleContext
    );
    assert_eq!(stale_save.save_context_token, save_token);

    conn.execute("DELETE FROM snapshots WHERE id = 1", [])
        .expect("delete original snapshot");
    let replacement_snapshot_token = insert_current_snapshot(&conn, 1);
    assert_ne!(replacement_snapshot_token, original_snapshot_token);

    let stale_snapshot = optimize_staff_assignments(&conn, &save_token, &original_snapshot_token)
        .expect("read stale-snapshot state");
    assert_eq!(
        stale_snapshot.state,
        StaffAssignmentOptimizationState::StaleContext
    );
    assert_eq!(stale_snapshot.snapshot_id, Some(1));
    assert_eq!(
        stale_snapshot.snapshot_context_token.as_deref(),
        Some(replacement_snapshot_token.as_str())
    );
}

#[test]
fn joins_only_current_shortlist_staff_and_preserves_scores_and_classification() {
    let (_temp_dir, conn, save_token) = open();
    let snapshot_token = insert_current_snapshot(&conn, 1);
    configure_managed_club(&conn);
    conn.execute_batch(
        "INSERT INTO planner_teams (save_id, team, display_name) VALUES
             (1, 'senior', 'First Team'),
             (1, 'reserves', 'Reserves');
         INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count) VALUES
             (1, 'senior', 'assistant_manager', 2),
             (1, 'senior', 'coaches', 1),
             (1, 'senior', 'head_physio', 1),
             (1, 'reserves', 'manager', 1),
             (1, 'reserves', 'head_sports_science', 1),
             (1, 'club', 'chief_scout', 1);",
    )
    .expect("configure targets");

    insert_staff(&conn, 1, "Current assistant", Some("Club A"));
    insert_staff(&conn, 2, "Outside shortlist", Some("Club A"));
    insert_staff(&conn, 3, "Recruitment scout", Some("Other FC"));
    insert_staff(&conn, 4, "Coach with score", Some("Club A"));
    insert_staff(&conn, 5, "Coach missing score", Some("Club A"));
    insert_staff(&conn, 6, "Coach null score", Some("Club A"));
    insert_staff(&conn, 7, "Unsupported job", Some("Club A"));
    insert_staff(&conn, 8, "Manager", Some("Club A"));
    insert_staff(&conn, 9, "Head physio", Some("Club A"));
    insert_staff(&conn, 10, "Head sports science", Some("Club A"));
    insert_staff_with_name(&conn, 11, None, Some("Club A"));
    shortlist(&conn, 1, "Assistant Manager", "-");
    shortlist(&conn, 3, "Chief Scout", "Current");
    shortlist(&conn, 4, "Coach", "-");
    shortlist(&conn, 5, "Coach", "-");
    shortlist(&conn, 6, "Coach", "-");
    shortlist(&conn, 7, "Fitness Coach", "-");
    shortlist(&conn, 8, "Manager", "-");
    shortlist(&conn, 9, "Head Physio", "-");
    shortlist(&conn, 10, "Head of Sports Science", "-");
    shortlist(&conn, 11, "Assistant Manager", "-");
    conn.execute_batch(
        "INSERT INTO staff_role_scores (snapshot_id, uid, role_id, score) VALUES
             (1, 1, 'assistant_manager', 72),
             (1, 2, 'assistant_manager', 99),
             (1, 3, 'scout', 83),
             (1, 4, 'coach_attacking_technical', 80),
             (1, 4, 'coach_possession_tactical', 90),
             (1, 6, 'coach_attacking_technical', NULL),
             (1, 8, 'manager', 81),
             (1, 9, 'physio', 82),
             (1, 10, 'sports_scientist', 84),
             (1, 11, 'assistant_manager', 73);",
    )
    .expect("insert persisted scores");

    let result = optimize_staff_assignments(&conn, &save_token, &snapshot_token)
        .expect("optimize shortlist");

    assert_eq!(result.state, StaffAssignmentOptimizationState::Ready);
    assert_eq!(result.joined_candidate_count, 10);
    assert_eq!(result.configured_slot_count, 7);
    assert_eq!(result.unsupported_preferred_job_count, 1);
    assert!(result.slots.iter().any(|slot| matches!(
        &slot.slot,
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 1
                && recommendation.classification == StaffAssignmentClassification::CurrentStaff
                && recommendation.score == 72
    )));
    assert!(result.slots.iter().any(|slot| matches!(
        &slot.slot,
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 4
                && recommendation.classification == StaffAssignmentClassification::CurrentStaff
                && recommendation.score == 90
                && recommendation.coach_discipline == Some(CoachDiscipline::PossessionTactical)
    )));
    assert!(result.slots.iter().any(|slot| matches!(
        &slot.slot,
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 11 && recommendation.name.is_none()
    )));
    assert!(result.slots.iter().any(|slot| matches!(
        &slot.slot,
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 3
                && recommendation.classification == StaffAssignmentClassification::Recruitment
                && recommendation.score == 83
    )));
    assert!(result.slots.iter().any(|slot| matches!(
        &slot.slot,
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 8 && recommendation.job_id == "manager" && recommendation.score == 81
    )));
    assert!(result.slots.iter().any(|slot| matches!(
        &slot.slot,
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 9 && recommendation.job_id == "head_physio" && recommendation.score == 82
    )));
    assert!(result.slots.iter().any(|slot| matches!(
        &slot.slot,
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 10
                && recommendation.job_id == "head_sports_science"
                && recommendation.score == 84
    )));
    assert!(result.slots.iter().any(|slot|
        slot.scope_display_name == "First Team"
            && matches!(&slot.slot, StaffAssignmentSlot::Recommendation(recommendation) if recommendation.scope == "senior")
    ));
    assert!(result.slots.iter().any(|slot|
        slot.scope_display_name == "Club"
            && matches!(&slot.slot, StaffAssignmentSlot::Recommendation(recommendation) if recommendation.scope == "club")
    ));
    let coach_evidence = result
        .evidence
        .iter()
        .find(|evidence| evidence.job_id == "coaches")
        .expect("coach evidence");
    assert_eq!(coach_evidence.joined_candidate_count, 3);
    assert_eq!(coach_evidence.eligible_score_count, 1);
    assert_eq!(coach_evidence.unavailable_score_count, 2);
}

#[test]
fn caps_the_ready_result_at_the_supported_slot_limit() {
    let (_temp_dir, conn, save_token) = open();
    let snapshot_token = insert_current_snapshot(&conn, 1);
    configure_managed_club(&conn);
    conn.execute_batch(
        "INSERT INTO planner_teams (save_id, team, display_name) VALUES
             (1, 'senior', 'Senior'),
             (1, 'reserves', 'Reserves'),
             (1, 'youth', 'Youth');",
    )
    .expect("configure teams");
    for scope in ["senior", "reserves", "youth"] {
        for job_id in [
            "manager",
            "assistant_manager",
            "coaches",
            "set_piece_coach",
            "head_performance_analyst",
            "performance_analyst",
            "head_physio",
            "physio",
            "head_sports_science",
            "sports_scientist",
        ] {
            if scope == "senior" && job_id == "manager" {
                continue;
            }
            conn.execute(
                "INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count)
                 VALUES (1, ?1, ?2, 50)",
                params![scope, job_id],
            )
            .expect("insert team target");
        }
    }
    for job_id in [
        "head_of_youth_development",
        "director_of_football",
        "technical_director",
        "loan_manager",
        "chief_scout",
        "scout",
    ] {
        conn.execute(
            "INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count)
             VALUES (1, 'club', ?1, 50)",
            [job_id],
        )
        .expect("insert club target");
    }
    shortlist(&conn, 999, "Assistant Manager", "-");

    let result = optimize_staff_assignments(&conn, &save_token, &snapshot_token)
        .expect("read bounded vacancies");

    assert_eq!(result.state, StaffAssignmentOptimizationState::Ready);
    assert_eq!(result.configured_slot_count, 1_750);
    assert_eq!(result.slots.len(), 1_750);
}
