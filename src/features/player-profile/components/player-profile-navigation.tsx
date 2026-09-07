import { type KeyboardEvent, useEffect } from "react";
import {
  PROFILE_SECTIONS,
  type ProfileSection,
} from "../utils/profile-section";

export type PlayerProfileView = "general" | "moneyball";

const SECTION_LABELS: Record<ProfileSection, string> = {
  overview: "Overview",
  attributes: "Attributes",
  "role-fit": "Role Fit",
  moneyball: "Moneyball",
};

export function parsePlayerProfileView(
  value: unknown,
): PlayerProfileView | undefined {
  if (value === "moneyball" || value === "general") return value;
  return undefined;
}

export function PlayerSectionTabs({
  section,
  onSectionChange,
  restoreFocus,
  onFocusRestored,
}: {
  section: ProfileSection;
  onSectionChange: (section: ProfileSection, restoreFocus?: boolean) => void;
  restoreFocus: boolean;
  onFocusRestored: () => void;
}) {
  useEffect(() => {
    if (!restoreFocus) return;
    document.getElementById(`player-analysis-tab-${section}`)?.focus();
    onFocusRestored();
  }, [onFocusRestored, restoreFocus, section]);

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const index = PROFILE_SECTIONS.indexOf(section);
    let nextIndex = index;
    if (event.key === "ArrowRight" || event.key === "ArrowDown") {
      nextIndex = (index + 1) % PROFILE_SECTIONS.length;
    } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
      nextIndex =
        (index - 1 + PROFILE_SECTIONS.length) % PROFILE_SECTIONS.length;
    } else if (event.key === "Home") {
      nextIndex = 0;
    } else if (event.key === "End") {
      nextIndex = PROFILE_SECTIONS.length - 1;
    } else {
      return;
    }

    event.preventDefault();
    const next = PROFILE_SECTIONS[nextIndex];
    onSectionChange(next, true);
  };

  return (
    <div
      role="tablist"
      aria-label="Player analysis view"
      className="inline-flex rounded-full bg-surface-container-high p-0.5"
      onKeyDown={onKeyDown}
    >
      {PROFILE_SECTIONS.map((candidate) => {
        const selected = candidate === section;
        return (
          <button
            key={candidate}
            id={`player-analysis-tab-${candidate}`}
            type="button"
            role="tab"
            aria-selected={selected}
            aria-controls="player-analysis-panel"
            tabIndex={selected ? 0 : -1}
            className={
              selected
                ? "cursor-pointer rounded-full bg-primary px-3 py-1.5 text-label-md text-on-primary"
                : "cursor-pointer rounded-full px-3 py-1.5 text-label-md text-on-surface-variant hover:text-on-surface"
            }
            onClick={() => onSectionChange(candidate, true)}
          >
            {SECTION_LABELS[candidate]}
          </button>
        );
      })}
    </div>
  );
}
