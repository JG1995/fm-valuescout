/**
 * Moneyball mapping contract (deterministic, see ledger Invariants):
 * 88 Moneyball defs (all mapped via attribute_role_id, 0 unmapped),
 * 119/129 General (role_id, base_position) combos mapped via (attribute_role_id, base_position(placement)),
 * 10 uncovered (NULL -> "—"): holding_wing_back_oop+DL,
 * holding_wing_back_oop+DR, pressing_wing_back_oop+DL, pressing_wing_back_oop+DR,
 * box_to_box_midfielder_ip+MC, box_to_box_playmaker_ip+MC, deep_lying_playmaker_ip+MC,
 * second_striker_ip+ST, wing_back_oop+DL, wing_back_oop+DR (generic wing_back_oop
 * retains DL/DR/WBL/WBR while its presentation row remains WBL/WBR). The Channel
 * Midfielder MC tag correction is already covered by existing Moneyball rows.
 */
export const TACTIC_CURRENT_PREFIX = "tactic_current.";
export const TACTIC_POTENTIAL_PREFIX = "tactic_potential.";
export const TACTIC_CUSTOM_PREFIX = "tactic_custom.";
export const TACTIC_COLUMN_DEFAULT_WIDTH = 112;

export const TACTIC_LANE_IDS = [
  "goalkeeper",
  "left_back",
  "left_centre_back",
  "right_centre_back",
  "right_back",
  "defensive_midfielder",
  "left_central_midfielder",
  "right_central_midfielder",
  "left_winger",
  "right_winger",
  "centre_forward",
] as const;

export type TacticColumnGroup = "current" | "potential" | "custom";

export type TacticLaneId = (typeof TACTIC_LANE_IDS)[number];

const tacticLaneIds = new Set<string>(TACTIC_LANE_IDS);

const TACTIC_GROUP_PREFIXES: Record<TacticColumnGroup, string> = {
  current: TACTIC_CURRENT_PREFIX,
  potential: TACTIC_POTENTIAL_PREFIX,
  custom: TACTIC_CUSTOM_PREFIX,
};

const TACTIC_GROUPS = Object.keys(
  TACTIC_GROUP_PREFIXES,
) as readonly TacticColumnGroup[];

/**
 * Tactic groups each table offers, in view order. Moneyball has no potential
 * statistics, so it offers Custom in place of Potential; Rust rejects each
 * group in the other view.
 */
export function tacticColumnGroupsForTable(
  table: string,
): readonly TacticColumnGroup[] {
  if (table === "search") {
    return ["current", "potential"];
  }
  if (table === "moneyball-search") {
    return ["current", "custom"];
  }
  return [];
}

export function isValidTacticLaneId(laneId: string): laneId is TacticLaneId {
  return tacticLaneIds.has(laneId);
}

export function isTacticColumnId(id: string): boolean {
  return tacticGroupForId(id) !== null;
}

export function tacticGroupForId(id: string): TacticColumnGroup | null {
  return (
    TACTIC_GROUPS.find((group) =>
      id.startsWith(TACTIC_GROUP_PREFIXES[group]),
    ) ?? null
  );
}

export function tacticLaneIdForId(id: string): string | null {
  const group = tacticGroupForId(id);
  return group === null ? null : id.slice(TACTIC_GROUP_PREFIXES[group].length);
}

export function tacticColumnId(
  group: TacticColumnGroup,
  laneId: string,
): string | null {
  if (!tacticLaneIds.has(laneId)) {
    return null;
  }
  return `${TACTIC_GROUP_PREFIXES[group]}${laneId}`;
}

/** Accepts every group, for header grouping and sort validation. */
export function isValidTacticColumnId(id: string): boolean {
  const laneId = tacticLaneIdForId(id);
  return laneId !== null && tacticLaneIds.has(laneId);
}

export function isValidTacticColumnIdForTable(
  id: string,
  table: string,
): boolean {
  const group = tacticGroupForId(id);
  return (
    group !== null &&
    isValidTacticColumnId(id) &&
    tacticColumnGroupsForTable(table).includes(group)
  );
}

export function isFullTacticGroup(
  columnIds: readonly string[],
  group: TacticColumnGroup,
): boolean {
  const ids = new Set(columnIds);
  return TACTIC_LANE_IDS.every((laneId) => {
    const columnId = tacticColumnId(group, laneId);
    return columnId !== null && ids.has(columnId);
  });
}

export function sanitizeTacticIds(columnIds: readonly string[]): string[] {
  return columnIds.filter(isValidTacticColumnId);
}
