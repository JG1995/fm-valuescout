import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { useEffect, useId, useMemo, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { TextField } from "@/components/ui/field/text-field";
import { useAnchoredPopover } from "@/components/ui/use-anchored-popover";
import { managedClubKeys } from "../api/managed-club-keys";
import {
  managedClubOptionsQueryOptions,
  managedClubQueryOptions,
} from "../api/managed-club-query-options";
import { setManagedClub } from "../api/set-managed-club";

const CLUB_SUGGEST_LIMIT = 10;

type ManagedClubPickerProps = {
  clubs: string[];
  value: string;
  onSelect: (club: string) => void;
  onSearchChange: (query: string) => void;
};

function ManagedClubPicker({
  clubs,
  value,
  onSelect,
  onSearchChange,
}: ManagedClubPickerProps) {
  const activeOptionRef = useRef<HTMLButtonElement>(null);
  const blurTimeoutRef = useRef<number | undefined>(undefined);
  const [query, setQuery] = useState(value);
  const [open, setOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(0);
  const listboxId = useId();
  const optionIdPrefix = useId();

  useEffect(() => {
    setQuery(value);
    return () => window.clearTimeout(blurTimeoutRef.current);
  }, [value]);

  const matches = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    if (!normalizedQuery) {
      return [];
    }
    return clubs
      .filter((club) => club.toLowerCase().includes(normalizedQuery))
      .slice(0, CLUB_SUGGEST_LIMIT);
  }, [clubs, query]);
  const activeClub = matches[activeIndex];
  const showSuggestions = open && matches.length > 0;
  const { anchorRef, popoverRef, popover } =
    useAnchoredPopover<HTMLDivElement>(showSuggestions);

  useEffect(() => {
    if (!activeClub) {
      return;
    }
    activeOptionRef.current?.scrollIntoView?.({ block: "nearest" });
  }, [activeClub]);

  const selectClub = (club: string) => {
    onSelect(club);
    setQuery(club);
    setOpen(false);
    setActiveIndex(0);
  };

  return (
    <div ref={anchorRef} className="relative">
      <TextField
        aria-activedescendant={
          showSuggestions ? `${optionIdPrefix}-${activeIndex}` : undefined
        }
        aria-autocomplete="list"
        aria-controls={showSuggestions ? listboxId : undefined}
        aria-expanded={showSuggestions}
        aria-haspopup="listbox"
        autoComplete="off"
        label="Managed club"
        placeholder="Search clubs…"
        role="combobox"
        type="text"
        value={query}
        onBlur={() => {
          window.clearTimeout(blurTimeoutRef.current);
          blurTimeoutRef.current = window.setTimeout(() => {
            setOpen(false);
            setQuery(value);
            onSearchChange(value);
          }, 150);
        }}
        onChange={(event) => {
          const nextQuery = event.target.value;
          setQuery(nextQuery);
          onSearchChange(nextQuery);
          setOpen(true);
          setActiveIndex(0);
        }}
        onFocus={() => {
          window.clearTimeout(blurTimeoutRef.current);
          setOpen(true);
        }}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            setOpen(false);
            setQuery(value);
            onSearchChange(value);
            return;
          }
          if (event.key === "Enter") {
            event.preventDefault();
            if (activeClub) {
              selectClub(activeClub);
            }
            return;
          }
          if (!showSuggestions) {
            return;
          }
          if (event.key === "ArrowDown") {
            event.preventDefault();
            setActiveIndex((index) => (index + 1) % matches.length);
          } else if (event.key === "ArrowUp") {
            event.preventDefault();
            setActiveIndex(
              (index) => (index - 1 + matches.length) % matches.length,
            );
          }
        }}
      />
      {showSuggestions ? (
        <div
          ref={popoverRef}
          aria-label="Club suggestions"
          className="absolute z-20 m-0 mt-1 max-h-64 w-full overflow-auto rounded-lg border border-outline-variant bg-surface-container-highest py-1 shadow-overlay"
          id={listboxId}
          popover={popover}
          role="listbox"
        >
          {matches.map((club, index) => (
            <button
              aria-selected={index === activeIndex}
              className={
                index === activeIndex
                  ? "flex w-full cursor-pointer bg-surface-container-high px-3 py-2 text-left text-body-sm text-on-surface"
                  : "flex w-full cursor-pointer px-3 py-2 text-left text-body-sm text-on-surface hover:bg-surface-container-high"
              }
              id={`${optionIdPrefix}-${index}`}
              key={club}
              ref={club === activeClub ? activeOptionRef : undefined}
              role="option"
              type="button"
              onMouseDown={(event) => event.preventDefault()}
              onMouseEnter={() => setActiveIndex(index)}
              onClick={() => selectClub(club)}
            >
              {club}
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}

export function ManagedClubSelector({
  onSaved,
}: {
  onSaved?: () => void;
} = {}) {
  const queryClient = useQueryClient();
  const { data: managedClub } = useSuspenseQuery(managedClubQueryOptions);
  const { data: availableClubs } = useSuspenseQuery(
    managedClubOptionsQueryOptions,
  );
  const [clubName, setClubName] = useState(managedClub.clubName ?? "");
  const [searchPending, setSearchPending] = useState(false);

  useEffect(() => {
    setClubName(managedClub.clubName ?? "");
    setSearchPending(false);
  }, [managedClub.clubName]);

  const clubOptions = useMemo(
    () =>
      Array.from(
        new Set([
          ...(managedClub.clubName ? [managedClub.clubName] : []),
          ...availableClubs,
        ]),
      ),
    [availableClubs, managedClub.clubName],
  );
  const save = useMutation({
    mutationFn: () => setManagedClub(clubName),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: managedClubKeys.all });
      onSaved?.();
    },
  });

  return (
    <form
      className="w-full max-w-md space-y-2"
      onSubmit={(event) => {
        event.preventDefault();
        save.mutate();
      }}
    >
      <ManagedClubPicker
        clubs={clubOptions}
        value={clubName}
        onSearchChange={(query) => setSearchPending(query !== clubName)}
        onSelect={(club) => {
          setClubName(club);
          setSearchPending(false);
        }}
      />

      {managedClub.status === "missing" ? (
        <p className="text-body-sm text-warning">
          {managedClub.clubName} is not in the latest snapshot. The saved
          selection remains active until you replace it.
        </p>
      ) : null}
      {save.error ? (
        <p className="text-body-sm text-error">{save.error.message}</p>
      ) : null}

      <Button
        disabled={
          !clubName || searchPending || clubName === managedClub.clubName
        }
        loading={save.isPending}
        type="submit"
      >
        Save managed club
      </Button>
    </form>
  );
}
