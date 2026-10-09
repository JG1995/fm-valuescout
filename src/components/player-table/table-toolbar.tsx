import { SlidersHorizontal } from "lucide-react";
import type { ReactNode } from "react";
import { Button } from "@/components/ui/button/button";

/**
 * Minimal reusable table-associated toolbar contract. The generic toolbar
 * owns only dataset slots — count/sort summary, filter chips with Clear-all
 * and edit, grouped column configuration, and caller-owned dataset toggles
 * — and ties them to the table region through the toolbar label. It
 * deliberately exposes no `children`, `actions`, or page-action slot: page
 * actions (Add Tactic, Upload Shortlist, Configure, Optimize, boosts, Squad
 * actions) stay in page-header or feature-owned areas outside the toolbar.
 * This file imports no store, no feature code, and knows no table IDs.
 */
export type TableToolbarProps = {
  toolbarLabel: string;
  summary?: ReactNode;
  filterSummary?: ReactNode;
  filterChips?: ReactNode;
  onClearAll?: () => void;
  onEditFilters?: () => void;
  columnsControl?: ReactNode;
  datasetToggles?: ReactNode;
};

export function TableToolbar({
  toolbarLabel,
  summary,
  filterSummary,
  filterChips,
  onClearAll,
  onEditFilters,
  columnsControl,
  datasetToggles,
}: TableToolbarProps) {
  return (
    <div
      role="toolbar"
      aria-label={toolbarLabel}
      className="shrink-0 border-b border-outline-variant px-4 py-2"
    >
      <div className="flex flex-wrap items-center gap-2">
        {summary ? (
          <div className="min-w-0 flex-1 basis-64 text-body-md text-on-surface-variant">
            {summary}
          </div>
        ) : null}
        <div className="ml-auto flex min-w-0 max-w-full flex-wrap items-center justify-end gap-2">
          {filterSummary}
          {onEditFilters ? (
            <Button
              variant="secondary"
              icon={SlidersHorizontal}
              onClick={onEditFilters}
            >
              Edit filters
            </Button>
          ) : null}
          {columnsControl}
          {datasetToggles}
        </div>
      </div>
      {filterChips || onClearAll ? (
        <fieldset className="mt-2 flex min-w-0 flex-wrap items-center gap-2">
          <legend className="sr-only">Applied filters</legend>
          {filterChips}
          {onClearAll ? (
            <Button variant="ghost" onClick={onClearAll}>
              Clear all
            </Button>
          ) : null}
        </fieldset>
      ) : null}
    </div>
  );
}
