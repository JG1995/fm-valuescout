export const STAFF_PROFILE_TABS = ["coaching", "mental", "knowledge"] as const;

export type StaffProfileTab = (typeof STAFF_PROFILE_TABS)[number];

export const STAFF_PROFILE_TAB_LABELS: Record<StaffProfileTab, string> = {
  coaching: "Coaching",
  mental: "Mental",
  knowledge: "Knowledge",
};

export function parseStaffProfileTab(
  value: unknown,
): StaffProfileTab | undefined {
  return typeof value === "string" &&
    (STAFF_PROFILE_TABS as readonly string[]).includes(value)
    ? (value as StaffProfileTab)
    : undefined;
}

export function defaultStaffProfileTab(): StaffProfileTab {
  return "coaching";
}
