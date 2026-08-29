use rusqlite::{params, Connection};

use super::assignment_optimizer::{
    CoachRequirement, StaffAssignmentClassification, StaffAssignmentSlot,
};
use super::assignment_optimizer_query::{
    load_candidates, optimize_staff_assignments, StaffAssignmentOptimizationState,
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
fn loads_fm26_candidate_scores_from_current_shortlist_only() {
    let (_temp_dir, conn, _save_token) = open();
    insert_current_snapshot(&conn, 1);
    insert_staff(&conn, 1, "Fitness", Some("Club A"));
    insert_staff(&conn, 2, "Goalkeeping", Some("Club A"));
    insert_staff(&conn, 3, "Recruitment analyst", Some("Other FC"));
    insert_staff(&conn, 4, "Outside shortlist", Some("Club A"));
    insert_staff(&conn, 6, "Fitness missing score", Some("Club A"));
    shortlist(&conn, 1, "Fitness Coach", "-");
    shortlist(&conn, 2, "Goalkeeping Coach", "-");
    shortlist(&conn, 3, "Recruitment Analyst", "-");
    shortlist(&conn, 5, "Recruitment Analyst", "-");
    shortlist(&conn, 6, "Fitness Coach", "-");
    conn.execute_batch(
        "INSERT INTO snapshots (
             id, save_id, is_current, schema_version, generated_at_utc,
             game_version, supported_game_version, bridge_version,
             protocol_version, game_date_source, scan_truncated, player_count
         ) VALUES (2, 1, 0, 8, 'now', '26.3', '26.3', '0.4', 1, 'unknown', 0, 0);
         INSERT INTO staff (
             snapshot_id, uid, name, age, nationalities_json, gender, ca, pa,
             staff_attributes_json, club
         ) VALUES (2, 5, 'Wrong snapshot', 40, '[]', 'unknown', 100, 120, '{}', 'Club A');
         INSERT INTO staff_role_scores (snapshot_id, uid, role_id, score) VALUES
             (1, 1, 'coach_fitness', 71),
             (1, 2, 'coach_goalkeeping', 72),
             (1, 3, 'recruitment_analyst', 83),
             (1, 4, 'coach_fitness', 99),
             (1, 6, 'coach_fitness', NULL),
             (2, 5, 'recruitment_analyst', 99);",
    )
    .expect("insert score fixtures");

    let candidates = load_candidates(&conn, 1, 1, "Club A").expect("load candidates");

    assert_eq!(candidates.len(), 4);
    assert_eq!(candidates[0].uid, 1);
    assert_eq!(candidates[0].scores.coach_fitness, Some(71));
    assert_eq!(candidates[0].scores.coach_goalkeeping, None);
    assert_eq!(candidates[1].uid, 2);
    assert_eq!(candidates[1].scores.coach_goalkeeping, Some(72));
    assert_eq!(candidates[2].uid, 3);
    assert_eq!(candidates[2].scores.recruitment_analyst, Some(83));
    assert_eq!(
        candidates[2].classification,
        StaffAssignmentClassification::Recruitment
    );
    assert_eq!(candidates[3].uid, 6);
    assert_eq!(candidates[3].scores.coach_fitness, None);
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
             (1, 'reserves', 'manager', 1),
             (1, 'club', 'head_physio', 1),
             (1, 'club', 'head_sports_science', 1),
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
             (1, 4, 'coach_attacking_technical', 80),
             (1, 4, 'coach_possession_tactical', 90),
             (1, 6, 'coach_attacking_technical', NULL),
             (1, 8, 'manager', 81),
             (1, 11, 'assistant_manager', 73);",
    )
    .expect("insert persisted scores");

    let result = optimize_staff_assignments(&conn, &save_token, &snapshot_token)
        .expect("optimize shortlist");

    assert_eq!(result.state, StaffAssignmentOptimizationState::Ready);
    assert_eq!(result.joined_candidate_count, 10);
    assert_eq!(result.configured_slot_count, 7);
    assert_eq!(result.unsupported_preferred_job_count, 3);
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
                && recommendation.coach_requirement == Some(CoachRequirement::PossessionTactical)
    )));
    assert!(result.slots.iter().any(|slot| matches!(
        &slot.slot,
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 11 && recommendation.name.is_none()
    )));
    assert!(result.slots.iter().any(|slot| matches!(
        &slot.slot,
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 8 && recommendation.job_id == "manager" && recommendation.score == 81
    )));
    assert!(result.slots.iter().filter(|slot| matches!(
        &slot.slot,
        StaffAssignmentSlot::Vacancy(vacancy)
            if matches!(vacancy.job_id.as_str(), "chief_scout" | "head_physio" | "head_sports_science")
    )).count() == 3);
    assert!(result.slots.iter().any(|slot|
        slot.scope_display_name == "First Team"
            && matches!(&slot.slot, StaffAssignmentSlot::Recommendation(recommendation) if recommendation.scope == "senior")
    ));
    assert!(result.slots.iter().any(|slot|
        slot.scope_display_name == "Club"
            && matches!(&slot.slot, StaffAssignmentSlot::Vacancy(vacancy) if vacancy.scope == "club")
    ));
}

