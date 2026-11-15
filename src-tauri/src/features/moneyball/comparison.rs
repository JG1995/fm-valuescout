//! Moneyball comparison groups: whom a player's season statistics are ranked against.
//!
//! Every percentile ranks a statistic among one group of peers: the players with at least
//! `MIN_SEASON_MINUTES` minutes this season rated at least `PEER_FAMILIARITY` at a position of
//! the group's position families. A role ranks against its own family; a tactic slot against
//! the families of its two positions together; a player on his own against the family of his
//! best position. A player outside a group, such as one with a short sample, is still ranked
//! against it without joining it.
//!
//! The groups are built once per snapshot and data generation and kept in
//! `ComparisonGroupsCache`. Each statistic keeps 100 cut values instead of its population, so a
//! lookup is a binary search over 100 numbers and the groups take about a megabyte.

use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex, OnceLock, PoisonError};

use rusqlite::{Connection, Row, Rows};

use super::catalog::is_lower_is_better_statistic_key;
use super::json_scan::{parse_json_value, scan_json_object};
use super::MONEYBALL_STATISTIC_KEYS;
use crate::db::result_cache::ResultCache;

/// The fewest competitive minutes this season that make a player one of a group's peers.
pub(crate) const MIN_SEASON_MINUTES: i64 = 450;

/// The lowest familiarity at a family's position that makes a player one of its peers.
pub(crate) const PEER_FAMILIARITY: i64 = 16;

/// The position families the Moneyball role catalog defines its roles for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum PositionFamily {
    Goalkeeper,
    CentralDefender,
    FullBack,
    WingBack,
    DefensiveMidfielder,
    CentralMidfielder,
    WideMidfielder,
    AttackingMidfielder,
    Winger,
    Striker,
}

impl PositionFamily {
    /// Every family, from the goal outwards. A player rated equally at two families' positions
    /// belongs, on his own, to the earlier one.
    pub(crate) const ALL: [Self; 10] = [
        Self::Goalkeeper,
        Self::CentralDefender,
        Self::FullBack,
        Self::WingBack,
        Self::DefensiveMidfielder,
        Self::CentralMidfielder,
        Self::WideMidfielder,
        Self::AttackingMidfielder,
        Self::Winger,
        Self::Striker,
    ];

    /// The role catalog's family ID.
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::Goalkeeper => "goalkeeper",
            Self::CentralDefender => "central_defender",
            Self::FullBack => "full_back",
            Self::WingBack => "wing_back",
            Self::DefensiveMidfielder => "defensive_midfielder",
            Self::CentralMidfielder => "central_midfielder",
            Self::WideMidfielder => "wide_midfielder",
            Self::AttackingMidfielder => "attacking_midfielder",
            Self::Winger => "winger",
            Self::Striker => "striker",
        }
    }

    pub(crate) fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|family| family.id() == id)
    }

    /// The base positions, as keys of a player's familiarity map, that make up the family.
    pub(crate) fn positions(self) -> &'static [&'static str] {
        match self {
            Self::Goalkeeper => &["GK"],
            Self::CentralDefender => &["DC"],
            Self::FullBack => &["DL", "DR"],
            Self::WingBack => &["WBL", "WBR"],
            Self::DefensiveMidfielder => &["DM"],
            Self::CentralMidfielder => &["MC"],
            Self::WideMidfielder => &["ML", "MR"],
            Self::AttackingMidfielder => &["AMC"],
            Self::Winger => &["AML", "AMR"],
            Self::Striker => &["ST"],
        }
    }

    /// The family of a base position (`DC`, not `DCR`). The sweeper has none.
    pub(crate) fn of_position(base_position: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|family| family.positions().contains(&base_position))
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// The peers a statistic is ranked among: one family's, or a slot's two families' together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ComparisonGroup(u16);

impl ComparisonGroup {
    pub(crate) fn family(family: PositionFamily) -> Self {
        Self(1 << family.index())
    }

    /// A tactic slot's group, from the base positions of its two phases: one family when both
    /// belong to it, else both families' peers together. `None` when a position has no family.
    pub(crate) fn slot(ip_base_position: &str, oop_base_position: &str) -> Option<Self> {
        let ip = PositionFamily::of_position(ip_base_position)?;
        let oop = PositionFamily::of_position(oop_base_position)?;
        Some(Self(Self::family(ip).0 | Self::family(oop).0))
    }

