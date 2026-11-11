use super::squad::{get_expiring_contracts, get_prospects};
use super::test_support::{open_with_players, player, Fixture, HOME_CLUB, OTHER_CLUB};

fn set_contract_end(fixture: &Fixture, uid: i64, year: i64, day_of_year: i64) {
    fixture.set_player_column(uid, "contract_expiry_year", year);
    fixture.set_player_column(uid, "contract_expiry_day_of_year", day_of_year);
}

#[test]
fn lists_the_clubs_own_contracts_that_end_within_a_year_soonest_first() {
    let fixture = open_with_players(&[
        player(1, "Ends Next Summer", HOME_CLUB, "GK"),
        player(2, "Ends This Winter", HOME_CLUB, "GK"),
        player(3, "Ends Later", HOME_CLUB, "GK"),
        player(4, "Loanee", HOME_CLUB, "GK"),
        player(5, "Rival", OTHER_CLUB, "GK"),
        player(6, "No Contract Data", HOME_CLUB, "GK"),
    ]);
    // The golden snapshot's in-game date is 2026-08-14.
    set_contract_end(&fixture, 1, 2027, 181);
    set_contract_end(&fixture, 2, 2026, 365);
    set_contract_end(&fixture, 3, 2028, 182);
    set_contract_end(&fixture, 4, 2026, 365);
    fixture.set_player_column(4, "on_loan", 1);
    set_contract_end(&fixture, 5, 2026, 365);
    fixture.set_player_column(6, "contract_expiry_year", None::<i64>);

    let contracts = get_expiring_contracts(&fixture.conn, fixture.save_id)
        .expect("read contracts")
        .expect("contracts for a managed club");

    assert_eq!(contracts.window_months, Some(12));
    assert_eq!(contracts.total, 2);
    assert_eq!(
        contracts
            .players
            .iter()
            .map(|player| (
                player.uid,
                player.expiry_date.as_str(),
                player.days_remaining
            ))
            .collect::<Vec<_>>(),
        [(2, "2026-12-31", Some(139)), (1, "2027-06-30", Some(320))]
    );

    fixture
        .conn
        .execute("UPDATE snapshots SET game_date = NULL", [])
        .expect("clear game date");
    let undated = get_expiring_contracts(&fixture.conn, fixture.save_id)
        .expect("read contracts")
        .expect("contracts for a managed club");

    assert_eq!(undated.window_months, None);
    assert_eq!(
        undated
            .players
            .iter()
            .map(|player| (player.uid, player.days_remaining))
            .collect::<Vec<_>>(),
        [(2, None), (1, None), (3, None)]
    );
}

#[test]
fn ranks_young_players_by_potential_in_their_best_tactic_slot() {
    let fixture = open_with_players(&[
        player(1, "Young Left Back", HOME_CLUB, "DL"),
        player(2, "Young Keeper", HOME_CLUB, "GK"),
        player(3, "Older Keeper", HOME_CLUB, "GK"),
        player(4, "Finished Centre Back", HOME_CLUB, "DC"),
        player(5, "Rival Youngster", OTHER_CLUB, "DL"),
    ]);
    for (uid, age, lane_id, current, potential) in [
        (1, 19, "left_back", 50, 80),
        (2, 21, "goalkeeper", 60, 70),
        (3, 22, "goalkeeper", 40, 90),
        (4, 18, "left_centre_back", 70, 70),
        (5, 17, "left_back", 30, 95),
    ] {
        fixture.set_player_column(uid, "age", age);
        fixture.set_slot_score(uid, lane_id, current);
        fixture.set_potential_slot_score(uid, lane_id, potential);
    }

    let prospects = get_prospects(&fixture.conn, fixture.save_id)
        .expect("read prospects")
        .expect("prospects for a managed club");

    assert_eq!(
        prospects
            .iter()
            .map(|prospect| (
                prospect.uid,
                prospect.lane_id.as_str(),
                prospect.current_score,
                prospect.potential_score
            ))
            .collect::<Vec<_>>(),
        [(1, "left_back", 50, 80), (2, "goalkeeper", 60, 70)]
    );
}
