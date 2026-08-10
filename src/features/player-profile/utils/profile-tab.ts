export const PROFILE_TABS = [
  "technical",
  "mental",
  "physical",
  "goalkeeping",
  "hidden",
  "personality",
] as const;

export type ProfileTab = (typeof PROFILE_TABS)[number];

export function parseProfileTab(value: unknown): ProfileTab {
  return typeof value === "string" &&
    (PROFILE_TABS as readonly string[]).includes(value)
    ? (value as ProfileTab)
    : "technical";
}
