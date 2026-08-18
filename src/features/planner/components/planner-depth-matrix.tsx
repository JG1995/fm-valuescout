import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  type KeyboardEvent,
  type RefObject,
  useEffect,
  useRef,
  useState,
} from "react";
import { Panel } from "@/components/ui/panel/panel";
import { addPlannerString } from "../api/add-planner-string";
import { clearPlannerDepth } from "../api/clear-planner-depth";
import {
  optimizePlannerDepth,
  type PlannerScoreBasis,
} from "../api/optimize-planner-depth";
import { plannerKeys } from "../api/planner-keys";
import { removePlannerString } from "../api/remove-planner-string";
import type {
  PlannerDepth,
  PlannerDepthTeam,
  PlannerString,
} from "../types/depth";
import type { TacticOptions } from "../types/tactic";
import { PLANNER_TEAMS, type PlannerTeam } from "../types/team";
import { PlannerClearAllControl } from "./planner-clear-all-control";
import {
  PlannerDepthTable,
  PlannerStringRemovalConfirmation,
} from "./planner-depth-table";
import { PlannerOptimizerControls } from "./planner-optimizer-controls";
import {
  PlannerSlotFitPicker,
  type PlannerSlotTarget,
} from "./planner-slot-fit-picker";
import { PlannerTeamManagement } from "./planner-team-management";

const MIN_MATRIX_COLUMN_REM = 13;

function matrixColumnMinimumWidth() {
  if (typeof document === "undefined") {
    return MIN_MATRIX_COLUMN_REM * 16;
  }
  const rootFontSize = Number.parseFloat(
    window.getComputedStyle(document.documentElement).fontSize,
  );
  return (
    MIN_MATRIX_COLUMN_REM * (Number.isFinite(rootFontSize) ? rootFontSize : 16)
  );
}

function combinedMatrixMinimumWidth(teamDepths: PlannerDepth["teams"]) {
  const stringCount = teamDepths.reduce(
    (count, teamDepth) => count + teamDepth.strings.length,
    0,
  );
  return matrixColumnMinimumWidth() * (stringCount + 1);
}

function useElementWidth(elementRef: RefObject<HTMLElement | null>) {
  const [width, setWidth] = useState(0);

  useEffect(() => {
    const element = elementRef.current;
    if (!element) {
      return;
    }

    const measure = () => {
      setWidth(element.clientWidth);
    };
    measure();
    window.addEventListener("resize", measure);
    if (typeof ResizeObserver === "undefined") {
      return () => window.removeEventListener("resize", measure);
    }
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", measure);
    };
  }, [elementRef]);

  return width;
}

