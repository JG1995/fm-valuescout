import { comparePositions } from "@/utils/position-order";

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
      return comparePositions(leftPosition, rightPosition);
    })
    .map(([position]) => position);
}