    /// The group's families, in `PositionFamily::ALL` order.
    pub(crate) fn families(self) -> impl Iterator<Item = PositionFamily> {
        PositionFamily::ALL
            .into_iter()
            .filter(move |family| self.0 & Self::family(*family).0 != 0)
    }

    fn single_family(self) -> Option<PositionFamily> {
        (self.0.count_ones() == 1).then(|| PositionFamily::ALL[self.0.trailing_zeros() as usize])
    }
}

/// The index of a statistic key in `MONEYBALL_STATISTIC_KEYS`.
pub(crate) fn statistic_index(key: &str) -> Option<usize> {
    static INDEXES: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();
    INDEXES
        .get_or_init(|| {
            MONEYBALL_STATISTIC_KEYS
                .iter()
                .enumerate()
                .map(|(index, key)| (*key, index))
                .collect()
        })
        .get(key)
        .copied()
}

fn lower_is_better(index: usize) -> bool {
    static LOWER_IS_BETTER: OnceLock<Vec<bool>> = OnceLock::new();
    LOWER_IS_BETTER.get_or_init(|| {
        MONEYBALL_STATISTIC_KEYS
            .iter()
            .map(|key| is_lower_is_better_statistic_key(key))
            .collect()
    })[index]
}

/// The number of cut values a varied population keeps: one per percentile above zero.
const CUTS: usize = 100;

/// One statistic's ranking within one group. A value scores the share of the population below
/// it, `below / (n - 1)` as a rounded percentage, and 50 when every peer shares one value; a
/// lower-is-better statistic scores 100 minus that. The ranking gives that score for any value,
/// in the population or not, without keeping the population.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Ranking {
    /// No peer has the statistic, so nobody is ranked.
    Empty,
    /// One peer, or peers that all share one value: everyone scores 50.
    Uniform,
    /// `cuts[s - 1]` is the largest population value that a value must exceed to score at
    /// least `s`, before lower-is-better inversion.
    Varied(Box<[f64]>),
}

impl Ranking {
    /// The ranking of `population`, finite values in ascending order.
    pub(crate) fn from_sorted(population: &[f64]) -> Self {
        let n = population.len();
        if n == 0 {
            return Self::Empty;
        }
        if n == 1 || population[0] == population[n - 1] {
            return Self::Uniform;
        }
        // A value above `below` of the population scores round(below / (n - 1) * 100), so it
        // scores at least `s` exactly when it exceeds the `k`th lowest value, where `k` is the
        // fewest values below that reach `s`.
        let score = |below: usize| {
            ((below as f64 / (n - 1) as f64) * 100.0)
                .round()
                .clamp(0.0, 100.0) as usize
        };
        let mut below = 0;
        let cuts = (1..=CUTS)
            .map(|minimum| {
                while score(below) < minimum {
                    below += 1;
                }
                population[below - 1]
            })
            .collect();
        Self::Varied(cuts)
    }

    /// The percentile of a finite `value`, inverted when lower is better.
    fn percentile(&self, value: f64, lower_is_better: bool) -> Option<u8> {
        let score = match self {
            Self::Empty => return None,
            Self::Uniform => return Some(50),
            Self::Varied(cuts) => cuts.partition_point(|cut| *cut < value) as u8,
        };
        Some(if lower_is_better { 100 - score } else { score })
    }
}

/// Every statistic's ranking within one group.
#[derive(Debug)]
pub(crate) struct GroupRankings {
    /// The group's peers, whether or not they have a given statistic.
    peers: usize,
    /// Indexed like `MONEYBALL_STATISTIC_KEYS`.
    statistics: Vec<Ranking>,
}

impl GroupRankings {
    /// Ranks each statistic among `peers`, one value per statistic each (NaN when absent).
    pub(crate) fn rank(peers: &[&[f64]]) -> Self {
        let mut population = Vec::with_capacity(peers.len());
        let statistics = (0..MONEYBALL_STATISTIC_KEYS.len())
            .map(|index| {
                population.clear();
                population.extend(
                    peers
                        .iter()
                        .map(|values| values[index])
                        .filter(|value| value.is_finite()),
                );
                population.sort_unstable_by(f64::total_cmp);
                Ranking::from_sorted(&population)
            })
            .collect();
        Self {
            peers: peers.len(),
            statistics,
        }
    }

