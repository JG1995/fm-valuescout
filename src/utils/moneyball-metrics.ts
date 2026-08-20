export type MoneyballMetricKind =
  | "count"
  | "decimal"
  | "percentage"
  | "distance"
  | "rating";

export type MoneyballMetricWidth = "compact" | "normal" | "wide";

export type MoneyballMetric = {
  id: string;
  categoryId: string;
  label: string;
  kind: MoneyballMetricKind;
  precision: number;
  percentageScale?: "fraction" | "percent";
  width: MoneyballMetricWidth;
};

export type MoneyballMetricCategory = {
  id: string;
  title: string;
  metricIds: readonly string[];
};

export const MONEYBALL_METRIC_CATEGORIES: readonly MoneyballMetricCategory[] = [
  {
    id: "shooting",
    title: "Shooting",
    metricIds: [
      "goals",
      "goals_per_90",
      "goals_from_outside_the_box",
      "goals_from_outside_the_box_per_90",
      "xg",
      "xg_per_90",
      "np-xg",
      "np-xg_per_90",
      "xg-op",
      "xg-op_per_90",
      "xg_per_shot",
      "shots",
      "shots_per_90",
      "shots_from_outside_the_box",
      "shots_from_outside_the_box_per_90",
      "shots_on_target",
      "shots_on_target_per_90",
      "penalties_taken",
      "penalties_taken_per_90",
      "penalties_scored",
      "penalties_scored_per_90",
      "penalties_scored_ratio",
      "free_kicks",
      "free_kicks_per_90",
      "minutes_per_goal",
      "minutes_per_goal_or_assist",
    ],
  },
  {
    id: "creation",
    title: "Creation",
    metricIds: [
      "assists",
      "assists_per_90",
      "xa",
      "xa_per_90",
      "chances_created",
      "chances_created_per_90",
      "clear_cut_chances_created",
      "clear_cut_chances_created_per_90",
      "key_passes",
      "key_passes_per_90",
      "open_play_key_passes",
      "open_play_key_passes_per_90",
      "crosses_attempted",
      "crosses_attempted_per_90",
      "crosses_completed",
      "crosses_completed_per_90",
      "cross_completion_ratio",
      "open_play_crosses_attempted",
      "open_play_crosses_attempted_per_90",
      "open_play_crosses_completed",
      "open_play_crosses_completed_per_90",
      "open_play_cross_completion_ratio",
      "minutes_per_assist",
    ],
  },
  {
    id: "possession",
    title: "Possession",
    metricIds: [
      "passes_attempted",
      "passes_attempted_per_90",
      "passes_completed",
      "passes_completed_per_90",
      "pass_completion_ratio",
      "progressive_passes",
      "progressive_passes_per_90",
      "dribbles_made",
      "dribbles_made_per_90",
      "distance_covered",
      "distance_covered_per_90",
      "high_intensity_sprints",
      "high_intensity_sprints_per_90",
      "possession_lost",
      "possession_lost_per_90",
    ],
  },
  {
    id: "defending",
    title: "Defending",
    metricIds: [
      "tackles_attempted",
      "tackles_attempted_per_90",
      "tackles_completed",
      "tackles_completed_per_90",
      "tackle_completion_ratio",
      "key_tackles",
      "key_tackles_per_90",
      "interceptions",
      "interceptions_per_90",
      "possession_won",
      "possession_won_per_90",
      "pressures_attempted",
      "pressures_attempted_per_90",
      "pressures_completed",
      "pressures_completed_per_90",
      "pressure_success_ratio",
      "blocks",
      "blocks_per_90",
      "shots_blocked_defending",
      "shots_blocked_defending_per_90",
      "clearances",
      "clearances_per_90",
    ],
  },
  {
    id: "aerial",
    title: "Aerial",
    metricIds: [
      "headers_attempted",
      "headers_attempted_per_90",
      "headers_won",
      "headers_won_per_90",
      "headers_lost",
      "headers_lost_per_90",
      "headers_won_ratio",
      "key_headers",
      "key_headers_per_90",
    ],
  },
  {
    id: "goalkeeping",
    title: "Goalkeeping",
    metricIds: [
      "clean_sheets",
      "clean_sheets_per_90",
      "clean_sheets_ratio",
      "goals_conceded",
      "goals_conceded_per_90",
      "saves",
      "saves_per_90",
      "save_ratio",
      "expected_save_percentage",
      "xgp",
      "xgp_per_90",
      "saves_held",
      "saves_held_per_90",
      "saves_parried",
      "saves_parried_per_90",
      "saves_tipped",
      "saves_tipped_per_90",
      "penalties_faced",
      "penalties_faced_per_90",
      "penalties_saved",
      "penalties_saved_per_90",
      "penalties_saved_ratio",
    ],
  },
  {
    id: "discipline",
    title: "Discipline",
    metricIds: [
      "fouls_made",
      "fouls_made_per_90",
      "fouls_against",
      "fouls_against_per_90",
      "yellow_cards",
      "yellow_cards_per_90",
      "red_cards",
      "red_cards_per_90",
      "offsides",
      "offsides_per_90",
      "mistakes_leading_to_goal",
      "mistakes_leading_to_goal_per_90",
    ],
  },
  {
    id: "results",
    title: "Results",
    metricIds: [
      "average_rating",
      "player_of_the_match",
      "player_of_the_match_per_90",
      "games_won",
      "games_drawn",
      "games_lost",
      "game_win_ratio",
      "team_goals",
      "team_goals_per_90",
    ],
  },
];

