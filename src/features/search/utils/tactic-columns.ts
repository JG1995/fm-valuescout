import { compareTacticPositions } from "@/utils/position-order";
import { allTacticColumnIdsForGroup, tacticColumnId } from "@/utils/tactic-ids";

export function orderedLaneIdsForColumns(
  lanes: readonly {
    laneId: string;
    ipPosition: string;
    oopPosition: string;
  }[],
): string[] {
  return [...lanes]
    .sort(
      (left, right) =>
        compareTacticPositions(left.ipPosition, right.ipPosition) ||
        compareTacticPositions(left.oopPosition, right.oopPosition),
    )
    .map((lane) => lane.laneId);
}

export function buildTacticColumnOrder(
  orderedLaneIds: readonly string[],
  currentActive: boolean,
  potentialActive: boolean,
): string[] {
  if (currentActive && potentialActive) {
    return orderedLaneIds.flatMap((laneId) => {
      const currentId = tacticColumnId("current", laneId);
      const potentialId = tacticColumnId("potential", laneId);
      const ids: string[] = [];
      if (currentId !== null) {
        ids.push(currentId);
      }
      if (potentialId !== null) {
        ids.push(potentialId);
      }
      return ids;
    });
  }
  if (currentActive) {
    return allTacticColumnIdsForGroup("current", orderedLaneIds);
  }
  if (potentialActive) {
    return allTacticColumnIdsForGroup("potential", orderedLaneIds);
  }
  return [];
}
