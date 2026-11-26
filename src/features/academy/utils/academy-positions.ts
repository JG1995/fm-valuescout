import { comparePositions } from "@/utils/position-order";
import { PLAYABLE_POSITION_FAMILIARITY } from "@/utils/profile-position-roles";

/** Return positions the player can play in the Youth Academy. */
export function playableAcademyPositions(
  positions: Readonly<Record<string, number | null>>,
): string[] {
  return Object.entries(positions)
    .filter(
      (entry): entry is [string, number] =>
        typeof entry[1] === "number" &&
        Number.isFinite(entry[1]) &&
        entry[1] >= PLAYABLE_POSITION_FAMILIARITY,
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
