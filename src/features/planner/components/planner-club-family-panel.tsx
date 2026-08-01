import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { Plus, Trash2 } from "lucide-react";
import { useEffect, useId, useMemo, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { SelectField } from "@/components/ui/field/select-field";
import { TextField } from "@/components/ui/field/text-field";
import { Panel } from "@/components/ui/panel/panel";
import { plannerClubFamilyQueryOptions } from "../api/get-planner-club-family-query-options";
import { plannerClubsQueryOptions } from "../api/planner-clubs-query-options";
import { plannerKeys } from "../api/planner-keys";
import { savePlannerClubFamily } from "../api/save-planner-club-family";
import type {
  ClubFamily,
  ClubSourceInput,
  PlannerTeam,
} from "../types/club-family";

type DraftSource = ClubSourceInput & { id: number };

const CLUB_SUGGEST_LIMIT = 10;

const TEAM_LABELS: Record<PlannerTeam, string> = {
  senior: "Senior",
  reserves: "Reserves",
  youth: "Youth",
};

function draftSources(family: ClubFamily): DraftSource[] {
  return family.sources
    .filter(
      (source): source is typeof source & { team: "reserves" | "youth" } =>
        !source.isPrimary && source.team !== "senior",
    )
    .map((source) => ({
      id: source.id,
      team: source.team,
      clubName: source.clubName,
      teamLevel: null,
    }));
}

function sourceIsMissing(source: DraftSource, availableClubs: string[]) {
  return source.clubName !== "" && !availableClubs.includes(source.clubName);
}

type PrimaryClubPickerProps = {
  clubs: string[];
  value: string;
  onSelect: (club: string) => void;
  onSearchChange: (query: string) => void;
};

function PrimaryClubPicker({
  clubs,
  value,
  onSelect,
  onSearchChange,
}: PrimaryClubPickerProps) {
  const activeOptionRef = useRef<HTMLButtonElement>(null);
  const [query, setQuery] = useState(value);
  const [open, setOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(0);
  const listboxId = useId();
  const optionIdPrefix = useId();

  useEffect(() => {
    setQuery(value);
  }, [value]);

  const matches = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    if (normalizedQuery.length === 0) {
      return [];
    }
    return clubs
      .filter((club) => club.toLowerCase().includes(normalizedQuery))
      .slice(0, CLUB_SUGGEST_LIMIT);
  }, [clubs, query]);
  const activeClub = matches[activeIndex];
  const showSuggestions = open && matches.length > 0;

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
    <div className="relative">
      <TextField
        aria-activedescendant={
          showSuggestions ? `${optionIdPrefix}-${activeIndex}` : undefined
        }
        aria-autocomplete="list"
        aria-controls={showSuggestions ? listboxId : undefined}
        aria-expanded={showSuggestions}
        aria-haspopup="listbox"
        autoComplete="off"
        label="Primary club"
        placeholder="Search clubs…"
        role="combobox"
        type="text"
        value={query}
        onBlur={() => {
          window.setTimeout(() => {
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
            const club = showSuggestions ? matches[activeIndex] : undefined;
            if (club) {
              selectClub(club);
            }
            return;
          }
          if (!showSuggestions) {
            return;
          }
          if (event.key === "ArrowDown") {
            event.preventDefault();
            setActiveIndex((index) => (index + 1) % matches.length);
            return;
          }
          if (event.key === "ArrowUp") {
            event.preventDefault();
            setActiveIndex(
              (index) => (index - 1 + matches.length) % matches.length,
            );
            return;
          }
        }}
      />
      {showSuggestions ? (
        <div
          aria-label="Club suggestions"
          className="absolute z-20 mt-1 max-h-64 w-full overflow-auto rounded-lg border border-outline-variant bg-surface-container-highest py-1 shadow-overlay"
          id={listboxId}
          role="listbox"
        >
          {matches.map((club, index) => (
            <button
              aria-selected={index === activeIndex}
              className={
                index === activeIndex
                  ? "flex w-full cursor-pointer px-3 py-2 text-left text-body-sm text-on-surface bg-surface-container-high"
                  : "flex w-full cursor-pointer px-3 py-2 text-left text-body-sm text-on-surface hover:bg-surface-container-high"
              }
              id={`${optionIdPrefix}-${index}`}
              key={club}
              ref={club === activeClub ? activeOptionRef : undefined}
              role="option"
              type="button"
              onMouseDown={(event) => {
                event.preventDefault();
              }}
              onMouseEnter={() => {
                setActiveIndex(index);
              }}
              onClick={() => {
                selectClub(club);
              }}
            >
              {club}
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}

export function PlannerClubFamilyPanel() {
  const queryClient = useQueryClient();
  const { data: family } = useSuspenseQuery(plannerClubFamilyQueryOptions);
  const { data: availableClubs } = useSuspenseQuery(plannerClubsQueryOptions);
  const [primaryClub, setPrimaryClub] = useState(family.primaryClub ?? "");
  const [primaryClubSearchPending, setPrimaryClubSearchPending] =
    useState(false);
  const [sources, setSources] = useState<DraftSource[]>(draftSources(family));

  useEffect(() => {
    setPrimaryClub(family.primaryClub ?? "");
    setPrimaryClubSearchPending(false);
    setSources(draftSources(family));
  }, [family]);

  const clubOptions = useMemo(
    () =>
      Array.from(
        new Set([
          ...availableClubs,
          primaryClub,
          ...sources.map((source) => source.clubName),
        ]),
      ).filter(Boolean),
    [availableClubs, primaryClub, sources],
  );

  const save = useMutation({
    mutationFn: () => savePlannerClubFamily(primaryClub, sources),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: plannerKeys.all });
    },
  });

  const addSource = (team: Exclude<PlannerTeam, "senior">) => {
    const nextId = Math.min(0, ...sources.map((source) => source.id)) - 1;
    setSources((current) => [
      ...current,
      {
        id: nextId,
        team,
        clubName: availableClubs[0] ?? "",
        teamLevel: null,
      },
    ]);
  };

  return (
    <Panel
      title={family.primaryClub ? "Club family" : "Set up your club family"}
      flush
    >
      <form
        className="space-y-5 p-4"
        onSubmit={(event) => {
          event.preventDefault();
          save.mutate();
        }}
      >
        <div className="max-w-md space-y-2">
          <PrimaryClubPicker
            clubs={clubOptions}
            value={primaryClub}
            onSearchChange={(query) => {
              setPrimaryClubSearchPending(query !== primaryClub);
            }}
            onSelect={(club) => {
              setPrimaryClub(club);
              setPrimaryClubSearchPending(false);
            }}
          />
          {primaryClub && !availableClubs.includes(primaryClub) ? (
            <p className="text-body-sm text-warning">
              This club is not in the current snapshot. The mapping stays saved
              until you replace it.
            </p>
          ) : null}
        </div>

        {family.primaryClub ? (
          <div className="space-y-3">
            <div>
              <h2 className="text-title-md text-on-surface">
                Associated clubs
              </h2>
              <p className="text-body-sm text-on-surface-variant">
                Add separate B teams or youth clubs when Football Manager lists
                them as different clubs.
              </p>
            </div>
            {sources.length === 0 ? (
              <p className="text-body-sm text-on-surface-variant">
                No associated clubs yet. Every primary-club player is available
                to all three teams.
              </p>
            ) : (
              <div className="space-y-3">
                {sources.map((source, index) => {
                  const teamSourceNumber =
                    sources
                      .slice(0, index)
                      .filter((item) => item.team === source.team).length + 1;
                  return (
                    <div
                      className="grid gap-3 rounded-md border border-outline-variant p-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-end"
                      key={source.id}
                    >
                      <SelectField
                        label={`${TEAM_LABELS[source.team]} club ${teamSourceNumber}`}
                        value={source.clubName}
                        onChange={(event) => {
                          const clubName = event.target.value;
                          setSources((current) =>
                            current.map((item) =>
                              item.id === source.id
                                ? { ...item, clubName }
                                : item,
                            ),
                          );
                        }}
                      >
                        <option value="">Choose a club</option>
                        {clubOptions.map((club) => (
                          <option key={club} value={club}>
                            {club}
                          </option>
                        ))}
                      </SelectField>
                      <Button
                        size="icon"
                        variant="ghost"
                        icon={Trash2}
                        aria-label={`Remove ${TEAM_LABELS[source.team]} source ${teamSourceNumber}`}
                        onClick={() => {
                          setSources((current) =>
                            current.filter((item) => item.id !== source.id),
                          );
                        }}
                      />
                      {sourceIsMissing(source, availableClubs) ? (
                        <p className="text-body-sm text-warning">
                          This source is not in the current snapshot. It remains
                          saved until you replace it.
                        </p>
                      ) : null}
                    </div>
                  );
                })}
              </div>
            )}
            <div className="flex flex-wrap gap-2">
              <Button
                variant="secondary"
                icon={Plus}
                onClick={() => addSource("reserves")}
              >
                Add Reserves source
              </Button>
              <Button
                variant="secondary"
                icon={Plus}
                onClick={() => addSource("youth")}
              >
                Add Youth source
              </Button>
            </div>
          </div>
        ) : null}

        {save.isError ? (
          <p className="text-body-sm text-error" role="alert">
            {save.error.message}
          </p>
        ) : null}
        <Button
          type="submit"
          disabled={!primaryClub || primaryClubSearchPending}
          loading={save.isPending}
          loadingLabel="Saving…"
        >
          Save club family
        </Button>
      </form>
    </Panel>
  );
}
