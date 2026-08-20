use std::collections::BTreeMap;

use crate::features::moneyball::{MoneyballMetricValue, MoneyballStatistics};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum YouthTrackerHiddenAttribute {
    Ambition,
    Consistency,
    ImportantMatches,
    InjuryProneness,
    Professionalism,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum YouthTrackerAttribute {
    Corners,
    Crossing,
    Dribbling,
    Finishing,
    FirstTouch,
    FreeKickTaking,
    Heading,
    LongShots,
    LongThrows,
    Marking,
    Passing,
    PenaltyTaking,
    Tackling,
    Technique,
    Aggression,
    Anticipation,
    Bravery,
    Composure,
    Concentration,
    Decisions,
    Flair,
    Leadership,
    OffTheBall,
    Positioning,
    TeamWork,
    Vision,
    WorkRate,
    Acceleration,
    Agility,
    Balance,
    JumpingReach,
    NaturalFitness,
    Pace,
    Stamina,
    Strength,
    AerialReach,
    CommandOfArea,
    Communication,
    Eccentricity,
    Handling,
    Kicking,
    OneOnOnes,
    Punching,
    Reflexes,
    RushingOutTendency,
    Throwing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YouthTrackerPlayer {
    pub uid: u32,
    pub name: Option<String>,
    pub age: Option<u8>,
    pub best_position: Option<String>,
    pub positions: Option<String>,
    pub nationality: Option<String>,
    pub ca: Option<u8>,
    pub pa: Option<u8>,
    pub hidden_attributes: BTreeMap<YouthTrackerHiddenAttribute, Option<u8>>,
    pub height: Option<String>,
    pub determination: Option<u8>,
    pub personality: Option<String>,
    pub preferred_foot: Option<String>,
    pub all_time_appearances: Option<u32>,
    pub international_appearances: Option<u32>,
    pub all_time_goals: Option<u32>,
    pub assists: Option<u32>,
    pub attributes: BTreeMap<YouthTrackerAttribute, Option<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyballTransferValue {
    Single { euros: u64 },
    Range { lower_euros: u64, upper_euros: u64 },
    NotForSale,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoneyballWage {
    pub euros_per_week: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoneyballAppearances {
    pub starts: u32,
    pub substitutes: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoneyballPlayer {
    pub uid: u32,
    pub name: Option<String>,
    pub nation: Option<String>,
    pub second_nation: Option<String>,
    pub club: Option<String>,
    pub division: Option<String>,
    pub position: Option<String>,
    pub age: Option<u8>,
    pub height_centimeters: Option<u16>,
    pub left_foot: Option<String>,
    pub right_foot: Option<String>,
    pub ca: Option<u8>,
    pub pa: Option<u8>,
    pub transfer_value: Option<MoneyballTransferValue>,
    pub asking_price: Option<MoneyballTransferValue>,
    pub wage: Option<MoneyballWage>,
    pub expires: Option<String>,
    pub appearances: Option<MoneyballAppearances>,
    pub minutes: Option<u32>,
    pub distance_kilometers: Option<f64>,
    pub metrics: BTreeMap<String, Option<MoneyballMetricValue>>,
}

#[cfg_attr(not(test), allow(dead_code))]
impl MoneyballPlayer {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn canonical_statistics(&self) -> MoneyballStatistics {
        super::statistics::canonical_statistics(self)
    }

    pub fn metric(&self, header: &str) -> Option<MoneyballMetricValue> {
        self.metrics.get(header).copied().flatten()
    }
}

#[cfg_attr(not(test), allow(dead_code))]
impl YouthTrackerPlayer {
    pub fn attribute(&self, attribute: YouthTrackerAttribute) -> Option<u8> {
        self.attributes.get(&attribute).copied().flatten()
    }

    pub fn hidden_attribute(&self, attribute: YouthTrackerHiddenAttribute) -> Option<u8> {
        self.hidden_attributes.get(&attribute).copied().flatten()
    }
}
