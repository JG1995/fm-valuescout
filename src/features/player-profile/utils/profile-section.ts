import type { PlayerProfileView } from "../components/player-profile-navigation";
import type { ProfileTab } from "./profile-tab";

export const PROFILE_SECTIONS = [
  "overview",
  "attributes",
  "role-fit",
  "moneyball",
] as const;

export type ProfileSection = (typeof PROFILE_SECTIONS)[number];

export function parseProfileSection(
  value: unknown,
): ProfileSection | undefined {
  return typeof value === "string" &&
    (PROFILE_SECTIONS as readonly string[]).includes(value)
    ? (value as ProfileSection)
    : undefined;
}

export function isStandardProfileSection(section: ProfileSection): boolean {
  return section !== "moneyball";
}

export function resolveProfileSection({
  section,
  view,
  tab,
  defaultAnalysisView,
}: {
  section: ProfileSection | undefined;
  view: PlayerProfileView | undefined;
  tab: ProfileTab | undefined;
  defaultAnalysisView: PlayerProfileView;
}): ProfileSection {
  if (section !== undefined) return section;
  if (view === "moneyball") return "moneyball";
  if (view === "general") return tab === undefined ? "overview" : "attributes";
  return defaultAnalysisView === "moneyball" ? "moneyball" : "overview";
}
