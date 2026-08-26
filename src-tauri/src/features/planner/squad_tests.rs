use rusqlite::params;
use serde_json::json;

use super::squad::{list_squad_players, SquadSortDir, SquadSortField, DEFAULT_SQUAD_PAGE_LIMIT};
use super::test_support::{
    add_picker_candidates, current_snapshot_id, open_with_snapshot, set_role_score,
};
use crate::features::player_metrics::{
    club_dna::SCORE_MODEL_VERSION, potential_cache::PROJECTION_MODEL_VERSION,
    resolver::DynamicValue,
};

#[test]
fn lists_the_exact_current_managed_club() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);

    let page = list_squad_players(
        &conn,
        save_id,
        0,
        DEFAULT_SQUAD_PAGE_LIMIT,
        SquadSortField::DEFAULT,
        SquadSortDir::DEFAULT,
        &[],
    )
    .expect("list configured squad");

    assert_eq!(page.total, 4);
    assert_eq!(
        page.players
            .iter()
            .map(|player| player.uid)
            .collect::<Vec<_>>(),
        [77, 78, 79, 80]
    );
}

#[test]
fn orders_every_fixed_column_and_pages_deterministically() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let snapshot_id = current_snapshot_id(&conn, save_id);
    for (uid, name, age, nationalities, division, ca, pa, market_value_gbp) in [
        (
            77,
            "Delta",
            Some(20),
            r#"["ZED"]"#,
            Some("Division B"),
            110,
            150,
            Some(1_000),
        ),
        (
            78,
            "Bravo",
            Some(21),
            r#"["YEN"]"#,
            Some("Division A"),
            120,
            140,
            None,
        ),
        (
            79,
            "Charlie",
            None,
            r#"["XEN"]"#,
            None,
            130,
            130,
            Some(3_000),
        ),
        (
            80,
            "Alpha",
            Some(19),
            r#"["WEN"]"#,
            Some("Division C"),
            140,
            120,
            Some(2_000),
        ),
    ] {
        conn.execute(
            "UPDATE players
             SET name = ?1,
                 age = ?2,
                 nationalities_json = ?3,
                 division = ?4,
                 ca = ?5,
                 pa = ?6,
                 market_value_gbp = ?7
             WHERE snapshot_id = ?8 AND uid = ?9",
            params![
                name,
                age,
                nationalities,
                division,
                ca,
                pa,
                market_value_gbp,
                snapshot_id,
                uid
            ],
        )
        .expect("set sortable values");
    }

    for (field, direction, expected) in [
        (
            SquadSortField::Name,
            SquadSortDir::Asc,
            vec![80, 78, 79, 77],
        ),
        (
            SquadSortField::Name,
            SquadSortDir::Desc,
            vec![77, 79, 78, 80],
        ),
        (SquadSortField::Age, SquadSortDir::Asc, vec![79, 80, 77, 78]),
        (
            SquadSortField::Age,
            SquadSortDir::Desc,
            vec![78, 77, 80, 79],
        ),
        (
            SquadSortField::Nationality,
            SquadSortDir::Asc,
            vec![80, 79, 78, 77],
        ),
        (
            SquadSortField::Nationality,
            SquadSortDir::Desc,
            vec![77, 78, 79, 80],
        ),
        (
            SquadSortField::Club,
            SquadSortDir::Asc,
            vec![77, 78, 79, 80],
        ),
        (
            SquadSortField::Club,
            SquadSortDir::Desc,
            vec![77, 78, 79, 80],
        ),
        (
            SquadSortField::Division,
            SquadSortDir::Asc,
            vec![79, 78, 77, 80],
        ),
        (
            SquadSortField::Division,
            SquadSortDir::Desc,
            vec![80, 77, 78, 79],
        ),
        (SquadSortField::Ca, SquadSortDir::Asc, vec![77, 78, 79, 80]),
        (SquadSortField::Ca, SquadSortDir::Desc, vec![80, 79, 78, 77]),
        (SquadSortField::Pa, SquadSortDir::Asc, vec![80, 79, 78, 77]),
        (SquadSortField::Pa, SquadSortDir::Desc, vec![77, 78, 79, 80]),
        (
            SquadSortField::Value,
            SquadSortDir::Asc,
            vec![78, 77, 80, 79],
        ),
        (
            SquadSortField::Value,
            SquadSortDir::Desc,
            vec![79, 80, 77, 78],
        ),
    ] {
        let page = list_squad_players(&conn, save_id, 0, 4, field, direction, &[])
            .expect("sort squad players");
        assert_eq!(
            page.players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            expected
        );
    }

    let page = list_squad_players(
        &conn,
        save_id,
        1,
        2,
        SquadSortField::DEFAULT,
        SquadSortDir::DEFAULT,
        &[],
    )
    .expect("page squad players");
    assert_eq!(page.total, 4);
    assert_eq!(
        page.players
            .iter()
            .map(|player| player.uid)
            .collect::<Vec<_>>(),
        [79, 78]
    );
}

