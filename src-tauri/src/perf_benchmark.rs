//! The private large-save benchmark behind `./scripts/dev perf-benchmark`.
//!
//! It builds a synthetic reference save through the production database, Load Data, and
//! Moneyball seams, times the table and suggestion requests through the functions their
//! commands call, and prints one path-free JSON report. `FM_VALUESCOUT_PERF_SCALE` selects
//! `250k` (the default) or `1m`. `FM_VALUESCOUT_PERF_LOAD=1` or `0` turns the timed Load Data
//! section on or off; it defaults to on at `250k` and off at `1m`, where a real load needs
//! streaming dump preparation. Without it, the `1m` query snapshot replicates the loaded
//! `250k` snapshot four times.

use std::collections::BTreeMap;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

use rusqlite::{params, Connection};
use serde_json::{json, Map, Value};

use crate::db::result_cache::TableRegistry;
use crate::db::{self, Db};
use crate::features::memory_read::service::DumpRequestResult;
use crate::features::moneyball::persistence::{
    prepare_snapshot_cohort, replace_snapshot_cohort, CohortRow,
};
use crate::features::moneyball::{
    MoneyballMetricValue, MoneyballStatistics, MONEYBALL_STATISTIC_KEYS,
};
use crate::features::player_metrics::resolver::{
    HIDDEN_ATTRIBUTE_KEYS, PERSONALITY_KEYS, POSITION_KEYS, TACTIC_LANE_IDS,
};
use crate::features::scoring::catalog::DUMP_ATTRIBUTE_KEYS;
use crate::features::search::commands::SearchCaches;
use crate::features::search::filter::{parse_filter_ast, FilterAst, FilterRule, FilterValue};
use crate::features::search::query::{
    search_general_players, search_moneyball_players, ComparisonPool, SearchPlayersRequest,
    SearchView, SortDir, SortField, DEFAULT_PAGE_LIMIT,
};
use crate::features::search::suggest::{suggest_players, SuggestionCache, DEFAULT_SUGGEST_LIMIT};
use crate::features::snapshot::{load_data, service};
use crate::features::staff;

/// The dump validator's staff attribute keys, which it keeps private.
const STAFF_ATTRIBUTE_KEYS: [&str; 24] = [
    "Attacking",
    "Defending",
    "Fitness",
    "Possession",
    "Technical",
    "Tactical",
    "SetPieces",
    "Determination",
    "ManManagement",
    "Motivating",
    "JudgingPlayerAbility",
    "JudgingPlayerPotential",
    "JudgingStaffAbility",
    "Negotiating",
    "TacticalKnowledge",
    "Physiotherapy",
    "SportsScience",
    "Authority",
    "Adaptability",
    "DataAnalysis",
    "WorkingWithYoungsters",
    "GoalkeepingDistribution",
    "GoalkeepingHandling",
    "GoalkeepingReflexes",
];
const NATIONS: [&str; 24] = [
    "ENG", "BRA", "ARG", "FRA", "ESP", "GER", "ITA", "NED", "POR", "BEL", "DEN", "SWE", "NOR",
    "USA", "MEX", "JPN", "KOR", "NGA", "GHA", "SEN", "CRO", "SRB", "POL", "TUR",
];
const FIRST_NAMES: [&str; 20] = [
    "John", "James", "Lucas", "Mateo", "Noah", "Liam", "Oliver", "Leo", "Hugo", "Luca", "Kenji",
    "Ahmed", "Ivan", "Pedro", "Joao", "Marco", "Emil", "Lars", "Tomas", "Diego",
];
const LAST_NAMES: [&str; 40] = [
    "Smith",
    "Jones",
    "Silva",
    "Santos",
    "Garcia",
    "Muller",
    "Rossi",
    "Jensen",
    "Nielsen",
    "Hansen",
    "Martin",
    "Bernard",
    "Dubois",
    "Novak",
    "Kowalski",
    "Petrovic",
    "Yilmaz",
    "Sato",
    "Kim",
    "Okafor",
    "Mensah",
    "Diallo",
    "Costa",
    "Pereira",
    "Fernandez",
    "Lopez",
    "Gonzalez",
    "Schmidt",
    "Schneider",
    "Fischer",
    "Weber",
    "Becker",
    "Larsen",
    "Andersen",
    "Johansson",
    "Karlsson",
    "Nilsson",
    "Eriksson",
    "Berg",
    "Haaland",
];

