import type { KeyboardEvent } from "react";
import { cn } from "@/utils/cn";

export const MY_CLUB_WORKSPACES = [
  "squad",
  "planner",
  "tactic",
  "staff",
] as const;
export type MyClubWorkspace = (typeof MY_CLUB_WORKSPACES)[number];

const WORKSPACE_LABELS: Record<MyClubWorkspace, string> = {
  squad: "Squad",
  planner: "Planner",
  tactic: "Tactic",
  staff: "Staff",
};

export function parseMyClubWorkspace(raw: unknown): MyClubWorkspace | null {
  return typeof raw === "string" && isMyClubWorkspace(raw) ? raw : null;
}

function isMyClubWorkspace(raw: string): raw is MyClubWorkspace {
  return (MY_CLUB_WORKSPACES as readonly string[]).includes(raw);
}

function focusWorkspaceTab(workspace: MyClubWorkspace) {
  document.getElementById(`my-club-workspace-tab-${workspace}`)?.focus();
}

type MyClubWorkspaceTabsProps = {
  workspace: MyClubWorkspace;
  onWorkspaceChange: (workspace: MyClubWorkspace) => void;
};

export function MyClubWorkspaceTabs({
  workspace,
  onWorkspaceChange,
}: MyClubWorkspaceTabsProps) {
  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const index = MY_CLUB_WORKSPACES.indexOf(workspace);
    let nextIndex = index;

    if (event.key === "ArrowRight" || event.key === "ArrowDown") {
      nextIndex = (index + 1) % MY_CLUB_WORKSPACES.length;
    } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
      nextIndex =
        (index - 1 + MY_CLUB_WORKSPACES.length) % MY_CLUB_WORKSPACES.length;
    } else if (event.key === "Home") {
      nextIndex = 0;
    } else if (event.key === "End") {
      nextIndex = MY_CLUB_WORKSPACES.length - 1;
    } else {
      return;
    }

    event.preventDefault();
    const next = MY_CLUB_WORKSPACES[nextIndex];
    onWorkspaceChange(next);
    focusWorkspaceTab(next);
  };

  return (
    <div
      role="tablist"
      aria-label="My Club workspaces"
      className="inline-flex rounded-full bg-surface-container-high p-0.5"
      onKeyDown={onKeyDown}
    >
      {MY_CLUB_WORKSPACES.map((id) => {
        const selected = id === workspace;
        return (
          <button
            key={id}
            type="button"
            role="tab"
            id={`my-club-workspace-tab-${id}`}
            aria-selected={selected}
            aria-controls={`my-club-workspace-panel-${id}`}
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

export function myClubWorkspacePanelProps(
  workspace: MyClubWorkspace,
  activeWorkspace: MyClubWorkspace,
) {
  return {
    id: `my-club-workspace-panel-${workspace}`,
    role: "tabpanel" as const,
    "aria-labelledby": `my-club-workspace-tab-${workspace}`,
    hidden: workspace !== activeWorkspace,
  };
}
