export type StaffProfileAttributeGroup = {
  id: "coaching" | "mental" | "knowledge";
  title: string;
  keys: readonly string[];
};

export const STAFF_PROFILE_ATTRIBUTE_GROUPS: readonly StaffProfileAttributeGroup[] =
  [
    {
      id: "coaching",
      title: "Coaching",
      keys: [
        "Attacking",
        "Defending",
        "Fitness",
        "GoalkeepingDistribution",
        "GoalkeepingHandling",
        "GoalkeepingReflexes",
        "Possession",
        "SetPieces",
        "Tactical",
        "Technical",
      ],
    },
    {
      id: "mental",
      title: "Mental",
      keys: [
        "Adaptability",
        "Authority",
        "Determination",
        "ManManagement",
        "Motivating",
        "WorkingWithYoungsters",
      ],
    },
    {
      id: "knowledge",
      title: "Knowledge",
      keys: [
        "DataAnalysis",
        "JudgingPlayerAbility",
        "JudgingPlayerPotential",
        "JudgingStaffAbility",
        "Negotiating",
        "Physiotherapy",
        "SportsScience",
        "TacticalKnowledge",
      ],
    },
  ] as const;