#[test]
fn orders_targeted_scalar_sorts_in_the_exact_managed_club_cohort() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    let previous_snapshot_id = current_snapshot_id(&conn, save_id);
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let snapshot_id = current_snapshot_id(&conn, save_id);
    for (uid, pa, age, market_value_gbp) in [
        (77, 100, Some(20), Some(100)),
        (78, 100, Some(20), Some(100)),
        (79, 150, Some(22), Some(300)),
        (80, 150, None, None),
    ] {
        conn.execute(
            "UPDATE players
             SET pa = ?1, age = ?2, market_value_gbp = ?3
             WHERE snapshot_id = ?4 AND uid = ?5",
            params![pa, age, market_value_gbp, snapshot_id, uid],
        )
        .expect("set targeted scalar values");
    }
    conn.execute(
        "INSERT INTO players (
             snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
             nationalities_json, preferred_foot, positions_json, attributes_json,
             hidden_attributes_json, personality_json, current_club
         ) VALUES (?1, 99, 100, 999, 'Other club', 2000, 1,
                   '[]', 'right', '{}', '{}', '{}', '{}', 'Other FC')",
        params![snapshot_id],
    )
    .expect("insert current non-cohort player");
    conn.execute(
        "INSERT INTO players (
             snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
             nationalities_json, preferred_foot, positions_json, attributes_json,
             hidden_attributes_json, personality_json, current_club
         ) VALUES (?1, 999, 100, 999, 'Archived club player', 2000, 1,
                   '[]', 'right', '{}', '{}', '{}', '{}', 'Loan FC')",
        params![previous_snapshot_id],
    )
    .expect("insert archived cohort player");

    for (field, direction, expected, expected_page) in [
        (
            SquadSortField::Pa,
            SquadSortDir::Asc,
            vec![77, 78, 79, 80],
            vec![78, 79],
        ),
        (
            SquadSortField::Pa,
            SquadSortDir::Desc,
            vec![79, 80, 77, 78],
            vec![80, 77],
        ),
        (
            SquadSortField::Age,
            SquadSortDir::Asc,
            vec![80, 77, 78, 79],
            vec![77, 78],
        ),
        (
            SquadSortField::Age,
            SquadSortDir::Desc,
            vec![79, 77, 78, 80],
            vec![77, 78],
        ),
        (
            SquadSortField::Value,
            SquadSortDir::Asc,
            vec![80, 77, 78, 79],
            vec![77, 78],
        ),
        (
            SquadSortField::Value,
            SquadSortDir::Desc,
            vec![79, 77, 78, 80],
            vec![77, 78],
        ),
    ] {
        let page = list_squad_players(&conn, save_id, 0, 4, field.clone(), direction, &[])
            .expect("sort targeted squad scalar values");
        assert_eq!(page.total, 4);
        assert_eq!(
            page.players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            expected
        );

        let bounded_page = list_squad_players(&conn, save_id, 1, 2, field, direction, &[])
            .expect("page targeted squad scalar values");
        assert_eq!(bounded_page.total, 4);
        assert_eq!(
            bounded_page
                .players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            expected_page
        );
    }
}

