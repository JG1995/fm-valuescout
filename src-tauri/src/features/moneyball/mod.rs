use std::collections::BTreeMap;

mod catalog;
pub(crate) mod percentile;

pub(crate) use catalog::MONEYBALL_STATISTIC_KEYS;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum MoneyballMetricValue {
    Count(u32),
    Decimal(f64),
}

pub(crate) type MoneyballStatistics = BTreeMap<String, Option<MoneyballMetricValue>>;
