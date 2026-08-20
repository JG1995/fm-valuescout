import { describe, expect, it } from "vitest";
import {
  MONEYBALL_METRIC_CATEGORIES,
  MONEYBALL_METRICS,
} from "./moneyball-metrics";

const CANONICAL_MONEYBALL_METRIC_IDS = [
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
  "headers_attempted",
  "headers_attempted_per_90",
  "headers_won",
  "headers_won_per_90",
  "headers_lost",
  "headers_lost_per_90",
  "headers_won_ratio",
  "key_headers",
  "key_headers_per_90",
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
  "average_rating",
  "player_of_the_match",
  "player_of_the_match_per_90",
  "games_won",
  "games_drawn",
  "games_lost",
  "game_win_ratio",
  "team_goals",
  "team_goals_per_90",
] as const;

describe("Moneyball metric presentation catalogue", () => {
  it("covers the canonical 138 performance IDs across eight non-empty categories", () => {
    expect(MONEYBALL_METRICS).toHaveLength(
      CANONICAL_MONEYBALL_METRIC_IDS.length,
    );
    expect(new Set(MONEYBALL_METRICS.map((metric) => metric.id)).size).toBe(
      CANONICAL_MONEYBALL_METRIC_IDS.length,
    );
    expect(MONEYBALL_METRIC_CATEGORIES).toHaveLength(8);
    expect(
      MONEYBALL_METRIC_CATEGORIES.every(
        (category) => category.metricIds.length > 0,
      ),
    ).toBe(true);
    expect(MONEYBALL_METRICS.map((metric) => metric.id)).toEqual(
      CANONICAL_MONEYBALL_METRIC_IDS,
    );
  });

  it("declares raw presentation metadata without colliding with General attribute IDs", () => {
    const goalsPerNinety = MONEYBALL_METRICS.find(
      (metric) => metric.id === "goals_per_90",
    );
    const saveRatio = MONEYBALL_METRICS.find(
      (metric) => metric.id === "save_ratio",
    );
    const distance = MONEYBALL_METRICS.find(
      (metric) => metric.id === "distance_covered",
    );

    expect(goalsPerNinety).toMatchObject({
      label: "Goals / 90",
      kind: "decimal",
      precision: 2,
    });
    expect(saveRatio).toMatchObject({ kind: "percentage", precision: 1 });
    expect(distance).toMatchObject({ kind: "distance", precision: 1 });
    expect(
      MONEYBALL_METRICS.some((metric) => /^[A-Z][A-Za-z]*$/.test(metric.id)),
    ).toBe(false);
  });
});
