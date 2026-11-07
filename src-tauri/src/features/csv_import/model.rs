use std::collections::BTreeMap;

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

#[cfg_attr(not(test), allow(dead_code))]
impl YouthTrackerPlayer {
    pub fn attribute(&self, attribute: YouthTrackerAttribute) -> Option<u8> {
        self.attributes.get(&attribute).copied().flatten()
    }

    pub fn hidden_attribute(&self, attribute: YouthTrackerHiddenAttribute) -> Option<u8> {
        self.hidden_attributes.get(&attribute).copied().flatten()
    }
}
