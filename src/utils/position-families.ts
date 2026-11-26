/**
 * The position families Moneyball ranks statistics within. A family's peers
 * are the players with 450+ minutes this season rated 15+ at one of its
 * positions. IDs match the Rust role catalog's `position_family`.
 */
export const POSITION_FAMILIES = [
  { id: "goalkeeper", label: "Goalkeepers", positions: ["GK"] },
  { id: "central_defender", label: "Centre-backs", positions: ["DC"] },
  { id: "full_back", label: "Full-backs", positions: ["DL", "DR"] },
  { id: "wing_back", label: "Wing-backs", positions: ["WBL", "WBR"] },
  {
    id: "defensive_midfielder",
    label: "Defensive midfielders",
    positions: ["DM"],
  },
  { id: "central_midfielder", label: "Central midfielders", positions: ["MC"] },
  { id: "wide_midfielder", label: "Wide midfielders", positions: ["ML", "MR"] },
  {
    id: "attacking_midfielder",
    label: "Attacking midfielders",
    positions: ["AMC"],
  },
  { id: "winger", label: "Wingers", positions: ["AML", "AMR"] },
  { id: "striker", label: "Strikers", positions: ["ST"] },
] as const;

export type PositionFamilyId = (typeof POSITION_FAMILIES)[number]["id"];

export function isPositionFamilyId(value: unknown): value is PositionFamilyId {
  return POSITION_FAMILIES.some((family) => family.id === value);
}

/** The family's plural label, such as "Centre-backs", or the ID when unknown. */
export function positionFamilyLabel(id: string): string {
  return POSITION_FAMILIES.find((family) => family.id === id)?.label ?? id;
}

/** The family of a base position (`DC`, not `DCR`), if it has one. */
export function positionFamilyOf(
  basePosition: string,
): PositionFamilyId | null {
  return (
    POSITION_FAMILIES.find((family) =>
      (family.positions as readonly string[]).includes(basePosition),
    )?.id ?? null
  );
}
