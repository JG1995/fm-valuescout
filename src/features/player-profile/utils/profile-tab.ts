export const PROFILE_TABS = ["overview", "attributes", "roles"] as const;

export type ProfileTab = (typeof PROFILE_TABS)[number];

export function parseProfileTab(value: unknown): ProfileTab {
  return typeof value === "string" &&
    (PROFILE_TABS as readonly string[]).includes(value)
    ? (value as ProfileTab)
    : "overview";
}