#[test]
fn current_role_sort_retains_missing_nullable_duplicate_and_tied_scores() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let snapshot_id = current_snapshot_id(&conn, save_id);
    let role_id = "deep_lying_playmaker_ip";
    set_role_score(&conn, save_id, 77, role_id, Some(80));
    set_role_score(&conn, save_id, 78, role_id, Some(80));
    set_role_score(&conn, save_id, 79, role_id, None);
    conn.execute(
        "DELETE FROM player_role_scores
         WHERE snapshot_id = ?1 AND uid = 80 AND role_id = ?2",
        params![snapshot_id, role_id],
    )
    .expect("remove current role score");
    let sort_by = SquadSortField::parse(&format!("role.{role_id}")).expect("parse role sort");

    for (direction, expected) in [
        (SquadSortDir::Asc, vec![79, 80, 77, 78]),
        (SquadSortDir::Desc, vec![77, 78, 79, 80]),
    ] {
        let page = list_squad_players(&conn, save_id, 0, 4, sort_by.clone(), direction, &[])
            .expect("sort current roles");
        assert_eq!(page.total, 4);
        assert_eq!(
            page.players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            expected
        );
    }

    let page = list_squad_players(&conn, save_id, 1, 2, sort_by, SquadSortDir::Asc, &[])
        .expect("page current roles");
    assert_eq!(page.total, 4);
    assert_eq!(
        page.players
            .iter()
            .map(|player| player.uid)
            .collect::<Vec<_>>(),
        vec![80, 77]
    );
}

#[test]
fn current_role_sort_uses_a_missing_preserving_relation() {
    let source = include_str!("squad.rs");
    let query = &source[source
        .find("pub fn list_squad_players")
        .expect("squad query function")
        ..source.find("fn empty_page").expect("following helper")];

    assert!(query.contains("LEFT JOIN player_role_scores current_role_sort"));
    assert!(query.contains("current_role_sort.snapshot_id = p.snapshot_id"));
    assert!(query.contains("current_role_sort.role_id = '{role_id}'"));
    assert!(query.contains("current_role_sort.uid = p.uid"));
    assert!(query.contains("ORDER BY current_role_sort.score"));
}

#[test]
fn potential_sort_uses_a_missing_preserving_exact_version_relation_and_skips_its_page_pass() {
    let source = include_str!("squad.rs");
    let query = &source[source
        .find("pub fn list_squad_players")
        .expect("squad query function")
        ..source.find("fn empty_page").expect("following helper")];

    assert!(query.contains("LEFT JOIN player_potential_role_scores potential_role_sort"));
    assert!(query.contains("potential_role_sort.snapshot_id = p.snapshot_id"));
    assert!(query.contains("potential_role_sort.uid = p.uid"));
    assert!(query.contains("potential_role_sort.projection_model_version"));
    assert!(query.contains("ORDER BY potential_role_sort.score"));
    assert!(query.contains("potential_display_roles.retain"));
    assert!(query.contains("role_id != identity.role_id"));
}

