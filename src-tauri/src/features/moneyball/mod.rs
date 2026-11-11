use std::collections::BTreeMap;

mod catalog;
pub(crate) mod commands;
pub(crate) mod league_strength;
pub(crate) mod percentile;
pub(crate) mod persistence;
pub(crate) mod query;
#[allow(dead_code)]
pub(crate) mod role_catalog;
#[allow(dead_code)]
pub(crate) mod role_score;
mod save_statistics;
pub(crate) mod save_stats_import;

pub(crate) use catalog::{
    is_moneyball_statistic_key, is_save_unavailable_statistic_key, MONEYBALL_STATISTIC_KEYS,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum MoneyballMetricValue {
    Count(u32),
    Decimal(f64),
}

pub(crate) type MoneyballStatistics = BTreeMap<String, Option<MoneyballMetricValue>>;