    pub(crate) fn peers(&self) -> usize {
        self.peers
    }

    /// The percentile of statistic `index` at `value`. `None` for a missing or non-finite value
    /// and when no peer has the statistic.
    pub(crate) fn percentile(&self, index: usize, value: Option<f64>) -> Option<u8> {
        let value = value.filter(|value| value.is_finite())?;
        self.statistics[index].percentile(value, lower_is_better(index))
    }

    /// `percentile` by statistic key. `None` for a key outside the catalog.
    pub(crate) fn percentile_of(&self, key: &str, value: Option<f64>) -> Option<u8> {
        self.percentile(statistic_index(key)?, value)
    }
}

/// The rankings of the groups one request looks up, each resolved once so that scoring many
/// players takes no lock and builds nothing.
pub(crate) struct ResolvedGroups(Vec<(ComparisonGroup, Arc<GroupRankings>)>);

impl ResolvedGroups {
    /// Every family's rankings and those of `slots`, building a slot group on first use.
    pub(crate) fn resolve(
        groups: &ComparisonGroups,
        conn: &Connection,
        slots: impl IntoIterator<Item = ComparisonGroup>,
    ) -> Result<Self, String> {
        let mut resolved = PositionFamily::ALL
            .into_iter()
            .map(|family| {
                (
                    ComparisonGroup::family(family),
                    Arc::clone(groups.family(family)),
                )
            })
            .collect::<Vec<_>>();
        for group in slots {
            if !resolved.iter().any(|(known, _)| *known == group) {
                resolved.push((group, groups.rankings(conn, group)?));
            }
        }
        Ok(Self(resolved))
    }

    /// The group's rankings. `None` for a slot group `resolve` was not given.
    pub(crate) fn get(&self, group: ComparisonGroup) -> Option<&GroupRankings> {
        self.0
            .iter()
            .find(|(known, _)| *known == group)
            .map(|(_, rankings)| &**rankings)
    }

    /// A player's percentile of `key` within `group`, from his statistics in
    /// `MONEYBALL_STATISTIC_KEYS` order (NaN when absent).
    pub(crate) fn percentile(
        &self,
        group: ComparisonGroup,
        key: &str,
        statistics: &[f64],
    ) -> Option<u8> {
        let index = statistic_index(key)?;
        self.get(group)?.percentile(index, Some(statistics[index]))
    }
}

/// A snapshot's comparison groups.
pub(crate) struct ComparisonGroups {
    snapshot_id: i64,
    /// Indexed by `PositionFamily::index`.
    families: Vec<Arc<GroupRankings>>,
    /// Each family's peers' UIDs in ascending order, for building a slot's group.
    members: Vec<Vec<i64>>,
    /// Each player with season statistics and the family of his best position.
    own: HashMap<i64, PositionFamily>,
    /// Slot groups of two families, built when first needed.
    slots: Mutex<HashMap<ComparisonGroup, Arc<GroupRankings>>>,
}

/// One row of a snapshot's season statistics, read for the groups.
struct ScannedPlayer {
    uid: i64,
    own: Option<PositionFamily>,
    /// The families the player is a peer of, as a `ComparisonGroup` bit set.
    peer_of: u16,
    /// Every statistic for a peer, NaN when absent; `None` otherwise.
    statistics: Option<Box<[f64]>>,
}