#[test]
fn potential_sort_orders_nullable_ties_and_materializes_only_the_distinct_visible_page_role() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let snapshot_id = current_snapshot_id(&conn, save_id);
    let sort_role = "line_holding_keeper_oop";
    let sort_metric = format!("potential_role.{sort_role}");
    let distinct_visible_role = "sweeper_keeper_oop";
    let distinct_visible_metric = format!("potential_role.{distinct_visible_role}");
    conn.execute("DELETE FROM player_potential_role_scores", [])
        .expect("clear eager rows for lazy nullable sort test");
    for (uid, score) in [(77, Some(80)), (78, Some(80)), (79, Some(40)), (80, None)] {
        conn.execute(
            "INSERT INTO player_potential_role_scores (
                 snapshot_id, uid, role_id, score, projection_model_version
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![snapshot_id, uid, sort_role, score, PROJECTION_MODEL_VERSION],
        )
        .expect("seed exact-version potential sort score");
    }
    let sort_by = SquadSortField::parse(&sort_metric).expect("parse potential sort");

    for (direction, expected) in [
        (SquadSortDir::Asc, vec![80, 79, 77, 78]),
        (SquadSortDir::Desc, vec![77, 78, 79, 80]),
    ] {
        let page = list_squad_players(&conn, save_id, 0, 4, sort_by.clone(), direction, &[])
            .expect("sort warm potential scores");
        assert_eq!(page.total, 4);
        assert_eq!(
            page.players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            expected
        );
    }

    let page = list_squad_players(
        &conn,
        save_id,
        1,
        2,
        sort_by,
        SquadSortDir::Asc,
        &[sort_metric, distinct_visible_metric],
    )
    .expect("page warm potential scores with a distinct visible role");
    assert_eq!(page.total, 4);
    assert_eq!(
        page.players
            .iter()
            .map(|player| player.uid)
            .collect::<Vec<_>>(),
        [79, 77]
    );
    let sorted_role_rows: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM player_potential_role_scores
             WHERE snapshot_id = ?1
               AND role_id = ?2
               AND projection_model_version = ?3",
            params![snapshot_id, sort_role, PROJECTION_MODEL_VERSION],
            |row| row.get(0),
        )
        .expect("count globally sorted role rows");
    assert_eq!(sorted_role_rows, 4);
    let distinct_visible_uids = conn
        .prepare(
            "SELECT uid FROM player_potential_role_scores
             WHERE snapshot_id = ?1
               AND role_id = ?2
               AND projection_model_version = ?3
             ORDER BY uid ASC",
        )
        .expect("prepare distinct visible role query")
        .query_map(
            params![snapshot_id, distinct_visible_role, PROJECTION_MODEL_VERSION],
            |row| row.get::<_, i64>(0),
        )
        .expect("query distinct visible role rows")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect distinct visible role rows");
    assert_eq!(distinct_visible_uids, [77, 79]);
}

#[test]
fn club_dna_sort_uses_a_missing_preserving_exact_identity_relation() {
    let source = include_str!("squad.rs");
    let query = &source[source
        .find("pub fn list_squad_players")
        .expect("squad query function")
        ..source.find("fn empty_page").expect("following helper")];

    assert!(query.contains("LEFT JOIN club_dna_scores club_dna_sort"));
    assert!(query.contains("club_dna_sort.snapshot_id = p.snapshot_id"));
    assert!(query.contains("club_dna_sort.uid = p.uid"));
    assert!(query.contains("club_dna_sort.definition_version"));
    assert!(query.contains("club_dna_sort.score_model_version"));
    assert!(query.contains("ORDER BY club_dna_sort.score IS NULL ASC"));
}

#[test]
fn current_role_sort_materializes_requested_potential_page_fields() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "UPDATE players
         SET positions_json = ?1, attributes_json = ?2
         WHERE snapshot_id = ?3 AND uid IN (77, 80)",
        params![
            json!({ "GK": 20 }).to_string(),
            json!({ "Positioning": 16, "Concentration": 16 }).to_string(),
            snapshot_id,
        ],
    )
    .expect("set potential source values");
    let role_id = "deep_lying_playmaker_ip";
    set_role_score(&conn, save_id, 77, role_id, Some(80));
    set_role_score(&conn, save_id, 78, role_id, Some(80));
    set_role_score(&conn, save_id, 79, role_id, None);
    conn.execute(
        "DELETE FROM player_role_scores
         WHERE snapshot_id = ?1 AND uid = 80 AND role_id = ?2",
        params![snapshot_id, role_id],
    )
    .expect("remove current role score");

    let potential_field = "potential_role.line_holding_keeper_oop".to_string();
    conn.execute(
        "DELETE FROM player_potential_role_scores
         WHERE snapshot_id = ?1 AND role_id = 'line_holding_keeper_oop'",
        [snapshot_id],
    )
    .expect("clear eager requested role for lazy page test");
    let page = list_squad_players(
        &conn,
        save_id,
        1,
        2,
        SquadSortField::parse(&format!("role.{role_id}")).expect("parse role sort"),
        SquadSortDir::Asc,
        std::slice::from_ref(&potential_field),
    )
    .expect("query current role page with potential field");

    assert_eq!(page.total, 4);
    assert_eq!(
        page.players
            .iter()
            .map(|player| player.uid)
            .collect::<Vec<_>>(),
        vec![80, 77]
    );
    assert!(matches!(
        page.players[0].dynamic_values.get(&potential_field),
        Some(Some(DynamicValue::Integer(_)))
    ));
}

