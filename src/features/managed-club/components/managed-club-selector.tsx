import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import {
  type ReactNode,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
} from "react";
import {
  type ClearPlayerResultContext,
  playerResultContextMutationKey,
} from "@/components/player-table/player-result-context";
import { Button } from "@/components/ui/button/button";
import { TextField } from "@/components/ui/field/text-field";
import { useAnchoredPopover } from "@/components/ui/use-anchored-popover";
import { managedClubKeys } from "../api/managed-club-keys";
import {
  managedClubOptionsQueryOptions,
  managedClubQueryOptions,
} from "../api/managed-club-query-options";
import { setManagedClub } from "../api/set-managed-club";
import type { ManagedClubOption } from "../types/managed-club";

const CLUB_SUGGEST_LIMIT = 10;

type ManagedClubPickerProps = {
  clubs: ManagedClubOption[];
  value: string;
  autoFocus: boolean;
  onSelect: (club: ManagedClubOption) => void;
  onSearchChange: (query: string) => void;
};

function ManagedClubPicker({
  clubs,
  value,
  autoFocus,
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
      .filter((club) => club.clubName.toLowerCase().includes(normalizedQuery))
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

  const selectClub = (club: ManagedClubOption) => {
    onSelect(club);
    setQuery(club.clubName);
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
        autoFocus={autoFocus}
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
              key={`${club.clubName}-${club.clubUid ?? "unknown"}`}
              ref={club === activeClub ? activeOptionRef : undefined}
              role="option"
              type="button"
              onMouseDown={(event) => event.preventDefault()}
              onMouseEnter={() => setActiveIndex(index)}
              onClick={() => selectClub(club)}
            >
              {club.clubName}
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}

export function ManagedClubSelector({
  action,
  onSaved,
  onBeforeContextChange,
}: {
  action?: ReactNode;
  onSaved?: () => void;
  onBeforeContextChange: ClearPlayerResultContext;
}) {
  const queryClient = useQueryClient();
  const { data: managedClub } = useSuspenseQuery(managedClubQueryOptions);
  const { data: availableClubs } = useSuspenseQuery(
    managedClubOptionsQueryOptions,
  );
  const [selectedOption, setSelectedOption] = useState<ManagedClubOption>({
    clubName: managedClub.clubName ?? "",
    clubUid: managedClub.clubUid ?? null,
  });
  const [searchPending, setSearchPending] = useState(false);
  const [editing, setEditing] = useState(!managedClub.clubName);
  const controlsRef = useRef<HTMLFieldSetElement>(null);
  const wasEditing = useRef(editing);

  useEffect(() => {
    setSelectedOption({
      clubName: managedClub.clubName ?? "",
      clubUid: managedClub.clubUid ?? null,
    });
    setSearchPending(false);
    setEditing(!managedClub.clubName);
  }, [managedClub.clubName, managedClub.clubUid]);

  const clubOptions = useMemo(
    () =>
      managedClub.clubName &&
      !availableClubs.some(
        (option) =>
          option.clubName === managedClub.clubName &&
          option.clubUid === (managedClub.clubUid ?? null),
      )
        ? [
            {
              clubName: managedClub.clubName,
              clubUid: managedClub.clubUid ?? null,
            },
            ...availableClubs,
          ]
        : availableClubs,
    [availableClubs, managedClub.clubName, managedClub.clubUid],
  );
  const save = useMutation({
    mutationKey: playerResultContextMutationKey,
    mutationFn: async () => {
      await onBeforeContextChange();
      return setManagedClub(selectedOption.clubName, selectedOption.clubUid);
    },
    onSuccess: () => {
      setEditing(false);
      void queryClient.invalidateQueries({ queryKey: managedClubKeys.all });
      onSaved?.();
    },
  });

  useEffect(() => {
    if (save.isPending) return;
    if (wasEditing.current && !editing) {
      controlsRef.current?.querySelector("button")?.focus();
    }
    wasEditing.current = editing;
  }, [editing, save.isPending]);

  const selectionChanged =
    selectedOption.clubName !== (managedClub.clubName ?? "") ||
    selectedOption.clubUid !== (managedClub.clubUid ?? null);
  const canSave =
    selectedOption.clubName.length > 0 && !searchPending && selectionChanged;

  return (
    <form
      className="w-full max-w-2xl space-y-2"
      onSubmit={(event) => {
        event.preventDefault();
        if (canSave && !save.isPending) save.mutate();
      }}
    >
      <fieldset
        ref={controlsRef}
        disabled={save.isPending}
        className="m-0 flex min-w-0 flex-wrap items-end gap-2 border-0 p-0"
      >
        <legend className="sr-only">Managed club controls</legend>
        {editing ? (
          <>
            <div className="min-w-64 flex-1">
              <ManagedClubPicker
                autoFocus={managedClub.clubName !== null}
                clubs={clubOptions}
                value={selectedOption.clubName}
                onSearchChange={(query) =>
                  setSearchPending(query !== selectedOption.clubName)
                }
                onSelect={(club) => {
                  setSelectedOption(club);
                  setSearchPending(false);
                }}
              />
            </div>
            <Button disabled={!canSave} loading={save.isPending} type="submit">
              Save managed club
            </Button>
            {managedClub.clubName ? (
              <Button
                variant="secondary"
                onClick={() => {
                  setSelectedOption({
                    clubName: managedClub.clubName ?? "",
                    clubUid: managedClub.clubUid,
                  });
                  setSearchPending(false);
                  save.reset();
                  setEditing(false);
                }}
              >
                Cancel
              </Button>
            ) : null}
          </>
        ) : (
          <Button
            variant="secondary"
            onClick={() => {
              save.reset();
              setEditing(true);
            }}
          >
            Edit managed club
          </Button>
        )}
        {action}
      </fieldset>

      {editing && (selectionChanged || searchPending) ? (
        <p className="text-body-sm text-on-surface-variant" role="status">
          {managedClub.clubName
            ? `Unsaved selection. Analysis still uses ${managedClub.clubName}.`
            : "Unsaved selection. Save an exact club to use it for analysis."}
        </p>
      ) : null}
      {managedClub.status === "missing" ? (
        <p className="text-body-sm text-warning">
          {managedClub.clubName} is not in the latest snapshot. The saved
          selection remains active until you replace it.
        </p>
      ) : null}
      {save.error ? (
        <p className="text-body-sm text-error">{save.error.message}</p>
      ) : null}
    </form>
  );
}