const DECIMAL_IDS = new Set([
  "xg",
  "np-xg",
  "xg-op",
  "xg_per_shot",
  "xa",
  "xgp",
  "minutes_per_goal",
  "minutes_per_goal_or_assist",
  "minutes_per_assist",
]);

const MINUTES_PER_EVENT_IDS = new Set([
  "minutes_per_goal",
  "minutes_per_goal_or_assist",
  "minutes_per_assist",
]);

const PERCENTAGE_IDS = new Set([
  "penalties_scored_ratio",
  "cross_completion_ratio",
  "open_play_cross_completion_ratio",
  "pass_completion_ratio",
  "tackle_completion_ratio",
  "pressure_success_ratio",
  "headers_won_ratio",
  "clean_sheets_ratio",
  "save_ratio",
  "expected_save_percentage",
  "penalties_saved_ratio",
  "game_win_ratio",
]);

const SPECIAL_LABELS: Record<string, string> = {
  "np-xg": "NP-xG",
  "np-xg_per_90": "NP-xG / 90",
  "xg-op": "xG Open Play",
  "xg-op_per_90": "xG Open Play / 90",
  xg: "xG",
  xg_per_90: "xG / 90",
  xg_per_shot: "xG / Shot",
  xa: "xA",
  xa_per_90: "xA / 90",
  xgp: "xGP",
  xgp_per_90: "xGP / 90",
  expected_save_percentage: "Expected Save %",
  average_rating: "Average Rating",
};

function labelFor(id: string) {
  const special = SPECIAL_LABELS[id];
  if (special) return special;
  const title = id
    .replace(/_per_90$/, " / 90")
    .split("_")
    .map((word) => `${word.slice(0, 1).toUpperCase()}${word.slice(1)}`)
    .join(" ");
  return title.replace("Ratio", "%");
}

function definition(categoryId: string, id: string): MoneyballMetric {
  const kind: MoneyballMetricKind = PERCENTAGE_IDS.has(id)
    ? "percentage"
    : id.includes("distance_covered")
      ? "distance"
      : id === "average_rating"
        ? "rating"
        : id.endsWith("_per_90") || DECIMAL_IDS.has(id)
          ? "decimal"
          : "count";
  const precision = MINUTES_PER_EVENT_IDS.has(id)
    ? 1
    : kind === "count"
      ? 0
      : kind === "rating"
        ? 2
        : kind === "percentage"
          ? 1
          : kind === "distance"
            ? 1
            : 2;
  return {
    id,
    categoryId,
    label: labelFor(id),
    kind,
    precision,
    percentageScale:
      kind === "percentage"
        ? id === "expected_save_percentage"
          ? "percent"
          : "fraction"
        : undefined,
    width: kind === "count" ? "compact" : id.length > 24 ? "wide" : "normal",
  };
}

export const MONEYBALL_METRICS: readonly MoneyballMetric[] =
  MONEYBALL_METRIC_CATEGORIES.flatMap((category) =>
    category.metricIds.map((id) => definition(category.id, id)),
  );

export function moneyballMetric(id: string): MoneyballMetric {
  const metric = MONEYBALL_METRICS.find((candidate) => candidate.id === id);
  if (!metric) throw new Error(`Unknown Moneyball metric: ${id}`);
  return metric;
}