#[test]
fn returns_no_players_without_a_configuration_or_matching_current_players() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    conn.execute(
        "DELETE FROM managed_club_settings WHERE save_id = ?1",
        params![save_id],
    )
    .expect("remove configuration");

    let no_configuration = list_squad_players(
        &conn,
        save_id,
        0,
        DEFAULT_SQUAD_PAGE_LIMIT,
        SquadSortField::DEFAULT,
        SquadSortDir::DEFAULT,
        &[],
    )
    .expect("list without configuration");
    assert_eq!(no_configuration.total, 0);
    assert!(no_configuration.players.is_empty());

    add_picker_candidates(&temp_dir, &mut conn, save_id);
    conn.execute(
        "UPDATE players
         SET current_club = 'Elsewhere FC'
         WHERE snapshot_id = ?1",
        params![current_snapshot_id(&conn, save_id)],
    )
    .expect("move current players outside managed club");
    let empty_current_result = list_squad_players(
        &conn,
        save_id,
        0,
        DEFAULT_SQUAD_PAGE_LIMIT,
        SquadSortField::DEFAULT,
        SquadSortDir::DEFAULT,
        &[],
    )
    .expect("list empty configured squad");
    assert_eq!(empty_current_result.total, 0);
    assert!(empty_current_result.players.is_empty());
}

#[test]
fn excludes_retained_players_from_an_earlier_snapshot() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    let previous_snapshot_id = current_snapshot_id(&conn, save_id);
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    conn.execute(
        "INSERT INTO players (
             snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
             nationalities_json, preferred_foot, positions_json, attributes_json,
             hidden_attributes_json, personality_json, current_club
         ) VALUES (?1, 999, 180, 190, 'Archived Player', 2000, 1,
                   '[]', 'right', '{}', '{}', '{}', '{}', 'Loan FC')",
        params![previous_snapshot_id],
    )
    .expect("add archived player");

    let page = list_squad_players(
        &conn,
        save_id,
        0,
        DEFAULT_SQUAD_PAGE_LIMIT,
        SquadSortField::DEFAULT,
        SquadSortDir::DEFAULT,
        &[],
    )
    .expect("list current squad");

    assert_eq!(page.total, 4);
    assert!(page.players.iter().all(|player| player.uid != 999));
}

#[test]
fn rejects_unknown_sort_inputs() {
    assert!(SquadSortField::parse("ca; DROP TABLE players").is_err());
    assert!(SquadSortDir::parse("sideways").is_err());
}

#[test]
fn accepts_position_as_a_sortable_display_metric() {
    assert!(SquadSortField::parse("position").is_ok());
}

