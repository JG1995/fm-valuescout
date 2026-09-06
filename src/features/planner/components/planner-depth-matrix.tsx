import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Panel } from "@/components/ui/panel/panel";
import { clearPlannerDepth } from "../api/clear-planner-depth";
import {
  optimizePlannerDepth,
  type PlannerScoreBasis,
} from "../api/optimize-planner-depth";
import { plannerKeys } from "../api/planner-keys";
import type { PlannerDepth } from "../types/depth";
import type { TacticOptions } from "../types/tactic";
import type { PlannerTeam } from "../types/team";
import { PlannerClearAllControl } from "./planner-clear-all-control";
import { PlannerOptimizerControls } from "./planner-optimizer-controls";
import { PlannerRoleReferenceModal } from "./planner-role-reference-modal";
import {
  PlannerSlotFitPicker,
  type PlannerSlotTarget,
} from "./planner-slot-fit-picker";
import { PlannerSquadBoard } from "./planner-squad-board";
import { PlannerTeamManagement } from "./planner-team-management";

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
  const queryClient = useQueryClient();
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
      <div className="space-y-4 p-4">
        <fieldset
          aria-label="Squad controls"
          className="flex min-w-0 flex-wrap items-center justify-between gap-3 rounded-lg border border-outline-variant bg-surface-container-low p-3"
        >
          <div className="flex flex-wrap items-center justify-end gap-2">
            <Button
              variant="secondary"
              onClick={(event) => {
                setRoleReferenceReturnFocus(event.currentTarget);
                setRoleReferenceOpen(true);
              }}
              className="!h-7 !px-3 !text-label-md focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
            >
              Best role fit
            </Button>
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
