import { useCallback, useState } from "react";
import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import { SearchFilterEditorModal } from "./search-filter-editor-modal";
import { SearchFilterStrip } from "./search-filter-strip";

type SearchFilterBarProps = {
  rules: FilterRule[];
  combine: FilterCombineMode;
  onRulesChange: (rules: FilterRule[]) => void;
  onCombineChange: (combine: FilterCombineMode) => void;
};

export function SearchFilterBar({
  rules,
  combine,
  onRulesChange,
  onCombineChange,
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
      />
      <SearchFilterEditorModal
        open={editorOpen}
        onClose={closeEditor}
        rules={rules}
        combine={combine}
        onRulesChange={onRulesChange}
        onCombineChange={onCombineChange}
      />
    </>
  );
}
