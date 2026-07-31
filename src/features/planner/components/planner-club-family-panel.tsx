import {
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { Plus, Trash2 } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { SelectField } from "@/components/ui/field/select-field";
import { Panel } from "@/components/ui/panel/panel";
import { plannerClubFamilyQueryOptions } from "../api/get-planner-club-family-query-options";
import { plannerClubsQueryOptions } from "../api/planner-clubs-query-options";
import { plannerKeys } from "../api/planner-keys";
import { savePlannerClubFamily } from "../api/save-planner-club-family";
import type {
  ClubFamily,
  ClubSourceInput,
  PlannerTeam,
  PlannerTeamLevel,
} from "../types/club-family";

type DraftSource = ClubSourceInput & { id: number };

const TEAM_LABELS: Record<PlannerTeam, string> = {
  senior: "Senior",
  reserves: "Reserves",
  youth: "Youth",
};

const LEVEL_LABELS: Record<PlannerTeamLevel, string> = {
  senior: "Senior players",
  reserve: "Reserves players",
  youth: "Youth players",
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
      teamLevel: source.teamLevel,
    }));
}

function sourceIsMissing(source: DraftSource, availableClubs: string[]) {
  return source.clubName !== "" && !availableClubs.includes(source.clubName);
}

export function PlannerClubFamilyPanel() {
  const queryClient = useQueryClient();
  const { data: family } = useSuspenseQuery(plannerClubFamilyQueryOptions);
  const { data: availableClubs } = useSuspenseQuery(plannerClubsQueryOptions);
  const [primaryClub, setPrimaryClub] = useState(family.primaryClub ?? "");
  const [sources, setSources] = useState<DraftSource[]>(draftSources(family));

  useEffect(() => {
    setPrimaryClub(family.primaryClub ?? "");
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
          <SelectField
            label="Primary club"
            value={primaryClub}
            onChange={(event) => setPrimaryClub(event.target.value)}
          >
            <option value="">Choose a club</option>
            {clubOptions.map((club) => (
              <option key={club} value={club}>
                {club}
              </option>
            ))}
          </SelectField>
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
                No associated clubs yet. The primary club supplies all three
                team levels.
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
                      className="grid gap-3 rounded-md border border-outline-variant p-3 sm:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto] sm:items-end"
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
                      <SelectField
                        label={`${TEAM_LABELS[source.team]} player level ${teamSourceNumber}`}
                        value={source.teamLevel ?? ""}
                        onChange={(event) => {
                          const teamLevel = event.target.value
                            ? (event.target.value as PlannerTeamLevel)
                            : null;
                          setSources((current) =>
                            current.map((item) =>
                              item.id === source.id
                                ? { ...item, teamLevel }
                                : item,
                            ),
                          );
                        }}
                      >
                        <option value="">All levels</option>
                        {Object.entries(LEVEL_LABELS).map(([level, label]) => (
                          <option key={level} value={level}>
                            {label}
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
                        <p className="text-body-sm text-warning sm:col-span-2">
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
          disabled={!primaryClub}
          loading={save.isPending}
          loadingLabel="Saving…"
        >
          Save club family
        </Button>
      </form>
    </Panel>
  );
}
