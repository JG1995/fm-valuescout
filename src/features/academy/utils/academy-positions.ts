const ACADEMY_POSITION_ORDER = [
  "GK",
  "SW",
  "DL",
  "DC",
  "DR",
  "DM",
  "ML",
  "MC",
  "MR",
  "AML",
  "AMC",
  "AMR",
  "ST",
  "WBL",
  "WBR",
] as const;

const positionOrder = new Map<string, number>(
  ACADEMY_POSITION_ORDER.map((position, index) => [position, index]),
);

/** Return recorded Academy positions without promoting zero or unread slots. */
export function recordedAcademyPositions(
  positions: Readonly<Record<string, number | null>>,
): string[] {
  return Object.entries(positions)
    .filter(
      (entry): entry is [string, number] =>
        typeof entry[1] === "number" &&
        Number.isFinite(entry[1]) &&
        entry[1] > 0,
    )
    .sort(([leftPosition, leftValue], [rightPosition, rightValue]) => {
      const familiarity = rightValue - leftValue;
      if (familiarity !== 0) {
        return familiarity;
      }
      return (
        (positionOrder.get(leftPosition) ?? Number.MAX_SAFE_INTEGER) -
          (positionOrder.get(rightPosition) ?? Number.MAX_SAFE_INTEGER) ||
        leftPosition.localeCompare(rightPosition)
      );
    })
    .map(([position]) => position);
}