const FIRST_DUMP_DATE: &str = "2026-08-01";
const REPEAT_DUMP_DATE: &str = "2026-09-01";
const REPLICA_COPIES: i64 = 4;
const REPLICA_UID_OFFSET: i64 = 100_000_000;
const LATER_REQUESTS: usize = 5;
const LATER_WINDOW_OFFSET: usize = 5_000;

#[derive(Clone, Copy)]
struct Scale {
    label: &'static str,
    players: usize,
    staff: usize,
    cohort: usize,
}

const SCALE_250K: Scale = Scale {
    label: "250k",
    players: 250_000,
    staff: 50_000,
    cohort: 36_000,
};
const SCALE_1M: Scale = Scale {
    label: "1m",
    players: 1_000_000,
    staff: 200_000,
    cohort: 144_000,
};

/// A fixed-seed xorshift generator, so every run writes the same dumps.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn range(&mut self, low: i64, high: i64) -> i64 {
        low + (self.next() % (high - low + 1) as u64) as i64
    }

    fn pick<'a>(&mut self, values: &[&'a str]) -> &'a str {
        values[self.range(0, values.len() as i64 - 1) as usize]
    }

    fn chance(&mut self, percent: u64) -> bool {
        self.next() % 100 < percent
    }
}

/// Streams one schema-v9 dump record by record, so no dump is ever held in memory.
fn write_dump(path: &Path, scale: Scale, game_date: &str, seed: u64) -> std::io::Result<()> {
    let mut out = BufWriter::with_capacity(1 << 20, std::fs::File::create(path)?);
    let mut rng = Rng(seed);
    write!(
        out,
        r#"{{"schemaVersion":9,"generatedAtUtc":"{game_date}T10:00:00.000Z","gameVersion":"26.3.2.2329565","supportedGameVersion":"26.3","bridgeVersion":"0.1.0","protocolVersion":1,"gameDate":"{game_date}","gameDateSource":"derived","gameDateBasis":"next-fixture-consensus","playerDatabaseScope":"men","scanTruncated":false,"maxAccepted":null,"playerCount":{},"staffCount":{},"manager":null,"players":["#,
        scale.players, scale.staff
    )?;
    for index in 0..scale.players {
        if index > 0 {
            out.write_all(b",")?;
        }
        write_player(&mut out, &mut rng, 1_000 + index as i64)?;
    }
    out.write_all(br#"],"staff":["#)?;
    for index in 0..scale.staff {
        if index > 0 {
            out.write_all(b",")?;
        }
        write_staff(&mut out, &mut rng, 50_000_000 + index as i64)?;
    }
    out.write_all(b"]}")?;
    out.flush()
}

fn write_player(out: &mut impl Write, rng: &mut Rng, uid: i64) -> std::io::Result<()> {
    let age = rng.range(15, 38);
    let ca = rng.range(30, 190);
    let pa = (ca + rng.range(0, 60)).min(200);
    let club = rng.range(1, 5_000);
    let attribute_base = ca * 18 / 200 + 1;
    let natural_position = rng.range(0, POSITION_KEYS.len() as i64 - 1) as usize;
    write!(
        out,
        r#"{{"uid":{uid},"ca":{ca},"pa":{pa},"name":"{} {} {uid}","birthYear":{},"birthDayOfYear":{},"age":{age},"nationalities":["{}"],"nationUid":{},"gender":"male","heightCm":{},"preferredFoot":"{}","positions":{{"#,
        rng.pick(&FIRST_NAMES),
        rng.pick(&LAST_NAMES),
        2026 - age,
        rng.range(1, 365),
        rng.pick(&NATIONS),
        rng.range(1, 200),
        rng.range(160, 200),
        rng.pick(&["right", "left", "either"]),
    )?;
    for (index, key) in POSITION_KEYS.iter().enumerate() {
        let rating = if index == natural_position {
            20
        } else if rng.chance(15) {
            rng.range(10, 19)
        } else {
            rng.range(1, 6)
        };
        write!(out, r#"{}"{key}":{rating}"#, separator(index))?;
    }
    out.write_all(br#"},"attributes":{"#)?;
    for (index, key) in DUMP_ATTRIBUTE_KEYS.iter().enumerate() {
        let value = (attribute_base + rng.range(-4, 4)).clamp(1, 20);
        write!(out, r#"{}"{key}":{value}"#, separator(index))?;
    }
    out.write_all(br#"},"hiddenAttributes":{"#)?;
    for (index, key) in HIDDEN_ATTRIBUTE_KEYS.iter().enumerate() {
        write!(out, r#"{}"{key}":{}"#, separator(index), rng.range(1, 20))?;
    }
    out.write_all(br#"},"personality":{"#)?;
    for (index, key) in PERSONALITY_KEYS.iter().enumerate() {
        write!(out, r#"{}"{key}":{}"#, separator(index), rng.range(1, 20))?;
    }
    write!(
        out,
        r#"}},"weeklyWageGbp":{},"contractExpiryYear":{},"contractExpiryDayOfYear":{},"transferListed":false,"loanListed":false,"notForSale":false,"setForRelease":false,"marketValueGbp":{},"reputation":{{"current":{},"world":{}}},"currentClub":"Club {club}","currentClubUid":{club},"parentClub":null,"parentClubUid":null,"onLoan":false,"division":"Division {}","teamLevel":"senior","clubReputation":{},"teamType":0}}"#,
        rng.range(100, 300_000),
        rng.range(2026, 2031),
        rng.range(1, 365),
        rng.range(0, 150_000_000),
        rng.range(1, 10_000),
        rng.range(1, 10_000),
        club % 300,
        rng.range(100, 10_000),
    )
}

fn write_staff(out: &mut impl Write, rng: &mut Rng, uid: i64) -> std::io::Result<()> {
    let club = rng.range(1, 5_000);
    write!(
        out,
        r#"{{"uid":{uid},"name":"Staff {uid}","birthYear":1980,"birthDayOfYear":100,"age":46,"nationalities":["{}"],"nationUid":1,"gender":"male","ca":{},"pa":{},"attributes":{{"#,
        rng.pick(&NATIONS),
        rng.range(40, 120),
        rng.range(120, 180),
    )?;
    for (index, key) in STAFF_ATTRIBUTE_KEYS.iter().enumerate() {
        write!(out, r#"{}"{key}":{}"#, separator(index), rng.range(1, 20))?;
    }
    write!(
        out,
        r#"}},"jobId":16,"weeklyWageGbp":null,"contractExpiryYear":null,"contractExpiryDayOfYear":null,"club":"Club {club}","division":"Division {}","clubUid":{club}}}"#,
        club % 300,
    )
}

fn separator(index: usize) -> &'static str {
    if index == 0 {
        ""
    } else {
        ","
    }
}

/// Resets the process's peak resident set size, so the next reading covers one phase only.
fn reset_peak_memory() {
    if cfg!(target_os = "linux") {
        std::fs::write("/proc/self/clear_refs", "5").expect("reset peak memory");
    }
}

/// The peak resident set size since the last reset, in MiB, or `None` off Linux.
fn peak_memory_mib() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let kib: u64 = status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    Some(kib / 1024)
}

/// Runs one Load Data publication the way `execute_load_data_with` does, timing and
/// measuring the peak memory of each phase.
fn load_dump(db: &Db, dump: &Path, request_id: &str) -> Value {
    let started = Instant::now();
    let save_context = {
        let conn = db.0.lock().expect("database lock");
        service::capture_active_save_context(&conn).expect("active save context")
    };

    reset_peak_memory();
    let phase_started = Instant::now();
    let raw = load_data::prepare_raw_for_publish(dump).expect("prepare dump");
    let prepare_ms = phase_started.elapsed().as_millis() as u64;
    let prepare_peak = peak_memory_mib();

    reset_peak_memory();
    let phase_started = Instant::now();
    let prepared = load_data::score_raw_for_publish(raw).expect("score dump");
    let scoring_ms = phase_started.elapsed().as_millis() as u64;
    let scoring_peak = peak_memory_mib();

    let dump_result = DumpRequestResult {
        request_id: request_id.to_string(),
        state: "ready".to_string(),
        players_found: None,
        dump_present: true,
        error: None,
    };
    let mut conn = db.0.lock().expect("database lock");
    let clock = Instant::now();
    let mut now_ms = move || clock.elapsed().as_millis() as u64;
    let mut save_peak = None;
    let mut on_save_boundary = || {
        save_peak = peak_memory_mib();
        reset_peak_memory();
    };
    reset_peak_memory();
    let (_result, save_ms, finalize_ms) = load_data::publish_prepared_with_progress(
        &mut conn,
        &save_context,
        prepared,
        &dump_result,
        &mut now_ms,
        &mut on_save_boundary,
    )
    .expect("publish snapshot");
    let finalize_peak = peak_memory_mib();

    json!({
        "prepare_ms": prepare_ms,
        "scoring_ms": scoring_ms,
        "save_ms": save_ms,
        "finalize_ms": finalize_ms,
        "total_ms": started.elapsed().as_millis() as u64,
        "prepare_peak_mib": prepare_peak,
        "scoring_peak_mib": scoring_peak,
        "save_peak_mib": save_peak,
        "finalize_peak_mib": finalize_peak,
    })
}

fn current_snapshot(conn: &Connection) -> (i64, String) {
    conn.query_row(
        "SELECT s.id, s.game_date FROM snapshots s
         JOIN saves sv ON sv.id = s.save_id AND sv.is_active = 1
         WHERE s.is_current = 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .expect("current snapshot")
}

/// Seeds the Moneyball cohort through the production preparation and replacement. It times
/// the preparation, which ranks percentiles and serializes every row before any transaction,
/// apart from the write transaction, which spans begin to commit and only deletes, inserts,
/// and sets the as-of date.
fn seed_moneyball_cohort(conn: &mut Connection, cohort: usize) -> Value {
    let (snapshot_id, game_date) = current_snapshot(conn);
    let player_uids: Vec<i64> = conn
        .prepare("SELECT uid FROM players WHERE snapshot_id = ?1 ORDER BY uid LIMIT ?2")
        .expect("prepare cohort players")
        .query_map(params![snapshot_id, cohort as i64], |row| row.get(0))
        .expect("query cohort players")
        .collect::<Result<_, _>>()
        .expect("read cohort players");
    let mut rng = Rng(99);
    let rows: Vec<CohortRow> = player_uids
        .iter()
        .map(|&player_uid| {
            let statistics: MoneyballStatistics = MONEYBALL_STATISTIC_KEYS
                .iter()
                .enumerate()
                .map(|(index, key)| {
                    let value = if rng.chance(5) {
                        None
                    } else if index % 3 == 0 {
                        Some(MoneyballMetricValue::Count(rng.range(0, 40) as u32))
                    } else {
                        Some(MoneyballMetricValue::Decimal(
                            rng.range(0, 10_000) as f64 / 1000.0,
                        ))
                    };
                    (key.to_string(), value)
                })
                .collect::<BTreeMap<_, _>>();
            CohortRow {
                player_uid,
                starts: rng.range(0, 40),
                substitute_appearances: rng.range(0, 10),
                minutes: rng.range(0, 3_600),
                statistics,
            }
        })
        .collect();

    let started = Instant::now();
    let prepared = prepare_snapshot_cohort(&rows).expect("prepare cohort");
    let prepare_ms = started.elapsed().as_millis() as u64;
    let started = Instant::now();
    let tx = conn.transaction().expect("begin cohort write");
    replace_snapshot_cohort(&tx, snapshot_id, &prepared, &game_date).expect("replace cohort");
    tx.commit().expect("commit cohort write");
    json!({
        "cohort": rows.len(),
        "prepare_ms": prepare_ms,
        "write_transaction_ms": started.elapsed().as_millis() as u64,
    })
}

fn columns_except(conn: &Connection, table: &str, excluded: &[&str]) -> Vec<String> {
    conn.prepare(&format!("SELECT name FROM pragma_table_info('{table}')"))
        .expect("prepare table info")
        .query_map([], |row| row.get::<_, String>(0))
        .expect("query table info")
        .collect::<Result<Vec<_>, _>>()
        .expect("read table info")
        .into_iter()
        .filter(|column| !excluded.contains(&column.as_str()))
        .collect()
}

/// Copies one snapshot's rows of `table` into another, offsetting the UID column.
fn copy_rows(
    tx: &Connection,
    table: &str,
    uid_column: &str,
    from: i64,
    to: i64,
    uid_offset: i64,
) -> rusqlite::Result<usize> {
    let columns = columns_except(tx, table, &["snapshot_id", uid_column]).join(", ");
    tx.execute(
        &format!(
            "INSERT INTO {table} (snapshot_id, {uid_column}, {columns})
             SELECT ?1, {uid_column} + ?2, {columns} FROM {table}
             WHERE snapshot_id = ?3 ORDER BY rowid"
        ),
        params![to, uid_offset, from],
    )
}

/// Builds a current snapshot four times the size of the current one in one writer
/// transaction, copying its players, staff, compact metric rows, and Moneyball cohort with
/// UIDs offset per copy. The displaced snapshot keeps raw facts only, as after a load.
fn replicate_current_snapshot(conn: &mut Connection) -> Value {
    let started = Instant::now();
    let (source, _) = current_snapshot(conn);
    let tx = conn.transaction().expect("begin replication");
    let snapshot_columns = columns_except(&tx, "snapshots", &["id", "is_current", "context_token"]);
    let snapshot_values = snapshot_columns
        .iter()
        .map(|column| match column.as_str() {
            "player_count" | "staff_count" => format!("{column} * {REPLICA_COPIES}"),
            _ => column.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ");
    tx.execute(
        &format!(
            "INSERT INTO snapshots ({}, is_current) SELECT {snapshot_values}, 0
             FROM snapshots WHERE id = ?1",
            snapshot_columns.join(", ")
        ),
        [source],
    )
    .expect("insert replica snapshot");
    let replica = tx.last_insert_rowid();
    for copy in 0..REPLICA_COPIES {
        let offset = copy * REPLICA_UID_OFFSET;
        for (table, uid_column) in [
            ("players", "uid"),
            ("player_role_metrics", "uid"),
            ("player_moneyball_stats", "player_uid"),
            ("staff", "uid"),
            ("staff_role_metrics", "uid"),
        ] {
            copy_rows(&tx, table, uid_column, source, replica, offset).expect("copy rows");
        }
    }
    tx.execute_batch(&format!(
        "UPDATE snapshots SET is_current = 0 WHERE id = {source};
         UPDATE snapshots SET is_current = 1 WHERE id = {replica};
         DELETE FROM player_role_metrics WHERE snapshot_id = {source};
         DELETE FROM staff_role_metrics WHERE snapshot_id = {source};
         DELETE FROM player_moneyball_stats WHERE snapshot_id = {source};
         UPDATE snapshots SET moneyball_stats_game_date = NULL WHERE id = {source};
         UPDATE players SET potential_attributes_json = NULL,
                            potential_projection_model_version = NULL
          WHERE snapshot_id = {source};"
    ))
    .expect("promote replica snapshot");
    tx.commit().expect("commit replication");
    json!({ "copies": REPLICA_COPIES, "ms": started.elapsed().as_millis() as u64 })
}

fn elapsed_ms(started: Instant) -> f64 {
    (started.elapsed().as_secs_f64() * 10_000.0).round() / 10.0
}

/// Times the first request and the median of five later ones.
fn time_repeated(mut request: impl FnMut()) -> Value {
    let started = Instant::now();
    request();
    let first_ms = elapsed_ms(started);
    let mut later: Vec<f64> = (0..LATER_REQUESTS)
        .map(|_| {
            let started = Instant::now();
            request();
            elapsed_ms(started)
        })
        .collect();
    later.sort_by(f64::total_cmp);
    json!({ "first_ms": first_ms, "median_ms": later[LATER_REQUESTS / 2] })
}

/// A page at `LATER_WINDOW_OFFSET`, or halfway for a deep window, aligned to whole pages.
fn later_window_offset(total: usize, deep: bool) -> usize {
    let offset = if deep {
        total / 2
    } else {
        LATER_WINDOW_OFFSET.min(total / 2)
    };
    offset - offset % DEFAULT_PAGE_LIMIT
}

#[derive(Clone)]
struct PlayerShape {
    name: &'static str,
    view: SearchView,
    pool: ComparisonPool,
    sort: SortField,
    fields: Vec<String>,
    filters: Option<FilterAst>,
    deep_window: bool,
    /// Fields of an already computed query whose order the first page reuses.
    warm_base_fields: Option<Vec<String>>,
}

impl PlayerShape {
    fn general(name: &'static str, sort: &str, fields: Vec<String>) -> Self {
        Self {
            name,
            view: SearchView::General,
            pool: ComparisonPool::FullCsv,
            sort: SortField::parse(sort).expect("general sort"),
            fields,
            filters: None,
            deep_window: false,
            warm_base_fields: None,
        }
    }

    fn moneyball(name: &'static str, pool: ComparisonPool, fields: Vec<String>) -> Self {
        Self {
            name,
            view: SearchView::Moneyball,
            pool,
            sort: SortField::parse_for_moneyball("moneyball.average_rating", true)
                .expect("moneyball sort"),
            fields,
            filters: None,
            deep_window: false,
            warm_base_fields: None,
        }
    }
}

fn fields(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

fn player_shapes() -> Vec<PlayerShape> {
    let default_fields = fields(&["height"]);
    let role_fields = fields(&[
        "height",
        "position",
        "role.centre_back_ip",
        "role.goalkeeper_ip",
        "potential_role.centre_back_ip",
    ]);
    let tactic_fields: Vec<String> = TACTIC_LANE_IDS
        .iter()
        .map(|lane| format!("tactic_current.{lane}"))
        .collect();
    let moneyball_role_fields = fields(&[
        "moneyball.goals",
        "moneyball.xg",
        "moneyball.average_rating",
        "moneyball_role.amc_attacking_midfielder_ip",
    ]);
    let attribute_age_filter = parse_filter_ast(
        vec![
            FilterRule {
                field: "attr.Finishing".to_string(),
                op: "gte".to_string(),
                value: FilterValue::Integer(15),
            },
            FilterRule {
                field: "age".to_string(),
                op: "lte".to_string(),
                value: FilterValue::Integer(23),
            },
        ],
        Some("and"),
    )
    .expect("attribute and age filter");

    vec![
        PlayerShape::general("general_ca", "ca", default_fields.clone()),
        PlayerShape {
            warm_base_fields: Some(default_fields.clone()),
            ..PlayerShape::general(
                "general_added_column",
                "ca",
                [default_fields.clone(), fields(&["attr.Passing"])].concat(),
            )
        },
        PlayerShape::general(
            "general_role_position_tactic_columns",
            "ca",
            [role_fields.clone(), tactic_fields.clone()].concat(),
        ),
        PlayerShape::general(
            "general_current_role_sort",
            "role.centre_back_ip",
            role_fields.clone(),
        ),
        PlayerShape::general(
            "general_potential_role_sort",
            "potential_role.centre_back_ip",
            role_fields.clone(),
        ),
        PlayerShape::general(
            "general_attribute_sort",
            "attr.Finishing",
            fields(&["height", "attr.Finishing"]),
        ),
        PlayerShape::general(
            "general_tactic_sort",
            "tactic_current.centre_forward",
            tactic_fields,
        ),
        PlayerShape {
            filters: Some(attribute_age_filter),
            ..PlayerShape::general(
                "general_attribute_age_filter",
                "pa",
                fields(&["height", "attr.Finishing"]),
            )
        },
        PlayerShape {
            deep_window: true,
            ..PlayerShape::general("general_deep_window", "role.centre_back_ip", role_fields)
        },
        PlayerShape::moneyball(
            "moneyball_role_filtered_pool",
            ComparisonPool::Filtered,
            moneyball_role_fields.clone(),
        ),
        PlayerShape::moneyball(
            "moneyball_role_all_players_pool",
            ComparisonPool::FullCsv,
            moneyball_role_fields,
        ),
    ]
}

/// Requests one page as `search_players` does, through the computed General orders or
/// Moneyball cohorts in `caches`.
fn search_player_page(db: &Db, caches: &SearchCaches, shape: &PlayerShape, offset: usize) -> usize {
    let request = SearchPlayersRequest {
        offset,
        limit: DEFAULT_PAGE_LIMIT,
        sort_by: shape.sort.clone(),
        sort_dir: SortDir::Desc,
        filter_ast: shape.filters.as_ref(),
        requested_fields: &shape.fields,
        view: shape.view,
        comparison_pool: shape.pool,
        shortlist_only: false,
    };
    match shape.view {
        SearchView::General => search_general_players(
            db,
            &caches.general_orders,
            &TableRegistry::default(),
            None,
            request,
        ),
        SearchView::Moneyball => search_moneyball_players(
            db,
            &caches.moneyball_cohorts,
            &TableRegistry::default(),
            None,
            request,
        ),
    }
    .unwrap_or_else(|error| panic!("{} request failed: {error}", shape.name))
    .total as usize
}

fn measure_player_shape(db: &Db, shape: &PlayerShape) -> Value {
    let first_page = match &shape.warm_base_fields {
        // A column change keeps the query's fingerprint, so the app reuses its computed order.
        Some(base_fields) => {
            let caches = SearchCaches::default();
            let base = PlayerShape {
                fields: base_fields.clone(),
                ..shape.clone()
            };
            search_player_page(db, &caches, &base, 0);
            time_repeated(|| {
                search_player_page(db, &caches, shape, 0);
            })
        }
        // Every other first page computes its query afresh, as after a sort or filter change.
        None => time_repeated(|| {
            search_player_page(db, &SearchCaches::default(), shape, 0);
        }),
    };
    // Later windows reuse the query computed by its first page.
    let caches = SearchCaches::default();
    let total = search_player_page(db, &caches, shape, 0);
    let offset = later_window_offset(total, shape.deep_window);
    let later_window = time_repeated(|| {
        search_player_page(db, &caches, shape, offset);
    });
    json!({
        "total": total,
        "first_page": first_page,
        "later_window": later_window,
        "later_window_offset": offset,
    })
}

/// Requests one Staff Search page as `search_staff` does, through its computed orders in
/// `orders`.
fn search_staff_page(db: &Db, orders: &staff::query::StaffOrderCache, offset: usize) -> usize {
    staff::query::search_staff_page(
        db,
        orders,
        &TableRegistry::default(),
        None,
        staff::query::StaffSearchRequest {
            offset,
            limit: DEFAULT_PAGE_LIMIT,
            sort: staff::query::SortField::parse("role.coach_fitness").expect("staff sort"),
            direction: staff::query::SortDir::Desc,
            filters: None,
            shortlist_only: false,
            preferred_job: None,
            unemployed_only: false,
            requested_fields: &fields(&["role.coach_fitness"]),
        },
    )
    .expect("staff request")
    .total as usize
}

/// Runs every request on a read connection, as the commands do.
fn measure_requests(db: &Db) -> Map<String, Value> {
    let mut requests = Map::new();
    for shape in player_shapes() {
        requests.insert(shape.name.to_string(), measure_player_shape(db, &shape));
    }

    // Every first page computes its query afresh; later windows reuse the first page's.
    let first_page = time_repeated(|| {
        search_staff_page(db, &staff::query::StaffOrderCache::new(4), 0);
    });
    let orders = staff::query::StaffOrderCache::new(4);
    let total = search_staff_page(db, &orders, 0);
    let offset = later_window_offset(total, false);
    let later_window = time_repeated(|| {
        search_staff_page(db, &orders, offset);
    });
    requests.insert(
        "staff_role_sort".to_string(),
        json!({
            "total": total,
            "first_page": first_page,
            "later_window": later_window,
            "later_window_offset": offset,
        }),
    );

    // The first request builds the name list; later ones reuse it, as the app does.
    let suggestions = SuggestionCache::new(1);
    requests.insert(
        "suggest_three_letters".to_string(),
        time_repeated(|| {
            suggest_players(db, &suggestions, "smi", DEFAULT_SUGGEST_LIMIT)
                .expect("suggest request");
        }),
    );
    requests
}

fn scale_from_env() -> Scale {
    match std::env::var("FM_VALUESCOUT_PERF_SCALE").as_deref() {
        Err(_) | Ok("250k") => SCALE_250K,
        Ok("1m") => SCALE_1M,
        Ok(other) => panic!("FM_VALUESCOUT_PERF_SCALE must be 250k or 1m, not {other}"),
    }
}

fn load_enabled(scale: Scale) -> bool {
    match std::env::var("FM_VALUESCOUT_PERF_LOAD").as_deref() {
        Err(_) => scale.label == SCALE_250K.label,
        Ok("1") => true,
        Ok("0") => false,
        Ok(other) => panic!("FM_VALUESCOUT_PERF_LOAD must be 1 or 0, not {other}"),
    }
}

fn execution_context() -> &'static str {
    if cfg!(target_os = "windows") {
        "native-windows"
    } else if std::env::var_os("WSL_INTEROP").is_some()
        || std::fs::read_to_string("/proc/version").is_ok_and(|v| v.contains("Microsoft"))
    {
        "wsl"
    } else {
        "native-unix"
    }
}

#[test]
#[ignore]
fn perf_benchmark_harness() {
    let scale = scale_from_env();
    let load = load_enabled(scale);
    // Under the build directory rather than the system temp directory, which may be RAM-backed.
    let work_dir = tempfile::Builder::new()
        .prefix("perf-benchmark-")
        .tempdir_in(concat!(env!("CARGO_MANIFEST_DIR"), "/target"))
        .expect("benchmark directory");
    let db = db::open(&work_dir.path().join(db::APP_DB_FILE)).expect("open database");

    // Without a timed load, the query snapshot comes from one untimed 250k load.
    let dump_scale = if load { scale } else { SCALE_250K };
    let first_dump = work_dir.path().join("first.json");
    write_dump(&first_dump, dump_scale, FIRST_DUMP_DATE, 7).expect("write first dump");
    let first_load = load_dump(&db, &first_dump, "perf-benchmark-first");
    std::fs::remove_file(&first_dump).expect("remove first dump");
    let load_data = if load {
        let repeat_dump = work_dir.path().join("repeat.json");
        write_dump(&repeat_dump, scale, REPEAT_DUMP_DATE, 11).expect("write repeat dump");
        let repeat_load = load_dump(&db, &repeat_dump, "perf-benchmark-repeat");
        std::fs::remove_file(&repeat_dump).expect("remove repeat dump");
        json!({ "first": first_load, "repeat": repeat_load })
    } else {
        Value::Null
    };

    let (moneyball_write, replication) = {
        let mut conn = db.0.lock().expect("database lock");
        let moneyball_write = seed_moneyball_cohort(&mut conn, dump_scale.cohort);
        let replication = if dump_scale.players == scale.players {
            Value::Null
        } else {
            replicate_current_snapshot(&mut conn)
        };
        (moneyball_write, replication)
    };
    let requests = measure_requests(&db);

    let report = json!({
        "test": "perf_benchmark_harness",
        "scale": scale.label,
        "players": scale.players,
        "staff": scale.staff,
        "moneyball_cohort": scale.cohort,
        "host": {
            "os": std::env::consts::OS,
            "architecture": std::env::consts::ARCH,
            "execution_context": execution_context(),
            "build_profile": if cfg!(debug_assertions) { "debug" } else { "release" },
        },
        "load_data": load_data,
        "moneyball_write": moneyball_write,
        "replication": replication,
        "requests": requests,
    });
    let json = serde_json::to_string(&report).expect("benchmark report serializes");
    assert!(!json.contains(work_dir.path().to_string_lossy().as_ref()));
    println!("{json}");
}
