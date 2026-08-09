//! Empirical position profiles pinned to FMSuperScout commit 0f270d39.
//!
//! Source: https://github.com/mavarobli/FMSuperScout/blob/0f270d39a9cdc850ddfe653710d4904f13709cb5/app/app.js

pub(super) struct PositionProfile {
    pub(super) group: &'static str,
    pub(super) attributes: &'static [AttributeProfile],
}

pub(super) struct AttributeProfile {
    pub(super) key: &'static str,
    pub(super) anchors: [f64; 4],
}

pub(super) const POSITION_PROFILES: &[PositionProfile] = &[
    PositionProfile {
        group: "ALL",
        attributes: &[
            AttributeProfile {
                key: "Crossing",
                anchors: [7.5, 9.5, 10.7, 12.0],
            },
            AttributeProfile {
                key: "Dribbling",
                anchors: [8.2, 10.5, 12.1, 14.2],
            },
            AttributeProfile {
                key: "Finishing",
                anchors: [7.4, 8.9, 10.0, 11.8],
            },
            AttributeProfile {
                key: "Heading",
                anchors: [8.3, 9.8, 10.7, 11.7],
            },
            AttributeProfile {
                key: "LongShots",
                anchors: [6.7, 9.1, 10.5, 11.9],
            },
            AttributeProfile {
                key: "Marking",
                anchors: [7.9, 9.3, 10.3, 11.2],
            },
            AttributeProfile {
                key: "OffTheBall",
                anchors: [8.8, 11.0, 12.3, 13.9],
            },
            AttributeProfile {
                key: "Passing",
                anchors: [9.3, 11.3, 12.8, 14.8],
            },
            AttributeProfile {
                key: "PenaltyTaking",
                anchors: [5.0, 8.1, 10.0, 11.2],
            },
            AttributeProfile {
                key: "Tackling",
                anchors: [8.5, 9.6, 10.8, 11.6],
            },
            AttributeProfile {
                key: "Vision",
                anchors: [8.3, 10.7, 12.4, 14.5],
            },
            AttributeProfile {
                key: "Handling",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "AerialReach",
                anchors: [2.1, 2.1, 2.1, 2.0],
            },
            AttributeProfile {
                key: "CommandOfArea",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "Communication",
                anchors: [2.1, 2.1, 2.1, 2.2],
            },
            AttributeProfile {
                key: "Kicking",
                anchors: [2.1, 2.1, 2.1, 2.0],
            },
            AttributeProfile {
                key: "Throwing",
                anchors: [2.1, 2.1, 2.1, 2.0],
            },
            AttributeProfile {
                key: "Anticipation",
                anchors: [9.4, 11.6, 13.3, 15.1],
            },
            AttributeProfile {
                key: "Decisions",
                anchors: [10.9, 11.5, 12.6, 14.4],
            },
            AttributeProfile {
                key: "OneOnOnes",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "Positioning",
                anchors: [8.3, 9.9, 11.1, 12.0],
            },
            AttributeProfile {
                key: "Reflexes",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "FirstTouch",
                anchors: [10.1, 11.6, 13.0, 15.1],
            },
            AttributeProfile {
                key: "Technique",
                anchors: [9.9, 11.8, 13.3, 15.2],
            },
            AttributeProfile {
                key: "Flair",
                anchors: [8.9, 10.3, 11.9, 13.8],
            },
            AttributeProfile {
                key: "Corners",
                anchors: [5.5, 7.8, 8.9, 9.6],
            },
            AttributeProfile {
                key: "Teamwork",
                anchors: [9.1, 11.7, 13.2, 14.4],
            },
            AttributeProfile {
                key: "WorkRate",
                anchors: [10.1, 12.2, 13.7, 14.9],
            },
            AttributeProfile {
                key: "LongThrows",
                anchors: [4.7, 6.9, 7.8, 7.7],
            },
            AttributeProfile {
                key: "Eccentricity",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "RushingOut",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "Punching",
                anchors: [2.1, 2.1, 2.1, 2.0],
            },
            AttributeProfile {
                key: "Acceleration",
                anchors: [11.7, 12.3, 13.4, 14.7],
            },
            AttributeProfile {
                key: "FreeKicks",
                anchors: [5.4, 7.8, 9.1, 10.2],
            },
            AttributeProfile {
                key: "Strength",
                anchors: [8.0, 10.5, 11.9, 13.5],
            },
            AttributeProfile {
                key: "Stamina",
                anchors: [10.0, 11.9, 13.6, 14.9],
            },
            AttributeProfile {
                key: "Pace",
                anchors: [11.7, 12.3, 13.5, 14.9],
            },
            AttributeProfile {
                key: "JumpingReach",
                anchors: [9.5, 10.6, 11.2, 12.0],
            },
            AttributeProfile {
                key: "Leadership",
                anchors: [8.1, 9.2, 10.2, 11.8],
            },
            AttributeProfile {
                key: "Balance",
                anchors: [9.5, 11.4, 12.9, 14.7],
            },
            AttributeProfile {
                key: "Bravery",
                anchors: [8.8, 11.2, 12.8, 13.7],
            },
            AttributeProfile {
                key: "Aggression",
                anchors: [10.4, 11.3, 12.4, 13.3],
            },
            AttributeProfile {
                key: "Agility",
                anchors: [11.2, 11.9, 13.2, 14.8],
            },
            AttributeProfile {
                key: "NaturalFitness",
                anchors: [12.1, 12.5, 13.4, 14.8],
            },
            AttributeProfile {
                key: "Determination",
                anchors: [11.1, 12.4, 13.8, 15.5],
            },
            AttributeProfile {
                key: "Composure",
                anchors: [8.6, 11.1, 12.7, 14.9],
            },
            AttributeProfile {
                key: "Concentration",
                anchors: [8.3, 10.7, 12.3, 13.9],
            },
        ],
    },
    PositionProfile {
        group: "AMC",
        attributes: &[
            AttributeProfile {
                key: "Crossing",
                anchors: [8.0, 10.5, 11.9, 13.2],
            },
            AttributeProfile {
                key: "Dribbling",
                anchors: [10.0, 12.1, 13.7, 15.7],
            },
            AttributeProfile {
                key: "Finishing",
                anchors: [9.6, 10.5, 11.9, 13.8],
            },
            AttributeProfile {
                key: "Heading",
                anchors: [6.5, 7.9, 8.6, 9.6],
            },
            AttributeProfile {
                key: "LongShots",
                anchors: [8.4, 10.7, 12.2, 13.8],
            },
            AttributeProfile {
                key: "Marking",
                anchors: [5.8, 7.0, 7.9, 8.8],
            },
            AttributeProfile {
                key: "OffTheBall",
                anchors: [10.3, 12.1, 13.5, 15.1],
            },
            AttributeProfile {
                key: "Passing",
                anchors: [10.6, 12.3, 13.8, 15.5],
            },
            AttributeProfile {
                key: "PenaltyTaking",
                anchors: [5.9, 9.3, 11.7, 12.6],
            },
            AttributeProfile {
                key: "Tackling",
                anchors: [6.2, 7.4, 8.4, 9.1],
            },
            AttributeProfile {
                key: "Vision",
                anchors: [10.6, 12.4, 13.9, 15.9],
            },
            AttributeProfile {
                key: "Handling",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "AerialReach",
                anchors: [2.1, 2.1, 2.1, 2.0],
            },
            AttributeProfile {
                key: "CommandOfArea",
                anchors: [2.1, 2.1, 2.1, 2.2],
            },
            AttributeProfile {
                key: "Communication",
                anchors: [2.1, 2.1, 2.1, 2.3],
            },
            AttributeProfile {
                key: "Kicking",
                anchors: [2.1, 2.1, 2.1, 1.9],
            },
            AttributeProfile {
                key: "Throwing",
                anchors: [2.1, 2.1, 2.2, 2.1],
            },
            AttributeProfile {
                key: "Anticipation",
                anchors: [9.1, 11.3, 13.0, 14.8],
            },
            AttributeProfile {
                key: "Decisions",
                anchors: [10.9, 11.6, 12.8, 14.4],
            },
            AttributeProfile {
                key: "OneOnOnes",
                anchors: [2.1, 2.1, 2.1, 2.0],
            },
            AttributeProfile {
                key: "Positioning",
                anchors: [6.7, 8.2, 9.2, 9.9],
            },
            AttributeProfile {
                key: "Reflexes",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "FirstTouch",
                anchors: [11.5, 12.6, 14.2, 16.1],
            },
            AttributeProfile {
                key: "Technique",
                anchors: [11.7, 13.1, 14.8, 16.4],
            },
            AttributeProfile {
                key: "Flair",
                anchors: [11.9, 13.0, 14.4, 15.6],
            },
            AttributeProfile {
                key: "Corners",
                anchors: [6.7, 9.9, 11.8, 12.5],
            },
            AttributeProfile {
                key: "Teamwork",
                anchors: [9.2, 11.3, 12.7, 14.1],
            },
            AttributeProfile {
                key: "WorkRate",
                anchors: [10.2, 11.7, 13.1, 14.6],
            },
            AttributeProfile {
                key: "LongThrows",
                anchors: [4.0, 5.4, 5.9, 6.5],
            },
            AttributeProfile {
                key: "Eccentricity",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "RushingOut",
                anchors: [2.1, 2.1, 2.1, 1.9],
            },
            AttributeProfile {
                key: "Punching",
                anchors: [2.1, 2.1, 2.1, 1.9],
            },
            AttributeProfile {
                key: "Acceleration",
                anchors: [12.0, 12.3, 13.3, 14.3],
            },
            AttributeProfile {
                key: "FreeKicks",
                anchors: [6.6, 9.8, 11.6, 12.7],
            },
            AttributeProfile {
                key: "Strength",
                anchors: [6.7, 8.8, 10.0, 11.9],
            },
            AttributeProfile {
                key: "Stamina",
                anchors: [9.7, 11.4, 13.0, 14.6],
            },
            AttributeProfile {
                key: "Pace",
                anchors: [11.7, 12.0, 13.0, 14.3],
            },
            AttributeProfile {
                key: "JumpingReach",
                anchors: [8.0, 8.6, 8.9, 9.8],
            },
            AttributeProfile {
                key: "Leadership",
                anchors: [8.1, 8.7, 9.7, 10.9],
            },
            AttributeProfile {
                key: "Balance",
                anchors: [9.1, 11.1, 12.8, 14.6],
            },
            AttributeProfile {
                key: "Bravery",
                anchors: [7.5, 9.8, 11.2, 12.1],
            },
            AttributeProfile {
                key: "Aggression",
                anchors: [9.7, 10.1, 11.0, 12.1],
            },
            AttributeProfile {
                key: "Agility",
                anchors: [11.5, 12.5, 14.0, 15.6],
            },
            AttributeProfile {
                key: "NaturalFitness",
                anchors: [12.0, 12.2, 13.0, 14.1],
            },
            AttributeProfile {
                key: "Determination",
                anchors: [10.8, 12.2, 13.4, 15.4],
            },
            AttributeProfile {
                key: "Composure",
                anchors: [9.5, 11.5, 13.0, 15.1],
            },
            AttributeProfile {
                key: "Concentration",
                anchors: [7.5, 10.0, 11.6, 13.3],
            },
        ],
    },
    PositionProfile {
        group: "DC",
        attributes: &[
            AttributeProfile {
                key: "Crossing",
                anchors: [5.1, 7.2, 8.4, 8.9],
            },
            AttributeProfile {
                key: "Dribbling",
                anchors: [4.8, 7.8, 9.8, 11.3],
            },
            AttributeProfile {
                key: "Finishing",
                anchors: [4.0, 6.1, 7.0, 8.0],
            },
            AttributeProfile {
                key: "Heading",
                anchors: [11.0, 12.7, 14.0, 15.4],
            },
            AttributeProfile {
                key: "LongShots",
                anchors: [4.6, 6.8, 7.7, 8.3],
            },
            AttributeProfile {
                key: "Marking",
                anchors: [11.7, 12.6, 13.9, 15.4],
            },
            AttributeProfile {
                key: "OffTheBall",
                anchors: [5.8, 8.3, 9.5, 9.8],
            },
            AttributeProfile {
                key: "Passing",
                anchors: [7.5, 10.6, 12.4, 14.1],
            },
            AttributeProfile {
                key: "PenaltyTaking",
                anchors: [3.9, 6.2, 7.6, 8.4],
            },
            AttributeProfile {
                key: "Tackling",
                anchors: [11.6, 12.7, 14.0, 15.6],
            },
            AttributeProfile {
                key: "Vision",
                anchors: [6.3, 9.5, 11.3, 13.2],
            },
            AttributeProfile {
                key: "Handling",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "AerialReach",
                anchors: [2.1, 2.1, 2.1, 1.9],
            },
            AttributeProfile {
                key: "CommandOfArea",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "Communication",
                anchors: [2.1, 2.1, 2.1, 2.0],
            },
            AttributeProfile {
                key: "Kicking",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "Throwing",
                anchors: [2.1, 2.1, 2.2, 2.1],
            },
            AttributeProfile {
                key: "Anticipation",
                anchors: [10.0, 12.0, 13.8, 15.8],
            },
            AttributeProfile {
                key: "Decisions",
                anchors: [11.6, 11.7, 12.7, 14.6],
            },
            AttributeProfile {
                key: "OneOnOnes",
                anchors: [2.1, 2.1, 2.1, 1.9],
            },
            AttributeProfile {
                key: "Positioning",
                anchors: [11.6, 12.6, 13.8, 15.5],
            },
            AttributeProfile {
                key: "Reflexes",
                anchors: [2.1, 2.1, 2.0, 2.0],
            },
            AttributeProfile {
                key: "FirstTouch",
                anchors: [7.5, 10.2, 11.7, 13.4],
            },
            AttributeProfile {
                key: "Technique",
                anchors: [6.9, 10.0, 11.8, 13.5],
            },
            AttributeProfile {
                key: "Flair",
                anchors: [5.8, 7.1, 8.6, 9.8],
            },
            AttributeProfile {
                key: "Corners",
                anchors: [3.6, 5.1, 5.4, 5.2],
            },
            AttributeProfile {
                key: "Teamwork",
                anchors: [8.3, 11.8, 13.6, 14.7],
            },
            AttributeProfile {
                key: "WorkRate",
                anchors: [9.0, 12.0, 13.8, 15.1],
            },
            AttributeProfile {
                key: "LongThrows",
                anchors: [4.8, 7.9, 9.4, 9.7],
            },
            AttributeProfile {
                key: "Eccentricity",
                anchors: [2.1, 2.1, 2.2, 2.0],
            },
            AttributeProfile {
                key: "RushingOut",
                anchors: [2.1, 2.1, 2.2, 2.3],
            },
            AttributeProfile {
                key: "Punching",
                anchors: [2.1, 2.1, 2.1, 1.9],
            },
            AttributeProfile {
                key: "Acceleration",
                anchors: [10.6, 11.3, 12.5, 13.8],
            },
            AttributeProfile {
                key: "FreeKicks",
                anchors: [4.1, 5.6, 6.3, 6.6],
            },
            AttributeProfile {
                key: "Strength",
                anchors: [9.7, 12.4, 14.1, 15.8],
            },
            AttributeProfile {
                key: "Stamina",
                anchors: [9.3, 11.8, 13.5, 14.5],
            },
            AttributeProfile {
                key: "Pace",
                anchors: [10.9, 11.8, 13.4, 14.9],
            },
            AttributeProfile {
                key: "JumpingReach",
                anchors: [12.1, 13.5, 14.4, 15.6],
            },
            AttributeProfile {
                key: "Leadership",
                anchors: [8.6, 10.6, 12.0, 13.7],
            },
            AttributeProfile {
                key: "Balance",
                anchors: [9.4, 11.6, 13.2, 14.2],
            },
            AttributeProfile {
                key: "Bravery",
                anchors: [10.7, 12.7, 14.2, 15.6],
            },
            AttributeProfile {
                key: "Aggression",
                anchors: [11.3, 12.4, 13.6, 14.4],
            },
            AttributeProfile {
                key: "Agility",
                anchors: [10.2, 10.7, 11.8, 13.1],
            },
            AttributeProfile {
                key: "NaturalFitness",
                anchors: [12.1, 12.6, 13.3, 14.7],
            },
            AttributeProfile {
                key: "Determination",
                anchors: [11.2, 12.6, 14.1, 15.9],
            },
            AttributeProfile {
                key: "Composure",
                anchors: [7.5, 10.8, 12.7, 14.9],
            },
            AttributeProfile {
                key: "Concentration",
                anchors: [9.7, 11.6, 13.2, 14.8],
            },
        ],
    },
    PositionProfile {
        group: "DM",
        attributes: &[
            AttributeProfile {
                key: "Crossing",
                anchors: [6.0, 8.7, 10.2, 11.7],
            },
            AttributeProfile {
                key: "Dribbling",
                anchors: [6.5, 9.7, 11.6, 13.8],
            },
            AttributeProfile {
                key: "Finishing",
                anchors: [6.0, 7.8, 9.0, 10.6],
            },
            AttributeProfile {
                key: "Heading",
                anchors: [7.5, 9.7, 10.4, 11.5],
            },
            AttributeProfile {
                key: "LongShots",
                anchors: [7.9, 9.7, 11.1, 12.3],
            },
            AttributeProfile {
                key: "Marking",
                anchors: [9.4, 11.0, 12.0, 12.8],
            },
            AttributeProfile {
                key: "OffTheBall",
                anchors: [8.0, 10.4, 12.0, 14.2],
            },
            AttributeProfile {
                key: "Passing",
                anchors: [10.8, 12.4, 13.9, 16.0],
            },
            AttributeProfile {
                key: "PenaltyTaking",
                anchors: [4.8, 8.1, 10.0, 10.8],
            },
            AttributeProfile {
                key: "Tackling",
                anchors: [11.2, 11.9, 13.1, 14.0],
            },
            AttributeProfile {
                key: "Vision",
                anchors: [9.9, 11.7, 13.3, 15.5],
            },
            AttributeProfile {
                key: "Handling",
                anchors: [2.1, 2.1, 2.0, 2.0],
            },
            AttributeProfile {
                key: "AerialReach",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "CommandOfArea",
                anchors: [2.1, 2.1, 2.0, 2.4],
            },
            AttributeProfile {
                key: "Communication",
                anchors: [2.1, 2.1, 2.1, 2.3],
            },
            AttributeProfile {
                key: "Kicking",
                anchors: [2.1, 2.1, 2.1, 1.9],
            },
            AttributeProfile {
                key: "Throwing",
                anchors: [2.1, 2.1, 2.1, 1.9],
            },
            AttributeProfile {
                key: "Anticipation",
                anchors: [9.9, 12.1, 13.8, 15.8],
            },
            AttributeProfile {
                key: "Decisions",
                anchors: [11.9, 12.1, 13.2, 15.0],
            },
            AttributeProfile {
                key: "OneOnOnes",
                anchors: [2.1, 2.1, 2.1, 2.3],
            },
            AttributeProfile {
                key: "Positioning",
                anchors: [10.3, 11.9, 13.3, 14.3],
            },
            AttributeProfile {
                key: "Reflexes",
                anchors: [2.1, 2.1, 2.1, 2.0],
            },
            AttributeProfile {
                key: "FirstTouch",
                anchors: [10.9, 12.1, 13.4, 15.4],
            },
            AttributeProfile {
                key: "Technique",
                anchors: [10.4, 12.0, 13.5, 15.5],
            },
            AttributeProfile {
                key: "Flair",
                anchors: [6.8, 9.3, 11.1, 13.6],
            },
            AttributeProfile {
                key: "Corners",
                anchors: [5.5, 8.0, 9.6, 9.7],
            },
            AttributeProfile {
                key: "Teamwork",
                anchors: [10.2, 12.9, 14.6, 15.5],
            },
            AttributeProfile {
                key: "WorkRate",
                anchors: [11.6, 13.2, 14.7, 15.8],
            },
            AttributeProfile {
                key: "LongThrows",
                anchors: [4.2, 6.3, 7.2, 6.9],
            },
            AttributeProfile {
                key: "Eccentricity",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "RushingOut",
                anchors: [2.1, 2.1, 2.2, 2.1],
            },
            AttributeProfile {
                key: "Punching",
                anchors: [2.1, 2.1, 2.1, 1.9],
            },
            AttributeProfile {
                key: "Acceleration",
                anchors: [10.8, 11.4, 12.4, 13.6],
            },
            AttributeProfile {
                key: "FreeKicks",
                anchors: [6.0, 8.3, 9.7, 10.1],
            },
            AttributeProfile {
                key: "Strength",
                anchors: [8.4, 10.9, 12.3, 13.8],
            },
            AttributeProfile {
                key: "Stamina",
                anchors: [10.2, 12.5, 14.4, 16.1],
            },
            AttributeProfile {
                key: "Pace",
                anchors: [10.9, 11.5, 12.6, 13.7],
            },
            AttributeProfile {
                key: "JumpingReach",
                anchors: [9.6, 10.7, 11.0, 11.8],
            },
            AttributeProfile {
                key: "Leadership",
                anchors: [8.6, 10.4, 11.6, 12.4],
            },
            AttributeProfile {
                key: "Balance",
                anchors: [9.7, 11.7, 13.2, 14.9],
            },
            AttributeProfile {
                key: "Bravery",
                anchors: [9.5, 12.0, 13.5, 14.1],
            },
            AttributeProfile {
                key: "Aggression",
                anchors: [10.9, 12.2, 13.3, 14.1],
            },
            AttributeProfile {
                key: "Agility",
                anchors: [10.4, 11.4, 12.7, 14.4],
            },
            AttributeProfile {
                key: "NaturalFitness",
                anchors: [12.2, 12.7, 13.9, 15.1],
            },
            AttributeProfile {
                key: "Determination",
                anchors: [11.2, 12.8, 14.2, 15.5],
            },
            AttributeProfile {
                key: "Composure",
                anchors: [8.6, 11.5, 13.2, 15.4],
            },
            AttributeProfile {
                key: "Concentration",
                anchors: [9.3, 11.6, 13.4, 14.7],
            },
        ],
    },
    PositionProfile {
        group: "FB",
        attributes: &[
            AttributeProfile {
                key: "Crossing",
                anchors: [9.9, 11.3, 12.6, 13.9],
            },
            AttributeProfile {
                key: "Dribbling",
                anchors: [7.6, 10.3, 12.0, 13.4],
            },
            AttributeProfile {
                key: "Finishing",
                anchors: [4.9, 7.1, 8.4, 9.9],
            },
            AttributeProfile {
                key: "Heading",
                anchors: [7.6, 9.3, 10.1, 11.0],
            },
            AttributeProfile {
                key: "LongShots",
                anchors: [5.4, 8.1, 9.4, 10.8],
            },
            AttributeProfile {
                key: "Marking",
                anchors: [9.3, 10.9, 12.0, 12.7],
            },
            AttributeProfile {
                key: "OffTheBall",
                anchors: [7.8, 10.8, 12.5, 14.2],
            },
            AttributeProfile {
                key: "Passing",
                anchors: [8.8, 11.0, 12.4, 14.5],
            },
            AttributeProfile {
                key: "PenaltyTaking",
                anchors: [4.3, 6.9, 8.4, 9.3],
            },
            AttributeProfile {
                key: "Tackling",
                anchors: [10.7, 11.6, 12.8, 13.3],
            },
            AttributeProfile {
                key: "Vision",
                anchors: [7.8, 10.1, 11.7, 13.8],
            },
            AttributeProfile {
                key: "Handling",
                anchors: [2.1, 2.1, 2.1, 1.9],
            },
            AttributeProfile {
                key: "AerialReach",
                anchors: [2.1, 2.1, 2.1, 2.2],
            },
            AttributeProfile {
                key: "CommandOfArea",
                anchors: [2.1, 2.1, 2.2, 1.8],
            },
            AttributeProfile {
                key: "Communication",
                anchors: [2.1, 2.1, 2.1, 2.2],
            },
            AttributeProfile {
                key: "Kicking",
                anchors: [2.1, 2.1, 2.0, 1.8],
            },
            AttributeProfile {
                key: "Throwing",
                anchors: [2.1, 2.1, 2.1, 1.9],
            },
            AttributeProfile {
                key: "Anticipation",
                anchors: [9.3, 11.4, 12.9, 14.8],
            },
            AttributeProfile {
                key: "Decisions",
                anchors: [11.1, 11.4, 12.4, 14.3],
            },
            AttributeProfile {
                key: "OneOnOnes",
                anchors: [2.1, 2.1, 2.1, 2.0],
            },
            AttributeProfile {
                key: "Positioning",
                anchors: [9.4, 11.2, 12.4, 13.9],
            },
            AttributeProfile {
                key: "Reflexes",
                anchors: [2.1, 2.1, 2.1, 2.2],
            },
            AttributeProfile {
                key: "FirstTouch",
                anchors: [9.3, 11.0, 12.5, 14.3],
            },
            AttributeProfile {
                key: "Technique",
                anchors: [9.2, 11.2, 12.7, 14.4],
            },
            AttributeProfile {
                key: "Flair",
                anchors: [7.4, 9.3, 11.1, 12.7],
            },
            AttributeProfile {
                key: "Corners",
                anchors: [5.4, 8.0, 9.1, 9.4],
            },
            AttributeProfile {
                key: "Teamwork",
                anchors: [10.0, 12.2, 13.6, 14.7],
            },
            AttributeProfile {
                key: "WorkRate",
                anchors: [10.3, 12.7, 14.2, 15.1],
            },
            AttributeProfile {
                key: "LongThrows",
                anchors: [6.5, 9.8, 11.2, 11.1],
            },
            AttributeProfile {
                key: "Eccentricity",
                anchors: [2.1, 2.1, 2.2, 2.1],
            },
            AttributeProfile {
                key: "RushingOut",
                anchors: [2.1, 2.1, 2.2, 2.4],
            },
            AttributeProfile {
                key: "Punching",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "Acceleration",
                anchors: [12.4, 12.8, 14.0, 15.4],
            },
            AttributeProfile {
                key: "FreeKicks",
                anchors: [4.3, 7.1, 8.5, 9.5],
            },
            AttributeProfile {
                key: "Strength",
                anchors: [8.0, 10.2, 11.7, 13.5],
            },
            AttributeProfile {
                key: "Stamina",
                anchors: [10.9, 12.4, 14.1, 15.2],
            },
            AttributeProfile {
                key: "Pace",
                anchors: [12.2, 12.7, 14.1, 15.5],
            },
            AttributeProfile {
                key: "JumpingReach",
                anchors: [9.2, 9.9, 10.7, 11.6],
            },
            AttributeProfile {
                key: "Leadership",
                anchors: [8.3, 9.0, 10.0, 11.3],
            },
            AttributeProfile {
                key: "Balance",
                anchors: [9.8, 11.3, 12.7, 14.5],
            },
            AttributeProfile {
                key: "Bravery",
                anchors: [9.9, 11.6, 13.2, 14.0],
            },
            AttributeProfile {
                key: "Aggression",
                anchors: [10.5, 11.5, 12.8, 14.0],
            },
            AttributeProfile {
                key: "Agility",
                anchors: [11.8, 12.2, 13.3, 14.9],
            },
            AttributeProfile {
                key: "NaturalFitness",
                anchors: [12.2, 12.7, 13.7, 15.1],
            },
            AttributeProfile {
                key: "Determination",
                anchors: [11.1, 12.4, 13.9, 15.6],
            },
            AttributeProfile {
                key: "Composure",
                anchors: [7.8, 10.5, 12.2, 14.4],
            },
            AttributeProfile {
                key: "Concentration",
                anchors: [9.7, 11.2, 12.6, 14.1],
            },
        ],
    },
    PositionProfile {
        group: "GK",
        attributes: &[
            AttributeProfile {
                key: "Crossing",
                anchors: [2.1, 2.3, 2.3, 2.3],
            },
            AttributeProfile {
                key: "Dribbling",
                anchors: [2.1, 2.4, 2.8, 3.7],
            },
            AttributeProfile {
                key: "Finishing",
                anchors: [2.0, 2.1, 2.1, 2.2],
            },
            AttributeProfile {
                key: "Heading",
                anchors: [4.3, 5.3, 5.4, 7.3],
            },
            AttributeProfile {
                key: "LongShots",
                anchors: [2.1, 2.5, 2.7, 2.5],
            },
            AttributeProfile {
                key: "Marking",
                anchors: [2.1, 2.3, 2.4, 1.8],
            },
            AttributeProfile {
                key: "OffTheBall",
                anchors: [2.4, 3.4, 4.1, 3.9],
            },
            AttributeProfile {
                key: "Passing",
                anchors: [8.2, 10.1, 11.3, 12.6],
            },
            AttributeProfile {
                key: "PenaltyTaking",
                anchors: [2.4, 2.8, 3.5, 4.8],
            },
            AttributeProfile {
                key: "Tackling",
                anchors: [2.1, 2.5, 2.6, 3.0],
            },
            AttributeProfile {
                key: "Vision",
                anchors: [6.6, 9.0, 10.5, 12.0],
            },
            AttributeProfile {
                key: "Handling",
                anchors: [11.5, 12.3, 13.4, 15.1],
            },
            AttributeProfile {
                key: "AerialReach",
                anchors: [11.4, 13.2, 14.1, 15.4],
            },
            AttributeProfile {
                key: "CommandOfArea",
                anchors: [9.6, 11.5, 12.9, 14.6],
            },
            AttributeProfile {
                key: "Communication",
                anchors: [9.0, 11.4, 12.8, 14.6],
            },
            AttributeProfile {
                key: "Kicking",
                anchors: [9.9, 11.6, 13.0, 14.0],
            },
            AttributeProfile {
                key: "Throwing",
                anchors: [8.7, 11.6, 13.2, 14.3],
            },
            AttributeProfile {
                key: "Anticipation",
                anchors: [8.8, 11.5, 13.0, 14.5],
            },
            AttributeProfile {
                key: "Decisions",
                anchors: [11.7, 11.5, 12.4, 13.9],
            },
            AttributeProfile {
                key: "OneOnOnes",
                anchors: [9.9, 12.6, 14.4, 16.6],
            },
            AttributeProfile {
                key: "Positioning",
                anchors: [9.7, 12.1, 13.3, 15.1],
            },
            AttributeProfile {
                key: "Reflexes",
                anchors: [12.0, 13.3, 15.2, 17.1],
            },
            AttributeProfile {
                key: "FirstTouch",
                anchors: [4.3, 7.6, 9.7, 11.9],
            },
            AttributeProfile {
                key: "Technique",
                anchors: [4.9, 8.1, 9.9, 12.0],
            },
            AttributeProfile {
                key: "Flair",
                anchors: [2.5, 4.4, 5.7, 8.2],
            },
            AttributeProfile {
                key: "Corners",
                anchors: [3.2, 3.7, 4.1, 4.7],
            },
            AttributeProfile {
                key: "Teamwork",
                anchors: [7.7, 10.8, 12.2, 12.9],
            },
            AttributeProfile {
                key: "WorkRate",
                anchors: [6.8, 10.3, 11.9, 13.4],
            },
            AttributeProfile {
                key: "LongThrows",
                anchors: [2.1, 2.6, 2.9, 3.0],
            },
            AttributeProfile {
                key: "Eccentricity",
                anchors: [6.5, 8.0, 8.9, 9.3],
            },
            AttributeProfile {
                key: "RushingOut",
                anchors: [9.5, 10.7, 12.0, 13.2],
            },
            AttributeProfile {
                key: "Punching",
                anchors: [9.3, 10.3, 11.0, 10.6],
            },
            AttributeProfile {
                key: "Acceleration",
                anchors: [9.4, 9.7, 10.3, 11.1],
            },
            AttributeProfile {
                key: "FreeKicks",
                anchors: [4.3, 4.9, 5.2, 5.4],
            },
            AttributeProfile {
                key: "Strength",
                anchors: [7.4, 10.8, 12.4, 13.4],
            },
            AttributeProfile {
                key: "Stamina",
                anchors: [5.7, 9.4, 11.3, 12.2],
            },
            AttributeProfile {
                key: "Pace",
                anchors: [8.9, 9.6, 10.3, 11.3],
            },
            AttributeProfile {
                key: "JumpingReach",
                anchors: [12.7, 14.1, 14.9, 15.6],
            },
            AttributeProfile {
                key: "Leadership",
                anchors: [8.0, 9.9, 11.4, 13.0],
            },
            AttributeProfile {
                key: "Balance",
                anchors: [8.3, 10.8, 12.0, 12.4],
            },
            AttributeProfile {
                key: "Bravery",
                anchors: [11.5, 12.2, 13.2, 14.1],
            },
            AttributeProfile {
                key: "Aggression",
                anchors: [9.2, 9.5, 10.1, 11.7],
            },
            AttributeProfile {
                key: "Agility",
                anchors: [10.7, 12.0, 13.3, 15.0],
            },
            AttributeProfile {
                key: "NaturalFitness",
                anchors: [11.7, 12.2, 13.1, 13.6],
            },
            AttributeProfile {
                key: "Determination",
                anchors: [11.1, 12.3, 13.6, 15.5],
            },
            AttributeProfile {
                key: "Composure",
                anchors: [7.2, 10.7, 12.6, 14.3],
            },
            AttributeProfile {
                key: "Concentration",
                anchors: [10.2, 11.6, 12.9, 14.5],
            },
        ],
    },
    PositionProfile {
        group: "MC",
        attributes: &[
            AttributeProfile {
                key: "Crossing",
                anchors: [6.7, 9.5, 10.9, 12.2],
            },
            AttributeProfile {
                key: "Dribbling",
                anchors: [7.6, 10.6, 12.3, 14.3],
            },
            AttributeProfile {
                key: "Finishing",
                anchors: [7.2, 8.8, 10.1, 11.8],
            },
            AttributeProfile {
                key: "Heading",
                anchors: [6.6, 8.7, 9.6, 10.9],
            },
            AttributeProfile {
                key: "LongShots",
                anchors: [8.4, 10.3, 11.6, 13.0],
            },
            AttributeProfile {
                key: "Marking",
                anchors: [8.3, 9.7, 10.7, 11.8],
            },
            AttributeProfile {
                key: "OffTheBall",
                anchors: [9.1, 11.3, 12.7, 14.7],
            },
            AttributeProfile {
                key: "Passing",
                anchors: [11.4, 12.7, 14.0, 15.9],
            },
            AttributeProfile {
                key: "PenaltyTaking",
                anchors: [5.1, 8.6, 10.7, 11.5],
            },
            AttributeProfile {
                key: "Tackling",
                anchors: [9.1, 10.5, 11.6, 12.6],
            },
            AttributeProfile {
                key: "Vision",
                anchors: [10.8, 12.2, 13.6, 15.7],
            },
            AttributeProfile {
                key: "Handling",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "AerialReach",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "CommandOfArea",
                anchors: [2.1, 2.1, 2.1, 2.2],
            },
            AttributeProfile {
                key: "Communication",
                anchors: [2.1, 2.1, 2.1, 2.2],
            },
            AttributeProfile {
                key: "Kicking",
                anchors: [2.1, 2.1, 2.1, 1.9],
            },
            AttributeProfile {
                key: "Throwing",
                anchors: [2.1, 2.1, 2.2, 1.9],
            },
            AttributeProfile {
                key: "Anticipation",
                anchors: [9.4, 11.9, 13.5, 15.3],
            },
            AttributeProfile {
                key: "Decisions",
                anchors: [11.5, 12.0, 13.1, 14.9],
            },
            AttributeProfile {
                key: "OneOnOnes",
                anchors: [2.1, 2.1, 2.1, 2.2],
            },
            AttributeProfile {
                key: "Positioning",
                anchors: [8.7, 10.8, 12.1, 13.2],
            },
            AttributeProfile {
                key: "Reflexes",
                anchors: [2.1, 2.1, 2.1, 2.0],
            },
            AttributeProfile {
                key: "FirstTouch",
                anchors: [11.6, 12.4, 13.7, 15.6],
            },
            AttributeProfile {
                key: "Technique",
                anchors: [11.2, 12.6, 14.0, 15.8],
            },
            AttributeProfile {
                key: "Flair",
                anchors: [8.8, 10.7, 12.4, 14.4],
            },
            AttributeProfile {
                key: "Corners",
                anchors: [6.1, 9.0, 10.6, 11.0],
            },
            AttributeProfile {
                key: "Teamwork",
                anchors: [9.9, 12.5, 14.1, 15.3],
            },
            AttributeProfile {
                key: "WorkRate",
                anchors: [11.4, 12.9, 14.2, 15.6],
            },
            AttributeProfile {
                key: "LongThrows",
                anchors: [4.1, 6.0, 6.8, 7.0],
            },
            AttributeProfile {
                key: "Eccentricity",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "RushingOut",
                anchors: [2.1, 2.1, 2.2, 2.2],
            },
            AttributeProfile {
                key: "Punching",
                anchors: [2.1, 2.1, 2.1, 1.9],
            },
            AttributeProfile {
                key: "Acceleration",
                anchors: [11.3, 11.7, 12.6, 13.8],
            },
            AttributeProfile {
                key: "FreeKicks",
                anchors: [6.2, 9.0, 10.5, 11.4],
            },
            AttributeProfile {
                key: "Strength",
                anchors: [7.7, 10.1, 11.5, 13.1],
            },
            AttributeProfile {
                key: "Stamina",
                anchors: [10.3, 12.2, 14.1, 15.8],
            },
            AttributeProfile {
                key: "Pace",
                anchors: [11.3, 11.7, 12.7, 13.9],
            },
            AttributeProfile {
                key: "JumpingReach",
                anchors: [8.8, 9.7, 10.3, 11.2],
            },
            AttributeProfile {
                key: "Leadership",
                anchors: [8.4, 9.9, 11.0, 12.1],
            },
            AttributeProfile {
                key: "Balance",
                anchors: [9.4, 11.5, 13.0, 14.5],
            },
            AttributeProfile {
                key: "Bravery",
                anchors: [8.3, 11.2, 12.8, 13.8],
            },
            AttributeProfile {
                key: "Aggression",
                anchors: [10.6, 11.6, 12.6, 13.8],
            },
            AttributeProfile {
                key: "Agility",
                anchors: [11.0, 11.9, 13.1, 14.7],
            },
            AttributeProfile {
                key: "NaturalFitness",
                anchors: [12.2, 12.5, 13.6, 15.0],
            },
            AttributeProfile {
                key: "Determination",
                anchors: [11.2, 12.6, 13.9, 15.7],
            },
            AttributeProfile {
                key: "Composure",
                anchors: [8.9, 11.6, 13.2, 15.3],
            },
            AttributeProfile {
                key: "Concentration",
                anchors: [8.7, 11.1, 12.8, 14.4],
            },
        ],
    },
    PositionProfile {
        group: "ST",
        attributes: &[
            AttributeProfile {
                key: "Crossing",
                anchors: [6.5, 8.7, 9.7, 11.4],
            },
            AttributeProfile {
                key: "Dribbling",
                anchors: [10.6, 11.3, 12.4, 14.9],
            },
            AttributeProfile {
                key: "Finishing",
                anchors: [11.8, 12.4, 13.9, 15.5],
            },
            AttributeProfile {
                key: "Heading",
                anchors: [10.5, 11.5, 12.7, 12.5],
            },
            AttributeProfile {
                key: "LongShots",
                anchors: [7.0, 9.8, 11.4, 13.0],
            },
            AttributeProfile {
                key: "Marking",
                anchors: [4.8, 6.2, 6.9, 7.6],
            },
            AttributeProfile {
                key: "OffTheBall",
                anchors: [11.3, 12.7, 14.1, 15.7],
            },
            AttributeProfile {
                key: "Passing",
                anchors: [8.2, 10.4, 11.8, 13.8],
            },
            AttributeProfile {
                key: "PenaltyTaking",
                anchors: [6.4, 10.2, 12.6, 14.5],
            },
            AttributeProfile {
                key: "Tackling",
                anchors: [4.1, 6.0, 6.9, 7.5],
            },
            AttributeProfile {
                key: "Vision",
                anchors: [7.4, 10.1, 11.8, 13.9],
            },
            AttributeProfile {
                key: "Handling",
                anchors: [2.1, 2.1, 2.1, 2.0],
            },
            AttributeProfile {
                key: "AerialReach",
                anchors: [2.1, 2.1, 2.1, 2.0],
            },
            AttributeProfile {
                key: "CommandOfArea",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "Communication",
                anchors: [2.1, 2.1, 2.1, 2.2],
            },
            AttributeProfile {
                key: "Kicking",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "Throwing",
                anchors: [2.1, 2.1, 2.1, 1.9],
            },
            AttributeProfile {
                key: "Anticipation",
                anchors: [9.8, 11.8, 13.4, 15.2],
            },
            AttributeProfile {
                key: "Decisions",
                anchors: [10.0, 11.2, 12.4, 14.1],
            },
            AttributeProfile {
                key: "OneOnOnes",
                anchors: [2.1, 2.1, 2.2, 2.3],
            },
            AttributeProfile {
                key: "Positioning",
                anchors: [5.7, 7.4, 7.9, 8.1],
            },
            AttributeProfile {
                key: "Reflexes",
                anchors: [2.1, 2.1, 2.1, 1.9],
            },
            AttributeProfile {
                key: "FirstTouch",
                anchors: [11.0, 11.8, 13.0, 15.2],
            },
            AttributeProfile {
                key: "Technique",
                anchors: [10.5, 11.9, 13.4, 15.4],
            },
            AttributeProfile {
                key: "Flair",
                anchors: [11.0, 11.5, 12.9, 15.2],
            },
            AttributeProfile {
                key: "Corners",
                anchors: [5.3, 7.0, 7.6, 8.9],
            },
            AttributeProfile {
                key: "Teamwork",
                anchors: [8.2, 11.0, 12.6, 13.9],
            },
            AttributeProfile {
                key: "WorkRate",
                anchors: [9.1, 12.0, 13.5, 14.4],
            },
            AttributeProfile {
                key: "LongThrows",
                anchors: [3.7, 5.5, 6.1, 6.4],
            },
            AttributeProfile {
                key: "Eccentricity",
                anchors: [2.1, 2.1, 2.1, 2.4],
            },
            AttributeProfile {
                key: "RushingOut",
                anchors: [2.1, 2.1, 2.1, 1.8],
            },
            AttributeProfile {
                key: "Punching",
                anchors: [2.1, 2.1, 2.0, 2.2],
            },
            AttributeProfile {
                key: "Acceleration",
                anchors: [12.0, 12.5, 13.6, 15.5],
            },
            AttributeProfile {
                key: "FreeKicks",
                anchors: [5.5, 7.7, 8.9, 10.4],
            },
            AttributeProfile {
                key: "Strength",
                anchors: [8.5, 11.4, 13.0, 13.8],
            },
            AttributeProfile {
                key: "Stamina",
                anchors: [9.7, 11.6, 13.2, 14.3],
            },
            AttributeProfile {
                key: "Pace",
                anchors: [11.8, 12.7, 13.8, 15.6],
            },
            AttributeProfile {
                key: "JumpingReach",
                anchors: [10.0, 11.8, 12.5, 12.5],
            },
            AttributeProfile {
                key: "Leadership",
                anchors: [7.8, 8.4, 9.3, 11.0],
            },
            AttributeProfile {
                key: "Balance",
                anchors: [9.5, 11.7, 13.1, 14.9],
            },
            AttributeProfile {
                key: "Bravery",
                anchors: [7.8, 11.0, 12.8, 13.2],
            },
            AttributeProfile {
                key: "Aggression",
                anchors: [9.8, 10.9, 12.2, 12.8],
            },
            AttributeProfile {
                key: "Agility",
                anchors: [11.1, 11.8, 13.0, 15.2],
            },
            AttributeProfile {
                key: "NaturalFitness",
                anchors: [12.1, 12.3, 13.2, 14.9],
            },
            AttributeProfile {
                key: "Determination",
                anchors: [11.0, 12.4, 14.0, 15.3],
            },
            AttributeProfile {
                key: "Composure",
                anchors: [10.2, 11.6, 13.0, 14.9],
            },
            AttributeProfile {
                key: "Concentration",
                anchors: [6.9, 10.1, 11.8, 13.1],
            },
        ],
    },
    PositionProfile {
        group: "W",
        attributes: &[
            AttributeProfile {
                key: "Crossing",
                anchors: [10.0, 11.1, 12.2, 13.9],
            },
            AttributeProfile {
                key: "Dribbling",
                anchors: [10.7, 12.4, 14.0, 16.4],
            },
            AttributeProfile {
                key: "Finishing",
                anchors: [9.0, 10.3, 11.6, 14.1],
            },
            AttributeProfile {
                key: "Heading",
                anchors: [6.4, 7.9, 8.4, 9.2],
            },
            AttributeProfile {
                key: "LongShots",
                anchors: [7.1, 10.0, 11.6, 13.1],
            },
            AttributeProfile {
                key: "Marking",
                anchors: [5.4, 6.7, 7.5, 8.0],
            },
            AttributeProfile {
                key: "OffTheBall",
                anchors: [10.2, 12.0, 13.3, 15.3],
            },
            AttributeProfile {
                key: "Passing",
                anchors: [9.8, 11.3, 12.5, 14.6],
            },
            AttributeProfile {
                key: "PenaltyTaking",
                anchors: [5.4, 8.8, 10.8, 12.4],
            },
            AttributeProfile {
                key: "Tackling",
                anchors: [5.5, 6.9, 7.7, 7.8],
            },
            AttributeProfile {
                key: "Vision",
                anchors: [8.4, 10.9, 12.6, 14.9],
            },
            AttributeProfile {
                key: "Handling",
                anchors: [2.1, 2.1, 2.0, 2.3],
            },
            AttributeProfile {
                key: "AerialReach",
                anchors: [2.1, 2.1, 2.1, 2.0],
            },
            AttributeProfile {
                key: "CommandOfArea",
                anchors: [2.1, 2.1, 2.1, 1.9],
            },
            AttributeProfile {
                key: "Communication",
                anchors: [2.1, 2.1, 2.1, 2.2],
            },
            AttributeProfile {
                key: "Kicking",
                anchors: [2.1, 2.1, 2.1, 2.0],
            },
            AttributeProfile {
                key: "Throwing",
                anchors: [2.1, 2.1, 2.1, 2.1],
            },
            AttributeProfile {
                key: "Anticipation",
                anchors: [8.7, 11.0, 12.5, 14.3],
            },
            AttributeProfile {
                key: "Decisions",
                anchors: [9.7, 10.8, 12.0, 14.0],
            },
            AttributeProfile {
                key: "OneOnOnes",
                anchors: [2.1, 2.1, 2.1, 2.2],
            },
            AttributeProfile {
                key: "Positioning",
                anchors: [5.8, 7.5, 8.4, 8.7],
            },
            AttributeProfile {
                key: "Reflexes",
                anchors: [2.1, 2.1, 2.1, 2.0],
            },
            AttributeProfile {
                key: "FirstTouch",
                anchors: [10.8, 12.0, 13.4, 15.8],
            },
            AttributeProfile {
                key: "Technique",
                anchors: [11.0, 12.5, 14.1, 16.2],
            },
            AttributeProfile {
                key: "Flair",
                anchors: [11.5, 12.7, 14.2, 16.0],
            },
            AttributeProfile {
                key: "Corners",
                anchors: [6.8, 9.3, 10.8, 12.3],
            },
            AttributeProfile {
                key: "Teamwork",
                anchors: [9.0, 10.7, 12.0, 13.2],
            },
            AttributeProfile {
                key: "WorkRate",
                anchors: [10.1, 11.6, 12.9, 14.1],
            },
            AttributeProfile {
                key: "LongThrows",
                anchors: [4.7, 6.2, 6.6, 6.2],
            },
            AttributeProfile {
                key: "Eccentricity",
                anchors: [2.1, 2.1, 2.1, 2.2],
            },
            AttributeProfile {
                key: "RushingOut",
                anchors: [2.1, 2.1, 2.1, 1.8],
            },
            AttributeProfile {
                key: "Punching",
                anchors: [2.1, 2.1, 2.1, 2.0],
            },
            AttributeProfile {
                key: "Acceleration",
                anchors: [12.5, 13.2, 14.5, 15.7],
            },
            AttributeProfile {
                key: "FreeKicks",
                anchors: [6.1, 8.8, 10.4, 11.8],
            },
            AttributeProfile {
                key: "Strength",
                anchors: [6.6, 8.9, 10.1, 11.6],
            },
            AttributeProfile {
                key: "Stamina",
                anchors: [10.0, 11.4, 12.9, 14.1],
            },
            AttributeProfile {
                key: "Pace",
                anchors: [12.3, 12.9, 14.1, 15.2],
            },
            AttributeProfile {
                key: "JumpingReach",
                anchors: [7.9, 8.6, 8.9, 9.6],
            },
            AttributeProfile {
                key: "Leadership",
                anchors: [7.7, 7.8, 8.5, 10.4],
            },
            AttributeProfile {
                key: "Balance",
                anchors: [9.5, 11.1, 12.6, 14.7],
            },
            AttributeProfile {
                key: "Bravery",
                anchors: [7.5, 9.8, 11.3, 12.0],
            },
            AttributeProfile {
                key: "Aggression",
                anchors: [9.6, 10.1, 11.1, 11.6],
            },
            AttributeProfile {
                key: "Agility",
                anchors: [12.2, 12.9, 14.4, 16.2],
            },
            AttributeProfile {
                key: "NaturalFitness",
                anchors: [12.0, 12.2, 13.1, 14.4],
            },
            AttributeProfile {
                key: "Determination",
                anchors: [11.1, 12.1, 13.4, 15.2],
            },
            AttributeProfile {
                key: "Composure",
                anchors: [8.4, 10.7, 12.3, 14.5],
            },
            AttributeProfile {
                key: "Concentration",
                anchors: [6.9, 9.6, 11.1, 12.9],
            },
        ],
    },
];
