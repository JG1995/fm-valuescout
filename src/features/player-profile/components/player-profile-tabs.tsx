import type { KeyboardEvent } from "react";
import { cn } from "@/utils/cn";
import type { ProfileTab } from "../utils/profile-tab";
import { PROFILE_TABS } from "../utils/profile-tab";

const TAB_LABELS: Record<ProfileTab, string> = {
  technical: "Technical",
  mental: "Mental",
  physical: "Physical",
  goalkeeping: "GK",
  hidden: "Hidden",
  personality: "Personality",
};

type PlayerProfileTabsProps = {
  tab: ProfileTab;
  onTabChange: (tab: ProfileTab) => void;
};

function focusTabButton(id: ProfileTab) {
  document.getElementById(`profile-tab-${id}`)?.focus();
}

export function PlayerProfileTabs({
  tab,
  onTabChange,
}: PlayerProfileTabsProps) {
  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const index = PROFILE_TABS.indexOf(tab);
    if (index < 0) {
      return;
    }

    let nextIndex = index;
    if (event.key === "ArrowRight" || event.key === "ArrowDown") {
      nextIndex = (index + 1) % PROFILE_TABS.length;
    } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
      nextIndex = (index - 1 + PROFILE_TABS.length) % PROFILE_TABS.length;
    } else if (event.key === "Home") {
      nextIndex = 0;
    } else if (event.key === "End") {
      nextIndex = PROFILE_TABS.length - 1;
    } else {
      return;
    }

    event.preventDefault();
    const next = PROFILE_TABS[nextIndex];
    onTabChange(next);
    focusTabButton(next);
  };

  return (
    <div
      role="tablist"
      aria-label="Attribute groups"
      className="inline-flex max-w-full rounded-full bg-surface-container-high p-0.5"
      onKeyDown={onKeyDown}
    >
      {PROFILE_TABS.map((id) => {
        const selected = id === tab;
        return (
          <button
            key={id}
            type="button"
            role="tab"
            id={`profile-tab-${id}`}
            aria-selected={selected}
            aria-controls={`profile-panel-${id}`}
            tabIndex={selected ? 0 : -1}
            className={cn(
              "cursor-pointer rounded-full px-3 py-1.5 text-label-md transition-colors duration-150 ease-out",
              "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary",
              selected
                ? "bg-primary text-on-primary"
                : "text-on-surface-variant hover:text-on-surface",
            )}
            onClick={() => {
              onTabChange(id);
            }}
          >
            {TAB_LABELS[id]}
          </button>
        );
      })}
    </div>
  );
}

export function profileTabPanelProps(tab: ProfileTab, active: ProfileTab) {
  return {
    id: `profile-panel-${tab}`,
    role: "tabpanel" as const,
    "aria-labelledby": `profile-tab-${tab}`,
    hidden: tab !== active,
  };
}
