import type { KeyboardEvent } from "react";
import { cn } from "@/utils/cn";

export const PLANNER_WORKSPACES = ["squad", "tactic", "clubs"] as const;
export type PlannerWorkspace = (typeof PLANNER_WORKSPACES)[number];

const WORKSPACE_LABELS: Record<PlannerWorkspace, string> = {
  squad: "Squad",
  tactic: "Tactic",
  clubs: "Club setup",
};

export function parsePlannerWorkspace(raw: unknown): PlannerWorkspace | null {
  return typeof raw === "string" && isPlannerWorkspace(raw) ? raw : null;
}

function isPlannerWorkspace(raw: string): raw is PlannerWorkspace {
  return (PLANNER_WORKSPACES as readonly string[]).includes(raw);
}

function focusWorkspaceTab(workspace: PlannerWorkspace) {
  document.getElementById(`planner-workspace-tab-${workspace}`)?.focus();
}

type PlannerWorkspaceTabsProps = {
  workspace: PlannerWorkspace;
  onWorkspaceChange: (workspace: PlannerWorkspace) => void;
};

export function PlannerWorkspaceTabs({
  workspace,
  onWorkspaceChange,
}: PlannerWorkspaceTabsProps) {
  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const index = PLANNER_WORKSPACES.indexOf(workspace);
    let nextIndex = index;

    if (event.key === "ArrowRight" || event.key === "ArrowDown") {
      nextIndex = (index + 1) % PLANNER_WORKSPACES.length;
    } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
      nextIndex =
        (index - 1 + PLANNER_WORKSPACES.length) % PLANNER_WORKSPACES.length;
    } else if (event.key === "Home") {
      nextIndex = 0;
    } else if (event.key === "End") {
      nextIndex = PLANNER_WORKSPACES.length - 1;
    } else {
      return;
    }

    event.preventDefault();
    const next = PLANNER_WORKSPACES[nextIndex];
    onWorkspaceChange(next);
    focusWorkspaceTab(next);
  };

  return (
    <div
      role="tablist"
      aria-label="Planner workspaces"
      className="inline-flex rounded-full bg-surface-container-high p-0.5"
      onKeyDown={onKeyDown}
    >
      {PLANNER_WORKSPACES.map((id) => {
        const selected = id === workspace;
        return (
          <button
            key={id}
            type="button"
            role="tab"
            id={`planner-workspace-tab-${id}`}
            aria-selected={selected}
            aria-controls={`planner-workspace-panel-${id}`}
            tabIndex={selected ? 0 : -1}
            className={cn(
              "cursor-pointer rounded-full px-4 py-1.5 text-label-lg transition-colors duration-150 ease-out",
              "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary",
              selected
                ? "bg-primary text-on-primary"
                : "text-on-surface-variant hover:text-on-surface",
            )}
            onClick={() => {
              onWorkspaceChange(id);
            }}
          >
            {WORKSPACE_LABELS[id]}
          </button>
        );
      })}
    </div>
  );
}

export function plannerWorkspacePanelProps(
  workspace: PlannerWorkspace,
  activeWorkspace: PlannerWorkspace,
) {
  return {
    id: `planner-workspace-panel-${workspace}`,
    role: "tabpanel" as const,
    "aria-labelledby": `planner-workspace-tab-${workspace}`,
    hidden: workspace !== activeWorkspace,
  };
}
