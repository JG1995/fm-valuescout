import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Maximize2 } from "lucide-react";
import { type ReactNode, useEffect, useRef, useState } from "react";
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import { Button } from "@/components/ui/button/button";
import { Panel } from "@/components/ui/panel/panel";
import { clearPlannerDepth } from "../api/clear-planner-depth";
import { optimizePlannerDepth } from "../api/optimize-planner-depth";
import { plannerKeys } from "../api/planner-keys";
import {
  type PlannerStringSettingInput,
  type PlannerTeamSettingInput,
  savePlannerTeams,
} from "../api/save-planner-teams";
import type { PlannerDepth } from "../types/depth";
import type { TacticOptions } from "../types/tactic";
import type { PlannerTeam } from "../types/team";
import { teamSettingInputs } from "../utils/team-settings";
import { PlannerClearAllControl } from "./planner-clear-all-control";
import { PlannerRoleReferenceModal } from "./planner-role-reference-modal";
import {
  PlannerSlotFitPicker,
  type PlannerSlotTarget,
} from "./planner-slot-fit-picker";
import { PlannerSquadBoard } from "./planner-squad-board";
import { PlannerStringEditor } from "./planner-string-editor";
import { PlannerTeamManagement } from "./planner-team-management";

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
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
  const [editedString, setEditedString] = useState<{
    team: PlannerTeam;
    stringId: number;
  } | null>(null);
  const [stringError, setStringError] = useState<string | null>(null);
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
    onSuccess: async (nextDepth) => {
      queryClient.setQueryData(plannerKeys.depth(), nextDepth);
      await queryClient.invalidateQueries({
        queryKey: plannerKeys.slotCandidates(),
      });
      setOptimizeError(null);
      setActionStatus("Squads optimized.");
    },
    onError: (error) => {
      setOptimizeError(`Optimization failed: ${errorMessage(error)}`);
    },
  });

  const saveStrings = useMutation({
    mutationKey: playerResultContextMutationKey,
    mutationFn: (request: {
      teams: PlannerTeamSettingInput[];
      confirmPopulatedRemoval: boolean;
      status: string;
    }) => savePlannerTeams(request.teams, request.confirmPopulatedRemoval),
    onSuccess: async (nextDepth, request) => {
      queryClient.setQueryData(plannerKeys.depth(), nextDepth);
      await queryClient.invalidateQueries({
        queryKey: plannerKeys.slotCandidates(),
      });
      setEditedString(null);
      setActionStatus(request.status);
    },
    onError: (error) => {
      setStringError(errorMessage(error));
    },
  });
  const controlsBusy =
    clearAll.isPending ||
    optimize.isPending ||
    teamManagementPending ||
    saveStrings.isPending;

  /** Saves one team's new string list, leaving every other team as it is. */
  const saveTeamStrings = (
    team: PlannerTeam,
    strings: PlannerStringSettingInput[],
    confirmPopulatedRemoval: boolean,
    status: string,
  ) => {
    if (controlsBusy) {
      return;
    }
    setStringError(null);
    setActionStatus(null);
    saveStrings.mutate({
      teams: teamSettingInputs(depth).map((input) =>
        input.team === team ? { ...input, strings } : input,
      ),
      confirmPopulatedRemoval,
      status,
    });
  };
  const editedTeam = depth.teams.find(
    (team) => team.team === editedString?.team,
  );
  const editedPlannerString = editedTeam?.strings.find(
    (plannerString) => plannerString.id === editedString?.stringId,
  );

  const requestClearAll = () => {
    if (clearAll.isPending || optimize.isPending || teamManagementPending) {
      return;
    }
    setClearAllError(null);
    setActionStatus(null);
    setClearAllOpen(true);
  };

  const runOptimization = () => {
    if (clearAll.isPending || optimize.isPending || teamManagementPending) {
      return;
    }
    setOptimizeError(null);
    setActionStatus(null);
    optimize.mutate();
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
                teamManagementPending
              }
              loading={optimize.isPending}
              loadingLabel="Optimizing…"
              onClick={runOptimization}
            >
              Optimize squads
            </Button>
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
        {stringError && !editedString ? (
          <p className="text-body-sm text-error" role="alert">
            {stringError}
          </p>
        ) : null}
        <PlannerSquadBoard
          teamDepths={orderedTeamDepths}
          tactic={tactic}
          options={options}
          stringsDisabled={controlsBusy}
          onOpen={openPicker}
          onAddString={(team) => {
            const strings =
              teamSettingInputs(depth).find((input) => input.team === team)
                ?.strings ?? [];
            // Rust names a blank new string after its place in the team.
            saveTeamStrings(
              team,
              [...strings, { id: null, displayName: "" }],
              false,
              "String added.",
            );
          }}
          onEditString={(team, plannerString) => {
            setStringError(null);
            setEditedString({ team, stringId: plannerString.id });
          }}
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
      {editedTeam && editedPlannerString ? (
        <PlannerStringEditor
          open
          team={editedTeam}
          plannerString={editedPlannerString}
          pending={saveStrings.isPending}
          error={stringError}
          onClose={() => setEditedString(null)}
          onSave={(strings, confirmPopulatedRemoval) =>
            saveTeamStrings(
              editedTeam.team,
              strings,
              confirmPopulatedRemoval,
              strings.length < editedTeam.strings.length
                ? "String removed."
                : "String saved.",
            )
          }
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
