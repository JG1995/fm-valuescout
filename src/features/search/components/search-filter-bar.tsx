import { useCallback, useState } from "react";
import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import type { SearchView } from "../types/search-view";
import { SearchFilterEditorModal } from "./search-filter-editor-modal";
import { SearchFilterStrip } from "./search-filter-strip";

type SearchFilterBarProps = {
  rules: FilterRule[];
  combine: FilterCombineMode;
  onRulesChange: (rules: FilterRule[]) => void;
  onApply: (rules: FilterRule[], combine: FilterCombineMode) => void;
  view?: SearchView;
};

export function SearchFilterBar({
  rules,
  combine,
  onRulesChange,
  onApply,
  view = "general",
}: SearchFilterBarProps) {
  const [editorOpen, setEditorOpen] = useState(false);
  const closeEditor = useCallback(() => {
    setEditorOpen(false);
  }, []);
  const openEditor = useCallback(() => {
    setEditorOpen(true);
  }, []);

  return (
    <>
      <SearchFilterStrip
        rules={rules}
        combine={combine}
        onRulesChange={onRulesChange}
        onEdit={openEditor}
        view={view}
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
