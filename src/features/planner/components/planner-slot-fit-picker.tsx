import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useId, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import {
  fieldClasses,
  fieldLabelClasses,
} from "@/components/ui/field/field-styles";
import { Modal } from "@/components/ui/modal/modal";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { assignPlannerPlayer } from "../api/assign-planner-player";
import { clearPlannerAssignment } from "../api/clear-planner-assignment";
import { movePlannerPlayer } from "../api/move-planner-player";
import { plannerKeys } from "../api/planner-keys";
import { plannerSlotCandidatesQueryOptions } from "../api/planner-slot-candidates-query-options";
import type { PlannerTeam } from "../types/club-family";
import type { PlannerSlotCandidate } from "../types/depth";
import type { PlannerTactic, TacticOptions } from "../types/tactic";
import { linkedPositionDescriptionForId } from "../utils/tactic-editor";

const SEARCH_DEBOUNCE_MS = 200;

const TEAM_LABELS: Record<PlannerTeam, string> = {
  senior: "Senior",
  reserves: "Reserves",
  youth: "Youth",
};

export type PlannerSlotTarget = {
  team: PlannerTeam;
  stringId: number;
  stringOrder: number;
  laneId: string;
  laneName: string;
  occupantName: string | null;
};

type PlannerSlotFitPickerProps = {
  activeSaveId: number;
  open: boolean;
  target: PlannerSlotTarget;
  tactic: PlannerTactic;
  options: TacticOptions;
  onClose: () => void;
  onMutationError: (message: string) => void;
};

function scoreEvidence(score: number | null) {
  return score === null ? "—" : score;
}

function ordinal(value: number) {
  const number = value + 1;
  const suffix =
    number % 100 >= 11 && number % 100 <= 13
      ? "th"
      : number % 10 === 1
        ? "st"
        : number % 10 === 2
          ? "nd"
          : number % 10 === 3
            ? "rd"
            : "th";
  return `${number}${suffix} string`;
}

function assignmentLocation(
  candidate: PlannerSlotCandidate,
  tactic: PlannerTactic,
  options: TacticOptions,
) {
  const location = candidate.assignmentLocation;
  if (!location) {
    return "Unassigned";
  }
  return `Assigned: ${slotLocation(
    location.team,
    location.stringOrder,
    linkedPositionDescriptionForId(location.laneId, tactic.lanes, options),
  )}`;
}

function slotLocation(
  team: PlannerTeam,
  stringOrder: number,
  laneName: string,
) {
  return `${TEAM_LABELS[team]} · ${ordinal(stringOrder)} · ${laneName}`;
}

function targetLocation(target: PlannerSlotTarget) {
  return slotLocation(target.team, target.stringOrder, target.laneName);
}