impl ComparisonGroups {
    /// Reads the snapshot's season statistics and ranks every family's peers.
    pub(crate) fn load(conn: &Connection, snapshot_id: i64) -> Result<Self, String> {
        let mut statement = conn
            .prepare(
                "SELECT moneyball.player_uid, players.positions_json,
                        CASE WHEN moneyball.minutes >= ?2 THEN moneyball.statistics_json END
                 FROM player_moneyball_stats moneyball
                 INNER JOIN players ON players.snapshot_id = moneyball.snapshot_id
                    AND players.uid = moneyball.player_uid
                 WHERE moneyball.snapshot_id = ?1 AND moneyball.percentiles_json IS NOT NULL",
            )
            .map_err(|error| error.to_string())?;
        let mut rows = statement
            .query([snapshot_id, MIN_SEASON_MINUTES])
            .map_err(|error| error.to_string())?;
        let scanned = scan_in_parallel(
            &mut rows,
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
            |(uid, positions_json, statistics_json)| {
                let (own, peer_of) = scan_positions(&positions_json)?;
                let statistics = match statistics_json {
                    Some(json) if peer_of != 0 => Some(scan_statistics(&json)?),
                    _ => None,
                };
                Ok(ScannedPlayer {
                    uid,
                    own,
                    peer_of,
                    statistics,
                })
            },
        )?;

        let mut peers = vec![Vec::new(); PositionFamily::ALL.len()];
        let mut members = vec![Vec::new(); PositionFamily::ALL.len()];
        for player in &scanned {
            let Some(statistics) = &player.statistics else {
                continue;
            };
            for family in ComparisonGroup(player.peer_of).families() {
                peers[family.index()].push(&statistics[..]);
                members[family.index()].push(player.uid);
            }
        }
        let families = std::thread::scope(|scope| {
            let handles = peers
                .iter()
                .map(|peers| scope.spawn(|| GroupRankings::rank(peers)))
                .collect::<Vec<_>>();
            handles
                .into_iter()
                .map(|handle| Arc::new(handle.join().expect("a ranking worker panicked")))
                .collect()
        });
        for uids in &mut members {
            uids.sort_unstable();
        }
        Ok(Self {
            snapshot_id,
            families,
            members,
            own: scanned
                .iter()
                .filter_map(|player| Some((player.uid, player.own?)))
                .collect(),
            slots: Mutex::default(),
        })
    }

    /// The family of the best position of a player with season statistics.
    pub(crate) fn own_family(&self, uid: i64) -> Option<PositionFamily> {
        self.own.get(&uid).copied()
    }

    pub(crate) fn family(&self, family: PositionFamily) -> &Arc<GroupRankings> {
        &self.families[family.index()]
    }

    /// The group's rankings. A slot group of two families is built from its peers' stored
    /// statistics the first time it is asked for, so `conn` must read this snapshot's data
    /// generation.
    pub(crate) fn rankings(
        &self,
        conn: &Connection,
        group: ComparisonGroup,
    ) -> Result<Arc<GroupRankings>, String> {
        if let Some(family) = group.single_family() {
            return Ok(Arc::clone(self.family(family)));
        }
        let mut slots = self.slots.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(rankings) = slots.get(&group) {
            return Ok(Arc::clone(rankings));
        }
        let rankings = Arc::new(self.rank_slot(conn, group)?);
        slots.insert(group, Arc::clone(&rankings));
        Ok(rankings)
    }

    fn rank_slot(
        &self,
        conn: &Connection,
        group: ComparisonGroup,
    ) -> Result<GroupRankings, String> {
        let mut uids = group
            .families()
            .flat_map(|family| self.members[family.index()].iter().copied())
            .collect::<Vec<_>>();
        uids.sort_unstable();
        uids.dedup();
        let mut statement = conn
            .prepare(
                "SELECT statistics_json FROM player_moneyball_stats
                 WHERE snapshot_id = ?1 AND player_uid IN (SELECT value FROM json_each(?2))",
            )
            .map_err(|error| error.to_string())?;
        let uids = serde_json::to_string(&uids).map_err(|error| error.to_string())?;
        let mut rows = statement
            .query(rusqlite::params![self.snapshot_id, uids])
            .map_err(|error| error.to_string())?;
        let statistics = scan_in_parallel(
            &mut rows,
            |row| row.get::<_, String>(0),
            |json| scan_statistics(&json),
        )?;
        let peers = statistics
            .iter()
            .map(|values| &values[..])
            .collect::<Vec<_>>();
        Ok(GroupRankings::rank(&peers))
    }
}

/// The families a player can be compared within, from his familiarity map: his best
/// position's first, then every other family he is rated at least `PEER_FAMILIARITY` in.
pub(crate) fn player_families(positions_json: &str) -> Result<Vec<PositionFamily>, String> {
    let (own, peer_of) = scan_positions(positions_json)?;
    Ok(own
        .into_iter()
        .chain(
            ComparisonGroup(peer_of)
                .families()
                .filter(|family| Some(*family) != own),
        )
        .collect())
}

