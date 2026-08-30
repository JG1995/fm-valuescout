export const POSITION_ORDER = [
  "GK",
  "SW",
  "DR",
  "DC",
  "DL",
  "WBR",
  "DM",
  "WBL",
  "MR",
  "MC",
  "ML",
  "AMR",
  "AMC",
  "AML",
  "ST",
] as const;

export const TACTIC_POSITION_ORDER = [
  "GK",
  "DR",
  "DCR",
  "DC",
  "DCL",
  "DL",
  "WBR",
  "DMCR",
  "DM",
  "DMCL",
  "WBL",
  "MR",
  "MCR",
  "MC",
  "MCL",
  "ML",
  "AMR",
  "AMCR",
  "AMC",
  "AMCL",
  "AML",
  "STCR",
  "STC",
  "STCL",
] as const;

const positionRank = new Map<string, number>(
  POSITION_ORDER.map((position, index) => [position, index]),
);
const tacticPositionRank = new Map<string, number>(
  TACTIC_POSITION_ORDER.map((position, index) => [position, index]),
);

function compareByRank(
  left: string,
  right: string,
  ranks: ReadonlyMap<string, number>,
): number {
  const leftRank = ranks.get(left);
  const rightRank = ranks.get(right);
  if (leftRank === undefined) {
    return rightRank === undefined ? left.localeCompare(right) : 1;
  }
  return rightRank === undefined ? -1 : leftRank - rightRank;
}

export function comparePositions(left: string, right: string): number {
  return compareByRank(left, right, positionRank);
}

export function compareTacticPositions(left: string, right: string): number {
  return compareByRank(left, right, tacticPositionRank);
}

export function orderedPositions<T extends string>(
  positions: readonly T[],
): T[] {
  return [...positions].sort(comparePositions);
}

export function orderedTacticPositions<T extends string>(
  positions: readonly T[],
): T[] {
  return [...positions].sort(compareTacticPositions);
}