#[test]
fn allocates_leads_and_recruitment_analyst_from_persisted_scores() {
    let (_temp_dir, conn, save_token) = open();
    let snapshot_token = insert_current_snapshot(&conn, 1);
    configure_managed_club(&conn);
    conn.execute_batch(
        "INSERT INTO planner_teams (save_id, team, display_name) VALUES
             (1, 'senior', 'Senior'),
             (1, 'reserves', 'Reserves');
         INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count) VALUES
             (1, 'senior', 'performance_analyst', 1),
             (1, 'senior', 'physio', 1),
             (1, 'reserves', 'sports_scientist', 1),
             (1, 'club', 'head_performance_analyst', 1),
             (1, 'club', 'chief_scout', 1),
             (1, 'club', 'scout', 1),
             (1, 'club', 'recruitment_analyst', 1),
             (1, 'club', 'head_physio', 1),
             (1, 'club', 'head_sports_science', 1);",
    )
    .expect("configure targets");
    for (uid, preferred_job) in [
        (1, "Performance Analyst"),
        (2, "Performance Analyst"),
        (3, "Scout"),
        (4, "Scout"),
        (5, "Physio"),
        (6, "Physio"),
        (7, "Sports Scientist"),
        (8, "Sports Scientist"),
        (9, "Recruitment Analyst"),
    ] {
        insert_staff(&conn, uid, &format!("Staff {uid}"), Some("Club A"));
        shortlist(&conn, uid, preferred_job, "-");
    }
    conn.execute_batch(
        "INSERT INTO staff_role_scores (snapshot_id, uid, role_id, score) VALUES
             (1, 1, 'head_performance_analyst', 95),
             (1, 1, 'performance_analyst', 60),
             (1, 2, 'head_performance_analyst', 80),
             (1, 2, 'performance_analyst', 90),
             (1, 3, 'scout', 85),
             (1, 4, 'scout', 70),
             (1, 5, 'physio', 85),
             (1, 6, 'physio', 70),
             (1, 7, 'sports_scientist', 85),
             (1, 8, 'sports_scientist', 70),
             (1, 9, 'recruitment_analyst', 99);",
    )
    .expect("insert persisted scores");

    let result = optimize_staff_assignments(&conn, &save_token, &snapshot_token)
        .expect("optimize staff assignments");

    assert_eq!(result.state, StaffAssignmentOptimizationState::Ready);
    assert_eq!(result.configured_slot_count, 9);
    for (job_id, uid) in [
        ("performance_analyst", 2),
        ("physio", 6),
        ("sports_scientist", 8),
        ("head_performance_analyst", 1),
        ("chief_scout", 3),
        ("scout", 4),
        ("recruitment_analyst", 9),
        ("head_physio", 5),
        ("head_sports_science", 7),
    ] {
        assert!(result.slots.iter().any(|slot| matches!(
            &slot.slot,
            StaffAssignmentSlot::Recommendation(recommendation)
                if recommendation.job_id == job_id && recommendation.uid == uid
        )));
    }
    assert_eq!(
        result
            .slots
            .iter()
            .map(|result_slot| match &result_slot.slot {
                StaffAssignmentSlot::Recommendation(recommendation) => {
                    recommendation.job_id.as_str()
                }
                StaffAssignmentSlot::Vacancy(vacancy) => vacancy.job_id.as_str(),
            })
            .collect::<Vec<_>>(),
        [
            "head_performance_analyst",
            "performance_analyst",
            "chief_scout",
            "scout",
            "recruitment_analyst",
            "head_physio",
            "physio",
            "head_sports_science",
            "sports_scientist",
        ]
    );
}

