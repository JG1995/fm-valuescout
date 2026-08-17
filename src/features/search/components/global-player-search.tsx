import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { Search, X } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import { useAnchoredPopover } from "@/components/ui/use-anchored-popover";
import { cn } from "@/utils/cn";
import { formatCount } from "@/utils/format";
import { suggestPlayersQueryOptions } from "../api/suggest-players-query-options";

export const SUGGEST_DEBOUNCE_MS = 200;

/** Pill search chrome — same tokens as `fieldClasses`, without `rounded-md` / `px-2`
 *  that would fight `rounded-full` / horizontal icon padding when `cn` concatenates. */
const searchFieldClasses = [
  "h-8 w-full rounded-full border border-outline bg-surface-container-high py-0 pr-8 pl-8",
  "text-body-md text-on-surface placeholder:text-on-surface-variant",
  "hover:border-on-surface-variant",
  "focus-visible:outline-offset-0",
  "disabled:cursor-not-allowed disabled:opacity-45",
  "transition-colors duration-150 ease-out",
].join(" ");

export function GlobalPlayerSearch() {
  const navigate = useNavigate();
  const inputRef = useRef<HTMLInputElement>(null);
  const blurCloseTimerRef = useRef<number | null>(null);
  const listboxId = useId();
  const optionIdPrefix = useId();
  const [value, setValue] = useState("");
  const [debounced, setDebounced] = useState("");
  const [open, setOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(0);

  useEffect(() => {
    const trimmed = value.trim();
    if (trimmed.length === 0) {
      setDebounced("");
      setActiveIndex(0);
      return;
    }
    const timer = window.setTimeout(() => {
      setDebounced(trimmed);
      setActiveIndex(0);
    }, SUGGEST_DEBOUNCE_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [value]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        inputRef.current?.focus();
        setOpen(true);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);

  useEffect(() => {
    return () => {
      if (blurCloseTimerRef.current !== null) {
        window.clearTimeout(blurCloseTimerRef.current);
      }
    };
  }, []);

  const { data: hits = [] } = useQuery(suggestPlayersQueryOptions(debounced));
  const showPopover = open && debounced.length > 0 && hits.length > 0;
  const { anchorRef, popoverRef, popover } =
    useAnchoredPopover<HTMLDivElement>(showPopover);

  const activateHit = (uid: number) => {
    setValue("");
    setDebounced("");
    setOpen(false);
    setActiveIndex(0);
    void navigate({
      to: "/players/$uid",
      params: { uid: String(uid) },
      search: {},
    });
  };

  const clearField = () => {
    setValue("");
    setDebounced("");
    setOpen(false);
    setActiveIndex(0);
  };

  return (
    <div className="relative min-w-0 flex-1">
      <div ref={anchorRef} className="relative">
        <Search
          aria-hidden
          className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-on-surface-variant"
        />
        <input
          ref={inputRef}
          aria-activedescendant={
            showPopover ? `${optionIdPrefix}-${activeIndex}` : undefined
          }
          aria-autocomplete="list"
          aria-controls={showPopover ? listboxId : undefined}
          aria-expanded={showPopover}
          aria-haspopup="listbox"
          aria-label="Search players"
          className={searchFieldClasses}
          placeholder="Search players…"
          role="combobox"
          type="text"
          value={value}
          onBlur={() => {
            if (blurCloseTimerRef.current !== null) {
              window.clearTimeout(blurCloseTimerRef.current);
            }
            // Delay so option click can fire before the list unmounts.
            blurCloseTimerRef.current = window.setTimeout(() => {
              setOpen(false);
              blurCloseTimerRef.current = null;
            }, 150);
          }}
          onChange={(event) => {
            setValue(event.target.value);
            setOpen(true);
          }}
          onFocus={() => {
            if (blurCloseTimerRef.current !== null) {
              window.clearTimeout(blurCloseTimerRef.current);
              blurCloseTimerRef.current = null;
            }
            setOpen(true);
          }}
          onKeyDown={(event) => {
            if (event.key === "Escape") {
              event.preventDefault();
              if (value.length > 0) {
                clearField();
                return;
              }
              setOpen(false);
              inputRef.current?.blur();
              return;
            }

            if (!showPopover) {
              return;
            }

            if (event.key === "ArrowDown") {
              event.preventDefault();
              setActiveIndex((index) => (index + 1) % hits.length);
              return;
            }

            if (event.key === "ArrowUp") {
              event.preventDefault();
              setActiveIndex(
                (index) => (index - 1 + hits.length) % hits.length,
              );
              return;
            }

            if (event.key === "Enter") {
              const hit = hits[activeIndex];
              if (hit) {
                event.preventDefault();
                activateHit(hit.uid);
              }
            }
          }}
        />
        {value.length > 0 ? (
          <button
            aria-label="Clear search"
            className="absolute top-1/2 right-1.5 inline-flex size-6 -translate-y-1/2 items-center justify-center rounded-full text-on-surface-variant hover:bg-surface-container-highest hover:text-on-surface"
            type="button"
            onClick={clearField}
          >
            <X aria-hidden className="size-3.5" />
          </button>
        ) : null}
      </div>
      {showPopover ? (
        <div
          ref={popoverRef}
          aria-label="Player suggestions"
          className="absolute z-20 m-0 mt-1 max-h-64 w-full overflow-auto rounded-lg border border-outline-variant bg-surface-container-highest py-1 shadow-overlay"
          id={listboxId}
          popover={popover}
          role="listbox"
        >
          {hits.map((hit, index) => (
            <button
              key={hit.uid}
              aria-selected={index === activeIndex}
              className={cn(
                "flex w-full cursor-pointer items-center justify-between gap-3 px-3 py-2 text-left text-body-sm text-on-surface",
                index === activeIndex
                  ? "bg-surface-container-high"
                  : "hover:bg-surface-container-high",
              )}
              id={`${optionIdPrefix}-${index}`}
              role="option"
              type="button"
              onMouseDown={(event) => {
                // Keep focus long enough for activation; avoid blur race.
                event.preventDefault();
              }}
              onMouseEnter={() => {
                setActiveIndex(index);
              }}
              onClick={() => {
                activateHit(hit.uid);
              }}
            >
              <span>{hit.name}</span>
              <span className="font-mono text-mono-sm text-on-surface-variant tabular-nums">
                CA {formatCount(hit.ca)}
              </span>
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}
