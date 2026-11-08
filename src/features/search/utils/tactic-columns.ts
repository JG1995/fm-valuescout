import { compareTacticPositions } from "@/utils/position-order";
import { type TacticColumnGroup, tacticColumnId } from "@/utils/tactic-ids";

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

/** Interleaves the active groups per lane, in the order the view lists them. */
export function buildTacticColumnOrder(
  orderedLaneIds: readonly string[],
  activeGroups: readonly TacticColumnGroup[],
): string[] {
  return orderedLaneIds.flatMap((laneId) =>
    activeGroups.flatMap((group) => tacticColumnId(group, laneId) ?? []),
  );
}
