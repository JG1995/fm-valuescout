import { Bookmark, Pencil, Trash2 } from "lucide-react";
import { type FormEvent, useEffect, useId, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { TextField } from "@/components/ui/field/text-field";
import {
  type SavedFilter,
  useSavedFiltersStore,
} from "../stores/use-saved-filters-store";
import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import type { MoneyballComparison } from "../types/search-view";
import { completeFilterRules } from "../utils/filter-registry";
import {
  parseSearchFilters,
  searchFiltersForUrl,
} from "../utils/search-url-search";

/** A tactic slot's recruitment filters, offered as a ready-made filter set. */
export type SlotFilterPreset = {
  laneId: string;
  /** The slot's compact placement, such as `AML/ML`. */
  label: string;
  /** Its roles, such as `Inside Forward / Winger`. */
  description: string;
  rules: FilterRule[];
  /** The slot's two position families, whose peers its percentiles rank among. */
  comparison: MoneyballComparison;
};

type SavedFiltersControlProps = {
  rules: FilterRule[];
  combine: FilterCombineMode;
  comparison: MoneyballComparison;
  slotPresets: readonly SlotFilterPreset[];
  onApply: (
    rules: FilterRule[],
    combine: FilterCombineMode,
    comparison: MoneyballComparison,
  ) => void;
};

const ROW_BUTTON =
  "min-w-0 flex-1 truncate rounded-md px-2 py-1 text-left text-body-md text-on-surface hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary";

/**
 * Saves the applied filters under a name and applies a saved set or a tactic
 * slot's recruitment filters. Applying replaces the current filters, how they
 * combine, and Percentiles vs.
 */
export function SavedFiltersControl({
  rules,
  combine,
  comparison,
  slotPresets,
  onApply,
}: SavedFiltersControlProps) {
  const savedFilters = useSavedFiltersStore((state) => state.savedFilters);
  const save = useSavedFiltersStore((state) => state.save);
  const rename = useSavedFiltersStore((state) => state.rename);
  const remove = useSavedFiltersStore((state) => state.remove);
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [renaming, setRenaming] = useState<{ id: string; name: string } | null>(
    null,
  );
  const containerRef = useRef<HTMLDivElement>(null);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const panelId = useId();
  const applied = completeFilterRules(rules);
  const replaces = savedFilters.some(
    (saved) => saved.name.toLowerCase() === name.trim().toLowerCase(),
  );

  useEffect(() => {
    if (!open) {
      return;
    }
    const handlePointerDown = (event: PointerEvent) => {
      const container = containerRef.current;
      if (container && !event.composedPath().includes(container)) {
        setOpen(false);
      }
    };
    document.addEventListener("pointerdown", handlePointerDown, true);
    return () =>
      document.removeEventListener("pointerdown", handlePointerDown, true);
  }, [open]);

  const close = () => {
    setOpen(false);
    setRenaming(null);
    buttonRef.current?.focus();
  };
  const applySaved = (saved: SavedFilter) => {
    onApply(parseSearchFilters(saved.filters), saved.combine, saved.comparison);
    close();
  };
  const submitSave = (event: FormEvent) => {
    event.preventDefault();
    if (applied.length === 0 || name.trim() === "") {
      return;
    }
    save({
      name,
      filters: searchFiltersForUrl(applied),
      combine,
      comparison,
    });
    setName("");
  };
  const submitRename = (event: FormEvent) => {
    event.preventDefault();
    if (renaming) {
      rename(renaming.id, renaming.name);
      setRenaming(null);
    }
  };

  return (
    <div ref={containerRef} className="relative">
      <button
        ref={buttonRef}
        type="button"
        aria-expanded={open}
        aria-controls={open ? panelId : undefined}
        // A short visible label keeps the season controls on one toolbar row.
        aria-label="Saved filters"
        title="Saved filters"
        onClick={() => (open ? close() : setOpen(true))}
        className="inline-flex h-8 items-center gap-2 rounded-md border border-outline px-3 text-label-md text-on-surface-variant transition-colors duration-150 ease-out hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
      >
        <Bookmark aria-hidden size={14} strokeWidth={2} />
        Saved
      </button>
      {open ? (
        <div
          id={panelId}
          role="dialog"
          aria-label="Saved filters"
          className="absolute right-0 top-full z-30 mt-1 max-h-[28rem] w-80 space-y-3 overflow-y-auto rounded-xl border border-outline-variant bg-surface-container-highest p-3 text-left shadow-overlay"
          onKeyDown={(event) => {
            if (event.key === "Escape") {
              event.preventDefault();
              if (renaming) {
                setRenaming(null);
              } else {
                close();
              }
            }
          }}
        >
          <section aria-label="Your saved filters" className="space-y-1">
            <h3 className="text-label-md text-on-surface-variant uppercase">
              Saved
            </h3>
            {savedFilters.length === 0 ? (
              <p className="text-body-sm text-on-surface-variant">
                No saved filters yet. Apply some, then save them below.
              </p>
            ) : (
              <ul className="space-y-0.5">
                {savedFilters.map((saved) =>
                  renaming?.id === saved.id ? (
                    <li key={saved.id}>
                      <form
                        className="flex items-end gap-2"
                        onSubmit={submitRename}
                      >
                        <TextField
                          label={`Rename ${saved.name}`}
                          className="min-w-0 flex-1"
                          value={renaming.name}
                          autoFocus
                          onChange={(event) =>
                            setRenaming({
                              id: saved.id,
                              name: event.target.value,
                            })
                          }
                        />
                        <Button
                          type="submit"
                          disabled={renaming.name.trim() === ""}
                        >
                          Save
                        </Button>
                      </form>
                    </li>
                  ) : (
                    <li key={saved.id} className="flex items-center gap-1">
                      <button
                        type="button"
                        className={ROW_BUTTON}
                        title={`${saved.filters.length} ${
                          saved.filters.length === 1 ? "filter" : "filters"
                        }`}
                        onClick={() => applySaved(saved)}
                      >
                        {saved.name}
                      </button>
                      <Button
                        variant="ghost"
                        size="icon"
                        icon={Pencil}
                        aria-label={`Rename ${saved.name}`}
                        onClick={() =>
                          setRenaming({ id: saved.id, name: saved.name })
                        }
                      />
                      <Button
                        variant="ghost"
                        size="icon"
                        icon={Trash2}
                        aria-label={`Delete ${saved.name}`}
                        onClick={() => remove(saved.id)}
                      />
                    </li>
                  ),
                )}
              </ul>
            )}
          </section>
          {slotPresets.length > 0 ? (
            <section aria-label="Tactic slot filters" className="space-y-1">
              <h3 className="text-label-md text-on-surface-variant uppercase">
                Tactic slots
              </h3>
              <ul className="space-y-0.5">
                {slotPresets.map((preset) => (
                  <li key={preset.laneId}>
                    <button
                      type="button"
                      className={ROW_BUTTON}
                      aria-label={`${preset.label} slot: ${preset.description}`}
                      onClick={() => {
                        onApply(preset.rules, "and", preset.comparison);
                        close();
                      }}
                    >
                      <span className="font-mono text-mono-sm">
                        {preset.label}
                      </span>{" "}
                      <span className="text-body-sm text-on-surface-variant">
                        {preset.description}
                      </span>
                    </button>
                  </li>
                ))}
              </ul>
            </section>
          ) : null}
          <form
            className="flex items-end gap-2 border-t border-outline-variant pt-3"
            onSubmit={submitSave}
          >
            <TextField
              label="Save current filters as"
              className="min-w-0 flex-1"
              value={name}
              placeholder={
                applied.length === 0 ? "Apply a filter first" : "Name"
              }
              disabled={applied.length === 0}
              onChange={(event) => setName(event.target.value)}
            />
            <Button
              type="submit"
              disabled={applied.length === 0 || name.trim() === ""}
            >
              {replaces ? "Replace" : "Save"}
            </Button>
          </form>
        </div>
      ) : null}
    </div>
  );
}
