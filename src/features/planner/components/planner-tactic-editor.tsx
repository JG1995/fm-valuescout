import { useMutation, useQueryClient } from "@tanstack/react-query";
import { type KeyboardEvent, useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { Panel } from "@/components/ui/panel/panel";
import { plannerKeys } from "../api/planner-keys";
import { savePlannerTactic } from "../api/save-planner-tactic";
import type { PlannerTactic, TacticLane, TacticOptions } from "../types/tactic";
import {
  cloneTactic,
  phasePosition,
  phaseRoleId,
  rolesForPhase,
  TACTIC_VIEWS,
  type TacticPhase,
  type TacticView,
  tacticEquals,
  updatePhaseLane,
  validateTacticDraft,
} from "../utils/tactic-editor";
import { PlannerTacticInspector } from "./planner-tactic-inspector";
import { PlannerTacticPitch } from "./planner-tactic-pitch";

type PlannerTacticEditorProps = {
  activeSaveRefreshError: boolean;
  isActiveSaveUnavailable: boolean;
  tactic: PlannerTactic;
  options: TacticOptions;
};

const VIEW_LABELS: Record<TacticView, string> = {
  ip: "IP",
  oop: "OOP",
  both: "Both",
};

function visiblePhases(view: TacticView): TacticPhase[] {
  return view === "both" ? ["ip", "oop"] : [view];
}

function nextView(view: TacticView, key: string): TacticView | null {
  const index = TACTIC_VIEWS.indexOf(view);
  if (index < 0) {
    return null;
  }
  if (key === "Home") {
    return TACTIC_VIEWS[0];
  }
  if (key === "End") {
    return TACTIC_VIEWS[TACTIC_VIEWS.length - 1];
  }
  if (key === "ArrowRight" || key === "ArrowDown") {
    return TACTIC_VIEWS[(index + 1) % TACTIC_VIEWS.length];
  }
  if (key === "ArrowLeft" || key === "ArrowUp") {
    return TACTIC_VIEWS[
      (index - 1 + TACTIC_VIEWS.length) % TACTIC_VIEWS.length
    ];
  }
  return null;
}

export function PlannerTacticEditor({
  activeSaveRefreshError,
  isActiveSaveUnavailable,
  tactic,
  options,
}: PlannerTacticEditorProps) {
  const queryClient = useQueryClient();
  const [draft, setDraft] = useState(() => cloneTactic(tactic));
  const [lastSavedTactic, setLastSavedTactic] = useState(() =>
    cloneTactic(tactic),
  );
  const [view, setView] = useState<TacticView>("both");
  const [selectedLaneId, setSelectedLaneId] = useState(
    tactic.lanes[0]?.laneId ?? "",
  );
  const [highlightedLaneId, setHighlightedLaneId] = useState<string | null>(
    null,
  );
  const [saveSucceeded, setSaveSucceeded] = useState(false);
  const viewButtonRefs = useRef<Record<TacticView, HTMLButtonElement | null>>({
    ip: null,
    oop: null,
    both: null,
  });

  useEffect(() => {
    if (
      tacticEquals(draft, lastSavedTactic) &&
      !tacticEquals(tactic, lastSavedTactic)
    ) {
      const nextTactic = cloneTactic(tactic);
      setDraft(nextTactic);
      setLastSavedTactic(nextTactic);
      if (!nextTactic.lanes.some((lane) => lane.laneId === selectedLaneId)) {
        setSelectedLaneId(nextTactic.lanes[0]?.laneId ?? "");
      }
    }
  }, [draft, lastSavedTactic, selectedLaneId, tactic]);

  const save = useMutation({
    mutationFn: () => savePlannerTactic(draft),
    onSuccess: async (savedTactic) => {
      const nextTactic = cloneTactic(savedTactic);
      setDraft(nextTactic);
      setLastSavedTactic(nextTactic);
      setSaveSucceeded(true);
      queryClient.setQueryData(plannerKeys.tactic(), nextTactic);
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: plannerKeys.depth() }),
        queryClient.invalidateQueries({
          queryKey: plannerKeys.slotCandidates(),
        }),
      ]);
    },
  });

  const validationError = validateTacticDraft(draft, options);
  const selectedLane = draft.lanes.find(
    (lane) => lane.laneId === selectedLaneId,
  );
  const updateDraft = (nextDraft: PlannerTactic) => {
    save.reset();
    setSaveSucceeded(false);
    setDraft(nextDraft);
  };

  const updateSelectedLaneWeight = (ipWeight: number) => {
    if (!selectedLane) {
      return;
    }
    updateDraft({
      ...draft,
      lanes: draft.lanes.map((lane) =>
        lane.laneId === selectedLane.laneId ? { ...lane, ipWeight } : lane,
      ),
    });
  };

  const updateSelectedLaneRank = (importanceRank: number | null) => {
    if (!selectedLane) {
      return;
    }
    updateDraft({
      ...draft,
      lanes: draft.lanes.map((lane) =>
        lane.laneId === selectedLane.laneId
          ? { ...lane, importanceRank }
          : lane,
      ),
    });
  };

  const updateSelectedLaneFoot = (
    preferredFoot: TacticLane["preferredFoot"],
  ) => {
    if (!selectedLane) {
      return;
    }
    updateDraft({
      ...draft,
      lanes: draft.lanes.map((lane) =>
        lane.laneId === selectedLane.laneId
          ? {
              ...lane,
              preferredFoot,
              footPreference:
                preferredFoot === "any" ? "preferred" : lane.footPreference,
            }
          : lane,
      ),
    });
  };

  const updateSelectedLaneFootPreference = (
    footPreference: TacticLane["footPreference"],
  ) => {
    if (!selectedLane) {
      return;
    }
    updateDraft({
      ...draft,
      lanes: draft.lanes.map((lane) =>
        lane.laneId === selectedLane.laneId
          ? { ...lane, footPreference }
          : lane,
      ),
    });
  };

  const updatePosition = (
    laneId: string,
    phase: TacticPhase,
    position: string,
  ) => {
    const currentLane = draft.lanes.find((lane) => lane.laneId === laneId);
    if (!currentLane) {
      return;
    }
    const currentRoleId = phaseRoleId(currentLane, phase);
    const keepsCurrentRole = rolesForPhase(options, phase, position).some(
      (role) => role.roleId === currentRoleId,
    );
    updateDraft({
      ...draft,
      lanes: draft.lanes.map((lane) =>
        lane.laneId === laneId
          ? updatePhaseLane(
              lane,
              phase,
              position,
              keepsCurrentRole ? currentRoleId : "",
            )
          : lane,
      ),
    });
  };

  const updateRole = (laneId: string, phase: TacticPhase, roleId: string) => {
    const currentLane = draft.lanes.find((lane) => lane.laneId === laneId);
    if (!currentLane) {
      return;
    }
    updateDraft({
      ...draft,
      lanes: draft.lanes.map((lane) =>
        lane.laneId === laneId
          ? updatePhaseLane(
              lane,
              phase,
              phasePosition(currentLane, phase),
              roleId,
            )
          : lane,
      ),
    });
  };

  const handleViewKeyDown = (event: KeyboardEvent<HTMLFieldSetElement>) => {
    const next = nextView(view, event.key);
    if (!next) {
      return;
    }
    event.preventDefault();
    setView(next);
    viewButtonRefs.current[next]?.focus();
  };

  return (
    <Panel
      title="Tactic editor"
      flush
      actions={
        <Button
          disabled={Boolean(validationError) || isActiveSaveUnavailable}
          loading={save.isPending}
          loadingLabel="Saving…"
          onClick={() => save.mutate()}
        >
          Save tactic
        </Button>
      }
    >
      <div className="space-y-5 p-4">
        <div className="space-y-1">
          <p className="text-body-md text-on-surface-variant">
            {draft.lanes.length} linked positions
          </p>
          <p className="text-body-sm text-on-surface-variant">
            Each linked position connects the In-Possession and
            Out-of-Possession shapes.
          </p>
        </div>

        <fieldset
          className="inline-flex rounded-full bg-surface-container-high p-0.5"
          onKeyDown={handleViewKeyDown}
        >
          <legend className="sr-only">Tactic phase views</legend>
          {TACTIC_VIEWS.map((candidate) => {
            const selected = candidate === view;
            return (
              <button
                key={candidate}
                ref={(element) => {
                  viewButtonRefs.current[candidate] = element;
                }}
                type="button"
                aria-pressed={selected}
                tabIndex={selected ? 0 : -1}
                className={`cursor-pointer rounded-full px-4 py-1.5 text-label-lg transition-colors duration-150 ease-out focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary ${
                  selected
                    ? "bg-primary text-on-primary"
                    : "text-on-surface-variant hover:text-on-surface"
                }`}
                onClick={() => setView(candidate)}
              >
                {VIEW_LABELS[candidate]}
              </button>
            );
          })}
        </fieldset>

        {validationError ? (
          <p className="text-body-sm text-warning" role="alert">
            {validationError}
          </p>
        ) : null}
        {isActiveSaveUnavailable && !activeSaveRefreshError ? (
          <p className="text-body-sm text-on-surface-variant" role="status">
            Refreshing active save…
          </p>
        ) : null}
        {activeSaveRefreshError ? (
          <p className="text-body-sm text-error" role="alert">
            Could not refresh the active save. Saving is disabled until it
            reloads.
          </p>
        ) : null}
        {save.isError ? (
          <p className="text-body-sm text-error" role="alert">
            {save.error.message}
          </p>
        ) : null}
        {saveSucceeded ? (
          <p className="text-body-sm text-success" role="status">
            Tactic saved.
          </p>
        ) : null}

        <div className="grid items-start gap-6 xl:grid-cols-[minmax(0,1fr)_minmax(18rem,22rem)]">
          <div className="grid gap-4 lg:grid-cols-2">
            {visiblePhases(view).map((phase) => (
              <PlannerTacticPitch
                key={phase}
                phase={phase}
                lanes={draft.lanes}
                options={options}
                selectedLaneId={selectedLaneId}
                highlightedLaneId={highlightedLaneId}
                onHighlight={setHighlightedLaneId}
                onSelectLane={(laneId) => {
                  setSelectedLaneId(laneId);
                  setHighlightedLaneId(laneId);
                }}
              />
            ))}
          </div>
          {selectedLane ? (
            <PlannerTacticInspector
              selectedLane={selectedLane}
              lanes={draft.lanes}
              options={options}
              phases={visiblePhases(view)}
              onWeightChange={updateSelectedLaneWeight}
              onRankChange={updateSelectedLaneRank}
              onPreferredFootChange={updateSelectedLaneFoot}
              onFootPreferenceChange={updateSelectedLaneFootPreference}
              onPositionChange={(phase, position) =>
                updatePosition(selectedLane.laneId, phase, position)
              }
              onRoleChange={(phase, roleId) =>
                updateRole(selectedLane.laneId, phase, roleId)
              }
            />
          ) : null}
        </div>
      </div>
    </Panel>
  );
}