#[test]
fn allocates_persisted_fitness_and_goalkeeping_scores_with_exact_vacancy_evidence() {
    let (_temp_dir, conn, save_token) = open();
    let snapshot_token = insert_current_snapshot(&conn, 1);
    configure_managed_club(&conn);
    conn.execute(
        "INSERT INTO planner_teams (save_id, team, display_name) VALUES (1, 'senior', 'Senior')",
        [],
    )
    .expect("configure senior");
    conn.execute(
        "INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count)
         VALUES (1, 'senior', 'coaches', 10)",
        [],
    )
    .expect("configure coaches");
    for (uid, preferred_job) in [
        (1, "Coach"),
        (2, "Coach"),
        (3, "Coach"),
        (4, "Coach"),
        (5, "Coach"),
        (6, "Coach"),
        (7, "Fitness Coach"),
        (8, "Fitness Coach"),
        (9, "Goalkeeping Coach"),
        (10, "Goalkeeping Coach"),
    ] {
        insert_staff(&conn, uid, &format!("Staff {uid}"), Some("Club A"));
        shortlist(&conn, uid, preferred_job, "-");
    }
    conn.execute_batch(
        "INSERT INTO staff_role_scores (snapshot_id, uid, role_id, score) VALUES
             (1, 1, 'coach_attacking_technical', 80),
             (1, 2, 'coach_attacking_tactical', 80),
             (1, 3, 'coach_defending_technical', 80),
             (1, 4, 'coach_defending_tactical', 80),
             (1, 5, 'coach_possession_technical', 80),
             (1, 6, 'coach_possession_tactical', 80),
             (1, 7, 'coach_fitness', 75),
             (1, 8, 'coach_fitness', NULL),
             (1, 9, 'coach_goalkeeping', 76),
             (1, 10, 'coach_goalkeeping', NULL);",
    )
    .expect("insert persisted scores");

    let result = optimize_staff_assignments(&conn, &save_token, &snapshot_token)
        .expect("optimize typed coach requirements");

    assert!(matches!(
        &result.slots[1].slot,
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 9
                && recommendation.coach_requirement == Some(CoachRequirement::Goalkeeping)
    ));
    assert!(matches!(
        &result.slots[2].slot,
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 7
                && recommendation.coach_requirement == Some(CoachRequirement::Fitness)
    ));
    assert!(matches!(
        &result.slots[8].slot,
        StaffAssignmentSlot::Vacancy(vacancy)
            if vacancy.coach_requirement == Some(CoachRequirement::Goalkeeping)
                && vacancy.evidence.joined_candidate_count == 2
                && vacancy.evidence.eligible_score_count == 1
                && vacancy.evidence.unavailable_score_count == 1
    ));
    assert!(matches!(
        &result.slots[9].slot,
        StaffAssignmentSlot::Vacancy(vacancy)
            if vacancy.coach_requirement == Some(CoachRequirement::Fitness)
                && vacancy.evidence.joined_candidate_count == 2
                && vacancy.evidence.eligible_score_count == 1
                && vacancy.evidence.unavailable_score_count == 1
    ));
}

#[test]
fn places_standalone_club_results_after_enabled_squads_without_senior() {
    let (_temp_dir, conn, save_token) = open();
    let snapshot_token = insert_current_snapshot(&conn, 1);
    configure_managed_club(&conn);
    conn.execute_batch(
        "INSERT INTO planner_teams (save_id, team, display_name) VALUES
             (1, 'reserves', 'B Squad'),
             (1, 'youth', 'Academy');
         INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count) VALUES
             (1, 'reserves', 'coaches', 1),
             (1, 'youth', 'manager', 1),
             (1, 'club', 'head_physio', 1);",
    )
    .expect("configure targets");
    insert_staff(&conn, 1, "Unsupported", Some("Club A"));
    shortlist(&conn, 1, "Kit Manager", "-");

    let result = optimize_staff_assignments(&conn, &save_token, &snapshot_token)
        .expect("optimize without senior");

    assert_eq!(result.state, StaffAssignmentOptimizationState::Ready);
    assert_eq!(
        result
            .slots
            .iter()
            .map(|result_slot| {
                let scope = match &result_slot.slot {
                    StaffAssignmentSlot::Recommendation(recommendation) => {
                        recommendation.scope.as_str()
                    }
                    StaffAssignmentSlot::Vacancy(vacancy) => vacancy.scope.as_str(),
                };
                (scope, result_slot.scope_display_name.as_str())
            })
            .collect::<Vec<_>>(),
        [
            ("reserves", "B Squad"),
            ("youth", "Academy"),
            ("club", "Club"),
        ]
    );
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
    for (scope, job_ids) in [
        (
            "senior",
            [
                "assistant_manager",
                "coaches",
                "performance_analyst",
                "physio",
                "sports_scientist",
            ]
            .as_slice(),
        ),
        (
            "reserves",
            [
                "manager",
                "assistant_manager",
                "coaches",
                "performance_analyst",
                "physio",
                "sports_scientist",
            ]
            .as_slice(),
        ),
        (
            "youth",
            [
                "manager",
                "assistant_manager",
                "coaches",
                "performance_analyst",
                "physio",
                "sports_scientist",
            ]
            .as_slice(),
        ),
    ] {
        for job_id in job_ids {
            conn.execute(
                "INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count)
                 VALUES (1, ?1, ?2, 50)",
                params![scope, job_id],
            )
            .expect("insert team target");
        }
    }
    for job_id in ["scout", "recruitment_analyst"] {
        conn.execute(
            "INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count)
             VALUES (1, 'club', ?1, 50)",
            [job_id],
        )
        .expect("insert club count target");
    }
    for job_id in [
        "head_of_youth_development",
        "head_performance_analyst",
        "set_piece_coach",
        "director_of_football",
        "chief_scout",
        "technical_director",
        "loan_manager",
        "head_physio",
        "head_sports_science",
    ] {
        conn.execute(
            "INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count)
             VALUES (1, 'club', ?1, 1)",
            [job_id],
        )
        .expect("insert club lead target");
    }
    shortlist(&conn, 999, "Assistant Manager", "-");

    let result = optimize_staff_assignments(&conn, &save_token, &snapshot_token)
        .expect("read bounded vacancies");

    assert_eq!(result.state, StaffAssignmentOptimizationState::Ready);
    assert_eq!(result.configured_slot_count, 959);
    assert_eq!(result.slots.len(), 959);
}
