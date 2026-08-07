import type { KeyboardEvent } from "react";
import { cn } from "@/utils/cn";
import { ACADEMY_VIEWS, type AcademyView } from "../types/academy";

const VIEW_LABELS: Record<AcademyView, string> = {
  overview: "Overview",
  graduates: "Graduates",
  class: "Class",
};

function focusWorkspaceTab(view: AcademyView) {
  document.getElementById(`academy-workspace-tab-${view}`)?.focus();
}

type AcademyWorkspaceTabsProps = {
  view: AcademyView;
  onViewChange: (view: AcademyView) => void;
};

export function AcademyWorkspaceTabs({
  view,
  onViewChange,
}: AcademyWorkspaceTabsProps) {
  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const index = ACADEMY_VIEWS.indexOf(view);
    let nextIndex = index;

    if (event.key === "ArrowRight" || event.key === "ArrowDown") {
      nextIndex = (index + 1) % ACADEMY_VIEWS.length;
    } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
      nextIndex = (index - 1 + ACADEMY_VIEWS.length) % ACADEMY_VIEWS.length;
    } else if (event.key === "Home") {
      nextIndex = 0;
    } else if (event.key === "End") {
      nextIndex = ACADEMY_VIEWS.length - 1;
    } else {
      return;
    }

    event.preventDefault();
    const nextView = ACADEMY_VIEWS[nextIndex];
    onViewChange(nextView);
    focusWorkspaceTab(nextView);
  };

  return (
    <div
      role="tablist"
      aria-label="Youth Academy workspaces"
      className="inline-flex rounded-full bg-surface-container-high p-0.5"
      onKeyDown={onKeyDown}
    >
      {ACADEMY_VIEWS.map((id) => {
        const selected = id === view;
        return (
          <button
            key={id}
            type="button"
            role="tab"
            id={`academy-workspace-tab-${id}`}
            aria-selected={selected}
            aria-controls={`academy-workspace-panel-${id}`}
            tabIndex={selected ? 0 : -1}
            className={cn(
              "cursor-pointer rounded-full px-4 py-1.5 text-label-lg transition-colors duration-150 ease-out",
              "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary",
              selected
                ? "bg-primary text-on-primary"
                : "text-on-surface-variant hover:text-on-surface",
            )}
            onClick={() => {
              onViewChange(id);
            }}
          >
            {VIEW_LABELS[id]}
          </button>
        );
      })}
    </div>
  );
}

export function academyWorkspacePanelProps(
  view: AcademyView,
  activeView: AcademyView,
) {
  return {
    id: `academy-workspace-panel-${view}`,
    role: "tabpanel" as const,
    "aria-labelledby": `academy-workspace-tab-${view}`,
    hidden: view !== activeView,
  };
}
