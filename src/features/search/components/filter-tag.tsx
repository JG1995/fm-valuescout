import { X } from "lucide-react";
import type { FilterRule } from "../types/filter-rule";
import type { SearchView } from "../types/search-view";
import { formatFilterTagLabel } from "../utils/format-filter-label";

type FilterTagProps = {
  rule: FilterRule;
  onRemove: () => void;
  /** Opens the rule's editor, anchored to the chip. */
  onEdit?: (anchor: HTMLElement) => void;
  view?: SearchView;
};

export function FilterTag({
  rule,
  onRemove,
  onEdit,
  view = "general",
}: FilterTagProps) {
  const label = formatFilterTagLabel(rule, view);

  return (
    <span className="inline-flex h-7 max-w-full items-center rounded-full border border-outline-variant bg-surface-container-high text-label-md text-on-surface">
      {onEdit ? (
        <button
          type="button"
          aria-label={`Edit filter ${label}`}
          className="h-full min-w-0 truncate rounded-l-full pr-1 pl-3 hover:bg-surface-container-highest"
          onClick={(event) =>
            onEdit(event.currentTarget.parentElement ?? event.currentTarget)
          }
        >
          {label}
        </button>
      ) : (
        <span className="truncate pr-1 pl-3">{label}</span>
      )}
      <button
        type="button"
        className="inline-flex size-7 shrink-0 items-center justify-center rounded-full text-on-surface-variant hover:bg-surface-container-highest hover:text-on-surface"
        aria-label={`Remove filter ${label}`}
        onClick={onRemove}
      >
        <X aria-hidden size={14} strokeWidth={2} />
      </button>
    </span>
  );
}
