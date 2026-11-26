import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Maximize2, Split } from "lucide-react";
import { type ReactNode, useEffect, useMemo, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Panel } from "@/components/ui/panel/panel";
import { clearPlannerDepth } from "../api/clear-planner-depth";
import { optimizePlannerDepth } from "../api/optimize-planner-depth";
import {
  plannerDevelopmentQueryOptions,
  setPlannerOptimizeFor,
} from "../api/planner-development";
import { plannerKeys } from "../api/planner-keys";
import { plannerLoanCandidatesQueryOptions } from "../api/planner-loan-candidates-query-options";
import { useOptimizeOptions } from "../stores/use-optimize-options";
import type { PlannerDepth } from "../types/depth";
import type { OptimizeFor, PlannerDevelopment } from "../types/development";
import type { TacticOptions } from "../types/tactic";
import type { PlannerTeam } from "../types/team";
import { OptimizeForControl } from "./optimize-for-control";
import { PlannerClearAllControl } from "./planner-clear-all-control";
import { PlannerFmMoves } from "./planner-fm-moves";
import { PlannerLoanCandidates } from "./planner-loan-candidates";
import { PlannerRoleReferenceModal } from "./planner-role-reference-modal";
import {
  PlannerSlotFitPicker,
  type PlannerSlotTarget,
} from "./planner-slot-fit-picker";
import { PlannerSquadBoard } from "./planner-squad-board";
import { PlannerTeamManagement } from "./planner-team-management";
import { PlannerUnassignedPlayers } from "./planner-unassigned-players";
import { PositionFamiliarityToggle } from "./position-familiarity-toggle";

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

/**
 * What Optimize did: without position familiarity, how many players it leaves
 * out of position, and for development, what its starters give up against the
 * strongest lineup.
 */
function optimizedStatus(depth: PlannerDepth, positionFamiliarity: boolean) {
  const comparison = depth.starterComparison ?? null;
  const parts = [
    comparison ? "Squads optimized for development" : "Squads optimized",
  ];
  if (!positionFamiliarity) {
    const count = depth.teams
      .flatMap((team) => team.strings)
      .flatMap((plannerString) => plannerString.assignments)
      .filter((assignment) => assignment.outOfPosition).length;
    parts[0] += " without position familiarity";
    parts.push(
      `${
        count === 0
          ? "Nobody is"
          : count === 1
            ? "1 player is"
            : `${count} players are`
      } out of position`,
    );
  }
  if (comparison && comparison.starters > 0) {
    const average = comparison.score / comparison.starters;
    const strongest =
      comparison.strongestStarters > 0
        ? comparison.strongestScore / comparison.strongestStarters
        : average;
    const gap = strongest - average;
    parts.push(
      `The starters it placed average ${average.toFixed(1)}, ${
        gap >= 0.05
          ? `${gap.toFixed(1)} below the strongest lineup`
          : "level with the strongest lineup"
      }`,
    );
  }
  return `${parts.join(". ")}.`;
}

/** The players who play a slot that trains differently from their development slot. */
function awayFromDevelopmentSlot(
  depth: PlannerDepth,
  development: PlannerDevelopment | undefined,
) {
  const lanes = new Map(
    (development?.players ?? []).map((player) => [
      player.playerUid,
      player.laneIds,
    ]),
  );
  return new Set(
    depth.teams
      .flatMap((team) => team.strings)
      .flatMap((plannerString) => plannerString.assignments)
      .filter((assignment) => {
        const laneIds = lanes.get(assignment.playerUid);
        return laneIds !== undefined && !laneIds.includes(assignment.laneId);
      })
      .map((assignment) => assignment.playerUid),
  );
}

/** Separates the squad controls that change the board, open a view, and set it up. */
function ControlDivider() {
  return (
    <span
      aria-hidden="true"
      className="mx-1 h-5 w-px shrink-0 bg-outline-variant"
    />
  );
}

