import { useMutation, useQueryClient } from "@tanstack/react-query";
import { type KeyboardEvent, useEffect, useRef, useState } from "react";
import { Panel } from "@/components/ui/panel/panel";
import { addPlannerString } from "../api/add-planner-string";
import { clearPlannerTeam } from "../api/clear-planner-team";
import { optimizePlannerDepth } from "../api/optimize-planner-depth";
import { plannerKeys } from "../api/planner-keys";
import { removePlannerString } from "../api/remove-planner-string";
import { PLANNER_TEAMS, type PlannerTeam } from "../types/club-family";
import type { PlannerDepth, PlannerString } from "../types/depth";
import type { TacticOptions } from "../types/tactic";
import { PlannerClearTeamControl } from "./planner-clear-team-control";
import {
  PlannerDepthTable,
  PlannerStringRemovalConfirmation,
} from "./planner-depth-table";
import { PlannerOptimizerControls } from "./planner-optimizer-controls";
import {
  PlannerSlotFitPicker,
  type PlannerSlotTarget,
} from "./planner-slot-fit-picker";

const TEAM_LABELS: Record<PlannerTeam, string> = {
  senior: "Senior",
  reserves: "Reserves",
  youth: "Youth",
};

function nextTeam(team: PlannerTeam, key: string): PlannerTeam | null {
  const index = PLANNER_TEAMS.indexOf(team);
  if (index < 0) {
    return null;
  }
  if (key === "Home") {
    return PLANNER_TEAMS[0];
  }
  if (key === "End") {
    return PLANNER_TEAMS[PLANNER_TEAMS.length - 1];
  }
  if (key === "ArrowRight" || key === "ArrowDown") {
    return PLANNER_TEAMS[(index + 1) % PLANNER_TEAMS.length];
  }
  if (key === "ArrowLeft" || key === "ArrowUp") {
    return PLANNER_TEAMS[
      (index - 1 + PLANNER_TEAMS.length) % PLANNER_TEAMS.length
    ];
  }
  return null;
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function PlannerDepthMatrix({
  depth,
  tactic,
  options,
  activeSaveId,
}: {
  depth: PlannerDepth;
  tactic: PlannerDepth["tactic"];
  options: TacticOptions;
  activeSaveId: number;
}) {
  const [selectedTeam, setSelectedTeam] = useState<PlannerTeam>("senior");
  const [picker, setPicker] = useState<PlannerSlotTarget | null>(null);
  const [pickerOpen, setPickerOpen] = useState(false);
  const [pickerError, setPickerError] = useState<string | null>(null);
  const [stringError, setStringError] = useState<string | null>(null);
  const [openStringId, setOpenStringId] = useState<number | null>(null);
  const [removalTarget, setRemovalTarget] = useState<PlannerString | null>(
    null,
  );
  const [removalOpen, setRemovalOpen] = useState(false);
  const [clearTeamTarget, setClearTeamTarget] = useState<PlannerTeam | null>(
    null,
  );
  const [clearTeamOpen, setClearTeamOpen] = useState(false);
  const [clearTeamError, setClearTeamError] = useState<string | null>(null);
  const [optimizeError, setOptimizeError] = useState<string | null>(null);
  const [actionStatus, setActionStatus] = useState<string | null>(null);
  const queryClient = useQueryClient();
  const closeTimerRef = useRef<number | null>(null);
  const removalTimerRef = useRef<number | null>(null);
  const stringHeaderRefs = useRef(new Map<number, HTMLButtonElement>());
  const tabRefs = useRef<Record<PlannerTeam, HTMLButtonElement | null>>({
    senior: null,
    reserves: null,
    youth: null,
  });

  useEffect(() => {
    return () => {
      if (closeTimerRef.current !== null) {
        window.clearTimeout(closeTimerRef.current);
      }
      if (removalTimerRef.current !== null) {
        window.clearTimeout(removalTimerRef.current);
      }
    };
  }, []);

  const openPicker = (target: PlannerSlotTarget) => {
    if (closeTimerRef.current !== null) {
      window.clearTimeout(closeTimerRef.current);
      closeTimerRef.current = null;
    }
    setPickerError(null);
    setPicker(target);
    setPickerOpen(true);
  };

  const closePicker = () => {
    setPickerOpen(false);
    closeTimerRef.current = window.setTimeout(() => {
      setPicker(null);
      closeTimerRef.current = null;
    }, 200);
  };

  const handleTabKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const next = nextTeam(selectedTeam, event.key);
    if (!next) {
      return;
    }
    event.preventDefault();
    setSelectedTeam(next);
    tabRefs.current[next]?.focus();
  };

  const returnToStringHeader = (
    stringId: number,
    clearRemovalTarget = false,
  ) => {
    if (removalTimerRef.current !== null) {
      window.clearTimeout(removalTimerRef.current);
    }
    removalTimerRef.current = window.setTimeout(() => {
      stringHeaderRefs.current.get(stringId)?.focus();
      if (clearRemovalTarget) {
        setRemovalTarget(null);
      }
      removalTimerRef.current = null;
    }, 200);
  };

  const completeStringAction = (stringId: number) => {
    setOpenStringId(null);
    setRemovalOpen(false);
    returnToStringHeader(stringId, true);
  };

  const addString = useMutation({
    mutationFn: ({ team }: { team: PlannerTeam; originStringId: number }) =>
      addPlannerString(team),
    onSuccess: (nextDepth, { team }) => {
      queryClient.setQueryData(plannerKeys.depth(), nextDepth);
      setOpenStringId(null);
      const teamDepth = nextDepth.teams.find(
        (candidate) => candidate.team === team,
      );
      const addedString = teamDepth?.strings.at(-1);
      if (addedString) {
        returnToStringHeader(addedString.id);
      }
    },
    onError: (error, { originStringId }) => {
      setStringError(errorMessage(error));
      setOpenStringId(null);
      returnToStringHeader(originStringId);
    },
  });

  const removeString = useMutation({
    mutationFn: ({
      plannerString,
      confirmPopulated,
    }: {
      plannerString: PlannerString;
      confirmPopulated: boolean;
    }) => removePlannerString(plannerString.id, confirmPopulated),
    onSuccess: async (nextDepth, variables) => {
      queryClient.setQueryData(plannerKeys.depth(), nextDepth);
      await queryClient.invalidateQueries({
        queryKey: plannerKeys.slotCandidates(),
      });
      const team = depth.teams.find((candidate) =>
        candidate.strings.some(
          (plannerString) => plannerString.id === variables.plannerString.id,
        ),
      );
      const remainingStrings = nextDepth.teams.find(
        (candidate) => candidate.team === team?.team,
      )?.strings;
      const focusTarget = remainingStrings?.at(
        Math.min(
          variables.plannerString.stringOrder,
          (remainingStrings.length ?? 1) - 1,
        ),
      );
      completeStringAction(focusTarget?.id ?? variables.plannerString.id);
    },
    onError: (error, variables) => {
      setStringError(errorMessage(error));
      completeStringAction(variables.plannerString.id);
    },
  });

  const requestRemoveString = (plannerString: PlannerString) => {
    setStringError(null);
    if (plannerString.assignments.length === 0) {
      removeString.mutate({ plannerString, confirmPopulated: false });
      return;
    }
    setOpenStringId(null);
    setRemovalTarget(plannerString);
    setRemovalOpen(true);
  };

  const clearTeam = useMutation({
    mutationFn: (team: PlannerTeam) => clearPlannerTeam(team, true),
    onSuccess: async (nextDepth, team) => {
      queryClient.setQueryData(plannerKeys.depth(), nextDepth);
      await queryClient.invalidateQueries({
        queryKey: plannerKeys.slotCandidates(),
      });
      setClearTeamError(null);
      setActionStatus(`${TEAM_LABELS[team]} squad cleared.`);
      setClearTeamOpen(false);
    },
    onError: (error) => {
      setClearTeamError(errorMessage(error));
    },
  });

  const optimize = useMutation({
    mutationFn: optimizePlannerDepth,
    onSuccess: async (nextDepth) => {
      queryClient.setQueryData(plannerKeys.depth(), nextDepth);
      await queryClient.invalidateQueries({
        queryKey: plannerKeys.slotCandidates(),
      });
      setOptimizeError(null);
      setActionStatus("Squads optimized.");
    },
    onError: (error) => {
      setOptimizeError(errorMessage(error));
    },
  });

  const requestClearTeam = () => {
    setClearTeamError(null);
    setActionStatus(null);
    setClearTeamTarget(selectedTeam);
    setClearTeamOpen(true);
  };

  const closeClearTeam = () => {
    if (!clearTeam.isPending) {
      setClearTeamOpen(false);
    }
  };

  const closeRemoval = () => {
    if (!removalTarget) {
      return;
    }
    completeStringAction(removalTarget.id);
  };

  if (depth.teams.length === 0) {
    return null;
  }

  return (
    <Panel title="Squad depth" flush>
      <div className="space-y-4 p-4">
        <fieldset
          aria-label="Squad controls"
          className="flex min-w-0 flex-wrap items-center justify-between gap-3 rounded-lg border border-outline-variant bg-surface-container-low p-3"
        >
          <div
            role="tablist"
            aria-label="Squad planner teams"
            className="inline-flex rounded-full bg-surface-container-high p-0.5"
            onKeyDown={handleTabKeyDown}
          >
            {PLANNER_TEAMS.map((team) => {
              const selected = team === selectedTeam;
              return (
                <button
                  key={team}
                  ref={(element) => {
                    tabRefs.current[team] = element;
                  }}
                  type="button"
                  role="tab"
                  id={`${team}-depth-tab`}
                  aria-selected={selected}
                  aria-controls={`${team}-depth-panel`}
                  tabIndex={selected ? 0 : -1}
                  className={`cursor-pointer rounded-full px-4 py-1.5 text-label-lg transition-colors duration-150 ease-out focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary ${
                    selected
                      ? "bg-primary text-on-primary"
                      : "text-on-surface-variant hover:text-on-surface"
                  }`}
                  onClick={() => setSelectedTeam(team)}
                >
                  {TEAM_LABELS[team]}
                </button>
              );
            })}
          </div>
          <div className="flex flex-wrap items-center justify-end gap-2">
            <PlannerOptimizerControls
              pending={optimize.isPending}
              onOptimize={() => {
                if (optimize.isPending) {
                  return;
                }
                setOptimizeError(null);
                setActionStatus(null);
                optimize.mutate();
              }}
            />
            <PlannerClearTeamControl
              selectedTeam={selectedTeam}
              target={clearTeamTarget}
              open={clearTeamOpen}
              pending={clearTeam.isPending}
              disabled={clearTeam.isPending || optimize.isPending}
              error={clearTeamError}
              onRequest={requestClearTeam}
              onClose={closeClearTeam}
              onConfirm={(team) => clearTeam.mutate(team)}
            />
          </div>
        </fieldset>
        {actionStatus ? (
          <p className="text-body-sm text-success" role="status">
            {actionStatus}
          </p>
        ) : null}
        {pickerError ? (
          <p className="text-body-sm text-error" role="alert">
            {pickerError}
          </p>
        ) : null}
        {stringError ? (
          <p className="text-body-sm text-error" role="alert">
            {stringError}
          </p>
        ) : null}
        {optimizeError ? (
          <p className="text-body-sm text-error" role="alert">
            {optimizeError}
          </p>
        ) : null}
        {PLANNER_TEAMS.map((team) => {
          const teamDepth = depth.teams.find(
            (candidate) => candidate.team === team,
          );
          if (!teamDepth) {
            return null;
          }
          return (
            <div
              id={`${team}-depth-panel`}
              key={team}
              role="tabpanel"
              aria-labelledby={`${team}-depth-tab`}
              hidden={team !== selectedTeam}
            >
              <PlannerDepthTable
                teamDepth={teamDepth}
                teamLabel={TEAM_LABELS[team]}
                tactic={tactic}
                options={options}
                onOpen={openPicker}
                openStringId={openStringId}
                onOpenStringMenu={setOpenStringId}
                onCloseStringMenu={() => setOpenStringId(null)}
                onAddString={(team, originStringId) => {
                  setStringError(null);
                  addString.mutate({ team, originStringId });
                }}
                onRemoveString={requestRemoveString}
                addDisabled={addString.isPending}
                stringHeaderRef={(stringId) => (element) => {
                  if (element) {
                    stringHeaderRefs.current.set(stringId, element);
                  } else {
                    stringHeaderRefs.current.delete(stringId);
                  }
                }}
              />
            </div>
          );
        })}
      </div>
      {picker ? (
        <PlannerSlotFitPicker
          activeSaveId={activeSaveId}
          open={pickerOpen}
          target={picker}
          tactic={tactic}
          options={options}
          onClose={closePicker}
          onMutationError={setPickerError}
        />
      ) : null}
      <PlannerStringRemovalConfirmation
        target={removalTarget}
        open={removalOpen}
        pending={removeString.isPending}
        onClose={closeRemoval}
        onConfirm={(plannerString) =>
          removeString.mutate({ plannerString, confirmPopulated: true })
        }
      />
    </Panel>
  );
}
