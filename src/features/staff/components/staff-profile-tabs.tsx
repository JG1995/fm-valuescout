import type { KeyboardEvent } from "react";
import { cn } from "@/utils/cn";
import {
  STAFF_PROFILE_TAB_LABELS,
  type StaffProfileTab,
} from "../utils/staff-profile-tab";

export function StaffProfileTabs({
  tab,
  onTabChange,
}: {
  tab: StaffProfileTab;
  onTabChange: (tab: StaffProfileTab) => void;
}) {
  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const tabs = Object.keys(STAFF_PROFILE_TAB_LABELS) as StaffProfileTab[];
    const index = tabs.indexOf(tab);
    let nextIndex = index;
    if (event.key === "ArrowRight" || event.key === "ArrowDown") {
      nextIndex = (index + 1) % tabs.length;
    } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
      nextIndex = (index - 1 + tabs.length) % tabs.length;
    } else if (event.key === "Home") {
      nextIndex = 0;
    } else if (event.key === "End") {
      nextIndex = tabs.length - 1;
    } else {
      return;
    }
    event.preventDefault();
    const next = tabs[nextIndex];
    onTabChange(next);
    document.getElementById(`staff-profile-tab-${next}`)?.focus();
  };

  return (
    <div
      role="tablist"
      aria-label="Staff attribute groups"
      className="inline-flex max-w-full rounded-full bg-surface-container-high p-0.5"
      onKeyDown={onKeyDown}
    >
      {(Object.keys(STAFF_PROFILE_TAB_LABELS) as StaffProfileTab[]).map(
        (id) => (
          <button
            key={id}
            type="button"
            role="tab"
            id={`staff-profile-tab-${id}`}
            aria-selected={id === tab}
            aria-controls={`staff-profile-panel-${id}`}
            tabIndex={id === tab ? 0 : -1}
            className={cn(
              "cursor-pointer rounded-full px-3 py-1.5 text-label-md transition-colors duration-150 ease-out",
              "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary",
              id === tab
                ? "bg-primary text-on-primary"
                : "text-on-surface-variant hover:text-on-surface",
            )}
            onClick={() => onTabChange(id)}
          >
            {STAFF_PROFILE_TAB_LABELS[id]}
          </button>
        ),
      )}
    </div>
  );
}

export function staffProfileTabPanelProps(
  tab: StaffProfileTab,
  active: StaffProfileTab,
) {
  return {
    id: `staff-profile-panel-${tab}`,
    role: "tabpanel" as const,
    "aria-labelledby": `staff-profile-tab-${tab}`,
    hidden: tab !== active,
  };
}