/// A player's best family and the families he is a peer of, from his familiarity map. Ties
/// between families go to the earlier one in `PositionFamily::ALL`.
fn scan_positions(json: &str) -> Result<(Option<PositionFamily>, u16), String> {
    let mut familiarity = [0_i64; PositionFamily::ALL.len()];
    scan_json_object(json, |position, text| {
        if let Some(family) = PositionFamily::of_position(position) {
            if let Some(value) = parse_json_value(text, |text| text.parse::<i64>().ok())? {
                let best = &mut familiarity[family.index()];
                *best = (*best).max(value);
            }
        }
        Ok(())
    })?;
    let mut own = None;
    let mut peer_of = 0;
    for family in PositionFamily::ALL {
        let value = familiarity[family.index()];
        if value > 0 && own.map_or(true, |own: PositionFamily| value > familiarity[own.index()]) {
            own = Some(family);
        }
        if value >= PEER_FAMILIARITY {
            peer_of |= ComparisonGroup::family(family).0;
        }
    }
    Ok((own, peer_of))
}

/// Every statistic of one stored statistics object, NaN when null or absent.
pub(crate) fn scan_statistics(json: &str) -> Result<Box<[f64]>, String> {
    let mut values = vec![f64::NAN; MONEYBALL_STATISTIC_KEYS.len()].into_boxed_slice();
    scan_json_object(json, |key, text| {
        if let Some(index) = statistic_index(key) {
            if let Some(value) = parse_json_value(text, |text| text.parse::<f64>().ok())? {
                values[index] = value;
            }
        }
        Ok(())
    })?;
    Ok(values)
}

