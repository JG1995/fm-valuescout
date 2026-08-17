import type { KeyboardEvent } from "react";
import { cn } from "@/utils/cn";
import type { StaffView } from "../utils/staff-url-search";

export const STAFF_VIEWS = [
  "search",
  "my-staff",
  "shortlist",
] as const satisfies readonly StaffView[];

const VIEW_LABELS: Record<StaffView, string> = {
  search: "Search",
  "my-staff": "My Staff",
  shortlist: "Shortlist",
};

function focusStaffTab(view: StaffView) {
  document.getElementById(`staff-workspace-tab-${view}`)?.focus();
}

export function StaffWorkspaceTabs({
  view,
  onViewChange,
}: {
  view: StaffView;
  onViewChange: (view: StaffView) => void;
}) {
  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const index = STAFF_VIEWS.indexOf(view);
    let nextIndex = index;
    if (event.key === "ArrowRight" || event.key === "ArrowDown") {
      nextIndex = (index + 1) % STAFF_VIEWS.length;
    } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
      nextIndex = (index - 1 + STAFF_VIEWS.length) % STAFF_VIEWS.length;
    } else if (event.key === "Home") {
      nextIndex = 0;
    } else if (event.key === "End") {
      nextIndex = STAFF_VIEWS.length - 1;
    } else {
      return;
    }
    event.preventDefault();
    const nextView = STAFF_VIEWS[nextIndex];
    onViewChange(nextView);
    focusStaffTab(nextView);
  };

  return (
    <div
      role="tablist"
      aria-label="Staff workspaces"
      className="inline-flex rounded-full bg-surface-container-high p-0.5"
      onKeyDown={onKeyDown}
    >
      {STAFF_VIEWS.map((id) => {
        const selected = id === view;
        return (
          <button
            key={id}
            type="button"
            role="tab"
            id={`staff-workspace-tab-${id}`}
            aria-selected={selected}
            aria-controls={`staff-workspace-panel-${id}`}
            tabIndex={selected ? 0 : -1}
            className={cn(
              "cursor-pointer rounded-full px-4 py-1.5 text-label-lg transition-colors duration-150 ease-out",
              "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary",
              selected
                ? "bg-primary text-on-primary"
                : "text-on-surface-variant hover:text-on-surface",
            )}
            onClick={() => onViewChange(id)}
          >
            {VIEW_LABELS[id]}
          </button>
        );
      })}
    </div>
  );
}

export function staffWorkspacePanelProps(
  view: StaffView,
  activeView: StaffView,
) {
  return {
    id: `staff-workspace-panel-${view}`,
    role: "tabpanel" as const,
    "aria-labelledby": `staff-workspace-tab-${view}`,
    hidden: view !== activeView,
    className: "flex min-h-0 flex-1 flex-col",
  };
}