#[test]
fn returns_requested_metrics_with_the_shared_search_value_contract() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "UPDATE players
         SET parent_club = 'Parent FC',
             positions_json = ?1,
             attributes_json = ?2,
             hidden_attributes_json = ?3,
             personality_json = ?4
         WHERE snapshot_id = ?5 AND uid = 77",
        params![
            json!({ "MC": 16, "AMC": 20, "AMR": 14, "GK": 0, "SW": null }).to_string(),
            json!({ "Acceleration": 16 }).to_string(),
            json!({ "Consistency": 12 }).to_string(),
            json!({ "Ambition": 14 }).to_string(),
            snapshot_id,
        ],
    )
    .expect("set requested metrics");
    set_role_score(&conn, save_id, 77, "deep_lying_playmaker_ip", Some(82));

    let requested_fields = vec![
        "parent_club".to_string(),
        "position".to_string(),
        "pos.AMR".to_string(),
        "pos.GK".to_string(),
        "pos.SW".to_string(),
        "attr.Acceleration".to_string(),
        "hidden.Consistency".to_string(),
        "personality.Ambition".to_string(),
        "role.deep_lying_playmaker_ip".to_string(),
        "attr.Acceleration".to_string(),
    ];
    let page = list_squad_players(
        &conn,
        save_id,
        0,
        DEFAULT_SQUAD_PAGE_LIMIT,
        SquadSortField::DEFAULT,
        SquadSortDir::DEFAULT,
        &requested_fields,
    )
    .expect("list requested metrics");
    let player = page
        .players
        .iter()
        .find(|player| player.uid == 77)
        .expect("configured player");

    assert_eq!(
        player.dynamic_values.get("parent_club"),
        Some(&Some(DynamicValue::Text("Parent FC".to_string())))
    );
    assert_eq!(
        player.dynamic_values.get("position"),
        Some(&Some(DynamicValue::Text("AMC, MC, AMR".to_string())))
    );
    assert_eq!(
        player.dynamic_values.get("pos.AMR"),
        Some(&Some(DynamicValue::Integer(14)))
    );
    assert_eq!(
        player.dynamic_values.get("pos.GK"),
        Some(&Some(DynamicValue::Integer(0)))
    );
    assert_eq!(player.dynamic_values.get("pos.SW"), Some(&None));
    assert_eq!(
        player.dynamic_values.get("attr.Acceleration"),
        Some(&Some(DynamicValue::Integer(16)))
    );
    assert_eq!(
        player.dynamic_values.get("hidden.Consistency"),
        Some(&Some(DynamicValue::Integer(12)))
    );
    assert_eq!(
        player.dynamic_values.get("personality.Ambition"),
        Some(&Some(DynamicValue::Integer(14)))
    );
    assert_eq!(
        player.dynamic_values.get("role.deep_lying_playmaker_ip"),
        Some(&Some(DynamicValue::Integer(82)))
    );
}

fn club_dna_score_rows(conn: &rusqlite::Connection) -> Vec<(i64, i64, i64, i64, Option<i64>)> {
    let mut statement = conn
        .prepare(
            "SELECT snapshot_id, uid, definition_version, score_model_version, score
             FROM club_dna_scores
             ORDER BY snapshot_id, uid, definition_version, score_model_version",
        )
        .expect("prepare Club DNA score rows");
    statement
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })
        .expect("read Club DNA score rows")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect Club DNA score rows")
}

fn seed_club_dna_squad() -> (tempfile::TempDir, rusqlite::Connection, i64) {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let snapshot_id = current_snapshot_id(&conn, save_id);
    conn.execute(
        "INSERT INTO club_dna_definitions (save_id, attribute_ids_json, definition_version)
         VALUES (?1, '[\"attr.Acceleration\"]', 2)",
        [save_id],
    )
    .expect("insert Club DNA definition");
    for uid in [81, 82, 99] {
        conn.execute(
            "INSERT INTO players (
                snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                nationalities_json, preferred_foot, positions_json, attributes_json,
                hidden_attributes_json, personality_json, current_club
             ) VALUES (?1, ?2, 100, 120, 'Extra', 2000, 1, '[]', 'right', '{}', '{}', '{}', '{}', ?3)",
            params![snapshot_id, uid, if uid == 99 { "Elsewhere FC" } else { "Loan FC" }],
        )
        .expect("insert Club DNA test player");
    }
    for (uid, definition_version, score_model_version, score) in [
        (77, 2, SCORE_MODEL_VERSION, Some(20)),
        (78, 2, SCORE_MODEL_VERSION, Some(80)),
        (79, 2, SCORE_MODEL_VERSION, None),
        (81, 2, SCORE_MODEL_VERSION, Some(20)),
        (80, 1, SCORE_MODEL_VERSION, Some(95)),
        (82, 2, SCORE_MODEL_VERSION + 1, Some(90)),
        (99, 2, SCORE_MODEL_VERSION, Some(100)),
    ] {
        conn.execute(
            "INSERT INTO club_dna_scores (
                snapshot_id, uid, definition_version, score_model_version, score
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                snapshot_id,
                uid,
                definition_version,
                score_model_version,
                score
            ],
        )
        .expect("insert Club DNA score");
    }
    conn.execute(
        "CREATE INDEX club_dna_squad_test_order
         ON players(snapshot_id, current_club, uid DESC)",
        [],
    )
    .expect("create non-UID scan order");
    (temp_dir, conn, save_id)
}

