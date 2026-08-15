export const PROFILE_TABS = [
  "outfield",
  "goalkeeping",
  "hidden",
  "personality",
] as const;

export const GOALKEEPER_PROFILE_TABS = [
  "goalkeeping",
  "outfield",
  "hidden",
  "personality",
] as const;

export type ProfileTab = (typeof PROFILE_TABS)[number];

const LEGACY_OUTFIELD_TABS = ["technical", "mental", "physical"] as const;

export function parseProfileTab(value: unknown): ProfileTab | undefined {
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
    : undefined;
}

export function profileTabsForPlayer(
  goalkeeper: boolean,
): readonly ProfileTab[] {
  return goalkeeper ? GOALKEEPER_PROFILE_TABS : PROFILE_TABS;
}

export function defaultProfileTab(goalkeeper: boolean): ProfileTab {
  return goalkeeper ? "goalkeeping" : "outfield";
}