/// Reads `rows` on this thread and runs `scan` on each one on worker threads, so the JSON
/// scanning overlaps SQLite's reads. The results are in no particular order.
fn scan_in_parallel<I: Send, T: Send>(
    rows: &mut Rows<'_>,
    read: impl Fn(&Row<'_>) -> rusqlite::Result<I>,
    scan: impl Fn(I) -> Result<T, String> + Sync,
) -> Result<Vec<T>, String> {
    const CHUNK_ROWS: usize = 512;
    let workers = std::thread::available_parallelism()
        .map_or(1, usize::from)
        .min(8);
    let (sender, receiver) = mpsc::sync_channel::<Vec<I>>(workers * 2);
    let receiver = Mutex::new(receiver);
    let scan_chunks = || {
        let mut scanned = Vec::new();
        let mut result = Ok(());
        // A worker drains every chunk even after an error, so the reader never blocks.
        while let Ok(chunk) = receiver
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .recv()
        {
            for item in chunk {
                if result.is_ok() {
                    match scan(item) {
                        Ok(value) => scanned.push(value),
                        Err(error) => result = Err(error),
                    }
                }
            }
        }
        result.map(|()| scanned)
    };
    std::thread::scope(|scope| {
        let handles = (0..workers)
            .map(|_| scope.spawn(scan_chunks))
            .collect::<Vec<_>>();
        let read = (|| {
            let mut chunk = Vec::with_capacity(CHUNK_ROWS);
            while let Some(row) = rows.next().map_err(|error| error.to_string())? {
                chunk.push(read(row).map_err(|error| error.to_string())?);
                if chunk.len() == CHUNK_ROWS {
                    let full = std::mem::replace(&mut chunk, Vec::with_capacity(CHUNK_ROWS));
                    sender.send(full).map_err(|error| error.to_string())?;
                }
            }
            sender.send(chunk).map_err(|error| error.to_string())
        })();
        drop(sender);
        let mut scanned = Vec::new();
        for handle in handles {
            scanned.extend(handle.join().expect("a Moneyball scan worker panicked")?);
        }
        read.map(|()| scanned)
    })
}

/// The comparison groups of the newest data generation, kept as app state. Clones share them.
#[derive(Clone)]
pub struct ComparisonGroupsCache(ResultCache<ComparisonGroups>);

impl Default for ComparisonGroupsCache {
    fn default() -> Self {
        Self(ResultCache::new(1))
    }
}

/// Where a request gets its comparison groups: the app's cache at the data generation its
/// reader reads, or a fresh build for a connection outside the app's database.
#[derive(Clone, Copy)]
pub(crate) enum GroupsSource<'a> {
    Cached(&'a ComparisonGroupsCache, u64),
    #[cfg_attr(not(test), allow(dead_code))]
    Uncached,
}

impl GroupsSource<'_> {
    pub(crate) fn load(
        &self,
        conn: &Connection,
        snapshot_id: i64,
    ) -> Result<Arc<ComparisonGroups>, String> {
        match self {
            Self::Cached(cache, generation) => {
                let groups =
                    cache
                        .0
                        .get_or_compute(&snapshot_id.to_string(), *generation, || {
                            Ok::<_, String>((
                                *generation,
                                ComparisonGroups::load(conn, snapshot_id)?,
                            ))
                        })?;
                Ok(groups)
            }
            Self::Uncached => ComparisonGroups::load(conn, snapshot_id).map(Arc::new),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ranking rule applied to the whole population.
    fn reference_percentile(population: &[f64], value: f64, lower_is_better: bool) -> Option<u8> {
        let n = population.len();
        if n == 0 {
            return None;
        }
        if n == 1 || population[0] == population[n - 1] {
            return Some(50);
        }
        let below = population.iter().filter(|peer| **peer < value).count();
        let score = ((below as f64 / (n - 1) as f64) * 100.0)
            .round()
            .clamp(0.0, 100.0) as u8;
        Some(if lower_is_better { 100 - score } else { score })
    }

    /// A deterministic xorshift generator.
    struct Generator(u64);

    impl Generator {
        fn below(&mut self, bound: u64) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0 % bound
        }
    }

    #[test]
    fn cuts_rank_every_value_exactly_as_the_whole_population_does() {
        let mut generator = Generator(0x9e37_79b9_7f4a_7c15);
        for round in 0..400 {
            let size = 1 + generator.below(if round % 4 == 0 { 8 } else { 400 }) as usize;
            // Few distinct values make ties common; many make every rank distinct.
            let spread = 1 + generator.below(if round % 3 == 0 { 6 } else { 10_000 });
            let mut population = (0..size)
                .map(|_| generator.below(spread) as f64 / 4.0)
                .collect::<Vec<_>>();
            population.sort_by(f64::total_cmp);
            let ranking = Ranking::from_sorted(&population);
            let probes = population
                .iter()
                .copied()
                .chain((0..50).map(|_| (generator.below(spread + 2) as f64 - 1.0) / 4.0 + 0.125))
                .chain([f64::MIN, f64::MAX]);
            for value in probes {
                for lower_is_better in [false, true] {
                    assert_eq!(
                        ranking.percentile(value, lower_is_better),
                        reference_percentile(&population, value, lower_is_better),
                        "round {round}, value {value}, lower is better: {lower_is_better}"
                    );
                }
            }
        }
    }

    #[test]
    fn an_empty_population_ranks_nobody() {
        assert_eq!(Ranking::from_sorted(&[]).percentile(1.0, false), None);
    }

    #[test]
    fn slot_groups_join_two_families_and_keep_one() {
        let centre_backs = ComparisonGroup::family(PositionFamily::CentralDefender);
        assert_eq!(ComparisonGroup::slot("DC", "DC"), Some(centre_backs));
        assert_eq!(
            ComparisonGroup::slot("DL", "DR"),
            Some(ComparisonGroup::family(PositionFamily::FullBack))
        );
        let wide = ComparisonGroup::slot("AML", "ML").expect("slot group");
        assert_eq!(
            wide.families().collect::<Vec<_>>(),
            vec![PositionFamily::WideMidfielder, PositionFamily::Winger]
        );
        assert_eq!(ComparisonGroup::slot("SW", "DC"), None);
    }

    #[test]
    fn a_player_belongs_to_his_best_family_and_is_a_peer_where_rated_sixteen() {
        let (own, peer_of) =
            scan_positions(r#"{"GK":1,"DC":16,"DM":18,"MC":18,"ST":15,"SW":20}"#).unwrap();
        assert_eq!(own, Some(PositionFamily::DefensiveMidfielder));
        assert_eq!(
            ComparisonGroup(peer_of).families().collect::<Vec<_>>(),
            vec![
                PositionFamily::CentralDefender,
                PositionFamily::DefensiveMidfielder,
                PositionFamily::CentralMidfielder
            ]
        );
        assert_eq!(scan_positions(r#"{"DL":null}"#).unwrap(), (None, 0));
    }
}
