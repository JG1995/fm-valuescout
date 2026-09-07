import { type ReactNode, useCallback, useState } from "react";
import { TableToolbar } from "@/components/player-table/table-toolbar";
import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import type { SearchView } from "../types/search-view";
import { completeFilterRules } from "../utils/filter-registry";
import { SearchFilterEditorModal } from "./search-filter-editor-modal";
import { SearchFilterStrip } from "./search-filter-strip";

type SearchFilterBarProps = {
  rules: FilterRule[];
  combine: FilterCombineMode;
  onRulesChange: (rules: FilterRule[]) => void;
  onApply: (rules: FilterRule[], combine: FilterCombineMode) => void;
  view?: SearchView;
  summary?: ReactNode;
  columnsControl?: ReactNode;
  datasetToggles?: ReactNode;
};

/**
 * Search dataset controls composed into the shared table-associated
 * toolbar: summary, removable filter chips with Clear-all and Edit, grouped
 * column configuration, and caller-owned dataset toggles. Filter drafts
 * stay query-silent in the editor modal until Done applies one URL update;
 * the caller applies filter-adds-column-once in `onApply`. Page actions
 * (Add Tactic, Upload Shortlist) are owned by the Search page header and
 * are never accepted here.
 */
export function SearchFilterBar({
  rules,
  combine,
  onRulesChange,
  onApply,
  view = "general",
  summary,
  columnsControl,
  datasetToggles,
}: SearchFilterBarProps) {
  const [editorOpen, setEditorOpen] = useState(false);
  const closeEditor = useCallback(() => {
    setEditorOpen(false);
  }, []);
  const openEditor = useCallback(() => {
    setEditorOpen(true);
  }, []);
  const appliedRules = completeFilterRules(rules, view);

  return (
    <>
      <TableToolbar
        toolbarLabel="Player results toolbar"
        summary={summary}
        filterChips={
          <SearchFilterStrip
            rules={rules}
            combine={combine}
            onRulesChange={onRulesChange}
            view={view}
          />
        }
        onClearAll={
          appliedRules.length > 0 ? () => onRulesChange([]) : undefined
        }
        onEditFilters={openEditor}
        columnsControl={columnsControl}
        datasetToggles={datasetToggles}
      />
      <SearchFilterEditorModal
        open={editorOpen}
        onClose={closeEditor}
        rules={rules}
        combine={combine}
        onApply={onApply}
        view={view}
      />
    </>
  );
}