function moveConfirmation(
  candidate: PlannerSlotCandidate,
  target: PlannerSlotTarget,
  tactic: PlannerTactic,
  options: TacticOptions,
) {
  const location = candidate.assignmentLocation;
  if (!location) {
    return null;
  }
  return `Move ${candidate.name} from ${slotLocation(
    location.team,
    location.stringOrder,
    linkedPositionDescriptionForId(location.laneId, tactic.lanes, options),
  )} to ${targetLocation(target)}?`;
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function PlannerSlotFitPicker({
  activeSaveId,
  open,
  target,
  tactic,
  options,
  onClose,
  onMutationError,
}: PlannerSlotFitPickerProps) {
  const queryClient = useQueryClient();
  const inputRef = useRef<HTMLInputElement>(null);
  const activeOptionRef = useRef<HTMLButtonElement>(null);
  const returningToSearchRef = useRef(false);
  const searchInputId = useId();
  const listboxId = useId();
  const optionIdPrefix = useId();
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const [moveCandidate, setMoveCandidate] =
    useState<PlannerSlotCandidate | null>(null);
  const isOccupied = target.occupantName !== null;

  useEffect(() => {
    setActiveIndex(0);
    const timer = window.setTimeout(() => {
      setDebouncedSearch(search.trim());
    }, SEARCH_DEBOUNCE_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [search]);

  useEffect(() => {
    if (moveCandidate) {
      returningToSearchRef.current = true;
    } else if (returningToSearchRef.current) {
      inputRef.current?.focus();
      returningToSearchRef.current = false;
    }
  }, [moveCandidate]);

  const candidatesQuery = useQuery({
    ...plannerSlotCandidatesQueryOptions(activeSaveId, {
      team: target.team,
      laneId: target.laneId,
      search: debouncedSearch,
    }),
    enabled: !isOccupied,
  });
  const candidates = isOccupied ? [] : (candidatesQuery.data ?? []);

  const closeAfterSuccess = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: plannerKeys.depth() }),
      queryClient.invalidateQueries({ queryKey: plannerKeys.slotCandidates() }),
    ]);
    onClose();
  };
  const closeAfterError = (error: unknown) => {
    onMutationError(errorMessage(error));
    onClose();
  };
  const assign = useMutation({
    mutationFn: (candidate: PlannerSlotCandidate) =>
      assignPlannerPlayer(target.stringId, target.laneId, candidate.playerUid),
    onSuccess: closeAfterSuccess,
    onError: closeAfterError,
  });
  const move = useMutation({
    mutationFn: (candidate: PlannerSlotCandidate) =>
      movePlannerPlayer(target.stringId, target.laneId, candidate.playerUid),
    onSuccess: closeAfterSuccess,
    onError: closeAfterError,
  });
  const clear = useMutation({
    mutationFn: () => clearPlannerAssignment(target.stringId, target.laneId),
    onSuccess: closeAfterSuccess,
    onError: closeAfterError,
  });
  const isMutating = assign.isPending || move.isPending || clear.isPending;

  const selectCandidate = (candidate: PlannerSlotCandidate) => {
    if (candidate.assignmentLocation) {
      setMoveCandidate(candidate);
      return;
    }
    assign.mutate(candidate);
  };

  const activeCandidate = candidates[activeIndex];
  const isMoveConfirmation = moveCandidate !== null;
  const showCandidates = candidates.length > 0;

  useEffect(() => {
    if (!activeCandidate) {
      return;
    }
    activeOptionRef.current?.scrollIntoView?.({ block: "nearest" });
  }, [activeCandidate]);

  return (
    <Modal
      open={open}
      title={
        isOccupied
          ? `Clear ${target.occupantName}?`
          : isMoveConfirmation
            ? `Move ${moveCandidate.name}?`
            : `Find a player for ${target.laneName}`
      }
      onClose={onClose}
      footer={
        isOccupied ? (
          <>
            <Button disabled={isMutating} variant="secondary" onClick={onClose}>
              Cancel
            </Button>
            <Button disabled={isMutating} onClick={() => clear.mutate()}>
              {isMutating ? "Clearing…" : "Clear slot"}
            </Button>
          </>
        ) : isMoveConfirmation ? (
          <>
            <Button
              disabled={isMutating}
              variant="secondary"
              onClick={() => setMoveCandidate(null)}
            >
              Back
            </Button>
            <Button
              autoFocus
              disabled={isMutating}
              onClick={() => {
                if (moveCandidate) {
                  move.mutate(moveCandidate);
                }
              }}
            >
              {isMutating ? "Moving…" : "Confirm move"}
            </Button>
          </>
        ) : (
          <Button disabled={isMutating} variant="secondary" onClick={onClose}>
            Cancel
          </Button>
        )
      }
    >
      {isOccupied ? (
        <p className="text-body-md text-on-surface-variant">
          {target.occupantName} is assigned to {targetLocation(target)}. It must
          be cleared before assigning or moving a player.
        </p>
      ) : isMoveConfirmation ? (
        moveCandidate ? (
          <p className="text-body-md text-on-surface-variant">
            {moveConfirmation(moveCandidate, target, tactic, options)}
          </p>
        ) : null
      ) : (
        <div className="space-y-3">
          <div className="space-y-1">
            <label className={fieldLabelClasses} htmlFor={searchInputId}>
              Search squad candidates
            </label>
            <input
              ref={inputRef}
              aria-activedescendant={
                showCandidates && activeCandidate
                  ? `${optionIdPrefix}-${activeCandidate.playerUid}`
                  : undefined
              }
              aria-autocomplete="list"
              aria-controls={showCandidates ? listboxId : undefined}
              aria-expanded={showCandidates}
              aria-haspopup="listbox"
              className={`${fieldClasses} w-full`}
              id={searchInputId}
              role="combobox"
              type="text"
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              onKeyDown={(event) => {
                if (candidates.length === 0) {
                  return;
                }
                if (event.key === "ArrowDown") {
                  event.preventDefault();
                  setActiveIndex((index) => (index + 1) % candidates.length);
                } else if (event.key === "ArrowUp") {
                  event.preventDefault();
                  setActiveIndex(
                    (index) =>
                      (index - 1 + candidates.length) % candidates.length,
                  );
                } else if (event.key === "Enter" && activeCandidate) {
                  event.preventDefault();
                  selectCandidate(activeCandidate);
                }
              }}
            />
          </div>
          {candidatesQuery.isError ? (
            <p className="text-body-sm text-error" role="alert">
              {errorMessage(candidatesQuery.error)}
            </p>
          ) : null}
          {candidatesQuery.isPending ? (
            <p className="text-body-sm text-on-surface-variant">
              Finding candidates…
            </p>
          ) : null}
          {!candidatesQuery.isPending && candidates.length === 0 ? (
            <p className="text-body-sm text-on-surface-variant">
              No configured club-family players match this search.
            </p>
          ) : null}
          {candidates.length > 0 ? (
            <div
              aria-label="Slot-fit candidates"
              className="max-h-80 overflow-y-auto rounded-md border border-outline-variant"
              id={listboxId}
              role="listbox"
            >
              {candidates.map((candidate, index) => (
                <button
                  key={candidate.playerUid}
                  ref={
                    candidate.playerUid === activeCandidate?.playerUid
                      ? activeOptionRef
                      : undefined
                  }
                  aria-selected={index === activeIndex}
                  className={`flex w-full items-center justify-between gap-3 border-b border-outline-variant px-3 py-2 text-left text-body-sm text-on-surface last:border-b-0 hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-primary ${
                    index === activeIndex ? "bg-surface-container-high" : ""
                  }`}
                  disabled={isMutating}
                  id={`${optionIdPrefix}-${candidate.playerUid}`}
                  role="option"
                  type="button"
                  onMouseDown={(event) => event.preventDefault()}
                  onMouseEnter={() => setActiveIndex(index)}
                  onClick={() => selectCandidate(candidate)}
                >
                  <span className="min-w-0">
                    <span className="block truncate">{candidate.name}</span>
                    <span className="block text-body-sm text-on-surface-variant">
                      {candidate.currentClub} ·{" "}
                      {assignmentLocation(candidate, tactic, options)}
                    </span>
                    <span className="block font-mono text-mono-sm text-on-surface-variant">
                      IP {scoreEvidence(candidate.ipScore)} · OOP{" "}
                      {scoreEvidence(candidate.oopScore)}
                    </span>
                  </span>
                  {candidate.combinedScore === null ? (
                    <span className="font-mono text-mono-sm text-on-surface-variant">
                      —
                    </span>
                  ) : (
                    <ScoreBadge
                      roleName="Combined role score"
                      score={candidate.combinedScore}
                    />
                  )}
                </button>
              ))}
            </div>
          ) : null}
        </div>
      )}
    </Modal>
  );
}
