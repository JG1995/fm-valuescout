import { X } from "lucide-react";
import type { FilterRule } from "../types/filter-rule";
import type { SearchView } from "../types/search-view";
import { formatFilterTagLabel } from "../utils/format-filter-label";

type FilterTagProps = {
  rule: FilterRule;
  onRemove: () => void;
  view?: SearchView;
};

export function FilterTag({
  rule,
  onRemove,
  view = "general",
}: FilterTagProps) {
  const label = formatFilterTagLabel(rule, view);

  return (
    <span className="inline-flex h-7 max-w-full items-center gap-1 rounded-full bg-primary-container pl-3 text-label-md text-on-primary-container">
      <span className="truncate">{label}</span>
      <button
        type="button"
        className="inline-flex size-7 shrink-0 items-center justify-center rounded-full text-on-primary-container hover:bg-surface-container-highest"
        aria-label={`Remove filter ${label}`}
        onClick={onRemove}
      >
        <X aria-hidden size={14} strokeWidth={2} />
      </button>
    </span>
  );
}