function nextTeam(
  team: PlannerTeam,
  key: string,
  availableTeams: PlannerTeam[],
): PlannerTeam | null {
  const index = availableTeams.indexOf(team);
  if (index < 0) {
    return null;
  }
  if (key === "Home") {
    return availableTeams[0] ?? null;
  }
  if (key === "End") {
    return availableTeams.at(-1) ?? null;
  }
  if (key === "ArrowRight" || key === "ArrowDown") {
    return availableTeams[(index + 1) % availableTeams.length] ?? null;
  }
  if (key === "ArrowLeft" || key === "ArrowUp") {
    return (
      availableTeams[
        (index - 1 + availableTeams.length) % availableTeams.length
      ] ?? null
    );
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
  const [clearAllOpen, setClearAllOpen] = useState(false);
  const [clearAllError, setClearAllError] = useState<string | null>(null);
  const [optimizeError, setOptimizeError] = useState<string | null>(null);
  const [actionStatus, setActionStatus] = useState<string | null>(null);
  const [teamManagementPending, setTeamManagementPending] = useState(false);
  const matrixContainerRef = useRef<HTMLDivElement>(null);
  const matrixWidth = useElementWidth(matrixContainerRef);
  const queryClient = useQueryClient();
  const closeTimerRef = useRef<number | null>(null);
  const removalTimerRef = useRef<number | null>(null);
  const teamFocusTimerRef = useRef<number | null>(null);
  const stringHeaderRefs = useRef(new Map<number, HTMLButtonElement>());
  const cellRefs = useRef(new Map<string, HTMLButtonElement>());
  const tabRefs = useRef<Record<PlannerTeam, HTMLButtonElement | null>>({
    senior: null,
    reserves: null,
    youth: null,
  });
  const lastFocusContext = useRef<
    | { kind: "tab"; team: PlannerTeam }
    | { kind: "clear" }
    | { kind: "string"; team: PlannerTeam; stringId: number }
    | {
        kind: "cell";
        team: PlannerTeam;
        stringId: number;
        laneId: string;
      }
    | null
  >(null);

  useEffect(() => {
    return () => {
      if (closeTimerRef.current !== null) {
        window.clearTimeout(closeTimerRef.current);
      }
      if (removalTimerRef.current !== null) {
        window.clearTimeout(removalTimerRef.current);
      }
      if (teamFocusTimerRef.current !== null) {
        window.clearTimeout(teamFocusTimerRef.current);
      }
    };
  }, []);

  const orderedTeamDepths = depth.teams;
  const availableTeams = orderedTeamDepths.map((teamDepth) => teamDepth.team);
  const firstAvailableTeam = availableTeams[0] ?? "senior";
  const selectedTeamAvailable = availableTeams.includes(selectedTeam);
  const teamLabels = orderedTeamDepths.reduce<
    Partial<Record<PlannerTeam, string>>
  >((labels, teamDepth) => {
    labels[teamDepth.team] = teamDepth.displayName;
    return labels;
  }, {});
  const availableTeamNames = orderedTeamDepths.map(
    (teamDepth) => teamDepth.displayName,
  );

  useEffect(() => {
    if (!selectedTeamAvailable) {
      setSelectedTeam(firstAvailableTeam);
    }
  }, [firstAvailableTeam, selectedTeamAvailable]);

  const showCombinedTeams =
    matrixWidth > 0 &&
    matrixWidth >= combinedMatrixMinimumWidth(orderedTeamDepths);
  const previousLayoutMode = useRef(showCombinedTeams);

  useEffect(() => {
    if (previousLayoutMode.current === showCombinedTeams) {
      return;
    }
    previousLayoutMode.current = showCombinedTeams;
    const context = lastFocusContext.current;
    if (!context) {
      return;
    }
    const frame = window.requestAnimationFrame(() => {
      if (context.kind === "string") {
        stringHeaderRefs.current.get(context.stringId)?.focus();
        return;
      }
      if (context.kind === "cell") {
        cellRefs.current
          .get(`${context.team}:${context.stringId}:${context.laneId}`)
          ?.focus();
        return;
      }
      if (context.kind === "clear") {
        document
          .querySelector<HTMLButtonElement>("[data-planner-clear-all]")
          ?.focus();
        return;
      }
      if (showCombinedTeams) {
        document
          .querySelector<HTMLButtonElement>(
            `[data-planner-team="${context.team}"][data-planner-string-id]`,
          )
          ?.focus();
        return;
      }
      tabRefs.current[context.team]?.focus();
    });
    return () => window.cancelAnimationFrame(frame);
  }, [showCombinedTeams]);

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
    const next = nextTeam(selectedTeam, event.key, availableTeams);
    if (!next) {
      return;
    }
    event.preventDefault();
    setSelectedTeam(next);
    lastFocusContext.current = { kind: "tab", team: next };
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
      setSelectedTeam(team);
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
    const owner = orderedTeamDepths.find((teamDepth) =>
      teamDepth.strings.some((candidate) => candidate.id === plannerString.id),
    );
    if (owner) {
      setSelectedTeam(owner.team);
    }
    setStringError(null);
    if (plannerString.assignments.length === 0) {
      removeString.mutate({ plannerString, confirmPopulated: false });
      return;
    }
    setOpenStringId(null);
    setRemovalTarget(plannerString);
    setRemovalOpen(true);
  };

  const clearAll = useMutation({
    mutationFn: () => clearPlannerDepth(true),
    onSuccess: async (nextDepth) => {
      queryClient.setQueryData(plannerKeys.depth(), nextDepth);
      await queryClient.invalidateQueries({
        queryKey: plannerKeys.slotCandidates(),
      });
      setClearAllError(null);
      setActionStatus("All squads cleared.");
      setClearAllOpen(false);
    },
    onError: (error) => {
      setClearAllError(errorMessage(error));
    },
  });

  const optimize = useMutation({
    mutationFn: optimizePlannerDepth,
    onSuccess: async (nextDepth, scoreBasis) => {
      queryClient.setQueryData(plannerKeys.depth(), nextDepth);
      await queryClient.invalidateQueries({
        queryKey: plannerKeys.slotCandidates(),
      });
      setOptimizeError(null);
      setActionStatus(
        scoreBasis === "potential"
          ? "Squads optimized by potential."
          : "Squads optimized by current scores.",
      );
    },
    onError: (error, scoreBasis) => {
      setOptimizeError(
        `${scoreBasis === "potential" ? "Potential" : "Current"} optimization failed: ${errorMessage(error)}`,
      );
    },
  });

  const optimizePendingBasis = optimize.isPending ? optimize.variables : null;

  const requestClearAll = () => {
    if (clearAll.isPending || optimize.isPending || teamManagementPending) {
      return;
    }
    setClearAllError(null);
    setActionStatus(null);
    setClearAllOpen(true);
  };

  const runOptimization = (scoreBasis: PlannerScoreBasis) => {
    if (clearAll.isPending || optimize.isPending || teamManagementPending) {
      return;
    }
    setOptimizeError(null);
    setActionStatus(null);
    optimize.mutate(scoreBasis);
  };

  const closeClearAll = () => {
    if (!clearAll.isPending) {
      setClearAllOpen(false);
    }
  };

  const closeRemoval = () => {
    if (!removalTarget) {
      return;
    }
    completeStringAction(removalTarget.id);
  };

  const reconcileTeamSettings = (
    nextDepth: PlannerDepth,
    removedTeams: PlannerTeam[],
  ) => {
    queryClient.setQueryData(plannerKeys.depth(), nextDepth);
    void queryClient.invalidateQueries({
      queryKey: plannerKeys.slotCandidates(),
    });
    if (closeTimerRef.current !== null) {
      window.clearTimeout(closeTimerRef.current);
      closeTimerRef.current = null;
    }
    setPickerOpen(false);
    setPicker(null);
    setOpenStringId(null);
    setRemovalOpen(false);
    setRemovalTarget(null);
    setPickerError(null);
    setStringError(null);
    setOptimizeError(null);
    setActionStatus("Team settings saved.");

    if (!removedTeams.includes(selectedTeam)) {
      return;
    }

    const availableTeams = nextDepth.teams.map((team) => team.team);
    const selectedIndex = PLANNER_TEAMS.indexOf(selectedTeam);
    const nextTeam =
      availableTeams.find(
        (team) => PLANNER_TEAMS.indexOf(team) > selectedIndex,
      ) ?? availableTeams[0];
    if (!nextTeam) {
      return;
    }
    setSelectedTeam(nextTeam);
    if (teamFocusTimerRef.current !== null) {
      window.clearTimeout(teamFocusTimerRef.current);
    }
    teamFocusTimerRef.current = window.setTimeout(() => {
      const focusTarget =
        document.querySelector<HTMLButtonElement>(
          `[data-planner-team-tab="${nextTeam}"]`,
        ) ??
        document.querySelector<HTMLButtonElement>(
          "[data-planner-manage-teams]",
        );
      focusTarget?.focus();
      teamFocusTimerRef.current = null;
    }, 220);
  };

  const stringHeaderRef =
    (stringId: number) => (element: HTMLButtonElement | null) => {
      if (element) {
        stringHeaderRefs.current.set(stringId, element);
      } else {
        stringHeaderRefs.current.delete(stringId);
      }
    };
  const onStringHeaderFocus = (team: PlannerTeam, stringId: number) => {
    lastFocusContext.current = { kind: "string", team, stringId };
  };
  const cellRef =
    (team: PlannerTeam, stringId: number, laneId: string) =>
    (element: HTMLButtonElement | null) => {
      const key = `${team}:${stringId}:${laneId}`;
      if (element) {
        cellRefs.current.set(key, element);
      } else {
        cellRefs.current.delete(key);
      }
    };
  const onCellFocus = (team: PlannerTeam, stringId: number, laneId: string) => {
    lastFocusContext.current = {
      kind: "cell",
      team,
      stringId,
      laneId,
    };
  };
  const renderDepthTable = (
    teamDepths: PlannerDepthTeam[],
    combined: boolean,
  ) => (
    <PlannerDepthTable
      teamDepths={teamDepths}
      combined={combined}
      tactic={tactic}
      options={options}
      onOpen={openPicker}
      openStringId={openStringId}
      onOpenStringMenu={setOpenStringId}
      onCloseStringMenu={() => setOpenStringId(null)}
      onAddString={(team, originStringId) => {
        setSelectedTeam(team);
        setStringError(null);
        addString.mutate({ team, originStringId });
      }}
      onRemoveString={requestRemoveString}
      addDisabled={addString.isPending || teamManagementPending}
      stringHeaderRef={stringHeaderRef}
      onStringHeaderFocus={onStringHeaderFocus}
      cellRef={cellRef}
      onCellFocus={onCellFocus}
    />
  );

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
          {!showCombinedTeams ? (
            <div
              role="tablist"
              aria-label="Squad planner teams"
              className="inline-flex rounded-full bg-surface-container-high p-0.5"
              onKeyDown={handleTabKeyDown}
            >
              {orderedTeamDepths.map((teamDepth) => {
                const team = teamDepth.team;
                const selected = team === selectedTeam;
                return (
                  <button
                    key={team}
                    ref={(element) => {
                      tabRefs.current[team] = element;
                    }}
                    data-planner-team-tab={team}
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
                    onFocus={() => {
                      lastFocusContext.current = { kind: "tab", team };
                    }}
                    onClick={() => setSelectedTeam(team)}
                  >
                    {teamDepth.displayName}
                  </button>
                );
              })}
            </div>
          ) : null}
          <div className="flex flex-wrap items-center justify-end gap-2">
            <PlannerOptimizerControls
              pendingBasis={optimizePendingBasis}
              disabled={
                clearAll.isPending ||
                optimize.isPending ||
                teamManagementPending
              }
              onOptimize={runOptimization}
            />
            <PlannerTeamManagement
              depth={depth}
              disabled={
                teamManagementPending ||
                clearAll.isPending ||
                optimize.isPending ||
                addString.isPending ||
                removeString.isPending
              }
              onPendingChange={setTeamManagementPending}
              onSaved={reconcileTeamSettings}
            />
            <PlannerClearAllControl
              open={clearAllOpen}
              pending={clearAll.isPending}
              disabled={
                clearAll.isPending ||
                optimize.isPending ||
                teamManagementPending
              }
              error={clearAllError}
              teamNames={availableTeamNames}
              onRequest={requestClearAll}
              onFocus={() => {
                lastFocusContext.current = { kind: "clear" };
              }}
              onClose={closeClearAll}
              onConfirm={() => {
                if (
                  !clearAll.isPending &&
                  !optimize.isPending &&
                  !teamManagementPending
                ) {
                  clearAll.mutate();
                }
              }}
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
        <div
          ref={matrixContainerRef}
          data-testid="planner-depth-matrix-container"
          className="min-w-0"
        >
          {showCombinedTeams
            ? renderDepthTable(orderedTeamDepths, true)
            : orderedTeamDepths.map((teamDepth) => (
                <div
                  id={`${teamDepth.team}-depth-panel`}
                  key={teamDepth.team}
                  role="tabpanel"
                  aria-labelledby={`${teamDepth.team}-depth-tab`}
                  hidden={teamDepth.team !== selectedTeam}
                >
                  {renderDepthTable([teamDepth], false)}
                </div>
              ))}
        </div>
      </div>
      {picker ? (
        <PlannerSlotFitPicker
          activeSaveId={activeSaveId}
          open={pickerOpen}
          target={picker}
          tactic={tactic}
          options={options}
          teamLabels={teamLabels}
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