#[test]
fn sorts_club_dna_squad_ascending_with_exact_membership_and_read_only_pages() {
    let (_temp_dir, conn, save_id) = seed_club_dna_squad();
    conn.pragma_update(None, "reverse_unordered_selects", true)
        .expect("reverse unordered ties");
    let score_rows_before = club_dna_score_rows(&conn);
    let score_row_count_before = score_rows_before.len();
    let requested_fields = vec!["club_dna".to_string()];

    let page = list_squad_players(
        &conn,
        save_id,
        0,
        6,
        SquadSortField::parse("club_dna").expect("parse Club DNA sort"),
        SquadSortDir::Asc,
        &requested_fields,
    )
    .expect("sort Club DNA squad ascending");
    assert_eq!(page.total, 6);
    assert_eq!(
        page.players
            .iter()
            .map(|player| player.uid)
            .collect::<Vec<_>>(),
        [77, 81, 78, 79, 80, 82]
    );
    assert_eq!(
        page.players[0].dynamic_values.get("club_dna"),
        Some(&Some(DynamicValue::Integer(20)))
    );
    assert!(page.players[3..]
        .iter()
        .all(|player| player.dynamic_values.get("club_dna") == Some(&None)));

    let bounded_page = list_squad_players(
        &conn,
        save_id,
        1,
        2,
        SquadSortField::parse("club_dna").expect("parse Club DNA sort"),
        SquadSortDir::Asc,
        &requested_fields,
    )
    .expect("page Club DNA squad ascending");
    assert_eq!(bounded_page.total, 6);
    assert_eq!(
        bounded_page
            .players
            .iter()
            .map(|player| player.uid)
            .collect::<Vec<_>>(),
        [81, 78]
    );
    let unavailable_page = list_squad_players(
        &conn,
        save_id,
        3,
        3,
        SquadSortField::parse("club_dna").expect("parse Club DNA sort"),
        SquadSortDir::Asc,
        &requested_fields,
    )
    .expect("page unavailable Club DNA ties");
    assert_eq!(
        unavailable_page
            .players
            .iter()
            .map(|player| player.uid)
            .collect::<Vec<_>>(),
        [79, 80, 82]
    );
    assert_eq!(club_dna_score_rows(&conn).len(), score_row_count_before);
    assert_eq!(club_dna_score_rows(&conn), score_rows_before);
}

