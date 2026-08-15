export const PROFILE_TABS = [
  "outfield",
  "goalkeeping",
  "hidden",
  "personality",
] as const;

export type ProfileTab = (typeof PROFILE_TABS)[number];

const LEGACY_OUTFIELD_TABS = ["technical", "mental", "physical"] as const;

export function parseProfileTab(value: unknown): ProfileTab {
  if (
    typeof value === "string" &&
    (value === "outfield" ||
      (LEGACY_OUTFIELD_TABS as readonly string[]).includes(value))
  ) {
    return "outfield";
  }
  return typeof value === "string" &&
    (PROFILE_TABS as readonly string[]).includes(value)
    ? (value as ProfileTab)
    : "outfield";
}
