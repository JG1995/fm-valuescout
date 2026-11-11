import type { ReactNode } from "react";

/** Where each panel sits in the dashboard grid. */
export const dashboardArea = {
  signings: "col-start-1 row-start-1",
  prospects: "col-start-1 row-start-2",
  lineup: "col-start-2 row-span-2 row-start-1",
  contracts: "col-start-3 row-span-2 row-start-1",
} as const;

/** Every dashboard panel fills its grid area and lays its content out in a column. */
export const dashboardPanelClass = "flex flex-col";
export const dashboardPanelContentClass = "flex min-h-0 flex-1 flex-col";

/**
 * The line that says what a card panel's cards are. It sits close under the
 * panel's title as its first content, and keeps both card panels' cards the
 * same height.
 */
export const cardPanelSummaryClass =
  "-mt-3 shrink-0 px-4 pb-2 text-body-sm text-on-surface-variant";

/**
 * The dashboard's one screen: two card panels stacked on the left, and the
 * starting XI and the contracts as full-height lists on the right. The rows
 * share the height the grid is given and never shrink below their panels, so a
 * window too short for them scrolls the page instead of clipping a panel.
 */
export function DashboardGrid({ children }: { children: ReactNode }) {
  return (
    <div className="grid flex-1 grid-cols-[minmax(0,1fr)_clamp(23rem,23vw,32rem)_clamp(20rem,18.5vw,27rem)] grid-rows-[minmax(min-content,1fr)_minmax(min-content,1fr)] gap-4">
      {children}
    </div>
  );
}