#[test]
fn sorts_club_dna_squad_descending_and_missing_definition_as_uid_stable_all_null() {
    let (_temp_dir, conn, save_id) = seed_club_dna_squad();
    conn.pragma_update(None, "reverse_unordered_selects", true)
        .expect("reverse unordered ties");
    let score_rows_before = club_dna_score_rows(&conn);

    let descending = list_squad_players(
        &conn,
        save_id,
        0,
        6,
        SquadSortField::parse("club_dna").expect("parse Club DNA sort"),
        SquadSortDir::Desc,
        &[],
    )
    .expect("sort Club DNA squad descending");
    assert_eq!(
        descending
            .players
            .iter()
            .map(|player| player.uid)
            .collect::<Vec<_>>(),
        [78, 77, 81, 79, 80, 82]
    );
    let bounded_page = list_squad_players(
        &conn,
        save_id,
        1,
        2,
        SquadSortField::parse("club_dna").expect("parse Club DNA sort"),
        SquadSortDir::Desc,
        &[],
    )
    .expect("page Club DNA descending ties");
    assert_eq!(
        bounded_page
            .players
            .iter()
            .map(|player| player.uid)
            .collect::<Vec<_>>(),
        [77, 81]
    );
    let unavailable_page = list_squad_players(
        &conn,
        save_id,
        3,
        3,
        SquadSortField::parse("club_dna").expect("parse Club DNA sort"),
        SquadSortDir::Desc,
        &[],
    )
    .expect("page unavailable Club DNA ties");
    assert_eq!(
        unavailable_page
            .players
            .iter()
            .map(|player| player.uid)
            .collect::<Vec<_>>(),
        [79, 80, 82]
    );

    conn.execute("DELETE FROM club_dna_definitions", [])
        .expect("remove Club DNA definition");
    let missing_definition = list_squad_players(
        &conn,
        save_id,
        0,
        6,
        SquadSortField::parse("club_dna").expect("parse Club DNA sort"),
        SquadSortDir::Desc,
        &["club_dna".to_string()],
    )
    .expect("sort squad without Club DNA definition");
    assert_eq!(missing_definition.total, 6);
    assert_eq!(
        missing_definition
            .players
            .iter()
            .map(|player| player.uid)
            .collect::<Vec<_>>(),
        [77, 78, 79, 80, 81, 82]
    );
    assert!(missing_definition
        .players
        .iter()
        .all(|player| player.dynamic_values.get("club_dna") == Some(&None)));
    assert_eq!(club_dna_score_rows(&conn), score_rows_before);
}

#[test]
fn potential_display_is_page_scoped_and_potential_sort_materializes_the_squad_cohort() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);
    let snapshot_id = current_snapshot_id(&conn, save_id);
    for (uid, score) in [(77, 8), (78, 10), (79, 12), (80, 16)] {
        conn.execute(
            "UPDATE players
             SET ca = 100,
                 pa = 100,
                 positions_json = ?1,
                 attributes_json = ?2
             WHERE snapshot_id = ?3 AND uid = ?4",
            params![
                json!({ "GK": 20 }).to_string(),
                json!({ "Positioning": score, "Concentration": score }).to_string(),
                snapshot_id,
                uid,
            ],
        )
        .expect("set potential source values");
    }
    let requested_fields = vec!["potential_role.line_holding_keeper_oop".to_string()];
    conn.execute("DELETE FROM player_potential_role_scores", [])
        .expect("clear eager rows for lazy display and sort test");

    let display_page = list_squad_players(
        &conn,
        save_id,
        0,
        1,
        SquadSortField::DEFAULT,
        SquadSortDir::DEFAULT,
        &requested_fields,
    )
    .expect("display potential page");
    assert_eq!(display_page.players.len(), 1);
    assert!(matches!(
        display_page.players[0]
            .dynamic_values
            .get("potential_role.line_holding_keeper_oop"),
        Some(Some(DynamicValue::Integer(_)))
    ));
    let display_cache_rows: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM player_potential_role_scores",
            [],
            |row| row.get(0),
        )
        .expect("count page cache rows");
    assert_eq!(display_cache_rows, 1);

    let visible_sort_role = "potential_role.line_holding_keeper_oop".to_string();
    let distinct_visible_role = "potential_role.sweeper_keeper_oop".to_string();
    let sorted_page = list_squad_players(
        &conn,
        save_id,
        0,
        1,
        SquadSortField::parse(&visible_sort_role).expect("parse potential sort"),
        SquadSortDir::Desc,
        &[visible_sort_role.clone(), distinct_visible_role.clone()],
    )
    .expect("sort squad potential");
    assert_eq!(
        sorted_page
            .players
            .iter()
            .map(|player| player.uid)
            .collect::<Vec<_>>(),
        [80]
    );
    let selected_role_rows: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM player_potential_role_scores
             WHERE role_id = 'line_holding_keeper_oop'",
            [],
            |row| row.get(0),
        )
        .expect("count globally sorted role rows");
    assert_eq!(selected_role_rows, 4);
    let distinct_visible_rows: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM player_potential_role_scores
             WHERE role_id = 'sweeper_keeper_oop'",
            [],
            |row| row.get(0),
        )
        .expect("count page-lazy distinct visible role rows");
    assert_eq!(distinct_visible_rows, 1);
}
