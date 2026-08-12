use rusqlite::params;

use super::squad::{list_squad_players, SquadSortDir, SquadSortField, DEFAULT_SQUAD_PAGE_LIMIT};
use super::test_support::{add_picker_candidates, current_snapshot_id, open_with_snapshot};

#[test]
fn lists_the_distinct_current_club_family_union() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    add_picker_candidates(&temp_dir, &mut conn, save_id);

    let page = list_squad_players(
        &conn,
        save_id,
        0,
        DEFAULT_SQUAD_PAGE_LIMIT,
        SquadSortField::DEFAULT,
        SquadSortDir::DEFAULT,
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
            vec![79, 80, 77, 78],
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
        let page =
            list_squad_players(&conn, save_id, 0, 4, field, direction).expect("sort squad players");
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
fn returns_no_players_without_a_configuration_or_matching_current_players() {
    let (temp_dir, mut conn, save_id) = open_with_snapshot();
    conn.execute(
        "DELETE FROM planner_club_sources WHERE save_id = ?1",
        params![save_id],
    )
    .expect("remove sources");
    conn.execute(
        "DELETE FROM planner_club_settings WHERE save_id = ?1",
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
    .expect("move current players outside club family");
    let empty_current_result = list_squad_players(
        &conn,
        save_id,
        0,
        DEFAULT_SQUAD_PAGE_LIMIT,
        SquadSortField::DEFAULT,
        SquadSortDir::DEFAULT,
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
