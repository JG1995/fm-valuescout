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
  filterChips?: ReactNode;
  onClearAll?: () => void;
  onEditFilters?: () => void;
  columnsControl?: ReactNode;
  datasetToggles?: ReactNode;
};

export function TableToolbar({
  toolbarLabel,
  summary,
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
      className="flex shrink-0 flex-wrap items-center gap-x-4 gap-y-2 px-4 pb-3"
    >
      {summary ? (
        <div className="text-body-md text-on-surface-variant">{summary}</div>
      ) : null}
      {filterChips}
      {onClearAll ? (
        <Button variant="ghost" onClick={onClearAll}>
          Clear all
        </Button>
      ) : null}
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
  );
}
