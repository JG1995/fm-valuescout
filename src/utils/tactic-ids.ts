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

export type TacticColumnGroup = "current" | "potential";

export type TacticLaneId = (typeof TACTIC_LANE_IDS)[number];

const tacticLaneIds = new Set<string>(TACTIC_LANE_IDS);

export function isValidTacticLaneId(laneId: string): laneId is TacticLaneId {
  return tacticLaneIds.has(laneId);
}

export function isTacticColumnId(id: string): boolean {
  return (
    id.startsWith(TACTIC_CURRENT_PREFIX) ||
    id.startsWith(TACTIC_POTENTIAL_PREFIX)
  );
}

export function tacticGroupForId(id: string): TacticColumnGroup | null {
  if (id.startsWith(TACTIC_CURRENT_PREFIX)) {
    return "current";
  }
  if (id.startsWith(TACTIC_POTENTIAL_PREFIX)) {
    return "potential";
  }
  return null;
}

export function tacticLaneIdForId(id: string): string | null {
  const group = tacticGroupForId(id);
  if (group === null) {
    return null;
  }
  const prefix =
    group === "current" ? TACTIC_CURRENT_PREFIX : TACTIC_POTENTIAL_PREFIX;
  return id.slice(prefix.length);
}

export function tacticColumnId(
  group: TacticColumnGroup,
  laneId: string,
): string | null {
  if (!tacticLaneIds.has(laneId)) {
    return null;
  }
  const prefix =
    group === "current" ? TACTIC_CURRENT_PREFIX : TACTIC_POTENTIAL_PREFIX;
  return `${prefix}${laneId}`;
}

export function isValidTacticColumnId(id: string): boolean {
  const laneId = tacticLaneIdForId(id);
  return laneId !== null && tacticLaneIds.has(laneId);
}

export function allTacticColumnIdsForGroup(
  group: TacticColumnGroup,
  orderedLaneIds: readonly string[],
): string[] {
  const ids: string[] = [];
  for (const laneId of orderedLaneIds) {
    const columnId = tacticColumnId(group, laneId);
    if (columnId !== null) {
      ids.push(columnId);
    }
  }
  return ids;
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