export function PlannerDepthMatrix({
  depth,
  tactic,
  options,
  activeSaveId,
  actions,
}: {
  depth: PlannerDepth;
  tactic: PlannerDepth["tactic"];
  options: TacticOptions;
  activeSaveId: number;
  /** Further read-only views the page adds beside Best role fit. */
  actions?: ReactNode;
}) {
  const [picker, setPicker] = useState<PlannerSlotTarget | null>(null);
  const [pickerOpen, setPickerOpen] = useState(false);
  const [pickerError, setPickerError] = useState<string | null>(null);
  const [clearAllOpen, setClearAllOpen] = useState(false);
  const [clearAllError, setClearAllError] = useState<string | null>(null);
  const [optimizeError, setOptimizeError] = useState<string | null>(null);
  const [actionStatus, setActionStatus] = useState<string | null>(null);
  const [teamManagementPending, setTeamManagementPending] = useState(false);
  const [roleReferenceOpen, setRoleReferenceOpen] = useState(false);
  const [roleReferenceReturnFocus, setRoleReferenceReturnFocus] =
    useState<HTMLElement | null>(null);
  const positionFamiliarity = useOptimizeOptions(
    (state) => state.positionFamiliarity,
  );
  const setPositionFamiliarity = useOptimizeOptions(
    (state) => state.setPositionFamiliarity,
  );
  const queryClient = useQueryClient();
  const loanCandidates = useQuery(
    plannerLoanCandidatesQueryOptions(activeSaveId),
  );
  const development = useQuery(plannerDevelopmentQueryOptions(activeSaveId));
  const optimizeFor = development.data?.optimizeFor ?? "strongest";
  const awayUids = useMemo(
    () => awayFromDevelopmentSlot(depth, development.data),
    [depth, development.data],
  );
  const chooseGoal = useMutation({
    mutationFn: setPlannerOptimizeFor,
    onSuccess: (_, goal: OptimizeFor) => {
      queryClient.setQueryData(
        plannerKeys.development(activeSaveId),
        (current: PlannerDevelopment | undefined) =>
          current ? { ...current, optimizeFor: goal } : current,
      );
      setOptimizeError(null);
    },
    onError: (error) => {
      setOptimizeError(
        `Could not save the Optimize goal: ${errorMessage(error)}`,
      );
    },
  });
  const loanCandidateUids = useMemo(
    () =>
      new Set(
        (loanCandidates.data ?? [])
          .filter((candidate) => candidate.recommended)
          .map((candidate) => candidate.playerUid),
      ),
    [loanCandidates.data],
  );
  const closeTimerRef = useRef<number | null>(null);
  const teamFocusTimerRef = useRef<number | null>(null);

  useEffect(() => {
    return () => {
      if (closeTimerRef.current !== null) {
        window.clearTimeout(closeTimerRef.current);
      }
      if (teamFocusTimerRef.current !== null) {
        window.clearTimeout(teamFocusTimerRef.current);
      }
    };
  }, []);

  const orderedTeamDepths = depth.teams;
  const teamLabels = orderedTeamDepths.reduce<
    Partial<Record<PlannerTeam, string>>
  >((labels, teamDepth) => {
    labels[teamDepth.team] = teamDepth.displayName;
    return labels;
  }, {});
  const availableTeamNames = orderedTeamDepths.map(
    (teamDepth) => teamDepth.displayName,
  );

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
    onSuccess: async (nextDepth, optimizedWithFamiliarity) => {
      queryClient.setQueryData(plannerKeys.depth(), nextDepth);
      await queryClient.invalidateQueries({
        queryKey: plannerKeys.slotCandidates(),
      });
      setOptimizeError(null);
      setActionStatus(optimizedStatus(nextDepth, optimizedWithFamiliarity));
    },
    onError: (error) => {
      setOptimizeError(`Optimization failed: ${errorMessage(error)}`);
    },
  });

  const requestClearAll = () => {
    if (clearAll.isPending || optimize.isPending || teamManagementPending) {
      return;
    }
    setClearAllError(null);
    setActionStatus(null);
    setClearAllOpen(true);
  };

  const runOptimization = () => {
    if (
      clearAll.isPending ||
      optimize.isPending ||
      chooseGoal.isPending ||
      teamManagementPending
    ) {
      return;
    }
    setOptimizeError(null);
    setActionStatus(null);
    optimize.mutate(positionFamiliarity);
  };

  const closeClearAll = () => {
    if (!clearAll.isPending) {
      setClearAllOpen(false);
    }
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
    setPickerError(null);
    setOptimizeError(null);
    setActionStatus("Team settings saved.");

    if (removedTeams.length === 0) {
      return;
    }
    if (teamFocusTimerRef.current !== null) {
      window.clearTimeout(teamFocusTimerRef.current);
    }
    teamFocusTimerRef.current = window.setTimeout(() => {
      document
        .querySelector<HTMLButtonElement>("[data-planner-manage-teams]")
        ?.focus();
      teamFocusTimerRef.current = null;
    }, 220);
  };

  if (depth.teams.length === 0) {
    return null;
  }

  return (
    <Panel title="Squad depth" flush>
      <div className="flex flex-col gap-4 p-4">
        <fieldset
          aria-label="Squad controls"
          className="m-0 flex min-w-0 flex-wrap items-center justify-between gap-2 border-0 p-0"
        >
          <div className="flex flex-wrap items-center gap-2">
            <Button
              disabled={
                clearAll.isPending ||
                optimize.isPending ||
                chooseGoal.isPending ||
                teamManagementPending
              }
              loading={optimize.isPending}
              loadingLabel="Optimizing…"
              onClick={runOptimization}
            >
              Optimize squads
            </Button>
            <PlannerFmMoves
              activeSaveId={activeSaveId}
              disabled={
                clearAll.isPending ||
                optimize.isPending ||
                teamManagementPending
              }
            />
            <ControlDivider />
            <Button
              variant="secondary"
              icon={Maximize2}
              onClick={(event) => {
                setRoleReferenceReturnFocus(event.currentTarget);
                setRoleReferenceOpen(true);
              }}
              className="focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
            >
              Best role fit
            </Button>
            {actions}
            <PlannerLoanCandidates
              candidates={loanCandidates.data}
              error={loanCandidates.error}
              tactic={tactic}
              options={options}
              teamLabels={teamLabels}
            />
            <PlannerUnassignedPlayers
              activeSaveId={activeSaveId}
              tactic={tactic}
              options={options}
            />
            <ControlDivider />
            <PlannerTeamManagement
              depth={depth}
              disabled={
                teamManagementPending ||
                clearAll.isPending ||
                optimize.isPending
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
              onFocus={() => undefined}
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
        <div className="flex flex-col gap-1.5">
          {/* On its own line, as the familiarity option is: the action row
              must stay on one line at 1280px. */}
          <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
            <OptimizeForControl
              optimizeFor={optimizeFor}
              disabled={
                !development.data ||
                chooseGoal.isPending ||
                clearAll.isPending ||
                optimize.isPending ||
                teamManagementPending
              }
              onChange={(goal) => chooseGoal.mutate(goal)}
            />
            <span className="text-body-sm text-on-surface-variant">
              {optimizeFor === "development" && development.data
                ? `Optimize squads gives a player aged ${development.data.maxAge} or under only his development slot. He starts there if he earns it, and otherwise sits behind it.`
                : "Optimize squads puts every player where the squads are strongest today."}
            </span>
          </div>
          <PositionFamiliarityToggle
            checked={positionFamiliarity}
            disabled={
              clearAll.isPending || optimize.isPending || teamManagementPending
            }
            onChange={setPositionFamiliarity}
          />
          {awayUids.size > 0 ? (
            <p className="flex items-start gap-2 text-body-sm text-on-surface">
              <Split
                aria-hidden="true"
                className="mt-0.5 size-4 shrink-0 text-info"
              />
              {awayUids.size === 1
                ? "1 developing player plays away from his development slot."
                : `${awayUids.size} developing players play away from their development slots.`}{" "}
              Training can still follow it: choose Development slot on the
              Training page.
            </p>
          ) : null}
        </div>
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
        {optimizeError ? (
          <p className="text-body-sm text-error" role="alert">
            {optimizeError}
          </p>
        ) : null}
        <PlannerSquadBoard
          teamDepths={orderedTeamDepths}
          tactic={tactic}
          options={options}
          loanCandidateUids={loanCandidateUids}
          awayFromDevelopmentUids={awayUids}
          onOpen={openPicker}
        />
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
      <PlannerRoleReferenceModal
        activeSaveId={activeSaveId}
        open={roleReferenceOpen}
        tactic={tactic}
        options={options}
        onClose={() => setRoleReferenceOpen(false)}
        returnFocusTo={roleReferenceReturnFocus}
      />
    </Panel>
  );
}
